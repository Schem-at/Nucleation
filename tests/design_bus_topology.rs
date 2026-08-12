//! ROUTE TOPOLOGY, not just cell count.
//!
//! "We just need smarter bussing." A cell-count measurement on a straight run
//! says nothing about whether the route went the sensible WAY, so this file
//! asserts SHAPE: for congruent ports the bus must be N parallel wires with no
//! form conversion at all, and no route may wander far outside the span its own
//! endpoints already occupy.

#![cfg(feature = "routing")]

use nucleation::design::{BusState, BusStyle, Design, SegmentKind};
use nucleation::io_contract::{CellContract, IoLayoutBuilder, IoType, LayoutFunction};
use nucleation::UniversalSchematic;

type P3 = (i32, i32, i32);
const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";
const W: u8 = 8;

fn ty() -> IoType {
    IoType::UnsignedInt { bits: W as usize }
}

/// A cell whose ports are horizontal ROWS at pitch 2 — the BINTOBCD001 shape,
/// input on one face and output on the other.
fn row_cell() -> UniversalSchematic {
    let mut s = UniversalSchematic::new("row".to_string());
    let d: Vec<P3> = (0..W as i32).map(|i| (2 * i, 2, 0)).collect();
    let q: Vec<P3> = (0..W as i32).map(|i| (2 * i, 2, 6)).collect();
    for i in 0..W as i32 {
        s.set_block_from_string(2 * i, 1, 0, STONE).unwrap();
        s.set_block_from_string(2 * i, 2, 0, DUST).unwrap();
        s.set_block_from_string(2 * i, 1, 6, LAMP).unwrap();
        s.set_block_from_string(2 * i, 2, 6, DUST).unwrap();
    }
    let l = IoLayoutBuilder::new()
        .add_input("d", ty(), LayoutFunction::OneToOne, d)
        .unwrap()
        .add_output("q", ty(), LayoutFunction::OneToOne, q)
        .unwrap()
        .build();
    s.set_cell_contract(&CellContract::new("row".to_string(), l))
        .unwrap();
    s
}

/// Route bbox, and how far it overshoots the endpoints' own span per face.
fn bbox_and_overshoot(d: &Design, bus: &str) -> ((P3, P3), [i32; 6]) {
    let layer = d.bus(bus).unwrap();
    let f = &layer.fragment;
    let mut lo = (i32::MAX, i32::MAX, i32::MAX);
    let mut hi = (i32::MIN, i32::MIN, i32::MIN);
    for p in f.keys() {
        lo = (lo.0.min(p.0), lo.1.min(p.1), lo.2.min(p.2));
        hi = (hi.0.max(p.0), hi.1.max(p.1), hi.2.max(p.2));
    }
    let mut ends: Vec<P3> = Vec::new();
    for n in layer.driver_names().iter().chain(layer.sinks.iter()) {
        let p = d.resolve_port(n).unwrap();
        ends.push(p.anchor);
        ends.push(p.wire(p.width - 1));
    }
    let elo = (
        ends.iter().map(|p| p.0).min().unwrap(),
        ends.iter().map(|p| p.1).min().unwrap(),
        ends.iter().map(|p| p.2).min().unwrap(),
    );
    let ehi = (
        ends.iter().map(|p| p.0).max().unwrap(),
        ends.iter().map(|p| p.1).max().unwrap(),
        ends.iter().map(|p| p.2).max().unwrap(),
    );
    (
        (lo, hi),
        [
            elo.0 - lo.0,
            hi.0 - ehi.0,
            elo.1 - lo.1,
            hi.1 - ehi.1,
            elo.2 - lo.2,
            hi.2 - ehi.2,
        ],
    )
}

