//! Two composite-document bugs that made a `Design` look broken from outside.
//!
//! 1. `get_block` precedence — the default region's allocated BOUNDING BOX used
//!    to shadow every named layer inside it, so a composite's `inst:*` /
//!    `bus:*` layers were invisible to point queries even though `iter_blocks`
//!    and the export path could see them. FIXED: the masking volume is now
//!    `Region::get_tight_bounds()`, the extent of the blocks a region actually
//!    holds. The three facts that made the bug:
//!
//!    - `Region::get_bounding_box()` reports a region's ALLOCATED envelope,
//!      not its contents — a 10x18x6 schematic reports `(0,0,0)..(10,65,65)`.
//!    - a region densely materializes air, answering `Some(air)` for every
//!      in-bounds cell it was never written to.
//!    - so the default region masked that entire phantom envelope.
//!
//!    Making air transparent in `get_block` would have fixed composites but
//!    broken `main_air_masks` in
//!    `universal_schematic::tests::overlapping_named_regions_have_stable_lexicographic_precedence`,
//!    which deliberately asserts that Main's air DOES mask a named layer.
//!    Tight bounds satisfy both: air BETWEEN two of Main's own blocks is inside
//!    Main's tight bounds and still masks, while an envelope Main never wrote to
//!    is outside them and masks nothing.
//!
//!    What tight bounds do NOT fix is dense air INSIDE a region's true extent,
//!    which is the one case `main_air_masks` pins to the opposite answer. See
//!    `a_routed_design_is_point_queryable_through_its_bus_layer`, still ignored,
//!    for the measurement and why it needs its own decision.
//!
//!    This is the THIRD defect from the allocated-envelope class. The first was
//!    every ROTATED instance landing tens of blocks off, which `Design`
//!    sidesteps in `design.rs::cell_bounds` by computing the true extent
//!    itself. Anything sizing, rotating or transforming content wants tight
//!    bounds; only storage/allocation questions want the envelope.
//! 2. `check()` blamed the design for a LIBRARY CELL's interior. Hand-built
//!    community redstone breaks the route-oriented DRC conventions by design
//!    (one 8-bit community adder reports hundreds of "floating" cells), so
//!    `clean` could never be true once a real cell was placed. Cells are
//!    verified black boxes behind their keepouts; their interiors are now
//!    reported under `cells`, informationally.

#![cfg(feature = "routing")]

use nucleation::design::{BusState, BusStyle, Design};
use nucleation::io_contract::{CellContract, IoLayoutBuilder, IoType, LayoutFunction};
use nucleation::UniversalSchematic;

const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";

#[test]
fn a_named_layer_is_visible_through_the_default_regions_envelope() {
    let mut s = UniversalSchematic::new("composite".to_string());
    // A default-region block far out, so the default region's envelope grows
    // to cover the origin area without holding anything there.
    s.set_block_from_string(80, 40, 80, STONE).unwrap();
    // A named layer inside that envelope.
    assert!(s.set_block_in_region_str("inst:u0", 4, 5, 6, STONE));

    assert_eq!(
        s.get_block(4, 5, 6).map(|b| b.name.to_string()).as_deref(),
        Some("minecraft:stone"),
        "a named layer inside the default region's envelope must still be visible"
    );
    // And the default region's own block still wins where it has one.
    assert_eq!(
        s.get_block(80, 40, 80)
            .map(|b| b.name.to_string())
            .as_deref(),
        Some("minecraft:stone")
    );
    // A cell nobody wrote is still empty, not a phantom.
    let empty = s.get_block(4, 5, 7).map(|b| b.name.to_string());
    assert!(
        empty.is_none() || empty.as_deref() == Some("minecraft:air"),
        "unwritten cell answered {empty:?}"
    );
}

