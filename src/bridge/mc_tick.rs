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

use crate::formats::gametest::to_gametest_snbt;

/// `{simulate=true}`: place a block into the schematic *as a world* and let
/// the engine react — connectivity, power, scheduled ticks, all of it — then
/// write every resulting block change back into the schematic.
///
/// The semantics are "a hand placed this block in a loaded world": the rest
/// of the schematic is trusted exactly as saved (`InWorld`), the new block
/// arrives through the engine's `place_block` (vanilla's flag-3 write, so
/// `onPlace` runs and neighbours hear about it), and the world then runs to
/// quiescence. A wire comes out with its real connections and power; a
/// repeater locks or lights; a piston that ends up powered genuinely
/// extends, head and all — the write-back records whatever the world became.
///
/// A placement more than three blocks from everything else can interact with
/// nothing, so it short-circuits to a plain write — which also covers the
/// first block of an empty schematic.
///
/// Returns the number of blocks the write-back touched (at least one: the
/// placed block itself).
pub(crate) fn simulate_placement_into(
    schematic: &mut crate::UniversalSchematic,
    x: i32,
    y: i32,
    z: i32,
    descriptor: &str,
) -> Result<usize, String> {
    use mc_tick::Pos;

    // Far from everything (or into an empty schematic): nothing to react.
    let bb = schematic.get_bounding_box();
    let isolated = schematic.total_blocks() == 0 || {
        let (min, max) = (bb.min, bb.max);
        x < min.0 - 3
            || x > max.0 + 3
            || y < min.1 - 3
            || y > max.1 + 3
            || z < min.2 - 3
            || z > max.2 + 3
    };
    if isolated {
        schematic.set_block_from_string(x, y, z, descriptor)?;
        return Ok(1);
    }

    check_volume((
        bb.max.0 - bb.min.0 + 1,
        bb.max.1 - bb.min.1 + 1,
        bb.max.2 - bb.min.2 + 1,
    ))?;

    // The world is the schematic *without* the new block; gametest SNBT
    // rebases everything to the bounding box's low corner, so the engine
    // works in shifted coordinates and the write-back shifts them home.
    let offset = (bb.min.0, bb.min.1, bb.min.2);
    let snbt = to_gametest_snbt(schematic);
    let structure = mc_tick::Structure::parse(&snbt)
        .map_err(|e| format!("simulate=true could not load this schematic: {e:?}"))?;
    let mut sim = wire_simulation(
        &structure,
        Pos::new(0, 0, 0),
        ffi::TickSettleMode::InWorld,
        &[descriptor],
        schematic.metadata.source_data_version,
    )
    .map_err(|e| format!("simulate=true could not simulate this schematic: {e}"))?;

    let state = sim
        .registry_mut()
        .intern(descriptor)
        .map_err(|e| format!("simulate=true: interning {descriptor}: {e:?}"))?;
    let pos = Pos::new(x - offset.0, y - offset.1, z - offset.2);
    let placed_bounds = structure.bounds(4);
    if pos.x < placed_bounds.min.x
        || pos.x > placed_bounds.max.x
        || pos.y < placed_bounds.min.y
        || pos.y > placed_bounds.max.y
        || pos.z < placed_bounds.min.z
        || pos.z > placed_bounds.max.z
    {
        // Inside the 3-block interaction range but outside the engine's
        // padded world — cannot happen while the margin is 4, but a changed
        // margin must fail loudly here rather than panic in the engine.
        return Err("simulate=true: position outside the simulated bounds".to_string());
    }
    sim.record();
    // A *genuine* hand placement: the state derives its shape from the
    // neighbourhood first (a wire arrives connected), then `onPlace` runs
    // (the wire powers, a repeater locks), then the neighbours are told.
    sim.place_block_by_hand(pos, state);
    sim.run_until_quiescent(255);

    // The placed cell first, then every recorded change on top — including
    // the placed cell's own evolved state, and anything the placement set
    // off elsewhere.
    schematic.set_block_from_string(x, y, z, descriptor)?;
    let mut finals: std::collections::HashMap<Pos, mc_tick::StateId> =
        std::collections::HashMap::new();
    for change in sim.recorded() {
        finals.insert(change.pos, change.to);
    }
    let mut written = 1;
    for (cell, state) in finals {
        let descriptor = sim
            .registry()
            .descriptor(state)
            .ok_or_else(|| "simulate=true: a written state with no descriptor".to_string())?;
        schematic.set_block_from_string(
            cell.x + offset.0,
            cell.y + offset.1,
            cell.z + offset.2,
            descriptor,
        )?;
        written += 1;
    }
    Ok(written)
}

/// Whether a block's simulated behaviour depends on block-entity data.
///
/// Only the ones whose *absence changes the run*: a comparator without
/// `OutputSignal` reads 0, a container without `Items` is empty and so reads
/// 0 through a comparator and has nothing to transfer. Signs, banners and
/// heads carry block entities too, and losing them changes nothing that
/// ticks — listing them would bury the two that matter.
fn needs_block_entity(name: &str) -> bool {
    let short = name.strip_prefix("minecraft:").unwrap_or(name);
    matches!(
        short,
        "comparator"
            | "chest"
            | "trapped_chest"
            | "barrel"
            | "hopper"
            | "dropper"
            | "dispenser"
            | "furnace"
            | "blast_furnace"
            | "smoker"
            | "brewing_stand"
            | "crafter"
            | "chiseled_bookshelf"
            | "jukebox"
            | "lectern"
            | "decorated_pot"
    ) || short.ends_with("shulker_box")
}

/// See [`ffi::TickSimulation::block_entity_audit_json`].
fn block_entity_audit(schematic: &crate::UniversalSchematic) -> String {
    use std::collections::{HashMap, HashSet};
    use std::fmt::Write as _;

    let have: HashSet<(i32, i32, i32)> = schematic
        .get_block_entities_as_list()
        .into_iter()
        .map(|be| be.position)
        .collect();

    let mut missing: HashMap<String, u32> = HashMap::new();
    for (pos, state) in schematic.iter_blocks() {
        if !needs_block_entity(&state.name) {
            continue;
        }
        if !have.contains(&(pos.x, pos.y, pos.z)) {
            *missing.entry(state.name.to_string()).or_default() += 1;
        }
    }

    let mut rows: Vec<(String, u32)> = missing.into_iter().collect();
    // Descending count, then name — a stable order so two runs of the same
    // file produce byte-identical JSON.
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let total: u32 = rows.iter().map(|(_, n)| *n).sum();

    let mut json = String::from("{\"present\":");
    let _ = write!(json, "{}", have.len());
    let _ = write!(json, ",\"missing_total\":{total},\"missing\":[");
    for (i, (name, count)) in rows.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        let _ = write!(json, "{{\"name\":\"{name}\",\"count\":{count}}}");
    }
    json.push_str("],\"summary\":\"");
    if total > 0 {
        let named: Vec<String> = rows
            .iter()
            .take(3)
            .map(|(name, count)| {
                let short = name.strip_prefix("minecraft:").unwrap_or(name);
                let plural = if *count == 1 { "" } else { "s" };
                format!("{count} {short}{plural}")
            })
            .collect();
        let more = if rows.len() > 3 { ", and others" } else { "" };
        let _ = write!(
            json,
            "This schematic contains {}{} with no block-entity data. \
             Comparator outputs and container contents are simulated as empty, \
             so results may not reflect the original build.",
            named.join(", "),
            more
        );
    }
    json.push_str("\"}");
    json
}

/// The sentence shown to whoever is holding a structure that will not load.
///
/// Two very different failures reach the same `parse` call and they need
/// opposite answers. An unsupported entity means their *build* names something
/// the engine cannot model — nothing is wrong with the file, so it is reported
/// by name and without blame. Anything else means the text itself is bad;
/// when we generated that text (`converted`), that is our bug and saying so
/// keeps us from accusing a perfectly good upload.
fn structure_parse_detail(error: &mc_tick::structure::StructureError, converted: bool) -> String {
    if let mc_tick::structure::StructureError::UnsupportedEntity { entity_type, .. } = error {
        return format!(
            "this build contains a `{entity_type}` entity, which the engine cannot simulate \
             yet — loading it would mean dropping the entity, and a run without it would not \
             match the real build"
        );
    }
    if converted {
        format!(
            "converted structure did not parse: {error:?} — this is an engine fault, \
             not a problem with the uploaded file"
        )
    } else {
        format!("structure SNBT did not parse: {error:?}")
    }
}

/// Serialise recorded updates for ticks in `[from, to)`.
///
/// Shared by the whole-log and per-tick-range accessors so both emit exactly
/// one schema. `state` is the block at dispatch time, not at the tick boundary.
fn updates_json_range(sim: &mc_tick::Simulation, from: u64, to: u64) -> String {
    use std::fmt::Write as _;
    let mut json = String::from("[");
    let mut first = true;
    for update in sim.recorded_updates() {
        if update.tick < from || update.tick >= to {
            continue;
        }
        if !first {
            json.push(',');
        }
        first = false;
        let state = sim
            .registry()
            .descriptor(update.state)
            .unwrap_or("minecraft:air");
        let kind = match update.kind {
            mc_tick::UpdateKind::Neighbor => "neighbor",
            mc_tick::UpdateKind::Shape => "shape",
        };
        // No phase means a boundary dispatch: placement, a click, a break —
        // the server loop rather than a phase of the tick.
        let phase = update.phase.map_or("boundary", |p| p.name());
        let _ = write!(
            json,
            "{{\"tick\":{},\"seq\":{},\"pos\":[{},{},{}],\"from\":\"{:?}\",\"kind\":\"{}\",\"phase\":\"{}\",\"state\":\"{}\"}}",
            update.tick,
            update.seq,
            update.pos.x,
            update.pos.y,
            update.pos.z,
            update.from,
            kind,
            phase,
            state
        );
    }
    json.push(']');
    json
}

/// One detected cycle as `{"start":T,"end":T,"period":N,"drift":[x,y,z]}`, or
/// `null` when none was found — a build with no recurrence is normal, not an
/// error.
fn cycle_json(cycle: Option<mc_tick::Cycle>) -> String {
    match cycle {
        None => "null".to_string(),
        Some(c) => format!(
            "{{\"start\":{},\"end\":{},\"period\":{},\"drift\":[{},{},{}]}}",
            c.start_tick, c.end_tick, c.period, c.drift.x, c.drift.y, c.drift.z
        ),
    }
}

/// The phase legend shared by the compact update views: index 0 is a boundary
/// dispatch (outside the phase walk), then [`mc_tick::PHASE_ORDER`].
fn phase_legend() -> Vec<&'static str> {
    let mut names = vec!["boundary"];
    names.extend(mc_tick::PHASE_ORDER.iter().map(|p| p.name()));
    names
}

/// A record's index into [`phase_legend`].
fn phase_code(update: &mc_tick::UpdateRecord) -> usize {
    match update.phase {
        None => 0,
        Some(phase) => mc_tick::PHASE_ORDER
            .iter()
            .position(|p| *p == phase)
            .map_or(0, |i| i + 1),
    }
}

/// A direction's index into [`mc_tick::ALL_DIRS`].
fn dir_code(dir: mc_tick::Dir) -> usize {
    mc_tick::ALL_DIRS
        .iter()
        .position(|d| *d == dir)
        .unwrap_or(0)
}

/// Per-tick, per-cell update counts — the resolution playback runs at.
///
/// The raw log is unusable for a UI: one tick of a 6x6 door is ~20k updates and
/// megabytes of JSON, and twenty thousand individual flares are not legible
/// anyway. Collapsing to "which cells lit up this tick, and how hot" turns that
/// into a few hundred rows while keeping the two breakdowns worth colouring by.
fn updates_heat_range(sim: &mc_tick::Simulation, from: u64, to: u64) -> String {
    use std::collections::BTreeMap;
    use std::fmt::Write as _;

    let phases = phase_legend();
    // BTreeMap so ticks and cells come out in a stable, sorted order.
    let mut per_tick: BTreeMap<u64, BTreeMap<(i32, i32, i32), (u32, u32, u32, Vec<u32>)>> =
        BTreeMap::new();
    for update in sim.recorded_updates() {
        if update.tick < from || update.tick >= to {
            continue;
        }
        let cells = per_tick.entry(update.tick).or_default();
        let cell = cells
            .entry((update.pos.x, update.pos.y, update.pos.z))
            .or_insert_with(|| (0, 0, 0, vec![0; phases.len()]));
        cell.0 += 1;
        match update.kind {
            mc_tick::UpdateKind::Neighbor => cell.1 += 1,
            mc_tick::UpdateKind::Shape => cell.2 += 1,
        }
        cell.3[phase_code(update)] += 1;
    }

    let mut json = String::from("{\"phases\":[");
    for (i, name) in phases.iter().enumerate() {
        let _ = write!(json, "{}\"{name}\"", if i > 0 { "," } else { "" });
    }
    json.push_str("],\"ticks\":[");
    for (i, (tick, cells)) in per_tick.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        let total: u32 = cells.values().map(|c| c.0).sum();
        let _ = write!(json, "{{\"tick\":{tick},\"total\":{total},\"cells\":[");
        for (j, ((x, y, z), (n, nb, sh, ph))) in cells.iter().enumerate() {
            if j > 0 {
                json.push(',');
            }
            let _ = write!(
                json,
                "{{\"p\":[{x},{y},{z}],\"n\":{n},\"nb\":{nb},\"sh\":{sh},\"ph\":["
            );
            for (k, count) in ph.iter().enumerate() {
                let _ = write!(json, "{}{count}", if k > 0 { "," } else { "" });
            }
            json.push_str("]}");
        }
        json.push_str("]}");
    }
    json.push_str("]}");
    json
}

