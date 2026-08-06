//! TEMPORARY diagnostic probe for lithium hopper-family reds — not a
//! regression test; delete when the cluster is green.
//!
//! ```sh
//! MC_PROBE=item_sorter MC_PROBE_TICKS=400 \
//!   cargo test -p nucleation-cli --test probe_hopper -- --ignored --nocapture
//! ```

use mc_test::mc_tick::{Pos, Structure};
use nucleation::formats::gametest::to_gametest_snbt;

#[test]
#[ignore = "diagnostic probe, run by hand with --ignored"]
fn probe_command_nbt_shape() {
    let schematic = if let Ok(file) = std::env::var("MC_PROBE_FILE") {
        let bytes = std::fs::read(&file).expect("readable");
        let manager = nucleation::formats::manager::get_manager();
        let manager = manager.lock().expect("manager");
        manager.read(&bytes).expect("imports")
    } else {
        let text = std::fs::read_to_string(
            "../../tests/corpus/lithium/gametest/structure/tnt_above_world.snbt",
        )
        .expect("fetch the corpus");
        nucleation::formats::structure_snbt::from_structure_snbt(text.as_bytes()).expect("imports")
    };
    for be in schematic.get_block_entities_as_list() {
        if be.id.contains("command_block") {
            eprintln!("stored nbt value for Command: {:?}", be.nbt.get("Command"));
        }
    }
    panic!("output above");
}

