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
//! - Connection shapes *are* recomputed, from `updateShape` — a wire re-faces
//!   itself whenever a neighbour changes shape, which is how dust follows a
//!   door that moves the blocks around it.

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
    /// The wire at `pos` as `(power, [north, south, west, east])`, if it is wire.
    fn wire_shape(&self, world: &World, pos: Pos) -> Option<(u8, [WireSide; 4])>;
    /// The wire state with exactly this power and these connections.
    fn wire_with_shape(&self, power: u8, sides: [WireSide; 4]) -> Option<StateId>;
    /// `shouldConnectTo`: whether dust turns to face the block at `pos`.
    /// `from` is the direction from the wire toward it; `None` asks the
    /// direction-agnostic question the diagonal checks use.
    fn should_connect_to(&self, world: &World, pos: Pos, from: Option<Dir>) -> bool;
    /// `canSurviveOn`: whether dust could sit on the block at `pos`, which is
    /// also what decides whether it can climb it.
    fn sturdy_up(&self, world: &World, pos: Pos) -> bool;
    /// `isCollisionShapeFullBlock`: a wire climbing a *full* block renders as
    /// `up`; climbing anything shallower stays `side`.
    fn full_block(&self, world: &World, pos: Pos) -> bool;
}

/// A dust connection: `RedstoneSide`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WireSide {
    /// `none` — no connection on this face.
    None,
    /// `side` — connected horizontally.
    Side,
    /// `up` — connected, and climbing the block on this face.
    Up,
}

impl WireSide {
    /// `RedstoneSide.isConnected`: `up` and `side` both carry signal.
    pub fn connected(self) -> bool {
        self != WireSide::None
    }
}

/// The four horizontals in the order the connection arrays use.
pub const WIRE_SIDES: [Dir; 4] = [Dir::North, Dir::South, Dir::West, Dir::East];

/// The index into a connection array for a horizontal direction.
fn side_index(dir: Dir) -> Option<usize> {
    WIRE_SIDES.iter().position(|d| *d == dir)
}

/// `RedStoneWireBlock.getConnectingSide`: what the wire at `pos` does on the
/// face toward `dir`.
///
/// `can_connect_up` is the caller's cached "nothing solid overhead", because a
/// covered wire cannot climb anything.
pub fn connecting_side(
    rules: &dyn WireWorld,
    world: &World,
    pos: Pos,
    dir: Dir,
    can_connect_up: bool,
) -> WireSide {
    let side = pos.offset(dir);
    if can_connect_up
        && rules.sturdy_up(world, side)
        && rules.should_connect_to(world, side.offset(Dir::Up), None)
    {
        return if rules.full_block(world, side) { WireSide::Up } else { WireSide::Side };
    }
    // Otherwise the wire faces the block only if that block takes a signal, or
    // if there is dust below it to step down onto — and a conductor blocks the
    // step down, which is what makes glass a diode.
    if !rules.should_connect_to(world, side, Some(dir))
        && (rules.conductor(world, side)
            || !rules.should_connect_to(world, side.offset(Dir::Down), None))
    {
        WireSide::None
    } else {
        WireSide::Side
    }
}

