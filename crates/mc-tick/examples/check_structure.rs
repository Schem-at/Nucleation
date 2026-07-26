//! Load a structure and report every block state the engine cannot simulate.
//!
//!     cargo run -p mc-tick --example check_structure -- <file.snbt>...
use mc_tick::{Pos, Simulation, Structure};

fn main() {
    let mut failed = false;
    for path in std::env::args().skip(1) {
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
        match sim.unknown_report() {
            None => println!("OK   {path}"),
            Some(report) => {
                failed = true;
                println!("GAPS {path}\n     {report}");
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
}
