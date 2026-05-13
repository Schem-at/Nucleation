//! Round-trip regression tests for redstone-relevant block states through
//! the Bedrock `.mcstructure` exporter+importer.
//!
//! Each test:
//! 1. Builds a UniversalSchematic with a single block carrying specific
//!    property values
//! 2. Exports it via `to_mcstructure`
//! 3. Re-imports the bytes via `from_mcstructure`
//! 4. Asserts the property values survive the round-trip
//!
//! Failures in this file are the symptoms reported by community users
//! (hopper-always-down, repeater-wrong-delay, sticky-piston-missing, etc.)
//! and pin down which translations in nucleation/blockpedia are broken.
//!
//! The tests *intentionally* permit either the Java identifier
//! (`minecraft:hopper`) or the Bedrock identifier (`minecraft:hopper`) on
//! reload, since both editions share most names. They check ONLY that
//! the orientation / state properties relevant to the bug come back
//! correctly. If the relevant property is missing, the assertion fails
//! with a clear message saying which property was lost.

use nucleation::formats::mcstructure::{from_mcstructure, to_mcstructure};
use nucleation::{BlockState, UniversalSchematic};
use std::collections::HashMap;

/// Build a 1-block schematic at (0,0,0) with the given state and ensure
/// the bounds round-trip cleanly.
fn one_block(name: &str, props: &[(&str, &str)]) -> UniversalSchematic {
    let mut state = BlockState::new(name.to_string());
    for (k, v) in props {
        state = state.with_property(k.to_string(), v.to_string());
    }
    let mut s = UniversalSchematic::new("test".to_string());
    s.set_block(0, 0, 0, &state);
    s
}

fn round_trip(s: &UniversalSchematic) -> UniversalSchematic {
    let bytes = to_mcstructure(s).expect("to_mcstructure must succeed");
    from_mcstructure(&bytes).expect("from_mcstructure must succeed")
}

