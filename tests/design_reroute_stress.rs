//! RE-ROUTING STRESS: the invariants a drag must never break.
//!
//! User report: "sometimes when I move a component the bus doesn't update
//! right." The studio redraws only what the engine reports as changed (that is
//! what took drag from 431 ms to 0.05 ms/frame), so an engine that is right
//! internally but under-reports shows STALE GEOMETRY. This file makes the
//! report the source of truth and pins it.
//!
//! After EVERY operation:
//!   (a) `check()` is DRC + LVS clean;
//!   (b) no orphaned cells — a rip+reroute cycle returns the exact same block
//!       count, and a rip leaves nothing behind;
//!   (c) no stale geometry — every routed bus still MEETS its current port
//!       anchors, so nothing stays wired to where a port used to be;
//!   (d) the reported changed set is COMPLETE — it is compared against an
//!       independent block-by-block diff computed in this test.

#![cfg(feature = "routing")]

use nucleation::design::{BusState, BusStyle, Design, Gate};
use nucleation::io_contract::IoType;
use nucleation::UniversalSchematic;
use std::collections::{BTreeMap, BTreeSet};

type P3 = (i32, i32, i32);

const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";

const W: u8 = 4;

fn ty() -> IoType {
    IoType::UnsignedInt { bits: W as usize }
}

fn lever_bank(s: &mut UniversalSchematic, x: i32, y0: i32, z: i32, dx: i32, dz: i32) -> P3 {
    for i in 0..W as i32 {
        let y = y0 + 2 * i;
        s.set_block_from_string(x, y - 1, z, STONE).unwrap();
        s.set_block_from_string(x, y, z, LEVER).unwrap();
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE)
            .unwrap();
        s.set_block_from_string(x + dx, y, z + dz, DUST).unwrap();
    }
    (x + dx, y0, z + dz)
}

fn lamp_bank(s: &mut UniversalSchematic, x: i32, y0: i32, z: i32) -> P3 {
    for i in 0..W as i32 {
        let y = y0 + 2 * i;
        s.set_block_from_string(x, y - 1, z, LAMP).unwrap();
        s.set_block_from_string(x, y, z, DUST).unwrap();
    }
    (x, y0, z)
}

/// A community-shaped library cell: LEVER inputs, LAMP outputs (so the buses
/// on either side are genuinely different nets), with an explicit keepout
/// TIGHTER than the default grow-by-1 — it covers the body only, so a bus's
/// first cell sits OUTSIDE the halo. That is the shape that hid the
/// stale-geometry bug: a fragment-intersects-region test misses a bus wired
/// straight into the cell.
fn wire_cell() -> UniversalSchematic {
    use nucleation::io_contract::{CellContract, IoLayoutBuilder, LayoutFunction};
    let mut s = UniversalSchematic::new("wire".to_string());
    let d_hw: Vec<P3> = (0..W as i32).map(|i| (0, 2 + 2 * i, 0)).collect();
    let q_hw: Vec<P3> = (0..W as i32).map(|i| (4, 1 + 2 * i, 0)).collect();
    for i in 0..W as i32 {
        let y = 2 + 2 * i;
        s.set_block_from_string(0, y - 1, 0, STONE).unwrap();
        s.set_block_from_string(0, y, 0, LEVER).unwrap();
        s.set_block_from_string(1, y - 1, 0, STONE).unwrap();
        s.set_block_from_string(1, y, 0, DUST).unwrap();
        s.set_block_from_string(4, y - 1, 0, LAMP).unwrap();
        s.set_block_from_string(4, y, 0, DUST).unwrap();
    }
    let layout = IoLayoutBuilder::new()
        .add_input("d".to_string(), ty(), LayoutFunction::OneToOne, d_hw)
        .unwrap()
        .add_output("q".to_string(), ty(), LayoutFunction::OneToOne, q_hw)
        .unwrap()
        .build();
    let mut c = CellContract::new("wire".to_string(), layout);
    c.physical.keepouts = vec![nucleation::BoundingBox {
        min: (0, 1, 0),
        max: (4, 2 + 2 * (W as i32 - 1), 0),
    }];
    s.set_cell_contract(&c).unwrap();
    s
}

// ----------------------------------------------------------------------
// The invariant battery
// ----------------------------------------------------------------------

/// The realized geometry of every bus layer.
fn geometry(d: &Design) -> BTreeMap<String, BTreeMap<P3, String>> {
    d.bus_geometry()
}

fn total_bus_cells(d: &Design) -> usize {
    geometry(d).values().map(|f| f.len()).sum()
}

/// Gating DRC violation count (`drc` only; `cells` is informational).
fn violations(json: &str) -> usize {
    let Some(start) = json.find("\"drc\":[") else {
        return 0;
    };
    let rest = &json[start + 7..];
    let end = rest.find("],\"cells\"").unwrap_or(rest.len());
    rest[..end].matches("\"kind\"").count()
}