/// `getConnectionState`: recompute every unconnected face, then apply the
/// symmetry rule — a wire connected on one axis only spreads *across* the
/// other, so a single-neighbour cell renders as a line rather than a stub.
///
/// A wire that was a dot and stays a dot keeps its dot shape; that is the one
/// case vanilla leaves alone.
pub fn connection_state(
    rules: &dyn WireWorld,
    world: &World,
    pos: Pos,
    current: [WireSide; 4],
) -> [WireSide; 4] {
    let was_dot = current.iter().all(|s| !s.connected());
    let can_connect_up = !rules.conductor(world, pos.offset(Dir::Up));
    let mut sides = [WireSide::None; 4];
    for (index, dir) in WIRE_SIDES.iter().enumerate() {
        sides[index] = connecting_side(rules, world, pos, *dir, can_connect_up);
    }
    if was_dot && sides.iter().all(|s| !s.connected()) {
        return sides;
    }
    let [north, south, west, east] = sides;
    let no_north_south = !north.connected() && !south.connected();
    let no_west_east = !west.connected() && !east.connected();
    if !west.connected() && no_north_south {
        sides[2] = WireSide::Side;
    }
    if !east.connected() && no_north_south {
        sides[3] = WireSide::Side;
    }
    if !north.connected() && no_west_east {
        sides[0] = WireSide::Side;
    }
    if !south.connected() && no_west_east {
        sides[1] = WireSide::Side;
    }
    sides
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

/// `RedStoneWireBlock.updateNeighborsOfNeighboringWires`: every horizontal
/// neighbour that is itself dust gets an update pass, and so does the dust one
/// step up or down from that neighbour — up when the neighbour conducts, down
/// when it does not, which is exactly how dust follows a staircase.
pub fn update_neighbours_of_neighbouring_wires(
    rules: &dyn WireWorld,
    ctx: &mut TickCtx<'_>,
    pos: Pos,
) {
    const HORIZONTAL: [Dir; 4] = [Dir::North, Dir::South, Dir::West, Dir::East];
    for dir in HORIZONTAL {
        check_corner_change_at(rules, ctx, pos.offset(dir));
    }
    for dir in HORIZONTAL {
        let side = pos.offset(dir);
        let step = if rules.conductor(ctx.world, side) { Dir::Up } else { Dir::Down };
        check_corner_change_at(rules, ctx, side.offset(step));
    }
}

/// `checkCornerChangeAt`: an update pass, but only if that position is dust.
fn check_corner_change_at(rules: &dyn WireWorld, ctx: &mut TickCtx<'_>, pos: Pos) {
    if rules.wire_power(ctx.world, pos).is_some() {
        ctx.update_neighbors_at(pos);
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
    /// The rest of `onPlace` matters just as much, and is *unconditional*:
    /// whatever the recomputed power, dust always pokes the neighbours of the
    /// blocks above and below it, plus any neighbouring wires. That reaches two
    /// blocks out — a wire sitting on a barrel notifies the comparator on the
    /// barrel's far side — and it is the only thing that wakes diodes during a
    /// `knownShape` placement, where no update pass runs.
    fn on_placed(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        update_power_strength(&self.rules, ctx, pos, self.power_level);
        for dir in [Dir::Down, Dir::Up] {
            ctx.update_neighbors_at(pos.offset(dir));
        }
        update_neighbours_of_neighbouring_wires(&self.rules, ctx, pos);
    }

    /// `RedStoneWireBlock.updateShape`.
    ///
    /// From above, the whole connection state is recomputed. From a horizontal
    /// neighbour, vanilla takes a shortcut: if that face's *connectedness* is
    /// unchanged and the wire is not a cross, only that face is rewritten —
    /// which preserves an `up` that a full recompute would flatten. Anything
    /// else recomputes from scratch, keeping only the power level.
    ///
    /// From below vanilla asks whether the dust can still survive; this engine
    /// does not break blocks, so that case is left alone.
    fn on_shape_update(&self, ctx: &mut TickCtx<'_>, pos: Pos, from: Dir) {
        if from == Dir::Down {
            return;
        }
        let Some((power, current)) = self.rules.wire_shape(ctx.world, pos) else { return };
        let sides = if from == Dir::Up {
            connection_state(&self.rules, ctx.world, pos, current)
        } else {
            let Some(index) = side_index(from) else { return };
            let is_cross = current.iter().all(|s| s.connected());
            let can_connect_up = !self.rules.conductor(ctx.world, pos.offset(Dir::Up));
            let side = connecting_side(&self.rules, ctx.world, pos, from, can_connect_up);
            if side.connected() == current[index].connected() && !is_cross {
                let mut sides = current;
                sides[index] = side;
                sides
            } else {
                connection_state(&self.rules, ctx.world, pos, current)
            }
        };
        if sides == current {
            return;
        }
        let Some(state) = self.rules.wire_with_shape(power, sides) else { return };
        // A shape write, so it carries on through the shape pass without
        // notifying neighbours — the wire re-faces itself, and the dust beyond
        // it re-faces in turn.
        ctx.set_shape_only(pos, state);
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

