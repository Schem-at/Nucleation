//! `run_with` reports outcome, horizon and timing without panicking.

use mc_test::{mc_tick::Structure, run_with, Case, RunOptions};

const STONE: &str = r#"{DataVersion: 4325, size: [1, 1, 1], entities: [], blocks: [{pos: [0, 0, 0], state: 0}], palette: [{Name: "minecraft:stone"}]}"#;

fn case(json: &str) -> Case {
    serde_json::from_str(json).expect("a well-formed case")
}

#[test]
fn a_passing_case_reports_pass_and_its_horizon() {
    let structure = Structure::parse(STONE).expect("parses");
    let case = case(
        r#"{"name":"stone stays","checks":[{"tick":3,"expect":"blocks","blocks":{"0,0,0":"minecraft:stone"}}]}"#,
    );
    let result = run_with(&structure, &case, None, &RunOptions::default());
    assert_eq!(result.outcome, Ok(()));
    assert_eq!(result.ticks, 3);
    assert_eq!(result.name, "stone stays");
}

#[test]
fn a_case_may_assert_a_foreign_block_inert() {
    // A structure carrying a block the engine does not model — the lithium
    // corpus's `test_block` — runs once the case itself asserts it inert.
    const FOREIGN: &str = r#"{DataVersion: 4325, size: [2, 1, 1], entities: [], blocks: [{pos: [0, 0, 0], state: 0}, {pos: [1, 0, 0], state: 1}], palette: [{Name: "minecraft:stone"}, {Name: "minecraft:test_block", Properties: {mode: "start"}}]}"#;
    let structure = Structure::parse(FOREIGN).expect("parses");
    let case = case(
        r#"{"name":"foreign blocks by assertion","inert":["minecraft:test_block"],"checks":[{"tick":1,"expect":"blocks","blocks":{"0,0,0":"minecraft:stone"}}]}"#,
    );
    let result = run_with(&structure, &case, None, &RunOptions::default());
    assert_eq!(result.outcome, Ok(()));
}

#[test]
fn a_failing_case_reports_the_diff_not_a_panic() {
    let structure = Structure::parse(STONE).expect("parses");
    let case = case(
        r#"{"name":"stone is not glass","checks":[{"tick":0,"expect":"blocks","blocks":{"0,0,0":"minecraft:glass"}}]}"#,
    );
    let outcome = run_with(&structure, &case, None, &RunOptions::default()).outcome;
    let report = outcome.expect_err("stone is not glass");
    assert!(report.contains("minecraft:glass"), "{report}");
    assert!(report.contains("minecraft:stone"), "{report}");
}
