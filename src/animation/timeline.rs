//! Timeline composition and frame sampling.
//!
//! A [`Timeline`] holds clips bound to targets at offsets, and [`Timeline::seek`]
//! evaluates them all at one instant. `seek` is pure — no interior mutation, no
//! wall-clock — so the same time always yields the same [`Frame`]. That is what
//! makes regenerated media byte-identical.

use serde::{Deserialize, Serialize};

use super::pose::Pose;
use super::stagger::{Group, GroupId, Stagger};
use super::track::Clip;

/// Camera state at an instant, in the same terms as
/// [`crate::rendering::RenderConfig`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraPose {
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    /// World-space orbit target offset.
    pub target_offset: [f32; 3],
}

impl Default for CameraPose {
    fn default() -> Self {
        CameraPose {
            yaw: 0.0,
            pitch: 0.0,
            zoom: 1.0,
            target_offset: [0.0; 3],
        }
    }
}

impl CameraPose {
    /// Reinterpret a sampled [`Pose`] as camera state.
    ///
    /// Clips are authored against `Pose`, so a camera clip reuses the same
    /// machinery under a documented mapping: `RotY -> yaw`, `RotX -> pitch`,
    /// `ScaleUniform`/`ScaleX -> zoom`, `X/Y/Z -> target offset`.
    pub fn from_pose(p: &Pose) -> Self {
        CameraPose {
            yaw: p.rotate_deg[1],
            pitch: p.rotate_deg[0],
            zoom: p.scale[0],
            target_offset: p.translate,
        }
    }
}

/// What a clip drives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Target {
    Group(GroupId),
    Groups(Vec<GroupId>),
    All,
    Camera,
}

impl Target {
    fn matches(&self, id: GroupId) -> bool {
        match self {
            Target::Group(g) => *g == id,
            Target::Groups(gs) => gs.contains(&id),
            Target::All => true,
            Target::Camera => false,
        }
    }
}

/// Everything needed to draw one instant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub time_ms: f32,
    /// Sorted by [`GroupId`], so serialisation is stable.
    pub poses: Vec<(GroupId, Pose)>,
    pub camera: Option<CameraPose>,
}

impl Frame {
    /// Look up one group's pose.
    pub fn pose(&self, id: GroupId) -> Option<&Pose> {
        self.poses
            .binary_search_by_key(&id, |(g, _)| *g)
            .ok()
            .map(|i| &self.poses[i].1)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Entry {
    clip: Clip,
    target: Target,
    offset_ms: f32,
}

/// Clips bound to targets over a shared clock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    groups: Vec<Group>,
    entries: Vec<Entry>,
}

impl Timeline {
    pub fn new(groups: Vec<Group>) -> Self {
        Timeline {
            groups,
            entries: Vec::new(),
        }
    }

    pub fn groups(&self) -> &[Group] {
        &self.groups
    }

    /// Bind a clip to a target, starting at `offset_ms`.
    pub fn add(&mut self, clip: Clip, target: Target, offset_ms: f32) -> &mut Self {
        self.entries.push(Entry {
            clip,
            target,
            offset_ms,
        });
        self
    }

    /// Bind one clip per group, delayed according to `stagger`.
    ///
    /// This is the workhorse: an assembly, a layer print, or blocks flying in
    /// along a curve are all this call with a different [`super::stagger::Order`].
    pub fn add_staggered(&mut self, clip: Clip, stagger: &Stagger, offset_ms: f32) -> &mut Self {
        let delays = stagger.delays(&self.groups);
        for (g, delay) in self.groups.iter().zip(delays) {
            self.entries.push(Entry {
                clip: clip.clone(),
                target: Target::Group(g.id),
                offset_ms: offset_ms + delay,
            });
        }
        self
    }

    /// Longest finite end time. Endlessly repeating clips do not extend it —
    /// otherwise a single looping camera would make the duration infinite.
    pub fn duration_ms(&self) -> f32 {
        self.entries
            .iter()
            .filter_map(|e| e.clip.total_ms().map(|d| e.offset_ms + d))
            .fold(0.0f32, f32::max)
    }

