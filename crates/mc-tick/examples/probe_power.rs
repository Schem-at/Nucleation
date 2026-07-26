//! Explain a dust cell's block signal after the placement settle.
//!
//!     cargo run -p mc-tick --example probe_power -- <file.snbt> x y z
use mc_tick::{Pos, Simulation, Structure};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("structure");
    let coords: Vec<i32> = args.map(|a| a.parse().expect("coord")).collect();
    let target = Pos::new(coords[0], coords[1], coords[2]);

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
    let order: Vec<Pos> = structure.blocks.iter().map(|(pos, _)| *pos).collect();
    sim.settle_with_order(&order);

    let describe = |p: Pos| {
        sim.registry()
            .descriptor(sim.world().get(p))
            .unwrap_or("minecraft:air")
            .to_string()
    };
    println!("{target:?} = {}", describe(target));
    for dir in mc_tick::pos::ALL_DIRS {
        let n = target.offset(dir);
        println!(
            "  {dir:?} {} \n      {}",
            describe(n),
            rules.explain_power(sim.world(), n, dir.opposite())
        );
    }
}
