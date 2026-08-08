//! Build a [`Simulation`] directly from host-provided blocks — no
//! [`Structure`](crate::structure::Structure) file, no SNBT round-trip.
//!
//! This is the embedding seam for live hosts (a running server, an editor)
//! whose world already exists in memory: feed `(pos, descriptor)` pairs from
//! any source and get a fully wired engine back — interning, behaviours,
//! physics/fluid/rail tables, block-entity tickers, and an initial `record()`
//! all handled, matching the wiring the bridge performs for structures.
//! Settle semantics are "in world": blocks stand as found, nothing re-runs
//! `onPlace` at build time.
//!
//! ```no_run
//! use mc_tick::embed::SimulationBuilder;
//! use mc_tick::pos::{Bounds, Pos};
//!
//! let mut b = SimulationBuilder::new(Bounds::new(
//!     Pos::new(-4, -4, -4),
//!     Pos::new(20, 12, 20),
//! ));
//! b.set_block(Pos::new(0, 0, 0), "minecraft:lever[face=floor,facing=east,powered=false]");
//! let mut sim = b.build().expect("known blocks only");
//! sim.step();
//! ```

use crate::pos::{Bounds, Pos};
use crate::sim::Simulation;
use crate::state::StateId;

/// Incrementally assembled input for a schematic-free [`Simulation`].
pub struct SimulationBuilder {
    bounds: Bounds,
    blocks: Vec<(Pos, String)>,
    extra_descriptors: Vec<String>,
}

impl SimulationBuilder {
    /// A builder for a simulation covering `bounds` (engine coordinates —
    /// the host owns any world-to-engine offset).
    #[must_use]
    pub fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            blocks: Vec::new(),
            extra_descriptors: vec!["minecraft:redstone_block".to_string()],
        }
    }

    /// Record a non-air block. `descriptor` uses the canonical
    /// `minecraft:name[key=value,…]` form; air cells are simply omitted.
    pub fn set_block(&mut self, pos: Pos, descriptor: &str) {
        self.blocks.push((pos, descriptor.to_string()));
    }

    /// Pre-intern an extra descriptor the running host may hand the engine
    /// later (e.g. blocks a player might place after the build).
    pub fn intern_later(&mut self, descriptor: &str) {
        self.extra_descriptors.push(descriptor.to_string());
    }

    /// Wire and return the engine. Fails — by design, correctness over
    /// coverage — if any descriptor has no registered behaviour.
    pub fn build(self) -> Result<Simulation, String> {
        let mut sim = Simulation::new(self.bounds);

        for descriptor in self
            .blocks
            .iter()
            .map(|(_, d)| d.as_str())
            .chain(self.extra_descriptors.iter().map(String::as_str))
        {
            sim.registry_mut()
                .intern(descriptor)
                .map_err(|e| format!("interning {descriptor}: {e:?}"))?;
        }

        {
            let (registry, world) = sim.registry_and_world_mut();
            for (pos, descriptor) in &self.blocks {
                if let Some(state) = registry.get(descriptor) {
                    world.set(*pos, state);
                }
            }
        }

        crate::vanilla::intern_companions(sim.registry_mut());
        {
            let mut table = std::mem::take(sim.behaviours_mut());
            crate::vanilla::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
            *sim.behaviours_mut() = table;
        }
        if let Some(report) = sim.unknown_report() {
            return Err(format!("blocks without behaviour: {report}"));
        }
        {
            let (solidity, frictions, heights, webs) =
                crate::vanilla::physics_tables(sim.registry());
            sim.set_physics_tables(solidity, frictions, heights, webs);
            let (water_kinds, bubble_kinds) = crate::vanilla::fluid_tables(sim.registry());
            sim.set_fluid_tables(water_kinds, bubble_kinds);
            let (rails, conductors) = crate::vanilla::rail_tables(sim.registry());
            sim.set_rail_tables(rails, conductors);
        }

        // Blocks whose behaviour ticks as a block entity get their tickers;
        // hosts feeding live worlds have no authored NBT to consult.
        let tickers: Vec<Pos> = self
            .blocks
            .iter()
            .filter_map(|(pos, descriptor)| {
                let state = sim.registry().get(descriptor)?;
                let ticks = sim
                    .behaviours()
                    .get(state)
                    .is_some_and(|b| b.ticks_as_block_entity());
                ticks.then_some(*pos)
            })
            .collect();
        for pos in tickers {
            sim.mark_block_entity(pos);
            sim.add_block_entity_ticker(pos);
        }

        sim.record();
        Ok(sim)
    }
}

/// Convenience for hosts syncing a live world: canonical break = placing air
/// by hand (shape updates run, no `onPlace`).
pub fn break_block_by_hand(sim: &mut Simulation, pos: Pos) {
    sim.place_block_by_hand(pos, StateId::AIR);
}
