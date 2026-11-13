//! Snapshot conformance against a playerless Minecraft 1.21.11 Fabric server.
//! See traces/hipster_door_1_21_11.md for capture provenance and scope.

use mc_test::{build_sim, SettleMode};
use mc_tick::{Pos, Simulation, Structure};
use mc_tick_trace::{EventKind, Trace};
use std::collections::BTreeMap;

fn simulation() -> Simulation {
    let structure = Structure::parse(include_str!("corpus/structures/hipster_door.snbt"))
        .expect("saved Hipster door");
    let mut sim = build_sim(
        &structure,
        Pos::new(0, 0, 0),
        SettleMode::InWorld,
        &[],
        &[],
        Some(2730),
        "hipster_door",
    );
    sim.set_rng_seed(0);
    sim
}

fn snapshot(sim: &Simulation) -> BTreeMap<[i32; 3], String> {
    sim.world()
        .iter_non_air()
        .map(|(pos, state)| {
            (
                [pos.x, pos.y, pos.z],
                sim.registry().descriptor(state).unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn saved_hipster_door_stays_idle_without_placement_updates() {
    let mut sim = simulation();
    let initial = snapshot(&sim);
    sim.run(160);
    assert_eq!(sim.tick_count(), 160);
    assert!(
        sim.recorded().is_empty(),
        "untouched saved machine must stay idle"
    );
    assert_eq!(snapshot(&sim), initial);
    assert!(sim.unknown_report().is_none());
}

#[test]
fn hipster_door_matches_every_vanilla_tick_and_repeats_the_cycle() {
    let reference = Trace::from_json(include_str!("traces/hipster_door_1_21_11.json"))
        .expect("Minecraft capture");
    assert_eq!(reference.mc_version, "1.21.11");
    assert_eq!(reference.ticks.len(), 320);
    let mut sim = simulation();
    let initial = snapshot(&sim);
    // A separate 640-tick vanilla run verified that the second cycle repeats
    // all first-cycle tick-end states. Keep the golden once and replay it twice.
    for cycle in 0..2 {
        let mut expected = initial.clone();
        for (tick, row) in reference.ticks.iter().enumerate() {
            assert_eq!(
                row.tick, tick as u64,
                "capture must have every tick in order"
            );
            if tick == 10 || tick == 170 {
                sim.use_block(Pos::new(12, 7, 13));
            }
            sim.step();
            assert_eq!(sim.tick_count(), (cycle * 320 + tick + 1) as u64);
            for event in &row.events {
                let EventKind::BlockChanged { pos, from, to } = &event.kind else {
                    panic!("this reference records block states only");
                };
                let pos = [pos.0, pos.1, pos.2];
                assert_eq!(
                    expected
                        .get(&pos)
                        .map(String::as_str)
                        .unwrap_or("minecraft:air"),
                    from,
                    "reference transition continuity at tick {tick}, {pos:?}"
                );
                if to == "minecraft:air" {
                    expected.remove(&pos);
                } else {
                    expected.insert(pos, to.clone());
                }
            }
            let actual = snapshot(&sim);
            let differences: Vec<_> = expected
                .keys()
                .chain(actual.keys())
                .filter(|pos| expected.get(*pos) != actual.get(*pos))
                .take(8)
                .map(|pos| (pos, expected.get(pos), actual.get(pos)))
                .collect();
            assert!(
                differences.is_empty(),
                "cycle {cycle}, tick {tick}: (position, vanilla, engine) {differences:?}"
            );
        }
        assert_eq!(
            snapshot(&sim),
            initial,
            "cycle {cycle} must reopen completely"
        );
        assert!(sim.unknown_report().is_none());
    }
}
