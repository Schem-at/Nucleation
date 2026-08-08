//! `SimBackend`: one drive/settle/read contract over interchangeable
//! simulation engines (design: `ROUTING_CRATE_DESIGN.md`, "Unifying
//! CellTemplate with TypedCircuitExecutor + Insign", move 3).
//!
//! Two backends implement it:
//!
//! - [`MchprsBackend`] — today's fast path: the MCHPRS compile graph with
//!   custom-IO *signal injection* (`set_signals_batch`). Values are pushed
//!   straight into the graph; `is_lit` latching and interning quirks are
//!   owned here, not by callers.
//! - [`McTickBackend`] — the vanilla-accurate oracle (crates/mc-tick).
//!   **mc-tick has no signal injection**: every input bit must be a lever
//!   block at the mapped position, and `drive` is the Levers idiom ported
//!   from the rca-4bit sessions — read the lever's current state, toggle
//!   only the bits that differ (`use_block`), and settle after every flip
//!   so each toggle propagates before the next (toggle-to-target,
//!   settle-per-flip). Driving a non-lever position is an error, not a
//!   silent no-op.
//!
//! [`BackendCircuitExecutor`] (entry point:
//! [`TypedCircuitExecutor::with_backend`]) reuses the typed word
//! encode/decode from [`IoMapping`] unchanged on either backend. The
//! existing `TypedCircuitExecutor` MCHPRS path is untouched.

use super::{IoLayout, IoMapping, Value};
use crate::simulation::{MchprsWorld, SimulationOptions};
use crate::UniversalSchematic;
use std::collections::HashMap;

/// A simulation engine the typed executor can run on.
///
/// Positions are schematic-space `(x, y, z)` — each backend owns its own
/// coordinate mapping. Values are signal strengths 0-15 per position
/// ("nibbles"), the same physical layer `IoMapping` encodes to.
pub trait SimBackend {
    /// Drive the given positions to the given signal strengths.
    ///
    /// Injection backends write the values directly; lever backends
    /// toggle levers toward the target (a nibble > 0 means "on") and may
    /// settle internally per flip.
    fn drive(&mut self, positions: &[(i32, i32, i32)], nibbles: &[u8]) -> Result<(), String>;

    /// Let the simulation propagate, spending at most `budget` ticks.
    /// Returns `true` when the world is quiescent (or the backend has no
    /// quiescence notion and completed the budget).
    fn settle(&mut self, budget: u32) -> bool;

    /// Read the signal strengths at the given positions.
    fn read(&mut self, positions: &[(i32, i32, i32)]) -> Result<Vec<u8>, String>;

    /// Rebuild the simulation from the originally loaded schematic.
    fn reset(&mut self) -> Result<(), String>;

    /// Write the settled world state back into `schem` so the saved file
    /// carries real wire connections and power. Returns how many blocks
    /// changed.
    fn bake_to(&mut self, schem: &mut UniversalSchematic) -> Result<u32, String>;
}

/// The MCHPRS compile-graph backend: fast, with custom-IO signal injection.
pub struct MchprsBackend {
    world: MchprsWorld,
    original: UniversalSchematic,
    options: SimulationOptions,
}

impl MchprsBackend {
    /// Load a schematic with the given simulation options. Positions the
    /// executor will drive or read must already be listed in
    /// `options.custom_io`.
    pub fn load(schem: UniversalSchematic, options: SimulationOptions) -> Result<Self, String> {
        let world = MchprsWorld::with_options(schem.clone(), options.clone())?;
        Ok(MchprsBackend {
            world,
            original: schem,
            options,
        })
    }

    /// Load a schematic registering every position of `layout` as custom IO.
    pub fn for_layout(schem: UniversalSchematic, layout: &IoLayout) -> Result<Self, String> {
        let mut options = SimulationOptions::default();
        for mapping in layout.inputs.values().chain(layout.outputs.values()) {
            for &(x, y, z) in &mapping.positions {
                let pos = crate::simulation::BlockPos::new(x, y, z);
                if !options.custom_io.contains(&pos) {
                    options.custom_io.push(pos);
                }
            }
        }
        Self::load(schem, options)
    }

