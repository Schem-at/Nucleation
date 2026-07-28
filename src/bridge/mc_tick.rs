//! mc-tick: the vanilla-accurate, headless tick engine.
//!
//! New surface — no old `ffi/` counterpart. An opaque [`ffi::TickSimulation`]
//! wraps `mc_tick::Simulation` with the full wiring recipe the engine's own
//! conformance tests use (inventories, behaviours, physics/fluid/rail tables,
//! entities, block-entity tickers), so any schematic that loads runs exactly
//! as the Rust test harness would run it.
//!
//! Design notes:
//! - Everything is headless — no rendering feature involved. Hosts pull
//!   per-tick JSON logs out and compute stats/animations themselves.
//! - Behaviours bind to *interned* states at construction, so any state a
//!   later `place_block` will write (a redstone block, typically) must be
//!   named up front: the constructors take a semicolon-separated
//!   `extra_states` list. `minecraft:redstone_block` and every facing of any
//!   shulker box held as an item are always pre-interned.
//! - Structured data crosses as JSON strings (PORTING.md rule 9).

/// Render a schematic as vanilla gametest structure SNBT — the flavor
/// `mc_tick::Structure::parse` reads (`palette` + indexed `blocks` +
/// bracketless `Properties`, block-entity `nbt` inline). The
/// `formats::structure_snbt` exporter emits the *data-flavor* instead
/// (inline `state:"id{k:v}"` strings), which mc-tick rejects — so this
/// builds the gametest flavor directly and keeps mc-tick's proven parser
/// as the single reader.
fn to_gametest_snbt(schematic: &crate::UniversalSchematic) -> String {
    use std::collections::HashMap;
    use std::fmt::Write as _;

    let bb = schematic.get_bounding_box();
    let (mx, my, mz) = bb.min;
    let size = (bb.max.0 - mx + 1, bb.max.1 - my + 1, bb.max.2 - mz + 1);

    let mut nbt_at: HashMap<(i32, i32, i32), String> = HashMap::new();
    for be in schematic.get_block_entities_as_list() {
        let snbt = quartz_nbt::NbtTag::Compound(be.nbt.to_quartz_nbt()).to_snbt();
        nbt_at.insert(be.position, snbt);
    }

    let mut palette: Vec<String> = Vec::new();
    let mut palette_index: HashMap<String, usize> = HashMap::new();
    let mut blocks = String::new();
    for (pos, state) in schematic.iter_blocks() {
        if state.name == "minecraft:air" {
            continue;
        }
        let mut props: Vec<(&str, &str)> =
            state.properties.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        props.sort();
        let mut entry = format!("{{Name:\"{}\"", state.name);
        if !props.is_empty() {
            entry.push_str(", Properties:{");
            for (i, (k, v)) in props.iter().enumerate() {
                if i > 0 {
                    entry.push_str(", ");
                }
                let _ = write!(entry, "{k}: \"{v}\"");
            }
            entry.push('}');
        }
        entry.push('}');
        let index = *palette_index.entry(entry.clone()).or_insert_with(|| {
            palette.push(entry);
            palette.len() - 1
        });
        if !blocks.is_empty() {
            blocks.push_str(",\n    ");
        }
        let _ = write!(
            blocks,
            "{{pos: [{}, {}, {}], state: {}",
            pos.x - mx,
            pos.y - my,
            pos.z - mz
        , index);
        if let Some(nbt) = nbt_at.get(&(pos.x, pos.y, pos.z)) {
            let _ = write!(blocks, ", nbt: {nbt}");
        }
        blocks.push('}');
    }

    format!(
        "{{\n  DataVersion: 4903,\n  size: [{}, {}, {}],\n  palette: [\n    {}\n  ],\n  blocks: [\n    {}\n  ],\n  entities: []\n}}\n",
        size.0,
        size.1,
        size.2,
        palette.join(",\n    "),
        blocks
    )
}

