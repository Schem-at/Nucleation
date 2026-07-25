//! Redstone wire and power sources.
//!
//! # Derived from captured vanilla traces, not from memory
//!
//! Every rule here is pinned to an observation from `tools/gametest`, running the
//! real game. The two that shaped the design:
//!
//! ```text
//! redstone_block at [0,1,0], dust running east:
//!   [1,1,0] power=15   [2,1,0] power=14   [3,1,0] power=13   [4,1,0] power=12
//!
//! break the redstone_block:
//!   tick 0 -> all four wires drop to power=0
//! ```
//!
//! The second is the important one. **Wire settles synchronously**: the whole line
//! depowered within a single tick, not one block per tick. So wire is not modelled
//! as a per-tick propagation — it is a network that reaches its fixed point inside
//! one update. Only genuinely delayed components (repeaters, torches, pistons) may
//! span ticks.
//!
//! Getting this backwards is a classic way to build a redstone simulator that looks
//! right on a short wire and reports every door timing wrong.

use crate::pos::{Pos, ALL_DIRS};
use crate::state::StateId;
use crate::world::World;
use std::collections::{HashMap, VecDeque};

/// The maximum power a redstone signal carries.
pub const MAX_POWER: u8 = 15;

/// The wire and source states a network knows about.
///
/// States are supplied by the caller rather than discovered, because this crate
/// deliberately knows nothing about Minecraft's block list — the same reasoning as
/// [`crate::state::StateRegistry`].
#[derive(Debug, Clone)]
pub struct RedstoneNetwork {
    /// Dust state for each power level, indexed 0..=15.
    wire: [StateId; 16],
    /// States that emit a constant power in all directions.
    sources: HashMap<StateId, u8>,
    /// Reverse lookup: which power level a dust state represents.
    wire_power: HashMap<StateId, u8>,
}

impl RedstoneNetwork {
    /// A network over the given dust states, indexed by power level.
    pub fn new(wire: [StateId; 16]) -> Self {
        let wire_power = wire
            .iter()
            .enumerate()
            .map(|(power, state)| (*state, power as u8))
            .collect();
        Self {
            wire,
            sources: HashMap::new(),
            wire_power,
        }
    }

    /// Register a constant power emitter, e.g. a redstone block at 15.
    pub fn add_source(&mut self, state: StateId, power: u8) {
        self.sources.insert(state, power.min(MAX_POWER));
    }

    /// The power level a dust state represents, if it is dust.
    pub fn power_of(&self, state: StateId) -> Option<u8> {
        self.wire_power.get(&state).copied()
    }

    /// Whether `state` is dust.
    pub fn is_wire(&self, state: StateId) -> bool {
        self.wire_power.contains_key(&state)
    }

    /// The constant power `state` emits, if it is a source.
    pub fn source_power(&self, state: StateId) -> Option<u8> {
        self.sources.get(&state).copied()
    }