    /// The wrapped world (for advanced use).
    pub fn world_mut(&mut self) -> &mut MchprsWorld {
        &mut self.world
    }
}

impl SimBackend for MchprsBackend {
    fn drive(&mut self, positions: &[(i32, i32, i32)], nibbles: &[u8]) -> Result<(), String> {
        self.world.set_signals_batch(positions, nibbles)?;
        self.world.flush();
        Ok(())
    }

    fn settle(&mut self, budget: u32) -> bool {
        // MCHPRS has no quiescence signal; a budget of ticks plus a flush
        // is the settle discipline every existing caller uses.
        self.world.flush();
        self.world.tick(budget);
        self.world.flush();
        true
    }

    fn read(&mut self, positions: &[(i32, i32, i32)]) -> Result<Vec<u8>, String> {
        // Flush first: block states (lamps) lag the compile graph otherwise.
        self.world.flush();
        Ok(self.world.get_signals_batch(positions))
    }

    fn reset(&mut self) -> Result<(), String> {
        self.world = MchprsWorld::with_options(self.original.clone(), self.options.clone())?;
        Ok(())
    }

    fn bake_to(&mut self, schem: &mut UniversalSchematic) -> Result<u32, String> {
        self.world.sync_to_schematic();
        let mut changed = 0u32;
        for (bp, bs) in self.world.get_schematic().iter_blocks() {
            let rendered = bs.to_string();
            if schem
                .get_block(bp.x, bp.y, bp.z)
                .is_some_and(|current| current.to_string() == rendered)
            {
                continue;
            }
            schem.set_block_from_string(bp.x, bp.y, bp.z, &rendered)?;
            changed += 1;
        }
        Ok(changed)
    }
}

/// The vanilla-accurate mc-tick backend. No signal injection: inputs are
/// levers, driven with toggle-to-target + settle-per-flip (see module docs).
#[cfg(all(feature = "bridge", feature = "mc-tick"))]
pub struct McTickBackend {
    sim: mc_tick::Simulation,
    original: UniversalSchematic,
    extra_states: Vec<String>,
    /// Schematic-space position of the simulation's `(0, 0, 0)` (the
    /// schematic bounding-box minimum, exactly as `TickSimulation::
    /// from_schematic` maps it).
    offset: (i32, i32, i32),
    /// Tick budget spent settling after each individual lever flip.
    per_flip_budget: u32,
}

#[cfg(all(feature = "bridge", feature = "mc-tick"))]
impl McTickBackend {
    /// Load a schematic into a fresh mc-tick simulation (InWorld settle:
    /// trust the saved block states). `extra_states` follows the
    /// `TickSimulation` contract: any state the run may create that is not
    /// in the build's own palette must be named up front.
    pub fn load(schem: UniversalSchematic, extra_states: &[&str]) -> Result<Self, String> {
        let sim = Self::build_sim(&schem, extra_states)?;
        let bb = schem.get_bounding_box();
        Ok(McTickBackend {
            sim,
            offset: bb.min,
            original: schem,
            extra_states: extra_states.iter().map(|s| s.to_string()).collect(),
            per_flip_budget: 64,
        })
    }

    /// Change the per-flip settle budget (default 64 ticks).
    pub fn set_per_flip_budget(&mut self, ticks: u32) {
        self.per_flip_budget = ticks;
    }

    fn build_sim(
        schem: &UniversalSchematic,
        extra_states: &[&str],
    ) -> Result<mc_tick::Simulation, String> {
        let snbt = crate::formats::gametest::to_gametest_snbt(schem);
        let structure =
            mc_tick::Structure::parse(&snbt).map_err(|e| format!("structure parse: {e:?}"))?;
        crate::bridge::mc_tick::wire_simulation(
            &structure,
            mc_tick::Pos::new(0, 0, 0),
            crate::bridge::mc_tick::ffi::TickSettleMode::InWorld,
            extra_states,
            schem.metadata.source_data_version,
        )
    }

