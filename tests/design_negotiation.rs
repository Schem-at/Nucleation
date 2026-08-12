//! The skip-hop cell row: the case negotiated congestion exists for.
//!
//! `tests/design_routability.rs` measures the whole sweep and takes minutes;
//! this is the same geometry as its `cell row: skip hops (detour required)`
//! scenario on its own, so the one bus that the corridor search cannot place by
//! declaration order can be iterated on in seconds.
//!
//! WHAT MAKES `skip4` THE HARD ONE (measured, not inferred): it contests FOUR
//! earlier buses at once, so reversing one ordering relationship — which is all
//! rip-up-and-retry does — still leaves three of them ahead of it. It is the
//! case that distinguishes reordering from negotiation.

#![cfg(feature = "routing")]

use nucleation::design::{BusState, BusStyle, Design};
use nucleation::io_contract::IoType;
use nucleation::UniversalSchematic;

const STONE: &str = "minecraft:stone";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";

fn ty8() -> IoType {
    IoType::UnsignedInt { bits: 8 }
}

/// A solid cell body with an 8-bit input port on -X and an output port on +X,
/// both on a 2y pitch — `design_routability`'s `dust_cell`, verbatim.
fn dust_cell(
    name: &str,
    sx: i32,
    sz: i32,
) -> (UniversalSchematic, nucleation::io_contract::CellContract) {
    use nucleation::io_contract::{CellContract, IoLayoutBuilder, LayoutFunction};
    let mut s = UniversalSchematic::new(name.to_string());
    for x in 0..sx {
        for z in 0..sz {
            for y in 0..18 {
                s.set_block_from_string(x, y, z, STONE).unwrap();
            }
        }
    }
    let mut ins = Vec::new();
    let mut outs = Vec::new();
    for k in 0..8i32 {
        let y = 2 + 2 * k;
        s.set_block_from_string(0, y, 1, DUST).unwrap();
        s.set_block_from_string(sx - 1, y, 1, DUST).unwrap();
        ins.push((0, y, 1));
        outs.push((sx - 1, y, 1));
    }
    let io = IoLayoutBuilder::new()
        .add_input("d", ty8(), LayoutFunction::OneToOne, ins)
        .unwrap()
        .add_output("q", ty8(), LayoutFunction::OneToOne, outs)
        .unwrap()
        .build();
    (s, CellContract::new(name, io))
}

/// The five skip hops. `order` lists the hop indices in declaration order, so
/// the same netlist can be declared in different orders. Returns the per-bus
/// outcome, named by hop index.
fn skip_row(order: &[usize]) -> Vec<(String, BusState)> {
    const HOPS: [(usize, usize); 5] = [(0, 2), (1, 3), (2, 4), (3, 0), (4, 1)];
    let mut d = Design::new("row2");
    let (sch, contract) = dust_cell("blk", 10, 4);
    d.add_cell_with_contract("blk", sch, contract);
    for k in 0..5 {
        d.place(format!("u{k}"), "blk", (24 * k, 0, 0), 0).unwrap();
    }
    let mut out = Vec::new();
    for &i in order {
        let (a, b) = HOPS[i];
        let name = format!("skip{i}");
        let state = d
            .route_bus(
                &name,
                &format!("u{a}.q"),
                &[format!("u{b}.d").as_str()],
                vec![],
                BusStyle::default(),
            )
            .unwrap_or_else(|e| BusState::Failed(format!("declaration refused: {e}")));
        out.push((name, state));
    }
    out
}

fn routed_count(res: &[(String, BusState)]) -> usize {
    res.iter()
        .filter(|(_, s)| matches!(s, BusState::Routed))
        .count()
}

/// The gate: how many of the five route, and which one does not.
///
/// Pinned at the MEASURED number rather than at five, so the file records what
/// the router actually does and a change either way is visible. `skip4` is the
/// residual; see the module docs for why.
#[test]
fn the_skip_row_routes_all_but_the_four_way_contender() {
    let res = skip_row(&[0, 1, 2, 3, 4]);
    let routed: Vec<&String> = res
        .iter()
        .filter(|(_, s)| matches!(s, BusState::Routed))
        .map(|(n, _)| n)
        .collect();
    let failed: Vec<(&String, &BusState)> = res
        .iter()
        .filter(|(_, s)| !matches!(s, BusState::Routed))
        .map(|(n, s)| (n, s))
        .collect();
    println!("SKIP|routed|{}/{}", routed.len(), res.len());
    for (n, s) in &failed {
        println!("SKIP|fail|{n}|{s:?}");
    }
    assert!(
        routed.len() >= 4,
        "the skip row lost ground: only {:?} routed, failures {:#?}",
        routed,
        failed
    );
}

/// THE RESIDUE IS TOPOLOGICAL, NOT ORDERING — and this is the measurement that
/// settles it, because it is the one experiment that reordering has to survive.
///
/// All ten terminals of this netlist sit on the single line `z = 1`, and a
/// single-level bus may route on either side of it (`z < 0` or `z > 3`). That
/// makes a crossing-free solution exactly a TWO-PAGE BOOK EMBEDDING of the five
/// chords `[9,48] [33,72] [57,96] [0,81] [24,105]`: two buses may share a side
/// only if their chords do not interleave. Those five chords interleave in a
/// 5-CYCLE (skip0-skip1-skip2-skip3-skip4-skip0), and an odd cycle is not
/// 2-colourable, so at least one crossing is unavoidable for EVERY declaration
/// order.
///
/// So the prediction is sharp: no order routes all five, and reordering only
/// changes WHICH bus is left over. That is why rip-up-and-retry recovered `g2`
/// and `g5` and could never recover `skip4`, why a 1.5M-node search rung
/// recovered zero, and why negotiated congestion — which drives this group down
/// to exactly ONE contested cell and no further, from any starting point — also
/// cannot. 4/5 is the ceiling for a crossing-free single-level repertoire.
///
/// Reaching 5/5 needs the obstruction BROKEN, not searched harder: one bus
/// crossing another (the crossing station exists — `tests/design_bus_cross.rs`)
/// or one bus on another level.
#[test]
fn one_crossing_is_unavoidable_whatever_the_declaration_order() {
    // Declaration-order-first, and the reverse; and skip4 (the usual loser)
    // first, which is the order rip-up-and-retry is trying to reach.
    for order in [
        vec![0, 1, 2, 3, 4],
        vec![4, 3, 2, 1, 0],
        vec![4, 0, 1, 2, 3],
        vec![2, 4, 0, 3, 1],
    ] {
        let res = skip_row(&order);
        let n = routed_count(&res);
        let losers: Vec<&str> = res
            .iter()
            .filter(|(_, s)| !matches!(s, BusState::Routed))
            .map(|(n, _)| n.as_str())
            .collect();
        println!("SKIP|order|{order:?}|routed {n}/5|left over {losers:?}");
        assert!(
            n < 5,
            "order {order:?} routed all five, which contradicts the 2-page \
             obstruction — if this fires, a crossing or a level change was \
             introduced and this test should be re-derived, not deleted"
        );
        assert_eq!(
            n, 4,
            "order {order:?} routed only {n}/5; one crossing is unavoidable but \
             only one, so anything below 4 is a regression. Left over: {losers:?}"
        );
    }
}
