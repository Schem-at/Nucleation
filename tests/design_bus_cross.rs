//! Acceptance sketch (1) of `redstone-eda/DESIGN_SPEC.md`, end to end in
//! Rust: endpoint hardware placed with raw `set_block`, ports declared as
//! anchor + step + width + type, two 8-bit buses routed with the crossing
//! IMPLICIT (dip-under tile), flatten / check / bake, then typed
//! walking-ones through the EMBEDDED contract — 8+8 patterns plus
//! isolation, no raw coordinates on the executor side.

#![cfg(all(
    feature = "routing",
    feature = "simulation",
    feature = "bridge",
    feature = "mc-tick"
))]

use nucleation::design::{BusState, BusStyle, Design};
use nucleation::io_contract::{IoType, Value};
use nucleation::simulation::typed_executor::BackendCircuitExecutor;
use nucleation::UniversalSchematic;

const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";

/// 8 levers stacked at 2y pitch, each with its connection dust one step
/// toward the field (`design_step1.py` geometry). Returns the bit-0
/// connection cell.
fn lever_bank(s: &mut UniversalSchematic, x: i32, z: i32, dx: i32, dz: i32) -> (i32, i32, i32) {
    for i in 0..8 {
        let y = 2 + 2 * i;
        s.set_block_from_string(x, y - 1, z, STONE).unwrap();
        s.set_block_from_string(x, y, z, LEVER).unwrap();
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE).unwrap();
        s.set_block_from_string(x + dx, y, z + dz, DUST).unwrap();
    }
    (x + dx, 2, z + dz)
}

/// 8 lamps stacked at 2y pitch, each lamp supporting its own dust.
fn lamp_bank(s: &mut UniversalSchematic, x: i32, z: i32) -> (i32, i32, i32) {
    for i in 0..8 {
        let y = 2 + 2 * i;
        s.set_block_from_string(x, y - 1, z, LAMP).unwrap();
        s.set_block_from_string(x, y, z, DUST).unwrap();
    }
    (x, 2, z)
}

#[test]
fn crossing_buses_pass_typed_walking_ones_through_the_embedded_contract() {
    // Loose block layer: endpoint hardware with raw set_block. Bus A runs
    // +X at z=8, bus B runs +Z at x=8 — they MUST cross.
    let mut s = UniversalSchematic::new("crossing".to_string());
    let a_in = lever_bank(&mut s, 0, 8, 1, 0);
    let a_out = lamp_bank(&mut s, 16, 8);
    let b_in = lever_bank(&mut s, 8, 0, 0, 1);
    let b_out = lamp_bank(&mut s, 8, 16);

    let mut d = Design::for_schematic("crossing", s);
    let step = (0, 2, 0);
    let ty = IoType::UnsignedInt { bits: 8 };
    d.declare_input("a_in", a_in, step, 8, ty.clone()).unwrap();
    d.declare_output("a_out", a_out, step, 8, ty.clone()).unwrap();
    d.declare_input("b_in", b_in, step, 8, ty.clone()).unwrap();
    d.declare_output("b_out", b_out, step, 8, ty).unwrap();

    // Two buses with per-bus styles; the crossing is implicit.
    let state = d
        .route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed);
    let style_b = BusStyle {
        bus_block: "minecraft:cyan_concrete".to_string(),
        transparent_block: "minecraft:cyan_stained_glass".to_string(),
    };
    let state = d
        .route_bus("bus_b", "b_in", &["b_out"], vec![], style_b)
        .unwrap();
    assert_eq!(state, BusState::Routed, "{:?}", d.bus_state("bus_b"));

    // check(): DRC + LVS clean over the flattened artifact.
    let check = d.check().unwrap();
    assert!(check.clean, "{}", check.json);

    // bake(): settled in the vanilla-accurate engine, states written back,
    // InitialState::Baked stamped into the embedded contract.
    let baked = d.bake(4000).unwrap();
    let contract = baked.embedded_cell_contract().unwrap().unwrap();
    assert!(
        matches!(
            contract.physical.initial_state,
            nucleation::io_contract::InitialState::Baked { .. }
        ),
        "bake stamps the initial state"
    );

    // The artifact is self-describing: executor bound purely through the
    // embedded contract, port NAMES and word VALUES only.
    let extra = nucleation::design::executor_extra_states();
    let extra_refs: Vec<&str> = extra.iter().map(String::as_str).collect();
    let mut cell = BackendCircuitExecutor::for_cell(baked.clone(), &contract, &extra_refs).unwrap();
    cell.settle(4000);

    let read = |cell: &mut BackendCircuitExecutor, name: &str| -> u32 {
        match cell.read_output(name).unwrap() {
            Value::U32(v) => v,
            other => panic!("unexpected value {other:?}"),
        }
    };

    // Walking ones on A with B held at 0 (isolation), then the reverse,
    // then both together.
    for i in 0..8u32 {
        cell.set_input("a_in", &Value::U32(1 << i)).unwrap();
        cell.set_input("b_in", &Value::U32(0)).unwrap();
        cell.settle(400);
        assert_eq!(read(&mut cell, "a_out"), 1 << i, "walkA-{i}");
        assert_eq!(read(&mut cell, "b_out"), 0, "walkA-{i} isolation");
    }
    for i in 0..8u32 {
        cell.set_input("a_in", &Value::U32(0)).unwrap();
        cell.set_input("b_in", &Value::U32(1 << i)).unwrap();
        cell.settle(400);
        assert_eq!(read(&mut cell, "b_out"), 1 << i, "walkB-{i}");
        assert_eq!(read(&mut cell, "a_out"), 0, "walkB-{i} isolation");
    }
    for (pa, pb) in [(0xFFu32, 0xFFu32), (0xAA, 0x55), (0x55, 0xAA)] {
        cell.set_input("a_in", &Value::U32(pa)).unwrap();
        cell.set_input("b_in", &Value::U32(pb)).unwrap();
        cell.settle(400);
        assert_eq!(read(&mut cell, "a_out"), pa, "joint {pa:02x}/{pb:02x}");
        assert_eq!(read(&mut cell, "b_out"), pb, "joint {pa:02x}/{pb:02x}");
    }

    // The in-memory artifact carries one named region per layer.
    let regions = baked.get_region_names();
    assert!(regions.iter().any(|r| r == "bus:bus_a"), "{regions:?}");
    assert!(regions.iter().any(|r| r == "bus:bus_b"), "{regions:?}");

    // The flat `.schem` artifact tier: regions merge (the "PNG"), but the
    // geometry survives and the contract rides embedded — the artifact is
    // still a self-describing cell.
    let bytes = nucleation::formats::schematic::to_schematic(&baked).unwrap();
    let back = nucleation::formats::schematic::from_schematic(&bytes).unwrap();
    let (found, warnings) = back.resolve_cell_contract().unwrap().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(found.name, "crossing");
    let station_dust = back
        .get_block(2, 2, 8)
        .map(|b| b.to_string())
        .unwrap_or_default();
    assert!(
        station_dust.contains("redstone_wire"),
        "bus geometry survives the flat tier, got `{station_dust}`"
    );
}
