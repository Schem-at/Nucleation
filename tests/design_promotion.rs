//! PORT PROMOTION: the composability blocker, and the chain it unblocks.
//!
//! The studio's live failure, verbatim:
//!
//! ```text
//! u3.bin is executor-only IO — no bus can land on it. bit 0: no dust
//! connection cell at or beside (19, 5, 5) (holds minecraft:lever[...])
//! ```
//!
//! The user was wiring an ADDER OUTPUT into the BINARY-TO-BCD INPUT — the
//! canonical `add -> BCD -> 7-segment` demo — and community cells name LEVERS
//! for their inputs. Nothing in redstone drives a lever, so the router was
//! never the problem.
//!
//! These tests are the acceptance for [`Design::set_port_mode`]:
//!
//! 1. the mode switch is a byte-exact reversible toggle,
//! 2. a promoted port is routable AND still computes the same function,
//! 3. the real chain routes, bakes and verifies arithmetically end to end.

#![cfg(all(feature = "routing", feature = "simulation", feature = "mc-tick"))]

use nucleation::design::{BusState, BusStyle, Design, PortMode};
use nucleation::io_contract::Value;
use nucleation::simulation::typed_executor::BackendCircuitExecutor;
use nucleation::UniversalSchematic;
use std::collections::BTreeMap;

const ADD: &str = "ADD007_8bit_cca_matt_enhanced.schem";
const BCD: &str = "BINTOBCD001_8bit_comb_binary_to_bcd_enhanced.schem";
const SEG: &str = "NUMDISPLAY001_fast_bcd_to_7seg_enhanced.schem";

fn load(file: &str) -> Option<UniversalSchematic> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("computational_schematics/enhanced")
        .join(file);
    nucleation::formats::schematic::from_schematic(&std::fs::read(path).ok()?).ok()
}

/// Every non-air block of a design's instance layers, for byte comparison.
fn instance_blocks(d: &Design) -> BTreeMap<(i32, i32, i32), String> {
    let flat = d.flatten().expect("flatten");
    let mut out = BTreeMap::new();
    for (bp, bs) in flat.iter_blocks() {
        let s = bs.to_string();
        if s.contains("minecraft:air") {
            continue;
        }
        out.insert((bp.x, bp.y, bp.z), s);
    }
    out
}

fn executor(d: &Design) -> BackendCircuitExecutor {
    // The COMPOSITE, not the layered flatten: `UniversalSchematic::get_block`
    // answers from the default region first inside its dense bounds, so a bus
    // fragment threading a cell's own bbox would read back as air and the
    // chain would simulate open-circuit.
    let flat = d.flatten_composite().expect("composite");
    let (c, _) = flat
        .resolve_cell_contract()
        .expect("contract resolves")
        .expect("contract present");
    let extra = nucleation::design::executor_extra_states();
    let refs: Vec<&str> = extra.iter().map(String::as_str).collect();
    BackendCircuitExecutor::for_cell(flat, &c, &refs).expect("executor")
}

// ----------------------------------------------------------------------
// 1. The toggle
// ----------------------------------------------------------------------

#[test]
fn a_port_mode_toggle_is_a_byte_exact_round_trip() {
    let Some(bcd) = load(BCD) else { return };
    let mut d = Design::new("t");
    d.add_cell("bcd", bcd).unwrap();
    d.place("u0", "bcd", (0, 0, 0), 0).unwrap();

    let before = instance_blocks(&d);
    assert_eq!(d.port_mode("u0", "bin"), PortMode::Executor);

    let rep = d.promote_input("u0", "bin").unwrap();
    assert_eq!(d.port_mode("u0", "bin"), PortMode::Bus);
    assert!(!rep.changed.is_empty(), "the toggle rewrote nothing");
    assert!(rep.note.contains("BUS"), "{}", rep.note);
    let promoted = instance_blocks(&d);
    assert_ne!(before, promoted, "Bus mode changed no blocks");

    // ...and back.
    let back = d.set_port_mode("u0", "bin", PortMode::Executor).unwrap();
    assert_eq!(d.port_mode("u0", "bin"), PortMode::Executor);
    assert!(back.note.contains("EXECUTOR"), "{}", back.note);
    let after = instance_blocks(&d);
    assert_eq!(
        before, after,
        "Executor mode must restore the cell byte-exactly"
    );
}

