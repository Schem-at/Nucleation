//! The scenario descriptor and its evaluator — one vocabulary, two carriers.
//!
//! A scenario is a structure plus a list of player actions plus end-state
//! checks at named ticks. It is deliberately blind to *how* the engine got
//! there (no traces, no event order, no intra-tick expectations): a faster
//! redstone backend that still opens and resets the door passes unchanged.
//!
//! This file is compiled into two test binaries, by `#[path]`:
//!
//! - `crates/mc-tick/tests/cases.rs`, where the carrier is a `*.test.json`
//!   descriptor beside an `.snbt` structure, and
//! - `tests/litematic_cases.rs` in nucleation, where the carrier is a single
//!   `.litematic` with the descriptor stored inside it.
//!
//! It lives under `tests/support/` rather than `tests/` so cargo does not
//! auto-discover it as a test target of its own.
//!
//! See `crates/mc-tick/tests/cases/README.md` for the descriptor format.

#![allow(dead_code)]

use std::collections::BTreeMap;

use mc_tick::{Pos, Simulation, Structure};
use serde::Deserialize;

/// Air margin around the build, matching `conformance.rs`.
pub const MARGIN: i32 = 4;

/// One scenario.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    /// What this proves, in one sentence. Used in every failure message.
    pub name: String,
    /// Structure file, relative to the case file. Defaults to `<stem>.snbt`.
    /// Meaningless — and rejected — for a carrier that *is* the structure.
    #[serde(default)]
    pub structure: Option<String>,
    /// Where the capture's (0,0,0) sat in the game's coordinates — wire update
    /// order hashes absolute positions, so an in-world build needs its origin.
    #[serde(default)]
    pub origin: [i32; 3],
    #[serde(default)]
    pub settle: SettleMode,
    /// Seed for the vanilla random source. Behaviours that jitter (dispense
    /// trajectories, dispenser slot choice, destroy drops) draw from it in a
    /// fixed order, so a seeded case is exactly reproducible. Omitted: the
    /// engine uses each distribution's mean.
    #[serde(default)]
    pub seed: Option<i64>,
    #[serde(default)]
    pub actions: Vec<Action>,
    pub checks: Vec<Check>,
}

/// How the loaded structure is settled before tick 0.
#[derive(Deserialize, PartialEq, Clone, Copy, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SettleMode {
    /// Vanilla placement pass + ordered settle — a build saved at rest.
    #[default]
    Placement,
    /// `onPlace` only, no settle — a knownShape capture.
    Quiet,
    /// Neither — the build was recorded in the world it stood in, mid-state.
    InWorld,
}

/// One thing a player does, during tick `tick`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub tick: u64,
    /// Right-click with an empty hand (a lever, a button, a note block).
    #[serde(rename = "use")]
    pub use_pos: Option<[i32; 3]>,
    /// Write a block state (`"minecraft:air"` breaks a block).
    pub place: Option<[i32; 3]>,
    pub state: Option<String>,
}

/// One end-state assertion, evaluated after exactly `tick` ticks.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub tick: u64,
    pub expect: Expect,
    #[serde(default)]
    pub as_tick: Option<u64>,
    /// Restrict the comparison to an inclusive box; whole world when absent.
    #[serde(default)]
    pub region: Option<[[i32; 3]; 2]>,
    /// With `expect: "blocks"`: `"x,y,z"` → expected state. A descriptor
    /// without properties matches on block name alone; listed properties must
    /// each hold, unlisted ones are free (`redstone_wire[power=15]` matches
    /// any fully-connected dust at power 15).
    #[serde(default)]
    pub blocks: Option<BTreeMap<String, String>>,
    /// With `expect: "entities"`: each entry must be satisfied.
    #[serde(default)]
    pub entities: Option<Vec<EntityExpect>>,
    /// With `expect: "fill"`: the cell set whose non-air members are counted.
    /// A doorway is nine cells; how many of them are filled is the whole
    /// question, and *which* nine is the authored part.
    #[serde(default)]
    pub cells: Option<Vec<String>>,
    /// Exact count, for `entity-count`, `fill` and `changes`.
    #[serde(default)]
    pub count: Option<usize>,
    /// Lower/upper bounds, for the same three. A count is pinned with `count`;
    /// a budget is expressed with these.
    #[serde(default)]
    pub at_least: Option<usize>,
    #[serde(default)]
    pub at_most: Option<usize>,
    /// With `expect: "min-entity-y"`: no entity may sit below this y. The
    /// cheap, backend-agnostic form of "the build did not fall apart" — a cart
    /// that lost its NaN velocity leaves through the floor.
    #[serde(default)]
    pub y: Option<f64>,
    /// With `expect: "riders"`: the passenger kind, and the exact seat heights
    /// the save records, ascending. Entity *seats* are structure, not physics:
    /// a rider 0.1875 above its cart is where the file put it.
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub seats: Option<Vec<f64>>,
}

