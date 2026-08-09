//! Phase 2 Lane A acceptance: the interference model + drag APIs of
//! `redstone-eda/DESIGN_SPEC.md` (sketches 2 and 3), multi-sink trunks and
//! the explicit wired-OR merge — all at the routing/LVS level (typed
//! executor verification lives in `tests/design_typed_drag.rs`).

#![cfg(feature = "routing")]

use nucleation::design::{BusState, BusStyle, Design, Gate, SegmentKind};
use nucleation::io_contract::{CellContract, IoLayoutBuilder, IoType, NetClassRule};
use nucleation::UniversalSchematic;

const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";

const N: u8 = 4;

/// N levers at 2y pitch, each with its connection dust one step in.
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

/// N lamps at 2y pitch, each lamp supporting its own connection dust.
fn lamp_bank(s: &mut UniversalSchematic, x: i32, z: i32) -> (i32, i32, i32) {
    for i in 0..N as i32 {
        let y = 2 + 2 * i;
        s.set_block_from_string(x, y - 1, z, LAMP).unwrap();
        s.set_block_from_string(x, y, z, DUST).unwrap();
    }
    (x, 2, z)
}

fn is_repeater(b: &str) -> bool {
    b.contains("minecraft:repeater")
}

/// DESIGN_SPEC acceptance sketch (2): dragging a gate rips and reroutes
/// EXACTLY the two adjacent segments; every other segment's cells are
/// untouched, and conduction still checks out afterwards.
#[test]
fn gate_drag_reroutes_exactly_two_segments() {
    let mut s = UniversalSchematic::new("drag".to_string());
    let a_in = lever_bank(&mut s, 0, 8, 1, 0);
    let a_out = lamp_bank(&mut s, 24, 8);
    let mut d = Design::for_schematic("drag", s);
    let ty = IoType::UnsignedInt { bits: N as usize };
    d.declare_input("a_in", a_in, (0, 2, 0), N, ty.clone()).unwrap();
    d.declare_output("a_out", a_out, (0, 2, 0), N, ty).unwrap();

    let gates = vec![
        Gate {
            name: "g0".to_string(),
            anchor: (8, 2, 8),
            step: (0, 2, 0),
        },
        Gate {
            name: "g1".to_string(),
            anchor: (16, 2, 8),
            step: (0, 2, 0),
        },
    ];
    let state = d
        .route_bus("bus_a", "a_in", &["a_out"], gates, BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed, "{:?}", d.bus_state("bus_a"));
    assert!(d.check().unwrap().clean);

    // The last segment (g1 -> a_out) must survive the drag untouched.
    let seg2_before = d
        .bus("bus_a")
        .unwrap()
        .segments
        .iter()
        .find(|s| s.kind == SegmentKind::Trunk(2))
        .unwrap()
        .cells
        .clone();

    let report = d.move_gate("bus_a", "g0", (8, 2, 12)).unwrap();
    assert_eq!(report.rerouted_segments, 2, "exactly the 2 adjacent segments");
    assert_eq!(report.state, BusState::Routed, "{:?}", report.state);

    let bus = d.bus("bus_a").unwrap();
    let seg2_after = bus
        .segments
        .iter()
        .find(|s| s.kind == SegmentKind::Trunk(2))
        .unwrap()
        .cells
        .clone();
    assert_eq!(seg2_before, seg2_after, "untouched segment kept its cells");

    // The old straight g0->g1 stretch is gone; the new dogleg exists.
    assert!(
        !bus.fragment.contains_key(&(12, 2, 8)),
        "old straight stretch was ripped"
    );
    assert!(
        bus.fragment.contains_key(&(8, 2, 10)),
        "new leg toward the dragged gate"
    );
    assert!(
        bus.fragment.contains_key(&(8, 2, 12)),
        "gate joint at the new anchor"
    );

    // Verified conduction after: DRC + LVS over the flattened artifact.
    let check = d.check().unwrap();
    assert!(check.clean, "{}", check.json);

    // The check report carries STA/skew for the bus.
    assert!(check.json.contains("\"sta\""), "{}", check.json);
    let skew = d.bus_skew_json("bus_a").unwrap();
    assert!(skew.contains("per_bit_rt"), "{skew}");

    // Net-class rule enforcement: an impossible delay budget turns the
    // report dirty with a named violation.
    d.set_bus_rule(
        "bus_a",
        NetClassRule {
            max_len_rt: Some(0),
            ..NetClassRule::default()
        },
    )
    .unwrap();
    let check = d.check().unwrap();
    assert!(!check.clean, "{}", check.json);
    assert!(check.json.contains("max_len_rt"), "{}", check.json);
}

