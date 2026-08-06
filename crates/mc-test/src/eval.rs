//! The evaluator: wire a structure into a simulation, run a [`Case`], report.
//!
//! Failure reports are multi-line human text, one line per missed check — a
//! scenario that misses in four places should say so once rather than four
//! runs later.

use std::collections::BTreeMap;

use mc_tick::{Pos, Simulation, Structure};

use crate::spec::{state_matches, Case, Check, Expect, SettleMode, Snapshot, MARGIN};

fn snapshot(sim: &Simulation) -> Snapshot {
    sim.world()
        .iter_non_air()
        .map(|(pos, id)| {
            let descriptor = sim
                .registry()
                .descriptor(id)
                .unwrap_or("<unknown>")
                .to_string();
            (pos, descriptor)
        })
        .collect()
}

fn in_region(pos: Pos, region: Option<[[i32; 3]; 2]>) -> bool {
    match region {
        None => true,
        Some([min, max]) => {
            pos.x >= min[0]
                && pos.x <= max[0]
                && pos.y >= min[1]
                && pos.y <= max[1]
                && pos.z >= min[2]
                && pos.z <= max[2]
        }
    }
}

fn restrict(snap: &Snapshot, region: Option<[[i32; 3]; 2]>) -> Snapshot {
    snap.iter()
        .filter(|(pos, _)| in_region(**pos, region))
        .map(|(p, d)| (*p, d.clone()))
        .collect()
}