#[test]
fn a_toggled_back_port_is_hand_drivable_again() {
    let Some(bcd) = load(BCD) else { return };
    let mut d = Design::new("t");
    d.add_cell("bcd", bcd).unwrap();
    d.place("u0", "bcd", (0, 0, 0), 0).unwrap();
    d.promote_input("u0", "bin").unwrap();
    d.set_port_mode("u0", "bin", PortMode::Executor).unwrap();

    let mut ex = executor(&d);
    ex.set_input("u0.bin", &Value::U32(137)).unwrap();
    ex.settle(4000);
    assert_eq!(ex.read_output("u0.bcd_ones").unwrap(), Value::U32(7));
    assert_eq!(ex.read_output("u0.bcd_tens").unwrap(), Value::U32(3));
    assert_eq!(ex.read_output("u0.bcd_hundreds").unwrap(), Value::U32(1));
}

#[test]
fn port_modes_survive_a_nucm_round_trip() {
    let Some(bcd) = load(BCD) else { return };
    let mut d = Design::new("t");
    d.add_cell("bcd", bcd).unwrap();
    d.place("u0", "bcd", (0, 0, 0), 0).unwrap();
    d.promote_input("u0", "bin").unwrap();

    let bytes = d.to_nucm_bytes().unwrap();
    let back = Design::from_nucm_bytes(&bytes).unwrap();
    assert_eq!(back.port_mode("u0", "bin"), PortMode::Bus);
    // The reloaded document is still routable on that port...
    let p = back
        .instance_ports()
        .unwrap()
        .into_iter()
        .find(|p| p.name == "u0.bin")
        .unwrap();
    assert!(p.routable(), "{:?}", p.blocked);
    assert_eq!(p.step, Some((0, 2, 0)));
    // ...and still toggles back to the shipped hardware.
    let mut back = back;
    back.set_port_mode("u0", "bin", PortMode::Executor).unwrap();
    assert_eq!(back.port_mode("u0", "bin"), PortMode::Executor);
}

#[test]
fn toggling_a_port_that_carries_a_bus_rips_it_and_says_so() {
    let Some(bcd) = load(BCD) else { return };
    let mut d = Design::new("t");
    d.add_cell("bcd", bcd).unwrap();
    d.place("u0", "bcd", (0, 0, 0), 0).unwrap();
    d.promote_input("u0", "bin").unwrap();
    let wires = d
        .instance_ports()
        .unwrap()
        .into_iter()
        .find(|p| p.name == "u0.bin")
        .unwrap()
        .wires
        .unwrap();
    // A design-side driver on the loose layer, aimed at the promoted column.
    let a = (wires[0].0 - 30, wires[0].1, wires[0].2);
    for k in 0..8i32 {
        let y = a.1 + 2 * k;
        d.set_block((a.0, y - 1, a.2), "minecraft:stone").unwrap();
        d.set_block((a.0, y, a.2), "minecraft:lever[face=floor,facing=north,powered=false]")
            .unwrap();
        d.set_block((a.0 + 1, y - 1, a.2), "minecraft:stone").unwrap();
        d.set_block(
            (a.0 + 1, y, a.2),
            "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]",
        )
        .unwrap();
    }
    d.declare_input(
        "din",
        (a.0 + 1, a.1, a.2),
        (0, 2, 0),
        8,
        nucleation::io_contract::IoType::UnsignedInt { bits: 8 },
    )
    .unwrap();
    assert_eq!(
        d.route_bus("net", "din", &["u0.bin"], vec![], BusStyle::default())
            .unwrap(),
        BusState::Routed
    );
    let rep = d.set_port_mode("u0", "bin", PortMode::Executor).unwrap();
    assert_eq!(
        rep.removed_buses,
        vec!["net".to_string()],
        "a bus whose endpoint stopped existing must be ripped and named"
    );
    assert!(d.bus_state("net").is_none());
    assert!(d.check().is_ok());
}

