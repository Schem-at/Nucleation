//! Compare the engine against a captured vanilla trace, tick by tick.
//!
//! ```sh
//! cargo run -p mc-tick --example diff_vanilla -- \
//!     door_4x4_vault.snbt door_4x4_vault.json --settle --use 6,4,0@10
//! ```
//!
//! Prints one line per tick — `MATCH` or the exact events each side has that
//! the other does not — then a summary naming the first divergence. This is
//! the tool to reach for before theorising: it turns "the door is wrong" into
//! "at tick 1, vanilla moves these two pistons and we move those two".
//!
//! Positions can be filtered with `--region x0,y0,z0..x1,y1,z1` to watch one
//! mechanism, and `--from N` / `--to N` bound the tick range.
use mc_tick::{Pos, Simulation, Structure};
use mc_tick_trace::{EventKind, Trace};
use std::collections::{BTreeMap, BTreeSet};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(structure_arg), Some(golden_arg)) = (args.first(), args.get(1)) else {
        eprintln!(
            "usage: diff_vanilla <structure.snbt> <golden.json> [--settle] \
             [--use x,y,z@T ...] [--break x,y,z@T] [--region x,y,z..x,y,z] \
             [--from N] [--to N]"
        );
        std::process::exit(2);
    };

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let structure_path = resolve(root.join("tests/corpus/structures"), structure_arg);
    let golden_path = resolve(root.join("tests/traces"), golden_arg);

    let text = std::fs::read_to_string(&structure_path).expect("read structure");
    let structure = Structure::parse(&text).expect("parse structure");
    let golden = Trace::from_json(&std::fs::read_to_string(&golden_path).expect("read golden"))
        .expect("parse golden");

    let settle = args.iter().any(|a| a == "--settle");
    let uses = actions(&args, "--use");
    let breaks = actions(&args, "--break");
    let region = args
        .iter()
        .position(|a| a == "--region")
        .and_then(|i| args.get(i + 1))
        .map(|spec| parse_region(spec));
    let from: u64 = flag(&args, "--from").map_or(0, |v| v.parse().expect("--from N"));
    let to: u64 = flag(&args, "--to").map_or(u64::MAX, |v| v.parse().expect("--to N"));

    // ── run the engine the way the conformance harness does ────────────────
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;
    }
    if let Some(report) = sim.unknown_report() {
        eprintln!("warning: {report}");
    }
    for pos in &structure.block_entities {
        sim.mark_block_entity(*pos);
    }
    for (pos, stacks) in &structure.inventories {
        let entry = structure.blocks.iter().find(|(p, _)| p == pos).map(|(_, e)| *e);
        let Some(entry) = entry else { continue };
        let name = structure.palette[entry].split('[').next().unwrap_or_default();
        if let Some(slots) = mc_tick::vanilla::container_slots(name) {
            sim.set_inventory(*pos, mc_tick::Inventory { slots, stacks: stacks.clone() });
        }
    }
    let air = sim.registry().get("minecraft:air").unwrap_or(mc_tick::StateId::AIR);
    {
        let order = structure.placement_order(
            mc_tick::vanilla::is_collision_full_cube,
            mc_tick::vanilla::has_dynamic_shape,
        );
        // onPlace runs whatever the placement flags; the settle pass does not.
        sim.place_on_place(&order);
        if settle {
            sim.settle_with_order(&order);
        }
    }
    sim.record();
    let horizon = golden.ticks.last().map(|t| t.tick + 1).unwrap_or(0).max(to.min(4096));
    for tick in 0..horizon {
        for (pos, at) in &uses {
            if *at == tick {
                sim.use_block(*pos);
            }
        }
        for (pos, at) in &breaks {
            if *at == tick {
                sim.place_block(*pos, air);
            }
        }
        sim.step();
    }

    // ── bucket both sides by tick ──────────────────────────────────────────
    let keep = |pos: (i32, i32, i32)| match region {
        None => true,
        Some((lo, hi)) => {
            pos.0 >= lo.0 && pos.0 <= hi.0 && pos.1 >= lo.1 && pos.1 <= hi.1 && pos.2 >= lo.2
                && pos.2 <= hi.2
        }
    };
    let mut vanilla: BTreeMap<u64, BTreeSet<String>> = BTreeMap::new();
    for record in &golden.ticks {
        for event in &record.events {
            if let EventKind::BlockChanged { pos, to, .. } = &event.kind {
                if keep((pos.0, pos.1, pos.2)) {
                    vanilla
                        .entry(record.tick)
                        .or_default()
                        .insert(format!("({},{},{}) {}", pos.0, pos.1, pos.2, to));
                }
            }
        }
    }
    let mut engine: BTreeMap<u64, BTreeSet<String>> = BTreeMap::new();
    // Net per (tick, pos), matching the snapshot capture's view.
    let mut net: BTreeMap<(u64, Pos), (mc_tick::StateId, mc_tick::StateId)> = BTreeMap::new();
    for change in sim.recorded() {
        net.entry((change.tick, change.pos))
            .and_modify(|entry| entry.1 = change.to)
            .or_insert((change.from, change.to));
    }
    for ((tick, pos), (from, to)) in net {
        if from == to || !keep((pos.x, pos.y, pos.z)) {
            continue;
        }
        let name = sim.registry().descriptor(to).unwrap_or("?");
        engine
            .entry(tick)
            .or_default()
            .insert(format!("({},{},{}) {}", pos.x, pos.y, pos.z, name));
    }

    // ── report ─────────────────────────────────────────────────────────────
    let mut first_divergence = None;
    let ticks: BTreeSet<u64> = vanilla.keys().chain(engine.keys()).copied().collect();
    for tick in ticks {
        if tick < from || tick > to {
            continue;
        }
        let empty = BTreeSet::new();
        let (v, e) = (
            vanilla.get(&tick).unwrap_or(&empty),
            engine.get(&tick).unwrap_or(&empty),
        );
        if v == e {
            println!("tick {tick:>4}: MATCH   ({} events)", v.len());
            continue;
        }
        first_divergence.get_or_insert(tick);
        println!(
            "tick {tick:>4}: DIFFER  vanilla={} engine={}  (+{} vanilla-only, +{} engine-only)",
            v.len(),
            e.len(),
            v.difference(e).count(),
            e.difference(v).count()
        );
        for only in v.difference(e) {
            println!("            V  {only}");
        }
        for only in e.difference(v) {
            println!("            E  {only}");
        }
    }
    match first_divergence {
        None => println!("\nidentical across {} ticks", vanilla.len().max(engine.len())),
        Some(tick) => {
            println!("\nfirst divergence: tick {tick}");
            std::process::exit(1);
        }
    }
}

fn resolve(dir: std::path::PathBuf, arg: &str) -> std::path::PathBuf {
    let direct = std::path::PathBuf::from(arg);
    if direct.exists() {
        direct
    } else {
        dir.join(arg)
    }
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn actions(args: &[String], name: &str) -> Vec<(Pos, u64)> {
    args.iter()
        .enumerate()
        .filter(|(_, a)| a.as_str() == name)
        .filter_map(|(i, _)| args.get(i + 1))
        .map(|v| {
            let (xyz, tick) = v.split_once('@').expect("x,y,z@T");
            let p: Vec<i32> = xyz.split(',').map(|c| c.parse().expect("coord")).collect();
            (Pos::new(p[0], p[1], p[2]), tick.parse().expect("tick"))
        })
        .collect()
}

fn parse_region(spec: &str) -> ((i32, i32, i32), (i32, i32, i32)) {
    let (lo, hi) = spec.split_once("..").expect("--region x,y,z..x,y,z");
    let parse = |s: &str| {
        let v: Vec<i32> = s.split(',').map(|c| c.parse().expect("coord")).collect();
        (v[0], v[1], v[2])
    };
    (parse(lo), parse(hi))
}