/// (a) DRC + LVS clean.
fn assert_clean(d: &Design, what: &str) {
    assert_every_routed_bus_built_something(d, what);
    let c = d
        .check()
        .unwrap_or_else(|e| panic!("{what}: check() failed: {e}"));
    assert!(
        c.clean,
        "{what}: NOT DRC/LVS clean ({} gating violation(s)): {}",
        violations(&c.json),
        c.json
    );
}

/// (d) The reported changed set must cover every layer whose blocks differ.
///
/// Over-reporting is allowed (redrawing an unchanged layer is wasted work,
/// never a wrong picture); UNDER-reporting is the bug.
fn assert_changed_set_complete(
    reported: &[String],
    before: &BTreeMap<String, BTreeMap<P3, String>>,
    after: &BTreeMap<String, BTreeMap<P3, String>>,
    what: &str,
) {
    let mut actually: BTreeSet<&String> = BTreeSet::new();
    for (n, frag) in after {
        if before.get(n) != Some(frag) {
            actually.insert(n);
        }
    }
    for n in before.keys() {
        if !after.contains_key(n) {
            actually.insert(n);
        }
    }
    let reported: BTreeSet<&String> = reported.iter().collect();
    let missed: Vec<&&String> = actually.difference(&reported).collect();
    assert!(
        missed.is_empty(),
        "{what}: changed set UNDER-REPORTS {missed:?} — the studio would keep drawing \
         stale geometry for those layers. reported={reported:?} actually_changed={actually:?}"
    );
}

/// A bus reported ROUTED must have BUILT something.
///
/// Green status over an empty layer is the worst possible outcome: a viewer that
/// trusts the status draws nothing and reports success, and nobody finds out
/// until the build does not work. Asserted GLOBALLY, after every operation, not
/// just around the API that happened to expose it once.
fn assert_every_routed_bus_built_something(d: &Design, what: &str) {
    let geo = geometry(d);
    let empty: Vec<String> = geo
        .iter()
        .filter(|(n, frag)| {
            frag.is_empty() && d.bus(n).is_some_and(|b| b.state == BusState::Routed)
        })
        .map(|(n, _)| n.clone())
        .collect();
    assert!(
        empty.is_empty(),
        "{what}: bus(es) {empty:?} report ROUTED with ZERO cells — green status, nothing built"
    );
}

/// (c) NO STALE GEOMETRY.
///
/// The literal form of this — "every fragment equals a fresh from-scratch
/// route" — is the WRONG invariant, and the harness proved it: routing is
/// deliberately INCREMENTAL. A bus nobody disturbed keeps its geometry
/// (including the dip-unders it grew around buses that have since been
/// deleted), and `move_gate` re-plans exactly two segments on purpose.
/// Re-routing the world on every edit is the 431 ms/frame regression this
/// design exists to avoid. Byte-identity is asserted where it IS the
/// invariant: an explicit full rip+reroute cycle.
///
/// What "stale" actually means, and what the user saw, is a fragment that no
/// longer MEETS ITS PORTS: the instance moved, the endpoint anchors moved with
/// it, and the bus was left wired to where they used to be. So: every routed
/// bus must touch every one of its CURRENT port anchors. (LVS `opens` covers
/// the same ground from the netlist side; this one localizes the blame.)
fn assert_fragments_meet_their_ports(d: &Design, what: &str) {
    for (name, frag) in geometry(d) {
        let Some(layer) = d.bus(&name) else { continue };
        if layer.state != BusState::Routed {
            continue;
        }
        let endpoints: Vec<String> = layer
            .driver_names()
            .into_iter()
            .chain(layer.sinks.iter().cloned())
            .collect();
        for ep in &endpoints {
            let port = d
                .resolve_port(ep)
                .unwrap_or_else(|e| panic!("{what}: bus `{name}` endpoint `{ep}`: {e}"));
            // The planner lays cells strictly BETWEEN anchors, so the anchor
            // itself belongs to the endpoint hardware: the fragment must own a
            // cell orthogonally adjacent to it (or the anchor itself, when a
            // gate column landed there).
            let a = port.anchor;
            let touches = frag.contains_key(&a)
                || [
                    (1, 0, 0),
                    (-1, 0, 0),
                    (0, 0, 1),
                    (0, 0, -1),
                    (0, 1, 0),
                    (0, -1, 0),
                ]
                .iter()
                .any(|(dx, dy, dz)| frag.contains_key(&(a.0 + dx, a.1 + dy, a.2 + dz)));
            assert!(
                touches,
                "{what}: bus `{name}` is STALE — it does not reach its port `{ep}`, whose \
                 anchor is now {a:?}. The geometry is still wired to where the port used to be."
            );
        }
    }
}

/// The full battery after one operation.
fn assert_all(
    d: &Design,
    reported: &[String],
    before: &BTreeMap<String, BTreeMap<P3, String>>,
    what: &str,
) {
    assert_clean(d, what);
    assert_changed_set_complete(reported, before, &geometry(d), what);
    assert_fragments_meet_their_ports(d, what);
}