    fn sim_pos(&self, (x, y, z): (i32, i32, i32)) -> mc_tick::Pos {
        mc_tick::Pos::new(x - self.offset.0, y - self.offset.1, z - self.offset.2)
    }

    fn descriptor_at(&self, p: mc_tick::Pos) -> &str {
        let id = self.sim.world().get(p);
        self.sim.registry().descriptor(id).unwrap_or("minecraft:air")
    }

    /// Signal strength a settled block state reports: wire `power=N`,
    /// otherwise `powered=true` / `lit=true` count as full strength.
    fn power_of(descriptor: &str) -> u8 {
        if let Some(idx) = descriptor.find("power=") {
            let tail = &descriptor[idx + "power=".len()..];
            let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
            return digits.parse().unwrap_or(0);
        }
        if descriptor.contains("powered=true") || descriptor.contains("lit=true") {
            15
        } else {
            0
        }
    }
}

#[cfg(all(feature = "bridge", feature = "mc-tick"))]
impl SimBackend for McTickBackend {
    fn drive(&mut self, positions: &[(i32, i32, i32)], nibbles: &[u8]) -> Result<(), String> {
        if positions.len() != nibbles.len() {
            return Err(format!(
                "position count ({}) does not match value count ({})",
                positions.len(),
                nibbles.len()
            ));
        }
        for (&pos, &nib) in positions.iter().zip(nibbles) {
            let p = self.sim_pos(pos);
            let descriptor = self.descriptor_at(p);
            if !descriptor.contains("lever") {
                return Err(format!(
                    "mc-tick has no signal injection: input bit at ({}, {}, {}) must be a \
                     lever, found `{descriptor}`",
                    pos.0, pos.1, pos.2
                ));
            }
            let current = descriptor.contains("powered=true");
            let target = nib > 0;
            if current != target {
                // Toggle-to-target: flip only when the state differs, and
                // settle after every flip so the next read sees the truth.
                self.sim.use_block(p);
                self.sim.run_until_quiescent(u64::from(self.per_flip_budget));
            }
        }
        Ok(())
    }

    fn settle(&mut self, budget: u32) -> bool {
        matches!(
            self.sim.run_until_quiescent(u64::from(budget)),
            mc_tick::StopReason::Quiescent
        ) || self.sim.is_quiescent()
    }

    fn read(&mut self, positions: &[(i32, i32, i32)]) -> Result<Vec<u8>, String> {
        Ok(positions
            .iter()
            .map(|&pos| Self::power_of(self.descriptor_at(self.sim_pos(pos))))
            .collect())
    }

    fn reset(&mut self) -> Result<(), String> {
        let extras: Vec<&str> = self.extra_states.iter().map(|s| s.as_str()).collect();
        self.sim = Self::build_sim(&self.original, &extras)?;
        Ok(())
    }

    fn bake_to(&mut self, schem: &mut UniversalSchematic) -> Result<u32, String> {
        Ok(crate::bridge::mc_tick::bake_into(&self.sim, schem))
    }
}

/// A typed executor generic over [`SimBackend`]: the same word
/// encode/decode as [`super::TypedCircuitExecutor`], on whichever engine
/// was loaded. Create via [`TypedCircuitExecutor::with_backend`].
///
/// [`TypedCircuitExecutor::with_backend`]: super::TypedCircuitExecutor::with_backend
pub struct BackendCircuitExecutor {
    backend: Box<dyn SimBackend>,
    inputs: HashMap<String, IoMapping>,
    outputs: HashMap<String, IoMapping>,
}

impl BackendCircuitExecutor {
    /// Wrap a loaded backend with an IO layout.
    pub fn new(backend: Box<dyn SimBackend>, layout: IoLayout) -> Self {
        BackendCircuitExecutor {
            backend,
            inputs: layout.inputs,
            outputs: layout.outputs,
        }
    }

