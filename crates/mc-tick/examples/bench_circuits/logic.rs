//! Active stationary workloads. Inputs and stepping are timed; fixture creation,
//! settling, checkpoint restoration, assertions and fingerprinting are not.
//!
//! `--verify --logic` is separate from the historical `--expanded` suite.
//! The same source works with master using RUSTFLAGS containing
//! `--cfg bench_master --check-cfg=cfg(bench_master)`; its unsupported
//! hipster_unrecorded case is excluded rather than mislabeled as unrecorded.
use super::*;

pub(super) const CASES: &[&str] = &["adder_active", "logic_network"];
const ADDER_PERIOD: u64 = 96;
const ADDER_INPUTS: &[(u32, u32, bool)] = &[
    (0, 0, false),
    (1, 0, false),
    (0, 1, false),
    (u32::MAX, 0, true),
    (0x5555_5555, 0xaaaa_aaaa, false),
    (0x1234_5678, 0x8765_4321, true),
    (0x8000_0000, 0x8000_0000, false),
    (u32::MAX, 1, false),
];
const NETWORK_PERIOD: u64 = 24;
const NETWORK_PHASES: u64 = 16;
const NETWORK_LANES: i32 = 4;
const GRAY_INPUTS: &[(bool, bool)] = &[(false, false), (true, false), (true, true), (false, true)];

pub(super) fn ticks(case: &str) -> u64 {
    match case {
        "adder_active" => ADDER_PERIOD * ADDER_INPUTS.len() as u64,
        "logic_network" => NETWORK_PERIOD * NETWORK_PHASES,
        _ => unreachable!(),
    }
}

fn drive(sim: &mut Simulation, pos: Pos, powered: bool) {
    // An inert full block is the fixture's original off-input. In particular it
    // retains support for the adder's carry-in sign when carry-in is false.
    let state = sim.registry().get(if powered {
        "minecraft:redstone_block"
    } else {
        "minecraft:light_gray_wool"
    }).unwrap();
    if sim.world().get(pos) != state { sim.place_block(pos, state); }
}

fn adder_inputs(sim: &mut Simulation, a: u32, b: u32, carry: bool) {
    // Low bit at y=3; every following two-block-high slice is the next bit.
    // Place A then B for each low-to-high bit, then carry-in. This order is part
    // of the workload and is identical in every compared binary.
    for bit in 0..32 {
        let y = 3 + 2 * bit;
        drive(sim, Pos::new(13, y, 0), a & (1 << bit) != 0);
        drive(sim, Pos::new(13, y, 4), b & (1 << bit) != 0);
    }
    drive(sim, Pos::new(13, 1, 0), carry);
}

pub(super) fn prepare_adder(sim: &mut Simulation) {
    adder_inputs(sim, 0, 0, false);
    sim.run(200);
    assert!(sim.is_quiescent(), "adder must settle before active input schedule");
    assert_stationary(sim);
}

pub(super) fn prepare_network(sim: &mut Simulation) {
    sim.run(40);
    assert!(sim.is_quiescent(), "logic network must settle before input schedule");
    assert_stationary(sim);
}

fn network_inputs(phase: u64, lane: i32) -> (bool, bool) {
    GRAY_INPUTS[((phase + lane as u64) % GRAY_INPUTS.len() as u64) as usize]
}

pub(super) fn inputs(sim: &mut Simulation, case: &str, tick: u64) {
    match case {
        "adder_active" if tick % ADDER_PERIOD == 0 => {
            let (a, b, carry) = ADDER_INPUTS[(tick / ADDER_PERIOD) as usize];
            adder_inputs(sim, a, b, carry);
        }
        "logic_network" if tick % NETWORK_PERIOD == 0 => {
            for lane in 0..NETWORK_LANES {
                let z = 4 + 10 * lane;
                let (a, b) = network_inputs(tick / NETWORK_PERIOD, lane);
                drive(sim, Pos::new(0, 1, z), a);
                drive(sim, Pos::new(4, 1, z - 4), b);
            }
        }
        _ => {}
    }
}

