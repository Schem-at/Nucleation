//! Animated GLB from a build animation: one node per group, TRS keyframe tracks
//! sampled from the timeline, anchor child nodes, and `extras.nucleation` for
//! the pose-colour and camera tracks core glTF cannot carry.
//!
//! Node convention (a public contract, see docs/features/animation.md):
//! root `build:<name>` → children `group:<id>` → children `anchor:<name>`.
use super::pose::decompose_trs;
use super::{BuildAnimation, CameraPose, Frame, GroupId};
use crate::meshing::{MeshConfig, MeshError, ResourcePackSource, Result};
use schematic_mesher::{
    export_build_glb, mesh_from_layers, BuildChild, BuildNode, BuildScene, Interpolation, Track,
};
use serde_json::json;

const EPSILON: f32 = 1e-5;

fn close<const N: usize>(a: &[f32; N], b: &[f32; N]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() <= EPSILON)
}

/// Keep the first and last key of every constant run, plus every change.
fn dedupe<const N: usize>(times: &[f32], values: &[[f32; N]]) -> (Vec<f32>, Vec<[f32; N]>) {
    let mut out_t = Vec::new();
    let mut out_v: Vec<[f32; N]> = Vec::new();
    for (i, value) in values.iter().enumerate() {
        let next_same = values.get(i + 1).is_some_and(|next| close(next, value));
        let prev_same = out_v.last().is_some_and(|last| close(last, value));
        if prev_same && next_same {
            continue;
        }
        out_t.push(times[i]);
        out_v.push(*value);
    }
    (out_t, out_v)
}

/// A track, or `None` when every sample equals the rest value.
fn track<const N: usize>(times: &[f32], values: &[[f32; N]], rest: [f32; N]) -> Option<Track<N>> {
    if values.iter().all(|value| close(value, &rest)) {
        return None;
    }
    let (times, values) = dedupe(times, values);
    Some(Track {
        times,
        values,
        interpolation: Interpolation::Linear,
    })
}

fn sample_times(duration_ms: f32, fps: f32) -> Vec<f32> {
    let count = ((duration_ms / 1000.0) * fps).ceil().max(1.0) as usize + 1;
    (0..count)
        .map(|i| ((i as f32) / fps * 1000.0).min(duration_ms))
        .collect()
}

fn camera_extras(frames: &[Frame], times_s: &[f32]) -> serde_json::Value {
    if frames.iter().all(|frame| frame.camera.is_none()) {
        return serde_json::Value::Null;
    }
    let pick = |f: fn(&CameraPose) -> f32, rest: f32| -> Vec<f32> {
        frames
            .iter()
            .map(|frame| frame.camera.as_ref().map_or(rest, f))
            .collect()
    };
    json!({
        "times": times_s,
        "yaw": pick(|c| c.yaw, 0.0),
        "pitch": pick(|c| c.pitch, 0.0),
        "zoom": pick(|c| c.zoom, 1.0),
        "targetOffset": frames
            .iter()
            .map(|frame| frame.camera.as_ref().map_or([0.0; 3], |c| c.target_offset))
            .collect::<Vec<_>>(),
    })
}

