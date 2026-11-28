use super::*;
use crate::io_contract::IoType;

const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";

/// The design_step1.py endpoint hardware: 8 levers stacked at 2y pitch,
/// each with its connection dust one step toward the field.
fn lever_bank(s: &mut UniversalSchematic, x: i32, z: i32, dx: i32, dz: i32) -> P3 {
    for i in 0..8 {
        let y = 2 + 2 * i;
        s.set_block_from_string(x, y - 1, z, STONE).unwrap();
        s.set_block_from_string(x, y, z, LEVER).unwrap();
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE)
            .unwrap();
        s.set_block_from_string(x + dx, y, z + dz, rblocks::DUST)
            .unwrap();
    }
    (x + dx, 2, z + dz)
}

/// 8 lamps stacked at 2y pitch, each lamp supporting its own dust.
fn lamp_bank(s: &mut UniversalSchematic, x: i32, z: i32) -> P3 {
    for i in 0..8 {
        let y = 2 + 2 * i;
        s.set_block_from_string(x, y - 1, z, LAMP).unwrap();
        s.set_block_from_string(x, y, z, rblocks::DUST).unwrap();
    }
    (x, 2, z)
}

/// 8 lamps in a horizontal ROW at 2z pitch — a port whose form is NOT the
/// canonical 2y stack, so routing to it builds a form adapter.
fn lamp_row(s: &mut UniversalSchematic, x: i32, z: i32) -> P3 {
    for i in 0..8 {
        let zz = z + 2 * i;
        s.set_block_from_string(x, 1, zz, LAMP).unwrap();
        s.set_block_from_string(x, 2, zz, rblocks::DUST).unwrap();
    }
    (x, 2, z)
}

/// The refresh stations on bit 0 of a straight bus, along its run axis.
fn station_pitches(d: &Design, bus: &str, along_x: bool) -> Vec<i32> {
    let layer = d.buses.get(bus).expect("bus");
    let mut at: Vec<i32> = layer
        .fragment
        .iter()
        .filter(|(p, b)| p.1 == 2 && b.starts_with("minecraft:repeater"))
        .map(|(p, _)| if along_x { p.0 } else { p.2 })
        .collect();
    at.sort();
    at.dedup();
    at.windows(2).map(|w| w[1] - w[0]).collect()
}