/// Regression pin for the allocated-envelope defect class, stating BOTH halves
/// of the contract in one place so a future "simplification" of `get_block`
/// cannot satisfy one by breaking the other.
///
/// Half A: air a region actually surrounds DOES mask the layers below it.
/// Half B: an envelope a region never wrote to masks NOTHING.
#[test]
fn masking_follows_tight_bounds_not_the_allocated_envelope() {
    // Half A — the `main_air_masks` contract. Main holds blocks either side of
    // (10,0,0) but not the cell itself, so (10,0,0) is inside Main's TIGHT
    // bounds and Main's air wins over the named layer.
    let mut air_masks = UniversalSchematic::new("air-masks".to_string());
    assert!(air_masks.set_block_in_region_str("inst:u0", 10, 0, 0, STONE));
    air_masks.set_block_from_string(9, 0, 0, STONE).unwrap();
    air_masks.set_block_from_string(11, 0, 0, STONE).unwrap();
    assert_eq!(
        air_masks
            .get_block(10, 0, 0)
            .map(|b| b.name.to_string())
            .as_deref(),
        Some("minecraft:air"),
        "air a region surrounds must still mask the layer below it"
    );

    // Half B — the phantom envelope. A region's storage is padded when it GROWS,
    // so two blocks at (0,0,0) and (80,40,80) allocate out to (144,104,144)
    // while the true extent stops at (80,40,80). (100,50,100) is inside the
    // allocated envelope and outside the content: pure phantom.
    let phantom_cell = (100, 50, 100);
    let mut phantom = UniversalSchematic::new("phantom".to_string());
    phantom.set_block_from_string(0, 0, 0, STONE).unwrap();
    phantom.set_block_from_string(80, 40, 80, STONE).unwrap();
    assert!(phantom.set_block_in_region_str(
        "bus:net",
        phantom_cell.0,
        phantom_cell.1,
        phantom_cell.2,
        STONE
    ));
    assert!(
        phantom
            .default_region
            .get_bounding_box()
            .contains(phantom_cell),
        "precondition: the allocated envelope must cover the queried cell, or \
         this test is not exercising the defect"
    );
    assert_eq!(
        phantom
            .default_region
            .get_tight_bounds()
            .map(|bb| bb.contains(phantom_cell)),
        Some(false),
        "precondition: tight bounds must NOT cover it"
    );
    assert_eq!(
        phantom
            .get_block(phantom_cell.0, phantom_cell.1, phantom_cell.2)
            .map(|b| b.name.to_string())
            .as_deref(),
        Some("minecraft:stone"),
        "a phantom envelope must not mask a named layer"
    );

    // A region holding nothing at all claims nothing at all.
    let mut empty_main = UniversalSchematic::new("empty-main".to_string());
    assert!(empty_main.set_block_in_region_str("inst:u0", 0, 0, 0, STONE));
    assert_eq!(
        empty_main
            .get_block(0, 0, 0)
            .map(|b| b.name.to_string())
            .as_deref(),
        Some("minecraft:stone")
    );
}

#[test]
#[ignore = "open, and NOT the same bug as above — see the comment in the body. Needs a \
            product decision on whether UNWRITTEN air masks, which directly contradicts \
            the `main_air_masks` contract. Workaround: query `bus:*` by region."]
fn a_routed_design_is_point_queryable_through_its_bus_layer() {
    // WHY THIS IS STILL IGNORED, precisely (measured 2026-08-09).
    //
    // Tight-bounds masking fixed the PHANTOM ENVELOPE half of this bug class:
    // a region no longer masks an area it never wrote to. This case is the
    // other half, and it is a genuine contract conflict, not an oversight:
    //
    //   - The loose layer legitimately holds blocks at x=0..1 and x=24, y=1..16,
    //     z=8, so its TIGHT bounds envelop the whole bus corridor. The bus is
    //     routed through cells like (2,1,8) that are inside those tight bounds
    //     and that the loose layer never wrote — dense air.
    //   - `main_air_masks` in
    //     `universal_schematic::tests::overlapping_named_regions_have_stable_lexicographic_precedence`
    //     asserts that Main's air at (10,0,0) — ALSO never written, also merely
    //     inside Main's tight bounds — DOES mask a named layer.
    //
    // Those are the same configuration with opposite expected answers, so no
    // masking volume can satisfy both. `Region` cannot tell written air from
    // dense air either: `set_block` only extends tight bounds for non-air
    // (src/region.rs:226), and nothing records explicit air writes. Resolving
    // this means deciding which contract wins and paying the blast radius
    // (export formats, meshing, stamping) — a reviewable change of its own.
    //
    // Until then `Design` consumers read a bus layer by name rather than by
    // point query, which is unambiguous and needs no precedence rule.
    // The end-to-end shape of bug 1: a routed bus lives in region `bus:*`,
    // inside the loose layer's envelope.
    let mut s = UniversalSchematic::new("d".to_string());
    for i in 0..8i32 {
        let y = 2 + 2 * i;
        s.set_block_from_string(0, y - 1, 8, STONE).unwrap();
        s.set_block_from_string(0, y, 8, LEVER).unwrap();
        s.set_block_from_string(1, y - 1, 8, STONE).unwrap();
        s.set_block_from_string(1, y, 8, DUST).unwrap();
        s.set_block_from_string(24, y - 1, 8, LAMP).unwrap();
        s.set_block_from_string(24, y, 8, DUST).unwrap();
    }
    let mut d = Design::for_schematic("d", s);
    let step = (0, 2, 0);
    let ty = IoType::UnsignedInt { bits: 8 };
    d.declare_input("din", (1, 2, 8), step, 8, ty.clone())
        .unwrap();
    d.declare_output("dout", (24, 2, 8), step, 8, ty).unwrap();
    assert_eq!(
        d.route_bus("net", "din", &["dout"], vec![], BusStyle::default())
            .unwrap(),
        BusState::Routed
    );

    let flat = d.flatten().unwrap();
    // Pick a cell the bus owns and query it by point.
    let (p, want) = d
        .bus("net")
        .unwrap()
        .fragment
        .iter()
        .next()
        .map(|(p, b)| (*p, b.clone()))
        .unwrap();
    let got = flat.get_block(p.0, p.1, p.2).map(|b| b.to_string());
    assert!(
        got.as_deref()
            .is_some_and(|g| want.starts_with(g.split('[').next().unwrap_or(g))
                || g.starts_with(want.split('[').next().unwrap_or(&want))),
        "bus cell {p:?} should read back as `{want}`, got {got:?}"
    );
}

