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
        let state = sim.registry().descriptor(update.state).unwrap_or("minecraft:air");
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
        Some(phase) => {
            mc_tick::PHASE_ORDER.iter().position(|p| *p == phase).map_or(0, |i| i + 1)
        }
    }
}

/// A direction's index into [`mc_tick::ALL_DIRS`].
fn dir_code(dir: mc_tick::Dir) -> usize {
    mc_tick::ALL_DIRS.iter().position(|d| *d == dir).unwrap_or(0)
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
        let _ = write!(pos, "{sep}{},{},{}", update.pos.x, update.pos.y, update.pos.z);
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
            table.push(sim.registry().descriptor(update.state).unwrap_or("minecraft:air"));
            table.len() - 1
        });
        let _ = write!(states_arr, "{sep}{index}");
        n += 1;
    }

    let mut json = String::new();
    let _ = write!(json, "{{\"tick\":{tick},\"n\":{n},\"pos\":[{pos}],\"kind\":[{kinds}],");
    let _ = write!(json, "\"phase\":[{phases_arr}],\"from\":[{froms}],\"state\":[{states_arr}],");
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
    (n, if n == 0 { f64::NAN } else { sum / f64::from(n) }, min, max)
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
        size: (bx + travel, by + 2, bz + 2),
        palette: palette.to_vec(),
        blocks,
        inventories: Vec::new(),
        comparator_outputs: Vec::new(),
        block_entities: Vec::new(),
        entities: Vec::new(),
        item_entities: Vec::new(),
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
    )?;
    fly_on(&mut sim, kick, eval_ticks, seed, must_move_by_tick, need_period, early_exit)
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
        if early_exit
            && t == EARLY_TICK
            && sim.is_quiescent()
            && (com - start_com).abs() < 0.25
        {
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
            super::clear_last_error();
            let structure = mc_tick::Structure::parse(snbt).map_err(|e| {
                super::set_last_error(format!("structure SNBT did not parse: {e:?}"));
                NucleationError::Parse
            })?;
            super::check_volume(structure.size).map_err(|e| {
                super::set_last_error(e);
                NucleationError::InvalidArgument
            })?;
            let extras: Vec<&str> =
                extra.split(';').map(str::trim).filter(|s| !s.is_empty()).collect();
            let sim = super::wire_simulation(
                &structure,
                mc_tick::Pos::new(origin_x, origin_y, origin_z),
                settle,
                &extras,
            )
            .map_err(|e| {
                super::set_last_error(e);
                NucleationError::Simulation
            })?;
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
                // The schematic loaded; it is our own rendering of it that the
                // engine rejected. Saying "Parse" here blames the user's file
                // for our bug, so this reports as an engine failure.
                super::set_last_error(format!(
                    "converted structure did not parse: {e:?} — this is an engine fault, \
                     not a problem with the uploaded file"
                ));
                NucleationError::Simulation
            })?;
            let extras: Vec<&str> =
                extra.split(';').map(str::trim).filter(|s| !s.is_empty()).collect();
            let sim = super::wire_simulation(
                &structure,
                mc_tick::Pos::new(origin_x, origin_y, origin_z),
                settle,
                &extras,
            )
            .map_err(|e| {
                super::set_last_error(e);
                NucleationError::Simulation
            })?;
            Ok(Box::new(TickSimulation { sim, checkpoints: Vec::new() }))
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
            )
            .map_err(|_| NucleationError::Simulation)?;
            Ok(Box::new(TickSimulation { sim, checkpoints: Vec::new() }))
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
            if volume == 0
                || cells.len() % volume != 0
                || kicks.len() != (cells.len() / volume) * 3
            {
                return Err(NucleationError::InvalidArgument);
            }
            let n_genomes = cells.len() / volume;
            // Wire ONE empty-corridor sim (registry, behaviours, physics
            // tables — the expensive part), checkpoint it pristine, and per
            // genome restore + place. Construction cost is paid once per
            // batch instead of once per machine.
            let empty = vec![air_index; volume];
            let empty_structure = super::structure_from_blocks(
                bx, by, bz, travel, x_off, &pal, &empty, air_index,
            )
            .map_err(|_| NucleationError::InvalidArgument)?;
            let mut sim = super::wire_simulation(
                &empty_structure,
                mc_tick::Pos::new(0, 0, 0),
                TickSettleMode::Quiet,
                &extras,
            )
            .map_err(|_| NucleationError::Simulation)?;
            let pristine = sim.checkpoint();
            let mut json = String::from("[");
            for g in 0..n_genomes {
                let slice = &cells[g * volume..(g + 1) * volume];
                let structure = super::structure_from_blocks(
                    bx, by, bz, travel, x_off, &pal, slice, air_index,
                )
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

        /// Start (or stop) recording every delivered redstone update.
        ///
        /// Off by default and much larger than the block-change log — a door's
        /// cycle runs several updates per change — so a propagation view asks
        /// for it explicitly and pages with
        /// [`TickSimulation::updates_json_between`].
        pub fn record_updates(&mut self, on: bool) {
            self.sim.record_updates(on);
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
        pub fn updates_json_between(
            &self,
            from_tick: u32,
            to_tick: u32,
            out: &mut DiplomatWrite,
        ) {
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
        pub fn updates_heat_json(
            &self,
            from_tick: u32,
            to_tick: u32,
            out: &mut DiplomatWrite,
        ) {
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
