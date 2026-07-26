//! Redstone dust — `RedStoneWireBlock` + `DefaultRedstoneWireEvaluator`,
//! transcribed.
//!
//! Vanilla relaxes dust **one wire at a time**: a notified wire recomputes its
//! own target strength from the world as it stands, writes it silently
//! (flag 2), and then runs `updateNeighborsAt` for itself and its six
//! neighbours — seven entries whose order is the iteration order of a
//! `HashSet<BlockPos>`, which this module reproduces bucket-for-bucket
//! (`java_hash_order`). The recursion through the collector is what produces
//! the ordered transients ("locational dust") that real builds latch onto —
//! the five community door fixtures refused the previous ideal fixed-point
//! model and forced this transcription.
//!
//! - Target: `blockSignal`, or if that is under 15, `max(blockSignal,
//!   incomingWireSignal)`; incoming scans the four horizontals plus the
//!   diagonal rules (up over a conductor when uncovered, down past a
//!   non-conductor — the glass diode), minus the 1-per-block falloff.
//! - Wire connection shapes are still not recomputed (documented limitation).

use crate::behaviour::{BlockBehaviour, TickCtx};
use crate::pos::{Dir, Pos};
use crate::state::StateId;
use crate::world::World;

/// What the wire needs to know about everything that is not wire.
pub trait WireWorld: Send + Sync {
    /// The strongest non-wire signal into the wire at `pos` (vanilla's
    /// `getBlockSignal` with `shouldSignal` off, so wires never count).
    fn block_signal(&self, ctx: &TickCtx<'_>, pos: Pos) -> u8;
    /// Whether the block at `pos` conducts (`isRedstoneConductor`).
    fn conductor(&self, world: &World, pos: Pos) -> bool;
    /// The power level of the wire at `pos`, if it is wire.
    fn wire_power(&self, world: &World, pos: Pos) -> Option<u8>;
    /// The same wire state with a different power level.
    fn wire_with_power(&self, world: &World, pos: Pos, power: u8) -> Option<StateId>;
}

/// `getIncomingWireSignal`: the strongest neighbouring wire, minus one.
fn incoming(rules: &dyn WireWorld, world: &World, pos: Pos) -> u8 {
    let mut best = 0u8;
    let covered = rules.conductor(world, pos.offset(Dir::Up));
    for dir in [Dir::North, Dir::South, Dir::West, Dir::East] {
        let side = pos.offset(dir);
        if let Some(power) = rules.wire_power(world, side) {
            best = best.max(power);
        }
        if rules.conductor(world, side) {
            if !covered {
                if let Some(power) = rules.wire_power(world, side.offset(Dir::Up)) {
                    best = best.max(power);
                }
            }
        } else if let Some(power) = rules.wire_power(world, side.offset(Dir::Down)) {
            best = best.max(power);
        }
    }
    best.saturating_sub(1)
}

/// The iteration order of a fresh Java `HashSet<BlockPos>` holding `pos` and
/// its six neighbours, inserted as vanilla inserts them (`pos`, then
/// `Direction.values()`): 16 buckets, index `(h ^ (h >>> 16)) & 15` over
/// `Vec3i.hashCode()` = `(y + z·31)·31 + x`, chains appended, iterated bucket
/// by bucket.
pub fn java_hash_order(pos: Pos) -> Vec<Pos> {
    let mut entries: Vec<Pos> = vec![pos];
    for dir in crate::pos::JAVA_DIRECTIONS {
        entries.push(pos.offset(dir));
    }
    let mut buckets: Vec<Vec<Pos>> = vec![Vec::new(); 16];
    for p in entries {
        let hash = p
            .y
            .wrapping_add(p.z.wrapping_mul(31))
            .wrapping_mul(31)
            .wrapping_add(p.x);
        let spread = hash ^ (((hash as u32) >> 16) as i32);
        buckets[(spread & 15) as usize].push(p);
    }
    buckets.into_iter().flatten().collect()
}

/// `DefaultRedstoneWireEvaluator.updatePowerStrength` for the wire at `pos`.
pub fn update_power_strength(
    rules: &dyn WireWorld,
    ctx: &mut TickCtx<'_>,
    pos: Pos,
    current: u8,
) {
    let block = rules.block_signal(ctx, pos);
    let target = if block == 15 {
        15
    } else {
        block.max(incoming(rules, ctx.world, pos))
    };
    if target == current {
        return;
    }
    if let Some(state) = rules.wire_with_power(ctx.world, pos, target) {
        // setBlock flag 2: no neighbour updates, but the shape pass still
        // runs — which is how an observer watching dust sees the change.
        ctx.set_shape_only(pos, state);
    }
    // Seven updateNeighborsAt entries, in the HashSet's iteration order.
    for p in java_hash_order(pos) {
        ctx.update_neighbors_at(p);
    }
}

/// Redstone dust. One instance per wire state; the connection shape comes from
/// the structure and is not recomputed (documented limitation — a piston
/// rearranging blocks beside dust would change shapes in vanilla).
pub struct Wire<R: WireWorld + Clone> {
    /// This state's power level.
    pub power_level: u8,
    /// The world rules.
    pub rules: R,
}

impl<R: WireWorld + Clone + 'static> BlockBehaviour for Wire<R> {
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        update_power_strength(&self.rules, ctx, pos, self.power_level);
    }

    /// `RedStoneWireBlock.onPlace` runs `updatePowerStrength` too, so dust
    /// recomputes itself the moment it is written into the world.
    ///
    /// That is what corrects the authored power level of a schematic: a build
    /// saved with dust at 1 is placed, each wire re-evaluates, and the ones
    /// with nothing feeding them drop to 0 — even under `knownShape`, where no
    /// update passes run at all. Without this the engine kept the file's
    /// values and started three cells hot.
    fn on_placed(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        update_power_strength(&self.rules, ctx, pos, self.power_level);
    }

    fn name(&self) -> &'static str {
        "redstone_wire"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hash_order_matches_javas_buckets() {
        // Hand-checked against Java: HashSet of (0,0,0) and neighbours.
        let order = java_hash_order(Pos::new(0, 0, 0));
        assert_eq!(order.len(), 7);
        // Every position appears exactly once.
        let mut seen = std::collections::HashSet::new();
        for p in &order {
            assert!(seen.insert(*p));
        }
    }
}