// ----------------------------------------------------------------------
// Layouts
// ----------------------------------------------------------------------

/// Two instances of the keepout cell, wired driver -> u0 -> u1 -> sink, plus
/// a crossing bus so amendments are in play.
fn layout_instances() -> Design {
    let mut s = UniversalSchematic::new("stress".to_string());
    let din = lever_bank(&mut s, 0, 2, 8, 1, 0);
    let dout = lamp_bank(&mut s, 60, 2, 8);
    let xin = lever_bank(&mut s, 30, 2, -12, 0, 1);
    let xout = lamp_bank(&mut s, 30, 2, 30);
    let mut d = Design::for_schematic("stress", s);
    d.add_cell("wire", wire_cell()).unwrap();
    d.place("u0", "wire", (16, 2, 8), 0).unwrap();
    d.place("u1", "wire", (40, 2, 8), 0).unwrap();
    let step = (0, 2, 0);
    d.declare_input("din", din, step, W, ty()).unwrap();
    d.declare_output("dout", dout, step, W, ty()).unwrap();
    d.declare_input("xin", xin, step, W, ty()).unwrap();
    d.declare_output("xout", xout, step, W, ty()).unwrap();
    d
}

fn routed(d: &mut Design, name: &str, drv: &str, snk: &str) {
    let st = d
        .route_bus(name, drv, &[snk], vec![], BusStyle::default())
        .unwrap_or_else(|e| panic!("{name}: declaration refused: {e}"));
    assert_eq!(st, BusState::Routed, "{name}: {:?}", d.bus_state(name));
}

// ----------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------

/// THE REPORTED BUG. A bus wired to an instance must re-route when the
/// instance moves, whatever the halo geometry says — otherwise its fragment
/// still points at where the ports used to be.
#[test]
fn moving_an_instance_reroutes_the_buses_wired_to_it() {
    let mut d = layout_instances();
    routed(&mut d, "a", "din", "u0.d");
    routed(&mut d, "b", "u0.q", "u1.d");
    routed(&mut d, "c", "u1.q", "dout");
    assert_clean(&d, "initial");

    let before = geometry(&d);
    let rev = d.layer_revision();
    let rep = d.move_instance("u0", (16, 2, 14), 0).unwrap();
    // The engine's own report and the revision query must agree.
    assert_eq!(
        rep.changed,
        d.changed_layers_since(rev),
        "MoveReport.changed disagrees with changed_layers_since"
    );
    assert!(
        rep.changed.iter().any(|n| n == "a") && rep.changed.iter().any(|n| n == "b"),
        "the buses WIRED to u0 were not reported as changed: {:?}",
        rep.changed
    );
    assert_all(&d, &rep.changed, &before, "move u0 (wired buses)");

    // The port really did move with the instance, and `assert_all` above has
    // already proved the fragment reaches it there (the planner lays cells
    // strictly BETWEEN anchors, so the closest fragment cell is the anchor's
    // neighbour, not the anchor itself).
    let anchor = d.resolve_port("u0.d").unwrap().anchor;
    assert_eq!(anchor.2, 14, "the port did not move with the instance");
}

/// A rotation is a move: same guarantees.
#[test]
fn rotating_an_instance_keeps_every_invariant() {
    let mut d = layout_instances();
    routed(&mut d, "a", "din", "u0.d");
    routed(&mut d, "c", "u1.q", "dout");
    for rot in [90, 180, 270, 0] {
        let before = geometry(&d);
        let rev = d.layer_revision();
        let rep = d.move_instance("u1", (40, 2, 8), rot).unwrap();
        assert_eq!(rep.changed, d.changed_layers_since(rev));
        assert_all(&d, &rep.changed, &before, &format!("rotate u1 to {rot}"));
    }
}

/// Deleting an instance: wired buses go away (reported), crossers re-route,
/// and nothing is left orphaned.
#[test]
fn removing_an_instance_reports_every_layer_it_touched() {
    let mut d = layout_instances();
    routed(&mut d, "a", "din", "u0.d");
    routed(&mut d, "b", "u0.q", "u1.d");
    routed(&mut d, "x", "xin", "xout");
    assert_clean(&d, "initial");

    let before = geometry(&d);
    let rev = d.layer_revision();
    let rep = d.remove_instance("u0").unwrap();
    let reported: Vec<String> = rep
        .removed_buses
        .iter()
        .chain(rep.moves.changed.iter())
        .cloned()
        .collect();
    assert_eq!(
        d.changed_layers_since(rev),
        {
            let mut all: Vec<String> = reported
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            all.sort();
            all
        },
        "the removal's reported set disagrees with changed_layers_since"
    );
    assert!(
        rep.removed_buses.contains(&"a".to_string())
            && rep.removed_buses.contains(&"b".to_string()),
        "buses wired to u0 were not reported as removed: {:?}",
        rep.removed_buses
    );
    assert_all(&d, &reported, &before, "remove u0");
    for n in &rep.removed_buses {
        assert!(
            d.bus(n).is_none(),
            "removed bus `{n}` is still in the document"
        );
    }
}