impl BuildAnimation {
    /// The build as an animated GLB: one textured node per group, TRS tracks
    /// sampled at `fps`, anchors as child nodes, `extras.nucleation` for
    /// opacity/tint/emissive and the camera track.
    pub fn to_animated_glb(
        &self,
        pack: &ResourcePackSource,
        config: &MeshConfig,
        fps: f32,
    ) -> Result<Vec<u8>> {
        let fps = if fps.is_finite() && fps > 0.0 {
            fps
        } else {
            30.0
        };
        let duration = self.duration_ms();
        let times_ms = sample_times(duration, fps);
        let times_s: Vec<f32> = times_ms.iter().map(|t| t / 1000.0).collect();
        let frames: Vec<Frame> = times_ms.iter().map(|t| self.frame_at(*t)).collect();

        let outputs = self.mesh_outputs_raw(pack, config)?;
        let block_counts = self.group_block_counts();
        let mut atlases = Vec::with_capacity(outputs.len());
        let mut nodes = Vec::with_capacity(outputs.len());
        for (id, output) in outputs.into_iter().enumerate() {
            let mesh = mesh_from_layers(&[
                &output.opaque_mesh,
                &output.cutout_mesh,
                &output.transparent_mesh,
            ]);
            atlases.push(output.atlas);

            let mut translation = Vec::with_capacity(frames.len());
            let mut rotation = Vec::with_capacity(frames.len());
            let mut scale = Vec::with_capacity(frames.len());
            let mut opacity = Vec::with_capacity(frames.len());
            let mut tint = Vec::with_capacity(frames.len());
            let mut emissive = Vec::with_capacity(frames.len());
            for frame in &frames {
                let pose = frame.pose(id as GroupId);
                let matrix = pose
                    .and_then(|pose| pose.matrix)
                    .unwrap_or_else(super::operation::identity);
                let (t, q, s) = decompose_trs(matrix);
                translation.push(t);
                rotation.push(q);
                scale.push(s);
                opacity.push([pose.map_or(1.0, |pose| pose.opacity)]);
                tint.push(pose.map_or([1.0; 4], |pose| pose.tint));
                emissive.push(pose.map_or([0.0; 4], |pose| pose.emissive));
            }
            let pose_track = if opacity.iter().all(|o| close(o, &[1.0]))
                && tint.iter().all(|t| close(t, &[1.0; 4]))
                && emissive.iter().all(|e| close(e, &[0.0; 4]))
            {
                serde_json::Value::Null
            } else {
                json!({
                    "times": times_s,
                    "opacity": opacity.iter().map(|o| o[0]).collect::<Vec<_>>(),
                    "tint": tint,
                    "emissive": emissive,
                })
            };
            let children = self
                .anchors()
                .iter()
                .filter(|anchor| anchor.group as usize == id)
                .map(|anchor| BuildChild {
                    name: format!("anchor:{}", anchor.name),
                    translation: anchor.local,
                    extras: Some(json!({ "nucleation": {
                        "anchor": anchor.name,
                        "group": anchor.group,
                    }})),
                })
                .collect();
            nodes.push(BuildNode {
                name: format!("group:{id}"),
                mesh,
                atlas: id,
                translation: track(&times_s, &translation, [0.0; 3]),
                rotation: track(&times_s, &rotation, [0.0, 0.0, 0.0, 1.0]),
                scale: track(&times_s, &scale, [1.0; 3]),
                extras: Some(json!({ "nucleation": {
                    "group": id,
                    "blocks": block_counts[id],
                    "poseTrack": pose_track,
                }})),
                children,
            });
        }

        let name = self
            .schematic()
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "build".to_string());
        let scene = BuildScene {
            name,
            atlases,
            nodes,
            extras: Some(json!({ "nucleation": {
                "version": 1,
                "durationMs": duration,
                "fps": fps,
                "groups": self.groups().len(),
                "camera": camera_extras(&frames, &times_s),
            }})),
        };
        export_build_glb(&scene).map_err(|e| MeshError::Meshing(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedupe_keeps_run_endpoints_and_changes() {
        let times = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let values = [[0.0], [0.0], [0.0], [1.0], [2.0], [2.0]];
        let (t, v) = dedupe(&times, &values);
        assert_eq!(t, vec![0.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(v, vec![[0.0], [0.0], [1.0], [2.0], [2.0]]);
    }

    #[test]
    fn constant_tracks_are_dropped() {
        assert!(track(&[0.0, 1.0], &[[1.0; 3], [1.0; 3]], [1.0; 3]).is_none());
        assert!(track(&[0.0, 1.0], &[[0.0; 3], [1.0; 3]], [1.0; 3]).is_some());
    }

    #[test]
    fn sample_times_end_exactly_at_the_duration() {
        let times = sample_times(2400.0, 30.0);
        assert_eq!(times.len(), 73);
        assert_eq!(times[0], 0.0);
        assert_eq!(*times.last().unwrap(), 2400.0);
        assert_eq!(sample_times(0.0, 30.0), vec![0.0, 0.0]);
    }
}
