//! Simulate a structure with lever clicks and summarise what moved.
//!
//!     cargo run -p mc-tick --example sim_summary -- <file.snbt> <ticks> [x,y,z@T ...]
use mc_tick::{Pos, Simulation, Structure};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("structure");
    let ticks: u64 = args.next().expect("ticks").parse().expect("ticks");
    let clicks: Vec<(Pos, u64)> = args
        .map(|a| {
            let (xyz, t) = a.split_once('@').expect("x,y,z@T");
            let p: Vec<i32> = xyz.split(',').map(|c| c.parse().expect("coord")).collect();
            (Pos::new(p[0], p[1], p[2]), t.parse().expect("tick"))
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
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;
    }
    for pos in &structure.block_entities {
        sim.mark_block_entity(*pos);
    }
    for (pos, stacks) in &structure.inventories {
        let entry = structure
            .blocks
            .iter()
            .find(|(p, _)| p == pos)
            .map(|(_, e)| *e)
            .unwrap();
        let name = structure.palette[entry]
            .split('[')
            .next()
            .unwrap_or_default();
        if let Some(slots) = mc_tick::vanilla::container_slots(name) {
            sim.set_inventory(
                *pos,
                mc_tick::Inventory {
                    slots,
                    stacks: stacks.clone(),
                    blocked_slots: structure.blocked_slots_at(*pos),
                },
            );
        }
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.settle_with_order(&order);
    sim.record();
    for tick in 0..ticks {
        for (pos, at) in &clicks {
            if *at == tick {
                sim.use_block(*pos);
            }
        }
        sim.step();
    }

    let air = mc_tick::StateId::AIR;
    let mut per_tick: std::collections::BTreeMap<u64, (usize, usize)> = Default::default();
    for change in sim.recorded() {
        let entry = per_tick.entry(change.tick).or_default();
        entry.0 += 1;
        if change.from == air && change.to != air {
            entry.1 += 1;
        }
    }
    for (tick, (total, fills)) in per_tick {
        println!("tick {tick}: {total} changes, {fills} air->block");
    }
}