/// A cell whose interior deliberately breaks the route-oriented DRC rules:
/// floating dust with nothing beneath it, the shape hand-built community
/// redstone hits constantly.
fn messy_cell() -> (UniversalSchematic, CellContract) {
    let mut s = UniversalSchematic::new("messy".to_string());
    for x in 0..8 {
        for z in 0..4 {
            for y in 0..18 {
                s.set_block_from_string(x, y, z, STONE).unwrap();
            }
        }
    }
    // Floating dust inside the body: supported by nothing (air below).
    for k in 0..6i32 {
        s.set_block_from_string(3, 6 + k, 2, "minecraft:air")
            .unwrap();
        s.set_block_from_string(4, 6 + k, 2, "minecraft:air")
            .unwrap();
    }
    s.set_block_from_string(4, 8, 2, DUST).unwrap();
    s.set_block_from_string(4, 10, 2, DUST).unwrap();

    let mut ins = Vec::new();
    let mut outs = Vec::new();
    for k in 0..8i32 {
        let y = 2 + 2 * k;
        s.set_block_from_string(0, y, 1, DUST).unwrap();
        s.set_block_from_string(7, y, 1, DUST).unwrap();
        ins.push((0, y, 1));
        outs.push((7, y, 1));
    }
    let ty = IoType::UnsignedInt { bits: 8 };
    let io = IoLayoutBuilder::new()
        .add_input("d", ty.clone(), LayoutFunction::OneToOne, ins)
        .unwrap()
        .add_output("q", ty, LayoutFunction::OneToOne, outs)
        .unwrap()
        .build();
    (s, CellContract::new("messy", io))
}

#[test]
fn a_library_cells_interior_does_not_make_the_design_dirty() {
    let mut d = Design::new("t");
    let (sch, contract) = messy_cell();
    d.add_cell_with_contract("messy", sch, contract);
    d.place("u0", "messy", (0, 0, 0), 0).unwrap();

    let check = d.check().unwrap();
    assert!(
        check.clean,
        "a placed library cell's interior must not gate the design: {}",
        check.json
    );
    // ...but it is still REPORTED, under `cells`, so nothing is swept away.
    assert!(
        check.json.contains("\"cells\":[{"),
        "interior findings must still be reported: {}",
        check.json
    );
    assert!(check.json.contains("floating"), "{}", check.json);
}

/// The real thing: community cells from the enhanced corpus, placed, no buses.
/// Before the cell-boundary split this reported hundreds of DRC violations plus
/// an LVS "accidental latch" for every register (a latch IS a repeater ring), so
/// `check()` could never come back green once a real cell was on the canvas.
#[test]
fn real_community_cells_placed_with_no_buses_check_clean() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("computational_schematics/enhanced");
    let files = [
        "ADD005_8bit_cle_enhanced.schem",
        "REGISTER001_8bit_register_enhanced.schem",
    ];
    let mut d = Design::new("community");
    let mut placed = 0;
    let mut x = 0i32;
    for (i, f) in files.iter().enumerate() {
        let Ok(bytes) = std::fs::read(dir.join(f)) else {
            continue;
        };
        let Ok(sch) = nucleation::formats::schematic::from_schematic(&bytes) else {
            continue;
        };
        let name = format!("c{i}");
        if d.add_cell(&name, sch).is_err() {
            continue;
        }
        if d.place(format!("u{i}"), &name, (x, 0, 0), 0).is_ok() {
            placed += 1;
        }
        x += 80;
    }
    if placed == 0 {
        eprintln!("enhanced corpus unavailable; skipping");
        return;
    }
    let check = d.check().unwrap();
    assert!(
        check.clean,
        "{placed} community cell(s) with no buses must check clean: {}",
        check.json
    );
    // The interior findings are still surfaced, just not as design errors.
    assert!(
        check.json.contains("\"cells\":[{"),
        "interior findings must still be reported"
    );
}

#[test]
fn a_violation_the_design_itself_owns_still_gates() {
    // Floating dust on the LOOSE layer is the design's own doing and must
    // still turn `clean` off.
    let mut s = UniversalSchematic::new("d".to_string());
    s.set_block_from_string(40, 30, 40, DUST).unwrap();
    let d = Design::for_schematic("d", s);
    let check = d.check().unwrap();
    assert!(
        !check.clean,
        "the design's own floating dust must gate: {}",
        check.json
    );
    assert!(check.json.contains("floating"), "{}", check.json);
}
