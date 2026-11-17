//! Wall shape signals, captured in playerless Minecraft Java 1.21.11.
#[allow(dead_code)]
#[path = "../examples/check_community_capture.rs"]
mod checker;

use mc_tick::Pos;

#[test]
fn heights_posts_and_observers_match_every_vanilla_tick() {
    checker::assert_matches_capture(include_bytes!("corpus/walls12111/heights.json"));
}

#[test]
fn connections_and_exceptions_match_every_vanilla_tick() {
    checker::assert_matches_capture(include_bytes!("corpus/walls12111/connections.json"));
}

#[test]
fn waterlogged_wall_preserves_fluid_tick_timing() {
    checker::assert_matches_capture(include_bytes!("corpus/walls12111/waterlogged.json"));
}

#[test]
fn piston_extension_and_retraction_preserve_wall_observer_timing() {
    checker::assert_matches_capture(include_bytes!("corpus/walls12111/piston.json"));
}

#[test]
fn raw_wall_states_stay_idle_and_checkpoint_replays_observer_pulses() {
    let mut sim = checker::fixture_sim(include_bytes!("corpus/walls12111/heights.json"));
    sim.run(24);
    assert!(sim.recorded().is_empty());
    let stone = sim.registry().get("minecraft:stone").unwrap();
    sim.place_block(Pos::new(1, 1, 1), stone);
    sim.step();
    let saved = sim.checkpoint();
    sim.clear_recorded();
    sim.run(12);
    let state = sim.checkpoint();
    let changes = sim.recorded().to_vec();
    assert!(!changes.is_empty(), "The wall must trigger its observer");
    sim.restore(&saved);
    sim.clear_recorded();
    sim.run(12);
    assert_eq!(sim.checkpoint(), state);
    assert_eq!(sim.recorded(), changes);
}

#[test]
fn every_wall_companion_is_active_but_not_a_collision_cube() {
    let sim = checker::fixture_sim(include_bytes!("corpus/walls12111/heights.json"));
    for n in ["none", "low", "tall"] {
        for e in ["none", "low", "tall"] {
            for s in ["none", "low", "tall"] {
                for w in ["none", "low", "tall"] {
                    for up in [false, true] {
                        for wet in [false, true] {
                            let d = format!("minecraft:cobblestone_wall[east={e},north={n},south={s},up={up},waterlogged={wet},west={w}]");
                            assert!(sim.registry().get(&d).is_some(), "{d}");
                            assert!(mc_tick::vanilla::is_simulation_component(&d));
                            assert!(!mc_tick::vanilla::is_collision_full_cube(&d));
                        }
                    }
                }
            }
        }
    }
}
