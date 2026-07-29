//! Differential conformance: the engine's trace against the real game's.
//!
//! Everything else in this crate's tests asserts what *we* think should happen.
//! This asserts that what happens matches what Minecraft actually did, on the same
//! structure file, tick for tick.
//!
//! # How a golden is made
//!
//! ```sh
//! tools/gametest/run.sh                         # once, to fetch and build
//! # then, per structure:
//! java -cp work/classes:$CP TraceCapture \
//!     --structure nucleation:piston_qc --out work/qc.json
//! cp work/qc.json crates/mc-tick/tests/traces/piston_qc.json
//! ```
//!
//! The structure lives once, in `tests/corpus/structures/`, and is copied into the
//! oracle's datapack — engine and game read the identical file, which is the only
//! way a diff means anything.
//!
//! # Why this is a hand-registered test rather than a corpus case
//!
//! The corpus runner registers no behaviour, so it cannot simulate. Wiring real
//! Minecraft blocks to behaviours needs a descriptor-to-behaviour registry that
//! does not exist yet — see `ROADMAP.md`. Until it does, conformance tests
//! register the handful of blocks their structure uses, explicitly, here.

use mc_tick::{Pos, Simulation, Structure};
use mc_tick_trace::{Detail, EventKind, Trace, TracePos};
use std::collections::HashMap;

/// Padding round a loaded structure; matches the corpus runner.
const MARGIN: i32 = 4;