/// The settle recipe, mirroring the engine's conformance harness.
fn wire_simulation(
    structure: &mc_tick::Structure,
    hash_origin: mc_tick::Pos,
    settle: ffi::TickSettleMode,
    extra_states: &[&str],
) -> Result<mc_tick::Simulation, String> {
    use mc_tick::{Pos, Simulation};
    const MARGIN: i32 = 4;

    let mut sim = Simulation::new(structure.bounds(MARGIN));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    // The universal actuator, plus anything the caller names.
    let mut wanted: Vec<String> = vec!["minecraft:redstone_block".to_string()];
    wanted.extend(extra_states.iter().map(|s| s.to_string()));
    // A dispenser can *place* a shulker box it holds as an item; behaviours
    // bind only to interned states, so intern every facing up front.
    for (_, stacks) in &structure.inventories {
        for stack in stacks {
            let base = stack.id.split('[').next().unwrap_or(&stack.id);
            if base.ends_with("_shulker_box") || base == "minecraft:shulker_box" {
                for facing in ["up", "down", "north", "south", "west", "east"] {
                    wanted.push(format!("{base}[facing={facing}]"));
                }
            }
        }
    }
    for descriptor in &wanted {
        sim.registry_mut()
            .intern(descriptor)
            .map_err(|e| format!("interning {descriptor}: {e:?}"))?;
    }
    for pos in &structure.block_entities {
        sim.mark_block_entity(*pos);
    }
    for (pos, strength) in &structure.comparator_outputs {
        sim.set_comparator_output(*pos, *strength);
    }
    for (pos, stacks) in &structure.inventories {
        let entry = structure
            .blocks
            .iter()
            .find(|(p, _)| p == pos)
            .map(|(_, e)| *e)
            .ok_or_else(|| format!("inventory at {pos:?} with no block"))?;
        let name = structure.palette[entry].split('[').next().unwrap_or_default().to_string();
        let slots = mc_tick::vanilla::container_slots(&name)
            .ok_or_else(|| format!("{name} has an inventory but no slot count"))?;
        sim.set_inventory(*pos, mc_tick::Inventory { slots, stacks: stacks.clone() });
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, hash_origin);
        *sim.behaviours_mut() = table;
    }
    if let Some(report) = sim.unknown_report() {
        return Err(format!("blocks without behaviour: {report}"));
    }
    {
        let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
        sim.set_physics_tables(solidity, frictions, heights, webs);
        let (water_kinds, bubble_kinds) = mc_tick::vanilla::fluid_tables(sim.registry());
        sim.set_fluid_tables(water_kinds, bubble_kinds);
        let (rails, conductors) = mc_tick::vanilla::rail_tables(sim.registry());
        sim.set_rail_tables(rails, conductors);
    }
    for spawned in &structure.entities {
        match spawned {
            mc_tick::structure::SpawnedEntity::Item(item) => {
                sim.spawn_item(item.item.clone(), item.pos, item.motion, item.pickup_delay);
            }
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                sim.spawn_minecart(cart.kind.clone(), cart.pos, cart.motion);
            }
        }
    }
    for (pos, entry) in &structure.blocks {
        let state = sim.registry().get(&structure.palette[*entry]);
        let is_ticker = state
            .and_then(|s| sim.behaviours().get(s))
            .is_some_and(|b| b.ticks_as_block_entity());
        if is_ticker {
            sim.add_block_entity_ticker(*pos);
        }
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    if settle != ffi::TickSettleMode::InWorld {
        sim.place_on_place(&order);
    }
    if settle == ffi::TickSettleMode::Placement {
        sim.settle_with_order(&order);
    }
    sim.record();
    Ok(sim)
}