/// One tick's updates in delivery order, as parallel arrays.
///
/// The wavefront resolution: everything the raw log has for a single tick, but
/// without repeating a field name per record. `seq` is the array index; every
/// small enum is an integer code with its legend in the payload; and the
/// dispatch-time state is an index into a deduplicated table, which is where
/// most of the saving comes from — a tick touches thousands of cells but only
/// tens of distinct states.
fn updates_wave(sim: &mc_tick::Simulation, tick: u64) -> String {
    use std::collections::HashMap;
    use std::fmt::Write as _;

    let mut pos = String::new();
    let mut kinds = String::new();
    let mut phases_arr = String::new();
    let mut froms = String::new();
    let mut states_arr = String::new();
    let mut table: Vec<&str> = Vec::new();
    let mut seen: HashMap<mc_tick::StateId, usize> = HashMap::new();
    let mut n = 0usize;

    for update in sim.recorded_updates() {
        if update.tick != tick {
            continue;
        }
        let sep = if n > 0 { "," } else { "" };
        let _ = write!(
            pos,
            "{sep}{},{},{}",
            update.pos.x, update.pos.y, update.pos.z
        );
        let _ = write!(
            kinds,
            "{sep}{}",
            match update.kind {
                mc_tick::UpdateKind::Neighbor => 0,
                mc_tick::UpdateKind::Shape => 1,
            }
        );
        let _ = write!(phases_arr, "{sep}{}", phase_code(update));
        let _ = write!(froms, "{sep}{}", dir_code(update.from));
        let index = *seen.entry(update.state).or_insert_with(|| {
            table.push(
                sim.registry()
                    .descriptor(update.state)
                    .unwrap_or("minecraft:air"),
            );
            table.len() - 1
        });
        let _ = write!(states_arr, "{sep}{index}");
        n += 1;
    }

    let mut json = String::new();
    let _ = write!(
        json,
        "{{\"tick\":{tick},\"n\":{n},\"pos\":[{pos}],\"kind\":[{kinds}],"
    );
    let _ = write!(
        json,
        "\"phase\":[{phases_arr}],\"from\":[{froms}],\"state\":[{states_arr}],"
    );
    json.push_str("\"states\":[");
    for (i, descriptor) in table.iter().enumerate() {
        let _ = write!(json, "{}\"{descriptor}\"", if i > 0 { "," } else { "" });
    }
    json.push_str("],\"phases\":[");
    for (i, name) in phase_legend().iter().enumerate() {
        let _ = write!(json, "{}\"{name}\"", if i > 0 { "," } else { "" });
    }
    json.push_str("],\"dirs\":[");
    for (i, dir) in mc_tick::ALL_DIRS.iter().enumerate() {
        let _ = write!(json, "{}\"{dir:?}\"", if i > 0 { "," } else { "" });
    }
    json.push_str("],\"kinds\":[\"neighbor\",\"shape\"]}");
    json
}

/// Largest build the simulator will accept, in cells.
///
/// `IRIS_B.schem` is 499x379x442 — 83.5 million cells, a saved world rather
/// than a door. Loading one exhausts the wasm heap, and a Rust OOM in wasm is
/// an `unreachable` trap that poisons the whole instance: every later call on
/// it traps too, so one oversized upload takes down every door after it. Eight
/// million cells is roughly a 200-cube, far past any real door and well inside
/// what the heap survives.
const MAX_VOLUME: usize = 8_000_000;