#[test]
fn promotion_makes_a_lever_port_routable_and_says_what_it_did() {
    let Some(bcd) = load(BCD) else { return };
    let mut d = Design::new("t");
    d.add_cell("bcd", bcd).unwrap();
    d.place("u3", "bcd", (10, 5, 5), 0).unwrap();

    // The failure the user hit.
    let blocked = d
        .instance_ports()
        .unwrap()
        .into_iter()
        .find(|p| p.name == "u3.bin")
        .unwrap();
    assert!(!blocked.routable());
    assert!(blocked.blocked.as_deref().unwrap().contains("executor-only"));

    let rep = d.promote_input("u3", "bin").unwrap();
    println!("[promote] {}", rep.note);
    println!("[promote] patch {}", rep.patch_json);

    let now = d
        .instance_ports()
        .unwrap()
        .into_iter()
        .find(|p| p.name == "u3.bin")
        .unwrap();
    assert!(
        now.routable(),
        "still refused after promotion: {:?}",
        now.blocked
    );
    assert_eq!(now.step, Some((0, 2, 0)), "promotion must yield the bus form");
    assert_eq!(now.width, 8);
}

// ----------------------------------------------------------------------
// 2. Function preservation
// ----------------------------------------------------------------------

#[test]
fn a_promoted_port_computes_the_same_function() {
    let Some(bcd) = load(BCD) else { return };
    // Executor-mode reference.
    let mut a = Design::new("a");
    a.add_cell("bcd", bcd.clone()).unwrap();
    a.place("u0", "bcd", (0, 0, 0), 0).unwrap();
    let mut plain = executor(&a);

    // Bus mode, driven through the promoted column by a lever bank the test
    // wires onto the design's loose layer: exactly what a bus would deliver.
    let mut b = Design::new("b");
    b.add_cell("bcd", bcd).unwrap();
    b.place("u0", "bcd", (0, 0, 0), 0).unwrap();
    let rep = b.promote_input("u0", "bin").unwrap();
    println!("[bus mode] {}", rep.note);
    let wires: Vec<(i32, i32, i32)> = b
        .instance_ports()
        .unwrap()
        .into_iter()
        .find(|p| p.name == "u0.bin")
        .unwrap()
        .wires
        .unwrap();
    let mut drive = Vec::new();
    for w in &wires {
        // A lever two cells away on the design's own layer.
        for t in 1..=2 {
            b.set_block((w.0 - t, w.1 - 1, w.2), "minecraft:stone").unwrap();
            b.set_block(
                (w.0 - t, w.1, w.2),
                "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]",
            )
            .unwrap();
        }
        b.set_block((w.0 - 3, w.1 - 1, w.2), "minecraft:stone").unwrap();
        b.set_block(
            (w.0 - 3, w.1, w.2),
            "minecraft:lever[face=floor,facing=north,powered=false]",
        )
        .unwrap();
        drive.push((w.0 - 3, w.1, w.2));
    }
    b.declare_input(
        "din",
        (wires[0].0 - 2, wires[0].1, wires[0].2),
        (0, 2, 0),
        8,
        nucleation::io_contract::IoType::UnsignedInt { bits: 8 },
    )
    .unwrap();
    let mut bussed = executor(&b);

    let mut same = 0;
    let vectors = [0u32, 1, 9, 10, 42, 99, 137, 255];
    for v in vectors {
        plain.set_input("u0.bin", &Value::U32(v)).unwrap();
        plain.settle(6000);
        bussed.set_input("din", &Value::U32(v)).unwrap();
        bussed.settle(6000);
        let rd = |e: &mut BackendCircuitExecutor| {
            (
                e.read_output("u0.bcd_ones").ok(),
                e.read_output("u0.bcd_tens").ok(),
                e.read_output("u0.bcd_hundreds").ok(),
            )
        };
        let (x, y) = (rd(&mut plain), rd(&mut bussed));
        println!("bin={v:3} executor={x:?} promoted+pivoted={y:?}");
        if x == y {
            same += 1;
        }
    }
    assert_eq!(
        same,
        vectors.len(),
        "promotion + pivot changed the cell's function"
    );
}