fn structure(name: &str) -> Structure {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/structures")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    Structure::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn golden(name: &str) -> Trace {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/traces")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    Trace::from_json(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Turn the engine's recorded changes into a trace comparable with a capture.
///
/// The capture observes *between* ticks, so it cannot attribute an event to a
/// phase and tags everything `tick_end`. The engine could say more, but says the
/// same thing here — a richer claim on one side would diff as a difference.
///
/// For the same reason, same-tick changes at one position are collapsed to their
/// **net** change. A retraction writes `piston_head -> air -> moving_piston` into
/// the head slot within one tick; a snapshot diff can only ever see
/// `piston_head -> moving_piston`, and a no-op round trip not at all.
fn engine_trace(sim: &Simulation, name: &str, ticks: u64, size: (i32, i32, i32)) -> Trace {
    let mut trace = Trace::new("26.2", name, Detail::Normal);
    // The capture's entity observation window: the structure box inflated by
    // its MARGIN, exactly as TraceCapture builds it (min − 4 to size + 4 + 1).
    // An item that flies out of it reads as removed and re-appears when it
    // falls back in — the bubble-column golden does exactly that at its apex.
    let win_min = [-f64::from(MARGIN); 3];
    let win_max = [
        f64::from(size.0 + MARGIN + 1),
        f64::from(size.1 + MARGIN + 1),
        f64::from(size.2 + MARGIN + 1),
    ];
    let in_window = |pos: &[f64; 3]| {
        let min = [pos[0] - 0.125, pos[1], pos[2] - 0.125];
        let max = [pos[0] + 0.125, pos[1] + 0.25, pos[2] + 0.125];
        (0..3).all(|axis| min[axis] < win_max[axis] && max[axis] > win_min[axis])
    };
    let mut visible: HashMap<u32, bool> = HashMap::new();
    for tick in 0..ticks {
        let mut first_seen: Vec<mc_tick::Pos> = Vec::new();
        let mut net: std::collections::HashMap<mc_tick::Pos, (mc_tick::StateId, mc_tick::StateId)> =
            std::collections::HashMap::new();
        for c in sim.recorded().iter().filter(|c| c.tick == tick) {
            match net.entry(c.pos) {
                std::collections::hash_map::Entry::Vacant(slot) => {
                    slot.insert((c.from, c.to));
                    first_seen.push(c.pos);
                }
                std::collections::hash_map::Entry::Occupied(mut slot) => {
                    slot.get_mut().1 = c.to;
                }
            }
        }
        let mut events: Vec<_> = first_seen
            .into_iter()
            .filter_map(|pos| {
                let (from, to) = net[&pos];
                if from == to {
                    return None; // a round trip is invisible between ticks
                }
                Some(mc_tick_trace::TraceEvent {
                    phase: "tick_end".to_string(),
                    kind: EventKind::BlockChanged {
                        pos: TracePos::new(pos.x, pos.y, pos.z),
                        from: sim.registry().descriptor(from).unwrap_or("?").to_string(),
                        to: sim.registry().descriptor(to).unwrap_or("?").to_string(),
                    },
                })
            })
            .collect();

        // Container-slot changes, identically net-collapsed: the capture diffs
        // NBT snapshots, so it sees each slot's net change per tick.
        let render = |stack: &Option<(String, u8)>| match stack {
            None => String::new(),
            Some((id, count)) => format!("{count}x {id}"),
        };
        let mut inv_first: Vec<(mc_tick::Pos, u8)> = Vec::new();
        let mut inv_net: HashMap<(mc_tick::Pos, u8), (String, String)> = HashMap::new();
        for change in sim.recorded_inventory().iter().filter(|c| c.tick == tick) {
            match inv_net.entry((change.pos, change.slot)) {
                std::collections::hash_map::Entry::Vacant(slot) => {
                    slot.insert((render(&change.from), render(&change.to)));
                    inv_first.push((change.pos, change.slot));
                }
                std::collections::hash_map::Entry::Occupied(mut slot) => {
                    slot.get_mut().1 = render(&change.to);
                }
            }
        }
        for key in inv_first {
            let (from, to) = inv_net[&key].clone();
            if from == to {
                continue;
            }
            events.push(mc_tick_trace::TraceEvent {
                phase: "tick_end".to_string(),
                kind: EventKind::InventoryChanged {
                    pos: TracePos::new(key.0.x, key.0.y, key.0.z),
                    slot: u32::from(key.1),
                    from,
                    to,
                },
            });
        }
        // Entity movements and removals, exactly as the simulation emitted them
        // (already position-diffed per tick, mirroring the capture).
        for (event_tick, event) in sim.recorded_entities() {
            if *event_tick != tick {
                continue;
            }
            let kind = match event {
                mc_tick::sim::EntityEvent::Moved { id, entity_type, pos, velocity } => {
                    let now = in_window(pos);
                    let was = visible.insert(*id, now).unwrap_or(true);
                    if now {
                        EventKind::EntityMoved {
                            id: *id,
                            entity_type: entity_type.clone(),
                            pos: *pos,
                            velocity: *velocity,
                        }
                    } else if was {
                        EventKind::EntityRemoved { id: *id }
                    } else {
                        continue; // moving entirely outside the window: unseen
                    }
                }
                mc_tick::sim::EntityEvent::Removed { id } => {
                    let was = visible.insert(*id, false).unwrap_or(true);
                    if !was {
                        continue; // it already vanished from view
                    }
                    EventKind::EntityRemoved { id: *id }
                }
            };
            events.push(mc_tick_trace::TraceEvent { phase: "tick_end".to_string(), kind });
        }
        if !events.is_empty() {
            trace.ticks.push(mc_tick_trace::TickRecord { tick, events });
        }
    }
    trace
}

/// The golden's raw entity ids in first-appearance order — the order the
/// capture's snapshot walks them, which for structure-placed entities is
/// their spawn order.
fn golden_entity_ids(trace: &Trace) -> Vec<u32> {
    let mut ids: Vec<u32> = Vec::new();
    for record in &trace.ticks {
        for event in &record.events {
            if let EventKind::EntityMoved { id, .. } | EventKind::EntityRemoved { id } =
                &event.kind
            {
                if !ids.contains(id) {
                    ids.push(*id);
                }
            }
        }
    }
    ids
}

/// Renumber entity ids by first appearance, so the capture's server-global ids
/// compare against the engine's zero-based ones. Spawn order is deterministic
/// on both sides, which is what makes the mapping meaningful.
fn normalize_entity_ids(trace: &mut Trace) {
    let mut mapping: HashMap<u32, u32> = HashMap::new();
    for record in &mut trace.ticks {
        for event in &mut record.events {
            match &mut event.kind {
                EventKind::EntityMoved { id, .. } | EventKind::EntityRemoved { id } => {
                    let next = mapping.len() as u32;
                    let mapped = *mapping.entry(*id).or_insert(next);
                    *id = mapped;
                }
                _ => {}
            }
        }
    }
}

/// How the structure enters the world.
#[derive(Clone, Copy, PartialEq)]
enum Settle {
    /// Ordinary structure placement: every block is notified from every side by
    /// the placement update pass, so observers pulse and QC pistons fire.
    Placement,
    /// `StructurePlaceSettings.knownShape` placement: the update pass is
    /// skipped and **nothing** is dispatched — captured with the manual engine,
    /// which sits completely still until clicked. The engine equivalent is
    /// simply not settling.
    Quiet,
    /// Not placed at all. The capture ticked a build where it already stood in
    /// a saved world, so the states are loaded as they were left and nothing —
    /// not even `onPlace` — runs over them. Anything else re-derives a
    /// repeater's `locked` and a wire's connections from scratch, which is
    /// exactly what pasting a built machine gets wrong.
    InWorld,
}

/// An actuation applied at a tick boundary, mirroring the capture tool's flags.
///
/// The capture applies `--break`/`--pulse`/`--use` *between* server ticks; the
/// tick number here is the tick that will run next — the one whose snapshot diff
/// records the action, which is how the golden buckets it.
enum Actuate {
    /// Write a block state, as `--pulse` placing or removing its source does.
    /// Breaking a block is placing `minecraft:air`.
    Place(Pos, &'static str),
    /// Right-click with an empty hand, as `--use` does.
    Use(Pos),
}

/// Load a structure, wire vanilla behaviour to it, settle, actuate, and run.
///
/// No hand-registration: `mc_tick::vanilla` turns descriptors into behaviour, which
/// is what makes running an arbitrary schematic possible at all. `extra_states` are
/// interned before behaviours are bound, so an actuation may introduce a block the
/// structure itself never mentions (a pulse's redstone block, typically).
fn run_conformance_actuated(
    structure_file: &str,
    golden_file: &str,
    label: &str,
    extra_states: &[&str],
    actions: &[(u64, Actuate)],
) {
    run_conformance_bounded(
        structure_file,
        golden_file,
        label,
        extra_states,
        actions,
        None,
        Settle::Placement,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_conformance_bounded(
    structure_file: &str,
    golden_file: &str,
    label: &str,
    extra_states: &[&str],
    actions: &[(u64, Actuate)],
    ticking: Option<mc_tick::Bounds>,
    settle: Settle,
) {
    run_conformance_full(
        structure_file,
        golden_file,
        label,
        extra_states,
        actions,
        ticking,
        settle,
        1.0e-6,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_conformance_full(
    structure_file: &str,
    golden_file: &str,
    label: &str,
    extra_states: &[&str],
    actions: &[(u64, Actuate)],
    ticking: Option<mc_tick::Bounds>,
    settle: Settle,
    tolerance: f64,
) {
    let structure = structure(structure_file);
    // The goldens are snapshot-derived, so intra-tick order is the capture's scan
    // order rather than the game's causal order. Canonicalising both sides compares
    // *what* changed each tick, which is what such a capture actually knows.
    // An instrumented capture would know the real order and must not be canonicalised.
    let mut expected = golden(golden_file);
    normalize_entity_ids(&mut expected);
    // Where the capture's (0,0,0) sat in the game's coordinates. Needed because
    // `updatePowerStrength` iterates a `HashSet<BlockPos>` whose order follows
    // from absolute position — a build recorded away from the origin hands out
    // its neighbour updates in an order a zero-based replay cannot guess.
    let hash_origin = expected
        .origin
        .map(|o| mc_tick::Pos::new(o[0], o[1], o[2]))
        .unwrap_or_default();
    let expected = expected.canonicalized();

    let mut sim = Simulation::new(structure.bounds(MARGIN));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    // A dispenser can *place* a shulker box it holds as an item; behaviours
    // bind only to interned states, so intern every facing up front — the
    // same pre-intern the dynamic-case harness and the bridge perform.
    for (_, stacks) in &structure.inventories {
        for stack in stacks {
            let base = stack.id.split('[').next().unwrap_or(&stack.id);
            if base.ends_with("_shulker_box") || base == "minecraft:shulker_box" {
                for facing in ["up", "down", "north", "south", "west", "east"] {
                    let _ = sim.registry_mut().intern(&format!("{base}[facing={facing}]"));
                }
            }
        }
    }
    for descriptor in extra_states {
        sim.registry_mut()
            .intern(descriptor)
            .unwrap_or_else(|e| panic!("{label}: interning {descriptor}: {e:?}"));
    }

    // Container contents from the structure's block-entity NBT. Slot counts come
    // from the block name — the structure format does not carry them.
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
            mc_tick::Inventory { slots, stacks: stacks.clone() },
        );
    }

    // A build only contains the states it was saved with; a block needs its
    // counterparts to be able to change at all.
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, hash_origin);
        *sim.behaviours_mut() = table;
    }

    assert_eq!(
        sim.unknown_report(),
        None,
        "{label}: every block must have behaviour, or this compares vanilla against \
         a partially-simulated world"
    );

    // Item physics needs to know which states are solid and how slippery.
    {
        let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
        sim.set_physics_tables(solidity, frictions, heights, webs);
        let (water_kinds, bubble_kinds) = mc_tick::vanilla::fluid_tables(sim.registry());
        sim.set_fluid_tables(water_kinds, bubble_kinds);
        let (rails, conductors) = mc_tick::vanilla::rail_tables(sim.registry());
        sim.set_rail_tables(rails, conductors);
    }
    // Authored item entities spawn in list order, matching placement — and
    // with the server ids the golden recorded, because vanilla's rest-flush
    // cadence is `(tickCount + id) % 4`: the wrong id lifts a settled item on
    // a different tick than the capture shows.
    let raw_ids = golden_entity_ids(&golden(golden_file));
    for (index, spawned) in structure.entities.iter().enumerate() {
        let raw_id = raw_ids.get(index).copied();
        match spawned {
            mc_tick::structure::SpawnedEntity::Item(item) => match raw_id {
                Some(id) => sim.spawn_item_with_id(
                    id,
                    item.item.clone(),
                    item.pos,
                    item.motion,
                    item.pickup_delay,
                ),
                None => sim.spawn_item(item.item.clone(), item.pos, item.motion, item.pickup_delay),
            },
            mc_tick::structure::SpawnedEntity::Minecart(cart) => match raw_id {
                Some(id) => sim.spawn_minecart_with_id(id, cart.kind.clone(), cart.pos, cart.motion),
                None => sim.spawn_minecart(cart.kind.clone(), cart.pos, cart.motion),
            },
        };
    }

    // Ticking block entities register in structure order — vanilla's
    // tickBlockEntities insertion order for a placed structure, which is what
    // decides which of two hoppers moves first.
    for (pos, entry) in &structure.blocks {
        let state = sim.registry().get(&structure.palette[*entry]);
        let is_ticker = state
            .and_then(|s| sim.behaviours().get(s))
            .is_some_and(|b| b.ticks_as_block_entity());
        if is_ticker {
            sim.add_block_entity_ticker(*pos);
        }
    }

    if let Some(bounds) = ticking {
        sim.set_ticking_bounds(bounds);
    }

    // Placing a build gives every block a chance to react, exactly as vanilla's
    // onPlace does — which is why a piston notices a quasi-connectivity source that
    // touches it nowhere, and why every observer pulses once at placement.
    // Quiet (knownShape) placement dispatches nothing, so it settles nothing.
    // The capture's baseline snapshot is taken AFTER placement, so the settle's
    // own writes are pre-baseline and must not be recorded.
    // Vanilla's placement pass walks the structure's block list in order.
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    // `onPlace` runs on every write whatever the flags, so it happens for a
    // knownShape placement too — a piston already powered when it is placed
    // resolves and queues its block event with no update pass in sight.
    if settle != Settle::InWorld {
        sim.place_on_place(&order);
    }
    if settle == Settle::Placement {
        sim.settle_with_order(&order);
    }
    sim.record();

    let horizon = expected.ticks.last().map(|t| t.tick + 1).unwrap_or(0);
    for tick in 0..horizon {
        for (at, action) in actions {
            if *at != tick {
                continue;
            }
            match action {
                Actuate::Place(pos, descriptor) => {
                    let state = sim
                        .registry()
                        .get(descriptor)
                        .unwrap_or_else(|| panic!("{label}: {descriptor} was not interned"));
                    sim.place_block(*pos, state);
                }
                Actuate::Use(pos) => sim.use_block(*pos),
            }
        }
        sim.step();
    }

    let mut actual = engine_trace(&sim, label, horizon, structure.size);
    // Entity capture is opt-in on the Java side (--entities); a golden without
    // entity events compares against an engine trace without them. RNG-fed
    // spawns are exactly the case: the trajectory is sample-specific, and the
    // conformance claim is the container-visible effects.
    let golden_has_entities = expected.ticks.iter().any(|t| {
        t.events.iter().any(|e| {
            matches!(
                e.kind,
                EventKind::EntityMoved { .. } | EventKind::EntityRemoved { .. }
            )
        })
    });
    if !golden_has_entities {
        for record in &mut actual.ticks {
            record.events.retain(|e| {
                !matches!(
                    e.kind,
                    EventKind::EntityMoved { .. } | EventKind::EntityRemoved { .. }
                )
            });
        }
        actual.ticks.retain(|t| !t.events.is_empty());
    }
    normalize_entity_ids(&mut actual);
    let actual = actual.canonicalized();
    // Entity positions are floats; everything else still compares exactly. The
    // engine mirrors vanilla's arithmetic types, so the bar is tight.
    if let Some(divergence) = expected.diff_with_tolerance(&actual, tolerance) {
        panic!(
            "{label} diverges from vanilla at {divergence}\n\nexpected (vanilla):\n{}\n\nactual (engine):\n{}",
            expected.to_json().unwrap(),
            actual.to_json().unwrap()
        );
    }
}

/// The no-actuation case: settle and run.
fn run_conformance(structure_file: &str, golden_file: &str, label: &str) {
    run_conformance_actuated(structure_file, golden_file, label, &[], &[]);
}

#[test]
fn the_golden_traces_are_well_formed_and_from_the_right_version() {
    // A golden captured from another Minecraft version looks authoritative while
    // encoding different rules, which is worse than having none.
    let trace = golden("piston_qc.json");
    assert_eq!(trace.mc_version, "26.2");
    assert_eq!(trace.format_version, mc_tick_trace::FORMAT_VERSION);
    assert!(!trace.ticks.is_empty(), "a golden with no events proves nothing");
}

#[test]
fn piston_qc_matches_vanilla_tick_for_tick() {
    run_conformance("piston_qc.snbt", "piston_qc.json", "nucleation:piston_qc");
}

#[test]
fn slime_adhesion_matches_vanilla_tick_for_tick() {
    run_conformance("slime_drag.snbt", "slime_drag.json", "nucleation:slime_drag");
}

#[test]
fn the_manual_engine_runs_its_placement_cycle_tick_for_tick() {
    // The first real community schematic: a 2-step slimestone flying-machine
    // engine (`tools/gametest/samples/manual_engine.litematic`). Placement pulses
    // its observers, which acts as one trigger: the machine advances two full
    // 9-game-tick steps — pistons, slime adhesion, moved observers re-pulsing —
    // and stops at tick 21. Twenty-one ticks of interlocking behaviour, with no
    // actuation at all.
    //
    // The ticking bounds mirror the capture: its origin sat exactly on a chunk
    // corner, so blocks the machine pushes to x < 0 land in a chunk that is
    // loaded but not block-entity-ticking. Their moving_piston placeholders
    // freeze there — and, being immovable, they are what *stops* the machine
    // after its second step. Without the bounds the engine happily resolves
    // them and the machine takes a third step vanilla never took.
    run_conformance_bounded(
        "manual_engine.snbt",
        "manual_engine_settle.json",
        "nucleation:manual_engine",
        &[],
        &[],
        Some(mc_tick::Bounds::new(Pos::new(0, -4, 0), Pos::new(15, 7, 15))),
        Settle::Placement,
    );
}

#[test]
fn the_broken_flying_machine_breaks_the_same_way_vanilla_breaks_it() {
    // A 6-block two-piston flying machine that does not, in fact, fly: the
    // placement pulse pushes the front half one block east, nothing re-triggers,
    // and the machine sits split in two from tick 5 on. Reproducing a *broken*
    // contraption matters as much as reproducing a working one — a simulator
    // that "fixes" it is wrong.
    run_conformance(
        "flying_machine.snbt",
        "flying_machine.json",
        "nucleation:flying_machine",
    );
}

#[test]
fn clicking_the_manual_engine_advances_it_two_more_steps() {
    // The same engine, placed with room to fly: padded to the east end of the
    // capture's chunk so nothing crosses the frozen chunk border. Placement
    // runs the first two steps (ticks 0-25); the world then goes quiet; a
    // right-click on the note block — at [13,0,2], where the machine's two
    // steps left it — cycles its pitch on tick 30, the observers see the
    // change, and the engine runs a complete second activation through
    // tick 55. Fifty-five ticks, two full activations, one of them started by
    // the player-input path.
    run_conformance_bounded(
        "manual_engine_padded.snbt",
        "manual_engine_click.json",
        "nucleation:manual_engine_padded",
        &[],
        &[(30, Actuate::Use(Pos::new(13, 0, 2)))],
        Some(mc_tick::Bounds::new(Pos::new(0, -4, 0), Pos::new(15, 7, 15))),
        Settle::Placement,
    );
}

#[test]
fn a_quietly_placed_engine_stays_still_until_clicked() {
    // The designed behaviour, isolated from placement side effects:
    // knownShape placement dispatches nothing — the machine sits completely
    // still, QC-powered piston included, for ten ticks — and the click at the
    // note block's *as-built* position then runs exactly one activation: two
    // steps, starting at ticks 10 and 19, nine game ticks apart.
    run_conformance_bounded(
        "manual_engine_padded.snbt",
        "manual_engine_quiet_click.json",
        "nucleation:manual_engine_padded (quiet)",
        &[],
        &[(10, Actuate::Use(Pos::new(15, 0, 2)))],
        None,
        Settle::Quiet,
    );
}

#[test]
fn a_comparator_reads_a_container_through_its_rear() {
    // The first container behaviour: a barrel holding three full stacks reads
    // analog 2 (floor(3/27 * 14) + 1), and the comparator behind it turns on
    // one tick after placement — the boundary-scheduled 2-game-tick delay.
    run_conformance(
        "comparator_barrel.snbt",
        "comparator_barrel.json",
        "nucleation:comparator_barrel",
    );
}

#[test]
fn an_empty_container_turns_a_lit_comparator_off() {
    // The negative control, and the Some(0)-vs-None distinction: an empty
    // barrel has a real analog signal of 0, so a comparator authored powered
    // must schedule and turn off.
    run_conformance(
        "comparator_barrel_off.snbt",
        "comparator_barrel_off.json",
        "nucleation:comparator_barrel_off",
    );
}

#[test]
fn a_hopper_pulls_one_item_every_eight_ticks() {
    // A barrel above a hopper: one item moves per 8 game ticks, the first on
    // tick 0 — the hopper's block entity ticks in phase 9 of the very first tick.
    run_conformance("hopper_pull.snbt", "hopper_pull.json", "nucleation:hopper_pull");
}

#[test]
fn two_hoppers_race_in_block_entity_order() {
    // The discriminating capture for tick order. Hopper A (placed first, ticks
    // first) pushes into empty hopper B on tick 0; the destination-cooldown
    // rule gives B cooldown 8, B's own tick the same game tick decrements it to
    // 7, so B forwards to the barrel on tick 7 — not 8. An engine with the
    // wrong block-entity order or without the tickedGameTime comparison gets
    // every following tick wrong.
    run_conformance("hopper_race.snbt", "hopper_race.json", "nucleation:hopper_race");
}

#[test]
fn a_powered_hopper_is_locked_until_the_power_is_broken() {
    // Authored disabled beside a redstone block; breaking the block flips
    // `enabled` (silently, flag 2 — but visible) and the first transfer lands
    // the same tick.
    run_conformance_actuated(
        "hopper_locked.snbt",
        "hopper_locked.json",
        "nucleation:hopper_locked",
        &[],
        &[(0, Actuate::Place(Pos::new(1, 1, 0), "minecraft:air"))],
    );
}

#[test]
fn a_dropper_with_no_container_ejects_into_the_world() {
    // The pulse triggers at tick 0 (TRIGGERED flips, 4-tick boundary schedule
    // fires at tick 3); the front is air by then, so the item leaves as an item
    // entity — Milestone B's territory — and the container-side effect is
    // exactly the decrement.
    run_conformance_actuated(
        "dropper_into_barrel.snbt",
        "dropper_into_barrel.json",
        "nucleation:dropper_into_barrel",
        &["minecraft:redstone_block"],
        &[
            (0, Actuate::Place(Pos::new(1, 1, 0), "minecraft:redstone_block")),
            (2, Actuate::Place(Pos::new(1, 1, 0), "minecraft:air")),
        ],
    );
}

#[test]
fn a_dropper_facing_a_container_transfers_into_it() {
    run_conformance_actuated(
        "dropper_fill.snbt",
        "dropper_fill.json",
        "nucleation:dropper_fill",
        &["minecraft:redstone_block"],
        &[
            (0, Actuate::Place(Pos::new(0, 2, 0), "minecraft:redstone_block")),
            (2, Actuate::Place(Pos::new(0, 2, 0), "minecraft:air")),
        ],
    );
}

#[test]
fn a_comparator_follows_a_container_a_hopper_is_draining() {
    // The full chain: hopper transfers mutate the barrel, each mutation
    // notifies the comparator (vanilla's updateNeighbourForOutputSignal), and
    // when the last item leaves at tick 16 the comparator schedules its
    // 2-game-tick delay and goes dark at tick 18.
    run_conformance(
        "comparator_drain.snbt",
        "comparator_drain.json",
        "nucleation:comparator_drain",
    );
}

#[test]
fn an_ejected_item_flies_the_mean_trajectory_within_jitter_bounds() {
    // The RNG policy's conformance shape: vanilla jitters every velocity
    // component (`triangle(mean, 0.103)` plus a random 0.2..0.3 speed); the
    // engine spawns at the distribution means. The capture is one sample, so
    // entity positions compare with a tolerance sized to the jitter bounds over
    // this short flight — while the inventory decrement and TRIGGERED flips
    // stay exact.
    run_conformance_full(
        "dropper_eject.snbt",
        "dropper_eject.json",
        "nucleation:dropper_eject",
        &["minecraft:redstone_block"],
        &[
            (0, Actuate::Place(Pos::new(0, 2, 0), "minecraft:redstone_block")),
            (2, Actuate::Place(Pos::new(0, 2, 0), "minecraft:air")),
        ],
        None,
        Settle::Placement,
        0.5,
    );
}

#[test]
fn a_falling_item_follows_vanilla_gravity_and_drag_exactly() {
    // The strictest physics test the harness can express: an item authored in
    // the structure (no dispenser RNG anywhere) falls three blocks and lands.
    // The engine mirrors vanilla's arithmetic types — f32 drag widened to f64 —
    // so every position matches to the diff's 1e-6 tolerance and the item goes
    // silent once it rests.
    run_conformance("item_fall.snbt", "item_fall.json", "nucleation:item_fall");
}

#[test]
fn a_hopper_vacuums_a_falling_item() {
    // The item falls into the suck column (a full block from y+11/16 to y+2
    // above the hopper) and the whole two-item stack is absorbed at once:
    // inventory event and entity removal on the same tick.
    run_conformance(
        "item_into_hopper.snbt",
        "item_into_hopper.json",
        "nucleation:item_into_hopper",
    );
}

#[test]
fn resting_items_merge_on_the_slow_interval() {
    // Two stacks 0.3 apart merge on the 40-tick interval (tickCount 40 at
    // tick 39); the larger stack survives. Resting items emit nothing — their
    // gravity-accumulating velocity is flushed by collisions without the
    // position ever changing — so the only event in the whole trace is the
    // removal of the absorbed entity.
    run_conformance("item_merge.snbt", "item_merge.json", "nucleation:item_merge");
}

#[test]
fn breaking_a_dust_lines_source_drops_the_whole_line_in_one_tick() {
    // The wire network settles synchronously; the piston at the end never
    // fires because its placement-queued extend event fails dispatch
    // re-validation once the dust is dark — a three-mechanism interaction the
    // golden captures in a single tick.
    run_conformance_actuated(
        "dust_line.snbt",
        "dust_line.json",
        "nucleation:dust_line",
        &[],
        &[(0, Actuate::Place(Pos::new(0, 1, 0), "minecraft:air"))],
    );
}

#[test]
fn dust_soft_powers_a_piston_through_the_block_it_sits_on() {
    // Same structure, no break: the dust strongly powers its floor block, the
    // conductor re-emits, and the piston extends on tick 0.
    run_conformance("dust_line.snbt", "dust_softpower.json", "nucleation:dust_softpower");
}

#[test]
fn a_comparators_analog_strength_reaches_a_dust_line() {
    // Six full stacks in a barrel read 4; the dust line shows 4, 3, 2 — the
    // strength-plumbing the container milestone deferred, closed by reading
    // the comparator's stored block-entity output during wire evaluation.
    run_conformance(
        "comparator_dust.snbt",
        "comparator_dust.json",
        "nucleation:comparator_dust",
    );
}

#[test]
fn dust_descends_a_stone_step() {
    run_conformance_actuated(
        "dust_down_stone.snbt",
        "dust_down_stone.json",
        "nucleation:dust_down_stone",
        &[],
        &[(0, Actuate::Place(Pos::new(3, 2, 0), "minecraft:air"))],
    );
}

#[test]
fn dust_never_descends_past_glass_the_glass_diode() {
    // The asymmetry from the wire evaluator's bytecode: climbing reads need a
    // conductor step, descending reads need a non-conductor — so the lower
    // wire behind glass stays dark and only the upper wire's drop is captured.
    run_conformance_actuated(
        "dust_down_glass.snbt",
        "dust_down_glass.json",
        "nucleation:dust_down_glass",
        &[],
        &[(0, Actuate::Place(Pos::new(3, 2, 0), "minecraft:air"))],
    );
}

#[test]
fn a_stone_button_presses_for_twenty_ticks_and_the_lamp_follows() {
    // Click at tick 2: button and lamp on the same tick (lamps light
    // immediately), button releases at tick 21 (a boundary-scheduled 20), lamp
    // dims four ticks later.
    run_conformance_actuated(
        "button_lamp.snbt",
        "button_lamp.json",
        "nucleation:button_lamp",
        &[],
        &[(2, Actuate::Use(Pos::new(0, 1, 0)))],
    );
}

#[test]
fn a_wooden_button_presses_for_thirty() {
    run_conformance_actuated(
        "button_wood.snbt",
        "button_wood.json",
        "nucleation:button_wood",
        &[],
        &[(2, Actuate::Use(Pos::new(0, 1, 0)))],
    );
}

#[test]
fn a_falling_item_presses_a_wooden_plate() {
    // The Milestone B/C junction: an authored item falls nine ticks onto an
    // oak pressure plate; the plate and its lamp light on the landing tick and
    // stay lit while the item rests there.
    run_conformance("plate_item.snbt", "plate_item.json", "nucleation:plate_item");
}

#[test]
fn a_note_block_follows_neighbour_power_synchronously() {
    // Captured with `--pulse 1,0,0 --pulse-ticks 2`: the powered flag flips on the
    // same tick the source appears and again on the tick it vanishes — NoteBlock
    // has no scheduled delay at all.
    run_conformance_actuated(
        "note_powered.snbt",
        "note_powered.json",
        "nucleation:note_powered",
        &["minecraft:redstone_block"],
        &[
            (0, Actuate::Place(Pos::new(1, 0, 0), "minecraft:redstone_block")),
            (2, Actuate::Place(Pos::new(1, 0, 0), "minecraft:air")),
        ],
    );
}

#[test]
fn clicking_a_note_block_cycles_its_pitch_and_the_observer_sees_it() {
    // Captured with `--use 0,0,0 --use-tick 6`. Three things in one golden:
    // the observer's placement pulse (ticks 1 and 3), the click cycling `note`
    // on its own tick (6), and the observer pulsing one tick after the click
    // (7 and 9) — boundary scheduling, not the in-phase two-tick offset.
    run_conformance_actuated(
        "note_click.snbt",
        "note_click.json",
        "nucleation:note_click",
        &[],
        &[(6, Actuate::Use(Pos::new(0, 0, 0)))],
    );
}

#[test]
fn released_water_spreads_down_a_channel_at_five_gt_a_block() {
    // Milestone D's opening pin: a walled source behind a broken wall. The
    // boundary break schedules the source's fluid tick with the −1 fold (first
    // flow lands on tick 4), then the flow front advances one block every five
    // game ticks, levels 1 through 5.
    run_conformance_actuated(
        "water_spread.snbt",
        "water_spread.json",
        "nucleation:water_spread",
        &[],
        &[(0, Actuate::Place(Pos::new(2, 1, 1), "minecraft:air"))],
    );
}

#[test]
fn water_flows_only_toward_the_nearest_hole() {
    // The slope search: a T-junction whose east arm has a floor hole three
    // blocks out and whose south arm is flat. Vanilla's four-deep hole scan
    // sends every drop east — the south arm never floods — and the flow ends
    // by falling into the hole as level=8 falling water on tick 29.
    run_conformance_actuated(
        "water_hole.snbt",
        "water_hole.json",
        "nucleation:water_hole",
        &[],
        &[(0, Actuate::Place(Pos::new(2, 2, 1), "minecraft:air"))],
    );
}

#[test]
fn an_item_floats_up_through_still_water() {
    // Buoyancy, isolated: an authored item at the bottom of a two-deep walled
    // column. setUnderwaterMovement's 5e-4 nudge accumulates against the 0.99
    // horizontal drag until the item bobs at the surface — 120 ticks of pure
    // fluid physics, no RNG anywhere.
    run_conformance_full(
        "item_float.snbt",
        "item_float_water.json",
        "nucleation:item_float",
        &[],
        &[],
        None,
        Settle::Quiet,
        1.0e-6,
    );
}

#[test]
fn a_stream_carries_an_item_to_the_far_wall() {
    // The sorting-machine primitive: an authored source-to-level-7 gradient
    // (quiet placement keeps it frozen, exactly as knownShape does) and an
    // item dropped upstream. getFlow's height differences push it 0.014 a
    // tick until it pins against the end wall six blocks later.
    run_conformance_full(
        "item_stream.snbt",
        "item_stream.json",
        "nucleation:item_stream",
        &[],
        &[],
        None,
        Settle::Quiet,
        1.0e-6,
    );
}

#[test]
fn bubble_columns_lift_and_sink_items() {
    // Both column kinds in one golden: soul sand's upward column launches its
    // item out of the water (the above-column +0.1/1.8 clamp at the open top),
    // magma's drag column pulls its item to the floor where it rests. The
    // per-cell clamps come straight from Entity.handleOnInsideBubbleColumn.
    run_conformance_full(
        "bubble.snbt",
        "bubble.json",
        "nucleation:bubble",
        &[],
        &[],
        None,
        Settle::Quiet,
        1.0e-6,
    );
}

#[test]
fn blue_ice_lets_an_item_glide_and_stone_stops_it() {
    // Milestone E's first surface: two identical items launched at 0.25/tick,
    // one over blue ice (ground drag 0.98 × 0.989), one over stone (0.98 ×
    // 0.6). A hundred ticks later the ice item is still gliding past x=9 while
    // the stone item stopped short — the friction table, pinned end to end.
    run_conformance_full(
        "item_ice.snbt",
        "item_ice.json",
        "nucleation:item_ice",
        &[],
        &[],
        None,
        Settle::Quiet,
        1.0e-6,
    );
}

#[test]
fn a_cobweb_slows_a_falling_item_to_a_crawl() {
    // WebBlock.entityInside arms Entity.stuckSpeedMultiplier (0.25, 0.05f,
    // 0.25): the next move is scaled per axis and the velocity zeroed, every
    // tick the box still touches the web — a 0.04 gravity step becomes a
    // 0.002 descent, for as long as the golden runs.
    run_conformance_full(
        "item_web.snbt",
        "item_web.json",
        "nucleation:item_web",
        &[],
        &[],
        None,
        Settle::Quiet,
        1.0e-6,
    );
}

#[test]
fn an_item_rests_on_soul_sand_at_fourteen_sixteenths() {
    // Soul sand's collision column tops at 14/16: the item dropped on it
    // lands at exactly y = 0.875 while its stone-lane twin lands at 1.0 —
    // the partial-height solid in the collision clip.
    run_conformance_full(
        "item_soulsand.snbt",
        "item_soulsand.json",
        "nucleation:item_soulsand",
        &[],
        &[],
        None,
        Settle::Quiet,
        1.0e-6,
    );
}

fn run_cart(structure_file: &str, golden_file: &str, label: &str) {
    run_conformance_full(
        structure_file,
        golden_file,
        label,
        &[],
        &[],
        None,
        Settle::Quiet,
        1.0e-6,
    );
}

#[test]
fn a_cart_powers_a_detector_rail_and_it_releases_twenty_ticks_later() {
    // The rail powers the tick the cart's box enters `getSearchBB` — the cell
    // inset 0.2 on every side but the bottom — and lights lamps above, beside
    // and below it at 15. It does *not* release when the cart leaves:
    // `checkPressed` books a tick 20 later and only that recheck clears it, so
    // the golden shows power on tick 13 and off on tick 33 with the cart long
    // gone. The lamps then follow four ticks behind, on their own off-delay.
    run_cart("detector_rail.snbt", "detector_rail.json", "nucleation:detector_rail");
}

#[test]
fn a_detector_rail_strongly_powers_only_the_block_beneath_it() {
    // `DetectorRailBlock.getDirectSignal` answers 15 for `Direction.UP` alone.
    // The dust here touches the block *under* the rail and nothing else — it
    // reads 15 — while the control dust under a plain rail one block along
    // stays dark for the whole run.
    run_cart("detector_strong.snbt", "detector_strong.json", "nucleation:detector_strong");
}

#[test]
fn weighted_plates_count_entities_and_the_two_kinds_scale_differently() {
    // `getSignalStrength` is `ceil(min(count, maxWeight) * 15 / maxWeight)`
    // over *every* entity class, so dropped items count. The light plate's
    // maxWeight is 15 — one item, one level — and the heavy plate's is 150,
    // one level per ten. The golden holds six plates: light under 1/3/5 items
    // reading 1/3/5, heavy under 1/3/11 reading 1/1/2.
    run_cart("weighted_plates.snbt", "weighted_plates.json", "nucleation:weighted_plates");
}

#[test]
fn a_coasting_cart_decays_at_ninety_six_percent_and_flies_off_the_end() {
    // OldMinecartBehavior on a plain line: velocity projected onto the rail
    // chord, ×0.96 a tick, then comeOffTrack past the last rail — clamp,
    // ground-halving, landing.
    run_cart("cart_flat.snbt", "cart_flat.json", "nucleation:cart_flat");
}

#[test]
fn powered_rails_accelerate_a_cart_past_the_movement_clamp() {
    // +0.06 along the motion every tick: the velocity grows well past 0.4
    // while the per-axis movement clamp holds the cart to 0.4 blocks a tick —
    // both visible in the golden's diverging velocity and constant stride.
    run_cart("cart_boost.snbt", "cart_boost.json", "nucleation:cart_boost");
}

#[test]
fn unpowered_golden_rails_brake_a_cart_to_a_standstill() {
    // The braking branch: ×0.5 a tick with vy zeroed, then a dead stop the
    // tick the horizontal speed dips under 0.03.
    run_cart("cart_brake.snbt", "cart_brake.json", "nucleation:cart_brake");
}

#[test]
fn a_cart_rolls_down_a_slope_and_gains_speed() {
    // The ascending rail's 0.0078125 downhill pull, the y+1 seat, the corner
    // fixup as the cart crosses onto the low line, and the 0.05 height
    // correction feeding slope drop into speed.
    run_cart("cart_slope.snbt", "cart_slope.json", "nucleation:cart_slope");
}

#[test]
fn a_cart_turns_a_corner_without_losing_the_rail() {
    // The chord projection through a north_west corner: eastbound velocity is
    // re-aimed north, the exit-crossing redirect included.
    run_cart("cart_curve.snbt", "cart_curve.json", "nucleation:cart_curve");
}

#[test]
fn a_cart_circulates_a_powered_loop_indefinitely() {
    // The integration pin: a rectangular circuit through all four corner
    // shapes with boosted straights. Two hundred ticks, several laps, every
    // projection, redirect, corner fixup and boost landing exactly where
    // vanilla put them.
    run_cart("cart_loop.snbt", "cart_loop.json", "nucleation:cart_loop");
}

#[test]
fn a_redstone_block_lights_nine_golden_rails_and_no_more() {
    // findPoweredRailSignal: the direct neighbour plus a chain of at most 8
    // already-powered rails — nine light up, the tenth stays dark, and the
    // whole wave lands inside one tick's update cascade. Both edges captured.
    run_conformance_actuated(
        "rails_chain.snbt",
        "rails_chain.json",
        "nucleation:rails_chain",
        &["minecraft:redstone_block"],
        &[
            (0, Actuate::Place(Pos::new(0, 1, 1), "minecraft:redstone_block")),
            (2, Actuate::Place(Pos::new(0, 1, 1), "minecraft:air")),
        ],
    );
}

#[test]
fn powering_the_rails_launches_a_parked_cart_off_the_wall() {
    // The full circle: a pulse powers the chain, the launch branch reads the
    // wall as a conductor and hands the resting cart 0.02 east, the boosts
    // take over, and the cart runs out onto plain rail and off the end.
    run_conformance_full(
        "cart_launch.snbt",
        "cart_launch.json",
        "nucleation:cart_launch",
        &["minecraft:redstone_block"],
        &[
            (0, Actuate::Place(Pos::new(1, 2, 1), "minecraft:redstone_block")),
            (40, Actuate::Place(Pos::new(1, 2, 1), "minecraft:air")),
        ],
        None,
        Settle::Quiet,
        1.0e-6,
    );
}

#[test]
fn flipping_a_lever_lights_a_lamp_through_its_support_block() {
    // The lever's strong emission: the flip powers its wall block strongly,
    // the conductor re-emits, and the lamp on the far side lights on the
    // click's own tick.
    run_conformance_full(
        "lever_lamp.snbt",
        "lever_lamp.json",
        "nucleation:lever_lamp",
        &[],
        &[(5, Actuate::Use(Pos::new(0, 1, 1)))],
        None,
        Settle::Quiet,
        1.0e-6,
    );
}

fn run_door(structure_file: &str, golden_file: &str, label: &str, lever: Option<(Pos, u64)>) {
    run_door_cycle(structure_file, golden_file, label, &lever.into_iter().collect::<Vec<_>>());
}

/// A door driven by any number of lever clicks — a full close/open cycle is
/// two of them.
fn run_door_cycle(structure_file: &str, golden_file: &str, label: &str, levers: &[(Pos, u64)]) {
    let actions: Vec<(u64, Actuate)> = levers
        .iter()
        .map(|(pos, tick)| (*tick, Actuate::Use(*pos)))
        .collect();
    run_conformance_full(
        structure_file,
        golden_file,
        label,
        &[],
        &actions,
        None,
        Settle::Placement,
        1.0e-6,
    );
}

#[test]
fn the_3x3_flush_synced_settles_like_vanilla() {
    // A whole community door, matched tick for tick — the first one. No lever
    // in the build: its placement settle *is* the behaviour, fourteen ticks of
    // torches, repeaters and two stacked sticky pistons closing the door.
    //
    // Held to the quiet capture, which the stepper (`step_vanilla`) compares on
    // world, pending schedules and queued block events alike. The loud capture
    // is kept alongside it as `door_3x3_flush.json`.
    run_door(
        "door_3x3_flush.snbt",
        "door_3x3_flush_quiet.json",
        "nucleation:door_3x3_flush",
        None,
    );
}

/// The 4x4 vault door, recorded in the world it was built in.
///
/// The one door that had never been compared against a reference worth
/// trusting. Every earlier capture pasted it into an empty world first, and
/// `placeInWorld` re-derives repeater `locked` and wire connections and loads
/// block-entity NBT after the block write — so the memory cell that holds the
/// door came up unlatched and the machine could not run. Ticked where it
/// stands, it seals completely, four strokes running.
///
/// Three lever clicks: close, open, close. The second close is event-for-event
/// identical to the first, which is the real test — a door that opens and does
/// not return to the state it started in is a door that works once.
#[test]
fn the_4x4_vault_door_runs_a_full_cycle_in_the_world_it_was_built_in() {
    run_conformance_full(
        "door_4x4_vault_inworld.snbt",
        "door_4x4_vault_inworld.json",
        "nucleation:door_4x4_vault",
        &[],
        &[
            (10, Actuate::Use(Pos::new(7, 5, 1))),
            (40, Actuate::Use(Pos::new(7, 5, 1))),
            (70, Actuate::Use(Pos::new(7, 5, 1))),
        ],
        None,
        Settle::InWorld,
        1.0e-6,
    );
}

/// The 4x4 sliding door, a full close/open cycle.
///
/// Slime and honey both, so its panels travel in dragged trains rather than
/// being pushed directly — the mechanism the vault door's opening stroke turned
/// on. Exact for all twenty-two recorded ticks once the wire hash was given the
/// world origin the capture recorded.
#[test]
fn the_4x4_sliding_door_runs_a_full_close_open_cycle() {
    run_door_cycle(
        "door_4x4_sliding.snbt",
        "door_4x4_sliding_cycle.json",
        "nucleation:door_4x4_sliding",
        &[(Pos::new(7, 3, 0), 10), (Pos::new(7, 3, 0), 60)],
    );
}

/// The same door under stress: six clicks, two of them mid-stroke.
///
/// A close interrupted five ticks in, resumed fifteen ticks later, then a pair
/// two ticks apart — the patterns a player produces and a fixed close/open
/// cycle never exercises. Thirty-six of the forty recorded ticks match,
/// including a 151-event tick, so interrupting a stroke and re-triggering it
/// are both right.
///
/// It matched the last four ticks only once the wire hash was given the world
/// origin: they turn on a piston race at tick 80, and which piston wins follows
/// from the order two of them queued their block events in thirty-five ticks
/// earlier.
#[test]
fn the_4x4_vault_door_survives_interrupted_and_repeated_clicks() {
    run_conformance_full(
        "door_4x4_vault_inworld.snbt",
        "door_4x4_vault_stress.json",
        "nucleation:door_4x4_vault",
        &[],
        &[
            (10, Actuate::Use(Pos::new(7, 5, 1))),
            (15, Actuate::Use(Pos::new(7, 5, 1))),
            (30, Actuate::Use(Pos::new(7, 5, 1))),
            (45, Actuate::Use(Pos::new(7, 5, 1))),
            (47, Actuate::Use(Pos::new(7, 5, 1))),
            (80, Actuate::Use(Pos::new(7, 5, 1))),
        ],
        None,
        Settle::InWorld,
        1.0e-6,
    );
}

/// The 6x6 sliding door, recorded in the world Harrison built it in.
///
/// He pasted the litematic into a creative world and it ran without a tweak, so
/// the door is sound and the paste is not what breaks it here — unlike the vault
/// door, whose latched memory cell a paste destroys.
///
/// Every tick of the cycle is identical, block for block — open and close, all
/// 125 block events of the opening tick included.
///
/// The last thing standing was a dust connection: an *extended* piston base is a
/// 12/16 box pushed to the far side of its facing, so its top face is a full
/// square only when it faces down. The wire beside the edge piston reads that
/// face to decide whether to climb, and getting it wrong dropped the connection
/// to `none` where the game lowers it to `side`.
#[test]
fn the_6x6_sliding_door_runs_a_cycle_in_the_world_it_was_built_in() {
    run_conformance_full(
        "door_6x6_inworld.snbt",
        "door_6x6_inworld.json",
        "nucleation:door_6x6_sliding",
        &[],
        &[
            (10, Actuate::Use(Pos::new(10, 4, 1))),
            (60, Actuate::Use(Pos::new(10, 4, 1))),
        ],
        None,
        Settle::InWorld,
        1.0e-6,
    );
}

/// A dust corner losing the block beside it, which is the vault door's opening
/// stroke reduced to twenty-four blocks.
///
/// A sticky piston pulls a redstone block out from beside a dust corner. The
/// slot it vacates is cleared with flag 82 — `UPDATE_KNOWN_SHAPE`, silent — and
/// `moveBlocks` then runs `updateNeighbourShapes` over it by hand, which is the
/// only thing that tells the dust to re-examine its connections. It drops the
/// west connection and the symmetry rule runs it north-south instead.
///
/// It also pins the registry: the state the dust turns into appears nowhere in
/// the schematic, and a rewrite naming a state that does not exist is dropped
/// in silence.
#[test]
fn a_piston_pulling_a_block_from_beside_a_dust_corner_reshapes_it() {
    run_conformance_full(
        "wire_corner_pull.snbt",
        "wire_corner_pull.json",
        "nucleation:wire_corner_pull",
        &[],
        &[(10, Actuate::Use(Pos::new(0, 2, 1)))],
        None,
        Settle::Placement,
        1.0e-6,
    );
}

#[test]
#[ignore = "the standing target: a full close/open cycle, two lever clicks 70 \
ticks apart. Re-captured with its world origin, which moved it much closer — \
the old reference disagreed from tick 0 by dozens of events. It now diverges at \
tick 1 on two wire states, during the placement cascade rather than the door. \
See ROADMAP 'The door fixtures'."]
fn the_6x6_door_runs_a_full_close_open_cycle() {
    // The cycle golden: vanilla opens on the tick-10 click (through tick 25)
    // and closes on the tick-80 one (through tick 95), 2035 events in all.
    run_door_cycle(
        "door_6x6_sliding.snbt",
        "door_6x6_cycle.json",
        "nucleation:door_6x6_cycle",
        &[(Pos::new(9, 3, 0), 10), (Pos::new(9, 3, 0), 80)],
    );
}

#[test]
fn opposed_pistons_race_for_one_gap() {
    // The vault door's failure, minimised: a piston facing up under five
    // concrete, a one-block gap, a piston facing down above it, and an
    // observer for each. A loud placement pulses both observers, so both
    // pistons want the gap, and vanilla's *down* piston wins — even though its
    // observer sits at y=9 and the other at y=1.
    //
    // Placement order has no say in it: the observers are pulsed by
    // `updateShapeAtEdge`, which walks the surface of the placed shape by axis
    // (`Simulation::update_shape_at_edge`), not by the placement list. The
    // companion `piston_race_quiet` fixture isolates the other half — pistons
    // already powered when placed, where placement order *is* the tiebreak.
    run_conformance("piston_race.snbt", "piston_race.json", "nucleation:piston_race");
}

#[test]
fn a_zero_tick_generator_teleports_its_redstone_block() {
    // A 0-tick generator, reduced from the 4x4 vault door's corner: dust
    // strongly powers a concrete block, the concrete powers the sticky piston
    // on top of it, and that piston pushes a redstone block one south. The
    // redstone block was quasi-powering a second sticky piston below, which
    // now retracts and pulls the concrete out from under the first — ending its
    // pulse before the push completes.
    //
    // Vanilla's answer is the 0-tick: the redstone block *teleports* one block
    // in the same tick, with no `moving_piston` stage, because the retract
    // finds an extending moving block two ahead and `finalTick`s it in place.
    //
    // Every step of that chain is a separate vanilla behaviour, and the last
    // one to arrive was the notification pass over the positions a *pull*
    // vacates — without it the upper piston never hears its pulse end and the
    // block completes an ordinary two-tick move instead.
    run_conformance_bounded(
        "piston_handoff.snbt",
        "piston_handoff.json",
        "nucleation:piston_handoff",
        &[],
        &[(3, Actuate::Use(Pos::new(3, 1, 1)))],
        None,
        Settle::Quiet,
    );
}

#[test]
fn the_first_placed_of_two_powered_pistons_takes_the_shared_gap() {
    // The placement-order discriminator. Two opposed pistons, each already
    // powered by its own redstone block, one gap between them: both resolve in
    // `onPlace`, so whichever is placed first queues its block event first and
    // takes the gap. Nothing else can break the tie — knownShape placement runs
    // no update pass at all.
    //
    // Vanilla's *bottom* piston (y=1) wins, which is the direct evidence that
    // the placement walk ascends y, matching `buildInfoList`'s comparators.
    run_conformance_bounded(
        "piston_race_quiet.snbt",
        "piston_race_quiet.json",
        "nucleation:piston_race_quiet",
        &[],
        &[],
        None,
        Settle::Quiet,
    );
}

#[test]
#[ignore = "isolated fixture: vanilla's slime drags the floor row and the side \
pistons; the engine only pushes the slime."]
fn a_slime_extender_drags_its_riders_and_the_floor() {
    // A sticky piston, a slime block, a piston riding each side of the slime,
    // and a lever. Vanilla drags both riders *and* the stone floor beneath the
    // slime, shoving the whole row; this engine extends and pushes the slime
    // alone. Same event count, different blocks — the difference is what the
    // push structure gathers, not whether it fires.
    run_conformance_actuated(
        "slime_extender.snbt",
        "slime_extender.json",
        "nucleation:slime_extender",
        &[],
        &[(5, Actuate::Use(Pos::new(1, 2, 1)))],
    );
}

#[test]
fn two_pistons_extend_into_a_gap_each() {
    // Symmetric double extension: two pistons face each other across two
    // free blocks, each with its own observer, so both extend and neither
    // races. Built as a discriminator for placement-walk order and failed at
    // that (it needs a *single* shared gap to force a race) — kept because it
    // pins symmetric extension cheaply, and it passes.
    run_conformance(
        "observer_order.snbt",
        "observer_order.json",
        "nucleation:observer_order",
    );
}

#[test]
fn a_trapdoor_toggles_open_and_shut_like_vanilla() {
    // Oak (facing=north, bottom) and iron (facing=south, top) beside one
    // redstone block: open+powered flip together on the power edge, both ways.
    // Captured with the --at schedule that mirrors the dynamic case exactly.
    run_conformance_actuated(
        "trapdoor_rig.snbt",
        "trapdoor_toggle.json",
        "nucleation:trapdoor_toggle",
        &["minecraft:redstone_block"],
        &[
            (2, Actuate::Place(Pos::new(1, 0, 0), "minecraft:redstone_block")),
            (8, Actuate::Place(Pos::new(1, 0, 0), "minecraft:air")),
        ],
    );
}

#[test]
fn a_piston_pushes_a_trapdoor_intact() {
    // Trapdoors are movable (no getPistonPushReaction override): the push
    // carries the trapdoor with its properties, and the retract leaves it.
    run_conformance_actuated(
        "trapdoor_rig.snbt",
        "trapdoor_push.json",
        "nucleation:trapdoor_push",
        &["minecraft:redstone_block"],
        &[
            (2, Actuate::Place(Pos::new(0, 1, 2), "minecraft:redstone_block")),
            (8, Actuate::Place(Pos::new(0, 1, 2), "minecraft:air")),
        ],
    );
}

#[test]
fn a_piston_destroys_a_door_and_takes_the_other_half_with_it() {
    // Five lanes, one capture, and the same answer in every one: a door is
    // `PushReaction.DESTROY`. Whichever half the piston reaches is broken
    // rather than carried, and the untouched half breaks with it because a
    // door half cannot survive its partner's absence.
    //
    //   z=0   normal piston at the LOWER half of an oak door
    //   z=3   normal piston at the UPPER half of an oak door
    //   z=6   normal piston at the lower half of an IRON door
    //   z=9   sticky piston: the push destroys, so the retract pulls nothing
    //   z=12  a slime array carrying a companion block, both halves at once
    //
    // Every destination cell is floored, so "the door is gone" cannot be
    // confused with "the door landed somewhere it could not stand". Iron and
    // oak behave identically, which is why one lane of each suffices.
    //
    // This overturned the engine: doors were modelled as ordinary pushable
    // material, and carried the door forward in all five lanes.
    run_conformance_actuated(
        "door_push.snbt",
        "door_push.json",
        "nucleation:door_push",
        &["minecraft:redstone_block"],
        &[
            (2, Actuate::Place(Pos::new(0, 1, 0), "minecraft:redstone_block")),
            (2, Actuate::Place(Pos::new(0, 2, 3), "minecraft:redstone_block")),
            (2, Actuate::Place(Pos::new(0, 1, 6), "minecraft:redstone_block")),
            (2, Actuate::Place(Pos::new(0, 1, 9), "minecraft:redstone_block")),
            (2, Actuate::Place(Pos::new(0, 1, 12), "minecraft:redstone_block")),
            (8, Actuate::Place(Pos::new(0, 1, 0), "minecraft:air")),
            (8, Actuate::Place(Pos::new(0, 2, 3), "minecraft:air")),
            (8, Actuate::Place(Pos::new(0, 1, 6), "minecraft:air")),
            (8, Actuate::Place(Pos::new(0, 1, 9), "minecraft:air")),
            (8, Actuate::Place(Pos::new(0, 1, 12), "minecraft:air")),
        ],
    );
}

#[test]
fn the_flying_machine_flies_as_vanilla_does() {
    // Engine B, quiet placement, redstone-block kick at t2 removed at t4 —
    // then 70 ticks of unassisted flight, block for block against the game.
    run_conformance_bounded(
        "flying_machine_east.snbt",
        "flying_kick.json",
        "nucleation:flying_machine_east",
        &["minecraft:redstone_block"],
        &[
            (2, Actuate::Place(Pos::new(2, 1, 1), "minecraft:redstone_block")),
            (4, Actuate::Place(Pos::new(2, 1, 1), "minecraft:air")),
        ],
        None,
        Settle::Quiet,
    );
}

#[test]
fn the_shulker_pipeline_runs_as_vanilla_does() {
    // The full two-phase machine: dispense-place, hopper-drain, double dropper
    // eject, lock, piston break, vacuum, ship. Item trajectories are RNG-fed
    // in the game and mean-fed here, so this compares the block and container
    // record; where a vacuum lands a tick apart the divergence is the sample,
    // not the machine — see the doc note beside the golden if that surfaces.
    run_conformance_actuated(
        "shulker_pipeline.snbt",
        "shulker_full.json",
        "nucleation:shulker_pipeline",
        &["minecraft:redstone_block"],
        &[
            (5, Actuate::Place(Pos::new(-1, 2, 1), "minecraft:redstone_block")),
            (30, Actuate::Place(Pos::new(2, 0, 1), "minecraft:redstone_block")),
            (34, Actuate::Place(Pos::new(2, 0, 1), "minecraft:air")),
            (38, Actuate::Place(Pos::new(2, 0, 1), "minecraft:redstone_block")),
            (42, Actuate::Place(Pos::new(2, 0, 1), "minecraft:air")),
            (46, Actuate::Place(Pos::new(1, 1, 2), "minecraft:redstone_block")),
            (50, Actuate::Place(Pos::new(1, 4, 1), "minecraft:redstone_block")),
            (54, Actuate::Place(Pos::new(1, 4, 1), "minecraft:air")),
            (60, Actuate::Place(Pos::new(1, 1, 2), "minecraft:air")),
            (82, Actuate::Place(Pos::new(2, 0, 1), "minecraft:redstone_block")),
            (86, Actuate::Place(Pos::new(2, 0, 1), "minecraft:air")),
        ],
    );
}
