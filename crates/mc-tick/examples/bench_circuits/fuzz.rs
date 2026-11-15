//! Untimed deterministic mutation sequences; compare JSON across binaries.
use super::*;
use mc_tick::inventory::{Inventory, ItemStack};

fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn snapshot(hash: &mut blake3::Hasher, sim: &Simulation, positions: &[Pos], cursors: &mut [usize; 4]) {
    hash_tick(hash, sim);
    let mut outputs: Vec<_> = sim.comparator_outputs().iter().collect();
    outputs.sort_by_key(|(p, _)| (p.x, p.y, p.z));
    hash.update(format!("{:?}{:?}{:?}{:?}{:?}{:?}", sim.world().bounds(), outputs,
        sim.moving_blocks(), sim.item_entities(), sim.minecarts(), sim.entity_bodies()).as_bytes());
    for &pos in positions {
        hash.update(format!("{:?}", sim.inventory(pos)).as_bytes());
    }
    // Include every ordered record exactly once, even when restore rewinds time.
    hash.update(format!("{:?}{:?}{:?}{:?}", &sim.recorded()[cursors[0]..],
        &sim.recorded_updates()[cursors[1]..], &sim.recorded_inventory()[cursors[2]..],
        &sim.recorded_entities()[cursors[3]..]).as_bytes());
    *cursors = [sim.recorded().len(), sim.recorded_updates().len(),
        sim.recorded_inventory().len(), sim.recorded_entities().len()];
}

fn sequence(sim: &mut Simulation, case: &str, seed: u64) -> (String, [usize; 10]) {
    let mut rng = seed + 0x9e3779b97f4a7c15;
    sim.set_rng_seed(seed as i64);
    sim.record(); sim.record_updates(true);
    let cells: Vec<_> = sim.world().iter_non_air().collect();
    let positions: Vec<_> = cells.iter().map(|&(p, _)| p).collect();
    let levers: Vec<_> = cells.iter().filter_map(|&(p, s)|
        sim.registry().descriptor(s).unwrap().starts_with("minecraft:lever[").then_some(p)).collect();
    let comparators: Vec<_> = cells.iter().filter_map(|&(p, s)|
        sim.registry().descriptor(s).unwrap().starts_with("minecraft:comparator[").then_some(p)).collect();
    let containers: Vec<_> = positions.iter().copied().filter(|&p| sim.inventory(p).is_some()).collect();
    let redstone = sim.registry().get("minecraft:redstone_block").unwrap();
    let original_bounds = sim.world().bounds();
    let mut saved = sim.checkpoint();
    let mut hash = blake3::Hasher::new();
    let mut cursors = [0; 4];
    let mut counts = [0; 10];
    for op in 0..96 {
        let value = next(&mut rng);
        let action = (value % 10) as usize;
        counts[action] += 1;
        match action {
            0 if !levers.is_empty() => { sim.use_block(levers[(value as usize / 10) % levers.len()]); }
            1 if !containers.is_empty() => {
                let pos = containers[(value as usize / 10) % containers.len()];
                let slots = sim.inventory(pos).map_or(27, |i| i.slots);
                let mut inventory = Inventory::empty(slots);
                let count = ((value >> 24) % 65) as u8;
                if count != 0 { inventory.stacks.push(ItemStack { slot: 0,
                    id: "minecraft:stone".into(), count, contents: None }); }
                sim.set_inventory(pos, inventory);
            }
            2 if !comparators.is_empty() => {
                // No World write: includes zero, >15, and the entire u8 domain.
                sim.set_comparator_output(comparators[(value as usize / 10) % comparators.len()],
                    (value >> 32) as u8);
            }
            3 => {
                // A safe exterior stimulus still causes ordinary neighbour updates.
                let pos = Pos::new(original_bounds.min.x - 1, original_bounds.min.y + 1,
                    original_bounds.min.z + 1 + (value % 3) as i32);
                sim.place_block(pos, if value & 0x100 == 0 { StateId::AIR } else { redstone });
            }
            4 => { saved = sim.checkpoint(); }
            5 => { sim.restore(&saved.clone()); sim.set_rng_seed((seed ^ op) as i64); }
            6 => {
                // Geometry changes without an active update; tests address/cache invalidation.
                let pos = Pos::new(original_bounds.min.x - 17, original_bounds.min.y,
                    original_bounds.min.z - 17);
                sim.world_mut().set(pos, StateId::AIR);
            }
            7 => {
                // Restore a block's original descriptor through the public update path.
                let (pos, state) = cells[(value as usize / 10) % cells.len()];
                sim.place_block(pos, state);
            }
            8 => {
                // Keep each fixture's intended stimulus in the random sequence.
                step(sim, case, if op % 2 == 0 { 10 } else { 170 });
            }
            _ => {}
        }
        for _ in 0..1 + (value >> 48) % 3 { sim.step(); }
        assert!(sim.unknown_report().is_none(), "{case}, seed {seed}, operation {op}");
        hash.update(&[action as u8]);
        snapshot(&mut hash, sim, &positions, &mut cursors);
    }
    (hash.finalize().to_hex().to_string(), counts)
}

pub(super) fn verify(seeds: u64) {
    assert!(seeds > 0);
    for case in ["hipster", "flyer", "comparator_drain", "item_into_hopper",
        "item_sorter", "hopper_storagecart_interaction", "trapdoor2x3"] {
        let mut sim = prepare(case);
        let initial = sim.checkpoint();
        for seed in 0..seeds {
            sim.restore(&initial);
            let (fingerprint, counts) = sequence(&mut sim, case, seed);
            let final_state = sim.checkpoint();
            sim.restore(&initial);
            let replay = sequence(&mut sim, case, seed);
            assert_eq!(fingerprint, replay.0, "ordered replay: {case}, seed {seed}");
            assert_eq!(final_state, sim.checkpoint(), "full checkpoint replay: {case}, seed {seed}");
            println!("{}", json!({"case":case,"seed":seed,"operations":96,
                "actions":counts,"fingerprint":fingerprint}));
        }
    }
}