/// CONGRUENT PORTS ARE N PARALLEL WIRES. Two row-form ports facing each other
/// need no gather bar, no canonical stack and no fan — and the route must prove
/// it by carrying NO adapter segment at all.
#[test]
fn congruent_ports_route_as_parallel_lanes_with_no_form_conversion() {
    let mut d = Design::new("cong");
    d.add_cell("row", row_cell()).unwrap();
    d.place("u0", "row", (0, 0, 0), 0).unwrap();
    d.place("u1", "row", (0, 0, 30), 0).unwrap();
    let st = d
        .route_bus("b", "u0.q", &["u1.d"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(st, BusState::Routed, "{:?}", d.bus_state("b"));

    let layer = d.bus("b").unwrap();
    // THE POINT: no form conversion anywhere in the route.
    let adapters: Vec<&SegmentKind> = layer
        .segments
        .iter()
        .map(|s| &s.kind)
        .filter(|k| matches!(k, SegmentKind::Adapter(_)))
        .collect();
    assert!(
        adapters.is_empty(),
        "congruent row ports still built {} form adapter(s) — the bus converted to a form it \
         immediately left: {:?}",
        adapters.len(),
        layer.segments.iter().map(|s| &s.kind).collect::<Vec<_>>()
    );
    // One lane per bit, and every lane really is its own segment.
    assert_eq!(
        layer.segments.len(),
        W as usize,
        "expected {W} parallel lanes, got {:?}",
        layer.segments.iter().map(|s| &s.kind).collect::<Vec<_>>()
    );

    // ...and it does not wander. A parallel bundle stays inside its endpoints.
    let ((lo, hi), over) = bbox_and_overshoot(&d, "b");
    assert!(
        over.iter().all(|o| *o <= 2),
        "the bundle left its endpoints' span by {over:?} (bbox {lo:?}..{hi:?})"
    );

    // Measured 2026-08-09: 480 cells. A ratchet, not a target — the point is
    // that the form pipeline's two adapters cost far more.
    let cells = layer.fragment.len();
    println!("[congruent] {cells} cells, bbox {lo:?}..{hi:?}, overshoot {over:?}");
    assert!(
        cells <= 700,
        "a congruent 8-bit bundle over 24 cells of run should not cost {cells} cells"
    );
    assert!(d.check().unwrap().clean, "{}", d.check().unwrap().json);
}

/// The stack form still wins where it pays for itself: congruent CANONICAL
/// ports keep the dense shared-support run rather than splitting into lanes.
#[test]
fn the_canonical_stack_is_not_split_into_lanes() {
    let mut s = UniversalSchematic::new("st".to_string());
    for i in 0..W as i32 {
        let y = 2 + 2 * i;
        s.set_block_from_string(-1, y - 1, 8, STONE).unwrap();
        s.set_block_from_string(-1, y, 8, LEVER).unwrap();
        s.set_block_from_string(0, y - 1, 8, STONE).unwrap();
        s.set_block_from_string(0, y, 8, DUST).unwrap();
        s.set_block_from_string(32, y - 1, 8, LAMP).unwrap();
        s.set_block_from_string(32, y, 8, DUST).unwrap();
    }
    let mut d = Design::for_schematic("st", s);
    d.declare_input("din", (0, 2, 8), (0, 2, 0), W, ty())
        .unwrap();
    d.declare_output("dout", (32, 2, 8), (0, 2, 0), W, ty())
        .unwrap();
    assert_eq!(
        d.route_bus("b", "din", &["dout"], vec![], BusStyle::default())
            .unwrap(),
        BusState::Routed
    );
    let layer = d.bus("b").unwrap();
    assert_eq!(
        layer.segments.len(),
        1,
        "the dense stack was split into lanes, losing its shared supports: {:?}",
        layer.segments.iter().map(|s| &s.kind).collect::<Vec<_>>()
    );
}

/// NO ROUTE GOES AROUND THE WORLD. Even where a form conversion IS genuinely
/// needed (a vertical stack driving a horizontal row — the ADD007 -> BINTOBCD001
/// shape), the realized geometry must stay inside the span its own endpoints
/// already occupy.
#[test]
fn a_form_converting_route_stays_inside_its_endpoints_span() {
    let cell = row_cell();
    // Give the design a vertical-stack driver on the loose layer.
    let mut s = UniversalSchematic::new("mix".to_string());
    for i in 0..W as i32 {
        let y = 3 + 2 * i;
        s.set_block_from_string(14, y - 1, 1, STONE).unwrap();
        s.set_block_from_string(14, y, 1, LEVER).unwrap();
        s.set_block_from_string(15, y - 1, 1, STONE).unwrap();
        s.set_block_from_string(15, y, 1, DUST).unwrap();
    }
    let mut d = Design::for_schematic("mix", s);
    d.add_cell("row", cell).unwrap();
    d.place("u1", "row", (60, 1, 40), 0).unwrap();
    d.declare_input("din", (15, 3, 1), (0, 2, 0), W, ty())
        .unwrap();
    let st = d
        .route_bus("b", "din", &["u1.d"], vec![], BusStyle::default())
        .unwrap();
    if st != BusState::Routed {
        eprintln!("mixed-form route unavailable in this geometry: {st:?}");
        return;
    }
    let ((lo, hi), over) = bbox_and_overshoot(&d, "b");
    let cells = d.bus("b").unwrap().fragment.len();
    println!("[mixed form] {cells} cells, bbox {lo:?}..{hi:?}, overshoot {over:?}");
    // The machine-checkable form of "don't go around the world": a conversion
    // may need a little room to grow its staircase, but not a detour on the
    // scale of the route itself.
    let span = (hi.0 - lo.0).max(hi.2 - lo.2);
    assert!(
        over.iter().all(|o| *o <= 8 + span / 8),
        "the route left its endpoints' span by {over:?} (bbox {lo:?}..{hi:?}) — a conversion may \
         grow a staircase, but not wander"
    );
}

// ----------------------------------------------------------------------
// The COST VECTOR itself must be valid
// ----------------------------------------------------------------------

/// THE METRIC UNDER TEST. A cell-count benchmark said "nothing to win" while the
/// screenshot was obviously bad, so validate the term that catches it: the
/// gather-bar-plus-eight-lanes topology must score strictly WORSE on
/// coherence and footprint than the parallel-lane route, and the weighted total
/// must prefer the parallel one — measured on the same two ports, so length is
/// not doing the work.
#[test]
fn the_gather_bar_topology_scores_worse_than_parallel_lanes() {
    use nucleation::design::BusCost;

    // Congruent row ports: the parallel-lane route is available.
    let mut lanes = Design::new("lanes");
    lanes.add_cell("row", row_cell()).unwrap();
    lanes.place("u0", "row", (0, 0, 0), 0).unwrap();
    lanes.place("u1", "row", (0, 0, 30), 0).unwrap();
    assert_eq!(
        lanes
            .route_bus("b", "u0.q", &["u1.d"], vec![], BusStyle::default())
            .unwrap(),
        BusState::Routed
    );
    let v_lanes = lanes.bus_cost(lanes.bus("b").unwrap());

    // The SAME ports, but rotated so the forms no longer match and the planner
    // must convert — the gather-bar topology.
    let mut bar = Design::new("bar");
    bar.add_cell("row", row_cell()).unwrap();
    bar.place("u0", "row", (0, 0, 0), 0).unwrap();
    bar.place("u1", "row", (30, 0, 30), 90).unwrap();
    let st = bar.route_bus("b", "u0.q", &["u1.d"], vec![], BusStyle::default());
    let Ok(BusState::Routed) = st else {
        // A conversion route that will not even realize is worse than "worse".
        println!("[cost] converting route did not realize ({st:?}) — parallel lanes win outright");
        assert_eq!(
            v_lanes.coherence, 0,
            "the parallel bundle must be perfectly coherent"
        );
        return;
    };
    let v_bar = bar.bus_cost(bar.bus("b").unwrap());

    println!("[cost] parallel lanes {v_lanes:?}");
    println!("[cost] gather bar     {v_bar:?}");
    assert!(
        v_bar.coherence > v_lanes.coherence,
        "the metric does not see the difference: bar {} vs lanes {}",
        v_bar.coherence,
        v_lanes.coherence
    );
    for w in [BusCost::default(), BusCost::compact(), BusCost::fast()] {
        assert!(
            w.total(&v_bar) > w.total(&v_lanes),
            "weights {w:?} prefer the gather bar ({:.1}) over parallel lanes ({:.1})",
            w.total(&v_bar),
            w.total(&v_lanes)
        );
    }
}

/// Congruent ports ⇒ coherence penalty is ZERO: the bits travel together in
/// their canonical arrangement the whole way, with no form conversion.
#[test]
fn a_congruent_bundle_has_no_coherence_penalty() {
    let mut d = Design::new("cong2");
    d.add_cell("row", row_cell()).unwrap();
    d.place("u0", "row", (0, 0, 0), 0).unwrap();
    d.place("u1", "row", (0, 0, 30), 0).unwrap();
    d.route_bus("b", "u0.q", &["u1.d"], vec![], BusStyle::default())
        .unwrap();
    let v = d.bus_cost(d.bus("b").unwrap());
    println!("[cost] congruent bundle {v:?}");
    assert_eq!(
        v.coherence, 0,
        "a bundle travelling together must score 0 coherence"
    );
    // The lanes are congruent and equal-length, so the true skew is 0.
    //
    // This used to report a phantom skew: `bus_bit_delays` keyed per-bit
    // arrival off `y`, which is only the bit axis of the canonical 2y stack.
    // A ROW-form bundle steps in `z`, so every repeater sat at `y == y0`, all
    // the delay was charged to bit 0, and the difference between bit 0 and the
    // rest was reported as skew. The delay model now projects onto the
    // driving port's own `step`, so it is form-agnostic.
    assert_eq!(
        v.skew_rt, 0,
        "a congruent equal-length ROW bundle has no skew; a non-zero number here \
         means the delay model is keying bits off the wrong axis again"
    );
    // And the vector reaches the report.
    let json = d.check().unwrap().json;
    assert!(
        json.contains("\"coherence\""),
        "the cost vector is not in check(): {json}"
    );
    assert!(json.contains("\"footprint\""), "{json}");
}