    /// Encode `value` and drive the named input's positions.
    pub fn set_input(&mut self, name: &str, value: &Value) -> Result<(), String> {
        let mapping = self
            .inputs
            .get(name)
            .ok_or_else(|| format!("Unknown input: {name}"))?;
        let nibbles = mapping.encode(value)?;
        self.backend.drive(&mapping.positions, &nibbles)
    }

    /// Settle the world for at most `budget` ticks; `true` = quiescent.
    pub fn settle(&mut self, budget: u32) -> bool {
        self.backend.settle(budget)
    }

    /// Read and decode the named output.
    pub fn read_output(&mut self, name: &str) -> Result<Value, String> {
        let mapping = self
            .outputs
            .get(name)
            .ok_or_else(|| format!("Unknown output: {name}"))?;
        let nibbles = self.backend.read(&mapping.positions)?;
        mapping.decode(&nibbles)
    }

    /// Drive all `inputs`, settle within `budget`, read all outputs.
    pub fn execute(
        &mut self,
        inputs: &HashMap<String, Value>,
        budget: u32,
    ) -> Result<HashMap<String, Value>, String> {
        for (name, value) in inputs {
            self.set_input(name, value)?;
        }
        self.settle(budget);
        let names: Vec<String> = self.outputs.keys().cloned().collect();
        let mut out = HashMap::new();
        for name in names {
            let v = self.read_output(&name)?;
            out.insert(name, v);
        }
        Ok(out)
    }

    /// Rebuild the simulation from the originally loaded schematic.
    pub fn reset(&mut self) -> Result<(), String> {
        self.backend.reset()
    }

    /// Bake the settled state back into `schem`.
    pub fn bake_to(&mut self, schem: &mut UniversalSchematic) -> Result<u32, String> {
        self.backend.bake_to(schem)
    }

    /// The backend, for engine-specific control.
    pub fn backend_mut(&mut self) -> &mut dyn SimBackend {
        self.backend.as_mut()
    }
}