    /// Recompute every dust power in `world` and apply the result.
    ///
    /// Returns how many blocks changed. Runs to a fixed point in one call, which is
    /// what the captured trace requires: breaking a source depowers the entire line
    /// within a single tick.
    ///
    /// Implemented as a breadth-first relaxation from the sources in descending power
    /// order, so each wire is assigned its final value once rather than being revised
    /// repeatedly.
    pub fn settle(&self, world: &mut World) -> usize {
        let wires: Vec<Pos> = world
            .iter_non_air()
            .filter(|(_, state)| self.is_wire(*state))
            .map(|(pos, _)| pos)
            .collect();

        if wires.is_empty() {
            return 0;
        }

        // Seed: each wire's power from any adjacent constant source. Dust beside a
        // redstone block reads 15, which the trace confirms.
        let mut power: HashMap<Pos, u8> = wires.iter().map(|pos| (*pos, 0)).collect();
        let mut queue: VecDeque<Pos> = VecDeque::new();

        for pos in &wires {
            let mut best = 0u8;
            for dir in ALL_DIRS {
                if let Some(source) = self.source_power(world.get(pos.offset(dir))) {
                    best = best.max(source);
                }
            }
            if best > 0 {
                power.insert(*pos, best);
                queue.push_back(*pos);
            }
        }

        // Relax outward: a neighbouring wire is one weaker, never below zero.
        while let Some(pos) = queue.pop_front() {
            let here = power[&pos];
            if here <= 1 {
                continue;
            }
            for dir in ALL_DIRS {
                let next = pos.offset(dir);
                let Some(current) = power.get(&next).copied() else {
                    continue;
                };
                if current < here - 1 {
                    power.insert(next, here - 1);
                    queue.push_back(next);
                }
            }
        }

        let mut changed = 0;
        for pos in wires {
            let level = power[&pos];
            let target = self.wire[level as usize];
            if world.get(pos) != target {
                world.set(pos, target);
                changed += 1;
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos::Bounds;
    use crate::state::StateRegistry;

    /// Builds the palette the captured traces use.
    fn setup() -> (StateRegistry, RedstoneNetwork, StateId, StateId) {
        let mut states = StateRegistry::new();
        let mut wire = [StateId::AIR; 16];
        for (power, slot) in wire.iter_mut().enumerate() {
            *slot = states
                .intern(&format!(
                    "minecraft:redstone_wire[east=side,north=none,power={power},south=none,west=side]"
                ))
                .unwrap();
        }
        let block = states.intern("minecraft:redstone_block").unwrap();
        let stone = states.intern("minecraft:stone").unwrap();

        let mut network = RedstoneNetwork::new(wire);
        network.add_source(block, 15);
        (states, network, block, stone)
    }

    fn world() -> World {
        World::new(Bounds::new(Pos::new(-2, 0, -2), Pos::new(10, 4, 2)))
    }

    #[test]
    fn dust_attenuates_exactly_as_vanilla_did() {
        // Ground truth, captured from real Minecraft:
        //   [1,1,0] power=15  [2,1,0] power=14  [3,1,0] power=13  [4,1,0] power=12
        let (_states, network, block, stone) = setup();
        let mut world = world();

        for x in 0..=4 {
            world.set(Pos::new(x, 0, 0), stone);
        }
        world.set(Pos::new(0, 1, 0), block);
        for x in 1..=4 {
            world.set(Pos::new(x, 1, 0), network.wire[0]);
        }

        network.settle(&mut world);

        let power_at = |x: i32| network.power_of(world.get(Pos::new(x, 1, 0))).unwrap();
        assert_eq!(power_at(1), 15, "dust beside the source reads 15");
        assert_eq!(power_at(2), 14);
        assert_eq!(power_at(3), 13);
        assert_eq!(power_at(4), 12);
    }

    #[test]
    fn breaking_the_source_depowers_the_whole_line_at_once() {
        // The trace showed all four wires dropping to 0 within tick 0 — wire settles
        // synchronously, it does not propagate one block per tick.
        let (_states, network, block, stone) = setup();
        let mut world = world();

        for x in 0..=4 {
            world.set(Pos::new(x, 0, 0), stone);
        }
        world.set(Pos::new(0, 1, 0), block);
        for x in 1..=4 {
            world.set(Pos::new(x, 1, 0), network.wire[0]);
        }
        network.settle(&mut world);

        world.set(Pos::new(0, 1, 0), StateId::AIR);
        let changed = network.settle(&mut world);

        assert_eq!(changed, 4, "every powered wire must fall in one settle");
        for x in 1..=4 {
            assert_eq!(
                network.power_of(world.get(Pos::new(x, 1, 0))),
                Some(0),
                "wire at x={x} must be unpowered"
            );
        }
    }

    #[test]
    fn power_runs_out_after_fifteen_blocks() {
        let (_states, network, block, stone) = setup();
        let mut world = World::new(Bounds::new(Pos::new(-2, 0, -2), Pos::new(20, 4, 2)));

        for x in 0..=18 {
            world.set(Pos::new(x, 0, 0), stone);
        }
        world.set(Pos::new(0, 1, 0), block);
        for x in 1..=18 {
            world.set(Pos::new(x, 1, 0), network.wire[0]);
        }
        network.settle(&mut world);

        let power_at = |x: i32| network.power_of(world.get(Pos::new(x, 1, 0))).unwrap();
        assert_eq!(power_at(15), 1, "fifteen blocks out still carries 1");
        assert_eq!(power_at(16), 0, "sixteen is out of range");
        assert_eq!(power_at(18), 0);
    }

    #[test]
    fn settling_is_idempotent() {
        // A second settle with nothing changed must be a no-op, or the engine would
        // never reach quiescence and every timing would run forever.
        let (_states, network, block, stone) = setup();
        let mut world = world();

        world.set(Pos::new(0, 0, 0), stone);
        world.set(Pos::new(0, 1, 0), block);
        world.set(Pos::new(1, 1, 0), network.wire[0]);

        assert!(network.settle(&mut world) > 0);
        assert_eq!(network.settle(&mut world), 0, "second settle must change nothing");
    }

    #[test]
    fn dust_with_no_source_stays_dark() {
        let (_states, network, _block, stone) = setup();
        let mut world = world();
        world.set(Pos::new(1, 0, 0), stone);
        world.set(Pos::new(1, 1, 0), network.wire[7]);

        network.settle(&mut world);
        assert_eq!(network.power_of(world.get(Pos::new(1, 1, 0))), Some(0));
    }

    #[test]
    fn two_sources_give_a_wire_the_stronger_signal() {
        let (_states, network, block, stone) = setup();
        let mut world = world();
        for x in 0..=6 {
            world.set(Pos::new(x, 0, 0), stone);
        }
        // Sources at both ends; the middle wire should take whichever is stronger.
        world.set(Pos::new(0, 1, 0), block);
        world.set(Pos::new(6, 1, 0), block);
        for x in 1..=5 {
            world.set(Pos::new(x, 1, 0), network.wire[0]);
        }
        network.settle(&mut world);

        let power_at = |x: i32| network.power_of(world.get(Pos::new(x, 1, 0))).unwrap();
        assert_eq!(power_at(1), 15);
        assert_eq!(power_at(5), 15);
        // Middle is three from each source: 15 - 2 = 13 either way.
        assert_eq!(power_at(3), 13);
    }
}
