//! Typed (in-sim) verification of the Phase 2 Lane A routes: fanout
//! trunks and wired-OR merges, driven purely through the EMBEDDED
//! contract — port names and word values, no coordinates.

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

const N: u8 = 4;

fn lever_bank(s: &mut UniversalSchematic, x: i32, z: i32, dx: i32, dz: i32) -> (i32, i32, i32) {
    for i in 0..N as i32 {
        let y = 2 + 2 * i;
        s.set_block_from_string(x, y - 1, z, STONE).unwrap();
        s.set_block_from_string(x, y, z, LEVER).unwrap();
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE).unwrap();
        s.set_block_from_string(x + dx, y, z + dz, DUST).unwrap();
    }
    (x + dx, 2, z + dz)
}

fn lamp_bank(s: &mut UniversalSchematic, x: i32, z: i32) -> (i32, i32, i32) {
    for i in 0..N as i32 {
        let y = 2 + 2 * i;
        s.set_block_from_string(x, y - 1, z, LAMP).unwrap();
        s.set_block_from_string(x, y, z, DUST).unwrap();
    }
    (x, 2, z)
}

fn executor_for(d: &Design) -> BackendCircuitExecutor {
    let baked = d.bake(4000).unwrap();
    let contract = baked.embedded_cell_contract().unwrap().unwrap();
    let extra = nucleation::design::executor_extra_states();
    let extra_refs: Vec<&str> = extra.iter().map(String::as_str).collect();
    let mut cell = BackendCircuitExecutor::for_cell(baked, &contract, &extra_refs).unwrap();
    cell.settle(4000);
    cell
}

fn read_u32(cell: &mut BackendCircuitExecutor, name: &str) -> u32 {
    match cell.read_output(name).unwrap() {
        Value::U32(v) => v,
        other => panic!("unexpected value {other:?}"),
    }
}

/// Fanout: one driver, two sinks — every pattern lands on BOTH sinks.
#[test]
fn fanout_delivers_to_both_sinks_in_sim() {
    let mut s = UniversalSchematic::new("fanout".to_string());
    let a_in = lever_bank(&mut s, 0, 8, 1, 0);
    let a_out = lamp_bank(&mut s, 16, 8);
    let c_out = lamp_bank(&mut s, 8, 16);
    let mut d = Design::for_schematic("fanout", s);
    let ty = IoType::UnsignedInt { bits: N as usize };
    d.declare_input("a_in", a_in, (0, 2, 0), N, ty.clone()).unwrap();
    d.declare_output("a_out", a_out, (0, 2, 0), N, ty.clone()).unwrap();
    d.declare_output("c_out", c_out, (0, 2, 0), N, ty).unwrap();
    let state = d
        .route_bus("fan", "a_in", &["a_out", "c_out"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed, "{:?}", d.bus_state("fan"));

    let mut cell = executor_for(&d);
    for i in 0..N as u32 {
        cell.set_input("a_in", &Value::U32(1 << i)).unwrap();
        cell.settle(400);
        assert_eq!(read_u32(&mut cell, "a_out"), 1 << i, "trunk sink, walk {i}");
        assert_eq!(read_u32(&mut cell, "c_out"), 1 << i, "branch sink, walk {i}");
    }
    cell.set_input("a_in", &Value::U32(0)).unwrap();
    cell.settle(400);
    assert_eq!(read_u32(&mut cell, "a_out"), 0);
    assert_eq!(read_u32(&mut cell, "c_out"), 0);
}

/// Wired-OR: the sink reads a | b for every driven pair.
#[test]
fn wired_or_reads_the_or_of_both_drivers_in_sim() {
    let mut s = UniversalSchematic::new("wor".to_string());
    let a_in = lever_bank(&mut s, 0, 8, 1, 0);
    let b_in = lever_bank(&mut s, 8, 0, 0, 1);
    let a_out = lamp_bank(&mut s, 16, 8);
    let mut d = Design::for_schematic("wor", s);
    let ty = IoType::UnsignedInt { bits: N as usize };
    d.declare_input("a_in", a_in, (0, 2, 0), N, ty.clone()).unwrap();
    d.declare_input("b_in", b_in, (0, 2, 0), N, ty.clone()).unwrap();
    d.declare_output("a_out", a_out, (0, 2, 0), N, ty).unwrap();
    let state = d
        .route_bus_or("wor", &["a_in", "b_in"], &["a_out"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed, "{:?}", d.bus_state("wor"));

    let mut cell = executor_for(&d);
    for (a, b) in [(0u32, 0u32), (0x5, 0x2), (0x2, 0x5), (0xF, 0x0), (0x0, 0xF), (0x9, 0x6)] {
        cell.set_input("a_in", &Value::U32(a)).unwrap();
        cell.set_input("b_in", &Value::U32(b)).unwrap();
        cell.settle(400);
        assert_eq!(read_u32(&mut cell, "a_out"), a | b, "a={a:x} b={b:x}");
    }
}

/// Sketch (2) end to end in-sim: conduction verified after the gate drag.
#[test]
fn gate_drag_conducts_in_sim() {
    let mut s = UniversalSchematic::new("drag".to_string());
    let a_in = lever_bank(&mut s, 0, 8, 1, 0);
    let a_out = lamp_bank(&mut s, 24, 8);
    let mut d = Design::for_schematic("drag", s);
    let ty = IoType::UnsignedInt { bits: N as usize };
    d.declare_input("a_in", a_in, (0, 2, 0), N, ty.clone()).unwrap();
    d.declare_output("a_out", a_out, (0, 2, 0), N, ty).unwrap();
    let gates = vec![
        nucleation::design::Gate {
            name: "g0".to_string(),
            anchor: (8, 2, 8),
            step: (0, 2, 0),
        },
        nucleation::design::Gate {
            name: "g1".to_string(),
            anchor: (16, 2, 8),
            step: (0, 2, 0),
        },
    ];
    let state = d
        .route_bus("bus_a", "a_in", &["a_out"], gates, BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed);
    let report = d.move_gate("bus_a", "g0", (8, 2, 12)).unwrap();
    assert_eq!(report.state, BusState::Routed, "{:?}", report.state);
    assert_eq!(report.rerouted_segments, 2);

    let mut cell = executor_for(&d);
    for i in 0..N as u32 {
        cell.set_input("a_in", &Value::U32(1 << i)).unwrap();
        cell.settle(400);
        assert_eq!(read_u32(&mut cell, "a_out"), 1 << i, "walk {i} after drag");
    }
}