#[test]
#[ignore = "diagnostic probe, run by hand with --ignored"]
fn probe_lithium_structure() {
    let name = std::env::var("MC_PROBE").unwrap_or_else(|_| "hopper_transfer_speed".to_string());
    let ticks: u64 = std::env::var("MC_PROBE_TICKS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(200);
    let schematic = if let Ok(file) = std::env::var("MC_PROBE_FILE") {
        let bytes = std::fs::read(&file).expect("readable");
        let manager = nucleation::formats::manager::get_manager();
        let manager = manager.lock().expect("manager");
        manager.read(&bytes).expect("imports")
    } else {
        let path = format!("../../tests/corpus/lithium/gametest/structure/{name}.snbt");
        let text = std::fs::read_to_string(&path).expect("fetch the lithium corpus first");
        nucleation::formats::structure_snbt::from_structure_snbt(text.as_bytes()).expect("imports")
    };
    let snbt = to_gametest_snbt(&schematic);
    if let Ok(dump) = std::env::var("MC_PROBE_DUMP") {
        std::fs::write(&dump, &snbt).expect("dump written");
    }
    let structure = Structure::parse(&snbt).expect("engine parses");

    // The start block, straight from the structure.
    let start = structure
        .blocks
        .iter()
        .find(|(_, entry)| structure.palette[*entry].contains("mode=start"))
        .map(|(pos, _)| *pos)
        .expect("a start test_block");
    eprintln!("start block at {start:?}");

    // Same inert probe as `nucleation port` would discover.
    let mut inert: Vec<String> = Vec::new();
    let mut sim = loop {
        match mc_test::try_build_sim(
            &structure,
            Pos::new(0, 0, 0),
            mc_test::SettleMode::Placement,
            &["minecraft:redstone_block".to_string()],
            &inert,
            None,
            "probe",
        ) {
            Ok(sim) => break sim,
            Err(report) => {
                let list = report.rsplit("simulated as nothing: ").next().unwrap_or("");
                let before = inert.len();
                for descriptor in list.split(", ") {
                    let name = descriptor.split('[').next().unwrap_or(descriptor).trim();
                    if name.starts_with("minecraft:") && !inert.iter().any(|n| n == name) {
                        inert.push(name.to_string());
                    }
                }
                assert!(inert.len() > before, "probe stuck: {report}");
            }
        }
    };
    eprintln!("inert: {inert:?}");
    // Parity with the synthesized specs: seed 0, lithium's randomTickSpeed.
    sim.set_rng_seed(0);
    sim.set_random_ticks(3);

    if std::env::var("MC_PROBE_RECORD_SETUP").is_ok() {
        sim.record();
    }
    for _ in 0..10 {
        sim.step();
    }
    sim.record();
    let rb = sim
        .registry()
        .get("minecraft:redstone_block")
        .expect("interned");
    // Constant start signal — the wiki-documented test_block behaviour.
    sim.place_block(start, rb);
    for _ in 0..ticks {
        sim.step();
    }

    eprintln!(
        "--- recorded inventory changes ({}):",
        sim.recorded_inventory().len()
    );
    for change in sim.recorded_inventory().iter().take(80) {
        eprintln!("  {change:?}");
    }
    if let Ok(cells) = std::env::var("MC_PROBE_CELLS") {
        eprintln!("--- final states of interest:");
        for triple in cells.split(';') {
            let coords: Vec<i32> = triple.split(',').filter_map(|c| c.parse().ok()).collect();
            if let [x, y, z] = coords[..] {
                let pos = Pos::new(x, y, z);
                let descriptor = sim
                    .registry()
                    .descriptor(sim.world().get(pos))
                    .unwrap_or("?");
                eprintln!("  ({x},{y},{z}) {descriptor}");
            }
        }
    }
    if std::env::var("MC_PROBE_TRACE").is_ok() {
        eprintln!("--- every recorded block change:");
        for change in sim.recorded() {
            let from = sim.registry().descriptor(change.from).unwrap_or("?");
            let to = sim.registry().descriptor(change.to).unwrap_or("?");
            eprintln!(
                "TRACE t{} [{},{},{}] {} -> {}",
                change.tick, change.pos.x, change.pos.y, change.pos.z, from, to
            );
        }
    }
    if let Ok(watch) = std::env::var("MC_PROBE_WATCH") {
        eprintln!("--- block changes at watched cells:");
        let cells: Vec<Pos> = watch
            .split(';')
            .filter_map(|triple| {
                let coords: Vec<i32> = triple.split(',').filter_map(|c| c.parse().ok()).collect();
                if let [x, y, z] = coords[..] {
                    Some(Pos::new(x, y, z))
                } else {
                    None
                }
            })
            .collect();
        for change in sim.recorded() {
            if cells.contains(&change.pos) {
                let from = sim.registry().descriptor(change.from).unwrap_or("?");
                let to = sim.registry().descriptor(change.to).unwrap_or("?");
                eprintln!("  t{} {:?} {} -> {}", change.tick, change.pos, from, to);
            }
        }
    }
    eprintln!("--- minecarts at the end:");
    for cart in sim.minecarts() {
        eprintln!("  {} removed={} @ {:?}", cart.kind, cart.removed, cart.pos);
    }
    eprintln!("--- entity bodies at the end:");
    for body in sim.entity_bodies() {
        eprintln!("  {} @ {:?}..{:?}", body.kind, body.min, body.max);
    }
    eprintln!("--- item entities at the end:");
    for entity in sim.item_entities() {
        if !entity.removed {
            eprintln!("  {}x{} @ {:?}", entity.item.1, entity.item.0, entity.pos);
        }
    }
    eprintln!("--- test blocks at the end:");
    for (pos, entry) in &structure.blocks {
        if structure.palette[*entry].starts_with("minecraft:test_block") {
            let now = sim
                .registry()
                .descriptor(sim.world().get(*pos))
                .unwrap_or("?");
            eprintln!("  {pos:?} was {} now {now}", structure.palette[*entry]);
        }
    }
    eprintln!("--- final container contents:");
    for (pos, entry) in &structure.blocks {
        let descriptor = &structure.palette[*entry];
        let name = descriptor.split('[').next().unwrap_or(descriptor);
        if mc_test::mc_tick::vanilla::container_slots(name).is_some() {
            if let Some(inv) = sim.inventory(*pos) {
                if !inv.stacks.is_empty() {
                    let stacks: Vec<String> = inv
                        .stacks
                        .iter()
                        .map(|s| format!("{}x{}@{}", s.count, s.id, s.slot))
                        .collect();
                    eprintln!("  {pos:?} {name}: {}", stacks.join(", "));
                }
            }
        }
    }
    panic!("probe output above — always red so the eprintln survives");
}
