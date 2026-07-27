//! Print whether a door's opening is sealed, tick by tick — looking *through*
//! it at every depth, which is the only view that answers the question.
use mc_tick::{Pos, Simulation, Structure};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let file = args[0].clone();
    let uses: Vec<(Pos, u64)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--use")
        .filter_map(|(i, _)| args.get(i + 1))
        .map(|s| {
            let (p, t) = s.split_once('@').unwrap();
            let c: Vec<i32> = p.split(',').map(|v| v.parse().unwrap()).collect();
            (Pos::new(c[0], c[1], c[2]), t.parse().unwrap())
        })
        .collect();
    let window: Vec<i32> = args[args.iter().position(|a| a == "--window").unwrap() + 1]
        .split(',')
        .map(|v| v.parse().unwrap())
        .collect();
    let ticks: u64 = args[args.iter().position(|a| a == "--ticks").unwrap() + 1].parse().unwrap();

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/structures")
        .join(&file);
    let structure = Structure::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    let hash_origin = args
        .iter()
        .position(|a| a == "--origin")
        .and_then(|i| args.get(i + 1))
        .map(|v| {
            let c: Vec<i32> = v.split(',').map(|p| p.parse().unwrap()).collect();
            Pos::new(c[0], c[1], c[2])
        })
        .unwrap_or_default();
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
        if let Some(entry) = structure.blocks.iter().find(|(p, _)| p == pos).map(|(_, e)| *e) {
            let name = structure.palette[entry].split('[').next().unwrap_or_default();
            if let Some(slots) = mc_tick::vanilla::container_slots(name) {
                sim.set_inventory(*pos, mc_tick::Inventory { slots, stacks: stacks.clone() });
            }
        }
    }
    sim.record();

    let air = sim.registry().get("minecraft:air").unwrap_or(mc_tick::StateId::AIR);
    let (x0, x1, y0, y1, z0, z1) = (window[0], window[1], window[2], window[3], window[4], window[5]);
    for tick in 0..ticks {
        for (pos, at) in &uses {
            if *at == tick {
                sim.use_block(*pos);
            }
        }
        sim.step();
        if [11u64, 25, 39, 55, 69, 85].contains(&tick) {
            println!("  tick {tick}");
            for y in (y0..=y1).rev() {
                let row: String = (x0..=x1)
                    .map(|x| {
                        if (z0..=z1).any(|z| sim.world().get(Pos::new(x, y, z)) != air) {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect();
                println!("    y{y:<3} {row}");
            }
        }
    }
}