    /// Evaluate every clip at `t_ms`.
    pub fn seek(&self, t_ms: f32) -> Frame {
        // Each group starts at identity, pivoting about its own centroid so
        // rotation and scale happen in place.
        let mut poses: Vec<(GroupId, Pose)> = self
            .groups
            .iter()
            .map(|g| (g.id, Pose::about(g.centroid)))
            .collect();
        poses.sort_by_key(|(id, _)| *id);

        let mut camera: Option<CameraPose> = None;

        for entry in &self.entries {
            let local = t_ms - entry.offset_ms;
            if let Target::Camera = entry.target {
                let mut p = Pose::IDENTITY;
                entry.clip.sample_into(local, &mut p);
                camera = Some(CameraPose::from_pose(&p));
                continue;
            }
            match &entry.target {
                Target::Group(id) => {
                    if let Ok(i) = poses.binary_search_by_key(id, |(g, _)| *g) {
                        entry.clip.sample_into(local, &mut poses[i].1);
                    }
                }
                _ => {
                    for (id, pose) in poses.iter_mut() {
                        if entry.target.matches(*id) {
                            entry.clip.sample_into(local, pose);
                        }
                    }
                }
            }
        }

        Frame {
            time_ms: t_ms,
            poses,
            camera,
        }
    }

    /// Deterministic frame times for a capture at `fps`.
    ///
    /// Times are computed in `f64` as `i * 1000 / fps` so they do not drift the
    /// way repeated accumulation would.
    pub fn frame_times(&self, fps: f64) -> Vec<f32> {
        let fps = if fps <= 0.0 { 1.0 } else { fps };
        let dur = self.duration_ms() as f64;
        let count = ((dur / 1000.0) * fps).round().max(1.0) as usize;
        (0..count)
            .map(|i| (i as f64 * 1000.0 / fps) as f32)
            .collect()
    }

