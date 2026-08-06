//! A replayed frame must equal the world the simulation actually had.
//!
//! Checked against an independent oracle — a second run of the same fixture
//! stepped to the same tick — rather than against the recorder's own stored
//! frames. That keeps the test meaningful once storage is removed, and it is
//! the guard that makes collapsing to one recorder safe.

use mc_test::SettleMode;
use mc_tick::{Pos, Simulation, StateFrame, Structure};

const DOOR: &str = include_str!("corpus/structures/door_6x6_inworld.snbt");

fn sim(snbt: &str) -> Simulation {
    let structure = Structure::parse(snbt).expect("fixture parses");
    let actuators = [
        "minecraft:redstone_block".to_string(),
        "minecraft:air".to_string(),
    ];
    mc_test::build_sim(
        &structure,
        Pos::new(0, 0, 0),
        SettleMode::InWorld,
        &actuators,
        &[],
        None,
        "timeline_replay",
    )
}

/// The same script the recorded run followed, stopped at `tick`.
///
/// Recording begins before the lever is touched, so the frame at tick 0 is
/// the untouched world — an oracle that clicks first is comparing a
/// different starting point, not a different replay.
fn oracle_at(tick: u64) -> Simulation {
    let mut oracle = sim(DOOR);
    if tick == 0 {
        return oracle;
    }
    oracle.use_block(Pos::new(10, 4, 1));
    while oracle.tick_count() < tick {
        oracle.step();
    }
    oracle
}

#[test]
fn a_replayed_frame_matches_the_world_the_simulation_had() {
    let mut recorded = sim(DOOR);
    recorded.record_timeline();
    recorded.use_block(Pos::new(10, 4, 1));
    recorded.run(40);
    let timeline = recorded.recorded_timeline().expect("timeline");

    // The oracle: the same fixture, same inputs, no recording, stepped to each
    // tick under test and read straight out of the world.
    for tick in [0, 1, 2, 7, 23, 40] {
        let oracle = oracle_at(tick);
        let want = StateFrame::of(tick, oracle.world(), oracle.registry());
        let got = timeline
            .frame_at(tick, recorded.registry())
            .unwrap_or_else(|| panic!("no frame for tick {tick}"));
        assert_eq!(got, want, "replayed frame differs at tick {tick}");
    }
}

#[test]
fn every_replayed_frame_equals_the_one_the_recorder_stored() {
    // The anti-drift check in its most direct form: while the recorder still
    // keeps a frame per tick, a replayed frame must equal the stored one for
    // every tick of the run. Task 5 removes the storage this compares
    // against, so this test is why removing it is safe.
    let mut recorded = sim(DOOR);
    recorded.record_timeline();
    recorded.use_block(Pos::new(10, 4, 1));
    recorded.run(40);
    let timeline = recorded.recorded_timeline().expect("timeline");

    assert!(timeline.frames.len() > 1, "the run recorded frames to compare");
    for stored in &timeline.frames {
        let replayed = timeline
            .frame_at(stored.tick, recorded.registry())
            .unwrap_or_else(|| panic!("no replayed frame for tick {}", stored.tick));
        assert_eq!(&replayed, stored, "replay differs at tick {}", stored.tick);
    }
}

#[test]
fn digests_cover_every_tick_of_the_run_in_order() {
    let mut recorded = sim(DOOR);
    recorded.record_timeline();
    recorded.use_block(Pos::new(10, 4, 1));
    recorded.run(12);
    let timeline = recorded.recorded_timeline().expect("timeline");

    let digests = timeline.digests(recorded.registry());
    let ticks: Vec<u64> = digests.iter().map(|d| d.tick).collect();
    assert_eq!(ticks, (timeline.start_tick..=timeline.end_tick).collect::<Vec<_>>());
    for digest in &digests {
        let frame = timeline
            .frame_at(digest.tick, recorded.registry())
            .expect("frame for a digest tick");
        assert_eq!(digest.exact, frame.exact);
        assert_eq!(digest.translated, frame.translated);
        assert_eq!(digest.origin, frame.origin);
    }
}
