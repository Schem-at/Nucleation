//! Ready-made clips and animators for the common cases.
//!
//! Presets are ordinary [`Clip`]s and [`BuildAnimator`]s — take one and keep
//! editing its timeline if it is nearly right.

use super::easing::Easing;
use super::pose::Pose;
use super::stagger::{Axis, Group, Grouping, Order, Pos, Stagger};
use super::timeline::Timeline;
use super::track::{Clip, Property, Track};
use super::BuildAnimator;
use crate::universal_schematic::UniversalSchematic;

/// Scale from nothing to full size with a slight overshoot — the default
/// "block appears" move.
pub fn pop_in(duration_ms: f32) -> Clip {
    Clip::new(duration_ms).track(Track::tween(
        Property::ScaleUniform,
        0.0,
        1.0,
        Easing::out_back(),
    ))
}

/// Fall into place from `height` blocks above, decelerating.
pub fn drop_in(duration_ms: f32, height: f32) -> Clip {
    Clip::new(duration_ms).track(Track::tween(
        Property::Y,
        height,
        0.0,
        Easing::Out(super::easing::Power::Cubic),
    ))
}

/// Fall in *and* scale up together.
pub fn drop_and_pop(duration_ms: f32, height: f32) -> Clip {
    drop_in(duration_ms, height).track(Track::tween(
        Property::ScaleUniform,
        0.0,
        1.0,
        Easing::out_back(),
    ))
}

/// Spin about Y while scaling in.
pub fn spin_in(duration_ms: f32, turns: f32) -> Clip {
    pop_in(duration_ms).track(Track::tween(
        Property::RotY,
        360.0 * turns,
        0.0,
        Easing::Out(super::easing::Power::Cubic),
    ))
}

/// A camera clip orbiting a full turn — the turntable, on the shared clock.
pub fn turntable(duration_ms: f32) -> Clip {
    Clip::new(duration_ms).track(Track::tween(Property::RotY, 0.0, 360.0, Easing::Linear))
}

/// Blocks appear bottom-to-top, one wave.
pub fn assemble(schem: &UniversalSchematic, clip_ms: f32, each_ms: f32) -> BuildAnimator {
    let mut anim = BuildAnimator::from_schematic(schem, Grouping::PerBlock);
    anim.timeline_mut().add_staggered(
        pop_in(clip_ms),
        &Stagger::each(Order::Axis(Axis::Y, true), each_ms),
        0.0,
    );
    anim
}

/// The printer: whole layers appear in sequence along `axis`.
pub fn print_layers(schem: &UniversalSchematic, axis: Axis, per_layer_ms: f32) -> BuildAnimator {
    let mut anim = BuildAnimator::from_schematic(schem, Grouping::Layer(axis));
    anim.timeline_mut().add_staggered(
        pop_in(per_layer_ms),
        &Stagger::each(Order::Axis(axis, true), per_layer_ms),
        0.0,
    );
    anim
}

/// Blocks arrive in the order the building tool would sweep the shape's own
/// parameter — along a bezier, around a torus, up a cylinder.
///
/// Groups whose blocks have no parameter (the shape is not parametric there)
/// sort last, so nothing is silently dropped.
pub fn along_shape(
    schem: &UniversalSchematic,
    shape: &crate::building::ShapeEnum,
    clip: Clip,
    total_ms: f32,
) -> BuildAnimator {
    let mut anim = BuildAnimator::from_schematic(schem, Grouping::PerBlock);
    let keys = shape_parameter_keys(shape, anim.groups());
    anim.timeline_mut()
        .add_staggered(clip, &Stagger::total(Order::Key(keys), total_ms), 0.0);
    anim
}

/// Mean parametric `t` per group, from [`crate::building::ShapeEnum::parameter_at`].
///
/// This is the bridge between geometry and timing: the same `t` that drives a
/// `curve_gradient` brush's colour drives when a block animates in.
pub fn shape_parameter_keys(shape: &crate::building::ShapeEnum, groups: &[Group]) -> Vec<f64> {
    groups
        .iter()
        .map(|g| {
            let mut sum = 0.0;
            let mut n = 0u32;
            for &(x, y, z) in &g.blocks {
                if let Some(t) = shape.parameter_at(x, y, z) {
                    sum += t;
                    n += 1;
                }
            }
            if n == 0 {
                f64::MAX
            } else {
                sum / n as f64
            }
        })
        .collect()
}

/// Order groups by an explicit build sequence — one entry per position, in the
/// order they were placed.
///
/// Positions not present in `sequence` sort last.
pub fn build_order_keys(sequence: &[Pos], groups: &[Group]) -> Vec<f64> {
    use std::collections::HashMap;
    let seq: HashMap<Pos, usize> = sequence.iter().enumerate().map(|(i, p)| (*p, i)).collect();
    groups
        .iter()
        .map(|g| {
            g.blocks
                .iter()
                .filter_map(|p| seq.get(p).copied())
                .min()
                .map(|v| v as f64)
                .unwrap_or(f64::MAX)
        })
        .collect()
}

/// A timeline that reveals `groups` using `clip` and `stagger`, without a schematic.
pub fn timeline_for(groups: Vec<Group>, clip: Clip, stagger: &Stagger) -> Timeline {
    let mut tl = Timeline::new(groups);
    tl.add_staggered(clip, stagger, 0.0);
    tl
}

