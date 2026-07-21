//! Construction-shaped animation builder shared by Rust and generated bindings.
//!
//! Mutations are recorded as animation groups while being applied immediately to
//! the underlying schematic. A normal mutation creates one group; `begin_group`
//! / `end_group` batches many mutations into one target.

use super::{Clip, Easing, Group, GroupId, Keyframe, Property, Repeat, Target, Timeline, Track};
use crate::universal_schematic::UniversalSchematic;

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationEffect {
    clip: Clip,
}

impl AnimationEffect {
    pub fn new(duration_ms: f32) -> Self {
        Self {
            clip: Clip::new(duration_ms),
        }
    }

    pub fn from_clip(clip: Clip) -> Self {
        Self { clip }
    }

    pub fn tween(mut self, property: Property, from: f32, to: f32, easing: Easing) -> Self {
        self.clip
            .tracks
            .push(Track::tween(property, from, to, easing));
        self
    }

    pub fn keyframe(mut self, property: Property, at: f32, value: f32, easing: Easing) -> Self {
        if let Some(track) = self.clip.tracks.iter_mut().find(|t| t.property == property) {
            track.keys.push(Keyframe::eased(at, value, easing));
            track.keys.sort_by(|a, b| {
                a.at.partial_cmp(&b.at)
                    .unwrap_or(core::cmp::Ordering::Equal)
            });
        } else {
            self.clip.tracks.push(Track::new(
                property,
                vec![Keyframe::eased(at, value, easing)],
            ));
        }
        self
    }

    pub fn repeat_forever(mut self) -> Self {
        self.clip.repeat = Repeat::Forever;
        self
    }

    pub fn clip(&self) -> &Clip {
        &self.clip
    }
}

impl From<Clip> for AnimationEffect {
    fn from(value: Clip) -> Self {
        Self::from_clip(value)
    }
}

#[derive(Debug, Clone)]
struct RecordedStep {
    blocks: Vec<(i32, i32, i32)>,
    effect: Option<Clip>,
    key: f32,
}

#[derive(Debug, Clone)]
struct OpenGroup {
    blocks: Vec<(i32, i32, i32)>,
    effect: Option<Clip>,
    key: f32,
}

/// A schematic plus its recorded construction timeline.
#[derive(Debug, Clone)]
pub struct BuildAnimation {
    schematic: UniversalSchematic,
    steps: Vec<RecordedStep>,
    open_group: Option<OpenGroup>,
    default_effect: Clip,
    pending_effect: Option<Clip>,
    step_ms: f32,
    stagger_total_ms: Option<f32>,
    stagger_offset_ms: f32,
    loop_period_ms: Option<f32>,
    camera: Vec<(Clip, f32)>,
}