/// A crossing stamps a station into ANOTHER bus's fragment. That bus is never
/// ripped and appears in no other report — if it is not named as changed, the
/// studio keeps its pre-station mesh. This is the amendment path.
#[test]
fn a_crossing_amendment_names_the_bus_it_amended() {
    let mut s = UniversalSchematic::new("cross".to_string());
    let ain = lever_bank(&mut s, 0, 2, 8, 1, 0);
    let aout = lamp_bank(&mut s, 32, 2, 8);
    let bin = lever_bank(&mut s, 16, 2, -8, 0, 1);
    let bout = lamp_bank(&mut s, 16, 2, 24);
    let mut d = Design::for_schematic("cross", s);
    let step = (0, 2, 0);
    d.declare_input("ain", ain, step, W, ty()).unwrap();
    d.declare_output("aout", aout, step, W, ty()).unwrap();
    d.declare_input("bin", bin, step, W, ty()).unwrap();
    d.declare_output("bout", bout, step, W, ty()).unwrap();

    routed(&mut d, "a", "ain", "aout");
    let before = geometry(&d);
    let rev = d.layer_revision();
    routed(&mut d, "b", "bin", "bout");
    let changed = d.changed_layers_since(rev);
    let after = geometry(&d);
    // Bus `a` really was amended...
    assert_ne!(
        before.get("a"),
        after.get("a"),
        "the crossing did not amend bus `a` — this test is measuring nothing"
    );
    // ...so routing `b` must report BOTH layers.
    assert_changed_set_complete(&changed, &before, &after, "route b across a");
    assert!(
        changed.iter().any(|n| n == "a"),
        "routing `b` amended `a` but did not report it: {changed:?}"
    );
    assert_clean(&d, "crossing");
}

/// Moving a gate: geometry stays legal and the report is complete.
#[test]
fn moving_a_gate_keeps_every_invariant() {
    let mut s = UniversalSchematic::new("gate".to_string());
    let din = lever_bank(&mut s, 0, 2, 8, 1, 0);
    let dout = lamp_bank(&mut s, 48, 2, 8);
    let mut d = Design::for_schematic("gate", s);
    let step = (0, 2, 0);
    d.declare_input("din", din, step, W, ty()).unwrap();
    d.declare_output("dout", dout, step, W, ty()).unwrap();
    let gates = vec![
        Gate {
            name: "g0".into(),
            anchor: (16, 2, 8),
            step,
        },
        Gate {
            name: "g1".into(),
            anchor: (32, 2, 8),
            step,
        },
    ];
    let st = d
        .route_bus("g", "din", &["dout"], gates, BusStyle::default())
        .unwrap();
    assert_eq!(st, BusState::Routed, "{:?}", d.bus_state("g"));

    for anchor in [(16, 2, 14), (20, 2, 8), (16, 2, 8)] {
        let before = geometry(&d);
        let rev = d.layer_revision();
        let rep = d.move_gate("g", "g0", anchor).unwrap();
        assert_eq!(rep.changed, d.changed_layers_since(rev));
        assert_all(
            &d,
            &rep.changed,
            &before,
            &format!("move gate to {anchor:?}"),
        );
    }
}

/// (b) DETERMINISM + NO ORPHANS: 10 rip+reroute cycles must reproduce
/// byte-identical geometry and the exact same block count. A leak shows up as
/// a growing count; nondeterminism shows up as a differing fragment.
#[test]
fn ten_rip_and_reroute_cycles_are_byte_identical() {
    let mut d = layout_instances();
    routed(&mut d, "a", "din", "u0.d");
    routed(&mut d, "b", "u0.q", "u1.d");
    routed(&mut d, "c", "u1.q", "dout");
    routed(&mut d, "x", "xin", "xout");
    assert_clean(&d, "initial");

    let names: Vec<String> = geometry(&d).keys().cloned().collect();
    let baseline = geometry(&d);
    let baseline_cells = total_bus_cells(&d);
    for cycle in 0..10 {
        for n in &names {
            d.rip(n).unwrap();
        }
        // A rip must leave NOTHING behind.
        assert_eq!(
            total_bus_cells(&d),
            0,
            "cycle {cycle}: {} orphaned cells survived a full rip",
            total_bus_cells(&d)
        );
        for n in &names {
            let st = d.reroute(n).unwrap();
            assert_eq!(st, BusState::Routed, "cycle {cycle}: `{n}` -> {st:?}");
        }
        assert_eq!(
            total_bus_cells(&d),
            baseline_cells,
            "cycle {cycle}: block count drifted (orphans or a lost cell)"
        );
        assert_eq!(
            geometry(&d),
            baseline,
            "cycle {cycle}: rip+reroute is NOT deterministic"
        );
        assert_clean(&d, &format!("cycle {cycle}"));
    }
}