/// Pull the BlockState at (0,0,0) from a round-tripped schematic. Returns
/// the (block_name, properties) pair for assertions.
fn block_at_origin(s: &UniversalSchematic) -> (String, HashMap<String, String>) {
    let b = s
        .get_block(0, 0, 0)
        .expect("expected a block at (0,0,0) after round-trip");
    (
        b.name.to_string(),
        b.properties
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
}

/// Asserts `props` contains every entry from `expected`. Permits extra
/// keys (e.g. defaults filled in by Bedrock) but flags missing or mismatched
/// values with a descriptive message.
#[track_caller]
fn assert_props_contain(props: &HashMap<String, String>, expected: &[(&str, &str)], context: &str) {
    for (k, v) in expected {
        match props.get(*k) {
            Some(got) if got == v => {}
            Some(got) => panic!(
                "[{}] property '{}' = '{}' (expected '{}'); full props: {:?}",
                context, k, got, v, props
            ),
            None => panic!(
                "[{}] property '{}' missing (expected '{}'); full props: {:?}",
                context, k, v, props
            ),
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
//  Hopper — the headline bug: "all hoppers face down"
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn hopper_facing_north_preserved() {
    let s = one_block("minecraft:hopper", &[("facing", "north")]);
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(&props, &[("facing", "north")], "hopper north");
}

#[test]
fn hopper_facing_south_preserved() {
    let s = one_block("minecraft:hopper", &[("facing", "south")]);
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(&props, &[("facing", "south")], "hopper south");
}

#[test]
fn hopper_facing_east_preserved() {
    let s = one_block("minecraft:hopper", &[("facing", "east")]);
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(&props, &[("facing", "east")], "hopper east");
}

#[test]
fn hopper_facing_west_preserved() {
    let s = one_block("minecraft:hopper", &[("facing", "west")]);
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(&props, &[("facing", "west")], "hopper west");
}

#[test]
fn hopper_facing_down_preserved() {
    let s = one_block("minecraft:hopper", &[("facing", "down")]);
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(&props, &[("facing", "down")], "hopper down");
}

// ───────────────────────────────────────────────────────────────────────────
//  Repeater — split block in Bedrock (powered_repeater vs unpowered_repeater)
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn repeater_unpowered_full_state() {
    let s = one_block(
        "minecraft:repeater",
        &[
            ("delay", "3"),
            ("facing", "east"),
            ("powered", "false"),
            ("locked", "false"),
        ],
    );
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(
        &props,
        &[("delay", "3"), ("facing", "east"), ("powered", "false")],
        "unpowered repeater",
    );
}

#[test]
fn repeater_powered_full_state() {
    let s = one_block(
        "minecraft:repeater",
        &[
            ("delay", "2"),
            ("facing", "west"),
            ("powered", "true"),
            ("locked", "false"),
        ],
    );
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(
        &props,
        &[("delay", "2"), ("facing", "west"), ("powered", "true")],
        "powered repeater",
    );
}

// ───────────────────────────────────────────────────────────────────────────
//  Sticky piston — "from the hologram only didn't build"
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn sticky_piston_facing_up_facing_survives() {
    // Bedrock pistons have no `extended` property — when a piston is
    // extended, Bedrock places a separate `pistonArmCollision` block in
    // the adjacent cell. Round-tripping `extended` therefore requires
    // multi-block awareness and lives outside this state-only test.
    // Here we verify only that `facing` survives.
    let s = one_block(
        "minecraft:sticky_piston",
        &[("facing", "up"), ("extended", "false")],
    );
    let reloaded = round_trip(&s);
    let (name, props) = block_at_origin(&reloaded);
    assert!(
        name.contains("sticky_piston"),
        "expected sticky_piston, got '{}' (props={:?})",
        name,
        props
    );
    assert_props_contain(&props, &[("facing", "up")], "sticky piston up");
}

#[test]
fn sticky_piston_facing_east_facing_survives() {
    let s = one_block(
        "minecraft:sticky_piston",
        &[("facing", "east"), ("extended", "true")],
    );
    let reloaded = round_trip(&s);
    let (name, props) = block_at_origin(&reloaded);
    assert!(
        name.contains("sticky_piston"),
        "expected sticky_piston, got '{}' (props={:?})",
        name,
        props
    );
    assert_props_contain(&props, &[("facing", "east")], "sticky piston east");
}

/// TODO: piston `extended` requires placing `pistonArmCollision` in the
/// adjacent cell during Java→Bedrock export. Until that's implemented,
/// `extended` is dropped silently. Reinstate this test when that work lands.
#[test]
#[ignore = "Bedrock piston `extended` requires adjacent pistonArmCollision block — not yet implemented"]
fn sticky_piston_extended_survives_round_trip() {
    let s = one_block(
        "minecraft:sticky_piston",
        &[("facing", "east"), ("extended", "true")],
    );
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(
        &props,
        &[("facing", "east"), ("extended", "true")],
        "sticky piston extended",
    );
}

// ───────────────────────────────────────────────────────────────────────────
//  Other commonly-broken redstone blocks
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn observer_facing_and_powered() {
    let s = one_block(
        "minecraft:observer",
        &[("facing", "east"), ("powered", "true")],
    );
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(
        &props,
        &[("facing", "east"), ("powered", "true")],
        "observer",
    );
}

#[test]
fn comparator_full_state() {
    let s = one_block(
        "minecraft:comparator",
        &[
            ("facing", "north"),
            ("mode", "subtract"),
            ("powered", "true"),
        ],
    );
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(
        &props,
        &[
            ("facing", "north"),
            ("mode", "subtract"),
            ("powered", "true"),
        ],
        "comparator",
    );
}

#[test]
fn lever_wall_east_powered() {
    let s = one_block(
        "minecraft:lever",
        &[("facing", "east"), ("face", "wall"), ("powered", "true")],
    );
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(
        &props,
        &[("facing", "east"), ("face", "wall"), ("powered", "true")],
        "lever",
    );
}

#[test]
fn wall_torch_facing_south() {
    let s = one_block("minecraft:wall_torch", &[("facing", "south")]);
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(&props, &[("facing", "south")], "wall_torch");
}

#[test]
fn chest_orientation_single() {
    let s = one_block(
        "minecraft:chest",
        &[
            ("facing", "south"),
            ("type", "single"),
            ("waterlogged", "false"),
        ],
    );
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(&props, &[("facing", "south"), ("type", "single")], "chest");
}

#[test]
fn oak_stairs_west_top_straight() {
    let s = one_block(
        "minecraft:oak_stairs",
        &[
            ("facing", "west"),
            ("half", "top"),
            ("shape", "straight"),
            ("waterlogged", "false"),
        ],
    );
    let reloaded = round_trip(&s);
    let (_, props) = block_at_origin(&reloaded);
    assert_props_contain(
        &props,
        &[("facing", "west"), ("half", "top"), ("shape", "straight")],
        "oak_stairs",
    );
}