/// A LONG STRAIGHT RUN BETWEEN TWO NATIVE-FORM PORTS SPENDS THE WHOLE
/// BUDGET.
///
/// This is the measured win: nothing upstream or downstream of the anchors
/// spends any strength, so the ledger has all 14 cells to give and the
/// stations sit ~15 apart instead of ~8. Pinning the pitch is what stops a
/// future edit from quietly reintroducing the unconditional constant.
#[test]
fn a_clear_straight_run_refreshes_at_the_full_dust_budget() {
    let mut s = UniversalSchematic::new("long".to_string());
    let a = lever_bank(&mut s, 0, 8, 1, 0);
    let b = lamp_bank(&mut s, 80, 8);
    let mut d = Design::for_schematic("long", s);
    let step = (0, 2, 0);
    let ty = IoType::UnsignedInt { bits: 8 };
    d.declare_input("a", a, step, 8, ty.clone()).unwrap();
    d.declare_output("b", b, step, 8, ty).unwrap();
    d.route_bus("w", "a", &["b"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(
        d.bus_state("w"),
        Some(&BusState::Routed),
        "the bus must route"
    );
    let pitches = station_pitches(&d, "w", true);
    assert!(!pitches.is_empty(), "a 79-cell run must refresh at all");
    for p in &pitches {
        assert!(
            *p >= 8,
            "the ledger placed a station every {p} cells; the whole point is that a clear \
             run no longer pays the old ~8-cell pitch (pitches {pitches:?})"
        );
        assert!(
            *p as u32 <= DUST_BUDGET - CROSSING_ALLOWANCE + 1,
            "a {p}-cell gap means {} dust cells, past the budget a run may spend \
             (pitches {pitches:?})",
            p - 1
        );
    }
    // The interior gaps should be AT the budget: latest-legal placement.
    assert!(
        pitches
            .iter()
            .any(|p| *p as u32 == DUST_BUDGET - CROSSING_ALLOWANCE + 1),
        "no gap reached the full budget; the placement is not latest-legal ({pitches:?})"
    );
}

/// Sparse refresh placement must not turn a diode back into dust when that
/// dust would join a foreign diagonal. Before the clearance-aware choice,
/// this exact shape routed and was then rejected by the completed-route
/// check: (8,2,-1) connected downhill to the loose wire at (8,1,0).
#[test]
fn a_run_keeps_a_diode_beside_a_foreign_dust_diagonal() {
    let mut s = UniversalSchematic::new("clearance-diode".to_string());
    s.set_block_from_string(0, 1, -1, STONE).unwrap();
    s.set_block_from_string(0, 2, -1, LEVER).unwrap();
    s.set_block_from_string(1, 1, -1, STONE).unwrap();
    s.set_block_from_string(1, 2, -1, rblocks::DUST).unwrap();
    s.set_block_from_string(14, 1, -1, LAMP).unwrap();
    s.set_block_from_string(14, 2, -1, rblocks::DUST).unwrap();
    // Foreign live wire, one down and one across from the planned run.
    s.set_block_from_string(8, 0, 0, STONE).unwrap();
    s.set_block_from_string(8, 1, 0, rblocks::DUST).unwrap();

    let mut d = Design::for_schematic("clearance-diode", s);
    let ty = IoType::Boolean;
    d.declare_input("a", (1, 2, -1), (0, 2, 0), 1, ty.clone())
        .unwrap();
    d.declare_output("b", (14, 2, -1), (0, 2, 0), 1, ty)
        .unwrap();
    let state = d
        .route_bus("w", "a", &["b"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed, "{state:?}");
    assert!(
        d.bus("w")
            .unwrap()
            .fragment
            .get(&(8, 2, -1))
            .is_some_and(|b| rblocks::is_repeater(b)),
        "the unsafe dust cell must remain a directional diode"
    );
}

/// A FORM CONVERSION KEEPS ITS APPROACH TIGHT.
///
/// The adversarial sweep's nine failures are all runs into or out of a form
/// adapter, whose gather column spends strength the run never saw. The
/// ledger charges that debt, so the stretch touching the adapter must stay
/// SHORTER than the budget even though the interior of the same bus widens.
#[test]
fn a_run_into_a_form_adapter_leaves_it_headroom() {
    let mut s = UniversalSchematic::new("conv".to_string());
    let a = lever_bank(&mut s, 0, 8, 1, 0);
    let b = lamp_row(&mut s, 80, 8);
    let mut d = Design::for_schematic("conv", s);
    let ty = IoType::UnsignedInt { bits: 8 };
    d.declare_input("a", a, (0, 2, 0), 8, ty.clone()).unwrap();
    d.declare_output("b", b, (0, 0, 2), 8, ty).unwrap();
    d.route_bus("w", "a", &["b"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(
        d.bus_state("w"),
        Some(&BusState::Routed),
        "the bus must route"
    );
    let layer = d.buses.get("w").expect("bus");
    // The adapter owes something: it is a row->stack gather with a diode at
    // the mouth and one per gather leg, so its column is not a fresh 15.
    let debt = layer
        .fragment
        .keys()
        .filter(|p| p.1 == 2)
        .map(|&p| {
            tile_dust_debt(p, |q| {
                layer.fragment.get(&q).is_some_and(|b| rblocks::is_dust(b))
            })
        })
        .max()
        .unwrap_or(0);
    assert!(
        debt <= DUST_BUDGET,
        "some dust chain on bit 0 runs {debt} cells with nothing refreshing it, past the \
         {DUST_BUDGET}-cell budget — the ledger is not charging something it should"
    );
}

fn crossing_design() -> Design {
    let mut s = UniversalSchematic::new("crossing".to_string());
    let a_in = lever_bank(&mut s, 0, 8, 1, 0);
    let a_out = lamp_bank(&mut s, 16, 8);
    let b_in = lever_bank(&mut s, 8, 0, 0, 1);
    let b_out = lamp_bank(&mut s, 8, 16);
    let mut d = Design::for_schematic("crossing", s);
    let step = (0, 2, 0);
    let ty = IoType::UnsignedInt { bits: 8 };
    d.declare_input("a_in", a_in, step, 8, ty.clone()).unwrap();
    d.declare_output("a_out", a_out, step, 8, ty.clone())
        .unwrap();
    d.declare_input("b_in", b_in, step, 8, ty.clone()).unwrap();
    d.declare_output("b_out", b_out, step, 8, ty).unwrap();
    d
}

#[test]
fn capability_scan_validates_loudly() {
    let mut s = UniversalSchematic::new("caps".to_string());
    lever_bank(&mut s, 0, 8, 1, 0);
    let mut d = Design::for_schematic("caps", s);
    // Anchor on the lever's support (not dust): refused, names the block.
    let err = d
        .declare_input(
            "bad",
            (0, 1, 8),
            (0, 2, 0),
            8,
            IoType::UnsignedInt { bits: 8 },
        )
        .unwrap_err();
    assert!(err.contains("not dust"), "{err}");
    // Output on lever hardware: not readable.
    let err = d
        .declare_output(
            "bad",
            (1, 2, 8),
            (0, 2, 0),
            8,
            IoType::UnsignedInt { bits: 8 },
        )
        .unwrap_err();
    assert!(err.contains("no adjacent lamp"), "{err}");
    // Input over the same cells: drivable, fine.
    d.declare_input(
        "good",
        (1, 2, 8),
        (0, 2, 0),
        8,
        IoType::UnsignedInt { bits: 8 },
    )
    .unwrap();
}

/// A library cell shaped like the community ones: its contract names
/// EXECUTOR hardware (levers in, lamps out), not dust. Bit 0 of `dead`
/// is a bare lamp with no dust anywhere near it.
fn buffer_cell() -> UniversalSchematic {
    let mut s = UniversalSchematic::new("buf".to_string());
    let d_hw: Vec<P3> = (0..8).map(|i| (0, 2 + 2 * i, 0)).collect();
    let q_hw: Vec<P3> = (0..8).map(|i| (4, 1 + 2 * i, 0)).collect();
    for i in 0..8 {
        let y = 2 + 2 * i;
        // input lever + its connection dust one step in
        s.set_block_from_string(0, y - 1, 0, STONE).unwrap();
        s.set_block_from_string(0, y, 0, LEVER).unwrap();
        s.set_block_from_string(1, y - 1, 0, STONE).unwrap();
        s.set_block_from_string(1, y, 0, rblocks::DUST).unwrap();
        // output lamp supporting its connection dust
        s.set_block_from_string(4, y - 1, 0, LAMP).unwrap();
        s.set_block_from_string(4, y, 0, rblocks::DUST).unwrap();
    }
    // A lamp with no dust neighbour at all: executor-readable, never
    // bus-routable.
    s.set_block_from_string(4, 40, 0, LAMP).unwrap();
    let layout = crate::io_contract::IoLayoutBuilder::new()
        .add_input(
            "d".to_string(),
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::OneToOne,
            d_hw,
        )
        .unwrap()
        .add_output(
            "q".to_string(),
            IoType::UnsignedInt { bits: 8 },
            LayoutFunction::OneToOne,
            q_hw,
        )
        .unwrap()
        .add_output(
            "dead".to_string(),
            IoType::Boolean,
            LayoutFunction::OneToOne,
            vec![(4, 40, 0)],
        )
        .unwrap()
        .build();
    s.set_cell_contract(&CellContract::new("buf".to_string(), layout))
        .unwrap();
    s
}

/// The blocker this module exists to remove: a PLACED INSTANCE exposes
/// its contract ports as routable endpoints under `{inst}.{port}`,
/// transformed, with the dust connection cell derived by hardware scan —
/// so `route_bus` takes `u0.q` directly.
#[test]
fn instance_ports_are_first_class_routing_endpoints() {
    let mut s = UniversalSchematic::new("host".to_string());
    let out = lamp_bank(&mut s, 24, 8);
    let mut d = Design::for_schematic("host", s);
    d.add_cell("buf", buffer_cell()).unwrap();
    // The cell's own bbox starts at y=1, so `at` y=1 lands bit 0's
    // connection dust on the design's canonical y=2 bus level.
    d.place("u0", "buf", (0, 1, 8), 0).unwrap();
    d.declare_output("q_out", out, (0, 2, 0), 8, IoType::UnsignedInt { bits: 8 })
        .unwrap();

    let ports = d.instance_ports().unwrap();
    let names: Vec<&str> = ports.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["u0.d", "u0.dead", "u0.q"], "{names:?}");

    // The contract names the lamp column; the routable endpoint is the
    // dust ON those lamps, one cell up, transformed by the placement.
    let q = ports.iter().find(|p| p.name == "u0.q").unwrap();
    assert!(q.routable(), "{:?}", q.blocked);
    assert_eq!(q.hardware[0], (4, 1, 8));
    assert_eq!(q.wires.as_ref().unwrap()[0], (4, 2, 8));
    assert_eq!(q.step, Some((0, 2, 0)));

    // A lamp with no dust anywhere near it is reported, not mis-routed.
    let dead = ports.iter().find(|p| p.name == "u0.dead").unwrap();
    assert!(!dead.routable());
    assert!(
        dead.blocked
            .as_ref()
            .unwrap()
            .contains("no dust connection cell"),
        "{:?}",
        dead.blocked
    );

    // resolve_port flips to the DESIGN-facing direction: the cell's
    // output drives the fabric.
    let rq = d.resolve_port("u0.q").unwrap();
    assert_eq!(rq.direction, PortDirection::Input);
    assert_eq!(
        d.resolve_port("u0.d").unwrap().direction,
        PortDirection::Output
    );
    let err = d.resolve_port("u0.nope").unwrap_err();
    assert!(err.contains("no contract port `nope`"), "{err}");

    // ...and a bus takes the instance port by name. The endpoint sits on
    // the instance's own body, so pin access must beat its halo.
    let state = d
        .route_bus("chain", "u0.q", &["q_out"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed, "{state:?}");
    assert!(d.check().unwrap().clean, "{}", d.check().unwrap().json);

    // Driving into a cell input is legal; driving OUT of one is not.
    let err = d
        .route_bus("bad", "u0.d", &["q_out"], vec![], BusStyle::default())
        .unwrap_err();
    assert!(err.contains("cannot drive"), "{err}");

    // Removing the instance takes its bus with it, loudly.
    let report = d.remove_instance("u0").unwrap();
    assert_eq!(report.removed_buses, vec!["chain".to_string()]);
    assert!(d.instance_ports().unwrap().is_empty());
}

#[test]
fn two_crossing_buses_route_with_an_implicit_dip_under() {
    let mut d = crossing_design();
    let a = d
        .route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(a, BusState::Routed);
    let mut style_b = BusStyle::default();
    style_b.bus_block = "minecraft:cyan_concrete".to_string();
    let b = d
        .route_bus("bus_b", "b_in", &["b_out"], vec![], style_b)
        .unwrap();
    assert_eq!(b, BusState::Routed, "{:?}", d.bus_state("bus_b"));

    // The crossing is implicit: bus_b dips (dust at the odd level) and
    // bus_a got its station upgrade (a repeater in its fragment).
    let bus_b = d.bus("bus_b").unwrap();
    assert!(bus_b.fragment.contains_key(&(8, 1, 7)), "dip dust");
    let bus_a = d.bus("bus_a").unwrap();
    assert!(
        bus_a.fragment.values().any(|b| rblocks::is_repeater(b)),
        "station repeater in the through bus"
    );

    // Flatten: one region per layer + embedded merged contract.
    let flat = d.flatten().unwrap();
    let regions = flat.get_region_names();
    assert!(regions.iter().any(|r| r == "bus:bus_a"), "{regions:?}");
    assert!(regions.iter().any(|r| r == "bus:bus_b"), "{regions:?}");
    let contract = flat.embedded_cell_contract().unwrap().unwrap();
    assert!(contract.io.get_input("a_in").is_some());
    assert!(contract.io.get_output("b_out").is_some());

    // check(): DRC + LVS come back clean.
    let check = d.check().unwrap();
    assert!(check.clean, "{}", check.json);
}

#[test]
fn unroutable_is_a_state_not_an_exception() {
    let mut d = crossing_design();
    // A SEALED stone wall across the a-line. A single blocking column is
    // no longer enough — the corridor search routes around one of those —
    // so unroutability now needs a wall the bus genuinely cannot get past.
    // The reason must name the blocker and what to do about it.
    for z in -260..=260 {
        for y in 0..=20 {
            d.set_block((12, y, z), STONE).unwrap();
        }
    }
    let state = d
        .route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
        .unwrap();
    match state {
        BusState::Failed(reason) => {
            assert!(reason.contains("no corridor"), "{reason}");
            assert!(
                reason.contains("(12,"),
                "names the blocker location: {reason}"
            );
            assert!(reason.contains("loose block"), "names the owner: {reason}");
            // And it must say whether the LEVEL is the problem or the
            // workspace is genuinely full — the two need opposite fixes, and
            // "no path" told the user neither. Here the wall spans y 0..=20,
            // so no level is clear.
            assert!(
                reason.contains("No level within 8 blocks"),
                "must report the cross-level verdict: {reason}"
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(d.bus("bus_a").unwrap().fragment.is_empty());
    // The failed layer does not poison later routing.
    let b = d
        .route_bus("bus_b", "b_in", &["b_out"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(b, BusState::Routed, "{:?}", d.bus_state("bus_b"));
}

#[test]
fn a_dogleg_routes_through_an_implicit_corner() {
    let mut d = crossing_design();
    // a_in (1,2,8) -> b_out (8,2,16): not axis-aligned; the planner
    // bends through one implicit corner joint instead of failing.
    let state = d
        .route_bus("diag", "a_in", &["b_out"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed, "{:?}", d.bus_state("diag"));
    let bus = d.bus("diag").unwrap();
    // One trunk segment realized as two perpendicular runs + the
    // corner joint dust at (8, 2, 8).
    assert_eq!(bus.segments.len(), 1);
    assert_eq!(bus.segments[0].runs.len(), 2);
    assert!(bus.fragment.contains_key(&(8, 2, 8)), "corner joint dust");
    let check = d.check().unwrap();
    assert!(check.clean, "{}", check.json);
}

#[test]
fn embedded_contract_round_trips_through_schem_bytes() {
    let mut d = crossing_design();
    d.route_bus("bus_a", "a_in", &["a_out"], vec![], BusStyle::default())
        .unwrap();
    let bytes = d.to_schem_bytes().unwrap();
    let back = crate::formats::schematic::from_schematic(&bytes).unwrap();
    let (contract, warnings) = back.resolve_cell_contract().unwrap().unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(contract.name, "crossing");
    // Executor-facing positions: inputs are the LEVER cells.
    let a_in = contract.io.get_input("a_in").unwrap();
    assert_eq!(a_in.positions[0], (0, 2, 8));
    let a_out = contract.io.get_output("a_out").unwrap();
    assert_eq!(a_out.positions[0], (16, 2, 8));
}