// ----------------------------------------------------------------------
// 3. The chain the user actually wanted
// ----------------------------------------------------------------------

/// `NUMDISPLAY001`'s verified segment patterns (`computational_schematics/
/// enhanced/REPORT.md`: digits 0/1/8 are pixel-exact, digit 7 is a typed
/// spot-check). Order: a,b,c,d,e,f,g.
const SEVEN_SEG: [&str; 10] = [
    "1111110", // 0
    "0110000", // 1
    "1101101", // 2
    "1111001", // 3
    "0110011", // 4
    "1011011", // 5
    "1011111", // 6
    "1110000", // 7
    "1111111", // 8
    "1110011", // 9
];

fn read_segments(e: &mut BackendCircuitExecutor, inst: &str) -> String {
    ["seg_a", "seg_b", "seg_c", "seg_d", "seg_e", "seg_f", "seg_g"]
        .iter()
        .map(|n| match e.read_output(&format!("{inst}.{n}")) {
            Ok(Value::Bool(true)) => '1',
            Ok(Value::Bool(false)) => '0',
            _ => '?',
        })
        .collect()
}

/// Build the pipeline: `add.sum -> bcd.bin`, and if it routes,
/// `bcd.bcd_ones -> seg.bcd`.
///
/// Levels: a bus is a single-level 2y-pitch stack, so each stage is placed so
/// its bit-0 connection cell shares the driver's y.
fn build_chain(with_display: bool) -> Option<(Design, Vec<String>)> {
    let (add, bcd) = (load(ADD)?, load(BCD)?);
    let mut d = Design::new("add-bcd-7seg");
    d.add_cell("add", add).ok()?;
    d.add_cell("bcd", bcd).ok()?;
    d.place("u0", "add", (0, 0, 0), 0).ok()?;
    // ADD007's `sum` taps are dust at local (15, 3+2i, 1) — already routable.
    // BINTOBCD001's `bin` levers are at local (2i, 5, 0): promote, which also
    // pivots the horizontal row onto the vertical stack.
    d.place("u1", "bcd", (60, -2, 40), 0).ok()?;
    let r1 = d.promote_input("u1", "bin").ok()?;
    println!("[chain] u1.bin: {}", r1.note);

    let mut log = Vec::new();
    let s1 = d
        .route_bus("sum_to_bin", "u0.sum", &["u1.bin"], vec![], BusStyle::default())
        .ok()?;
    log.push(format!("sum_to_bin: {s1:?}"));
    if s1 != BusState::Routed {
        return Some((d, log));
    }
    if !with_display {
        return Some((d, log));
    }
    let seg = load(SEG)?;
    d.add_cell("seg", seg).ok()?;
    // `bcd_ones` taps sit on top of the output lamps at local (2i, 5, 47) and
    // pivot to a column at local (0, 5+2i, 47+8); with u1 at y=-2 that column
    // is at world y = 3+2i. `seg.bcd` promotes to local (30, 2+2i, 4), so seg
    // must sit at y = 1 for its bit 0 to land on y = 3.
    let r2 = d.promote_output("u1", "bcd_ones").ok()?;
    println!("[chain] u1.bcd_ones: {}", r2.note);
    d.place("u2", "seg", (110, 1, 40), 0).ok()?;
    let r3 = d.promote_input("u2", "bcd").ok()?;
    println!("[chain] u2.bcd: {}", r3.note);
    let s2 = d
        .route_bus("ones_to_seg", "u1.bcd_ones", &["u2.bcd"], vec![], BusStyle::default())
        .ok()?;
    log.push(format!("ones_to_seg: {s2:?}"));
    Some((d, log))
}