    /// Sample every frame of a capture at `fps`.
    pub fn frames(&self, fps: f64) -> Vec<Frame> {
        self.frame_times(fps)
            .into_iter()
            .map(|t| self.seek(t))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::easing::Easing;
    use crate::animation::stagger::{Axis, Grouping, Order};
    use crate::animation::track::{Property, Track};

    fn groups(n: i32) -> Vec<Group> {
        Grouping::PerBlock.apply(&(0..n).map(|i| (i, 0, 0)).collect::<Vec<_>>())
    }

    fn slide() -> Clip {
        Clip::new(100.0).track(Track::tween(Property::Y, 10.0, 0.0, Easing::Linear))
    }

    #[test]
    fn groups_default_to_identity_pivoted_at_their_centroid() {
        let tl = Timeline::new(groups(2));
        let f = tl.seek(0.0);
        assert_eq!(f.poses.len(), 2);
        assert_eq!(f.pose(0).unwrap().pivot, [0.5, 0.5, 0.5]);
        assert_eq!(f.pose(1).unwrap().pivot, [1.5, 0.5, 0.5]);
        assert_eq!(f.pose(0).unwrap().scale, [1.0; 3]);
    }

    #[test]
    fn poses_are_sorted_by_group_id() {
        let tl = Timeline::new(groups(5));
        let f = tl.seek(0.0);
        let ids: Vec<GroupId> = f.poses.iter().map(|(g, _)| *g).collect();
        assert_eq!(ids, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn target_all_drives_every_group() {
        let mut tl = Timeline::new(groups(3));
        tl.add(slide(), Target::All, 0.0);
        let f = tl.seek(100.0);
        for id in 0..3 {
            assert!((f.pose(id).unwrap().translate[1]).abs() < 1e-4);
        }
    }

    #[test]
    fn target_group_drives_only_that_group() {
        let mut tl = Timeline::new(groups(3));
        tl.add(slide(), Target::Group(1), 0.0);
        let f = tl.seek(100.0);
        assert_eq!(f.pose(0).unwrap().translate[1], 0.0, "untouched");
        assert!((f.pose(1).unwrap().translate[1]).abs() < 1e-4, "animated");
    }

    #[test]
    fn offset_shifts_a_clip_in_time() {
        let mut tl = Timeline::new(groups(1));
        tl.add(slide(), Target::All, 500.0);
        assert_eq!(tl.seek(0.0).pose(0).unwrap().translate[1], 10.0);
        assert!((tl.seek(600.0).pose(0).unwrap().translate[1]).abs() < 1e-4);
    }

    #[test]
    fn staggered_entries_start_at_their_delays() {
        let mut tl = Timeline::new(groups(3));
        let st = Stagger::each(Order::Axis(Axis::X, true), 100.0);
        tl.add_staggered(slide(), &st, 0.0);
        // At t=100 the first has finished, the second is starting, the third waits.
        let f = tl.seek(100.0);
        assert!((f.pose(0).unwrap().translate[1]).abs() < 1e-4, "done");
        assert!(
            (f.pose(1).unwrap().translate[1] - 10.0).abs() < 1e-4,
            "starting"
        );
        assert!(
            (f.pose(2).unwrap().translate[1] - 10.0).abs() < 1e-4,
            "waiting"
        );
    }

    #[test]
    fn duration_covers_staggered_delays() {
        let mut tl = Timeline::new(groups(4));
        tl.add_staggered(slide(), &Stagger::each(Order::Index, 100.0), 0.0);
        // last delay 300 + clip 100
        assert!((tl.duration_ms() - 400.0).abs() < 1e-3);
    }

    #[test]
    fn forever_clips_do_not_make_duration_infinite() {
        use crate::animation::track::Repeat;
        let mut tl = Timeline::new(groups(1));
        tl.add(slide().repeat(Repeat::Forever), Target::Camera, 0.0);
        tl.add(slide(), Target::All, 0.0);
        assert!(tl.duration_ms().is_finite());
        assert!((tl.duration_ms() - 100.0).abs() < 1e-3);
    }

    #[test]
    fn camera_target_populates_camera_not_poses() {
        let mut tl = Timeline::new(groups(1));
        tl.add(
            Clip::new(1000.0).track(Track::tween(Property::RotY, 0.0, 360.0, Easing::Linear)),
            Target::Camera,
            0.0,
        );
        let f = tl.seek(500.0);
        let cam = f.camera.expect("camera should be animated");
        assert!((cam.yaw - 180.0).abs() < 1e-3);
        // Groups untouched by a camera clip.
        assert_eq!(f.pose(0).unwrap().rotate_deg[1], 0.0);
    }

    #[test]
    fn no_camera_clip_leaves_camera_none() {
        let tl = Timeline::new(groups(1));
        assert!(tl.seek(0.0).camera.is_none());
    }

    #[test]
    fn later_entries_layer_over_earlier_ones() {
        let mut tl = Timeline::new(groups(1));
        tl.add(
            Clip::new(100.0).track(Track::tween(Property::Y, 0.0, 5.0, Easing::Linear)),
            Target::All,
            0.0,
        );
        tl.add(
            Clip::new(100.0).track(Track::tween(Property::X, 0.0, 7.0, Easing::Linear)),
            Target::All,
            0.0,
        );
        let p = tl.seek(100.0);
        let pose = p.pose(0).unwrap();
        assert!((pose.translate[1] - 5.0).abs() < 1e-4, "first clip kept");
        assert!(
            (pose.translate[0] - 7.0).abs() < 1e-4,
            "second clip applied"
        );
    }

    #[test]
    fn seek_is_pure_and_repeatable() {
        let mut tl = Timeline::new(groups(6));
        tl.add_staggered(
            slide(),
            &Stagger::total(Order::Random(11), 500.0).eased(Easing::out_back()),
            0.0,
        );
        for t in [0.0f32, 37.5, 120.0, 480.0, 900.0] {
            assert_eq!(tl.seek(t), tl.seek(t), "seek must be pure at t={t}");
        }
        // Out-of-order sampling must not change results.
        let a = tl.seek(200.0);
        let _ = tl.seek(900.0);
        assert_eq!(tl.seek(200.0), a);
    }

    #[test]
    fn frame_times_are_evenly_spaced_and_drift_free() {
        let mut tl = Timeline::new(groups(1));
        tl.add(Clip::new(1000.0), Target::All, 0.0);
        let ts = tl.frame_times(30.0);
        assert_eq!(ts.len(), 30);
        assert_eq!(ts[0], 0.0);
        // Exact multiples, not accumulated sums.
        for (i, t) in ts.iter().enumerate() {
            let expect = (i as f64 * 1000.0 / 30.0) as f32;
            assert_eq!(*t, expect);
        }
    }

    #[test]
    fn degenerate_fps_and_empty_timeline_are_safe() {
        let tl = Timeline::new(groups(1));
        assert_eq!(tl.duration_ms(), 0.0);
        assert_eq!(tl.frame_times(0.0).len(), 1);
        assert_eq!(tl.frames(24.0).len(), 1);
        let empty = Timeline::new(vec![]);
        assert!(empty.seek(0.0).poses.is_empty());
    }
}
