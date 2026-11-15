//! Small vanilla 1.21.11 regressions isolated from the plot-door failures.
//! These replay captured state changes; expected edges do not come from mc-tick.

use mc_test::{build_sim, SettleMode};
use mc_tick::{Pos, Simulation, Structure};
use serde::Deserialize;
use std::collections::BTreeMap;

type Row = (i32, i32, i32, String);
type Change = (i32, i32, i32, String, String);

#[derive(Deserialize)]
struct Action {
    tick: u64,
    pos: [i32; 3],
}

#[derive(Deserialize)]
struct Changes {
    tick: u64,
    blocks: Vec<Change>,
}

#[derive(Deserialize)]
struct Run {
    case: String,
    origin: [i32; 3],
    capture_ticks: u64,
    actions: Vec<Action>,
    initial: Vec<Row>,
    changes: Vec<Changes>,
}

#[derive(Deserialize)]
struct Golden {
    mc_version: String,
    data_version: i32,
    runs: Vec<Run>,
}

fn snapshot(sim: &Simulation) -> BTreeMap<[i32; 3], String> {
    sim.world()
        .iter_non_air()
        .map(|(p, state)| {
            (
                [p.x, p.y, p.z],
                sim.registry().descriptor(state).unwrap().to_owned(),
            )
        })
        .collect()
}

fn check(case: &str, fixture: &str) {
    let golden: Golden =
        serde_json::from_str(include_str!("traces/flag2_observer_1_21_11.json")).unwrap();
    assert_eq!(golden.mc_version, "1.21.11");
    assert_eq!(golden.data_version, 4671);
    let run = golden.runs.iter().find(|r| r.case == case).unwrap();
    let structure = Structure::parse(fixture).unwrap();
    assert_eq!(structure.data_version, Some(4671));
    let [x, y, z] = run.origin;
    let mut sim = build_sim(
        &structure,
        Pos::new(x, y, z),
        SettleMode::InWorld,
        &[],
        &[],
        structure.data_version,
        case,
    );
    sim.set_rng_seed(0);
    let mut expected: BTreeMap<_, _> = run
        .initial
        .iter()
        .map(|(x, y, z, state)| ([*x, *y, *z], state.clone()))
        .collect();
    assert_eq!(snapshot(&sim), expected, "{case}: initial");
    for tick in 1..=run.capture_ticks {
        for action in run.actions.iter().filter(|a| a.tick == tick) {
            let [x, y, z] = action.pos;
            sim.use_block(Pos::new(x, y, z));
        }
        sim.step();
        for change in run.changes.iter().filter(|c| c.tick == tick) {
            for (x, y, z, from, to) in &change.blocks {
                let p = [*x, *y, *z];
                assert_eq!(
                    expected
                        .get(&p)
                        .map(String::as_str)
                        .unwrap_or("minecraft:air"),
                    from,
                    "golden transition continuity: {case}, tick {tick}, {p:?}"
                );
                if to == "minecraft:air" {
                    expected.remove(&p);
                } else {
                    expected.insert(p, to.clone());
                }
            }
        }
        assert_eq!(sim.tick_count(), tick);
        assert_eq!(snapshot(&sim), expected, "{case}: tick {tick}");
        assert!(sim.unknown_report().is_none(), "{case}: tick {tick}");
    }
}

#[test]
fn lamp_edges_notify_the_watching_observer_like_vanilla() {
    let fixture = include_str!("corpus/structures/flag2_lamp_observer.snbt");
    check("synthetic_lamp_observer_cycle", fixture);
    check("synthetic_lamp_observer_idle", fixture);
}

#[test]
fn trapdoor_edges_notify_the_watching_observer_like_vanilla() {
    let fixture = include_str!("corpus/structures/flag2_trapdoor_observer.snbt");
    check("synthetic_iron_trapdoor_observer_cycle", fixture);
    check("synthetic_iron_trapdoor_observer_idle", fixture);
}