// Tick-end state from the existing ordinary change log, with no extra observer
// or per-tick assertions in the timed path. Change tick N precedes tick_count N+1.
fn state_at(sim: &Simulation, pos: Pos, end_tick: u64) -> StateId {
    sim.recorded().iter().rev().find(|r| r.pos == pos && r.tick < end_tick)
        .map(|r| r.to)
        .or_else(|| sim.recorded().iter().find(|r| r.pos == pos).map(|r| r.from))
        .unwrap_or_else(|| sim.world().get(pos))
}

fn property<'a>(descriptor: &'a str, name: &str) -> &'a str {
    descriptor.split_once('[').unwrap().1.trim_end_matches(']').split(',')
        .find_map(|p| p.split_once('=').filter(|(k, _)| *k == name).map(|(_, v)| v))
        .unwrap_or_else(|| panic!("missing {name} in {descriptor}"))
}

fn assert_stationary(sim: &Simulation) {
    assert!(sim.moving_blocks().is_empty(), "stationary benchmark has moving blocks");
    assert!(sim.item_entities().is_empty() && sim.minecarts().is_empty()
        && sim.entity_bodies().is_empty(), "stationary benchmark has entities");
    assert!(sim.recorded_entities().is_empty(), "stationary benchmark produced entity events");
    assert!(sim.recorded_inventory().is_empty(), "stationary benchmark changed inventory");
    for (_, state) in sim.world().iter_non_air() {
        let d = sim.registry().descriptor(state).unwrap();
        assert!(!d.contains("piston"), "stationary fixture must contain no pistons: {d}");
    }
    // Check historical changes as well: absence only at the final tick would
    // fail to detect transient piston movement or an entity that was removed.
    for change in sim.recorded() {
        for state in [change.from, change.to] {
            assert!(!sim.registry().descriptor(state).unwrap().contains("piston"));
        }
    }
}

pub(super) fn validate(sim: &Simulation, case: &str) {
    assert_stationary(sim);
    let start = sim.tick_count() - ticks(case);
    match case {
        "adder_active" => {
            let mut transitioned = 0u32;
            for (phase, &(a, b, carry)) in ADDER_INPUTS.iter().enumerate() {
                let end = start + (phase as u64 + 1) * ADDER_PERIOD;
                let mut actual = 0u32;
                for bit in 0..32 {
                    let pos = Pos::new(0, 3 + 2 * bit, 2);
                    let d = sim.registry().descriptor(state_at(sim, pos, end)).unwrap();
                    assert!(d.starts_with("minecraft:repeater["), "adder sum port {pos:?}");
                    if property(d, "powered") == "true" { actual |= 1 << bit; }
                    if sim.recorded().iter().any(|r| r.pos == pos) { transitioned |= 1 << bit; }
                }
                let expected = a.wrapping_add(b).wrapping_add(u32::from(carry));
                assert_eq!(actual, expected,
                    "adder phase {phase}: {a:#010x} + {b:#010x} + {} at tick {end}", u8::from(carry));
            }
            assert_eq!(transitioned, u32::MAX, "every sum bit must actually change");
            assert!(sim.recorded().iter().any(|r| sim.registry().descriptor(r.to).unwrap()
                .starts_with("minecraft:comparator[")), "active adder must exercise comparator state changes");
        }
        "logic_network" => {
            for phase in 0..NETWORK_PHASES {
                let end = start + (phase + 1) * NETWORK_PERIOD;
                for lane in 0..NETWORK_LANES {
                    let z = 4 + 10 * lane;
                    let (a, b) = network_inputs(phase, lane);
                    // Main input has two wire cells after the first repeater:
                    // strengths 15,14. The side input has three: 15,14,13.
                    // Alternate compare/subtract lanes exercise distinct exact
                    // comparator strengths even when the final boolean is equal.
                    let main: u8 = if a { 14 } else { 0 };
                    let side: u8 = if b { 13 } else { 0 };
                    let expected = if lane % 2 == 0 {
                        if main >= side { main } else { 0 }
                    } else { main.saturating_sub(side) };
                    let wire = sim.registry().descriptor(state_at(sim, Pos::new(5, 1, z), end)).unwrap();
                    assert!(wire.starts_with("minecraft:redstone_wire["));
                    assert_eq!(property(wire, "power").parse::<u8>().unwrap(), expected,
                        "logic phase {phase}, lane {lane}: exact comparator output");
                    let lamp = sim.registry().descriptor(state_at(sim, Pos::new(7, 1, z), end)).unwrap();
                    assert_eq!(property(lamp, "lit") == "true", expected > 0,
                        "logic phase {phase}, lane {lane}: repeater/lamp output");
                }
            }
            for lane in 0..NETWORK_LANES {
                let pos = Pos::new(7, 1, 4 + 10 * lane);
                assert!(sim.recorded().iter().filter(|r| r.pos == pos).count() >= 6,
                    "each lane must repeatedly change its output");
            }
        }
        _ => unreachable!(),
    }
}