/// An unroutable gate drag leaves the bus FAILED (reason names the
/// segment), never half-routed — and the gate anchor still moved.
#[test]
fn unroutable_gate_drag_fails_visibly() {
    let mut s = UniversalSchematic::new("dragfail".to_string());
    let a_in = lever_bank(&mut s, 0, 8, 1, 0);
    let a_out = lamp_bank(&mut s, 24, 8);
    let mut d = Design::for_schematic("dragfail", s);
    let ty = IoType::UnsignedInt { bits: N as usize };
    d.declare_input("a_in", a_in, (0, 2, 0), N, ty.clone()).unwrap();
    d.declare_output("a_out", a_out, (0, 2, 0), N, ty).unwrap();
    let gates = vec![Gate {
        name: "g0".to_string(),
        anchor: (12, 2, 8),
        step: (0, 2, 0),
    }];
    let state = d
        .route_bus("bus_a", "a_in", &["a_out"], gates, BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed);

    // A wall around the dogleg target: both corner choices collide.
    for y in 0..=10 {
        for x in 4..=20 {
            d.set_block((x, y, 12), STONE).unwrap();
            d.set_block((x, y, 14), STONE).unwrap();
        }
        for z in 9..=16 {
            d.set_block((4, y, z), STONE).unwrap();
            d.set_block((20, y, z), STONE).unwrap();
        }
    }
    let report = d.move_gate("bus_a", "g0", (12, 2, 13)).unwrap();
    match &report.state {
        BusState::Failed(reason) => {
            assert!(reason.contains("g0"), "{reason}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    // Never half-routed.
    assert!(d.bus("bus_a").unwrap().fragment.is_empty());
    // The gate anchor moved anyway (the document's truth); dragging it
    // back re-attempts and succeeds.
    let report = d.move_gate("bus_a", "g0", (12, 2, 8)).unwrap();
    assert_eq!(report.state, BusState::Routed, "{:?}", report.state);
    assert!(d.check().unwrap().clean);
}

/// DESIGN_SPEC acceptance sketch (3): a component dragged through the A bus
/// co-reroutes the affected set or fails VISIBLY; dragged away, the reroute
/// succeeds. The move itself always succeeds.
///
/// Both halves of "or" are exercised here. A 3x3x3 cube dropped on the line is
/// something the corridor search routes AROUND, so the bus survives the drag —
/// it used to fail, because the planner only knew a straight run and one L
/// corner. A wall that genuinely seals the corridor still fails visibly, with a
/// reason that names the blocker.
#[test]
fn component_drag_through_bus_reroutes_then_fails_visibly_when_sealed() {
    let mut s = UniversalSchematic::new("blocker".to_string());
    let a_in = lever_bank(&mut s, 0, 8, 1, 0);
    let a_out = lamp_bank(&mut s, 16, 8);
    let mut d = Design::for_schematic("blocker", s);
    let ty = IoType::UnsignedInt { bits: N as usize };
    d.declare_input("a_in", a_in, (0, 2, 0), N, ty.clone()).unwrap();
    d.declare_output("a_out", a_out, (0, 2, 0), N, ty).unwrap();

    // A 3x3x3 solid cell with an (empty) contract; halo derives from the
    // cell bounds + 1 because it declares no keepouts.
    let mut body = UniversalSchematic::new("cube".to_string());
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                body.set_block_from_string(x, y, z, STONE).unwrap();
            }
        }
    }
    let contract = CellContract::new("cube".to_string(), IoLayoutBuilder::new().build());
    d.add_cell_with_contract("cube", body, contract);
    d.place("c0", "cube", (4, 0, 20), 0).unwrap();

    let state = d
        .route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed);
    assert!(d.check().unwrap().clean);

    // Drag c0 THROUGH the bus: the move succeeds AND the bus survives, by
    // co-rerouting around the obstacle instead of dying on it.
    let report = d.move_instance("c0", (4, 0, 8), 0).unwrap();
    assert_eq!(report.rerouted, vec!["bus_a".to_string()], "{report:?}");
    assert!(report.failed.is_empty(), "{report:?}");
    assert_eq!(d.bus_state("bus_a"), Some(&BusState::Routed));
    assert!(d.check().unwrap().clean, "{}", d.check().unwrap().json);
    // The detour is real: nothing of the bus sits in c0's footprint or halo.
    for p in d.bus("bus_a").unwrap().fragment.keys() {
        let inside = (3..=7).contains(&p.0) && (-1..=3).contains(&p.1) && (7..=11).contains(&p.2);
        assert!(!inside, "bus cell {p:?} is inside c0's footprint/halo");
    }
    // c0 IS moved — the truth of the document.
    let occ = d.occupancy_index();
    assert!(
        matches!(
            occ.cells.get(&(4, 0, 8)),
            Some((_, nucleation::design::Occupant::Instance(n))) if n == "c0"
        ),
        "instance moved"
    );

    // Drag it away again. The detoured bus no longer touches c0 at all, so the
    // affected set can legitimately be empty — what matters is that no bus
    // ends up failed and the design stays clean.
    let report = d.move_instance("c0", (4, 0, 20), 0).unwrap();
    assert!(report.failed.is_empty(), "{report:?}");
    assert_eq!(d.bus_state("bus_a"), Some(&BusState::Routed));
    // An explicit reroute with the obstacle out of the way takes the direct
    // line again.
    assert_eq!(d.reroute("bus_a").unwrap(), BusState::Routed);
    let check = d.check().unwrap();
    assert!(check.clean, "{}", check.json);

    // Now SEAL the corridor: a wall no detour can get around. The bus must
    // fail visibly, never half-routed, with a reason naming the blocker.
    for z in -260..=260 {
        for y in 0..=20 {
            d.set_block((12, y, z), STONE).unwrap();
        }
    }
    match d.reroute("bus_a").unwrap() {
        BusState::Failed(reason) => {
            assert!(reason.contains("no corridor"), "{reason}");
            assert!(reason.contains("(12,"), "names the blocker location: {reason}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(d.bus("bus_a").unwrap().fragment.is_empty(), "never half-routed");
}

/// The influence halo (bounds + 1 without declared keepouts) counts as
/// interference: a component parked NEXT to the bus line — no hard
/// overlap — still rips it, and the reroute refuses halo cells.
#[test]
fn influence_halo_counts_as_interference() {
    let mut s = UniversalSchematic::new("halo".to_string());
    let a_in = lever_bank(&mut s, 0, 8, 1, 0);
    let a_out = lamp_bank(&mut s, 16, 8);
    let mut d = Design::for_schematic("halo", s);
    let ty = IoType::UnsignedInt { bits: N as usize };
    d.declare_input("a_in", a_in, (0, 2, 0), N, ty.clone()).unwrap();
    d.declare_output("a_out", a_out, (0, 2, 0), N, ty).unwrap();
    let mut body = UniversalSchematic::new("cube".to_string());
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                body.set_block_from_string(x, y, z, STONE).unwrap();
            }
        }
    }
    let contract = CellContract::new("cube".to_string(), IoLayoutBuilder::new().build());
    d.add_cell_with_contract("cube", body, contract);
    d.place("c0", "cube", (4, 0, 20), 0).unwrap();
    let state = d
        .route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed);

    // Body at z 9..11 — one cell away from the z=8 dust line, but the
    // +1 halo reaches it (dust one step up/down shorts without sharing a
    // cell). The bus is ripped and rerouted, and the REROUTE must respect the
    // halo: interference is what forces the detour, not what kills the bus.
    let report = d.move_instance("c0", (4, 0, 9), 0).unwrap();
    assert_eq!(report.rerouted, vec!["bus_a".to_string()], "{report:?}");
    assert!(report.failed.is_empty(), "{report:?}");
    assert_eq!(d.bus_state("bus_a"), Some(&BusState::Routed));
    // c0 body x4..6, y0..2, z9..11 -> halo x3..7, y-1..3, z8..12. Not one bus
    // cell may sit in it, which forces the trunk off the z=8 lane.
    let mut in_halo = 0;
    for p in d.bus("bus_a").unwrap().fragment.keys() {
        if (3..=7).contains(&p.0) && (-1..=3).contains(&p.1) && (8..=12).contains(&p.2) {
            in_halo += 1;
        }
    }
    assert_eq!(in_halo, 0, "the reroute must refuse halo cells");
    assert!(d.check().unwrap().clean, "{}", d.check().unwrap().json);
}