/// Failure diffs print at most this many lines; raise with MC_TICK_DIFF_LIMIT
/// when authoring a case and you want the whole picture.
fn diff_limit() -> usize {
    std::env::var("MC_TICK_DIFF_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
}

fn diff(expected: &Snapshot, actual: &Snapshot) -> Vec<String> {
    let mut lines = Vec::new();
    for (pos, want) in expected {
        match actual.get(pos) {
            Some(got) if got == want => {}
            Some(got) => lines.push(format!("  {pos:?}: expected {want}, got {got}")),
            None => lines.push(format!("  {pos:?}: expected {want}, got air")),
        }
    }
    for (pos, got) in actual {
        if !expected.contains_key(pos) {
            lines.push(format!("  {pos:?}: expected air, got {got}"));
        }
    }
    lines
}

/// `"x,y,z"` → a position.
fn parse_pos(key: &str) -> Result<Pos, String> {
    let coords: Vec<i32> = key
        .split(',')
        .map(|c| c.trim().parse())
        .collect::<Result<_, _>>()
        .map_err(|e| format!("bad position {key:?}: {e}"))?;
    if coords.len() != 3 {
        return Err(format!("position {key:?} is not x,y,z"));
    }
    Ok(Pos::new(coords[0], coords[1], coords[2]))
}

/// `count` / `at_least` / `at_most` against one measured number.
fn bounds_of(
    count: Option<usize>,
    at_least: Option<usize>,
    at_most: Option<usize>,
    what: &str,
    got: usize,
) -> Option<String> {
    if let Some(want) = count {
        if got != want {
            return Some(format!("expected {what} to be {want}, got {got}"));
        }
    }
    if let Some(least) = at_least {
        if got < least {
            return Some(format!("expected {what} to be at least {least}, got {got}"));
        }
    }
    if let Some(most) = at_most {
        if got > most {
            return Some(format!("expected {what} to be at most {most}, got {got}"));
        }
    }
    if count.is_none() && at_least.is_none() && at_most.is_none() {
        return Some(format!(
            "{what} needs \"count\", \"at_least\" or \"at_most\""
        ));
    }
    None
}

/// [`bounds_of`], spelled with a [`Check`]'s fields.
fn check_bounds(check: &Check, what: &str, got: usize) -> Option<String> {
    bounds_of(check.count, check.at_least, check.at_most, what, got)
}

/// Is there a non-air block at `pos`? Read straight from the world rather than
/// from a snapshot, because `peak-fill` asks this every tick.
fn cell_filled(sim: &Simulation, pos: Pos) -> bool {
    let id = sim.world().get(pos);
    sim.registry()
        .descriptor(id)
        .is_some_and(|d| d != "minecraft:air")
}

/// The lowest y any entity occupies — box minimum for bodies, position for items.
fn min_entity_y(sim: &Simulation) -> Option<f64> {
    let bodies = sim.entity_bodies().iter().map(|b| b.min[1]);
    let items = sim
        .item_entities()
        .iter()
        .filter(|e| !e.removed)
        .map(|e| e.pos[1]);
    bodies.chain(items).fold(None, |acc: Option<f64>, y| {
        Some(match acc {
            None => y,
            Some(low) if y < low => y,
            Some(low) => low,
        })
    })
}

/// The full vanilla wiring recipe, mirrored from mc-tick's `conformance.rs`.
///
/// `motion_data_version` is the data version of the *save the entities came
/// from*, and it decides whether a NaN velocity survives being loaded. The
/// record 3x3 door is glued together by nan carts; read at the wrong version
/// they load as ordinary carts and the door quietly un-glues. `None` leaves the
/// engine's default.
pub fn build_sim(
    structure: &Structure,
    hash_origin: Pos,
    settle: SettleMode,
    extra_states: &[String],
    extra_inert: &[String],
    motion_data_version: Option<i32>,
    label: &str,
) -> Simulation {
    try_build_sim(
        structure,
        hash_origin,
        settle,
        extra_states,
        extra_inert,
        motion_data_version,
        label,
    )
    .unwrap_or_else(|report| panic!("{report}"))
}

/// [`build_sim`], reporting an unsimulable structure instead of panicking.
///
/// `Err` is the unknown-block report — the caller can read the block names out
/// of it (a porting tool probing what a foreign structure needs asserted
/// inert) where `build_sim`'s panic is the right behaviour for a test carrier.
pub fn try_build_sim(
    structure: &Structure,
    hash_origin: Pos,
    settle: SettleMode,
    extra_states: &[String],
    extra_inert: &[String],
    motion_data_version: Option<i32>,
    label: &str,
) -> Result<Simulation, String> {
    let mut sim = Simulation::new(structure.bounds(MARGIN));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    if let Some(version) = motion_data_version {
        sim.set_motion_semantics(mc_tick::MotionSemantics::for_data_version(version));
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
            .unwrap_or_else(|| panic!("{label}: inventory at {pos:?} with no block"));
        let name = structure.palette[entry]
            .split('[')
            .next()
            .unwrap_or_default()
            .to_string();
        let slots = mc_tick::vanilla::container_slots(&name)
            .unwrap_or_else(|| panic!("{label}: {name} has an inventory but no slot count"));
        sim.set_inventory(
            *pos,
            mc_tick::Inventory {
                slots,
                stacks: stacks.clone(),
                blocked_slots: structure.blocked_slots_at(*pos),
            },
        );
    }
    // Action states (a placed redstone block) must exist before behaviours
    // and power rules bind — both key on the StateIds interned at
    // registration, so a state interned mid-run is inert.
    for descriptor in extra_states {
        sim.registry_mut()
            .intern(descriptor)
            .unwrap_or_else(|e| panic!("{label}: interning {descriptor}: {e:?}"));
    }
    // A dispenser can *place* a block it holds as an item — a shulker box, or a
    // bucket's contents. Those block states are not in the structure's palette,
    // and behaviours bind only to interned states, so intern them up front.
    for (_, stacks) in &structure.inventories {
        for stack in stacks {
            for descriptor in mc_tick::vanilla::dispensable_states(&stack.id) {
                let _ = sim.registry_mut().intern(&descriptor);
            }
        }
    }
    // Command blocks: parse the supported subset, intern each program's
    // target state before behaviours bind, and hand the sim its programs.
    // Unsupported commands (summon, data, queries) get no program: the block
    // powers on and runs nothing, like an unparseable command in game.
    for (pos, text) in &structure.commands {
        let Some(parsed) = mc_tick::vanilla::parse_command(text) else {
            continue;
        };
        let program =
            match parsed {
                mc_tick::vanilla::ParsedCommand::SetBlock { offset, state } => {
                    let state = sim.registry_mut().intern(&state).unwrap_or_else(|e| {
                        panic!("{label}: interning command target {state}: {e:?}")
                    });
                    mc_tick::behaviour::CommandProgram::SetBlock { offset, state }
                }
                mc_tick::vanilla::ParsedCommand::Fill { a, b, state } => {
                    let state = sim.registry_mut().intern(&state).unwrap_or_else(|e| {
                        panic!("{label}: interning command target {state}: {e:?}")
                    });
                    mc_tick::behaviour::CommandProgram::Fill { a, b, state }
                }
                mc_tick::vanilla::ParsedCommand::Summon { kind, offset, fuse } => {
                    // Leaked once per build, like the retype's item id.
                    mc_tick::behaviour::CommandProgram::Summon {
                        kind: Box::leak(kind.into_boxed_str()),
                        offset,
                        fuse,
                    }
                }
                mc_tick::vanilla::ParsedCommand::RetypeNearestItem { radius, item } => {
                    // Leaked once per build: programs are build-time config and
                    // the id must live as long as the (Copy) program does.
                    mc_tick::behaviour::CommandProgram::RetypeNearestItem {
                        radius,
                        item: Box::leak(item.into_boxed_str()),
                    }
                }
            };
        sim.set_command(*pos, program);
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, hash_origin);
        *sim.behaviours_mut() = table;
    }
    // The case's own inert assertions, after vanilla registration and before
    // the unknown sweep: every matching interned state (any property set)
    // becomes explicitly inert, on the author's authority.
    if !extra_inert.is_empty() {
        let ids: Vec<mc_tick::StateId> = (0..sim.registry().len() as u16)
            .map(mc_tick::StateId)
            .filter(|id| {
                sim.registry().descriptor(*id).is_some_and(|d| {
                    let name = d.split('[').next().unwrap_or(d);
                    extra_inert.iter().any(|n| n == name)
                })
            })
            .collect();
        for id in ids {
            sim.behaviours_mut()
                .register(id, Box::new(mc_tick::Inert::new("case-inert")));
        }
    }
    if let Some(report) = sim.unknown_report() {
        return Err(format!(
            "{label}: every block must have behaviour, or this runs a partially-simulated \
             world — {report}"
        ));
    }
    {
        let (physics, fluids, lava, rails) = mc_tick::vanilla::environment_tables(sim.registry());
        let (solidity, frictions, heights, webs) = physics;
        sim.set_physics_tables(solidity, frictions, heights, webs);
        let (water_kinds, bubble_kinds) = fluids;
        sim.set_fluid_tables(water_kinds, bubble_kinds);
        sim.set_lava_table(lava);
        let (rail_kinds, rail_conductors) = rails;
        sim.set_rail_tables(rail_kinds, rail_conductors);
    }
    for spawned in &structure.entities {
        match spawned {
            mc_tick::structure::SpawnedEntity::Item(item) => {
                sim.spawn_item(item.item.clone(), item.pos, item.motion, item.pickup_delay);
            }
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                let vehicle = sim.spawn_authored_minecart(cart, None);
                for rider in &cart.passengers {
                    sim.spawn_authored_rider(vehicle, rider)
                        .unwrap_or_else(|e| panic!("{label}: {e}"));
                }
            }
            // A case that authors one of these would otherwise run with the
            // entity missing and quietly "pass". Loud beats silent in a test
            // harness, so a spawn that refuses still panics rather than
            // dropping the entity.
            mc_tick::structure::SpawnedEntity::FurnaceMinecart(cart) => {
                sim.spawn_authored_furnace_minecart(cart, None)
                    .unwrap_or_else(|e| panic!("{label}: {e}"));
            }
            mc_tick::structure::SpawnedEntity::Body(body) => {
                let vehicle = sim
                    .spawn_authored_body(body)
                    .unwrap_or_else(|e| panic!("{label}: {e}"));
                for rider in &body.passengers {
                    sim.spawn_authored_rider(vehicle, rider)
                        .unwrap_or_else(|e| panic!("{label}: {e}"));
                }
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
    if settle != SettleMode::InWorld {
        sim.place_on_place(&order);
    }
    if settle == SettleMode::Placement {
        sim.settle_with_order(&order);
    }
    sim.record();
    Ok(sim)
}

/// How a run reports, beyond pass/fail.
pub struct RunOptions {
    /// On failure, how many ticks of recorded block changes to include before
    /// the first failing tick. Diagnostics only — never part of pass/fail.
    pub trace_window: u64,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self { trace_window: 2 }
    }
}

/// One case's verdict, with the numbers a grid wants beside it.
pub struct CaseResult {
    /// `case.name`.
    pub name: String,
    /// The horizon the case ran to (its highest action or check tick).
    pub ticks: u64,
    /// Wall-clock for build + run.
    pub wall: std::time::Duration,
    /// `Err` is the multi-line human report, one line per missed check.
    pub outcome: Result<(), String>,
}

/// Wire `structure`, run `case`, and report the verdict with timing.
pub fn run_with(
    structure: &Structure,
    case: &Case,
    motion_data_version: Option<i32>,
    options: &RunOptions,
) -> CaseResult {
    let start = std::time::Instant::now();
    let (ticks, outcome) = run_inner(structure, case, motion_data_version, options);
    CaseResult {
        name: case.name.clone(),
        ticks,
        wall: start.elapsed(),
        outcome,
    }
}

/// Wire `structure`, run `case`, and report every check that failed.
///
/// `Err` is a multi-line human report, not a single sentence: a scenario that
/// misses in four places should say so once rather than four runs later.
pub fn run(
    structure: &Structure,
    case: &Case,
    motion_data_version: Option<i32>,
) -> Result<(), String> {
    run_with(structure, case, motion_data_version, &RunOptions::default()).outcome
}

fn run_inner(
    structure: &Structure,
    case: &Case,
    motion_data_version: Option<i32>,
    options: &RunOptions,
) -> (u64, Result<(), String>) {
    let label = &case.name;
    let hash_origin = Pos::new(case.origin[0], case.origin[1], case.origin[2]);
    let action_states: Vec<String> = case
        .actions
        .iter()
        .filter_map(|a| a.state.clone())
        .collect();
    let mut sim = build_sim(
        structure,
        hash_origin,
        case.settle,
        &action_states,
        &case.inert,
        motion_data_version,
        label,
    );
    if let Some(seed) = case.seed {
        sim.set_rng_seed(seed);
    }
    sim.set_random_ticks(case.random_ticks);
    // Setup ticks: placement transients play out off the record, exactly as
    // gametest `setup_ticks` do. Recorded ticks keep counting from the sim's
    // own clock, so everything below reads through `tick_base`.
    let tick_base = case.setup;
    if case.setup > 0 {
        for _ in 0..case.setup {
            sim.step();
        }
        sim.record();
        // Re-arm accept test blocks that latched during setup: placement
        // transients (an observer's placement pulse crossing the accept
        // wiring) can fire them before the test begins, and vanilla only
        // counts an accept once the test is running. The silent world write
        // keeps the reset out of the recorded log and out of the update
        // graph both.
        let rearm: Vec<(Pos, mc_tick::StateId)> = sim
            .world()
            .iter_non_air()
            .filter_map(|(pos, id)| {
                let descriptor = sim.registry().descriptor(id)?;
                if !descriptor.starts_with("minecraft:test_block")
                    || !descriptor.contains("fired=true")
                {
                    return None;
                }
                let unfired = descriptor.replace("fired=true", "fired=false");
                sim.registry().get(&unfired).map(|state| (pos, state))
            })
            .collect();
        if !rearm.is_empty() {
            let (_, world) = sim.registry_and_world_mut();
            for (pos, state) in rearm {
                world.set(pos, state);
            }
        }
    }

    let initial = snapshot(&sim);
    let mut at_tick: BTreeMap<u64, Snapshot> = BTreeMap::new();
    let horizon = case
        .actions
        .iter()
        .map(|a| a.tick)
        .chain(case.checks.iter().map(|c| c.tick))
        .max()
        .unwrap_or(0);

    // The rest may bail early (a malformed check, an impossible action); the
    // closure keeps `?` working while the horizon still reaches the caller.
    let outcome = (|| -> Result<(), String> {
        // `peak-fill` watches its cells on every tick, so its cell lists are parsed
        // once up front — a typo in one should fail before the run, not after it.
        let mut watched: Vec<(usize, Vec<Pos>, usize)> = Vec::new();
        for (index, check) in case.checks.iter().enumerate() {
            if check.expect != Expect::PeakFill {
                continue;
            }
            let cells = check
                .cells
                .as_ref()
                .ok_or_else(|| format!("{label}: \"peak-fill\" needs a \"cells\" list"))?;
            let parsed: Vec<Pos> = cells
                .iter()
                .map(|key| parse_pos(key).map_err(|e| format!("{label}: {e}")))
                .collect::<Result<_, _>>()?;
            watched.push((index, parsed, 0));
        }

        let mut failures: Vec<String> = Vec::new();
        // For the diagnostic dump: the first tick that pushed a failure.
        let mut first_failing_tick: Option<u64> = None;
        for tick in 0..=horizon {
            let failures_before = failures.len();
            for (_, cells, peak) in watched.iter_mut() {
                let filled = cells.iter().filter(|pos| cell_filled(&sim, **pos)).count();
                *peak = (*peak).max(filled);
            }
            // Checks first: a check at T sees the world after exactly T ticks,
            // before any action scheduled at T fires.
            for (index, check) in case
                .checks
                .iter()
                .enumerate()
                .filter(|(_, c)| c.tick == tick)
            {
                let now = snapshot(&sim);
                at_tick.entry(tick).or_insert_with(|| now.clone());
                let (want, got) = match check.expect {
                    Expect::Initial => (
                        restrict(&initial, check.region),
                        restrict(&now, check.region),
                    ),
                    Expect::Changed => {
                        if restrict(&initial, check.region) == restrict(&now, check.region) {
                            failures.push(format!(
                            "{label}: tick {tick}: expected the world to have changed from initial, but it is identical"
                        ));
                        }
                        continue;
                    }
                    Expect::SameAs => {
                        let reference_tick = check.as_tick.ok_or_else(|| {
                            format!("{label}: tick {tick}: \"same-as\" needs \"as_tick\"")
                        })?;
                        let reference = at_tick.get(&reference_tick).ok_or_else(|| {
                            format!(
                                "{label}: tick {tick}: as_tick {reference_tick} has no snapshot — \
                             it must be an earlier check's tick"
                            )
                        })?;
                        (
                            restrict(reference, check.region),
                            restrict(&now, check.region),
                        )
                    }
                    Expect::Blocks => {
                        let blocks = check.blocks.as_ref().ok_or_else(|| {
                            format!("{label}: tick {tick}: \"blocks\" needs a \"blocks\" map")
                        })?;
                        for (key, want) in blocks {
                            let pos =
                                parse_pos(key).map_err(|e| format!("{label}: tick {tick}: {e}"))?;
                            let got = now.get(&pos).map(String::as_str).unwrap_or("minecraft:air");
                            if !state_matches(want, got) {
                                failures.push(format!(
                                    "{label}: tick {tick}: at {key}: expected {want}, got {got}"
                                ));
                            }
                        }
                        continue;
                    }
                    Expect::Entities => {
                        let expects = check.entities.as_ref().ok_or_else(|| {
                            format!("{label}: tick {tick}: \"entities\" needs an \"entities\" list")
                        })?;
                        for expect in expects {
                            let inside = |pos: [f64; 3], region: [[i32; 3]; 2]| {
                                (0..3).all(|i| {
                                    pos[i] >= f64::from(region[0][i])
                                        && pos[i] < f64::from(region[1][i] + 1)
                                })
                            };
                            let carries = |entity_id: u32| {
                                let held = sim.item_contents(entity_id).unwrap_or(&[]);
                                let listed = match &expect.with_contents {
                                    None => true,
                                    Some(wanted) => wanted.iter().all(|w| {
                                        held.iter().any(|s| s.id == w.id && s.count == w.count)
                                    }),
                                };
                                let empty = match expect.empty_contents {
                                    Some(true) => held.iter().all(|s| s.count == 0),
                                    _ => true,
                                };
                                listed && empty
                            };
                            let (found, total) = match (&expect.item, &expect.kind) {
                            (Some(item), None) => sim
                                .item_entities()
                                .iter()
                                .filter(|e| {
                                    !e.removed
                                        && e.item.0 == *item
                                        && inside(e.pos, expect.region)
                                        && carries(e.id)
                                })
                                .fold((0usize, 0u32), |(n, t), e| {
                                    (n + 1, t + u32::from(e.item.1))
                                }),
                            (None, Some(kind)) => (
                                sim.minecarts()
                                    .iter()
                                    .filter(|c| {
                                        !c.removed && c.kind == *kind && inside(c.pos, expect.region)
                                    })
                                    .count(),
                                0,
                            ),
                            _ => {
                                return Err(format!(
                                    "{label}: tick {tick}: an entity expectation is either \"item\" or \"kind\""
                                ))
                            }
                        };
                            let ok = match expect.count {
                                Some(count) => found == count,
                                None => found > 0 || expect.items_total == Some(0),
                            };
                            if !ok {
                                let what = expect
                                    .item
                                    .as_deref()
                                    .or(expect.kind.as_deref())
                                    .unwrap_or("?");
                                failures.push(format!(
                                "{label}: tick {tick}: expected {} {what} in {:?}, found {found}",
                                expect.count.map_or("at least 1".to_string(), |c| c.to_string()),
                                expect.region,
                            ));
                            }
                            if let Some(want_total) = expect.items_total {
                                if total != want_total {
                                    let what = expect.item.as_deref().unwrap_or("?");
                                    failures.push(format!(
                                    "{label}: tick {tick}: expected {want_total} {what} in total in {:?}, found {total} across {found} entities",
                                    expect.region,
                                ));
                                }
                            }
                        }
                        continue;
                    }
                    Expect::Air => {
                        let region = check.region.ok_or_else(|| {
                            format!("{label}: tick {tick}: \"air\" needs a \"region\"")
                        })?;
                        let blocking = restrict(&now, Some(region));
                        if !blocking.is_empty() {
                            let mut lines: Vec<String> = blocking
                                .iter()
                                .map(|(p, d)| format!("  {p:?}: {d}"))
                                .collect();
                            lines.truncate(diff_limit());
                            failures.push(format!(
                                "{label}: tick {tick}: expected region air, found {} blocks:\n{}",
                                blocking.len(),
                                lines.join("\n")
                            ));
                        }
                        continue;
                    }
                    Expect::Quiescent => {
                        if !sim.is_quiescent() {
                            failures.push(format!(
                                "{label}: tick {tick}: expected the machine to be at rest, but \
                             something is still pending — this run has not finished"
                            ));
                        }
                        continue;
                    }
                    Expect::EntityCount => {
                        let got = sim.entity_bodies().len();
                        if let Some(why) = check_bounds(check, "the entity count", got) {
                            failures.push(format!("{label}: tick {tick}: {why}"));
                        }
                        continue;
                    }
                    Expect::Changes => {
                        let got = sim.recorded().len();
                        if let Some(why) = check_bounds(check, "the block-change count", got) {
                            failures.push(format!("{label}: tick {tick}: {why}"));
                        }
                        continue;
                    }
                    Expect::Fill => {
                        let cells = check.cells.as_ref().ok_or_else(|| {
                            format!("{label}: tick {tick}: \"fill\" needs a \"cells\" list")
                        })?;
                        let mut filled = 0usize;
                        let mut detail: Vec<String> = Vec::new();
                        for key in cells {
                            let pos =
                                parse_pos(key).map_err(|e| format!("{label}: tick {tick}: {e}"))?;
                            match now.get(&pos) {
                                Some(descriptor) => {
                                    filled += 1;
                                    detail.push(format!("  {key}: {descriptor}"));
                                }
                                None => detail.push(format!("  {key}: air")),
                            }
                        }
                        if let Some(why) = check_bounds(
                            check,
                            &format!("the fill of {} cells", cells.len()),
                            filled,
                        ) {
                            detail.truncate(diff_limit());
                            failures.push(format!(
                                "{label}: tick {tick}: {why}\n{}",
                                detail.join("\n")
                            ));
                        }
                        continue;
                    }
                    Expect::PeakFill => {
                        let (_, cells, peak) = watched
                            .iter()
                            .find(|(i, _, _)| *i == index)
                            .expect("every peak-fill check is watched");
                        if let Some(why) = check_bounds(
                            check,
                            &format!("the widest the {} cells ever got", cells.len()),
                            *peak,
                        ) {
                            failures.push(format!("{label}: tick {tick}: {why}"));
                        }
                        continue;
                    }
                    Expect::MinEntityY => {
                        let floor = check.y.ok_or_else(|| {
                            format!("{label}: tick {tick}: \"min-entity-y\" needs a \"y\"")
                        })?;
                        if let Some(low) = min_entity_y(&sim) {
                            if low < floor {
                                failures.push(format!(
                                    "{label}: tick {tick}: an entity sits at y={low}, below the \
                                 y={floor} floor — something left through the bottom"
                                ));
                            }
                        }
                        continue;
                    }
                    Expect::Riders => {
                        let kind = check.kind.as_ref().ok_or_else(|| {
                            format!("{label}: tick {tick}: \"riders\" needs a \"kind\"")
                        })?;
                        let want_seats = check.seats.as_ref().ok_or_else(|| {
                            format!("{label}: tick {tick}: \"riders\" needs a \"seats\" list")
                        })?;
                        let mut seats: Vec<f64> = Vec::new();
                        for (_, got_kind, pos) in sim.riders() {
                            if &got_kind != kind {
                                failures.push(format!(
                                "{label}: tick {tick}: expected only {kind} riders, found {got_kind}"
                            ));
                                continue;
                            }
                            seats.push(pos[1]);
                        }
                        seats.sort_by(f64::total_cmp);
                        if seats.len() != want_seats.len()
                            || seats
                                .iter()
                                .zip(want_seats)
                                .any(|(a, b)| (a - b).abs() > 1.0e-9)
                        {
                            failures.push(format!(
                            "{label}: tick {tick}: expected {kind} seats {want_seats:?}, got {seats:?}"
                        ));
                        }
                        continue;
                    }
                };
                let mut lines = diff(&want, &got);
                if !lines.is_empty() {
                    let total = lines.len();
                    lines.truncate(diff_limit());
                    failures.push(format!(
                        "{label}: tick {tick}: {total} blocks differ:\n{}",
                        lines.join("\n")
                    ));
                }
            }
            for action in case.actions.iter().filter(|a| a.tick == tick) {
                match (&action.use_pos, &action.place, &action.state) {
                    (Some(p), None, None) => sim.use_block(Pos::new(p[0], p[1], p[2])),
                    (None, Some(p), Some(descriptor)) => {
                        let state = sim
                            .registry_mut()
                            .intern(descriptor)
                            .map_err(|e| format!("{label}: interning {descriptor}: {e:?}"))?;
                        sim.place_block(Pos::new(p[0], p[1], p[2]), state);
                    }
                    _ => return Err(format!(
                        "{label}: tick {tick}: an action is either \"use\" or \"place\"+\"state\""
                    )),
                }
            }
            if failures.len() > failures_before {
                first_failing_tick.get_or_insert(tick);
            }
            if tick < horizon {
                sim.step();
            }
        }

        // Opt-in event assertions, against the whole recorded change log. They run
        // after the loop because `recorded()` persists everything with its tick.
        for expect in &case.events {
            if expect.kind != "block-changed" {
                failures.push(format!(
                "{label}: event kind {:?} is not one this runner understands (only \"block-changed\")",
                expect.kind
            ));
                continue;
            }
            let descriptor = |id| sim.registry().descriptor(id).unwrap_or("<unknown>");
            let matched = sim
                .recorded()
                .iter()
                .filter(|c| expect.tick.is_none_or(|t| c.tick == t + tick_base))
                .filter(|c| expect.after.is_none_or(|a| c.tick >= a + tick_base))
                .filter(|c| {
                    expect
                        .pos
                        .is_none_or(|p| c.pos == Pos::new(p[0], p[1], p[2]))
                })
                .filter(|c| {
                    expect
                        .from
                        .as_deref()
                        .is_none_or(|w| state_matches(w, descriptor(c.from)))
                })
                .filter(|c| {
                    expect
                        .to
                        .as_deref()
                        .is_none_or(|w| state_matches(w, descriptor(c.to)))
                })
                .count();
            let what = format!(
                "the count of block-changed events{}{}{}{}{}",
                expect
                    .tick
                    .map_or(String::new(), |t| format!(" at tick {t}")),
                expect
                    .after
                    .map_or(String::new(), |a| format!(" from tick {a} on")),
                expect.pos.map_or(String::new(), |p| format!(" at {p:?}")),
                expect
                    .from
                    .as_deref()
                    .map_or(String::new(), |w| format!(" from {w}")),
                expect
                    .to
                    .as_deref()
                    .map_or(String::new(), |w| format!(" to {w}")),
            );
            // All bounds absent means "it happened at least once".
            let at_least = if expect.count.is_none()
                && expect.at_least.is_none()
                && expect.at_most.is_none()
            {
                Some(1)
            } else {
                expect.at_least
            };
            if let Some(why) = bounds_of(expect.count, at_least, expect.at_most, &what, matched) {
                failures.push(format!("{label}: {why}"));
            }
        }

        // Diagnostics for every failure, opted into or not: the recorded block
        // changes around the first failing tick. This is the harness answering
        // "what actually happened" without a rerun.
        if !failures.is_empty() {
            let upto = first_failing_tick.unwrap_or(horizon);
            let from = upto.saturating_sub(options.trace_window);
            let mut log: Vec<String> = sim
                .recorded()
                .iter()
                .filter(|c| c.tick >= from + tick_base && c.tick <= upto + tick_base)
                .map(|c| {
                    format!(
                        "  tick {}: {:?}: {} -> {}",
                        c.tick - tick_base,
                        c.pos,
                        sim.registry().descriptor(c.from).unwrap_or("<unknown>"),
                        sim.registry().descriptor(c.to).unwrap_or("<unknown>"),
                    )
                })
                .collect();
            let total = log.len();
            log.truncate(200);
            failures.push(format!(
                "{label}: event log, ticks {from}..={upto} ({total} block changes):\n{}",
                if log.is_empty() {
                    "  (none recorded)".to_string()
                } else {
                    log.join("\n")
                }
            ));
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("\n"))
        }
    })();
    (horizon, outcome)
}

/// Run every scenario a driver found, printing one line each, and fail once
/// with the names that missed.
///
/// `cases` is `(display name, runnable)`; the driver decides where scenarios
/// come from and how one is loaded, and this decides nothing except that a
/// suite reports all of its failures rather than the first.
pub fn report<F>(what: &str, cases: Vec<(String, F)>)
where
    F: FnOnce() -> Result<(), String>,
{
    assert!(!cases.is_empty(), "no {what} matched");
    let total = cases.len();
    let mut failed = Vec::new();
    for (name, run_it) in cases {
        match run_it() {
            Ok(()) => eprintln!("case {name} ... ok"),
            Err(report) => {
                eprintln!("case {name} ... FAILED\n{report}");
                failed.push(name);
            }
        }
    }
    assert!(
        failed.is_empty(),
        "{} of {total} {what} failed: {}",
        failed.len(),
        failed.join(", ")
    );
}
