//! Corpus-level timeline and cycle-detection checks.

use mc_test::SettleMode;
use mc_tick::{CycleKind, Pos, Simulation, Structure};

const BB: &str = include_str!("corpus/structures/bb.snbt");
const ADDER32: &str = include_str!("corpus/structures/adder32.snbt");
const DOOR: &str = include_str!("corpus/structures/door_6x6_inworld.snbt");
const FLYER: &str = include_str!("corpus/structures/flying_machine_east.snbt");

fn sim(snbt: &str, settle: SettleMode) -> Simulation {
    let structure = Structure::parse(snbt).expect("fixture parses");
    let actuators = [
        "minecraft:redstone_block".to_string(),
        "minecraft:air".to_string(),
    ];
    mc_test::build_sim(
        &structure,
        Pos::new(0, 0, 0),
        settle,
        &actuators,
        &[],
        None,
        "timeline",
    )
}

#[test]
fn door_has_an_exact_active_cycle() {
    let mut sim = sim(DOOR, SettleMode::InWorld);
    sim.record_timeline();
    sim.run(10);
    sim.use_block(Pos::new(10, 4, 1));
    sim.run(50);
    sim.use_block(Pos::new(10, 4, 1));
    sim.run(120);

    let timeline = sim.recorded_timeline().expect("timeline");
    let cycle = timeline
        .detect_cycles(sim.registry())
        .exact
        .expect("door should return to an earlier absolute state");
    assert_eq!(cycle.kind, CycleKind::Exact);
    assert_eq!(cycle.drift, Pos::default());
    assert!(
        cycle.period > 1,
        "an active door cycle is not a stationary tick"
    );
    assert!(timeline
        .select_cycle(CycleKind::Exact, sim.registry())
        .is_ok());
}

#[test]
fn bb_fingerprint_tracks_westward_motion() {
    let mut sim = sim(BB, SettleMode::InWorld);
    let air = sim.registry_mut().intern("minecraft:air").unwrap();
    sim.record_timeline();
    sim.place_block(Pos::new(31, 7, 13), air);
    sim.run(500);

    let timeline = sim.recorded_timeline().expect("timeline");
    let first = timeline
        .frame_at(timeline.start_tick, sim.registry())
        .expect("initial frame");
    let last = timeline
        .frame_at(timeline.end_tick, sim.registry())
        .expect("last frame");
    assert!(
        last.origin.x < first.origin.x,
        "the recorded bounding box should follow BB west"
    );
    if let Some(cycle) = timeline.detect_cycles(sim.registry()).translated {
        assert!(cycle.drift.x < 0, "BB should only drift west: {cycle:?}");
        assert_eq!((cycle.drift.y, cycle.drift.z), (0, 0));
    }
}

#[test]
fn small_flyer_has_a_translated_cycle_east() {
    let mut sim = sim(FLYER, SettleMode::Quiet);
    let redstone = sim
        .registry_mut()
        .intern("minecraft:redstone_block")
        .unwrap();
    let air = sim.registry_mut().intern("minecraft:air").unwrap();
    sim.record_timeline();
    sim.run(2);
    sim.place_block(Pos::new(2, 1, 1), redstone);
    sim.run(2);
    sim.place_block(Pos::new(2, 1, 1), air);
    sim.run(150);

    let timeline = sim.recorded_timeline().expect("timeline");
    let cycle = timeline
        .detect_cycles(sim.registry())
        .translated
        .expect("flyer should repeat after translating");
    assert!(cycle.drift.x > 0, "flyer should drift east: {cycle:?}");
    assert_eq!(cycle.drift.y, 0);
    assert_eq!(cycle.drift.z, 0);
    let between = timeline
        .select_between_actions(0)
        .expect("kick-to-removal range");
    assert_eq!((between.start_tick(), between.end_tick()), (2, 4));
}

#[test]
fn settled_adder_does_not_claim_a_translated_cycle() {
    let mut sim = sim(ADDER32, SettleMode::Placement);
    sim.record_timeline();
    for _ in 0..400 {
        if sim.is_quiescent() {
            break;
        }
        sim.step();
    }

    let timeline = sim.recorded_timeline().expect("timeline");
    assert!(
        timeline.detect_cycles(sim.registry()).translated.is_none(),
        "a stationary computer must not be reported as a flying machine"
    );
}
