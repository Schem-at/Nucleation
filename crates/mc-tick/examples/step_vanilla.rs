//! Step the engine against a vanilla capture, tick by tick, and stop at the
//! first thing that disagrees — including the things a snapshot cannot see.
//!
//!     cargo run -p mc-tick --example step_vanilla -- <structure.snbt> <capture.json> \
//!         [--settle] [--use x,y,z@T] [--to N] [--all]
//!
//! `diff_vanilla` compares what *changed* each tick. This also compares what is
//! *pending*: the scheduled block ticks and the queued block events. That
//! matters because the two engines routinely agree on every block in the world
//! and disagree entirely on what is scheduled in it — the 3x3 flush door
//! reaches tick 1 with identical worlds and a different torch pending, and the
//! block diff only notices one tick later, by which point the cause is buried.
//!
//! The intended loop: step until it stops, fix the cause, re-capture if the fix
//! changes what vanilla would do, step again. `--all` reports every tick instead
//! of stopping, for judging whether a change helped overall.
use mc_tick::{Pos, Simulation, Structure};
use mc_tick_trace::{EventKind, Trace};
use std::collections::BTreeSet;

fn coords(text: &str) -> Pos {
    let p: Vec<i32> = text.split(',').map(|v| v.parse().expect("x,y,z")).collect();
    Pos::new(p[0], p[1], p[2])
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let structure_file = args.first().expect("structure.snbt").clone();
    let trace_file = args.get(1).expect("capture.json").clone();
    let settle = args.iter().any(|a| a == "--settle");
    let all = args.iter().any(|a| a == "--all");
    let limit: u64 = args
        .iter()
        .position(|a| a == "--to")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(u64::MAX);
    let uses: Vec<(Pos, u64)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--use")
        .filter_map(|(i, _)| args.get(i + 1))
        .map(|spec| {
            let (pos, tick) = spec.split_once('@').expect("x,y,z@tick");
            (coords(pos), tick.parse().expect("tick"))
        })
        .collect();

    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let structure_path = corpus.join("tests/corpus/structures").join(&structure_file);
    let trace_path = corpus.join("tests/traces").join(&trace_file);
    let text = std::fs::read_to_string(&structure_path).expect("read structure");
    let structure = Structure::parse(&text).expect("parse structure");
    let trace = Trace::from_json(&std::fs::read_to_string(&trace_path).expect("read trace"))
        .expect("parse trace");

    // The world position this build was recorded at. Only the `HashSet<BlockPos>`
    // in `updatePowerStrength` needs it, and only because that set iterates in an
    // order derived from absolute position — so a build recorded away from the
    // origin cannot be replayed zero-based without it.
    let hash_origin = trace
        .origin
        .map(|o| mc_tick::Pos::new(o[0], o[1], o[2]))
        .unwrap_or_default();
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, hash_origin);
        *sim.behaviours_mut() = table;
    }
    for pos in &structure.block_entities {
        sim.mark_block_entity(*pos);
    }
    for (pos, strength) in &structure.comparator_outputs {
        sim.set_comparator_output(*pos, *strength);
    }
    for (pos, stacks) in &structure.inventories {
        let entry = structure.blocks.iter().find(|(p, _)| p == pos).map(|(_, e)| *e);
        let Some(entry) = entry else { continue };
        let name = structure.palette[entry].split('[').next().unwrap_or_default();
        if let Some(slots) = mc_tick::vanilla::container_slots(name) {
            sim.set_inventory(*pos, mc_tick::Inventory { slots, stacks: stacks.clone() });
        }
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    // `--in-world` ticks the world exactly as loaded, with no placement pass.
    //
    // A placement recomputes what a running machine has already settled:
    // repeater LOCKED, wire connection shapes, and the live power a wire is
    // carrying. Reproducing a door that was *built* rather than stamped means
    // not re-deriving any of it — the states in the file are the truth.
    let in_world = args.iter().any(|a| a == "--in-world");
    if !in_world {
        sim.place_on_place(&order);
    }
    if settle && !in_world {
        sim.settle_with_order(&order);
    }
    sim.record();

    // The capture's trigger times are absolute game times; ours are tick
    // numbers. `game_time_at_start` is read *before* tick 0 runs, and vanilla
    // advances the clock at the head of a tick — so tick 0 happens at
    // `start + 1`, and a tick scheduled for game time G fires on tick
    // `G - start - 1`.
    let epoch = trace.game_time_at_start.unwrap_or(0).saturating_add(1);
    let horizon = trace
        .ticks
        .last()
        .map(|t| t.tick + 1)
        .unwrap_or(0)
        .max(trace.queues.last().map(|q| q.tick + 1).unwrap_or(0))
        .min(limit.saturating_add(1));

    let mut divergences = 0usize;
    for tick in 0..horizon {
        // Actuations first: the capture clicks *before* it dumps the queues, so
        // the recorded pending state already contains whatever the click
        // cascaded. Comparing before applying it here would hold the engine to
        // a world it has not been told about yet.
        for (pos, at) in &uses {
            if *at == tick {
                sim.use_block(*pos);
            }
        }

        // Compare what is *pending* before the tick runs. This is the earliest
        // point a divergence can be seen, and often several ticks before it
        // shows up as a block change.
        if let Some(record) = trace.queues.iter().find(|q| q.tick == tick) {
            let want: BTreeSet<(u64, i32, i32, i32)> = record
                .before
                .scheduled
                .iter()
                .map(|s| (s.at.saturating_sub(epoch), s.pos.0, s.pos.1, s.pos.2))
                .collect();
            let got: BTreeSet<(u64, i32, i32, i32)> = sim
                .pending_ticks()
                .into_iter()
                .map(|(at, pos)| (at, pos.x, pos.y, pos.z))
                .collect();
            if want != got {
                divergences += 1;
                println!("tick {tick:4}: SCHEDULES DIFFER");
                for entry in want.difference(&got) {
                    println!("            V  ({},{},{}) fires at tick {}", entry.1, entry.2, entry.3, entry.0);
                }
                for entry in got.difference(&want) {
                    println!("            E  ({},{},{}) fires at tick {}", entry.1, entry.2, entry.3, entry.0);
                }
                if !all {
                    println!("\nfirst divergence: pending schedules entering tick {tick}");
                    return;
                }
            }

            let want_events: Vec<(i32, i32, i32, i32)> = record
                .before
                .events
                .iter()
                .map(|e| (e.pos.0, e.pos.1, e.pos.2, e.id))
                .collect();
            let got_events: Vec<(i32, i32, i32, i32)> = sim
                .pending_events()
                .into_iter()
                .map(|(pos, id)| (pos.x, pos.y, pos.z, i32::from(id)))
                .collect();
            if want_events != got_events {
                divergences += 1;
                println!("tick {tick:4}: BLOCK EVENTS DIFFER (order is run order)");
                println!("            V  {want_events:?}");
                println!("            E  {got_events:?}");
                if !all {
                    println!("\nfirst divergence: queued block events entering tick {tick}");
                    return;
                }
            }
        }

        sim.step();

        // Then compare what changed during it.
        let want: BTreeSet<(i32, i32, i32, String)> = trace
            .ticks
            .iter()
            .find(|t| t.tick == tick)
            .map(|record| {
                record
                    .events
                    .iter()
                    .filter_map(|e| match &e.kind {
                        EventKind::BlockChanged { pos, to, .. } => {
                            Some((pos.0, pos.1, pos.2, to.clone()))
                        }
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        // Net change per position, matching the capture's snapshot view.
        let mut net: std::collections::BTreeMap<Pos, (mc_tick::StateId, mc_tick::StateId)> =
            std::collections::BTreeMap::new();
        for change in sim.recorded().iter().filter(|c| c.tick == tick) {
            net.entry(change.pos)
                .and_modify(|entry| entry.1 = change.to)
                .or_insert((change.from, change.to));
        }
        let got: BTreeSet<(i32, i32, i32, String)> = net
            .into_iter()
            .filter(|(_, (from, to))| from != to)
            .map(|(pos, (_, to))| {
                (
                    pos.x,
                    pos.y,
                    pos.z,
                    sim.registry().descriptor(to).unwrap_or("?").to_string(),
                )
            })
            .collect();
        if want != got {
            divergences += 1;
            println!("tick {tick:4}: BLOCKS DIFFER  vanilla={} engine={}", want.len(), got.len());
            for entry in want.difference(&got) {
                println!("            V  ({},{},{}) {}", entry.0, entry.1, entry.2, entry.3);
            }
            for entry in got.difference(&want) {
                println!("            E  ({},{},{}) {}", entry.0, entry.1, entry.2, entry.3);
            }
            if !all {
                println!("\nfirst divergence: block changes during tick {tick}");
                return;
            }
        } else if all && !want.is_empty() {
            println!("tick {tick:4}: match ({} events)", want.len());
        }
    }
    if divergences == 0 {
        println!("identical across {horizon} ticks — world, schedules and events");
    } else {
        println!("{divergences} divergence(s) across {horizon} ticks");
    }
}
