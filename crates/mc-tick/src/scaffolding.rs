//! Scaffolding support propagation, scheduled independently of redstone ticks.
//!
//! Raw InWorld loading preserves saved states. Placement and shape updates
//! schedule a one-tick recheck; a changed support state uses ordinary flag-3
//! notifications, including the observer shape pass.

use crate::fluid::{FluidWorld, WaterKind};
use crate::pos::Dir;
use crate::schedule::TickPriority;
use crate::{BlockBehaviour, Pos, StateId, TickCtx, World};

pub(crate) trait ScaffoldingWorld: FluidWorld {
    fn scaffold_distance(&self, world: &World, pos: Pos) -> Option<u8>;
    fn scaffold_support(&self, ctx: &TickCtx<'_>, pos: Pos) -> bool;
}

pub(crate) struct Scaffolding<R: ScaffoldingWorld> {
    pub distance: u8,
    pub waterlogged: bool,
    /// Index = distance * 2 + bottom; waterlogging is preserved.
    pub family: [Option<StateId>; 16],
    pub rules: R,
}

impl<R: ScaffoldingWorld> Scaffolding<R> {
    fn derived(&self, ctx: &TickCtx<'_>, pos: Pos) -> (u8, bool) {
        let world = &*ctx.world;
        let below = pos.offset(Dir::Down);
        let beneath = self.rules.scaffold_distance(world, below);
        let mut distance = beneath.unwrap_or_else(|| {
            if self.rules.scaffold_support(ctx, below) {
                0
            } else {
                7
            }
        });
        if distance != 0 {
            for dir in [Dir::North, Dir::East, Dir::South, Dir::West] {
                if let Some(other) = self.rules.scaffold_distance(world, pos.offset(dir)) {
                    distance = distance.min(other.saturating_add(1));
                }
                if distance == 1 {
                    break;
                }
            }
        }
        (distance, distance > 0 && beneath.is_none())
    }
}

impl<R: ScaffoldingWorld> BlockBehaviour for Scaffolding<R> {
    fn on_state_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        ctx.schedule(pos, 1, TickPriority::Normal);
    }

    fn on_placed(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        self.on_state_changed(ctx, pos);
    }

    fn on_shape_update(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        if self.waterlogged {
            crate::fluid::schedule_fluid(ctx, pos);
        }
        ctx.schedule(pos, 1, TickPriority::Normal);
    }

    fn on_fluid_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if self.waterlogged {
            crate::fluid::spread(&self.rules, ctx, pos, WaterKind::Source);
        }
    }

    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let (distance, bottom) = self.derived(ctx, pos);
        if distance == 7 {
            // An already-unsupported state enters vanilla's falling-block
            // entity path. General falling blocks are not simulated yet; do
            // not disguise that case as an immediate item drop.
            if self.distance == 7 {
                // Surface the unsupported runtime path in the existing report;
                // supported distance-7 states can still recover when anchored.
                if let Some(pump) = ctx.drain.as_mut() {
                    pump.unknown_seen.push(ctx.world.get(pos));
                }
                return;
            }
            let (offset, velocity) = if let Some(rng) = ctx.item_entities.rng.as_mut() {
                (
                    [
                        rng.next_double_between(-0.25, 0.25),
                        rng.next_double_between(-0.25, 0.25) - 0.125,
                        rng.next_double_between(-0.25, 0.25),
                    ],
                    [
                        rng.next_double() * 0.2 - 0.1,
                        0.2,
                        rng.next_double() * 0.2 - 0.1,
                    ],
                )
            } else {
                ([0.0, -0.125, 0.0], [0.0, 0.2, 0.0])
            };
            ctx.item_entities.spawn(
                ("minecraft:scaffolding".into(), 1),
                [
                    f64::from(pos.x) + 0.5 + offset[0],
                    f64::from(pos.y) + 0.5 + offset[1],
                    f64::from(pos.z) + 0.5 + offset[2],
                ],
                velocity,
                10,
            );
            let replacement = if self.waterlogged {
                self.rules.water_state(0).unwrap_or(StateId::AIR)
            } else {
                StateId::AIR
            };
            ctx.set(pos, replacement);
        } else if let Some(state) = self.family[usize::from(distance) * 2 + usize::from(bottom)] {
            ctx.set(pos, state);
        }
    }

    fn name(&self) -> &'static str {
        "scaffolding"
    }
}

/// Union of the boxes' upward support faces, clipped to the block square.
/// Partition by every rectangle edge, so this is an exact coverage check,
/// including the base/head union of a retracting piston (no sampling grid).
pub(crate) fn full_top_face(boxes: &[([f64; 3], [f64; 3])]) -> bool {
    let faces: Vec<_> = boxes
        .iter()
        .filter(|(lo, hi)| lo[1] < 1.0 && hi[1] >= 1.0)
        .map(|(lo, hi)| {
            (
                [lo[0].max(0.0), lo[2].max(0.0)],
                [hi[0].min(1.0), hi[2].min(1.0)],
            )
        })
        .filter(|(lo, hi)| lo[0] < hi[0] && lo[1] < hi[1])
        .collect();
    let mut edges = vec![0.0, 1.0];
    for (lo, hi) in &faces {
        edges.extend([lo[0], hi[0]]);
    }
    edges.sort_by(f64::total_cmp);
    edges.dedup();
    for strip in edges.windows(2) {
        let mut intervals: Vec<_> = faces
            .iter()
            .filter(|(lo, hi)| lo[0] <= strip[0] && hi[0] >= strip[1])
            .map(|(lo, hi)| (lo[1], hi[1]))
            .collect();
        intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut end: f64 = 0.0;
        for (lo, hi) in intervals {
            if lo > end {
                return false;
            }
            end = end.max(hi);
        }
        if end < 1.0 {
            return false;
        }
    }
    true
}