thread_local! {
    /// Why the last constructor failed. Set on every failure path, cleared on
    /// success, read back through `TickSimulation::last_error_detail`.
    static LAST_ERROR: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn set_last_error(detail: impl Into<String>) {
    LAST_ERROR.with(|e| *e.borrow_mut() = detail.into());
}

fn clear_last_error() {
    LAST_ERROR.with(|e| e.borrow_mut().clear());
}

/// Refuse a build too large to load before allocating for it.
fn check_volume(size: (i32, i32, i32)) -> Result<(), String> {
    let volume = (size.0 as i64) * (size.1 as i64) * (size.2 as i64);
    if volume > MAX_VOLUME as i64 {
        return Err(format!(
            "build is {} x {} x {} = {volume} cells, over the {MAX_VOLUME}-cell limit — \
             this looks like a saved world rather than a contraption",
            size.0, size.1, size.2
        ));
    }
    Ok(())
}

fn wire_simulation(
    structure: &mc_tick::Structure,
    hash_origin: mc_tick::Pos,
    settle: ffi::TickSettleMode,
    extra_states: &[&str],
    source_data_version: Option<i32>,
) -> Result<mc_tick::Simulation, String> {
    use mc_tick::{Pos, Simulation};
    const MARGIN: i32 = 4;

    let mut sim = Simulation::new(structure.bounds(MARGIN));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    // Which game's `Entity.load` these entities came through.
    //
    // The blocks in `structure` have been converted to the canonical data
    // version; the *entities* have not been reinterpreted, and whether a NaN
    // velocity survives being read is decided by the version of the save it
    // was read from. `Motion` handling changed at 1.21.11, and the record 3x3
    // door is 1.21.3 — under the new rules its nan carts do not exist. Only
    // the caller knows the source version, which is why it is a parameter and
    // not something the engine tries to infer. See [`mc_tick::MotionSemantics`].
    if let Some(version) = source_data_version {
        sim.set_motion_semantics(mc_tick::MotionSemantics::for_data_version(version));
    }
    // The universal actuator, plus anything the caller names.
    let mut wanted: Vec<String> = vec!["minecraft:redstone_block".to_string()];
    wanted.extend(extra_states.iter().map(|s| s.to_string()));
    // A dispenser can *place* a block it holds as an item — a shulker box, or a
    // bucket's contents. Behaviours bind only to interned states, and those
    // states are by definition absent from the build's own palette.
    for (_, stacks) in &structure.inventories {
        for stack in stacks {
            wanted.extend(mc_tick::vanilla::dispensable_states(&stack.id));
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
        let name = structure.palette[entry]
            .split('[')
            .next()
            .unwrap_or_default()
            .to_string();
        let slots = mc_tick::vanilla::container_slots(&name)
            .ok_or_else(|| format!("{name} has an inventory but no slot count"))?;
        sim.set_inventory(
            *pos,
            mc_tick::Inventory {
                slots,
                stacks: stacks.clone(),
                blocked_slots: structure.blocked_slots_at(*pos),
            },
        );
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
    // Entities the parser can read but the engine cannot yet run.
    //
    // This is the entity half of the `unknown_report` gate above: a build is
    // refused whole rather than simulated with pieces missing. It lives here,
    // at construction, rather than in the parser, because "can this be read"
    // and "is there a behaviour for it" are different questions — the parser
    // answering both meant the behaviours agent could not even load a villager
    // to develop against.
    //
    // The match is deliberately exhaustive with no catch-all: a new
    // `SpawnedEntity` variant will fail to compile here until someone decides
    // whether it can be simulated. That is the whole point of the split — the
    // gate cannot silently fall out of date.
    // Every arm below either spawns or records a refusal. The match has no
    // catch-all on purpose: a new `SpawnedEntity` variant fails to compile here
    // until someone decides whether it can be simulated, so the gate cannot
    // silently fall out of date.
    let mut refused: Vec<String> = Vec::new();
    for spawned in &structure.entities {
        match spawned {
            mc_tick::structure::SpawnedEntity::Item(item) => {
                sim.spawn_item(item.item.clone(), item.pos, item.motion, item.pickup_delay);
            }
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                let vehicle = sim.spawn_authored_minecart(cart, None);
                // Riders are seated immediately after their vehicle, so a
                // capture's ids line up and so the rider cannot outlive a
                // vehicle that failed to spawn.
                for rider in &cart.passengers {
                    if let Err(why) = sim.spawn_authored_rider(vehicle, rider) {
                        refused.push(why);
                    }
                }
            }
            // A furnace cart is dimensionally an ordinary cart, so an
            // *unfuelled* one needs nothing more than being a cart. A fuelled
            // one drives itself and is refused rather than run as a passenger.
            mc_tick::structure::SpawnedEntity::FurnaceMinecart(cart) => {
                if let Err(why) = sim.spawn_authored_furnace_minecart(cart, None) {
                    refused.push(why);
                }
            }
            // Fireballs and villagers exist in the record doors as scaffolding:
            // a hitbox a pressure plate can see and a piston can shove. That is
            // all that is implemented, and anything that would need more — a
            // fireball with velocity, a villager that should walk — refuses by
            // name rather than being quietly frozen.
            // A blaze reached here is one standing on its own, not riding — a
            // rider is spawned by its vehicle's arm above and never appears in
            // this list. Standing alone it is scaffolding like a villager, and a
            // blaze that should fly or fight refuses by name.
            mc_tick::structure::SpawnedEntity::Body(body) => {
                match sim.spawn_authored_body(body) {
                    Ok(vehicle) => {
                        for rider in &body.passengers {
                            if let Err(why) = sim.spawn_authored_rider(vehicle, rider) {
                                refused.push(why);
                            }
                        }
                    }
                    Err(why) => {
                        refused.push(why);
                    }
                }
            }
        }
    }
    if !refused.is_empty() {
        return Err(format!(
            "{} entit{} in this build need behaviour that is not implemented, and the \
             build is refused rather than simulated with them standing still:\n  - {}",
            refused.len(),
            if refused.len() == 1 { "y" } else { "ies" },
            refused.join("\n  - ")
        ));
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

/// One pass over the world: (non-air count, center-of-mass x, min x, max x).
fn non_air_stats(sim: &mc_tick::Simulation) -> (u32, f64, i32, i32) {
    let mut n = 0u32;
    let mut sum = 0.0;
    let mut min = i32::MAX;
    let mut max = i32::MIN;
    for (pos, _) in sim.world().iter_non_air() {
        n += 1;
        sum += f64::from(pos.x);
        if pos.x < min {
            min = pos.x;
        }
        if pos.x > max {
            max = pos.x;
        }
    }
    (
        n,
        if n == 0 { f64::NAN } else { sum / f64::from(n) },
        min,
        max,
    )
}

/// Build a Structure directly from a flat genome-cell array — the GA fast
/// path, no SNBT text. Layout mirrors the flying-ga corridor: machine at
/// `x_off`, world size `[bx + travel, by + 2, bz + 2]`, cells flattened as
/// `((y * bz) + z) * bx + x`, `air` the palette index meaning empty. The
/// full palette rides along (indices are alphabet indices verbatim), so
/// behaviours bind to every alphabet state exactly as the SNBT path did via
/// its EXTRA_STATES list.
fn structure_from_blocks(
    bx: i32,
    by: i32,
    bz: i32,
    travel: i32,
    x_off: i32,
    palette: &[String],
    cells: &[u16],
    air: u16,
) -> Result<mc_tick::Structure, String> {
    let volume = (bx.max(0) as usize) * (by.max(0) as usize) * (bz.max(0) as usize);
    if cells.len() != volume {
        return Err(format!("cells len {} != bbox volume {volume}", cells.len()));
    }
    let mut blocks = Vec::new();
    let mut i = 0usize;
    for _y in 0..by {
        for _z in 0..bz {
            for _x in 0..bx {
                let s = cells[i];
                let (x, y, z) = (_x, _y, _z);
                i += 1;
                if s == air {
                    continue;
                }
                if s as usize >= palette.len() {
                    return Err(format!("palette index {s} out of range"));
                }
                blocks.push((mc_tick::Pos::new(x + x_off, y, z), s as usize));
            }
        }
    }
    Ok(mc_tick::Structure {
        // Genome cells, not a save: there is no file and so no version to
        // report. The caller's own default decides Motion semantics, and this
        // path authors no entities for it to apply to.
        data_version: None,
        size: (bx + travel, by + 2, bz + 2),
        palette: palette.to_vec(),
        blocks,
        inventories: Vec::new(),
        inventory_blocked_slots: Vec::new(),
        comparator_outputs: Vec::new(),
        block_entities: Vec::new(),
        entities: Vec::new(),
        item_entities: Vec::new(),
        // Genome cells carry no command blocks.
        commands: Vec::new(),
    })
}

/// The modal gait period from min-x rise gaps — bit-identical port of the
/// app's `modalGap` (mode, ties to the smaller gap, modal share >= 0.6).
fn modal_gap(gaps: &[u32]) -> u32 {
    if gaps.len() < 3 {
        return 0;
    }
    let mut order: Vec<u32> = Vec::new();
    let mut counts: Vec<u32> = Vec::new();
    for &g in gaps {
        match order.iter().position(|&o| o == g) {
            Some(i) => counts[i] += 1,
            None => {
                order.push(g);
                counts.push(1);
            }
        }
    }
    let mut best = 0u32;
    let mut best_gap: Option<u32> = None;
    for (i, &g) in order.iter().enumerate() {
        let n = counts[i];
        if n > best || (n == best && best_gap.is_some_and(|b| g < b)) {
            best = n;
            best_gap = Some(g);
        }
    }
    match best_gap {
        Some(g) if (best as f64) / (gaps.len() as f64) >= 0.6 => g,
        _ => 0,
    }
}

/// One kicked flight, mirroring the app's `evalCore.fly` exactly: quiet
/// settle at construction, redstone-block kick at tick 2 removed at tick 4,
/// the same probe schedule (must-move deadline, mid-window centre of mass),
/// optional in-eval gait detection over the last 120 ticks, and an optional
/// early exit for machines that are provably frozen — quiescent and unmoved
/// at tick 40, where every later scalar equals the tick-40 scalar, so the
/// shortcut changes wall time and nothing else.
///
/// Row layout: `[n0, startCom, startMinX, startMaxX, comAtMoveCheck(NaN =
/// no deadline), comAtMid, period, n1, endCom, endMinX, endMaxX]`.
#[allow(clippy::too_many_arguments)]
fn fly_metrics(
    structure: &mc_tick::Structure,
    extras: &[&str],
    kick: (i32, i32, i32),
    eval_ticks: u32,
    seed: i64,
    must_move_by_tick: i32,
    need_period: bool,
    early_exit: bool,
) -> Result<[f64; 11], String> {
    let mut sim = wire_simulation(
        structure,
        mc_tick::Pos::new(0, 0, 0),
        ffi::TickSettleMode::Quiet,
        extras,
        // SNBT in, and the bridge emits the canonical DataVersion — there is no
        // source version to read here. `None` keeps the engine's default,
        // which is the version every captured trace came from.
        None,
    )?;
    fly_on(
        &mut sim,
        kick,
        eval_ticks,
        seed,
        must_move_by_tick,
        need_period,
        early_exit,
    )
}

/// The flight itself, on an already-wired sim (fresh, quiet-settled).
#[allow(clippy::too_many_arguments)]
fn fly_on(
    sim: &mut mc_tick::Simulation,
    kick: (i32, i32, i32),
    eval_ticks: u32,
    seed: i64,
    must_move_by_tick: i32,
    need_period: bool,
    early_exit: bool,
) -> Result<[f64; 11], String> {
    const PERIOD_WINDOW: u32 = 120;
    const EARLY_TICK: u32 = 40;
    sim.set_rng_seed(seed);

    let (n0, start_com, start_min, start_max) = non_air_stats(sim);
    let mut row = [f64::NAN; 11];
    row[0] = f64::from(n0);
    row[1] = start_com;
    row[2] = f64::from(start_min);
    row[3] = f64::from(start_max);
    if n0 == 0 {
        return Ok(row); // the caller short-circuits on n0 before reading on
    }

    let redstone = sim
        .registry()
        .get("minecraft:redstone_block")
        .ok_or("redstone_block not interned")?;
    let kick_pos = mc_tick::Pos::new(kick.0, kick.1, kick.2);
    sim.run(2);
    sim.place_block(kick_pos, redstone);
    sim.run(2);
    sim.place_block(kick_pos, mc_tick::StateId::AIR);
    let mut elapsed: u32 = 4;

    let mid_tick = eval_ticks.min((eval_ticks / 2).max(elapsed));
    let move_check: Option<u32> = if must_move_by_tick >= 0 {
        Some((must_move_by_tick as u32).max(elapsed).min(eval_ticks))
    } else {
        None
    };
    let mut probes: Vec<u32> = Vec::new();
    if let Some(mc) = move_check {
        probes.push(mc);
    }
    probes.push(mid_tick);
    if early_exit && eval_ticks > EARLY_TICK {
        probes.push(EARLY_TICK.max(elapsed));
    }
    probes.sort_unstable();
    probes.dedup();

    let mut com_mid = start_com;
    let mut com_move = f64::NAN;
    let mut frozen = false;
    for &t in &probes {
        if t > elapsed {
            sim.run(u64::from(t - elapsed));
            elapsed = t;
        }
        let (_, com, _, _) = non_air_stats(&sim);
        if Some(t) == move_check {
            com_move = com;
        }
        if t == mid_tick {
            com_mid = com;
        }
        if early_exit && t == EARLY_TICK && sim.is_quiescent() && (com - start_com).abs() < 0.25 {
            frozen = true;
            if mid_tick > t {
                com_mid = com;
            }
            if let Some(mc) = move_check {
                if mc > t && com_move.is_nan() {
                    com_move = com;
                }
            }
            break;
        }
    }

    let mut period = 0u32;
    if need_period && !frozen {
        let win_start = elapsed.max(eval_ticks.saturating_sub(PERIOD_WINDOW));
        if win_start > elapsed {
            sim.run(u64::from(win_start - elapsed));
            elapsed = win_start;
        }
        let mut gaps: Vec<u32> = Vec::new();
        let (_, _, mut prev_min, _) = non_air_stats(&sim);
        let mut last_rise: i64 = -1;
        while elapsed < eval_ticks {
            sim.run(1);
            elapsed += 1;
            let (_, _, mx, _) = non_air_stats(&sim);
            if mx > prev_min {
                if last_rise >= 0 {
                    gaps.push(elapsed - last_rise as u32);
                }
                last_rise = i64::from(elapsed);
            }
            prev_min = mx;
        }
        period = modal_gap(&gaps);
    }
    if !frozen && eval_ticks > elapsed {
        sim.run(u64::from(eval_ticks - elapsed));
    }

    let (n1, end_com, end_min, end_max) = non_air_stats(&sim);
    row[4] = com_move;
    row[5] = com_mid;
    row[6] = f64::from(period);
    row[7] = f64::from(n1);
    row[8] = end_com;
    row[9] = f64::from(end_min);
    row[10] = f64::from(end_max);
    Ok(row)
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
        /// The span the last [`TickSimulation::stop_timeline`] ended.
        ///
        /// Stop must end a recording *and keep it*: a host's Stop button leaves
        /// the span selectable and exportable, which it cannot be if stopping
        /// threw it away.
        pub(crate) stopped_timeline: Option<mc_tick::timeline::RunTimeline>,
    }

    impl TickSimulation {
        /// Why the last constructor on this thread failed, in words.
        ///
        /// The enum cannot carry a message, and "Simulation" is useless to
        /// someone holding a door that will not load: the engine already knows
        /// it is `minecraft:waxed_copper_bulb` at (4,2,1) and says so here.
        /// Empty when the last construction succeeded.
        pub fn last_error_detail(out: &mut DiplomatWrite) {
            super::LAST_ERROR.with(|e| {
                let _ = write!(out, "{}", e.borrow());
            });
        }

        /// Largest build this will attempt, in cells.
        ///
        /// A 500x379x442 "door" is a saved world, and loading one exhausts the
        /// wasm heap — after which every later call on that instance traps,
        /// not just the one that overflowed. Refused up front instead.
        pub fn max_volume() -> u32 {
            super::MAX_VOLUME as u32
        }

        /// Load from Java structure SNBT text.
        ///
        /// `extra_states`: semicolon-separated block-state descriptors that
        /// later `place_block` calls may write (behaviours bind at
        /// construction). `minecraft:redstone_block` is always available.
        /// `origin_*`: where the build's (0,0,0) sits in world coordinates —
        /// wire update order hashes absolute positions.
        ///
        /// The text's own `DataVersion` selects `Entity.load` Motion semantics,
        /// exactly as [`TickSimulation::from_schematic`] uses the schematic's —
        /// so `gametest_snbt` → `from_snbt` keeps a nan-cart build's NaN
        /// velocities instead of quietly sanitising them. A text with no
        /// `DataVersion` gets the engine default (the modern, NaN-dropping
        /// rule); read [`TickSimulation::motion_semantics`] to see which
        /// applied.
        pub fn from_snbt(
            snbt: &DiplomatStr,
            settle: TickSettleMode,
            origin_x: i32,
            origin_y: i32,
            origin_z: i32,
            extra_states: &DiplomatStr,
        ) -> Result<Box<TickSimulation>, NucleationError> {
            let snbt = std::str::from_utf8(snbt).map_err(|_| NucleationError::InvalidArgument)?;
            let extra =
                std::str::from_utf8(extra_states).map_err(|_| NucleationError::InvalidArgument)?;
            super::clear_last_error();
            let structure = mc_tick::Structure::parse(snbt).map_err(|e| {
                super::set_last_error(super::structure_parse_detail(&e, false));
                // An entity we cannot model is not a malformed file, so it
                // reports as a simulator limit rather than a parse failure.
                if matches!(
                    e,
                    mc_tick::structure::StructureError::UnsupportedEntity { .. }
                ) {
                    NucleationError::Simulation
                } else {
                    NucleationError::Parse
                }
            })?;
            super::check_volume(structure.size).map_err(|e| {
                super::set_last_error(e);
                NucleationError::InvalidArgument
            })?;
            let extras: Vec<&str> = extra
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            let sim = super::wire_simulation(
                &structure,
                mc_tick::Pos::new(origin_x, origin_y, origin_z),
                settle,
                &extras,
                // The text states the version it was saved at, and that decides
                // whether its non-finite `Motion` vectors survive being read.
                // Ignoring it made this path disagree with `from_schematic`
                // about the same build: the NaN token round-trips through the
                // SNBT fine, but the load rule chosen for it did not, so a
                // door held together by nan carts came out as ordinary carts.
                structure.data_version,
            )
            .map_err(|e| {
                super::set_last_error(e);
                NucleationError::Simulation
            })?;
            Ok(Box::new(TickSimulation {
                sim,
                checkpoints: Vec::new(),
                stopped_timeline: None,
            }))
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
            super::clear_last_error();
            let extra =
                std::str::from_utf8(extra_states).map_err(|_| NucleationError::InvalidArgument)?;
            // Before rendering anything: the SNBT for a world-sized build is
            // what exhausts the heap, and the trap it raises poisons the
            // instance for every door after it.
            let bb = schematic.0.get_bounding_box();
            super::check_volume((
                bb.max.0 - bb.min.0 + 1,
                bb.max.1 - bb.min.1 + 1,
                bb.max.2 - bb.min.2 + 1,
            ))
            .map_err(|e| {
                super::set_last_error(e);
                NucleationError::InvalidArgument
            })?;
            let snbt = super::to_gametest_snbt(&schematic.0);
            let structure = mc_tick::Structure::parse(&snbt).map_err(|e| {
                // The schematic loaded; either it names an entity we cannot
                // model, or our own rendering of it was rejected. Saying
                // "Parse" here blames the user's file for our bug, so both
                // report as an engine failure and the detail says which.
                super::set_last_error(super::structure_parse_detail(&e, true));
                NucleationError::Simulation
            })?;
            let extras: Vec<&str> = extra
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            let sim = super::wire_simulation(
                &structure,
                mc_tick::Pos::new(origin_x, origin_y, origin_z),
                settle,
                &extras,
                // The schematic remembers which game wrote it, and that
                // decides whether a non-finite `Motion` survives
                // `Entity.load` — the mechanism of the record nan-cart
                // doors. Passed through rather than defaulted: a 1.21.3
                // door and a 1.21.11 door are different machines, and
                // only this layer knows which one this is.
                schematic.0.metadata.source_data_version,
            )
            .map_err(|e| {
                super::set_last_error(e);
                NucleationError::Simulation
            })?;
            Ok(Box::new(TickSimulation {
                sim,
                checkpoints: Vec::new(),
                stopped_timeline: None,
            }))
        }

        /// GA fast path: construct from a flat genome-cell array — no SNBT
        /// text built or parsed. Corridor layout matches the flying-ga app:
        /// machine at `x_off`, world size `[bx + travel, by + 2, bz + 2]`,
        /// cells flattened `((y * bz) + z) * bx + x`, `air_index` = empty
        /// cell. `palette` is the run's alphabet, semicolon-separated; every
        /// entry is pre-interned so behaviours bind exactly as the SNBT
        /// path's EXTRA_STATES did.
        #[allow(clippy::too_many_arguments)]
        pub fn from_blocks(
            bx: i32,
            by: i32,
            bz: i32,
            travel: i32,
            x_off: i32,
            palette: &DiplomatStr,
            cells: &[u16],
            air_index: u16,
            settle: TickSettleMode,
            origin_x: i32,
            origin_y: i32,
            origin_z: i32,
        ) -> Result<Box<TickSimulation>, NucleationError> {
            let palette =
                std::str::from_utf8(palette).map_err(|_| NucleationError::InvalidArgument)?;
            let pal: Vec<String> = palette
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            let structure =
                super::structure_from_blocks(bx, by, bz, travel, x_off, &pal, cells, air_index)
                    .map_err(|_| NucleationError::InvalidArgument)?;
            let extras: Vec<&str> = pal.iter().map(String::as_str).collect();
            let sim = super::wire_simulation(
                &structure,
                mc_tick::Pos::new(origin_x, origin_y, origin_z),
                settle,
                &extras,
                None,
            )
            .map_err(|_| NucleationError::Simulation)?;
            Ok(Box::new(TickSimulation {
                sim,
                checkpoints: Vec::new(),
                stopped_timeline: None,
            }))
        }

        /// Evaluate a whole batch of kicked flights inside the engine — one
        /// wasm call per generation chunk instead of a dozen boundary calls
        /// per machine. `cells` holds N genomes concatenated (each
        /// `bx*by*bz` entries), `kicks` N structure-space `[x,y,z]` triples.
        /// The flight protocol, probe schedule and gait detection mirror the
        /// app's evalCore exactly; `early_exit` stops provably-frozen
        /// machines at tick 40 without changing any reported value. Writes
        /// JSON rows `[n0, startCom, startMinX, startMaxX, comAtMoveCheck |
        /// null, comAtMid, period, n1, endCom, endMinX, endMaxX]`.
        #[allow(clippy::too_many_arguments)]
        pub fn eval_flight_batch(
            bx: i32,
            by: i32,
            bz: i32,
            travel: i32,
            x_off: i32,
            palette: &DiplomatStr,
            cells: &[u16],
            air_index: u16,
            kicks: &[i32],
            eval_ticks: u32,
            seed: i64,
            must_move_by_tick: i32,
            need_period: bool,
            early_exit: bool,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let palette =
                std::str::from_utf8(palette).map_err(|_| NucleationError::InvalidArgument)?;
            let pal: Vec<String> = palette
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            let extras: Vec<&str> = pal.iter().map(String::as_str).collect();
            let volume = (bx.max(0) as usize) * (by.max(0) as usize) * (bz.max(0) as usize);
            if volume == 0 || cells.len() % volume != 0 || kicks.len() != (cells.len() / volume) * 3
            {
                return Err(NucleationError::InvalidArgument);
            }
            let n_genomes = cells.len() / volume;
            // Wire ONE empty-corridor sim (registry, behaviours, physics
            // tables — the expensive part), checkpoint it pristine, and per
            // genome restore + place. Construction cost is paid once per
            // batch instead of once per machine.
            let empty = vec![air_index; volume];
            let empty_structure =
                super::structure_from_blocks(bx, by, bz, travel, x_off, &pal, &empty, air_index)
                    .map_err(|_| NucleationError::InvalidArgument)?;
            let mut sim = super::wire_simulation(
                &empty_structure,
                mc_tick::Pos::new(0, 0, 0),
                TickSettleMode::Quiet,
                &extras,
                None,
            )
            .map_err(|_| NucleationError::Simulation)?;
            let pristine = sim.checkpoint();
            let mut json = String::from("[");
            for g in 0..n_genomes {
                let slice = &cells[g * volume..(g + 1) * volume];
                let structure =
                    super::structure_from_blocks(bx, by, bz, travel, x_off, &pal, slice, air_index)
                        .map_err(|_| NucleationError::InvalidArgument)?;
                sim.restore(&pristine);
                {
                    let (registry, world) = sim.registry_and_world_mut();
                    structure.place(world, registry, mc_tick::Pos::new(0, 0, 0));
                }
                // Quiet settle for the placed genome, exactly as
                // wire_simulation would have done for a fresh sim.
                let order = structure.placement_order(
                    mc_tick::vanilla::is_collision_full_cube,
                    mc_tick::vanilla::has_dynamic_shape,
                );
                sim.place_on_place(&order);
                sim.record();
                let kick = (kicks[g * 3], kicks[g * 3 + 1], kicks[g * 3 + 2]);
                let row = super::fly_on(
                    &mut sim,
                    kick,
                    eval_ticks,
                    seed,
                    must_move_by_tick,
                    need_period,
                    early_exit,
                )
                .map_err(|_| NucleationError::Simulation)?;
                if g > 0 {
                    json.push(',');
                }
                json.push('[');
                for (i, v) in row.iter().enumerate() {
                    if i > 0 {
                        json.push(',');
                    }
                    if v.is_nan() {
                        json.push_str("null");
                    } else {
                        let _ = write!(json, "{v:?}");
                    }
                }
                json.push(']');
            }
            json.push(']');
            let _ = write!(out, "{json}");
            Ok(())
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
            let state = std::str::from_utf8(state).map_err(|_| NucleationError::InvalidArgument)?;
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
            let descriptor = self
                .sim
                .registry()
                .descriptor(id)
                .unwrap_or("minecraft:air");
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

        /// Render a schematic as gametest-flavor structure SNBT — the text
        /// `from_snbt` and the corpus/render tooling consume. Lets hosts hand
        /// a converted `.litematic`/`.schem` to the video renderer.
        pub fn gametest_snbt(schematic: &Schematic, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", super::to_gametest_snbt(&schematic.0));
        }

        /// Report blocks whose behaviour is defined by block-entity data the
        /// file does not carry.
        ///
        /// Some exporters write the blocks and drop the block entities. The
        /// build then loads clean and simulates *wrongly but plausibly*: a
        /// comparator with no `OutputSignal` reads 0, a barrel holding the
        /// item that latched a repeater reads empty, and the door quietly
        /// fails to reset. Two files with identical block arrays get
        /// different verdicts and nothing says why. `0.45_4x4_funnel.schem`
        /// is exactly this — 4 comparators, 2 furnaces, `BlockEntities` of
        /// length 0, while its `.litematic` twin carries all 9.
        ///
        /// This does not refuse the build; it names the doubt so a host can.
        /// JSON: `{"present":N,"missing_total":N,"missing":[{"name":..,
        /// "count":N}],"summary":"..."}` — `summary` is empty when nothing
        /// is missing, and otherwise a sentence fit to show as-is.
        pub fn block_entity_audit_json(schematic: &Schematic, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", super::block_entity_audit(&schematic.0));
        }

        /// Start (or stop) recording every delivered redstone update.
        ///
        /// Off by default and much larger than the block-change log — a door's
        /// cycle runs several updates per change — so a propagation view asks
        /// for it explicitly and pages with
        /// [`TickSimulation::updates_json_between`].
        ///
        /// Switching it off keeps what was recorded; use
        /// [`TickSimulation::clear_updates`] to free it.
        pub fn record_updates(&mut self, on: bool) {
            self.sim.record_updates(on);
        }

        /// Drop the recorded updates without changing whether recording is on.
        ///
        /// A cycle of a 6x6 door is tens of megabytes of log, so a page that
        /// certifies several builds on one instance needs to release one
        /// before recording the next.
        pub fn clear_updates(&mut self) {
            self.sim.clear_updates();
        }

        /// Start recording a run timeline from the current tick.
        ///
        /// A timeline is what makes a span of simulation reviewable after the
        /// fact: block deltas, the inputs that caused them and the piston
        /// strokes they drove, plus one whole-world frame to replay them from.
        /// Off by default — a simulation used for timing should not pay for it.
        ///
        /// Called again, it restarts from the current tick, and the previously
        /// stopped span is released.
        ///
        /// Starting a recording also wipes the plain block-change log that
        /// [`TickSimulation::changes_json`] and [`TickSimulation::changes_count`]
        /// read back to empty — a separate reset from
        /// [`TickSimulation::record_updates`]/[`TickSimulation::clear_updates`],
        /// which govern a different log. A host holding a cursor into the
        /// change log (the sim lab keeps a cumulative one) must reset that
        /// cursor when it calls this, or it will read past the end of a log
        /// that is no longer the one it was walking.
        pub fn record_timeline(&mut self) {
            self.stopped_timeline = None;
            self.sim.record_timeline();
        }

        /// End the recording, keeping the span readable.
        ///
        /// This is a host's Stop button, and it is not a rewind: the span stays
        /// readable and exportable until the next
        /// [`TickSimulation::record_timeline`], while the simulation is free to
        /// run on without the recording following it. No-op if nothing was
        /// recording.
        pub fn stop_timeline(&mut self) {
            if let Some(timeline) = self.sim.stop_timeline() {
                self.stopped_timeline = Some(timeline);
            }
        }

        /// Which timeline a read query answers from.
        ///
        /// The live recorder if one is running, otherwise the span the last
        /// [`TickSimulation::stop_timeline`] ended — a stopped span stays
        /// readable until the next [`TickSimulation::record_timeline`]. Every
        /// timeline reader resolves through here so that they cannot disagree
        /// about which recording the host is looking at.
        ///
        /// It hands back a **borrowed** view because most readers only want the
        /// event vectors, and a host draws a timeline strip by polling. An
        /// owned `RunTimeline` carries the initial frame — every non-air block
        /// in the world, near a megabyte on a real build — so resolving to one
        /// here would copy that on every poll, which is the unbounded copying
        /// this recording model exists to avoid. Readers that genuinely need a
        /// whole timeline call `.to_timeline()` on the result, and say in their
        /// own docs that they pay for it.
        fn timeline(&self) -> Option<mc_tick::timeline::TimelineView<'_>> {
            self.sim.timeline_view().or_else(|| {
                self.stopped_timeline
                    .as_ref()
                    .map(mc_tick::timeline::TimelineView::of)
            })
        }

        /// Where the recorded run was busy, as JSON:
        /// `{"start":T,"end":T,"ticks":[{"tick":T,"changes":N,"inputs":N,
        ///   "pistons":N}]}`.
        ///
        /// The strip a host draws to let someone pick a span worth exporting.
        /// Only ticks that did something appear: an idle tick is **absent**
        /// rather than present with zeroes, so a build that sits still does not
        /// advance the strip and a long quiet run stays cheap to send.
        ///
        /// `{"start":0,"end":0,"ticks":[]}` when nothing has been recorded.
        pub fn timeline_activity_json(&self, out: &mut DiplomatWrite) {
            let Some(timeline) = self.timeline() else {
                let _ = write!(out, "{{\"start\":0,\"end\":0,\"ticks\":[]}}");
                return;
            };
            // Counted straight off the three borrowed event vectors: this
            // copies nothing, unlike the other timeline queries (no world-sized
            // clone, no frame or digest rebuild). But it is not free — it is
            // O(changes + inputs + pistons), re-scanning the whole accumulated
            // log into a fresh `BTreeMap` on every call, so a host should choose
            // its poll rate knowing the cost grows with the recording, not just
            // call this as often as it likes.
            let mut active: std::collections::BTreeMap<u64, [u32; 3]> =
                std::collections::BTreeMap::new();
            for change in timeline.changes {
                active.entry(change.tick).or_default()[0] += 1;
            }
            for input in timeline.inputs {
                active.entry(input.tick()).or_default()[1] += 1;
            }
            for piston in timeline.pistons {
                active.entry(piston.tick).or_default()[2] += 1;
            }
            let mut json = format!(
                "{{\"start\":{},\"end\":{},\"ticks\":[",
                timeline.start_tick, timeline.end_tick
            );
            for (i, (tick, counts)) in active.iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                let _ = write!(
                    json,
                    "{{\"tick\":{},\"changes\":{},\"inputs\":{},\"pistons\":{}}}",
                    tick, counts[0], counts[1], counts[2]
                );
            }
            json.push_str("]}");
            let _ = write!(out, "{json}");
        }

        /// Exact and translated recurrence in the timeline a read query would
        /// resolve to (see [`Self::timeline`]), as JSON:
        /// `{"exact":{"start":T,"end":T,"period":N,"drift":[x,y,z]}|null,
        ///   "translated":{...}|null}`.
        ///
        /// **An absent cycle is `null`, not an error.** Most builds — an
        /// adder, a door — never repeat their own state, and that is the
        /// ordinary outcome, not a failed search.
        ///
        /// **O(ticks × blocks): replays the whole recorded span to build one
        /// digest per tick boundary**, then rebuilds full frames for the
        /// handful of candidates that survive. This is an on-demand "find
        /// cycles" action for a host UI button, never something to call per
        /// tick or per frame — poll [`Self::timeline_activity_json`] instead.
        ///
        /// Materialises the whole recorded timeline once to answer this call
        /// (an owned `RunTimeline`'s `initial` frame copies every non-air
        /// block) — acceptable for one on-demand press, not for a loop.
        ///
        /// `{"exact":null,"translated":null}` when nothing has been recorded.
        pub fn timeline_cycles_json(&self, out: &mut DiplomatWrite) {
            let Some(view) = self.timeline() else {
                let _ = write!(out, "{{\"exact\":null,\"translated\":null}}");
                return;
            };
            let timeline = view.to_timeline();
            let report = timeline.detect_cycles(self.sim.registry());
            let _ = write!(
                out,
                "{{\"exact\":{},\"translated\":{}}}",
                super::cycle_json(report.exact),
                super::cycle_json(report.translated)
            );
        }

        /// Project `[start_tick, end_tick)` of the timeline a read query would
        /// resolve to (see [`Self::timeline`]) into the animated-GLB mesher's
        /// `Timeline` JSON — `{"origin":[x,y,z],"tick_ms":F,
        /// "events":[{"kind":"set_block"|"piston",...}]}` — via
        /// `crate::tick_timeline::mesher_timeline_json`.
        ///
        /// **Materialises the whole recorded timeline to answer this call**
        /// (an owned `RunTimeline`'s `initial` frame copies every non-air
        /// block in the world) — this is an on-demand "export this
        /// selection" action, not something to call per frame or poll.
        ///
        /// Fails if no timeline has been recorded, or if `start_tick..
        /// end_tick` is empty or outside the recorded span.
        pub fn animation_timeline_json(
            &self,
            start_tick: u32,
            end_tick: u32,
            tick_ms: f32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let Some(view) = self.timeline() else {
                super::set_last_error("no timeline has been recorded on this simulation");
                return Err(NucleationError::NotFound);
            };
            // One materialisation for this call: `select_ticks` and the
            // projection below both need an owned `RunTimeline` to read, and
            // there is nothing else in this call to share the copy with.
            let timeline = view.to_timeline();
            let selection = timeline
                .select_ticks(u64::from(start_tick), u64::from(end_tick))
                .map_err(|e| {
                    super::set_last_error(e.to_string());
                    NucleationError::InvalidArgument
                })?;
            // The `ProjectionWarnings` are discarded here: the host has no
            // channel to show them yet, and inventing one before anything
            // needs it is YAGNI.
            let (json, _warnings) = crate::tick_timeline::mesher_timeline_json(
                &timeline,
                selection,
                self.sim.registry(),
                tick_ms,
            )
            .map_err(|e| {
                super::set_last_error(e);
                NucleationError::Simulation
            })?;
            let _ = write!(out, "{json}");
            Ok(())
        }

        /// The selection's starting scene — `[start_tick, end_tick)` of the
        /// timeline a read query would resolve to (see [`Self::timeline`]) —
        /// as schematic bytes, base64-encoded.
        ///
        /// A WASM handle cannot cross a worker boundary, so this exists for a
        /// host to hand the bytes to a worker, which rebuilds the schematic
        /// with `Schematic.fromData`.
        ///
        /// **Materialises the whole recorded timeline to answer this call**
        /// — see [`Self::animation_timeline_json`]; an on-demand export
        /// action, not a per-frame poll.
        ///
        /// Fails if no timeline has been recorded, or if `start_tick..
        /// end_tick` is empty or outside the recorded span.
        pub fn selection_schematic_b64(
            &self,
            start_tick: u32,
            end_tick: u32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let Some(view) = self.timeline() else {
                super::set_last_error("no timeline has been recorded on this simulation");
                return Err(NucleationError::NotFound);
            };
            let timeline = view.to_timeline();
            let selection = timeline
                .select_ticks(u64::from(start_tick), u64::from(end_tick))
                .map_err(|e| {
                    super::set_last_error(e.to_string());
                    NucleationError::InvalidArgument
                })?;
            let schematic = crate::tick_timeline::selection_schematic(
                &timeline,
                selection,
                self.sim.registry(),
            )
            .map_err(|e| {
                super::set_last_error(e);
                NucleationError::Simulation
            })?;
            // Same serialisation `Schematic::to_schematic_b64` uses, so a
            // worker's `Schematic.fromData` reads this back identically.
            let data = crate::formats::schematic::to_schematic(&schematic).map_err(|e| {
                super::set_last_error(e.to_string());
                NucleationError::Serialize
            })?;
            // `schematic`'s b64 helper, not `meshing`'s: this module is gated on
            // `mc-tick` alone and must stay buildable without `meshing`.
            let _ = write!(out, "{}", super::super::schematic::b64(&data));
            Ok(())
        }

        /// How many updates have been recorded — page before pulling them.
        pub fn updates_count(&self) -> u32 {
            self.sim.recorded_updates().len() as u32
        }

        /// Every recorded update, in delivery order.
        ///
        /// `seq` counts from 0 within each tick: that is the sub-tick axis, and
        /// `(tick, seq)` is the order the engine actually delivered them in.
        /// `state` is the block as it stood **at dispatch time**, which is what
        /// makes intra-tick order legible — a snapshot cannot show it.
        pub fn updates_json(&self, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", super::updates_json_range(&self.sim, 0, u64::MAX));
        }

        /// The recorded updates for ticks in `[from_tick, to_tick)`.
        ///
        /// The whole log for a 6x6 door's cycle is megabytes; a scrubber only
        /// ever shows one tick, so it should ask for one tick.
        pub fn updates_json_between(&self, from_tick: u32, to_tick: u32, out: &mut DiplomatWrite) {
            let _ = write!(
                out,
                "{}",
                super::updates_json_range(&self.sim, u64::from(from_tick), u64::from(to_tick))
            );
        }

        /// Per-tick, per-cell update counts for ticks in `[from_tick, to_tick)`.
        ///
        /// The resolution playback should run at: `{phases, ticks:[{tick, total,
        /// cells:[{p:[x,y,z], n, nb, sh, ph:[…]}]}]}`, where `nb`/`sh` split
        /// neighbour from shape and `ph` indexes the `phases` legend. Collapses
        /// a tick's tens of thousands of updates into a few hundred cells.
        pub fn updates_heat_json(&self, from_tick: u32, to_tick: u32, out: &mut DiplomatWrite) {
            let _ = write!(
                out,
                "{}",
                super::updates_heat_range(&self.sim, u64::from(from_tick), u64::from(to_tick))
            );
        }

        /// One tick's updates in delivery order, as parallel arrays.
        ///
        /// For stepping *within* a tick: `seq` is the array index, `pos` is flat
        /// x,y,z triples, `kind`/`phase`/`from` are integer codes with legends
        /// in the payload, and `state` indexes a deduplicated `states` table.
        pub fn updates_wave_json(&self, tick: u32, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", super::updates_wave(&self.sim, u64::from(tick)));
        }

        /// Every block a piston currently has in flight, as JSON:
        /// `[{"to":[x,y,z],"from":[x,y,z],"state":"...","carried":"...",
        ///    "carried_short":"..."|null,"remains":"..."|null,"dir":"east",
        ///    "extending":bool,"started":T,"lands":T,"source_piston":bool}]`.
        ///
        /// Draw `carried` travelling `from` -> `to`, and `remains` (when it is
        /// not null) parked at `to` for the whole move. They differ from
        /// `state` — what actually lands — only for a retracting piston, whose
        /// body stays put while its head comes home; vanilla's
        /// `PistonHeadRenderer` splits exactly these two slots.
        ///
        /// `carried_short` is the same arm with `short=true`. Draw it while the
        /// head is **within half a block of its body** — `progress <= 0.5`
        /// extending, `progress >= 0.5` retracting — or the shaft passes
        /// visibly through the back of the piston as it comes home. Which form
        /// to use is yours; naming the state is the engine's.
        ///
        /// What a renderer needs to animate a stroke, from the simulator that
        /// dispatched it. The block-change stream cannot answer this: it says a
        /// cell became a `moving_piston` placeholder, not which block set off,
        /// which cell it left, or which tick it arrives — so a host that
        /// reconstructs strokes from changes is reimplementing piston mechanics
        /// downstream of the engine, and animating on a clock the simulation
        /// does not share. That desync is what draws a block twice, leaves a
        /// gap where one should be, and shears a piston head off its load.
        ///
        /// `started` and `lands` are tick numbers in the engine's frame, where
        /// [`Self::tick_count`] counts *completed* ticks: after stepping to
        /// `tick_count == t`, a flight's progress is
        /// `(t - started) / (lands - started)`, clamped to 1. Draw it while it
        /// is listed and drop it when it stops being listed — the same call
        /// that stops reporting it is the tick the real block is written, so
        /// there is no frame with both and none with neither.
        pub fn moving_blocks_json(&self, out: &mut DiplomatWrite) {
            let mut json = String::from("[");
            for (i, m) in self.sim.moving_blocks().iter().enumerate() {
                if i > 0 {
                    json.push(',');
                }
                let state = self.sim.registry().descriptor(m.state).unwrap_or("?");
                let carried = self.sim.registry().descriptor(m.carried).unwrap_or("?");
                let quoted = |s: Option<mc_tick::StateId>| {
                    match s.and_then(|s| self.sim.registry().descriptor(s)) {
                        Some(descriptor) => format!("\"{descriptor}\""),
                        None => "null".to_string(),
                    }
                };
                let carried_short = quoted(m.carried_short);
                let remains = quoted(m.remains);
                let _ = write!(
                    json,
                    "{{\"to\":[{},{},{}],\"from\":[{},{},{}],\"state\":\"{}\",\
                     \"carried\":\"{}\",\"carried_short\":{},\"remains\":{},\
                     \"dir\":\"{}\",\"extending\":{},\"started\":{},\"lands\":{},\
                     \"source_piston\":{}}}",
                    m.to.x,
                    m.to.y,
                    m.to.z,
                    m.from.x,
                    m.from.y,
                    m.from.z,
                    state,
                    carried,
                    carried_short,
                    remains,
                    m.travel.name(),
                    m.extending,
                    m.started_on,
                    m.lands_on,
                    m.source_piston
                );
            }
            json.push(']');
            let _ = write!(out, "{json}");
        }

        /// Drop the recorded block changes without stopping recording.
        ///
        /// The log grows for as long as the simulation runs and nothing
        /// empties it, so a long-running host — a browser session driving
        /// thousands of ticks — accumulates every block change forever. A
        /// host that has already consumed [`TickSimulation::changes_json`]
        /// can say so here and keep recording on. A host holding a cursor
        /// into the change log must reset that cursor when it calls this, or
        /// it will read past the end of a log that is no longer the one it
        /// was walking — the same hazard [`TickSimulation::record_timeline`]
        /// names for its own reset of this log.
        ///
        /// Refuses — and leaves the log untouched — while
        /// [`TickSimulation::record_timeline`] is recording: a run timeline
        /// is a seed frame plus this same log, and every timeline reader
        /// trusts that the log describes every mutation since recording
        /// began. Clearing it out from under a live recording would make
        /// replay silently wrong rather than fail loudly. Returns `true` if
        /// the log was cleared, `false` if the call was refused — following
        /// [`TickSimulation::run_until_quiescent`]'s convention of reporting
        /// whether the call achieved what it was asked, rather than swallowing
        /// a no-op.
        pub fn clear_changes(&mut self) -> bool {
            self.sim.clear_recorded()
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
                    cart.pos[0],
                    cart.pos[1],
                    cart.pos[2],
                    cart.vel[0],
                    cart.vel[1],
                    cart.vel[2],
                );
            }
            json.push_str("],\"frozen\":[");
            let mut first = true;
            for body in self.sim.entity_bodies() {
                if body.is_minecart {
                    continue;
                }
                if !first {
                    json.push(',');
                }
                first = false;
                // `size` is the measured hitbox, not a guess a viewer would
                // otherwise have to make from the kind name — a boat is
                // 1.375 x 0.5625 and nothing about "minecraft:oak_boat" says
                // so. `leashed` distinguishes a tethered boat from one resting
                // on the ground; they are the same box and not the same thing.
                let _ = write!(
                    json,
                    "{{\"id\":{},\"kind\":\"{}\",\"pos\":[{},{},{}],\"size\":[{},{},{}],\"leashed\":{}}}",
                    body.id,
                    body.kind,
                    (body.min[0] + body.max[0]) / 2.0,
                    body.min[1],
                    (body.min[2] + body.max[2]) / 2.0,
                    body.max[0] - body.min[0],
                    body.max[1] - body.min[1],
                    body.max[2] - body.min[2],
                    body.leashed,
                );
            }
            json.push_str("]}");
            let _ = write!(out, "{json}");
        }

        /// Which `Entity.load` Motion semantics this run uses:
        /// `"clamp_abs_ten"` (DataVersion <= 4556 — NaN survives a cold load)
        /// or `"drop_non_finite"` (>= 4671 — it does not).
        ///
        /// Exposed because a door built on nan carts is a *different machine*
        /// under the two, and a caller that cannot tell them apart cannot
        /// report why it came apart.
        pub fn motion_semantics(&self, out: &mut DiplomatWrite) {
            let name = match self.sim.motion_semantics() {
                mc_tick::MotionSemantics::ClampAbsTen => "clamp_abs_ten",
                mc_tick::MotionSemantics::DropNonFinite => "drop_non_finite",
            };
            let _ = write!(out, "{name}");
        }

        /// How many times an entity stood in a **retracting** piston's sweep
        /// that the engine could not reproduce.
        ///
        /// A tripwire from when retraction was unmodelled — extension
        /// displacement was measured and implemented while
        /// `tools/gametest/captures/piston_pull.entities.log`'s sub-0.03
        /// movements, not uniformly backwards, had no model here. All three
        /// retraction geometries are implemented now, so this reports **0**,
        /// including on the record 3x3 door, which used to name six. It is kept
        /// because the next geometry that turns out not to be covered should be
        /// reported rather than guessed at: non-zero means this run leaned on
        /// behaviour we do not reproduce and its result is not trustworthy.
        pub fn piston_retract_contacts(&self) -> u32 {
            self.sim.piston_retract_contacts().len() as u32
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
                let named =
                    |needle: &str| super::is_named(from, needle) || super::is_named(to, needle);
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

        /// Static structural analysis of the build standing in this world.
        ///
        /// One call, one JSON document: adhesion groups, piston/observer/source
        /// nodes, the four edge kinds, every minimal self-translating subgraph
        /// (the engine), payload, kickers, dead weight, and any proof that the
        /// machine cannot move.
        ///
        /// The analysis lives in the engine rather than in the caller on
        /// purpose. Every "what would this piston move?" answer comes from
        /// `resolve_push`/`resolve_pull` — the same oracle-verified resolver the
        /// tick loop runs — and a second copy of Minecraft's push rules written
        /// on the far side of this boundary would drift from it silently.
        pub fn machine_graph_json(&self, out: &mut DiplomatWrite) {
            let graph = super::analyse_world(self.sim.world(), self.sim.registry());
            let _ = write!(out, "{}", graph.to_json());
        }

        /// GA pre-filter: static verdicts for a whole batch of genomes.
        ///
        /// Same flat-cell layout as [`Self::eval_flight_batch`], and meant to run
        /// immediately before it: whatever this rejects never needs simulating.
        /// Writes one row per genome, `[rejected, rejected_for_sustained,
        /// engine_cell_count, payload_cell_count, dead_cell_count, "codes"]`.
        ///
        /// The registry, behaviour table and movability rules are built once for
        /// the batch — building them per genome costs more than the analysis.
        #[allow(clippy::too_many_arguments)]
        pub fn machine_graph_batch_json(
            bx: i32,
            by: i32,
            bz: i32,
            travel: i32,
            x_off: i32,
            palette: &DiplomatStr,
            cells: &[u16],
            air_index: u16,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let palette =
                std::str::from_utf8(palette).map_err(|_| NucleationError::InvalidArgument)?;
            let json =
                super::machine_graph_batch(bx, by, bz, travel, x_off, palette, cells, air_index)
                    .map_err(|_| NucleationError::InvalidArgument)?;
            let _ = write!(out, "{json}");
            Ok(())
        }
    }
}

/// Static verdicts for a batch of genomes, as JSON rows.
///
/// Split out of the bridge method so the GA's contract can be tested without
/// standing up a `DiplomatWrite`.
fn machine_graph_batch(
    bx: i32,
    by: i32,
    bz: i32,
    travel: i32,
    x_off: i32,
    palette: &str,
    cells: &[u16],
    air_index: u16,
) -> Result<String, String> {
    use std::fmt::Write as _;

    let pal: Vec<String> = palette
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let volume = (bx.max(0) as usize) * (by.max(0) as usize) * (bz.max(0) as usize);
    if volume == 0 || cells.is_empty() || cells.len() % volume != 0 {
        return Err("cells length is not a whole number of bbox volumes".into());
    }
    let n_genomes = cells.len() / volume;

    // One registry, one behaviour table and one set of movability rules for the
    // whole batch. Building them per genome costs more than the analysis does.
    let mut registry = mc_tick::StateRegistry::new();
    for descriptor in &pal {
        registry.intern(descriptor).map_err(|e| format!("{e:?}"))?;
    }
    mc_tick::intern_companions(&mut registry);
    let mut table = mc_tick::BehaviourTable::default();
    let rules = mc_tick::register_all_at(&mut registry, &mut table, mc_tick::Pos::new(0, 0, 0));

    let empty = vec![air_index; volume];
    let reference = structure_from_blocks(bx, by, bz, travel, x_off, &pal, &empty, air_index)?;
    let bounds = reference.bounds(4);

    let mut json = String::from("[");
    for g in 0..n_genomes {
        let slice = &cells[g * volume..(g + 1) * volume];
        let structure = structure_from_blocks(bx, by, bz, travel, x_off, &pal, slice, air_index)?;
        let mut world = mc_tick::World::new(bounds);
        structure.place(&mut world, &mut registry, mc_tick::Pos::new(0, 0, 0));
        let graph = mc_tick::machine_graph::analyse(&world, &registry, &rules);
        let codes: Vec<&str> = graph.rejections.iter().map(|r| r.code).collect();
        let engine_cells: usize = graph.engines.iter().map(|e| e.cells.len()).sum();
        if g > 0 {
            json.push(',');
        }
        let _ = write!(
            json,
            "[{},{},{},{},{},\"{}\"]",
            graph.rejected(),
            graph.rejected_for_sustained(),
            engine_cells,
            graph.payload.len(),
            graph.dead_weight.len(),
            codes.join("|")
        );
    }
    json.push(']');
    Ok(json)
}

/// Build the machine graph for a world, deriving the movability rules it needs.
///
/// `Simulation` does not keep the [`mc_tick::vanilla::VanillaRules`] its wiring
/// produced, so they are rebuilt against a *clone* of the registry: cloning
/// preserves every existing [`mc_tick::StateId`], so the rules are valid for the
/// caller's world, and re-registering cannot disturb a live simulation.
fn analyse_world(
    world: &mc_tick::World,
    registry: &mc_tick::StateRegistry,
) -> mc_tick::machine_graph::MachineGraph {
    let mut scratch = registry.clone();
    let mut table = mc_tick::BehaviourTable::default();
    let rules = mc_tick::register_all_at(&mut scratch, &mut table, mc_tick::Pos::new(0, 0, 0));
    mc_tick::machine_graph::analyse(world, registry, &rules)
}

#[cfg(test)]
mod tests {
    use super::{block_entity_audit, machine_graph_batch, needs_block_entity, to_gametest_snbt};
    use crate::{BlockState, UniversalSchematic};

    /// `{simulate=true}` places through the engine: a wire set next to a
    /// redstone block comes back with its real power and connections, not the
    /// default state — and the write-back also carries whatever the placement
    /// caused elsewhere.
    #[test]
    fn simulate_tag_derives_wire_power_and_connections() {
        let mut schem = UniversalSchematic::new("wired".into());
        for x in 0..4 {
            schem.set_block(x, 0, 0, &BlockState::new("minecraft:smooth_stone"));
        }
        schem.set_block(0, 1, 0, &BlockState::new("minecraft:redstone_block"));
        schem
            .set_block_from_string(1, 1, 0, "minecraft:redstone_wire{simulate=true}")
            .expect("simulated placement");
        let wire = schem.get_block(1, 1, 0).expect("wire exists").to_string();
        assert!(
            wire.contains("power=15"),
            "wire next to a redstone block reads 15, got {wire}"
        );
        assert!(
            wire.contains("west=side"),
            "wire connects toward the block powering it, got {wire}"
        );
    }

    /// The tag on an isolated placement (or the first block of an empty
    /// schematic) degrades to a plain write instead of refusing.
    #[test]
    fn simulate_tag_on_an_isolated_block_is_a_plain_write() {
        let mut schem = UniversalSchematic::new("empty".into());
        schem
            .set_block_from_string(0, 0, 0, "minecraft:redstone_wire{simulate=true}")
            .expect("plain write");
        let wire = schem.get_block(0, 0, 0).expect("wire exists").to_string();
        assert!(wire.contains("redstone_wire"), "got {wire}");
    }

    /// Combining simulate with the other brace shorthands is refused, not
    /// half-honoured.
    #[test]
    fn simulate_tag_refuses_company_in_the_braces() {
        let mut schem = UniversalSchematic::new("combo".into());
        let err = schem
            .set_block_from_string(0, 0, 0, "minecraft:barrel{signal=3,simulate=true}")
            .unwrap_err();
        assert!(err.contains("only tag"), "got {err}");
    }

    /// The GA's pre-filter contract, over the batch path it actually calls.
    ///
    /// Two genomes in one call: engine B, which must survive both tiers, and a
    /// lone slime block, which must be rejected outright. The order of the rows
    /// is the order of the genomes — the app maps them back positionally, so a
    /// silent reordering here would score the wrong machines zero.
    #[test]
    fn the_batch_prefilter_keeps_an_engine_and_rejects_a_lone_block() {
        // Same palette shape the app builds: air first, then the alphabet.
        const PALETTE: &str = "minecraft:air;minecraft:slime_block;\
            minecraft:sticky_piston[extended=false,facing=east];\
            minecraft:sticky_piston[extended=false,facing=west];\
            minecraft:observer[facing=east,powered=false];\
            minecraft:observer[facing=west,powered=false]";
        // bbox 4x1x2. `structure_from_blocks` walks y, then z, then x.
        // z=0: obsW slime stickyW  _        z=1: _ stickyE slime obsE
        let engine_b: [u16; 8] = [5, 1, 3, 0, 0, 2, 1, 4];
        let lone_slime: [u16; 8] = [0, 1, 0, 0, 0, 0, 0, 0];
        let mut cells = engine_b.to_vec();
        cells.extend_from_slice(&lone_slime);

        let json =
            machine_graph_batch(4, 1, 2, 26, 1, PALETTE, &cells, 0).expect("batch analysis runs");

        // [rejected, rejected_for_sustained, engine, payload, dead, "codes"]
        let rows: Vec<&str> = json
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split("],[")
            .map(|r| r.trim_matches(|c| c == '[' || c == ']'))
            .collect();
        assert_eq!(rows.len(), 2, "one row per genome: {json}");

        assert!(
            rows[0].starts_with("false,false,"),
            "engine B must survive both filter tiers, got {}",
            rows[0]
        );
        assert!(
            rows[0].contains(",6,"),
            "engine B's engine is its six blocks, got {}",
            rows[0]
        );
        assert!(
            rows[1].starts_with("true,true,"),
            "a lone slime block cannot move, got {}",
            rows[1]
        );
        assert!(
            rows[1].contains("no_piston"),
            "and the reason is why: {}",
            rows[1]
        );
    }

    /// A 1.12 build must reach the engine flattened.
    ///
    /// The trap this pins is `minecraft:slime`: in 1.12 that is the *slime
    /// block*, and no modern block has the id — so an unconverted bore loads
    /// with every sticky cell inert and simulates as a machine that cannot
    /// fly, without erroring anywhere.
    #[test]
    fn pre_flattening_ids_are_converted_before_the_engine_sees_them() {
        let mut schem = UniversalSchematic::new("legacy".into());
        schem.metadata.source_data_version = Some(1343); // 1.12.2
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:slime"));
        // 1.12 stored the sub-type as a property; the flattening rules key on
        // it, so a realistic file carries `variant` and a bare id does not
        // convert. Real .litematic/.schem imports always have it.
        schem.set_block(
            1,
            0,
            0,
            &BlockState::new("minecraft:stonebrick").with_property("variant", "stonebrick"),
        );

        let snbt = to_gametest_snbt(&schem);
        assert!(
            snbt.contains("minecraft:slime_block"),
            "slime block not flattened: {snbt}"
        );
        assert!(
            snbt.contains("minecraft:stone_bricks"),
            "stone brick not flattened: {snbt}"
        );
        assert!(
            !snbt.contains("\"minecraft:slime\""),
            "the 1.12 id survived into the engine's input: {snbt}"
        );
    }

    #[test]
    fn modern_builds_are_passed_through_untouched() {
        let mut schem = UniversalSchematic::new("modern".into());
        schem.metadata.source_data_version = Some(3955);
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:slime_block"));
        assert!(to_gametest_snbt(&schem).contains("minecraft:slime_block"));
    }

    #[test]
    fn audit_names_blocks_whose_block_entity_is_missing() {
        let mut schem = UniversalSchematic::new("stripped".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:comparator"));
        schem.set_block(1, 0, 0, &BlockState::new("minecraft:comparator"));
        schem.set_block(2, 0, 0, &BlockState::new("minecraft:furnace"));
        // Carries no block-entity state that ticks; must not be reported.
        schem.set_block(3, 0, 0, &BlockState::new("minecraft:stone"));

        let json = block_entity_audit(&schem);
        assert!(json.contains("\"missing_total\":3"), "{json}");
        assert!(
            json.contains("\"name\":\"minecraft:comparator\",\"count\":2"),
            "{json}"
        );
        assert!(
            json.contains("\"name\":\"minecraft:furnace\",\"count\":1"),
            "{json}"
        );
        assert!(
            !json.contains("stone"),
            "a block with no ticking NBT was reported: {json}"
        );
        assert!(
            json.contains("2 comparators"),
            "summary not written: {json}"
        );
    }

    #[test]
    fn audit_is_silent_when_nothing_is_missing() {
        let mut schem = UniversalSchematic::new("plain".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:stone"));
        let json = block_entity_audit(&schem);
        assert!(json.contains("\"missing_total\":0"), "{json}");
        assert!(json.contains("\"summary\":\"\""), "{json}");
    }

    #[test]
    fn every_colour_of_shulker_box_counts_as_a_container() {
        assert!(needs_block_entity("minecraft:shulker_box"));
        assert!(needs_block_entity("minecraft:lime_shulker_box"));
        assert!(!needs_block_entity("minecraft:oak_sign"));
    }

    /// Entities survive the trip from schematic to engine input.
    ///
    /// The converter used to write a hardcoded `entities: []`, so a build whose
    /// mechanism depends on entities loaded clean and simulated as though they
    /// were not there. This asserts the whole path: emitted, re-read by the
    /// engine's own parser, with the fields that change a run intact.
    #[test]
    fn entities_round_trip_from_schematic_into_the_engines_parser() {
        use crate::entity::{Entity, NbtValue};
        use std::collections::HashMap;

        let mut schem = UniversalSchematic::new("carts".into());
        // Away from the origin on purpose: entity positions are absolute in
        // the schematic and structure-relative in the SNBT, so a missing
        // `bb.min` shift would sail through a build placed at 0,0,0.
        schem.set_block(10, 0, 5, &BlockState::new("minecraft:rail"));
        schem.set_block(12, 2, 7, &BlockState::new("minecraft:stone"));

        let mut cart = Entity::new("minecraft:minecart".into(), (10.5, 0.0625, 5.5));
        cart.nbt.insert(
            "Motion".into(),
            NbtValue::List(vec![
                NbtValue::Double(0.25),
                NbtValue::Double(0.0),
                NbtValue::Double(-0.5),
            ]),
        );
        assert!(schem.add_entity(cart));

        let mut stack = HashMap::new();
        stack.insert(
            "id".to_string(),
            NbtValue::String("minecraft:redstone".into()),
        );
        stack.insert("count".to_string(), NbtValue::Byte(7));
        let mut item = Entity::new("minecraft:item".into(), (11.5, 1.0, 6.5));
        item.nbt.insert("Item".into(), NbtValue::Compound(stack));
        item.nbt.insert("PickupDelay".into(), NbtValue::Short(40));
        assert!(schem.add_entity(item));

        let snbt = to_gametest_snbt(&schem);
        let parsed = mc_tick::Structure::parse(&snbt)
            .unwrap_or_else(|e| panic!("engine rejected our own output: {e}\n{snbt}"));

        assert_eq!(parsed.entities.len(), 2, "entities dropped: {snbt}");
        match &parsed.entities[0] {
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                assert_eq!(cart.kind, "minecraft:minecart");
                assert_eq!(
                    cart.pos,
                    [0.5, 0.0625, 0.5],
                    "position not shifted into structure space"
                );
                assert_eq!(cart.motion, [0.25, 0.0, -0.5]);
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
        match &parsed.entities[1] {
            mc_tick::structure::SpawnedEntity::Item(item) => {
                assert_eq!(item.pos, [1.5, 1.0, 1.5]);
                assert_eq!(item.item, ("minecraft:redstone".to_string(), 7));
                assert_eq!(item.pickup_delay, 40);
            }
            other => panic!("expected an item, got {other:?}"),
        }
        // The same list also reaches the item-entity view the simulator spawns from.
        assert_eq!(parsed.item_entities.len(), 1);
    }

    /// A denormal motion must not corrupt the entity it belongs to.
    ///
    /// Real furnace minecarts in the 55_3x3 door carry motions like 4.3e-59.
    /// Written with an exponent, the engine's reader takes `4.3` and `-59` as
    /// two numbers, turning a three-element `Motion` into four — which it then
    /// silently discards. The value has to be spelled out in full.
    #[test]
    fn tiny_motions_are_written_without_an_exponent() {
        use crate::entity::{Entity, NbtValue};

        let mut schem = UniversalSchematic::new("denormal".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:rail"));
        let mut cart = Entity::new("minecraft:minecart".into(), (0.5, 0.0, 0.5));
        cart.nbt.insert(
            "Motion".into(),
            NbtValue::List(vec![
                NbtValue::Double(4.27987680632209e-59),
                NbtValue::Double(0.0),
                NbtValue::Double(0.0),
            ]),
        );
        assert!(schem.add_entity(cart));

        let snbt = to_gametest_snbt(&schem);
        let (_, entities) = snbt.split_once("entities:").expect("an entities section");
        assert!(
            !entities.contains("e-") && !entities.contains("e+"),
            "an exponent reached the engine's input: {entities}"
        );

        let parsed = mc_tick::Structure::parse(&snbt).expect("parse");
        match &parsed.entities[0] {
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                assert_eq!(cart.motion, [4.27987680632209e-59, 0.0, 0.0]);
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
    }

    /// A nan cart's velocity must reach the engine as NaN, not as zero.
    ///
    /// These exact numbers are lifted from `55_3x3.zip`: a furnace minecart
    /// whose `Motion` is `[4.27987680632209e-59, 0.0, NaN]`. The NaN is the
    /// mechanism — it is what makes the cart's physics dead so it can be used
    /// as glue — so rewriting it to 0.0 turns the cart back into an ordinary
    /// one that moves, and the door quietly falls apart. This pins both halves
    /// at once: the denormal must not become an exponent, and the NaN must not
    /// become a number.
    #[test]
    fn a_nan_cart_velocity_survives_the_round_trip() {
        use crate::entity::{Entity, NbtValue};

        let mut schem = UniversalSchematic::new("nan cart".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:rail"));
        let mut cart = Entity::new("minecraft:minecart".into(), (0.5, 0.0, 0.5));
        cart.nbt.insert(
            "Motion".into(),
            NbtValue::List(vec![
                NbtValue::Double(4.27987680632209e-59),
                NbtValue::Double(0.0),
                NbtValue::Double(f64::NAN),
            ]),
        );
        assert!(schem.add_entity(cart));

        let snbt = to_gametest_snbt(&schem);
        let parsed = mc_tick::Structure::parse(&snbt)
            .unwrap_or_else(|e| panic!("a NaN motion must parse, not error: {e}\n{snbt}"));

        match &parsed.entities[0] {
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                assert_eq!(
                    cart.motion[0], 4.27987680632209e-59,
                    "denormal mangled: {snbt}"
                );
                assert_eq!(cart.motion[1], 0.0);
                assert!(
                    cart.motion[2].is_nan(),
                    "the NaN was sanitised to {} — this un-glues the door: {snbt}",
                    cart.motion[2]
                );
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
    }

    /// The ±Infinity that a nan cart is made from round-trips too.
    ///
    /// Overflowed-but-not-yet-collided carts hold these, and a build captured
    /// mid-sequence carries them. Signed, because `+Inf` and `-Inf` are what
    /// collide to produce the NaN in the first place.
    #[test]
    fn infinite_velocities_survive_the_round_trip() {
        use crate::entity::{Entity, NbtValue};

        let mut schem = UniversalSchematic::new("overflowed".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:rail"));
        let mut cart = Entity::new("minecraft:minecart".into(), (0.5, 0.0, 0.5));
        cart.nbt.insert(
            "Motion".into(),
            NbtValue::List(vec![
                NbtValue::Double(f64::INFINITY),
                NbtValue::Double(f64::NEG_INFINITY),
                NbtValue::Double(0.0),
            ]),
        );
        assert!(schem.add_entity(cart));

        let snbt = to_gametest_snbt(&schem);
        let parsed = mc_tick::Structure::parse(&snbt).expect("infinities must parse");
        match &parsed.entities[0] {
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                assert_eq!(cart.motion[0], f64::INFINITY);
                assert_eq!(
                    cart.motion[1],
                    f64::NEG_INFINITY,
                    "the sign was lost: {snbt}"
                );
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
    }

    /// The record 3x3 door sample, as a schematic.
    fn record_door_schematic() -> UniversalSchematic {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/samples/55_3x3.zip");
        let bytes = std::fs::read(&path).expect("the record-door sample must be present");
        crate::formats::world::from_world_zip(&bytes).expect("the sample loads")
    }

    /// How many of a run's minecarts hold a NaN anywhere in their velocity.
    ///
    /// The carts the record door is glued together by. Counted off the live
    /// simulation rather than off the text, because the question is what
    /// `Entity.load` did with the token, not whether the token was written.
    fn nan_carts(sim: &mc_tick::Simulation) -> usize {
        sim.minecarts()
            .iter()
            .filter(|c| c.vel.iter().any(|v| v.is_nan()))
            .count()
    }

    /// Load gametest SNBT through the shipped bridge entry point.
    fn from_snbt(snbt: &str) -> mc_tick::Simulation {
        super::ffi::TickSimulation::from_snbt(
            snbt.as_bytes(),
            super::ffi::TickSettleMode::InWorld,
            0,
            0,
            0,
            b"",
        )
        .expect("the round-tripped text must load")
        .sim
    }

    /// A schematic's own DataVersion reaches the engine through the SNBT.
    ///
    /// The write half of the round trip. `to_gametest_snbt` used to stamp the
    /// canonical oracle version unconditionally, throwing the source's away
    /// before any reader could see it.
    #[test]
    fn the_emitted_snbt_states_the_schematics_own_data_version() {
        let mut schem = UniversalSchematic::new("versioned".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:stone"));

        schem.metadata.source_data_version = Some(4082);
        let snbt = to_gametest_snbt(&schem);
        assert!(
            snbt.contains("DataVersion: 4082"),
            "the file's own version must win: {snbt}"
        );
        assert_eq!(
            mc_tick::Structure::parse(&snbt)
                .expect("parses")
                .data_version,
            Some(4082),
            "and it must survive being read back"
        );

        // No provenance is not evidence of an old save, so it falls back to the
        // canonical version — a value, not a silent absence.
        schem.metadata.source_data_version = None;
        schem.metadata.mc_version = None;
        let snbt = to_gametest_snbt(&schem);
        assert!(
            snbt.contains(&format!(
                "DataVersion: {}",
                crate::dataconverter::CANONICAL_DATA_VERSION
            )),
            "a schematic with no version must still stamp one: {snbt}"
        );
    }

    /// `gametest_snbt` → `from_snbt` must not un-glue a nan-cart build.
    ///
    /// The NaN *token* always survived the text; the *load rule* did not. The
    /// emitted SNBT hardcoded the canonical oracle version (>= 4671) and
    /// mc-tick's parser never read `DataVersion` at all, so `from_snbt` chose
    /// `drop_non_finite` for a 1.21.3 save and its six nan carts came back as
    /// ordinary carts — live physics, in a machine whose whole construction
    /// depends on dead physics.
    ///
    /// `from_schematic` is the reference: it has always read the version off the
    /// file. Both sides assert `motion_semantics` *and* the cart count, because
    /// either alone can pass for the wrong reason — the semantics could be right
    /// while the entities were dropped, and the count could be right on a build
    /// that never had a NaN.
    #[test]
    fn the_snbt_round_trip_keeps_the_record_doors_nan_carts() {
        let schematic = record_door_schematic();
        assert_eq!(
            schematic.metadata.source_data_version,
            Some(4082),
            "the record door is a 1.21.3 save — if that changed, this test is measuring nothing"
        );

        let direct = wire_record_door(super::ffi::TickSettleMode::InWorld);
        assert_eq!(
            direct.motion_semantics(),
            mc_tick::MotionSemantics::ClampAbsTen,
            "4082 is below the boundary, so a cold load keeps NaN"
        );
        assert_eq!(nan_carts(&direct), 6, "the reference path's nan carts");

        let snbt = to_gametest_snbt(&schematic);
        let round_tripped = from_snbt(&snbt);
        assert_eq!(
            round_tripped.motion_semantics(),
            mc_tick::MotionSemantics::ClampAbsTen,
            "the round trip changed which game loaded the door"
        );
        assert_eq!(
            nan_carts(&round_tripped),
            nan_carts(&direct),
            "the same door, through its own SNBT, must be the same machine"
        );
    }

    /// The negative control: after the boundary, the NaN really is dropped.
    ///
    /// Without this, the test above could pass by the engine having simply
    /// stopped sanitising anything, which would be a different bug wearing the
    /// same result. The identical door stamped 4671 must lose every nan cart.
    #[test]
    fn a_build_stamped_after_the_boundary_still_drops_its_nan_carts() {
        let mut schematic = record_door_schematic();
        schematic.metadata.source_data_version =
            Some(mc_tick::motion::FIRST_NAN_DROPPING_DATA_VERSION);

        let snbt = to_gametest_snbt(&schematic);
        assert!(
            snbt.contains("DataVersion: 4671"),
            "the restamp must reach the text"
        );
        let sim = from_snbt(&snbt);
        assert_eq!(
            sim.motion_semantics(),
            mc_tick::MotionSemantics::DropNonFinite,
            "4671 and later guard the whole vector on `isFinite`"
        );
        assert_eq!(
            nan_carts(&sim),
            0,
            "the same six carts must come back finite — the door is un-glued, correctly"
        );
    }

    /// Load the record 3x3 door sample and wire it under `settle`.
    ///
    /// This goes through the product path — world zip, schematic, gametest
    /// SNBT, `wire_simulation` — because the question is about that path and a
    /// test that reached past it would answer a different one.
    fn wire_record_door(settle: super::ffi::TickSettleMode) -> mc_tick::Simulation {
        let schematic = record_door_schematic();
        let snbt = to_gametest_snbt(&schematic);
        let structure = mc_tick::Structure::parse(&snbt).expect("the sample parses");
        super::wire_simulation(
            &structure,
            mc_tick::Pos::new(0, 0, 0),
            settle,
            &[],
            schematic.metadata.source_data_version,
        )
        .expect("the engine must accept the record door")
    }

    /// A build cut out of a running world must load into that world's state.
    ///
    /// `--in-world` capture of this same save records **zero** block changes:
    /// the door is at rest in the game, so it must be at rest in us. The mode
    /// that means "the build *is* the world" is `InWorld`, and this pins that
    /// it genuinely places nothing and settles nothing.
    ///
    /// The `Quiet` half is the negative control, and it is not decoration —
    /// it is the entire reason this test is trustworthy. `Quiet` runs
    /// [`Simulation::place_on_place`], which blanks the region to air and
    /// re-writes every block one at a time, handing each landing block's
    /// already-placed neighbours a shape update. Every observer in the build
    /// therefore watches its facing neighbour *appear*, and pulses. That is
    /// correct for a paste and catastrophic for a load, and it is what made
    /// this door look like it actuated itself: the diagnostic was asking for
    /// `Quiet`. If the two modes ever stop differing here, one of them has
    /// silently become the other and this assertion says so.
    #[test]
    fn the_record_door_is_at_rest_under_in_world_and_disturbed_under_quiet() {
        let mut at_rest = wire_record_door(super::ffi::TickSettleMode::InWorld);
        at_rest.run(200);
        assert_eq!(
            at_rest.recorded().len(),
            0,
            "nobody touched this door: vanilla changes no block ticking the same save in \
             place, so neither may we. First few: {:?}",
            at_rest.recorded().iter().take(4).collect::<Vec<_>>()
        );
        assert!(
            at_rest.is_quiescent(),
            "a build at rest has nothing pending"
        );

        // The control. Placement *should* perturb this build, and if it does
        // not then the assertion above is passing for the wrong reason.
        let mut placed = wire_record_door(super::ffi::TickSettleMode::Quiet);
        placed.run(200);
        assert!(
            !placed.recorded().is_empty(),
            "placing this build must disturb it — an observer whose neighbour just \
             appeared pulses. If this is empty, `InWorld` proves nothing."
        );
    }

    /// The record door's two blazes are **passengers**, and they are now here.
    ///
    /// The save holds 22 top-level entities; vanilla's own capture of it
    /// (`tools/gametest/captures/door55_in_world.entities.log`) counts 24,
    /// because two of the four plain minecarts carry a `minecraft:blaze` in
    /// their `Passengers` list. This asserts both halves — that the top level is
    /// still 22, so the 24 cannot be a miscount of it, and that the two extra
    /// bodies are blazes seated at exactly the y the save records.
    ///
    /// `2.2500` and `2.1875` are lifted from the file, not from the seat
    /// constant: each is its vehicle's y plus 0.1875, which is what
    /// `blaze_ride.entities.log` measures a blaze's seat on a minecart to be,
    /// and getting them from the door as well makes this two independent
    /// measurements agreeing rather than one restated.
    #[test]
    fn the_record_doors_two_blazes_are_seated_passengers() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/samples/55_3x3.zip");
        let bytes = std::fs::read(&path).expect("the record-door sample must be present");
        let schematic = crate::formats::world::from_world_zip(&bytes).expect("the sample loads");
        let snbt = to_gametest_snbt(&schematic);
        let structure = mc_tick::Structure::parse(&snbt).expect("the sample parses");
        assert_eq!(
            structure.entities.len(),
            22,
            "the control: the *top level* is 22, so a 24 below cannot be a recount \
             of it — the two extra bodies have to come from somewhere else"
        );

        let sim = wire_record_door(super::ffi::TickSettleMode::InWorld);
        assert_eq!(
            sim.entity_bodies().len(),
            24,
            "22 top-level entities plus two riders is what vanilla counts in this world"
        );

        let riders = sim.riders();
        assert_eq!(
            riders.len(),
            2,
            "two blazes ride two of the four plain carts"
        );
        let mut seats: Vec<f64> = riders
            .iter()
            .map(|(_, kind, pos)| {
                assert_eq!(kind, "minecraft:blaze");
                pos[1]
            })
            .collect();
        seats.sort_by(f64::total_cmp);
        assert_eq!(seats, vec![2.1875, 2.25], "the exact y the save records");

        // And each sits 0.1875 above the cart it is on, horizontally identical.
        for (_, _, pos) in &riders {
            let vehicle = sim
                .minecarts()
                .iter()
                .find(|c| (c.pos[1] + 0.1875 - pos[1]).abs() < 1.0e-12)
                .expect("every rider has a vehicle 0.1875 below it");
            assert_eq!([vehicle.pos[0], vehicle.pos[2]], [pos[0], pos[2]]);
        }
    }

    /// Wire a one-rail structure carrying `entities`, as the app would.
    fn wire_with_entities(entities: &str) -> Result<mc_tick::Simulation, String> {
        let snbt = format!(
            "{{DataVersion: 4903, size: [1, 1, 1], \
              palette: [{{Name: \"minecraft:rail\"}}], \
              blocks: [{{pos: [0, 0, 0], state: 0}}], entities: [{entities}]}}"
        );
        let structure = mc_tick::Structure::parse(&snbt)
            .unwrap_or_else(|e| panic!("the parser must accept this: {e}\n{snbt}"));
        super::wire_simulation(
            &structure,
            mc_tick::Pos::new(0, 0, 0),
            super::ffi::TickSettleMode::InWorld,
            &[],
            None,
        )
    }

    /// The refusal moved from the parser to the simulator, and still names the
    /// type — but it now fires on *capability*, not on the type existing.
    ///
    /// Furnace carts, fireballs and villagers all load today, as the mass and
    /// hitboxes the record doors use them for. What is refused is any of them
    /// that would need behaviour nobody has implemented: a cart with fuel to
    /// drive itself, a fireball with velocity to fly, a villager with velocity
    /// to walk. That distinction is the whole gate — running one of those as a
    /// stationary box is a confident wrong answer, and the wrongness is
    /// invisible unless it is refused here.
    #[test]
    fn entities_needing_unimplemented_behaviour_are_refused_by_name() {
        for (entity, expected) in [
            (
                r#"{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:furnace_minecart", Fuel: 3600}}"#,
                "minecraft:furnace_minecart",
            ),
            (
                r#"{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:furnace_minecart", PushX: 1.0d}}"#,
                "minecraft:furnace_minecart",
            ),
            (
                r#"{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:dragon_fireball", Motion: [0.5d, 0.0d, 0.0d]}}"#,
                "minecraft:dragon_fireball",
            ),
            (
                r#"{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:small_fireball", Motion: [0.0d, -0.1d, 0.0d]}}"#,
                "minecraft:small_fireball",
            ),
            (
                r#"{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:villager", Motion: [0.0d, 0.0d, 0.2d]}}"#,
                "minecraft:villager",
            ),
        ] {
            let error = wire_with_entities(entity)
                .err()
                .unwrap_or_else(|| panic!("{expected} needs behaviour that does not exist"));
            assert!(
                error.contains(expected),
                "refusal does not name the type: {error}"
            );
        }
    }

    /// The scaffolding the record doors actually contain does load.
    ///
    /// An unfuelled furnace cart, a frozen fireball of each size and a
    /// motionless villager: mass and hitboxes, which is all those builds ask
    /// of them. Without this, a gate that refused every one of these types
    /// would pass the test above and look correct.
    #[test]
    fn frozen_scaffolding_entities_load_as_hitboxes() {
        let sim = wire_with_entities(
            r#"{pos: [0.5d, 0.0625d, 0.5d], nbt: {id: "minecraft:furnace_minecart", Fuel: 0}},
               {pos: [1.5d, 1.0d, 0.5d], nbt: {id: "minecraft:dragon_fireball"}},
               {pos: [2.5d, 1.0d, 0.5d], nbt: {id: "minecraft:small_fireball"}},
               {pos: [3.5d, 1.0d, 0.5d], nbt: {id: "minecraft:villager"}}"#,
        )
        .expect("the record doors' scaffolding is exactly what this supports");
        assert_eq!(sim.minecarts().len(), 1, "a furnace cart is a cart");
        let frozen: Vec<&str> = sim
            .entity_bodies()
            .iter()
            .filter(|b| !b.is_minecart)
            .map(|b| b.kind.as_str())
            .collect();
        assert_eq!(
            frozen,
            [
                "minecraft:dragon_fireball",
                "minecraft:small_fireball",
                "minecraft:villager"
            ],
            "each keeps its own identity, because each has its own hitbox"
        );
    }

    /// Negative control: the gate refuses the unmodelled, not everything.
    ///
    /// Without this, a gate that rejected every build in existence would pass
    /// the test above and look correct.
    #[test]
    fn entities_that_do_have_behaviour_still_load() {
        let sim = wire_with_entities(
            r#"{pos: [0.5d, 0.0625d, 0.5d], nbt: {id: "minecraft:minecart", Motion: [0.0d, 0.0d, 0.0d]}},
               {pos: [0.5d, 1.0d, 0.5d], nbt: {id: "minecraft:item", Item: {id: "minecraft:redstone", count: 1b}}}"#,
        )
        .expect("a plain cart and an item are both simulated today");
        assert_eq!(sim.minecarts().len(), 1, "the cart should be live");
        assert_eq!(sim.item_entities().len(), 1, "the item should be live");
    }

    /// A type the reader cannot even represent is still refused by name.
    ///
    /// This is the *other* refusal — not "no behaviour yet" but "no idea what
    /// this is". A creeper has no `SpawnedEntity` variant, so it cannot be
    /// carried at all, and saying so beats inventing a shape for it.
    #[test]
    fn an_unrepresentable_entity_is_refused_and_named() {
        use crate::entity::Entity;

        let mut schem = UniversalSchematic::new("creeper".into());
        schem.set_block(0, 0, 0, &BlockState::new("minecraft:rail"));
        assert!(schem.add_entity(Entity::new("minecraft:creeper".into(), (0.5, 0.0, 0.5))));

        let snbt = to_gametest_snbt(&schem);
        let err = mc_tick::Structure::parse(&snbt).expect_err("should refuse the creeper");
        assert!(
            matches!(
                &err,
                mc_tick::structure::StructureError::UnsupportedEntity { entity_type, .. }
                    if entity_type == "minecraft:creeper"
            ),
            "wrong error: {err}"
        );
        // And the sentence the app shows blames the build, not the converter.
        let detail = super::structure_parse_detail(&err, true);
        assert!(detail.contains("minecraft:creeper"), "{detail}");
        assert!(!detail.contains("engine fault"), "{detail}");
    }
}