/// Multi-sink trunk: 1 driver -> 2 sinks realized as a shared trunk plus
/// a diode-isolated branch; LVS sees ONE net per bit over all three
/// terminals.
#[test]
fn fanout_routes_a_shared_trunk_with_a_branch() {
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
    let bus = d.bus("fan").unwrap();
    assert!(
        bus.segments
            .iter()
            .any(|s| s.kind == SegmentKind::Branch("c_out".to_string())),
        "branch segment for the second sink"
    );
    // The branch joins the trunk at plain dust and is diode-isolated by a
    // repeater right after the junction.
    assert!(
        bus.fragment.get(&(8, 2, 8)).is_some_and(|b| b.contains("redstone_wire")),
        "junction dust"
    );
    assert!(
        bus.fragment.get(&(8, 2, 9)).is_some_and(|b| is_repeater(b)),
        "diode repeater on the branch: {:?}",
        bus.fragment.get(&(8, 2, 9))
    );
    // LVS: one intent net per bit with three terminals, and it checks out.
    let nets = d.intent_nets();
    assert_eq!(nets.len(), N as usize);
    assert_eq!(nets[0].terminals.len(), 3);
    let check = d.check().unwrap();
    assert!(check.clean, "{}", check.json);
}

/// Wired-OR: multiple drivers are legal only through the explicit merge;
/// the extra driver joins as a dust-merge branch and the LVS intent stays
/// ONE net.
#[test]
fn wired_or_merges_two_drivers_into_one_net() {
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
    let bus = d.bus("wor").unwrap();
    assert!(bus.merge_or);
    assert_eq!(bus.extra_drivers, vec!["b_in".to_string()]);
    assert!(
        bus.segments
            .iter()
            .any(|s| s.kind == SegmentKind::Branch("b_in".to_string())),
        "branch segment for the wired-OR driver"
    );
    // Diode into the junction: repeater on the last branch cell.
    assert!(
        bus.fragment.get(&(8, 2, 7)).is_some_and(|b| is_repeater(b)),
        "diode repeater before the junction: {:?}",
        bus.fragment.get(&(8, 2, 7))
    );
    // ONE intent net per bit spanning both drivers and the sink.
    let nets = d.intent_nets();
    assert_eq!(nets.len(), N as usize);
    assert_eq!(nets[0].terminals.len(), 3);
    let check = d.check().unwrap();
    assert!(check.clean, "{}", check.json);
}
