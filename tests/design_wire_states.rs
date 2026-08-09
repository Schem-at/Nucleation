//! ROUTED REDSTONE MUST DRAW AS WIRE, NOT DOTS.
//!
//! User report: "the redstone all places as dots". Every dust cell a design
//! authors was written in the fully-spelled-out DEFAULT state
//! (`east=none,north=none,south=none,west=none`). That is deliberate for
//! INTERNING — a bare `minecraft:redstone_wire` interns a property-less state
//! that tick engines never normalise, and those cells sit inert (the
//! long-standing trap documented on `rblocks::DUST` / `rs.DUST`) — but a wire
//! with four `none` sides is, to the renderer and the model files, a DOT. So a
//! correct route looked like a trail of unconnected pips, and exported or
//! rendered designs looked broken.
//!
//! The fix derives the connection state GEOMETRICALLY at emit time, exactly as
//! Minecraft does on placement (`nucleation_routing::wire`). No simulation, no
//! bake. These tests pin the appearance of the shapes a bus actually makes.

#![cfg(feature = "routing")]

use nucleation::design::{BusState, BusStyle, Design};
use nucleation::io_contract::IoType;
use nucleation::UniversalSchematic;

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
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE).unwrap();
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

fn one_bus(a: P3, b: P3) -> Design {
    let mut s = UniversalSchematic::new("wire".to_string());
    let (dx, dz) = if (b.0 - a.0).abs() >= (b.2 - a.2).abs() {
        ((b.0 - a.0).signum(), 0)
    } else {
        (0, (b.2 - a.2).signum())
    };
    let drv = lever_bank(&mut s, a.0 - dx, a.1, a.2 - dz, dx, dz);
    let snk = lamp_bank(&mut s, b.0, b.1, b.2);
    let mut d = Design::for_schematic("wire", s);
    d.declare_input("din", drv, (0, 2, 0), W, ty()).unwrap();
    d.declare_output("dout", snk, (0, 2, 0), W, ty()).unwrap();
    let st = d
        .route_bus("a", "din", &["dout"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(st, BusState::Routed, "{:?}", d.bus_state("a"));
    d
}

/// Every dust cell in a routed fragment that has a wire neighbour must be
/// CONNECTED to it. A fragment full of `all-none` dust is the bug.
fn assert_no_dots(d: &Design, bus: &str, what: &str) {
    let frag = &d.bus(bus).unwrap().fragment;
    let is_dust = |b: &String| b.contains("redstone_wire");
    let mut dots = Vec::new();
    for (p, blk) in frag.iter().filter(|(_, b)| is_dust(b)) {
        let all_none = blk.contains("east=none")
            && blk.contains("north=none")
            && blk.contains("south=none")
            && blk.contains("west=none");
        if !all_none {
            continue;
        }
        // A dot is only legal with nothing to connect to.
        let lonely = [(1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)]
            .iter()
            .all(|(dx, dy, dz)| {
                [(*dx, *dy, *dz), (*dx, 1, *dz), (*dx, -1, *dz)].iter().all(|(ax, ay, az)| {
                    !frag
                        .get(&(p.0 + ax, p.1 + ay, p.2 + az))
                        .is_some_and(is_dust)
                })
            });
        if !lonely {
            dots.push(*p);
        }
    }
    assert!(
        dots.is_empty(),
        "{what}: {} dust cell(s) drew as DOTS despite having wire neighbours, e.g. {:?}",
        dots.len(),
        &dots[..dots.len().min(6)]
    );
}

/// A straight run's middle cell reads `east=side,west=side` — the shape the
/// report was about.
#[test]
fn a_straight_run_draws_as_a_line() {
    let d = one_bus((1, 2, 8), (32, 2, 8));
    let frag = &d.bus("a").unwrap().fragment;
    // Bit 0 runs along +X at y=2, z=8; take a cell well inside the run.
    let mid = frag
        .iter()
        .find(|(p, b)| p.1 == 2 && p.2 == 8 && p.0 > 8 && p.0 < 24 && b.contains("redstone_wire"))
        .map(|(p, b)| (*p, b.clone()))
        .expect("no dust in the middle of the run");
    assert!(mid.1.contains("east=side"), "{:?} {}", mid.0, mid.1);
    assert!(mid.1.contains("west=side"), "{:?} {}", mid.0, mid.1);
    assert!(mid.1.contains("north=none"), "{:?} {}", mid.0, mid.1);
    assert!(mid.1.contains("south=none"), "{:?} {}", mid.0, mid.1);
    assert_no_dots(&d, "a", "straight run");
}

/// An L corner reads exactly its two sides.
#[test]
fn an_l_corner_draws_its_two_sides() {
    let d = one_bus((1, 2, 8), (32, 2, 32));
    let frag = &d.bus("a").unwrap().fragment;
    // The corner is the one bit-0 dust cell with both an x- and a z-neighbour.
    let corner = frag
        .iter()
        .filter(|(p, b)| p.1 == 2 && b.contains("redstone_wire"))
        .find(|(p, _)| {
            let dust_at = |q: P3| frag.get(&q).is_some_and(|b| b.contains("redstone_wire"));
            (dust_at((p.0 - 1, p.1, p.2)) || dust_at((p.0 + 1, p.1, p.2)))
                && (dust_at((p.0, p.1, p.2 - 1)) || dust_at((p.0, p.1, p.2 + 1)))
        })
        .map(|(p, b)| (*p, b.clone()))
        .expect("no corner cell found on an L route");
    let sides = ["north", "east", "south", "west"]
        .iter()
        .filter(|s| corner.1.contains(&format!("{s}=side")) || corner.1.contains(&format!("{s}=up")))
        .count();
    assert_eq!(
        sides, 2,
        "the corner at {:?} drew {sides} side(s), not 2: {}",
        corner.0, corner.1
    );
    assert_no_dots(&d, "a", "L corner");
}

/// The level-shift staircase is the hardest case: 1y steps, glass supports, and
/// a bit stacked directly above every support. Nothing may draw as a dot, and
/// the climbs must be drawn as climbs.
#[test]
fn a_level_shift_draws_its_climbs() {
    for (ya, yb, what) in [(2, 5, "ascending"), (6, 2, "descending")] {
        let d = one_bus((1, ya, 8), (40, yb, 8));
        assert_no_dots(&d, "a", what);
        let frag = &d.bus("a").unwrap().fragment;
        let climbs = frag
            .values()
            .filter(|b| b.contains("redstone_wire") && b.contains("=up"))
            .count();
        let diagonals = frag
            .iter()
            .filter(|(p, b)| {
                b.contains("redstone_wire")
                    && [(1, 1, 0), (-1, 1, 0), (0, 1, 1), (0, 1, -1)].iter().any(|(dx, dy, dz)| {
                        frag.get(&(p.0 + dx, p.1 + dy, p.2 + dz))
                            .is_some_and(|n| n.contains("redstone_wire"))
                    })
            })
            .count();
        assert!(
            diagonals > 0,
            "{what}: the level shift has no 1y diagonals at all — nothing to draw"
        );
        assert!(
            climbs > 0,
            "{what}: {diagonals} diagonal(s) but not one drawn as a climb (`=up`)"
        );
    }
}

/// A crossing dip-under, and the through-bus station stamped into the bus it
/// crosses: the amendment removes dust, which changes its NEIGHBOURS' states.
#[test]
fn a_crossing_redraws_both_buses() {
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
    for (n, drv, snk) in [("a", "ain", "aout"), ("b", "bin", "bout")] {
        let st = d.route_bus(n, drv, &[snk], vec![], BusStyle::default()).unwrap();
        assert_eq!(st, BusState::Routed, "{n}: {:?}", d.bus_state(n));
    }
    // Bus `a` was AMENDED by `b`'s crossing after it was already drawn.
    assert_no_dots(&d, "a", "the crossed bus, after the amendment");
    assert_no_dots(&d, "b", "the crossing bus");
    assert!(d.check().unwrap().clean, "the crossing is not clean");
}

/// A promoted port's own dust is authored by PROMOTION, not the bus, and must
/// be drawn too.
#[test]
fn a_promoted_port_draws_as_wire() {
    let mut s = UniversalSchematic::new("prom".to_string());
    let din = lever_bank(&mut s, 0, 2, 8, 1, 0);
    // A tiny cell: a 4-bit lever column input, dust-connected internally.
    let mut cell = UniversalSchematic::new("c".to_string());
    let d_hw: Vec<P3> = (0..W as i32).map(|i| (0, 2 + 2 * i, 0)).collect();
    let q_hw: Vec<P3> = (0..W as i32).map(|i| (3, 1 + 2 * i, 0)).collect();
    for i in 0..W as i32 {
        let y = 2 + 2 * i;
        cell.set_block_from_string(0, y - 1, 0, STONE).unwrap();
        cell.set_block_from_string(0, y, 0, LEVER).unwrap();
        cell.set_block_from_string(3, y - 1, 0, LAMP).unwrap();
        cell.set_block_from_string(3, y, 0, DUST).unwrap();
    }
    let layout = nucleation::io_contract::IoLayoutBuilder::new()
        .add_input(
            "d".to_string(),
            ty(),
            nucleation::io_contract::LayoutFunction::OneToOne,
            d_hw,
        )
        .unwrap()
        .add_output(
            "q".to_string(),
            ty(),
            nucleation::io_contract::LayoutFunction::OneToOne,
            q_hw,
        )
        .unwrap()
        .build();
    cell.set_cell_contract(&nucleation::io_contract::CellContract::new("c".to_string(), layout))
        .unwrap();

    let mut d = Design::for_schematic("prom", s);
    d.add_cell("c", cell).unwrap();
    d.place("u0", "c", (24, 0, 8), 0).unwrap();
    d.declare_input("din", din, (0, 2, 0), W, ty()).unwrap();
    let st = d
        .route_bus("a", "din", &["u0.d"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(st, BusState::Routed, "{:?}", d.bus_state("a"));
    assert_no_dots(&d, "a", "bus into a promoted port");

    // The promoted cell's own dust, in the flattened artifact, must connect to
    // the bus that lands on it — a dot there is the same bug one layer over.
    let flat = d.flatten().unwrap();
    let anchor = d.resolve_port("u0.d").unwrap().anchor;
    let at = flat
        .get_block(anchor.0, anchor.1, anchor.2)
        .map(|b| b.to_string())
        .unwrap_or_default();
    assert!(at.contains("redstone_wire"), "the promoted port is not dust: {at}");
    assert!(
        !(at.contains("east=none")
            && at.contains("north=none")
            && at.contains("south=none")
            && at.contains("west=none")),
        "the promoted port's dust drew as a DOT even with the bus landing on it: {at}"
    );
}