/// A round trip through .nucm must re-route to the same geometry: a reloaded
/// document is the same document.
#[test]
fn a_reloaded_document_reroutes_to_the_same_geometry() {
    let mut d = layout_instances();
    routed(&mut d, "a", "din", "u0.d");
    routed(&mut d, "b", "u0.q", "u1.d");
    routed(&mut d, "x", "xin", "xout");
    let before = geometry(&d);

    let bytes = d.to_nucm_bytes().expect("nucm export");
    let mut back = Design::from_nucm_bytes(&bytes).expect("nucm import");
    assert_eq!(
        geometry(&back),
        before,
        "the reload lost or altered geometry"
    );

    // A fresh reader has drawn nothing, so it starts at revision 0 and the
    // first edit reports everything it touches.
    let rev = back.layer_revision();
    let names: Vec<String> = before.keys().cloned().collect();
    for n in &names {
        back.rip(n).unwrap();
    }
    for n in &names {
        assert_eq!(
            back.reroute(n).unwrap(),
            BusState::Routed,
            "`{n}` after reload"
        );
    }
    let changed = back.changed_layers_since(rev);
    assert_changed_set_complete(&changed, &before, &geometry(&back), "reload + reroute");
    assert_eq!(
        geometry(&back),
        before,
        "re-routing a reloaded document produced different geometry"
    );
    assert_clean(&back, "after reload");
}

/// A long random-ish walk: many moves in sequence, every invariant after each.
/// Moves that push a bus into unroutability are fine — a FAILED bus is visible
/// and must still be reported and still leave the document clean.
#[test]
fn a_long_walk_of_moves_never_leaves_stale_geometry() {
    let mut d = layout_instances();
    routed(&mut d, "a", "din", "u0.d");
    routed(&mut d, "b", "u0.q", "u1.d");
    routed(&mut d, "c", "u1.q", "dout");
    routed(&mut d, "x", "xin", "xout");

    let steps: &[(&str, P3, i32)] = &[
        ("u0", (16, 2, 12), 0),
        ("u1", (40, 2, 12), 0),
        ("u0", (20, 2, 12), 180),
        ("u1", (44, 2, 4), 0),
        ("u0", (16, 2, 8), 0),
        ("u1", (40, 2, 8), 0),
        ("u0", (16, 4, 8), 0),
        ("u0", (16, 2, 8), 0),
    ];
    for (i, (inst, at, rot)) in steps.iter().enumerate() {
        let before = geometry(&d);
        let rev = d.layer_revision();
        let rep = d.move_instance(inst, *at, *rot).unwrap();
        let what = format!("step {i}: move {inst} to {at:?} rot {rot}");
        assert_eq!(rep.changed, d.changed_layers_since(rev), "{what}");
        assert_all(&d, &rep.changed, &before, &what);
    }
}

// ----------------------------------------------------------------------
// Adapter ownership: the bus owns its form adapter, so a rip removes it
// ----------------------------------------------------------------------

/// A community-shaped cell whose input port is a HORIZONTAL ROW at pitch 2 —
/// the `BINTOBCD001.bin` shape. Promotion must leave that row alone; the bus
/// grows the row->stack adapter it needs.
fn row_port_cell() -> UniversalSchematic {
    use nucleation::io_contract::{CellContract, IoLayoutBuilder, LayoutFunction};
    let mut s = UniversalSchematic::new("row".to_string());
    // Levers marching along x at pitch 2, each on its own support, plus a lamp
    // column output so the cell has something to read.
    let d_hw: Vec<P3> = (0..W as i32).map(|i| (2 * i, 5, 0)).collect();
    let q_hw: Vec<P3> = (0..W as i32).map(|i| (2 * i, 1, 0)).collect();
    for i in 0..W as i32 {
        s.set_block_from_string(2 * i, 4, 0, STONE).unwrap();
        s.set_block_from_string(2 * i, 5, 0, LEVER).unwrap();
        s.set_block_from_string(2 * i, 1, 0, LAMP).unwrap();
        s.set_block_from_string(2 * i, 2, 0, DUST).unwrap();
    }
    let layout = IoLayoutBuilder::new()
        .add_input("bin".to_string(), ty(), LayoutFunction::OneToOne, d_hw)
        .unwrap()
        .add_output("out".to_string(), ty(), LayoutFunction::OneToOne, q_hw)
        .unwrap()
        .build();
    s.set_cell_contract(&CellContract::new("row".to_string(), layout))
        .unwrap();
    s
}