/// One entity expectation inside an `entities` check.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityExpect {
    /// Item id for item entities (`minecraft:iron_ingot`).
    pub item: Option<String>,
    /// Entity kind for minecarts (`minecraft:minecart`).
    pub kind: Option<String>,
    /// Inclusive block box the entity's position must fall inside.
    pub region: [[i32; 3]; 2],
    /// Exact count; when absent, at least one.
    pub count: Option<usize>,
    /// Container contents the item must carry (a dropped shulker box's
    /// slots): every listed `{id, count}` must appear.
    #[serde(default)]
    pub with_contents: Option<Vec<ContentExpect>>,
    /// The item must carry no container contents at all — a shulker box that
    /// was drained before it dropped.
    #[serde(default)]
    pub empty_contents: Option<bool>,
    /// Total item count summed over matching entities. Two ejected diamonds
    /// may merge into one entity of two, or land as two of one — this asserts
    /// the diamonds, not the entity bookkeeping.
    #[serde(default)]
    pub items_total: Option<u32>,
}

/// One `{id, count}` a container must hold.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentExpect {
    pub id: String,
    pub count: u8,
}

/// What a check asserts.
#[derive(Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum Expect {
    /// Equals the settled pre-action world (a reset check).
    Initial,
    /// Differs from initial (the machine actually moved).
    Changed,
    /// Equals the world at an earlier check's tick (`as_tick`).
    SameAs,
    /// Every block in `region` is air.
    Air,
    /// Exact states at named positions.
    Blocks,
    /// Item entities and minecarts.
    Entities,
    /// Nothing is pending: no scheduled tick, no queued update. The
    /// backend-agnostic spelling of "the run finished", and the one thing that
    /// says a door came to rest rather than still being mid-cycle.
    Quiescent,
    /// How many entities the world holds. A door glued together by nan carts
    /// is a door whose entity count is load-bearing.
    EntityCount,
    /// How many of `cells` are non-air. The doorway metric.
    Fill,
    /// The most of `cells` that were *ever* non-air, up to this tick.
    ///
    /// How far the door got, rather than where it stopped. A door leaf sweeps
    /// through the doorway and settles somewhere; the width of the sweep is the
    /// claim worth pinning, and unlike a reading at one named tick it does not
    /// care which tick the sweep peaked on.
    PeakFill,
    /// How many blocks changed over the run so far.
    Changes,
    /// The lowest y any entity occupies.
    MinEntityY,
    /// The passenger seats, ascending.
    Riders,
}

/// Does `actual` satisfy `expected`? Same block name, and every property the
/// expectation lists holds in the actual state; unlisted properties are free.
pub fn state_matches(expected: &str, actual: &str) -> bool {
    let (want_name, want_props) = match expected.split_once('[') {
        Some((name, props)) => (name, props.trim_end_matches(']')),
        None => (expected, ""),
    };
    let (got_name, got_props) = match actual.split_once('[') {
        Some((name, props)) => (name, props.trim_end_matches(']')),
        None => (actual, ""),
    };
    if want_name != got_name {
        return false;
    }
    want_props
        .split(',')
        .filter(|p| !p.is_empty())
        .all(|want| got_props.split(',').any(|got| got == want))
}

/// A world snapshot: every non-air block.
pub type Snapshot = BTreeMap<Pos, String>;

