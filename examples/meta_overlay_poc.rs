//! Render a machine coloured by what each block *does* — headlessly.
//!
//! A proof of concept for the meta-structure overlay that needs no browser, no
//! GA app and no dev server. `machine_graph::analyse` says which cells are
//! engine, payload, kicker and dead weight; each cell is written out twice, once
//! as the real build and once recoloured by role, so the existing renderer can
//! draw both.
//!
//! ```text
//! cargo run --release --example meta_overlay_poc --features bridge,mc-tick -- <out-dir>
//! ```
//!
//! The pair is the argument: bolt cargo onto the canonical engine — including
//! SLIME cargo, which joins the adhesion group — and the engine must not move.
//! A classifier that merely coloured "everything connected" would swallow it.
use std::collections::HashMap;
use std::path::PathBuf;

use mc_tick::machine_graph::{analyse, MachineGraph};
use mc_tick::{Pos, Simulation, Structure};
use nucleation::UniversalSchematic;

/// Role colours, chosen the way the app chose them.
///
/// Three hues and one hueless class, because four categorical hues cannot pass
/// an all-pairs colour-vision check — the same reason the door validator draws
/// its fourth overlay class as a wireframe cage rather than inventing a fifth
/// colour. Dead weight is the class with nothing to say, so it is the one that
/// gives up its hue: glass reads as "present but inert" and you can see past it.
///
/// Payload is purple rather than blue because the renderer's sky is pale blue
/// and a blue payload sat almost on top of it — legible on a monitor, gone in
/// a screenshot someone shrinks.
const ENGINE: &str = "minecraft:lime_concrete";
const PAYLOAD: &str = "minecraft:purple_concrete";
const KICKER: &str = "minecraft:orange_concrete";
const DEAD: &str = "minecraft:glass";

fn main() {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&out).expect("create the output directory");

    let corpus =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("crates/mc-tick/tests/corpus/structures");
    let read = |n: &str| std::fs::read_to_string(corpus.join(n)).ok();

    let canonical = read("flying_machine.snbt").expect("the canonical engine must be present");
    let mut jobs: Vec<(String, String)> = vec![("canonical".into(), canonical.clone())];
    if let Some(east) = read("flying_machine_east.snbt") {
        jobs.push(("canonical_east".into(), east));
    }
    jobs.push(("canonical_with_cargo".into(), with_cargo(&canonical)));

    for (name, snbt) in jobs {
        let structure = match Structure::parse(&snbt) {
            Ok(s) => s,
            Err(e) => {
                println!("{name}: SKIPPED — {e:?}");
                continue;
            }
        };
        let graph = graph_of(&structure);

        write(&out, &name, "plain", plain(&structure));
        write(&out, &name, "meta", painted(&structure, &graph));

        let engine_cells: usize = graph.engines.iter().map(|e| e.cells.len()).sum();
        println!(
            "  {name}: engine {} · payload {} · kicker {} · dead {}{}",
            engine_cells,
            graph.payload.len(),
            graph.kickers.len(),
            graph.dead_weight.len(),
            if graph.rejected() { " · REJECTED" } else { "" },
        );
    }
}

/// Bolt four blocks onto the canonical engine, two of them slime.
///
/// The slime is the whole point: it joins the adhesion group, so anything keying
/// on connectivity would pull it into the engine. It has to come back as payload
/// with the engine unchanged. Appended as a second row directly above the bar.
fn with_cargo(snbt: &str) -> String {
    let slime = snbt
        .find("minecraft:slime_block")
        .map(|_| 0usize)
        .expect("the canonical machine is built from slime");
    let extra = (0..4)
        .map(|i| format!("{{pos: [{i}, 1, 0], state: {slime}}}"))
        .collect::<Vec<_>>()
        .join(",\n    ");
    snbt.replacen("  blocks: [\n", &format!("  blocks: [\n    {extra},\n"), 1)
}

/// The same world-building the crate's own tests use, so this analyses exactly
/// what a simulation would have seen.
fn graph_of(structure: &Structure) -> MachineGraph {
    let mut sim = Simulation::new(structure.bounds(8));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    mc_tick::intern_companions(sim.registry_mut());
    let rules = {
        let mut table = std::mem::take(sim.behaviours_mut());
        let rules = mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
        *sim.behaviours_mut() = table;
        rules
    };
    analyse(sim.world(), sim.registry(), &rules)
}

/// The build as authored.
fn plain(structure: &Structure) -> UniversalSchematic {
    let mut schem = UniversalSchematic::new("machine".into());
    for (pos, index) in &structure.blocks {
        if let Some(state) = structure.palette.get(*index) {
            schem.set_block_from_string(pos.x, pos.y, pos.z, state).ok();
        }
    }
    schem
}

/// Every cell rewritten to the colour of the role it plays.
fn painted(structure: &Structure, graph: &MachineGraph) -> UniversalSchematic {
    // Precedence engine > kicker > payload > dead, the order the app uses. A
    // kicker usually sits INSIDE the push closure — it is bolted to the machine
    // it starts — so painting payload over it under-reports kickers, which is
    // exactly the bug the 2-D panel had.
    let mut role: HashMap<(i32, i32, i32), &str> = HashMap::new();
    for c in &graph.dead_weight {
        role.insert((c.x, c.y, c.z), DEAD);
    }
    for c in &graph.payload {
        role.insert((c.x, c.y, c.z), PAYLOAD);
    }
    for id in &graph.kickers {
        if let Some(d) = graph.devices.get(*id) {
            role.insert((d.pos.x, d.pos.y, d.pos.z), KICKER);
        }
    }
    for e in &graph.engines {
        for c in &e.cells {
            role.insert((c.x, c.y, c.z), ENGINE);
        }
    }

    let mut schem = UniversalSchematic::new("machine-meta".into());
    for (pos, _) in &structure.blocks {
        let block = role.get(&(pos.x, pos.y, pos.z)).copied().unwrap_or(DEAD);
        schem.set_block_from_string(pos.x, pos.y, pos.z, block).ok();
    }
    schem
}

fn write(out: &std::path::Path, name: &str, suffix: &str, schem: UniversalSchematic) {
    let path = out.join(format!("{name}_{suffix}.litematic"));
    match nucleation::formats::litematic::to_litematic(&schem) {
        Ok(bytes) => {
            std::fs::write(&path, bytes).expect("write the litematic");
            println!("{}", path.display());
        }
        Err(e) => println!("{}: FAILED — {e:?}", path.display()),
    }
}
