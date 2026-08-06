//! Opt-in event assertions, and the diagnostic event log on failure.

use mc_test::{mc_tick::Structure, run, run_with, Case, RunOptions};

const LAMP: &str = r#"{DataVersion: 4325, size: [2, 1, 1], entities: [], blocks: [{pos: [0, 0, 0], state: 0}], palette: [{Name: "minecraft:redstone_lamp", Properties: {lit: "false"}}]}"#;

fn case(json: &str) -> Case {
    serde_json::from_str(json).expect("a well-formed case")
}

#[test]
fn a_lit_lamp_is_a_recorded_block_change() {
    let structure = Structure::parse(LAMP).expect("parses");
    let case = case(
        r#"{
            "name": "the lamp lights",
            "actions": [{"tick": 0, "place": [1, 0, 0], "state": "minecraft:redstone_block"}],
            "checks": [{"tick": 4, "expect": "blocks", "blocks": {"0,0,0": "minecraft:redstone_lamp[lit=true]"}}],
            "events": [{"kind": "block-changed", "pos": [0, 0, 0], "to": "minecraft:redstone_lamp[lit=true]"}]
        }"#,
    );
    run(&structure, &case, None).expect("the lamp must light, as an end state and as an event");
}

#[test]
fn an_event_that_never_happened_fails_with_bounds() {
    let structure = Structure::parse(LAMP).expect("parses");
    let case = case(
        r#"{
            "name": "no ghost events",
            "checks": [{"tick": 2, "expect": "quiescent"}],
            "events": [{"kind": "block-changed", "pos": [0, 0, 0], "to": "minecraft:redstone_lamp[lit=true]"}]
        }"#,
    );
    let report = run(&structure, &case, None).expect_err("nothing ever powers the lamp");
    assert!(report.contains("block-changed"), "{report}");
}

#[test]
fn a_failing_check_dumps_the_event_window() {
    let structure = Structure::parse(LAMP).expect("parses");
    let case = case(
        r#"{
            "name": "wrong expectation, useful dump",
            "actions": [{"tick": 0, "place": [1, 0, 0], "state": "minecraft:redstone_block"}],
            "checks": [{"tick": 4, "expect": "blocks", "blocks": {"0,0,0": "minecraft:redstone_lamp[lit=false]"}}]
        }"#,
    );
    let report = run_with(&structure, &case, None, &RunOptions { trace_window: 4 })
        .outcome
        .expect_err("the lamp did light");
    assert!(
        report.contains("event log"),
        "the dump must be present: {report}"
    );
    assert!(report.contains("redstone_lamp"), "{report}");
}

#[test]
fn an_accept_test_block_latches_when_powered() {
    // accept ← dust ← (redstone_block placed by the case). The engine's
    // TestAccept behaviour must latch the accept to `fired=true`, recorded
    // as a block change the events machinery can assert on — the vanilla
    // block-based pass condition, headless.
    const RIG: &str = r#"{DataVersion: 4325, size: [3, 1, 1], entities: [], blocks: [{pos: [0, 0, 0], state: 0}, {pos: [1, 0, 0], state: 1}], palette: [{Name: "minecraft:test_block", Properties: {mode: "accept"}}, {Name: "minecraft:redstone_wire", Properties: {east: "side", north: "none", power: "0", south: "none", west: "side"}}]}"#;
    let structure = Structure::parse(RIG).expect("parses");
    let case = case(
        r#"{
            "name": "the accept fires",
            "actions": [{"tick": 0, "place": [2, 0, 0], "state": "minecraft:redstone_block"}],
            "checks": [{"tick": 4, "expect": "blocks", "blocks": {"0,0,0": "minecraft:test_block[fired=true]"}}],
            "events": [{"kind": "block-changed", "pos": [0, 0, 0], "to": "minecraft:test_block[fired=true]"}]
        }"#,
    );
    run(&structure, &case, None)
        .expect("the accept must latch, as an end state and as a recorded event");
}

#[test]
fn a_powered_command_block_runs_its_setblock() {
    // command block (facing up, `setblock ~ ~2 ~ redstone_block`) beside the
    // cell the case powers. Rising edge → 1gt delay → the block appears two
    // above, recorded like any other change.
    const RIG: &str = r#"{DataVersion: 4325, size: [2, 4, 1], entities: [], blocks: [{pos: [0, 0, 0], state: 0, nbt: {id: "minecraft:command_block", Command: "setblock ~ ~2 ~ redstone_block"}}], palette: [{Name: "minecraft:command_block", Properties: {conditional: "false", facing: "up"}}]}"#;
    let structure = Structure::parse(RIG).expect("parses");
    let case = case(
        r#"{
            "name": "the command runs",
            "actions": [{"tick": 0, "place": [1, 0, 0], "state": "minecraft:redstone_block"}],
            "checks": [{"tick": 4, "expect": "blocks", "blocks": {"0,2,0": "minecraft:redstone_block"}}],
            "events": [{"kind": "block-changed", "pos": [0, 2, 0], "to": "minecraft:redstone_block"}]
        }"#,
    );
    run(&structure, &case, None).expect("the setblock must land, as end state and as event");
}

#[test]
fn an_unstackable_item_reads_its_own_stack_limit_on_a_comparator() {
    // hopper(axe) → chest ← comparator. A cooldown-0 hopper transfers on its
    // first tick, and one golden axe (max stack 1) in a 27-slot chest is
    // fullness 1/27 → signal 1. Computing fullness per 64 read it as
    // effectively empty, which broke every unstackable-item counter.
    const RIG: &str = r#"{DataVersion: 4325, size: [4, 1, 1], entities: [], blocks: [{pos: [0, 0, 0], state: 0, nbt: {id: "minecraft:hopper", Items: [{Slot: 0b, count: 1, id: "minecraft:golden_axe"}]}}, {pos: [1, 0, 0], state: 1, nbt: {id: "minecraft:chest", Items: []}}, {pos: [2, 0, 0], state: 2, nbt: {OutputSignal: 0, id: "minecraft:comparator"}}, {pos: [3, 0, 0], state: 3}], palette: [{Name: "minecraft:hopper", Properties: {enabled: "true", facing: "east"}}, {Name: "minecraft:chest", Properties: {facing: "west", type: "single", waterlogged: "false"}}, {Name: "minecraft:comparator", Properties: {facing: "west", mode: "compare", powered: "false"}}, {Name: "minecraft:redstone_wire", Properties: {east: "none", north: "none", power: "0", south: "none", west: "side"}}]}"#;
    let structure = Structure::parse(RIG).expect("parses");
    let case = case(
        r#"{
            "name": "one axe reads signal 1",
            "checks": [
                {"tick": 20, "expect": "blocks", "blocks": {"2,0,0": "minecraft:comparator[powered=true]", "3,0,0": "minecraft:redstone_wire[power=1]"}}
            ]
        }"#,
    );
    run(&structure, &case, None).expect("the axe lands and reads signal 1");
}

#[test]
fn an_unknown_event_kind_is_refused_loudly() {
    let structure = Structure::parse(LAMP).expect("parses");
    let case = case(
        r#"{
            "name": "future kinds fail, not skip",
            "checks": [{"tick": 0, "expect": "quiescent"}],
            "events": [{"kind": "entity-moved"}]
        }"#,
    );
    let report = run(&structure, &case, None).expect_err("v1 knows only block-changed");
    assert!(report.contains("entity-moved"), "{report}");
}