impl super::TypedCircuitExecutor {
    /// Run the typed IO contract on an explicit [`SimBackend`] — the
    /// constructor-choice seam from the design doc. The classic
    /// `TypedCircuitExecutor` (MCHPRS-only, execution modes, state modes)
    /// is unchanged; this returns the backend-generic executor instead.
    pub fn with_backend(backend: Box<dyn SimBackend>, layout: IoLayout) -> BackendCircuitExecutor {
        BackendCircuitExecutor::new(backend, layout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io_contract::{IoLayoutBuilder, IoType, LayoutFunction};

    /// stone floor + dust line from (0,1,0) to (2,1,0).
    fn dust_line() -> UniversalSchematic {
        let mut schem = UniversalSchematic::new("backend_line".into());
        for x in 0..=2 {
            schem
                .set_block_from_string(x, 0, 0, "minecraft:stone")
                .unwrap();
            schem
                .set_block_from_string(
                    x,
                    1,
                    0,
                    "minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]",
                )
                .unwrap();
        }
        schem
    }

    fn line_layout() -> IoLayout {
        IoLayoutBuilder::new()
            .add_input(
                "a",
                IoType::UnsignedInt { bits: 1 },
                LayoutFunction::OneToOne,
                vec![(0, 1, 0)],
            )
            .unwrap()
            .add_output(
                "y",
                IoType::UnsignedInt { bits: 1 },
                LayoutFunction::OneToOne,
                vec![(2, 1, 0)],
            )
            .unwrap()
            .build()
    }

    #[test]
    fn mchprs_backend_drives_and_reads_through_the_trait() {
        let schem = dust_line();
        let layout = line_layout();
        let backend = MchprsBackend::for_layout(schem, &layout).unwrap();
        let mut exec = super::super::TypedCircuitExecutor::with_backend(Box::new(backend), layout);

        let mut inputs = HashMap::new();
        inputs.insert("a".to_string(), Value::U32(1));
        let out = exec.execute(&inputs, 10).unwrap();
        assert_eq!(out["y"], Value::U32(1), "signal reaches the far dust");

        inputs.insert("a".to_string(), Value::U32(0));
        let out = exec.execute(&inputs, 10).unwrap();
        assert_eq!(out["y"], Value::U32(0), "and clears again");
    }

    #[test]
    fn mchprs_backend_bakes_back_into_a_schematic() {
        let schem = dust_line();
        let layout = line_layout();
        let mut backend = MchprsBackend::for_layout(schem.clone(), &layout).unwrap();
        backend.drive(&[(0, 1, 0)], &[15]).unwrap();
        backend.settle(10);
        let mut baked = schem.clone();
        backend.bake_to(&mut baked).unwrap();
        let wire = baked.get_block(1, 1, 0).unwrap().to_string();
        assert!(wire.contains("redstone_wire"), "{wire}");
    }

    #[cfg(all(feature = "bridge", feature = "mc-tick"))]
    mod mc_tick_backend {
        use super::*;

        /// lever on stone, dust beside it: the smallest drivable circuit.
        fn lever_and_dust() -> UniversalSchematic {
            let mut schem = UniversalSchematic::new("lever_dust".into());
            schem
                .set_block_from_string(0, 0, 0, "minecraft:stone")
                .unwrap();
            schem
                .set_block_from_string(1, 0, 0, "minecraft:stone")
                .unwrap();
            schem
                .set_block_from_string(
                    0,
                    1,
                    0,
                    "minecraft:lever[face=floor,facing=north,powered=false]",
                )
                .unwrap();
            schem
                .set_block_from_string(
                    1,
                    1,
                    0,
                    "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=side]",
                )
                .unwrap();
            schem
        }

        fn lever_layout() -> IoLayout {
            IoLayoutBuilder::new()
                .add_input(
                    "a",
                    IoType::Boolean,
                    LayoutFunction::OneToOne,
                    vec![(0, 1, 0)],
                )
                .unwrap()
                .add_output(
                    "y",
                    IoType::Boolean,
                    LayoutFunction::OneToOne,
                    vec![(1, 1, 0)],
                )
                .unwrap()
                .build()
        }

        #[test]
        fn lever_toggle_to_target_drives_the_wire() {
            let backend = McTickBackend::load(lever_and_dust(), &[]).unwrap();
            let mut exec = crate::simulation::typed_executor::TypedCircuitExecutor::with_backend(
                Box::new(backend),
                lever_layout(),
            );
            let mut inputs = HashMap::new();
            inputs.insert("a".to_string(), Value::Bool(true));
            let out = exec.execute(&inputs, 64).unwrap();
            assert_eq!(out["y"], Value::Bool(true), "lever powers the dust");

            // Driving the SAME value again must be a no-op (toggle-to-target,
            // not blind toggling): the output holds.
            let out = exec.execute(&inputs, 64).unwrap();
            assert_eq!(out["y"], Value::Bool(true), "idempotent re-drive");

            inputs.insert("a".to_string(), Value::Bool(false));
            let out = exec.execute(&inputs, 64).unwrap();
            assert_eq!(out["y"], Value::Bool(false), "and back off");
        }

        #[test]
        fn driving_a_non_lever_position_is_an_error() {
            let mut backend = McTickBackend::load(lever_and_dust(), &[]).unwrap();
            let err = backend.drive(&[(1, 1, 0)], &[15]).unwrap_err();
            assert!(err.contains("no signal injection"), "{err}");
        }

        #[test]
        fn bake_to_writes_settled_lever_state_home() {
            let schem = lever_and_dust();
            let mut backend = McTickBackend::load(schem.clone(), &[]).unwrap();
            backend.drive(&[(0, 1, 0)], &[15]).unwrap();
            backend.settle(64);
            let mut baked = schem.clone();
            let changed = backend.bake_to(&mut baked).unwrap();
            assert!(changed > 0, "settled states differ from the authored file");
            let lever = baked.get_block(0, 1, 0).unwrap().to_string();
            assert!(lever.contains("powered=true"), "{lever}");
        }
    }
}