pub(super) fn hash_tick(hash: &mut blake3::Hasher, sim: &Simulation) {
    // Comparator block entities can change strength while retaining the same
    // powered block state. Include those values in the new differential suite.
    let mut outputs: Vec<_> = sim.comparator_outputs().iter().collect();
    outputs.sort_by_key(|(p, _)| (p.x, p.y, p.z));
    for (pos, strength) in outputs {
        for coordinate in [pos.x, pos.y, pos.z] { hash.update(&coordinate.to_le_bytes()); }
        hash.update(&[*strength]);
    }
}

pub(super) fn network_fixture() -> String {
    // Authored fixture; no external schematic download or licensing dependency.
    // Four separated lanes combine delayed repeaters, two wire input strengths,
    // compare/subtract modes, output repeaters and lamps. No BEs with inventory,
    // moving parts, random ticks or entities. The source generator is part of the
    // shared harness, so master and candidate construct identical registries.
    let mut palette = vec![
        "{Name:\"minecraft:stone\"}".to_owned(),
        "{Name:\"minecraft:light_gray_wool\"}".to_owned(),
        "{Name:\"minecraft:redstone_wire\",Properties:{east:\"side\",north:\"side\",power:\"0\",south:\"side\",west:\"side\"}}".to_owned(),
        "{Name:\"minecraft:redstone_lamp\",Properties:{lit:\"false\"}}".to_owned(),
        "{Name:\"minecraft:comparator\",Properties:{facing:\"west\",mode:\"compare\",powered:\"false\"}}".to_owned(),
        "{Name:\"minecraft:comparator\",Properties:{facing:\"west\",mode:\"subtract\",powered:\"false\"}}".to_owned(),
    ];
    for delay in 1..=4 {
        palette.push(format!("{{Name:\"minecraft:repeater\",Properties:{{delay:\"{delay}\",facing:\"west\",locked:\"false\",powered:\"false\"}}}}"));
    }
    let mut blocks = Vec::new();
    let mut add = |x, y, z, state| blocks.push(format!("{{pos:[{x},{y},{z}],state:{state}}}"));
    for lane in 0..NETWORK_LANES {
        let z = 4 + 10 * lane;
        for x in 0..=7 { for dz in -4..=1 { add(x, 0, z + dz, 0); } }
        add(0, 1, z, 1);
        add(1, 1, z, 6 + lane);
        add(2, 1, z, 2);
        add(3, 1, z, 2);
        add(4, 1, z, 4 + lane % 2);
        add(5, 1, z, 2);
        add(6, 1, z, 9 - lane);
        add(7, 1, z, 3);
        add(4, 1, z - 4, 1);
        for dz in -3..=-1 { add(4, 1, z + dz, 2); }
    }
    format!("{{DataVersion:3700,size:[8,2,36],palette:[{}],blocks:[{}],entities:[]}}",
        palette.join(","), blocks.join(","))
}
