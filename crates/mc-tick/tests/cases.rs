//! Folder-driven, black-box scenario tests.
//!
//! Every `*.test.json` under `tests/cases/` is one case: a structure file, a
//! list of player actions, and end-state checks at named ticks. Adding a case
//! is adding files — nothing recompiles. The checks are deliberately blind to
//! *how* the engine got there (no traces, no event order): a faster redstone
//! backend that still opens and resets the door passes unchanged.
//!
//! Run one case while iterating: `MC_TICK_CASE=vault cargo test -p mc-tick --test cases`
//!
//! See `tests/cases/README.md` for the descriptor format.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mc_tick::{Pos, Simulation, Structure};
use serde::Deserialize;

const MARGIN: i32 = 4;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    /// Structure file, relative to the case file. Defaults to `<stem>.snbt`.
    #[serde(default)]
    structure: Option<String>,
    /// Where the capture's (0,0,0) sat in the game's coordinates — wire update
    /// order hashes absolute positions, so an in-world build needs its origin.
    #[serde(default)]
    origin: [i32; 3],
    #[serde(default)]
    settle: SettleMode,
    /// Seed for the vanilla random source. Behaviours that jitter (dispense
    /// trajectories, dispenser slot choice, destroy drops) draw from it in a
    /// fixed order, so a seeded case is exactly reproducible. Omitted: the
    /// engine uses each distribution's mean.
    #[serde(default)]
    seed: Option<i64>,
    #[serde(default)]
    actions: Vec<Action>,
    checks: Vec<Check>,
}

#[derive(Deserialize, PartialEq, Clone, Copy, Default)]
#[serde(rename_all = "kebab-case")]
enum SettleMode {
    /// Vanilla placement pass + ordered settle — a build saved at rest.
    #[default]
    Placement,
    /// `onPlace` only, no settle — a knownShape capture.
    Quiet,
    /// Neither — the build was recorded in the world it stood in, mid-state.
    InWorld,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Action {
    tick: u64,
    /// Right-click with an empty hand (a lever, a button, a note block).
    #[serde(rename = "use")]
    use_pos: Option<[i32; 3]>,
    /// Write a block state (`"minecraft:air"` breaks a block).
    place: Option<[i32; 3]>,
    state: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Check {
    tick: u64,
    /// "initial" — equals the world as settled, before any action.
    /// "changed" — differs from initial (the machine actually moved).
    /// "same-as" — equals the world at an earlier check's tick (`as_tick`).
    expect: Expect,
    #[serde(default)]
    as_tick: Option<u64>,
    /// Restrict the comparison to an inclusive box; whole world when absent.
    #[serde(default)]
    region: Option<[[i32; 3]; 2]>,
    /// With `expect: "blocks"`: `"x,y,z"` → expected state. A descriptor
    /// without properties matches on block name alone; listed properties must
    /// each hold, unlisted ones are free (`redstone_wire[power=15]` matches
    /// any fully-connected dust at power 15).
    #[serde(default)]
    blocks: Option<BTreeMap<String, String>>,
    /// With `expect: "entities"`: each entry must be satisfied.
    #[serde(default)]
    entities: Option<Vec<EntityExpect>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EntityExpect {
    /// Item id for item entities (`minecraft:iron_ingot`).
    item: Option<String>,
    /// Entity kind for minecarts (`minecraft:minecart`).
    kind: Option<String>,
    /// Inclusive block box the entity's position must fall inside.
    region: [[i32; 3]; 2],
    /// Exact count; when absent, at least one.
    count: Option<usize>,
    /// Container contents the item must carry (a dropped shulker box's
    /// slots): every listed `{id, count}` must appear.
    #[serde(default)]
    with_contents: Option<Vec<ContentExpect>>,
    /// The item must carry no container contents at all — a shulker box that
    /// was drained before it dropped.
    #[serde(default)]
    empty_contents: Option<bool>,
    /// Total item count summed over matching entities. Two ejected diamonds
    /// may merge into one entity of two, or land as two of one — this asserts
    /// the diamonds, not the entity bookkeeping.
    #[serde(default)]
    items_total: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentExpect {
    id: String,
    count: u8,
}

#[derive(Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum Expect {
    Initial,
    Changed,
    SameAs,
    Air,
    Blocks,
    Entities,
}

/// Does `actual` satisfy `expected`? Same block name, and every property the
/// expectation lists holds in the actual state; unlisted properties are free.
fn state_matches(expected: &str, actual: &str) -> bool {
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

/// A world snapshot: every non-air block, plus bounds for region reads.
type Snapshot = BTreeMap<Pos, String>;

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

/// The full vanilla wiring recipe, mirrored from `conformance.rs`.
fn build_sim(
    structure: &Structure,
    hash_origin: Pos,
    settle: SettleMode,
    extra_states: &[String],
    label: &str,
) -> Simulation {
    let mut sim = Simulation::new(structure.bounds(MARGIN));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
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

fn run_case(case_path: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(case_path)
        .map_err(|e| format!("reading {}: {e}", case_path.display()))?;
    let case: Case = serde_json::from_str(&text)
        .map_err(|e| format!("parsing {}: {e}", case_path.display()))?;
    let label = &case.name;

    let structure_path = match &case.structure {
        Some(rel) => case_path.parent().unwrap().join(rel),
        None => case_path.with_extension("").with_extension("snbt"),
    };
    let snbt = std::fs::read_to_string(&structure_path)
        .map_err(|e| format!("{label}: reading {}: {e}", structure_path.display()))?;
    let structure = Structure::parse(&snbt).map_err(|e| format!("{label}: parsing structure: {e:?}"))?;

    let hash_origin = Pos::new(case.origin[0], case.origin[1], case.origin[2]);
    let action_states: Vec<String> =
        case.actions.iter().filter_map(|a| a.state.clone()).collect();
    let mut sim = build_sim(&structure, hash_origin, case.settle, &action_states, label);
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

    let mut failures: Vec<String> = Vec::new();
    for tick in 0..=horizon {
        // Checks first: a check at T sees the world after exactly T ticks,
        // before any action scheduled at T fires.
        for check in case.checks.iter().filter(|c| c.tick == tick) {
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
                        let coords: Vec<i32> = key
                            .split(',')
                            .map(|c| c.trim().parse())
                            .collect::<Result<_, _>>()
                            .map_err(|e| format!("{label}: tick {tick}: bad position \"{key}\": {e}"))?;
                        if coords.len() != 3 {
                            return Err(format!("{label}: tick {tick}: position \"{key}\" is not x,y,z"));
                        }
                        let pos = Pos::new(coords[0], coords[1], coords[2]);
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

#[test]
fn every_bundled_case_passes() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
    let filter = std::env::var("MC_TICK_CASE").unwrap_or_default();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("tests/cases must exist")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().ends_with(".test.json")))
        .filter(|p| filter.is_empty() || p.to_string_lossy().contains(&filter))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no cases matched under {}", dir.display());

    let mut failed = Vec::new();
    for path in &paths {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        match run_case(path) {
            Ok(()) => eprintln!("case {name} ... ok"),
            Err(report) => {
                eprintln!("case {name} ... FAILED\n{report}");
                failed.push(name);
            }
        }
    }
    assert!(
        failed.is_empty(),
        "{} of {} cases failed: {}",
        failed.len(),
        paths.len(),
        failed.join(", ")
    );
}
