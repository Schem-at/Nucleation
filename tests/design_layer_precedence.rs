//! Two composite-document bugs that made a `Design` look broken from outside.
//!
//! 1. `get_block` precedence — the default region's allocated BOUNDING BOX
//!    shadows every named layer inside it, so a composite's `inst:*` /
//!    `bus:*` layers are invisible to point queries even though `iter_blocks`
//!    and the export path can see them. STILL OPEN, and the two tests below
//!    are `#[ignore]`d executable specifications of it rather than passing
//!    assertions, because the fix does not belong in `get_block`:
//!
//!    - `Region::get_bounding_box()` reports a region's ALLOCATED envelope,
//!      not its contents — a 10x18x6 schematic reports `(0,0,0)..(10,65,65)`.
//!    - a region densely materializes air, answering `Some(air)` for every
//!      in-bounds cell it was never written to.
//!    - so the default region masks that entire phantom envelope.
//!
//!    Making air transparent in `get_block` would fix composites but breaks
//!    `main_air_masks` in
//!    `universal_schematic::tests::overlapping_named_regions_have_stable_lexicographic_precedence`,
//!    which deliberately asserts that Main's air DOES mask a named layer. Both
//!    can only hold once `Region::get_bounding_box` reports the true block
//!    extent, which is a core change with a wide blast radius (export formats,
//!    meshing, stamping) and wants its own review. `Design` already sidesteps
//!    it: `design.rs::cell_bounds` computes the true extent itself, without
//!    which every ROTATED instance was placed tens of blocks off.
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
#[ignore = "open: Region::get_bounding_box reports the allocated envelope, not the \
            true block extent, so the default region's air masks named layers inside it. \
            See the module docs."]
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
        s.get_block(80, 40, 80).map(|b| b.name.to_string()).as_deref(),
        Some("minecraft:stone")
    );
    // A cell nobody wrote is still empty, not a phantom.
    let empty = s.get_block(4, 5, 7).map(|b| b.name.to_string());
    assert!(
        empty.is_none() || empty.as_deref() == Some("minecraft:air"),
        "unwritten cell answered {empty:?}"
    );
}

#[test]
#[ignore = "open: same phantom-envelope masking as above, seen end to end — a routed \
            bus layer is invisible to get_block inside the loose layer's envelope."]
fn a_routed_design_is_point_queryable_through_its_bus_layer() {
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
    d.declare_input("din", (1, 2, 8), step, 8, ty.clone()).unwrap();
    d.declare_output("dout", (24, 2, 8), step, 8, ty).unwrap();
    assert_eq!(
        d.route_bus("net", "din", &["dout"], vec![], BusStyle::default()).unwrap(),
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
        got.as_deref().is_some_and(|g| want.starts_with(g.split('[').next().unwrap_or(g))
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
        s.set_block_from_string(3, 6 + k, 2, "minecraft:air").unwrap();
        s.set_block_from_string(4, 6 + k, 2, "minecraft:air").unwrap();
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
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("computational_schematics/enhanced");
    let files = [
        "ADD005_8bit_cle_enhanced.schem",
        "REGISTER001_8bit_register_enhanced.schem",
    ];
    let mut d = Design::new("community");
    let mut placed = 0;
    let mut x = 0i32;
    for (i, f) in files.iter().enumerate() {
        let Ok(bytes) = std::fs::read(dir.join(f)) else { continue };
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
    assert!(!check.clean, "the design's own floating dust must gate: {}", check.json);
    assert!(check.json.contains("floating"), "{}", check.json);
}