fn is_named(descriptor: &str, needle: &str) -> bool {
    descriptor
        .split('[')
        .next()
        .unwrap_or(descriptor)
        .contains(needle)
}

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::{DiplomatStr, DiplomatWrite};
    use std::fmt::Write;

    /// How the loaded structure is settled before tick 0.
    #[derive(PartialEq, Eq)]
    pub enum TickSettleMode {
        /// Vanilla placement pass + ordered settle — a build saved at rest.
        Placement,
        /// `onPlace` only, no settle — a knownShape capture.
        Quiet,
        /// Neither — a build recorded mid-state in the world it stood in.
        InWorld,
    }

    /// A headless, vanilla-accurate tick simulation of one structure.
    #[diplomat::opaque_mut]
    pub struct TickSimulation {
        pub(crate) sim: mc_tick::Simulation,
        pub(crate) checkpoints: Vec<mc_tick::sim::Checkpoint>,
    }

    impl TickSimulation {
        /// Load from Java structure SNBT text.
        ///
        /// `extra_states`: semicolon-separated block-state descriptors that
        /// later `place_block` calls may write (behaviours bind at
        /// construction). `minecraft:redstone_block` is always available.
        /// `origin_*`: where the build's (0,0,0) sits in world coordinates —
        /// wire update order hashes absolute positions.
        pub fn from_snbt(
            snbt: &DiplomatStr,
            settle: TickSettleMode,
            origin_x: i32,
            origin_y: i32,
            origin_z: i32,
            extra_states: &DiplomatStr,
        ) -> Result<Box<TickSimulation>, NucleationError> {
            let snbt =
                std::str::from_utf8(snbt).map_err(|_| NucleationError::InvalidArgument)?;
            let extra =
                std::str::from_utf8(extra_states).map_err(|_| NucleationError::InvalidArgument)?;
            let structure =
                mc_tick::Structure::parse(snbt).map_err(|_| NucleationError::Parse)?;
            let extras: Vec<&str> =
                extra.split(';').map(str::trim).filter(|s| !s.is_empty()).collect();
            let sim = super::wire_simulation(
                &structure,
                mc_tick::Pos::new(origin_x, origin_y, origin_z),
                settle,
                &extras,
            )
            .map_err(|_| NucleationError::Simulation)?;
            Ok(Box::new(TickSimulation { sim, checkpoints: Vec::new() }))
        }

        /// Load from a schematic (any format nucleation can read), rendered
        /// to gametest-flavor structure SNBT for mc-tick's parser.
        pub fn from_schematic(
            schematic: &Schematic,
            settle: TickSettleMode,
            origin_x: i32,
            origin_y: i32,
            origin_z: i32,
            extra_states: &DiplomatStr,
        ) -> Result<Box<TickSimulation>, NucleationError> {
            let snbt = super::to_gametest_snbt(&schematic.0);
            let extra =
                std::str::from_utf8(extra_states).map_err(|_| NucleationError::InvalidArgument)?;
            let structure =
                mc_tick::Structure::parse(&snbt).map_err(|_| NucleationError::Parse)?;
            let extras: Vec<&str> =
                extra.split(';').map(str::trim).filter(|s| !s.is_empty()).collect();
            let sim = super::wire_simulation(
                &structure,
                mc_tick::Pos::new(origin_x, origin_y, origin_z),
                settle,
                &extras,
            )
            .map_err(|_| NucleationError::Simulation)?;
            Ok(Box::new(TickSimulation { sim, checkpoints: Vec::new() }))
        }

        /// Seed the vanilla random source (`java.util.Random`'s LCG,
        /// bit-for-bit). Unseeded, jittering behaviours use each
        /// distribution's mean — fully deterministic, no noise.
        pub fn set_rng_seed(&mut self, seed: i64) {
            self.sim.set_rng_seed(seed);
        }

        /// Advance one game tick.
        pub fn step(&mut self) {
            self.sim.step();
        }

        /// Advance `ticks` game ticks.
        pub fn run(&mut self, ticks: u32) {
            self.sim.run(u64::from(ticks));
        }

        /// Run until nothing is scheduled or `budget` ticks pass. Returns
        /// whether the world went quiet.
        pub fn run_until_quiescent(&mut self, budget: u32) -> bool {
            self.sim.run_until_quiescent(u64::from(budget));
            self.sim.is_quiescent()
        }

        /// Game ticks elapsed since settle.
        pub fn tick_count(&self) -> u32 {
            self.sim.tick_count() as u32
        }

        /// Whether nothing is scheduled or queued.
        pub fn is_quiescent(&self) -> bool {
            self.sim.is_quiescent()
        }

        /// Right-click a block with an empty hand (lever, button, note block).
        pub fn use_block(&mut self, x: i32, y: i32, z: i32) {
            self.sim.use_block(mc_tick::Pos::new(x, y, z));
        }

        /// Write a block state (`minecraft:air` breaks). The state must be in
        /// the structure, in `extra_states`, or `minecraft:redstone_block`.
        pub fn place_block(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            state: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let state =
                std::str::from_utf8(state).map_err(|_| NucleationError::InvalidArgument)?;
            let id = self
                .sim
                .registry()
                .get(state)
                .ok_or(NucleationError::NotFound)?;
            self.sim.place_block(mc_tick::Pos::new(x, y, z), id);
            Ok(())
        }

        /// The block state descriptor at a position (`minecraft:air` for empty).
        pub fn get_block(&self, x: i32, y: i32, z: i32, out: &mut DiplomatWrite) {
            let id = self.sim.world().get(mc_tick::Pos::new(x, y, z));
            let descriptor = self.sim.registry().descriptor(id).unwrap_or("minecraft:air");
            let _ = write!(out, "{descriptor}");
        }

        /// Snapshot the entire simulation; returns a checkpoint id.
        pub fn checkpoint(&mut self) -> u32 {
            self.checkpoints.push(self.sim.checkpoint());
            (self.checkpoints.len() - 1) as u32
        }

        /// Restore a checkpoint taken earlier on this simulation.
        pub fn restore(&mut self, id: u32) -> Result<(), NucleationError> {
            let checkpoint = self
                .checkpoints
                .get(id as usize)
                .ok_or(NucleationError::NotFound)?;
            self.sim.restore(checkpoint);
            Ok(())
        }

        /// Every recorded block change since settle, as JSON:
        /// `[{"tick":N,"pos":[x,y,z],"from":"...","to":"..."}]`.
        /// Render a schematic as gametest-flavor structure SNBT — the text
        /// `from_snbt` and the corpus/render tooling consume. Lets hosts hand
        /// a converted `.litematic`/`.schem` to the video renderer.
        pub fn gametest_snbt(schematic: &Schematic, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", super::to_gametest_snbt(&schematic.0));
        }

        pub fn changes_json(&self, out: &mut DiplomatWrite) {
            let mut json = String::from("[");
            for (i, change) in self.sim.recorded().iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                let from = self.sim.registry().descriptor(change.from).unwrap_or("?");
                let to = self.sim.registry().descriptor(change.to).unwrap_or("?");
                let _ = write!(
                    json,
                    "{{\"tick\":{},\"pos\":[{},{},{}],\"from\":\"{}\",\"to\":\"{}\"}}",
                    change.tick, change.pos.x, change.pos.y, change.pos.z, from, to
                );
            }
            json.push(']');
            let _ = write!(out, "{json}");
        }

        /// Live item entities and minecarts, as JSON:
        /// `{"items":[{"id":N,"item":"...","count":N,"pos":[..],"vel":[..],
        ///   "on_ground":bool,"contents":[{"id":"...","count":N}]}],
        ///  "minecarts":[{"id":N,"kind":"...","pos":[..],"vel":[..]}]}`.
        pub fn item_entities_json(&self, out: &mut DiplomatWrite) {
            let mut json = String::from("{\"items\":[");
            let mut first = true;
            for entity in self.sim.item_entities() {
                if entity.removed {
                    continue;
                }
                if !first {
                    json.push(',');
                }
                first = false;
                let _ = write!(
                    json,
                    "{{\"id\":{},\"item\":\"{}\",\"count\":{},\"pos\":[{},{},{}],\"vel\":[{},{},{}],\"on_ground\":{}",
                    entity.id,
                    entity.item.0,
                    entity.item.1,
                    entity.pos[0], entity.pos[1], entity.pos[2],
                    entity.vel[0], entity.vel[1], entity.vel[2],
                    entity.on_ground,
                );
                json.push_str(",\"contents\":[");
                let contents = self.sim.item_contents(entity.id).unwrap_or(&[]);
                for (i, stack) in contents.iter().enumerate() {
                    if i > 0 {
                        json.push(',');
                    }
                    let _ = write!(
                        json,
                        "{{\"id\":\"{}\",\"count\":{}}}",
                        stack.id, stack.count
                    );
                }
                json.push_str("]}");
            }
            json.push_str("],\"minecarts\":[");
            let mut first = true;
            for cart in self.sim.minecarts() {
                if cart.removed {
                    continue;
                }
                if !first {
                    json.push(',');
                }
                first = false;
                let _ = write!(
                    json,
                    "{{\"id\":{},\"kind\":\"{}\",\"pos\":[{},{},{}],\"vel\":[{},{},{}]}}",
                    cart.id,
                    cart.kind,
                    cart.pos[0], cart.pos[1], cart.pos[2],
                    cart.vel[0], cart.vel[1], cart.vel[2],
                );
            }
            json.push_str("]}");
            let _ = write!(out, "{json}");
        }

        /// Per-tick aggregates over the recorded changes, as JSON:
        /// `[{"tick":N,"changes":N,"piston":N,"redstone":N}]` — `piston`
        /// counts changes touching piston blocks (base, head, moving), and
        /// `redstone` changes touching wire/torch/repeater/comparator/
        /// observer/lamp/lever/button/pressure-plate states.
        pub fn events_summary_json(&self, out: &mut DiplomatWrite) {
            use std::collections::BTreeMap;
            #[derive(Default)]
            struct Row {
                changes: u32,
                piston: u32,
                redstone: u32,
            }
            let mut rows: BTreeMap<u64, Row> = BTreeMap::new();
            for change in self.sim.recorded() {
                let from = self.sim.registry().descriptor(change.from).unwrap_or("");
                let to = self.sim.registry().descriptor(change.to).unwrap_or("");
                let row = rows.entry(change.tick).or_default();
                row.changes += 1;
                let named = |needle: &str| {
                    super::is_named(from, needle) || super::is_named(to, needle)
                };
                if named("piston") {
                    row.piston += 1;
                }
                if named("redstone")
                    || named("repeater")
                    || named("comparator")
                    || named("observer")
                    || named("lever")
                    || named("button")
                    || named("pressure_plate")
                    || named("lamp")
                {
                    row.redstone += 1;
                }
            }
            let mut json = String::from("[");
            for (i, (tick, row)) in rows.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                let _ = write!(
                    json,
                    "{{\"tick\":{},\"changes\":{},\"piston\":{},\"redstone\":{}}}",
                    tick, row.changes, row.piston, row.redstone
                );
            }
            json.push(']');
            let _ = write!(out, "{json}");
        }

        /// Every non-air block, as JSON:
        /// `[{"pos":[x,y,z],"state":"..."}]`.
        /// How many non-air blocks stand in the world right now.
        pub fn non_air_count(&self) -> u32 {
            self.sim.world().non_air_count() as u32
        }

        /// Center of mass (x) of every non-air block — the GA's displacement
        /// metric without a JSON round-trip. NaN when the world is empty.
        pub fn non_air_center_x(&self) -> f64 {
            let mut sum = 0.0;
            let mut n = 0u32;
            for (pos, _) in self.sim.world().iter_non_air() {
                sum += f64::from(pos.x);
                n += 1;
            }
            if n == 0 {
                f64::NAN
            } else {
                sum / f64::from(n)
            }
        }

        /// Smallest x holding a non-air block; `i32::MAX` when empty.
        pub fn non_air_min_x(&self) -> i32 {
            self.sim
                .world()
                .iter_non_air()
                .map(|(pos, _)| pos.x)
                .min()
                .unwrap_or(i32::MAX)
        }

        /// Largest x holding a non-air block; `i32::MIN` when empty.
        pub fn non_air_max_x(&self) -> i32 {
            self.sim
                .world()
                .iter_non_air()
                .map(|(pos, _)| pos.x)
                .max()
                .unwrap_or(i32::MIN)
        }

        /// How many block changes recording has captured so far.
        pub fn changes_count(&self) -> u32 {
            self.sim.recorded().len() as u32
        }

        pub fn world_snapshot_json(&self, out: &mut DiplomatWrite) {
            let mut json = String::from("[");
            let mut first = true;
            for (pos, id) in self.sim.world().iter_non_air() {
                if !first {
                    json.push(',');
                }
                first = false;
                let state = self.sim.registry().descriptor(id).unwrap_or("?");
                let _ = write!(
                    json,
                    "{{\"pos\":[{},{},{}],\"state\":\"{}\"}}",
                    pos.x, pos.y, pos.z, state
                );
            }
            json.push(']');
            let _ = write!(out, "{json}");
        }
    }
}