impl BuildAnimation {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schematic: UniversalSchematic::new(name.into()),
            steps: Vec::new(),
            open_group: None,
            default_effect: super::presets::drop_and_pop(480.0, 4.5),
            pending_effect: None,
            step_ms: 600.0,
            stagger_total_ms: None,
            stagger_offset_ms: 0.0,
            loop_period_ms: None,
            camera: Vec::new(),
        }
    }

    pub fn schematic(&self) -> &UniversalSchematic {
        &self.schematic
    }

    pub fn schematic_mut(&mut self) -> &mut UniversalSchematic {
        &mut self.schematic
    }

    pub fn set_default_effect(&mut self, effect: impl Into<AnimationEffect>) {
        self.default_effect = effect.into().clip;
    }

    /// Apply an effect to exactly the next recorded operation or group.
    pub fn with_effect(&mut self, effect: impl Into<AnimationEffect>) -> &mut Self {
        self.pending_effect = Some(effect.into().clip);
        self
    }

    pub fn set_step_ms(&mut self, step_ms: f32) {
        self.step_ms = step_ms.max(0.0);
    }

    pub fn step_ms(&self) -> f32 {
        self.step_ms
    }

    pub fn set_stagger_total_ms(&mut self, stagger_total_ms: Option<f32>) {
        self.stagger_total_ms = stagger_total_ms.map(|v| v.max(0.0));
    }

    pub fn set_stagger_offset_ms(&mut self, offset_ms: f32) {
        self.stagger_offset_ms = if offset_ms.is_finite() {
            offset_ms
        } else {
            0.0
        };
    }

    pub fn set_loop_period_ms(&mut self, period_ms: Option<f32>) {
        self.loop_period_ms = period_ms.filter(|period| period.is_finite() && *period > 0.0);
    }

    pub fn begin_group(&mut self, effect: Option<Clip>) -> Result<(), String> {
        let key = self.steps.len() as f32;
        self.begin_group_with_key(effect, key)
    }

    pub fn begin_group_with_key(&mut self, effect: Option<Clip>, key: f32) -> Result<(), String> {
        let pending_effect = self.pending_effect.take();
        let effect = effect.or(pending_effect);
        if self.open_group.is_some() {
            return Err("animation groups cannot be nested".into());
        }
        self.open_group = Some(OpenGroup {
            blocks: Vec::new(),
            effect,
            key,
        });
        Ok(())
    }

    pub fn end_group(&mut self) -> Result<GroupId, String> {
        let mut group = self
            .open_group
            .take()
            .ok_or_else(|| "no animation group is open".to_string())?;
        group.blocks.sort_unstable();
        group.blocks.dedup();
        if group.blocks.is_empty() {
            return Err("an animation group cannot be empty".into());
        }
        let id = self.steps.len() as GroupId;
        self.steps.push(RecordedStep {
            blocks: group.blocks,
            effect: group.effect,
            key: group.key,
        });
        Ok(id)
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: &str) -> Result<GroupId, String> {
        let effect = self.pending_effect.take();
        self.set_block_inner(x, y, z, block, effect)
    }

    /// Fill a parametric shape and partition its voxels into ordered animation
    /// groups using the shape's normalised `parameter_at` value.
    pub fn fill_along_parameter(
        &mut self,
        shape: &crate::building::ShapeEnum,
        brush: &crate::building::BrushEnum,
        group_count: usize,
    ) -> Result<Vec<GroupId>, String> {
        use crate::building::Shape;
        use std::collections::BTreeMap;

        let effect = self.pending_effect.take();
        if self.open_group.is_some() {
            return Err("finish the open animation group before filling a shape".into());
        }
        if group_count == 0 {
            return Err("parametric fill requires at least one animation group".into());
        }

        let mut placements = BTreeMap::new();
        shape.for_each_point(|x, y, z| {
            let Some(parameter) = shape.parameter_at(x, y, z) else {
                return;
            };
            let normal = shape.normal_at(x, y, z);
            if let Some(block) = brush.get_block_with_parameter(x, y, z, normal, Some(parameter)) {
                let group = ((parameter.clamp(0.0, 1.0) * group_count as f64).floor() as usize)
                    .min(group_count - 1);
                placements.insert((x, y, z), (group, block));
            }
        });
        if placements.is_empty() {
            return Err("parametric shape produced no blocks".into());
        }
        let (min_x, min_y, min_z, max_x, max_y, max_z) = shape.bounds();
        self.schematic
            .ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
        let replaced: std::collections::HashSet<_> = placements.keys().copied().collect();
        for step in &mut self.steps {
            step.blocks.retain(|position| !replaced.contains(position));
        }

        let mut buckets = vec![Vec::new(); group_count];
        for (position, (group, block)) in placements {
            self.schematic
                .set_block(position.0, position.1, position.2, &block);
            buckets[group].push(position);
        }

        let mut ids = Vec::new();
        for (index, blocks) in buckets.into_iter().enumerate() {
            if blocks.is_empty() {
                continue;
            }
            let id = self.steps.len() as GroupId;
            self.steps.push(RecordedStep {
                blocks,
                effect: effect.clone(),
                key: index as f32,
            });
            ids.push(id);
        }
        Ok(ids)
    }

    fn set_block_inner(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block: &str,
        effect: Option<Clip>,
    ) -> Result<GroupId, String> {
        if self.open_group.is_some() && effect.is_some() {
            return Err("set the effect on the open group, not a mutation inside it".into());
        }
        self.schematic.set_block_from_string(x, y, z, block)?;
        let pos = (x, y, z);
        for step in &mut self.steps {
            step.blocks.retain(|&recorded| recorded != pos);
        }
        if let Some(group) = self.open_group.as_mut() {
            group.blocks.retain(|&recorded| recorded != pos);
            group.blocks.push(pos);
            return Ok(self.steps.len() as GroupId);
        }
        let id = self.steps.len() as GroupId;
        self.steps.push(RecordedStep {
            blocks: vec![pos],
            effect,
            key: id as f32,
        });
        Ok(id)
    }

    /// Record an entity's floored block position as an animation target after the
    /// caller adds the entity to `schematic_mut()`.
    pub fn record_entity_position(&mut self, x: f64, y: f64, z: f64) -> Result<GroupId, String> {
        let effect = self.pending_effect.take();
        let pos = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
        if let Some(group) = self.open_group.as_mut() {
            if effect.is_some() {
                return Err("set the effect on the open group, not an entity inside it".into());
            }
            group.blocks.push(pos);
            return Ok(self.steps.len() as GroupId);
        }
        let id = self.steps.len() as GroupId;
        self.steps.push(RecordedStep {
            blocks: vec![pos],
            effect,
            key: id as f32,
        });
        Ok(id)
    }

    pub fn animate_camera(&mut self, clip: Clip, offset_ms: f32) {
        self.camera.push((clip, offset_ms));
    }

    pub fn groups(&self) -> Vec<Group> {
        self.steps
            .iter()
            .enumerate()
            .map(|(i, step)| Group::new(i as GroupId, step.blocks.clone()))
            .collect()
    }
    pub fn timeline(&self) -> Timeline {
        let groups = self.groups();
        let mut timeline = Timeline::new(groups);
        let delays = self.delays();
        for (i, step) in self.steps.iter().enumerate() {
            timeline.add(
                step.effect
                    .clone()
                    .unwrap_or_else(|| self.default_effect.clone()),
                Target::Group(i as GroupId),
                delays[i],
            );
        }
        for (clip, offset) in &self.camera {
            timeline.add(clip.clone(), Target::Camera, *offset);
        }
        timeline
    }

    /// Sample deterministic frames and optionally hold the final state.
    pub fn frames(&self, fps: f64, hold_ms: f32) -> Vec<super::Frame> {
        let timeline = self.timeline();
        let fps = fps.max(1.0);
        let duration = self
            .loop_period_ms
            .map(f64::from)
            .unwrap_or_else(|| timeline.duration_ms() as f64 + hold_ms.max(0.0) as f64);
        let count = ((duration / 1000.0) * fps).round().max(1.0) as usize;
        (0..count)
            .map(|i| timeline.seek((i as f64 * 1000.0 / fps) as f32))
            .collect()
    }

    fn delays(&self) -> Vec<f32> {
        let Some(total) = self.stagger_total_ms else {
            return (0..self.steps.len())
                .map(|i| i as f32 * self.step_ms + self.stagger_offset_ms)
                .collect();
        };
        if self.steps.len() <= 1 {
            return vec![self.stagger_offset_ms; self.steps.len()];
        }
        let min = self
            .steps
            .iter()
            .map(|s| s.key)
            .fold(f32::INFINITY, f32::min);
        let max = self
            .steps
            .iter()
            .map(|s| s.key)
            .fold(f32::NEG_INFINITY, f32::max);
        let span = max - min;
        if span.abs() <= f32::EPSILON {
            return vec![self.stagger_offset_ms; self.steps.len()];
        }
        self.steps
            .iter()
            .map(|s| (s.key - min) / span * total + self.stagger_offset_ms)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{presets, Property};
    use crate::building::{BrushEnum, Curve3D, ShapeEnum, SolidBrush, TubePath};

    #[test]
    fn records_single_mutations_and_explicit_groups_as_animation_targets() {
        let mut animation = BuildAnimation::new("toy");
        animation.begin_group(None).unwrap();
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        animation.set_block(1, 0, 0, "minecraft:stone").unwrap();
        let floor = animation.end_group().unwrap();
        let prop = animation.set_block(0, 1, 0, "minecraft:furnace").unwrap();

        assert_eq!(floor, 0);
        assert_eq!(prop, 1);
        assert_eq!(animation.groups()[0].blocks.len(), 2);
        assert_eq!(animation.groups()[1].blocks, vec![(0, 1, 0)]);
    }

    #[test]
    fn replacing_a_block_transfers_the_position_to_the_latest_step() {
        let mut animation = BuildAnimation::new("replacement");
        animation.begin_group(None).unwrap();
        animation
            .set_block(0, 1, 0, "minecraft:oak_planks")
            .unwrap();
        animation
            .set_block(1, 1, 0, "minecraft:oak_planks")
            .unwrap();
        animation.end_group().unwrap();

        animation
            .set_block(0, 1, 0, "minecraft:wall_torch[facing=south]")
            .unwrap();

        let groups = animation.groups();
        assert_eq!(groups[0].blocks, vec![(1, 1, 0)]);
        assert_eq!(groups[1].blocks, vec![(0, 1, 0)]);
        assert_eq!(
            groups
                .iter()
                .flat_map(|group| &group.blocks)
                .filter(|&&position| position == (0, 1, 0))
                .count(),
            1,
            "a replaced coordinate must render in only its latest animation step"
        );
        assert_eq!(
            animation
                .schematic()
                .get_block(0, 1, 0)
                .expect("replacement block")
                .get_name(),
            "minecraft:wall_torch"
        );
    }

    #[test]
    fn operation_effect_overrides_the_animation_default() {
        let mut animation = BuildAnimation::new("toy");
        animation.set_default_effect(presets::drop_and_pop(500.0, 5.0));
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        let custom = animation
            .with_effect(presets::spin_in(800.0, 1.0))
            .set_block(1, 0, 0, "minecraft:stone")
            .unwrap();
        let plain = animation.set_block(2, 0, 0, "minecraft:stone").unwrap();

        let timeline = animation.timeline();
        assert_eq!(timeline.seek(0.0).pose(0).unwrap().translate[1], 5.0);
        assert_eq!(
            timeline
                .seek(animation.step_ms())
                .pose(custom)
                .unwrap()
                .rotate_deg[1],
            360.0
        );
        assert_eq!(
            timeline
                .seek(animation.step_ms() * 2.0)
                .pose(plain)
                .unwrap()
                .translate[1],
            5.0,
            "with_effect must be consumed by exactly one operation"
        );
    }

    #[test]
    fn camera_uses_the_same_clip_timeline_as_groups() {
        let mut animation = BuildAnimation::new("camera");
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        animation.animate_camera(presets::turntable(1_000.0), 0.0);

        let camera = animation.timeline().seek(500.0).camera.unwrap();
        assert!((camera.yaw - 180.0).abs() < 0.01);
    }

    #[test]
    fn failed_operation_consumes_the_pending_effect() {
        let mut animation = BuildAnimation::new("failed-operation-effect");
        let failed = animation
            .with_effect(presets::spin_in(800.0, 1.0))
            .set_block(0, 0, 0, "not a valid block[");
        assert!(failed.is_err());

        let plain = animation.set_block(1, 0, 0, "minecraft:stone").unwrap();
        let timeline = animation.timeline();
        let frame = timeline.seek(0.0);
        let pose = frame.pose(plain).unwrap();
        assert!(pose.rotate_deg.iter().all(|value| value.abs() < 1e-4));
    }

    #[test]
    fn with_effect_applies_to_the_next_group_as_one_target() {
        let mut animation = BuildAnimation::new("group-effect");
        animation
            .with_effect(presets::spin_in(700.0, 1.0))
            .begin_group(None)
            .unwrap();
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        animation.set_block(1, 0, 0, "minecraft:stone").unwrap();
        let group = animation.end_group().unwrap();
        let plain = animation.set_block(2, 0, 0, "minecraft:stone").unwrap();

        let timeline = animation.timeline();
        assert_eq!(timeline.seek(0.0).pose(group).unwrap().rotate_deg[1], 360.0);
        assert_eq!(
            timeline
                .seek(animation.step_ms())
                .pose(plain)
                .unwrap()
                .translate[1],
            4.5,
            "the group must consume the pending effect"
        );
    }

    #[test]
    fn with_effect_applies_to_entity_recording_and_is_consumed() {
        let mut animation = BuildAnimation::new("entity-effect");
        let custom = animation
            .with_effect(presets::spin_in(700.0, 1.0))
            .record_entity_position(0.5, 1.0, 0.5)
            .unwrap();
        let plain = animation.record_entity_position(2.5, 1.0, 0.5).unwrap();

        let timeline = animation.timeline();
        assert_eq!(
            timeline.seek(0.0).pose(custom).unwrap().rotate_deg[1],
            360.0
        );
        assert_eq!(
            timeline
                .seek(animation.step_ms())
                .pose(plain)
                .unwrap()
                .translate[1],
            4.5
        );
    }

    #[test]
    fn keyed_groups_can_drive_a_trefoil_style_stagger() {
        let mut animation = BuildAnimation::new("curve");
        for i in 0..3 {
            animation.begin_group_with_key(None, i as f32).unwrap();
            animation.set_block(i, 0, 0, "minecraft:stone").unwrap();
            animation.end_group().unwrap();
        }
        animation.set_stagger_total_ms(Some(1_000.0));
        animation.set_default_effect(presets::pop_in(200.0));

        let timeline = animation.timeline();
        assert_eq!(timeline.seek(0.0).pose(0).unwrap().scale, [0.0; 3]);
        assert_eq!(timeline.seek(0.0).pose(2).unwrap().scale, [0.0; 3]);
        assert!(timeline.seek(600.0).pose(0).unwrap().scale[0] > 0.99);
        assert_eq!(timeline.seek(600.0).pose(2).unwrap().scale, [0.0; 3]);
    }

    #[test]
    fn parametric_fill_records_one_ordered_group_per_curve_partition() {
        let curve = Curve3D::new(vec![(0.0, 0.0, 0.0), (9.0, 0.0, 0.0)], false).unwrap();
        let shape = ShapeEnum::TubePath(TubePath::new(curve, 0.6).unwrap());
        let brush = BrushEnum::Solid(SolidBrush::new(crate::BlockState::new("minecraft:stone")));
        let mut animation = BuildAnimation::new("parametric");

        animation.fill_along_parameter(&shape, &brush, 4).unwrap();

        assert_eq!(animation.groups().len(), 4);
        assert_eq!(
            animation
                .steps
                .iter()
                .map(|step| step.blocks.len())
                .sum::<usize>(),
            10
        );
        assert_eq!(
            animation
                .groups()
                .iter()
                .flat_map(|group| group.blocks.iter())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            10,
            "every filled voxel must belong to exactly one animation group"
        );
        assert_eq!(
            animation.schematic().get_block(9, 0, 0).unwrap().get_name(),
            "minecraft:stone"
        );
        assert!(animation
            .steps
            .windows(2)
            .all(|pair| pair[0].key < pair[1].key));
    }

    #[test]
    fn loop_period_and_negative_stagger_phase_capture_one_seamless_cycle() {
        let mut animation = BuildAnimation::new("loop");
        for i in 0..3 {
            animation.begin_group_with_key(None, i as f32).unwrap();
            animation.set_block(i, 0, 0, "minecraft:stone").unwrap();
            animation.end_group().unwrap();
        }
        let effect = AnimationEffect::new(1_000.0)
            .keyframe(Property::ScaleUniform, 0.0, 0.0, Easing::Linear)
            .keyframe(Property::ScaleUniform, 0.1, 1.0, Easing::Linear)
            .keyframe(Property::ScaleUniform, 0.58, 1.0, Easing::Linear)
            .keyframe(Property::ScaleUniform, 0.68, 0.0, Easing::Linear)
            .keyframe(Property::ScaleUniform, 1.0, 0.0, Easing::Linear)
            .repeat_forever();
        animation.set_default_effect(effect);
        animation.set_stagger_total_ms(Some(400.0));
        animation.set_stagger_offset_ms(-1_000.0);
        animation.set_loop_period_ms(Some(1_000.0));

        let frames = animation.frames(20.0, 5_000.0);

        assert_eq!(
            frames.len(),
            20,
            "loop capture must ignore one-shot hold time"
        );
        assert!(frames.first().unwrap().pose(2).unwrap().scale[0] > 0.0);
        assert!(frames.last().unwrap().pose(2).unwrap().scale[0] > 0.0);
    }

    #[test]
    fn frame_capture_can_add_a_readme_hold_after_the_timeline() {
        let mut animation = BuildAnimation::new("hold");
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        let frames = animation.frames(20.0, 1_000.0);
        assert_eq!(frames.len(), 30); // (480ms + 1000ms) * 20fps rounds to 30
        assert!(frames.last().unwrap().pose(0).unwrap().scale[0] > 0.99);
    }

    #[test]
    fn custom_effect_tracks_are_shared_by_all_language_adapters() {
        let effect = AnimationEffect::new(500.0)
            .tween(Property::Y, 4.0, 0.0, crate::animation::Easing::Linear)
            .tween(
                Property::ScaleUniform,
                0.0,
                1.0,
                crate::animation::Easing::out_back(),
            );
        assert_eq!(effect.clip().sample(0.0).translate[1], 4.0);
        assert_eq!(effect.clip().sample(0.0).scale, [0.0; 3]);
        assert!((effect.clip().sample(500.0).scale[0] - 1.0).abs() < 0.001);
    }
}
