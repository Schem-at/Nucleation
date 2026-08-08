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
fn cycles_found_by_replay_still_describe_the_flying_machine() {
    const FLYER: &str = include_str!("corpus/structures/flying_machine_east.snbt");
    let mut sim = sim(FLYER);
    let redstone = sim
        .registry_mut()
        .intern("minecraft:redstone_block")
        .unwrap();
    let air = sim.registry_mut().intern("minecraft:air").unwrap();
    sim.record_timeline();
    // The kick block has to be taken away again or the machine stalls after a
    // single step and never translates.
    sim.place_block(Pos::new(2, 1, 1), redstone);
    sim.run(2);
    sim.place_block(Pos::new(2, 1, 1), air);
    sim.run(58);
    let timeline = sim.recorded_timeline().expect("timeline");

    let report = timeline.detect_cycles(sim.registry());
    let translated = report
        .translated
        .expect("a flying machine repeats itself, displaced");
    assert!(translated.period > 0);
    assert!(
        translated.drift.x != 0,
        "it travels along x: {translated:?}"
    );

    // The selection resolves to the same span, and its initial frame is the
    // world at that tick.
    let selection = timeline
        .select_cycle(mc_tick::CycleKind::Translated, sim.registry())
        .expect("selectable");
    assert_eq!(selection.start_tick(), translated.start_tick);
    let frame = timeline.initial_frame(selection, sim.registry());
    assert_eq!(frame.tick, translated.start_tick);
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
    assert_eq!(
        ticks,
        (timeline.start_tick..=timeline.end_tick).collect::<Vec<_>>()
    );
    for digest in &digests {
        let frame = timeline
            .frame_at(digest.tick, recorded.registry())
            .expect("frame for a digest tick");
        assert_eq!(digest.exact, frame.exact);
        assert_eq!(digest.translated, frame.translated);
        assert_eq!(digest.origin, frame.origin);
    }
}

#[test]
fn a_recording_is_a_seed_and_a_log_not_a_world_per_tick() {
    // The structural half of this — that `RunTimeline` has no `frames` field —
    // is enforced by the compiler once every reader is gone. What is worth
    // asserting at runtime is the shape of what remains: exactly one
    // whole-world copy, and a log that tracks activity rather than build size
    // times ticks. A stored-frame recorder would fail the second assertion by
    // two orders of magnitude on this fixture.
    let mut sim = sim(DOOR);
    sim.record_timeline();
    sim.use_block(Pos::new(10, 4, 1));
    sim.run(200);
    let timeline = sim.recorded_timeline().expect("timeline");

    let blocks = timeline.initial.blocks.len();
    assert!(blocks > 100, "the door is a real build, not a toy");
    assert_eq!(
        timeline.initial.tick, timeline.start_tick,
        "the one snapshot held is the one recording started from"
    );
    assert!(
        timeline.changes.len() < blocks * 4,
        "a 200-tick door recorded {} changes against {blocks} blocks — the log \
         should be activity-shaped, not world-shaped",
        timeline.changes.len(),
    );
    // And it still answers for any tick in the run, from that seed alone.
    assert!(timeline
        .frame_at(timeline.end_tick, sim.registry())
        .is_some());
}

/// A selection can only come from the `select_*` constructors, which validate
/// against the recorded span — so `initial_frame` has no unreachable branch to
/// panic in. Stage 3 runs this in WASM, where a panic aborts the page.
#[test]
fn every_selection_yields_the_frame_at_its_own_start() {
    let mut sim = sim(DOOR);
    sim.record_timeline();
    sim.use_block(Pos::new(10, 4, 1));
    sim.run(40);
    let timeline = sim.recorded_timeline().expect("timeline");

    for (start, end) in [(1u64, 5u64), (0, 40), (12, 13)] {
        let selection = timeline.select_ticks(start, end).expect("in range");
        assert_eq!(selection.start_tick(), start);
        assert_eq!(selection.end_tick(), end);
        let frame = timeline.initial_frame(selection, sim.registry());
        assert_eq!(frame.tick, start, "the frame is the one the selection names");
    }
    assert!(timeline.select_ticks(0, 9_999).is_err(), "out of span is refused");
    assert!(timeline.select_ticks(5, 5).is_err(), "an empty range is refused");
}

#[test]
fn a_rewind_ends_the_recording_instead_of_corrupting_it() {
    // Replay is only correct while the change log describes every mutation
    // since the timeline began. `restore` rewinds the world out from under
    // that log without touching it, so the recording must end rather than
    // silently start describing a world that no longer exists.
    let mut sim = sim(DOOR);
    sim.record_timeline();
    let checkpoint = sim.checkpoint();
    sim.use_block(Pos::new(10, 4, 1));
    sim.run(5);
    sim.restore(&checkpoint);
    assert!(
        sim.recorded_timeline().is_none(),
        "restoring mid-recording must drop the timeline, not leave it describing \
         a world the restore just rewound past"
    );
}

