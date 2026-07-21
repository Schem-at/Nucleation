//! Build animation: which block is where, at what pose, at time *t*.
//!
//! A deterministic, GPU-free data model. Nothing here renders — [`Timeline::seek`]
//! returns a [`Frame`] of poses that a renderer (nucleation's own, or an
//! exported JSON timeline consumed elsewhere) turns into pixels.
//!
//! ```
//! use nucleation::animation::*;
//!
//! // Blocks pop into place one after another, bottom to top.
//! let positions = vec![(0, 0, 0), (0, 1, 0), (0, 2, 0)];
//! let mut anim = BuildAnimator::from_positions(&positions, Grouping::PerBlock);
//! anim.timeline_mut().add_staggered(
//!     presets::pop_in(200.0),
//!     &Stagger::each(Order::Axis(Axis::Y, true), 80.0),
//!     0.0,
//! );
//!
//! let frame = anim.frame_at(0.0);
//! assert_eq!(frame.poses.len(), 3);
//! // The first block has started; the last has not.
//! assert!(frame.pose(0).unwrap().scale[0] < frame.pose(2).unwrap().scale[0] + 1e-6);
//! ```
//!
//! See `docs/features/animation.md` for the guide, and
//! `docs/plans/2026-07-21-build-animator-design.md` for the design rationale.

pub mod builder;
pub mod easing;
pub mod pose;
pub mod presets;
pub mod stagger;
pub mod timeline;
pub mod track;

pub use builder::{AnimationEffect, BuildAnimation};
pub use easing::{Easing, Power};
pub use pose::{Mat4, Pose};
pub use stagger::{Axis, Group, GroupId, Grouping, Order, Pos, Spread, Stagger, StaggerFrom};
pub use timeline::{CameraPose, Frame, Target, Timeline};
pub use track::{Clip, Keyframe, Modifier, Property, Repeat, Track};

use crate::universal_schematic::UniversalSchematic;

/// Groups plus the timeline that drives them.
///
/// This is the ergonomic front door. The timeline stays reachable via
/// [`BuildAnimator::timeline_mut`], so presets are starting points rather than
/// walls.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildAnimator {
    timeline: Timeline,
}

impl BuildAnimator {
    /// Group an explicit position list.
    pub fn from_positions(positions: &[Pos], grouping: Grouping) -> Self {
        BuildAnimator {
            timeline: Timeline::new(grouping.apply(positions)),
        }
    }

    /// Group a schematic's non-air blocks.
    ///
    /// Air is skipped for the same reason the diff engine skips it: air is
    /// absence, not a block, and animating it would stagger over empty space.
    pub fn from_schematic(schem: &UniversalSchematic, grouping: Grouping) -> Self {
        Self::from_positions(&non_air_positions(schem), grouping)
    }

    pub fn groups(&self) -> &[Group] {
        self.timeline.groups()
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn timeline_mut(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    pub fn duration_ms(&self) -> f32 {
        self.timeline.duration_ms()
    }

    /// Poses at one instant.
    pub fn frame_at(&self, t_ms: f32) -> Frame {
        self.timeline.seek(t_ms)
    }

    /// Every frame of a capture at `fps`, at deterministic times.
    pub fn frames(&self, fps: f64) -> Vec<Frame> {
        self.timeline.frames(fps)
    }
}

/// Non-air block positions, sorted for reproducibility.
pub fn non_air_positions(schem: &UniversalSchematic) -> Vec<Pos> {
    let mut out: Vec<Pos> = schem
        .iter_blocks()
        .filter(|(_, b)| !crate::fingerprint::is_air(b.get_name()))
        .map(|(p, _)| (p.x, p.y, p.z))
        .collect();
    out.sort_unstable();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_positions_groups_and_animates() {
        let pos: Vec<Pos> = (0..4).map(|i| (0, i, 0)).collect();
        let mut anim = BuildAnimator::from_positions(&pos, Grouping::PerBlock);
        assert_eq!(anim.groups().len(), 4);
        anim.timeline_mut().add_staggered(
            presets::pop_in(100.0),
            &Stagger::each(Order::Axis(Axis::Y, true), 50.0),
            0.0,
        );
        // 3 delays of 50 + a 100ms clip
        assert!((anim.duration_ms() - 250.0).abs() < 1e-3);
    }

    #[test]
    fn from_schematic_skips_air() {
        let mut s = UniversalSchematic::new("anim".to_string());
        s.set_block_from_string(0, 0, 0, "minecraft:stone").ok();
        s.set_block_from_string(1, 0, 0, "minecraft:air").ok();
        s.set_block_from_string(2, 0, 0, "minecraft:stone").ok();
        let anim = BuildAnimator::from_schematic(&s, Grouping::PerBlock);
        assert_eq!(anim.groups().len(), 2, "air must not become a group");
    }

    #[test]
    fn empty_schematic_produces_no_groups() {
        let s = UniversalSchematic::new("empty".to_string());
        let anim = BuildAnimator::from_schematic(&s, Grouping::PerBlock);
        assert!(anim.groups().is_empty());
        assert_eq!(anim.duration_ms(), 0.0);
        assert!(anim.frame_at(0.0).poses.is_empty());
    }

    #[test]
    fn frames_are_reproducible_across_calls() {
        let pos: Vec<Pos> = (0..8).map(|i| (i, 0, 0)).collect();
        let mut anim = BuildAnimator::from_positions(&pos, Grouping::PerBlock);
        anim.timeline_mut().add_staggered(
            presets::pop_in(200.0),
            &Stagger::total(Order::Random(3), 400.0),
            0.0,
        );
        assert_eq!(anim.frames(24.0), anim.frames(24.0));
    }
}