fn snapshot(sim: &Simulation) -> Snapshot {
    sim.world()
        .iter_non_air()
        .map(|(pos, id)| {
            let descriptor = sim.registry().descriptor(id).unwrap_or("<unknown>").to_string();
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
    std::env::var("MC_TICK_DIFF_LIMIT").ok().and_then(|v| v.parse().ok()).unwrap_or(20)
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
fn check_bounds(check: &Check, what: &str, got: usize) -> Option<String> {
    if let Some(want) = check.count {
        if got != want {
            return Some(format!("expected {what} to be {want}, got {got}"));
        }
    }
    if let Some(least) = check.at_least {
        if got < least {
            return Some(format!("expected {what} to be at least {least}, got {got}"));
        }
    }
    if let Some(most) = check.at_most {
        if got > most {
            return Some(format!("expected {what} to be at most {most}, got {got}"));
        }
    }
    if check.count.is_none() && check.at_least.is_none() && check.at_most.is_none() {
        return Some(format!("{what} needs \"count\", \"at_least\" or \"at_most\""));
    }
    None
}

/// Is there a non-air block at `pos`? Read straight from the world rather than
/// from a snapshot, because `peak-fill` asks this every tick.
fn cell_filled(sim: &Simulation, pos: Pos) -> bool {
    let id = sim.world().get(pos);
    sim.registry().descriptor(id).is_some_and(|d| d != "minecraft:air")
}

/// The lowest y any entity occupies — box minimum for bodies, position for items.
fn min_entity_y(sim: &Simulation) -> Option<f64> {
    let bodies = sim.entity_bodies().iter().map(|b| b.min[1]);
    let items = sim.item_entities().iter().filter(|e| !e.removed).map(|e| e.pos[1]);
    bodies.chain(items).fold(None, |acc: Option<f64>, y| {
        Some(match acc {
            None => y,
            Some(low) if y < low => y,
            Some(low) => low,
        })
    })
}

/// The full vanilla wiring recipe, mirrored from `conformance.rs`.
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
    motion_data_version: Option<i32>,
    label: &str,
) -> Simulation {
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
        let name = structure.palette[entry].split('[').next().unwrap_or_default().to_string();
        let slots = mc_tick::vanilla::container_slots(&name)
            .unwrap_or_else(|| panic!("{label}: {name} has an inventory but no slot count"));
        sim.set_inventory(*pos, mc_tick::Inventory { slots, stacks: stacks.clone() });
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
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, hash_origin);
        *sim.behaviours_mut() = table;
    }
    assert_eq!(
        sim.unknown_report(),
        None,
        "{label}: every block must have behaviour, or this runs a partially-simulated world"
    );
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
                sim.spawn_authored_furnace_minecart(cart, None).unwrap_or_else(|e| panic!("{label}: {e}"));
            }
            mc_tick::structure::SpawnedEntity::Body(body) => {
                sim.spawn_authored_body(body).unwrap_or_else(|e| panic!("{label}: {e}"));
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
    sim
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
    let label = &case.name;
    let hash_origin = Pos::new(case.origin[0], case.origin[1], case.origin[2]);
    let action_states: Vec<String> = case.actions.iter().filter_map(|a| a.state.clone()).collect();
    let mut sim =
        build_sim(structure, hash_origin, case.settle, &action_states, motion_data_version, label);
    if let Some(seed) = case.seed {
        sim.set_rng_seed(seed);
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
    for tick in 0..=horizon {
        for (_, cells, peak) in watched.iter_mut() {
            let filled = cells.iter().filter(|pos| cell_filled(&sim, **pos)).count();
            *peak = (*peak).max(filled);
        }
        // Checks first: a check at T sees the world after exactly T ticks,
        // before any action scheduled at T fires.
        for (index, check) in case.checks.iter().enumerate().filter(|(_, c)| c.tick == tick) {
            let now = snapshot(&sim);
            at_tick.entry(tick).or_insert_with(|| now.clone());
            let (want, got) = match check.expect {
                Expect::Initial => (restrict(&initial, check.region), restrict(&now, check.region)),
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
                    (restrict(reference, check.region), restrict(&now, check.region))
                }
                Expect::Blocks => {
                    let blocks = check.blocks.as_ref().ok_or_else(|| {
                        format!("{label}: tick {tick}: \"blocks\" needs a \"blocks\" map")
                    })?;
                    for (key, want) in blocks {
                        let pos = parse_pos(key)
                            .map_err(|e| format!("{label}: tick {tick}: {e}"))?;
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
                                pos[i] >= f64::from(region[0][i]) && pos[i] < f64::from(region[1][i] + 1)
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
                            let what = expect.item.as_deref().or(expect.kind.as_deref()).unwrap_or("?");
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
                        let mut lines: Vec<String> =
                            blocking.iter().map(|(p, d)| format!("  {p:?}: {d}")).collect();
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
                    if let Some(why) =
                        check_bounds(check, &format!("the fill of {} cells", cells.len()), filled)
                    {
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
                        || seats.iter().zip(want_seats).any(|(a, b)| (a - b).abs() > 1.0e-9)
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
                _ => return Err(format!("{label}: tick {tick}: an action is either \"use\" or \"place\"+\"state\"")),
            }
        }
        if tick < horizon {
            sim.step();
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
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