/// Stopping is not rewinding. A rewind discards a recording because the log
/// stops describing the world; stopping leaves the log true and merely ends
/// it — the span must stay readable, and it must stop growing.
#[test]
fn stopping_a_recording_keeps_it_and_stops_the_log_growing() {
    let mut sim = sim(DOOR);
    sim.record_timeline();
    sim.use_block(Pos::new(10, 4, 1));
    sim.run(20);

    let stopped = sim.stop_timeline().expect("a recording was running");
    let changes_at_stop = stopped.changes.len();
    assert!(changes_at_stop > 0, "the door did something worth recording");
    assert_eq!(stopped.end_tick, sim.tick_count(), "the span ends where the run was");

    // The simulation carries on; the recording does not.
    sim.run(40);
    assert!(
        sim.recorded_timeline().is_none(),
        "nothing is recording after a stop"
    );
    assert_eq!(
        stopped.changes.len(),
        changes_at_stop,
        "and the span already taken did not grow behind the caller's back"
    );
    // It is still a usable recording.
    assert!(stopped.frame_at(stopped.end_tick, sim.registry()).is_some());
    assert!(sim.stop_timeline().is_none(), "stopping twice is not an error");
}

/// The change log is ordered by tick, which is what lets `changed_between`
/// binary-search it instead of scanning. If a recorder ever appended out of
/// order the search would silently answer wrongly, so pin the ordering here
/// rather than trusting it.
#[test]
fn the_change_log_is_sorted_by_tick() {
    let mut sim = sim(DOOR);
    sim.record_timeline();
    sim.use_block(Pos::new(10, 4, 1));
    sim.run(60);
    let timeline = sim.recorded_timeline().expect("timeline");

    assert!(timeline.changes.len() > 100, "the door is a real recording");
    assert!(
        timeline.changes.windows(2).all(|w| w[0].tick <= w[1].tick),
        "the log must be non-decreasing in tick for a binary search to be sound"
    );
}

/// A build that settles is the case the scan was worst at: a door clicked
/// once opens (here, done recording changes by tick 15) and then sits
/// through a long quiet tail (tick 16..120). Every one of those quiet ticks
/// shares a fingerprint with its predecessor, so `detect_cycles` builds a
/// candidate for each repetition and asks `changed_between` about a span
/// that holds no changes — the case a linear `any()` cannot short-circuit,
/// so it walked the whole log every time.
///
/// A door that is only ever opened does not revisit an earlier absolute
/// state (open != closed), and `detect_cycles` deliberately discards
/// stationary — no-change — candidates rather than reporting the tick after
/// itself as a trivial "cycle" (see
/// `cycle_candidates_are_verified_and_stationary_ticks_are_ignored`). So the
/// correct result here is *no* cycle at all; what this test pins is that the
/// long, candidate-heavy, all-quiet tail still produces that same no-cycle
/// answer once `changed_between` searches instead of scanning.
/// `clear_recorded` must refuse while a run timeline is recording, and say
/// so: a timeline is a seed frame plus the change log, and replay trusts that
/// the log describes every mutation since `start_tick`. Clearing it out from
/// under a live recording leaves a seed with nothing to replay against, and
/// every later read — `frame_at` among them — would silently reconstruct the
/// wrong world.
#[test]
fn clear_recorded_refuses_during_a_run_timeline_and_replay_stays_correct() {
    let mut recorded = sim(DOOR);
    recorded.record_timeline();
    recorded.use_block(Pos::new(10, 4, 1));
    recorded.run(40);

    let cleared = recorded.clear_recorded();
    assert!(
        !cleared,
        "clear_recorded must refuse while a run timeline is recording"
    );

    let timeline = recorded.recorded_timeline().expect("timeline");
    let end_tick = timeline.end_tick;
    let oracle = oracle_at(end_tick);
    let want = StateFrame::of(end_tick, oracle.world(), oracle.registry());
    let got = timeline
        .frame_at(end_tick, recorded.registry())
        .unwrap_or_else(|| panic!("no frame for tick {end_tick}"));
    assert_eq!(
        got, want,
        "a refused clear must leave replay reconstructing the same world an \
         independent simulation reaches by stepping to the same tick"
    );
}

#[test]
fn cycle_detection_is_unchanged_on_a_settled_build() {
    let mut sim = sim(DOOR);
    sim.record_timeline();
    sim.use_block(Pos::new(10, 4, 1));
    sim.run(40);
    // Let it come to rest: the tail is all quiet ticks.
    sim.run(80);
    let timeline = sim.recorded_timeline().expect("timeline");

    let report = timeline.detect_cycles(sim.registry());
    assert!(
        report.exact.is_none(),
        "opening a door once never revisits an earlier absolute state"
    );
    assert!(
        report.translated.is_none(),
        "opening a door once never revisits an earlier translated state either"
    );
}
