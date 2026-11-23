//! Fresh playerless 1.21.11 captures: support propagation, observers and water.
#[allow(dead_code)]
#[path = "../examples/check_community_capture.rs"]
mod checker;

use mc_test::{build_sim, SettleMode};
use mc_tick::{Pos, Simulation, StateId, Structure};

#[test]
fn note_trigger_matches_every_vanilla_tick() {
    checker::assert_matches_capture(include_bytes!("corpus/scaffolding12111/note_trigger.json"));
}

#[test]
fn support_faces_and_vertical_propagation_match_every_vanilla_tick() {
    checker::assert_matches_capture(include_bytes!("corpus/scaffolding12111/supports.json"));
}

#[test]
fn waterlogged_scaffolding_spreads_on_the_vanilla_fluid_tick() {
    checker::assert_matches_capture(include_bytes!("corpus/scaffolding12111/waterlogged.json"));
}

fn minimal() -> Simulation {
    checker::fixture_sim(include_bytes!("corpus/scaffolding12111/note_trigger.json"))
}

#[test]
fn raw_loading_is_idle_and_checkpoint_restores_pending_support_updates() {
    let mut sim = minimal();
    sim.run(24);
    assert!(sim.recorded().is_empty());
    sim.use_block(Pos::new(0, 3, 0));
    sim.run(2);
    let checkpoint = sim.checkpoint();
    sim.clear_recorded();
    sim.run(22);
    let final_state = sim.checkpoint();
    let records = sim.recorded().to_vec();
    sim.restore(&checkpoint);
    sim.clear_recorded();
    sim.run(22);
    assert_eq!(sim.checkpoint(), final_state);
    assert_eq!(sim.recorded(), records);
}

fn scaffold_column(distance: u8) -> Simulation {
    let text = serde_json::json!({
        "DataVersion": 4671, "size": [1,2,1],
        "palette": [{"Name":"minecraft:stone"},
            {"Name":"minecraft:scaffolding", "Properties": {
                "bottom":"false", "distance":distance.to_string(), "waterlogged":"false"}}],
        "blocks": [{"pos":[0,0,0],"state":0}, {"pos":[0,1,0],"state":1}],
        "entities": []
    }).to_string();
    let structure = Structure::parse(&text).unwrap();
    build_sim(
        &structure,
        Pos::new(-30, 100, -30),
        SettleMode::InWorld,
        &[],
        &[],
        Some(4671),
        "scaffolding loss",
    )
}

#[test]
fn loss_of_all_support_drops_one_item_and_removes_the_block() {
    let mut sim = scaffold_column(0);
    sim.place_block(Pos::new(0, 0, 0), StateId::AIR);
    sim.step();
    assert_eq!(sim.world().get(Pos::new(0, 1, 0)), StateId::AIR);
    assert_eq!(sim.item_entities().len(), 1);
    assert!(sim.unknown_report().is_none());
}

#[test]
fn preexisting_distance_seven_reports_the_unimplemented_falling_entity_path() {
    let mut sim = scaffold_column(7);
    sim.place_block(Pos::new(0, 0, 0), StateId::AIR);
    sim.step();
    assert!(sim.unknown_report().unwrap().contains("scaffolding"));
    assert!(sim.item_entities().is_empty());
}

#[test]
fn companions_include_all_scaffolding_states_and_water() {
    let sim = minimal();
    for distance in 0..=7 {
        for bottom in [false, true] {
            for waterlogged in [false, true] {
                let descriptor = format!("minecraft:scaffolding[bottom={bottom},distance={distance},waterlogged={waterlogged}]");
                assert!(sim.registry().get(&descriptor).is_some(), "{descriptor}");
                assert!(mc_tick::vanilla::is_simulation_component(&descriptor));
                assert!(!mc_tick::vanilla::is_collision_full_cube(&descriptor));
            }
        }
    }
    assert!(sim.registry().get("minecraft:water[level=0]").is_some());
}