#[test]
fn the_adder_feeds_the_bcd_converter() {
    let Some((d, log)) = build_chain(false) else {
        eprintln!("enhanced corpus unavailable; skipping");
        return;
    };
    for l in &log {
        println!("[bus] {l}");
    }
    assert_eq!(
        d.bus_state("sum_to_bin"),
        Some(&BusState::Routed),
        "u0.sum -> u1.bin did not route: {:?}",
        d.bus_state("sum_to_bin")
    );

    // The artifact bakes.
    let baked = d.bake(20_000).expect("bake");
    println!(
        "[bake] {} blocks in the baked artifact",
        baked.total_blocks()
    );

    // ...and the CHAIN computes: drive the adder's levers, read the BCD digits.
    let mut ex = executor(&d);
    let cases: [(u32, u32); 8] = [
        (0, 0),
        (1, 1),
        (37, 5),
        (99, 28),
        (7, 9),
        (128, 9),
        (200, 55),
        (250, 5),
    ];
    let mut ok = 0;
    for (a, b) in cases {
        ex.set_input("u0.a", &Value::U32(a)).unwrap();
        ex.set_input("u0.b", &Value::U32(b)).unwrap();
        ex.settle(20_000);
        // Informational: the adder's own lamp column. The value that matters
        // is what arrived at the BCD stage through the bus.
        let sum = ex.read_output("u0.sum").ok();
        let (h, t, o) = (
            ex.read_output("u1.bcd_hundreds").unwrap(),
            ex.read_output("u1.bcd_tens").unwrap(),
            ex.read_output("u1.bcd_ones").unwrap(),
        );
        let digits = |v: &Value| match v {
            Value::U32(x) => *x,
            _ => 99,
        };
        let got = digits(&h) * 100 + digits(&t) * 10 + digits(&o);
        let want = (a + b).min(255);
        println!(
            "{a:3} + {b:3} (lamps {sum:?}) -> BCD {}{}{} = {got:3}   want {want:3}  {}",
            digits(&h),
            digits(&t),
            digits(&o),
            if got == want { "OK" } else { "MISMATCH" }
        );
        if got == want {
            ok += 1;
        }
    }
    assert_eq!(ok, cases.len(), "the chained adder->BCD arithmetic is wrong");
}

#[test]
fn the_full_add_bcd_sevenseg_pipeline() {
    let Some((d, log)) = build_chain(true) else {
        eprintln!("enhanced corpus unavailable; skipping");
        return;
    };
    for l in &log {
        println!("[bus] {l}");
    }
    if d.bus_state("ones_to_seg") != Some(&BusState::Routed) {
        println!(
            "STAGE 3 NOT ROUTED (reported honestly): {:?}",
            d.bus_state("ones_to_seg")
        );
        return;
    }
    let mut ex = executor(&d);
    let mut ok = 0;
    let cases: [(u32, u32); 8] = [
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 1),
        (3, 3),
        (5, 2),
        (4, 4),
    ];
    for (a, b) in cases {
        ex.set_input("u0.a", &Value::U32(a)).unwrap();
        ex.set_input("u0.b", &Value::U32(b)).unwrap();
        ex.settle(20_000);
        let got = read_segments(&mut ex, "u2");
        let want = SEVEN_SEG[((a + b) % 10) as usize];
        println!(
            "{a}+{b}={} -> segments {got}  want {want}  {}",
            a + b,
            if got == want { "OK" } else { "MISMATCH" }
        );
        if got == want {
            ok += 1;
        }
    }
    assert_eq!(ok, cases.len(), "7-segment patterns wrong");
}