/// The pose a group holds before its clip starts, for a given reveal clip.
///
/// Useful for asserting that "not yet revealed" really is invisible.
pub fn resting_pose(clip: &Clip) -> Pose {
    clip.sample(f32::NEG_INFINITY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{GroupId, Grouping, Order, Stagger};

    fn column(n: i32) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("column".to_string());
        for y in 0..n {
            s.set_block_from_string(0, y, 0, "minecraft:stone").ok();
        }
        s
    }

    #[test]
    fn pop_in_starts_at_zero_scale_and_ends_at_one() {
        let c = pop_in(100.0);
        assert_eq!(c.sample(0.0).scale, [0.0; 3]);
        let end = c.sample(100.0).scale;
        assert!((end[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn drop_in_lands_at_zero() {
        let c = drop_in(100.0, 8.0);
        assert_eq!(c.sample(0.0).translate[1], 8.0);
        assert!((c.sample(100.0).translate[1]).abs() < 1e-4);
    }

    #[test]
    fn spin_in_ends_unrotated_and_full_size() {
        let p = spin_in(100.0, 2.0).sample(100.0);
        assert!((p.rotate_deg[1]).abs() < 1e-3);
        assert!((p.scale[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn assemble_staggers_bottom_to_top() {
        let anim = assemble(&column(4), 100.0, 50.0);
        assert_eq!(anim.groups().len(), 4);
        // Early on, lower blocks are further along than higher ones.
        let f = anim.frame_at(60.0);
        let s0 = f.pose(0).unwrap().scale[0];
        let s3 = f.pose(3).unwrap().scale[0];
        assert!(
            s0 > s3,
            "bottom block should lead the top one ({s0} vs {s3})"
        );
        assert_eq!(s3, 0.0, "top block has not started");
    }

    #[test]
    fn everything_is_full_size_once_the_timeline_ends() {
        let anim = assemble(&column(5), 100.0, 50.0);
        let f = anim.frame_at(anim.duration_ms() + 1.0);
        for (_, p) in &f.poses {
            assert!((p.scale[0] - 1.0).abs() < 1e-4, "left mid-animation: {p:?}");
        }
    }

    #[test]
    fn print_layers_makes_one_group_per_layer() {
        let mut s = UniversalSchematic::new("slab".to_string());
        for x in 0..3 {
            for y in 0..2 {
                s.set_block_from_string(x, y, 0, "minecraft:stone").ok();
            }
        }
        let anim = print_layers(&s, Axis::Y, 100.0);
        assert_eq!(anim.groups().len(), 2, "one group per y-layer");
        assert_eq!(anim.groups()[0].blocks.len(), 3);
    }

    #[test]
    fn build_order_keys_follow_the_sequence() {
        let groups = Grouping::PerBlock.apply(&[(0, 0, 0), (1, 0, 0), (2, 0, 0)]);
        // Placed right-to-left.
        let seq = vec![(2, 0, 0), (1, 0, 0), (0, 0, 0)];
        let keys = build_order_keys(&seq, &groups);
        assert_eq!(keys, vec![2.0, 1.0, 0.0]);
        let ranks = Order::Key(keys).ranks(&groups);
        assert_eq!(ranks, vec![2, 1, 0], "animation follows placement order");
    }

    #[test]
    fn build_order_keys_put_unknown_positions_last() {
        let groups = Grouping::PerBlock.apply(&[(0, 0, 0), (9, 9, 9)]);
        let keys = build_order_keys(&[(0, 0, 0)], &groups);
        assert_eq!(keys[0], 0.0);
        assert_eq!(keys[1], f64::MAX, "unsequenced blocks sort last");
    }

    #[test]
    fn timeline_for_builds_a_usable_timeline() {
        let groups = Grouping::PerBlock.apply(&[(0, 0, 0), (1, 0, 0)]);
        let tl = timeline_for(groups, pop_in(100.0), &Stagger::each(Order::Index, 100.0));
        assert!((tl.duration_ms() - 200.0).abs() < 1e-3);
        assert_eq!(tl.seek(0.0).poses.len(), 2);
    }

    #[test]
    fn resting_pose_is_invisible_for_pop_in() {
        assert_eq!(resting_pose(&pop_in(100.0)).scale, [0.0; 3]);
    }

    #[test]
    fn turntable_covers_a_full_turn() {
        let c = turntable(1000.0);
        assert_eq!(c.sample(0.0).rotate_deg[1], 0.0);
        assert!((c.sample(1000.0).rotate_deg[1] - 360.0).abs() < 1e-3);
        assert!((c.sample(250.0).rotate_deg[1] - 90.0).abs() < 1e-3);
    }

    #[test]
    fn group_ids_are_stable_across_rebuilds() {
        let a = assemble(&column(3), 100.0, 50.0);
        let b = assemble(&column(3), 100.0, 50.0);
        let ids_a: Vec<GroupId> = a.groups().iter().map(|g| g.id).collect();
        let ids_b: Vec<GroupId> = b.groups().iter().map(|g| g.id).collect();
        assert_eq!(ids_a, ids_b);
    }
}
