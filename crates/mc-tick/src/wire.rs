//! Redstone dust, wired into the simulation.
//!
//! Everything here mirrors `RedStoneWireBlock` + `DefaultRedstoneWireEvaluator`
//! bytecode:
//!
//! - A wire's target power is `max(blockSignal, incomingWireSignal - 1)`,
//!   where `blockSignal` is the strongest **non-wire** signal into it and
//!   `incomingWireSignal` scans the four horizontal neighbours plus the
//!   diagonal wires: **up** over a neighbour only when that neighbour is a
//!   conductor *and* nothing solid sits on the wire itself; **down** past a
//!   neighbour only when that neighbour is *not* a conductor. Those two rules
//!   are the glass diode: dust reads a wire below through glass, never a wire
//!   sitting on top of it.
//! - Power writes are silent (flag 2); the evaluator then updates the wire and
//!   its six neighbours explicitly, which is how components two steps away
//!   hear about a dust change.
//!
//! # The deliberate deviation
//!
//! Vanilla relaxes wire-by-wire, recursively, and the *intermediate* states of
//! that cascade are what produce locational quirks (a repeater latching a
//! transient). This engine settles the connected network to its fixed point
//! and only then notifies — ideal, order-free dust, the same choice
//! alternate-current makes. Captures that depend on a transient will diverge;
//! when one does, it documents the deviation rather than refuting the model.

use crate::behaviour::{BlockBehaviour, TickCtx};
use crate::pos::{Dir, Pos};
use crate::state::StateId;
use crate::world::World;
use std::collections::{HashMap, HashSet, VecDeque};

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

/// Collect the wire network reachable from `start` along signal paths.
fn network(rules: &dyn WireWorld, world: &World, start: Pos) -> Vec<Pos> {
    let mut seen: HashSet<Pos> = HashSet::new();
    let mut queue: VecDeque<Pos> = VecDeque::new();
    seen.insert(start);
    queue.push_back(start);
    while let Some(pos) = queue.pop_front() {
        for dir in [Dir::North, Dir::South, Dir::West, Dir::East] {
            let side = pos.offset(dir);
            let mut candidates = vec![side];
            // Diagonals, both directions of the same asymmetric rules so the
            // network is closed under signal flow.
            if rules.conductor(world, side) {
                candidates.push(side.offset(Dir::Up));
            } else {
                candidates.push(side.offset(Dir::Down));
            }
            for candidate in candidates {
                if rules.wire_power(world, candidate).is_some() && seen.insert(candidate) {
                    queue.push_back(candidate);
                }
            }
        }
    }
    seen.into_iter().collect()
}

/// The incoming wire signal for the wire at `pos` — the evaluator's
/// `getIncomingWireSignal`, reading from `powers` (the in-progress relaxation).
fn incoming(
    rules: &dyn WireWorld,
    world: &World,
    powers: &HashMap<Pos, u8>,
    pos: Pos,
) -> u8 {
    let at = |p: Pos| powers.get(&p).copied().or_else(|| rules.wire_power(world, p));
    let mut best = 0u8;
    let covered = rules.conductor(world, pos.offset(Dir::Up));
    for dir in [Dir::North, Dir::South, Dir::West, Dir::East] {
        let side = pos.offset(dir);
        if let Some(power) = at(side) {
            best = best.max(power);
        }
        if rules.conductor(world, side) {
            if !covered {
                if let Some(power) = at(side.offset(Dir::Up)) {
                    best = best.max(power);
                }
            }
        } else if let Some(power) = at(side.offset(Dir::Down)) {
            best = best.max(power);
        }
    }
    best.saturating_sub(1)
}

/// Settle the network containing `start` to its fixed point and write the
/// results, vanilla-style: silent power writes, loud updates for each changed
/// wire and its neighbours.
pub fn settle_network(rules: &dyn WireWorld, ctx: &mut TickCtx<'_>, start: Pos) {
    let members = network(rules, ctx.world, start);
    let mut powers: HashMap<Pos, u8> = HashMap::new();
    // Fixed-point relaxation from the block signals. Bounded: each pass can
    // only raise a wire toward 15 or settle it downward once the descending
    // pass runs, and the network is finite.
    let block_signals: HashMap<Pos, u8> = members
        .iter()
        .map(|pos| (*pos, rules.block_signal(ctx, *pos)))
        .collect();
    for pos in &members {
        powers.insert(*pos, 0);
    }
    loop {
        let mut changed = false;
        for pos in &members {
            let target = block_signals[pos].max(incoming(rules, ctx.world, &powers, *pos));
            if powers[pos] != target {
                powers.insert(*pos, target);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    for pos in &members {
        let current = rules.wire_power(ctx.world, *pos).unwrap_or(0);
        let target = powers[pos];
        if current == target {
            continue;
        }
        let Some(state) = rules.wire_with_power(ctx.world, *pos, target) else {
            continue;
        };
        // Vanilla: setBlock flag 2 (no neighbour updates), then explicit
        // updateNeighborsAt for the wire and each of its six neighbours —
        // notifications reach two steps out.
        ctx.set_quiet(*pos, state);
        for dir in crate::pos::ALL_DIRS {
            ctx.updates.push((pos.offset(dir), dir.opposite()));
            for far in crate::pos::ALL_DIRS {
                ctx.updates
                    .push((pos.offset(dir).offset(far), far.opposite()));
            }
        }
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
        settle_network(&self.rules, ctx, pos);
    }

    fn name(&self) -> &'static str {
        "redstone_wire"
    }
}
