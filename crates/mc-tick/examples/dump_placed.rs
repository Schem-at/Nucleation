//! Load a structure, run the placement settle, and print the resulting world —
//! the engine's counterpart to `TraceCapture --dump-placed`.
//!
//!     cargo run -p mc-tick --example dump_placed -- <file.snbt>
use mc_tick::{Pos, Simulation, Structure};

fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_placed <file.snbt>");
    let text = std::fs::read_to_string(&path).expect("read structure");
    let structure = Structure::parse(&text).expect("parse structure");
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
    let order: Vec<Pos> = structure.blocks.iter().map(|(pos, _)| *pos).collect();
    sim.settle_with_order(&order);

    let mut rows: Vec<(i32, i32, i32, String)> = sim
        .world()
        .iter_non_air()
        .map(|(pos, state)| {
            (
                pos.x,
                pos.y,
                pos.z,
                sim.registry().descriptor(state).unwrap_or("?").to_string(),
            )
        })
        .collect();
    rows.sort_by_key(|(x, y, z, _)| (*y, *z, *x));
    let mut out = String::new();
    for (x, y, z, name) in rows {
        out.push_str(&format!("{x} {y} {z} {name}\n"));
    }
    print!("{out}");
}
