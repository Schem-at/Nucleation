//! Explain what powers a position, optionally after running the world for a
//! while with actuations — the offline counterpart to the capture tool's
//! `--probe`, which costs a server boot per question.
//!
//!     cargo run -p mc-tick --example probe_power -- <file.snbt> x,y,z
//!     cargo run -p mc-tick --example probe_power -- <file.snbt> 5,2,1 \
//!         --quiet --use 6,4,0@10 --at 10
//!
//! `--quiet` places the way `knownShape` does (onPlace only, no update pass),
//! matching a `--known-shape` capture. `--at N` runs N ticks before probing, so
//! the answer is about the world the door is actually in.
use mc_tick::{Pos, Simulation, Structure};

fn coords(text: &str) -> Pos {
    let parts: Vec<i32> = text.split(',').map(|p| p.parse().expect("x,y,z")).collect();
    Pos::new(parts[0], parts[1], parts[2])
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.first().expect("structure").clone();
    let target = coords(args.get(1).expect("x,y,z"));
    let quiet = args.iter().any(|a| a == "--quiet");
    let at: u64 = args
        .iter()
        .position(|a| a == "--at")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    // `--use x,y,z@tick`, repeatable.
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

    let text = std::fs::read_to_string(&path).expect("read");
    let structure = Structure::parse(&text).expect("parse");
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    mc_tick::intern_companions(sim.registry_mut());
    let rules = {
        let mut table = std::mem::take(sim.behaviours_mut());
        let rules = mc_tick::register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;
        rules
    };
    for pos in &structure.block_entities {
        sim.mark_block_entity(*pos);
    }
    for (pos, strength) in &structure.comparator_outputs {
        sim.set_comparator_output(*pos, *strength);
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    if !quiet {
        sim.settle_with_order(&order);
    }
    for tick in 0..at {
        for (pos, when) in &uses {
            if *when == tick {
                sim.use_block(*pos);
            }
        }
        sim.step();
    }

    let describe = |p: Pos| {
        sim.registry()
            .descriptor(sim.world().get(p))
            .unwrap_or("minecraft:air")
            .to_string()
    };
    println!("tick {}: {target:?} = {}", sim.tick_count(), describe(target));
    // The direct neighbours, then the quasi-connectivity ring: a piston reads
    // the block above it and *that* block's neighbours, so a build can power one
    // from a block touching it nowhere.
    for (label, at) in [("", target), ("above ", target.offset(mc_tick::pos::Dir::Up))] {
        println!("  {label}{at:?} {}", describe(at));
        for dir in mc_tick::pos::ALL_DIRS {
            let n = at.offset(dir);
            println!(
                "    {dir:?} {} => {}",
                describe(n),
                rules.explain_power(sim.world(), n, dir.opposite())
            );
        }
    }
}
