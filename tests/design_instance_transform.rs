//! A placed instance's derived ports must land INSIDE that instance's own
//! footprint — at every rotation.
//!
//! Regression: `Design` derived a cell's rotation footprint from
//! `UniversalSchematic::get_bounding_box()`, which reports a programmatically
//! built schematic's allocated envelope rather than its contents (a 10x18x6
//! cell reports `(0,0,0)..(10,65,65)`). Rotation size comes from that box, so
//! every instance at rot_y 90/180/270 had its blocks, ports and influence halo
//! mapped tens of blocks away from where it was placed, while rot_y=0 looked
//! correct. Buses between rotated cells then failed for reasons that named
//! coordinates nowhere near the design.

#![cfg(feature = "routing")]

use nucleation::design::{Design, Occupant};
use nucleation::io_contract::{CellContract, IoLayoutBuilder, IoType, LayoutFunction};
use nucleation::UniversalSchematic;

const STONE: &str = "minecraft:stone";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";

const SX: i32 = 10;
const SY: i32 = 18;
const SZ: i32 = 6;

/// A solid cell with an 8-bit dust port on each of its -X and +X faces.
fn dust_cell() -> (UniversalSchematic, CellContract) {
    let mut s = UniversalSchematic::new("blk".to_string());
    for x in 0..SX {
        for z in 0..SZ {
            for y in 0..SY {
                s.set_block_from_string(x, y, z, STONE).unwrap();
            }
        }
    }
    let mut ins = Vec::new();
    let mut outs = Vec::new();
    for k in 0..8i32 {
        let y = 2 + 2 * k;
        s.set_block_from_string(0, y, 1, DUST).unwrap();
        s.set_block_from_string(SX - 1, y, 1, DUST).unwrap();
        ins.push((0, y, 1));
        outs.push((SX - 1, y, 1));
    }
    let ty = IoType::UnsignedInt { bits: 8 };
    let io = IoLayoutBuilder::new()
        .add_input("d", ty.clone(), LayoutFunction::OneToOne, ins)
        .unwrap()
        .add_output("q", ty, LayoutFunction::OneToOne, outs)
        .unwrap()
        .build();
    (s, CellContract::new("blk", io))
}

#[test]
fn derived_ports_land_inside_the_instance_footprint_at_every_rotation() {
    let at = (100, 0, 100);
    for rot in [0, 90, 180, 270] {
        let mut d = Design::new("t");
        let (sch, contract) = dust_cell();
        d.add_cell_with_contract("blk", sch, contract);
        d.place("u", "blk", at, rot).unwrap();

        // The instance's ACTUAL occupied cells.
        let occ = d.occupancy_index();
        let body: Vec<(i32, i32, i32)> = occ
            .cells
            .iter()
            .filter(|(_, (_, o))| matches!(o, Occupant::Instance(n) if n == "u"))
            .map(|(p, _)| *p)
            .collect();
        assert!(!body.is_empty(), "rot {rot}: instance has no footprint");
        let lo = body.iter().fold((i32::MAX, i32::MAX, i32::MAX), |a, p| {
            (a.0.min(p.0), a.1.min(p.1), a.2.min(p.2))
        });
        let hi = body.iter().fold((i32::MIN, i32::MIN, i32::MIN), |a, p| {
            (a.0.max(p.0), a.1.max(p.1), a.2.max(p.2))
        });

        // The footprint must be the cell's real size (X/Z swapped on a quarter
        // turn), anchored at `at` — not an inflated envelope.
        let (ex, ez) = if rot % 180 == 0 { (SX, SZ) } else { (SZ, SX) };
        assert_eq!(
            (hi.0 - lo.0 + 1, hi.1 - lo.1 + 1, hi.2 - lo.2 + 1),
            (ex, SY, ez),
            "rot {rot}: footprint size"
        );
        assert_eq!(lo, at, "rot {rot}: footprint is not anchored at `at`");

        // Every derived port wire must sit inside that footprint.
        for p in d.instance_ports().unwrap() {
            let wires = p.wires.unwrap_or_else(|| {
                panic!("rot {rot}: port {} lost its dust taps: {:?}", p.name, p.blocked)
            });
            for w in wires {
                assert!(
                    (lo.0..=hi.0).contains(&w.0)
                        && (lo.1..=hi.1).contains(&w.1)
                        && (lo.2..=hi.2).contains(&w.2),
                    "rot {rot}: port {} wire {w:?} is outside the instance footprint \
                     {lo:?}..{hi:?}",
                    p.name
                );
            }
        }
    }
}

#[test]
fn a_bus_between_two_rotated_instances_routes() {
    let mut d = Design::new("t");
    let (sch, contract) = dust_cell();
    d.add_cell_with_contract("blk", sch, contract);
    // Face the two cells' ports at each other across a clear gap.
    d.place("a", "blk", (0, 0, 0), 0).unwrap();
    d.place("b", "blk", (40, 0, 0), 180).unwrap();
    let state = d
        .route_bus("net", "a.q", &["b.d"], vec![], nucleation::design::BusStyle::default())
        .unwrap();
    assert_eq!(
        state,
        nucleation::design::BusState::Routed,
        "{:?}",
        d.bus_state("net")
    );
    let check = d.check().unwrap();
    assert!(check.clean, "{}", check.json);
}