/// (b) NO ORPHANS ACROSS THE OWNERSHIP BOUNDARY.
///
/// The row->stack form adapter used to be emitted by PROMOTION, into a
/// per-instance patch. Ripping the bus then left the staircase and gather
/// column behind: geometry that existed only to serve that bus, that the user
/// could not remove, and that reached far outside the component. The adapter
/// now belongs to the BUS, so a rip must return the flattened design to
/// exactly its pre-route block count.
#[test]
fn ripping_a_bus_removes_the_form_adapter_it_grew() {
    let mut s = UniversalSchematic::new("adapt".to_string());
    let din = lever_bank(&mut s, 0, 2, 20, 1, 0);
    let mut d = Design::for_schematic("adapt", s);
    d.add_cell("row", row_port_cell()).unwrap();
    d.place("u0", "row", (30, 0, 0), 0).unwrap();
    d.declare_input("din", din, (0, 2, 0), W, ty()).unwrap();

    // The port's NATIVE form survives promotion: a horizontal row at pitch 2.
    d.promote_input("u0", "bin").unwrap();
    let port = d
        .instance_ports()
        .unwrap()
        .into_iter()
        .find(|p| p.name == "u0.bin")
        .unwrap();
    assert_eq!(
        port.step,
        Some((2, 0, 0)),
        "promotion converted the component's form — that is the bus's job"
    );

    let before = d.flatten().unwrap().total_blocks();
    let st = d
        .route_bus("a", "din", &["u0.bin"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(st, BusState::Routed, "{:?}", d.bus_state("a"));
    let routed = d.flatten().unwrap().total_blocks();
    assert!(
        routed > before,
        "the bus laid nothing: {before} -> {routed}"
    );
    // The adapter really is the BUS's: its segments name it.
    let layer = d.bus("a").unwrap();
    assert!(
        layer.segments.iter().any(|s| matches!(
            &s.kind,
            nucleation::design::SegmentKind::Adapter(n) if n == "u0.bin"
        )),
        "no adapter segment on the bus: {:?}",
        layer.segments.iter().map(|s| &s.kind).collect::<Vec<_>>()
    );
    assert_clean(&d, "row-port bus");

    d.rip("a").unwrap();
    let after = d.flatten().unwrap().total_blocks();
    assert_eq!(
        after,
        before,
        "ripping the bus left {} orphaned cell(s) behind — geometry that existed only to serve \
         that bus and that the user cannot remove",
        after as i64 - before as i64
    );
    assert_eq!(total_bus_cells(&d), 0, "the ripped fragment is not empty");

    // And it re-routes to the same thing, adapter included.
    assert_eq!(d.reroute("a").unwrap(), BusState::Routed);
    assert_eq!(
        d.flatten().unwrap().total_blocks(),
        routed,
        "re-routing after a rip did not reproduce the same block count"
    );
}

// ----------------------------------------------------------------------
// Gates vs endpoints: deleting them means different things
// ----------------------------------------------------------------------

/// Removing a GATE relaxes a constraint: the bus survives, the two spans it
/// separated MERGE, and the result is genuinely straighter — not the two old
/// legs stitched together.
#[test]
fn removing_a_gate_merges_its_spans_into_a_more_direct_route() {
    let mut s = UniversalSchematic::new("rmg".to_string());
    let din = lever_bank(&mut s, 0, 2, 8, 1, 0);
    let dout = lamp_bank(&mut s, 48, 2, 8);
    let mut d = Design::for_schematic("rmg", s);
    let step = (0, 2, 0);
    d.declare_input("din", din, step, W, ty()).unwrap();
    d.declare_output("dout", dout, step, W, ty()).unwrap();
    // A gate dragged well off the straight line forces a dogleg.
    let gates = vec![Gate {
        name: "g0".into(),
        anchor: (24, 2, 24),
        step,
    }];
    assert_eq!(
        d.route_bus("g", "din", &["dout"], gates, BusStyle::default())
            .unwrap(),
        BusState::Routed
    );
    let with_gate = d.bus("g").unwrap().fragment.len();
    let segments_with = d.bus("g").unwrap().segments.len();
    assert_clean(&d, "with the gate");

    let before = geometry(&d);
    let rev = d.layer_revision();
    let rep = d.remove_gate("g", 0).unwrap();
    assert_eq!(rep.state, BusState::Routed, "{:?}", d.bus_state("g"));
    assert_eq!(rep.changed, d.changed_layers_since(rev));
    assert!(rep.changed.iter().any(|n| n == "g"), "{:?}", rep.changed);
    assert_all(&d, &rep.changed, &before, "remove gate g0");

    // The bus SURVIVED (a gate is a constraint, not a terminal)...
    assert!(d.bus("g").is_some());
    assert!(d.bus("g").unwrap().gates.is_empty());
    // ...and the merged span is genuinely more direct, not spliced.
    let without = d.bus("g").unwrap().fragment.len();
    assert!(
        without < with_gate,
        "removing the gate did not shorten the route: {with_gate} -> {without}"
    );
    assert!(
        d.bus("g").unwrap().segments.len() < segments_with,
        "the spans were not merged: still {} segment(s)",
        d.bus("g").unwrap().segments.len()
    );

    // Out-of-range says what there is.
    let err = d.remove_gate("g", 0).unwrap_err();
    assert!(
        err.contains("no gate at index 0") && err.contains("none"),
        "{err}"
    );
}

/// Removing an ENDPOINT changes the netlist: the bus loses a terminal, so it is
/// deleted — and never silently.
#[test]
fn removing_a_port_is_refused_before_it_deletes_a_bus() {
    let mut s = UniversalSchematic::new("rmp".to_string());
    let din = lever_bank(&mut s, 0, 2, 8, 1, 0);
    let dout = lamp_bank(&mut s, 32, 2, 8);
    let mut d = Design::for_schematic("rmp", s);
    let step = (0, 2, 0);
    d.declare_input("din", din, step, W, ty()).unwrap();
    d.declare_output("dout", dout, step, W, ty()).unwrap();
    routed(&mut d, "a", "din", "dout");

    // Unconfirmed: refused, and it names what would go.
    let err = d.remove_port("dout", false).unwrap_err();
    assert!(
        err.contains("ENDPOINT") && err.contains("\"a\"")
            || err.contains("a)")
            || err.contains(" a"),
        "{err}"
    );
    assert!(
        d.bus("a").is_some(),
        "the bus was deleted despite the refusal"
    );

    // Confirmed: the bus goes, and it is reported.
    let rev = d.layer_revision();
    let (removed, moves) = d.remove_port("dout", true).unwrap();
    assert_eq!(removed, vec!["a".to_string()]);
    assert!(d.bus("a").is_none(), "the bus outlived its endpoint");
    assert!(d.port("dout").is_none(), "the port declaration survived");
    let mut reported: Vec<String> = removed
        .iter()
        .chain(moves.changed.iter())
        .cloned()
        .collect();
    reported.sort();
    reported.dedup();
    assert_eq!(
        reported,
        d.changed_layers_since(rev),
        "removal under-reported"
    );

    // An instance port is derived, not declared: say so instead of no-op.
    let mut d2 = layout_instances();
    let err = d2.remove_port("u0.d", true).unwrap_err();
    assert!(
        err.contains("INSTANCE port") && err.contains("set_port_mode"),
        "{err}"
    );
}

/// BUG: `add_gate` resolved its endpoints through the DECLARED port table, so
/// every bus between two placed cells (`u0.q` -> `u1.d` — instance ports,
/// derived from the contract, never in that table) refused a gate outright,
/// while `move_gate` worked on the very same bus. Gates were therefore unusable
/// on real cell-to-cell buses.
#[test]
fn a_gate_can_be_added_to_a_bus_between_two_instance_ports() {
    let mut d = layout_instances();
    routed(&mut d, "b", "u0.q", "u1.d");
    assert!(d.bus("b").unwrap().gates.is_empty());

    let before = geometry(&d);
    let rev = d.layer_revision();
    let st = d
        .add_gate("b", "g0", (28, 2, 8), (0, 2, 0))
        .expect("a gate on an instance-port bus must be accepted");
    assert_eq!(st, BusState::Routed, "{:?}", d.bus_state("b"));
    assert_eq!(d.bus("b").unwrap().gates.len(), 1);
    assert!(
        d.bus("b").unwrap().segments.len() >= 2,
        "the gate did not split the bus: {} segment(s)",
        d.bus("b").unwrap().segments.len()
    );
    let changed = d.changed_layers_since(rev);
    assert_changed_set_complete(&changed, &before, &geometry(&d), "add_gate");
    assert_all(&d, &changed, &before, "add gate to an instance-port bus");

    // ...and it moves and comes back off again, on the same bus.
    let rep = d.move_gate("b", "g0", (28, 2, 12)).unwrap();
    assert_eq!(rep.state, BusState::Routed, "{:?}", rep.state);
    let rep = d.remove_gate_named("b", "g0").unwrap();
    assert_eq!(rep.state, BusState::Routed, "{:?}", rep.state);
    assert!(d.bus("b").unwrap().gates.is_empty());
    assert_clean(&d, "after the gate round trip");
}

// ----------------------------------------------------------------------
// Gate realization: a checkpoint the bus does not pass through is a bug
// ----------------------------------------------------------------------

/// Every bit of every gate's checkpoint column must be IN the realized route.
fn assert_route_passes_through_gates(d: &Design, bus: &str, what: &str) {
    let layer = d.bus(bus).expect("bus");
    if layer.state != BusState::Routed {
        return;
    }
    for g in &layer.gates {
        for k in 0..W as i32 {
            let p = (g.anchor.0, g.anchor.1 + 2 * k, g.anchor.2);
            assert!(
                layer.fragment.contains_key(&p),
                "{what}: bus `{bus}` reports routed but bit {k} of gate `{}`'s checkpoint column \
                 at {p:?} is not in the route — the bus does not pass through its own gate",
                g.name
            );
        }
    }
}

/// REGRESSION GUARD for the reported shape: "0 cells with the checkpoint, 1440
/// without". Adding a gate can only ever ADD constraint, so the gated route
/// must be non-empty, must pass through the checkpoint, and must cost at least
/// as much as the free route. Removing it must give the cells back.
#[test]
fn adding_a_gate_never_yields_an_empty_or_shorter_route() {
    for at in [(24, 2, 8), (24, 2, 20), (30, 2, 8), (24, 4, 8)] {
        let mut d = layout_instances();
        routed(&mut d, "b", "u0.q", "u1.d");
        let ungated = d.bus("b").unwrap().fragment.len();
        assert!(ungated > 0, "the ungated route is empty to begin with");

        let before = geometry(&d);
        let rev = d.layer_revision();
        let st = d
            .add_gate("b", "g0", at, (0, 2, 0))
            .unwrap_or_else(|e| panic!("add_gate {at:?} refused: {e}"));
        let what = format!("gate at {at:?}");
        // A refusal is fine; a green-but-empty bus is not.
        if st != BusState::Routed {
            assert!(
                d.bus("b").unwrap().fragment.is_empty(),
                "{what}: FAILED but left geometry behind"
            );
            continue;
        }
        let gated = d.bus("b").unwrap().fragment.len();
        assert!(gated > 0, "{what}: routed with ZERO cells (was {ungated})");
        assert_route_passes_through_gates(&d, "b", &what);
        assert!(
            gated >= ungated,
            "{what}: adding a constraint SHORTENED the route ({ungated} -> {gated}) — the \
             checkpoint cannot have been honoured"
        );
        assert_all(&d, &d.changed_layers_since(rev), &before, &what);

        // ...and taking it off gives the cells back without going empty.
        let rep = d.remove_gate("b", 0).unwrap();
        assert_eq!(rep.state, BusState::Routed, "{what}: ungating failed");
        let merged = d.bus("b").unwrap().fragment.len();
        assert!(merged > 0, "{what}: the MERGED span is empty");
        assert!(
            merged <= gated,
            "{what}: removing the constraint made the route longer ({gated} -> {merged})"
        );
        assert_clean(&d, &format!("{what}, ungated again"));
    }
}

/// The same on the REAL corpus chain the report was measured against
/// (`ADD007.sum -> BINTOBCD001.bin`, 1440 cells ungated), including the
/// off-trunk-level gates the studio's right-click now produces.
#[test]
fn gates_on_the_real_corpus_chain_realize() {
    fn load(file: &str) -> Option<UniversalSchematic> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("computational_schematics/enhanced")
            .join(file);
        nucleation::formats::schematic::from_schematic(&std::fs::read(path).ok()?).ok()
    }
    let (Some(add), Some(bcd)) = (
        load("ADD007_8bit_cca_matt_enhanced.schem"),
        load("BINTOBCD001_8bit_comb_binary_to_bcd_enhanced.schem"),
    ) else {
        eprintln!("enhanced corpus unavailable; skipping");
        return;
    };
    let mut base = Design::new("chain");
    base.add_cell("add", add).unwrap();
    base.add_cell("bcd", bcd).unwrap();
    base.place("u0", "add", (0, 0, 0), 0).unwrap();
    base.place("u1", "bcd", (60, -2, 40), 0).unwrap();
    base.promote_input("u1", "bin").unwrap();
    let st = base
        .route_bus(
            "sum_to_bin",
            "u0.sum",
            &["u1.bin"],
            vec![],
            BusStyle::default(),
        )
        .unwrap();
    assert_eq!(st, BusState::Routed, "{:?}", base.bus_state("sum_to_bin"));
    let ungated = base.bus("sum_to_bin").unwrap().fragment.len();
    assert!(ungated > 0);

    for at in [(30, 3, 1), (40, 3, 20), (28, 3, 26), (45, 3, 40)] {
        let mut d = base.clone();
        let st = d.add_gate("sum_to_bin", "g0", at, (0, 2, 0)).unwrap();
        let what = format!("corpus gate {at:?}");
        if st != BusState::Routed {
            continue; // a refusal is a legitimate answer; emptiness is not
        }
        let gated = d.bus("sum_to_bin").unwrap().fragment.len();
        assert!(
            gated > 0,
            "{what}: routed with ZERO cells (ungated was {ungated})"
        );
        // The 8-bit chain's gate column is 8 bits, not W.
        let layer = d.bus("sum_to_bin").unwrap();
        for g in &layer.gates {
            for k in 0..8i32 {
                let p = (g.anchor.0, g.anchor.1 + 2 * k, g.anchor.2);
                assert!(
                    layer.fragment.contains_key(&p),
                    "{what}: bit {k} of the checkpoint at {p:?} is not in the route"
                );
            }
        }
        assert!(gated >= ungated, "{what}: {ungated} -> {gated}");
        assert_every_routed_bus_built_something(&d, &what);

        let rep = d.remove_gate("sum_to_bin", 0).unwrap();
        assert_eq!(rep.state, BusState::Routed, "{what}: ungating failed");
        let merged = d.bus("sum_to_bin").unwrap().fragment.len();
        assert!(
            merged > 0 && merged <= gated,
            "{what}: merged {merged}, gated {gated}"
        );
    }
}
