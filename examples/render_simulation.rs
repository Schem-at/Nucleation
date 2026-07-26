//! Render a state of an mc-tick simulation as a PNG.
//!
//!     cargo run --release --example render_simulation --features rendering -- \
//!         <pack.zip|client.jar> <structure.snbt> <out.png> [--tick N] [--click x,y,z@T]
//!
//! Loads the structure the way the conformance tests do (quiet placement: no
//! settle), optionally clicks a block at a tick boundary, steps the simulation
//! to `--tick`, converts the world into a `UniversalSchematic`, and renders it
//! with nucleation's GPU renderer.
use mc_tick::{Pos, Simulation, Structure};
use nucleation::meshing::ResourcePackSource;
use nucleation::rendering::{GridConfig, RenderConfig};
use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(pack_path), Some(snbt_path), Some(out_path)) =
        (args.first(), args.get(1), args.get(2))
    else {
        eprintln!(
            "usage: render_simulation <pack> <structure.snbt> <out.png> [--tick N] [--click x,y,z@T]"
        );
        std::process::exit(2);
    };

    let tick: u64 = flag_value(&args, "--tick").map_or(0, |v| v.parse().expect("--tick N"));
    let click: Option<(Pos, u64)> = flag_value(&args, "--click").map(|v| {
        let (xyz, t) = v.split_once('@').expect("--click x,y,z@T");
        let p: Vec<i32> = xyz.split(',').map(|c| c.parse().expect("coord")).collect();
        (Pos::new(p[0], p[1], p[2]), t.parse().expect("tick"))
    });

    let sim = simulate(snbt_path, tick, click);

    let schem = world_to_schematic(&sim);
    let pack = ResourcePackSource::from_file(pack_path)?;
    let mut config = RenderConfig::isometric();
    config.width = 1280;
    config.height = 720;
    config.sphere_fit = true;
    config.grid = Some(GridConfig {
        fit_to_bounds: true,
        ..GridConfig::default()
    });

    schem.render_to_file(&pack, out_path, &config)?;
    println!("tick {tick} -> {out_path}");
    Ok(())
}

fn flag_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// Load quietly (no settle — knownShape placement) and run to `tick`.
fn simulate(snbt_path: &str, tick: u64, click: Option<(Pos, u64)>) -> Simulation {
    let text = std::fs::read_to_string(snbt_path).expect("read structure");
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
    if let Some(report) = sim.unknown_report() {
        panic!("unimplemented blocks: {report}");
    }

    for t in 0..tick {
        if let Some((pos, at)) = click {
            if at == t {
                sim.use_block(pos);
            }
        }
        sim.step();
    }
    sim
}

/// Convert the simulation's world into a schematic, descriptor by descriptor.
fn world_to_schematic(sim: &Simulation) -> UniversalSchematic {
    let mut schem = UniversalSchematic::new("mc-tick state".to_string());
    for (pos, state) in sim.world().iter_non_air() {
        let Some(descriptor) = sim.registry().descriptor(state) else {
            continue;
        };
        // moving_piston has no block model — the travelling block is a block
        // entity in vanilla. A still that hid them would show holes, so mark
        // them with a stand-in the mesher can draw.
        let descriptor = if descriptor.starts_with("minecraft:moving_piston") {
            "minecraft:tinted_glass"
        } else {
            descriptor
        };
        schem
            .set_block_from_string(pos.x, pos.y, pos.z, descriptor)
            .expect("set block");
    }
    schem
}
