//! BUS LEVEL SHIFT: endpoints on different levels route automatically.
//!
//! The user report this file exists for: "us bus2 failed: the two ends are on
//! different levels — driver bit 0 at y=5, sink bit 0 at y=2 between
//! (55,5,22) and (67,2,23)". A bus whose endpoints differ in Y must change
//! level BY CONSTRUCTION; being told to move an instance is not an answer.
//!
//! The geometry is the tile verified in mc-tick by
//! `redstone-eda/bus_levelshift.py` (8-bit stack, K in {1,2,3,5,8} x
//! {down,up} x {fresh,stale} arrival, 3040 output checks, zero crosstalk;
//! all-solid / all-glass / swapped-parity variants all FAIL, so the
//! alternation is load-bearing). Here we assert the ROUTER reaches for it,
//! that the result is DRC + LVS clean, and — where the ports are drivable and
//! readable — that the words actually arrive in the vanilla-accurate engine.

#![cfg(feature = "routing")]

use nucleation::design::{BusState, BusStyle, Design};
use nucleation::io_contract::IoType;
use nucleation::UniversalSchematic;

type P3 = (i32, i32, i32);

const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";
const GLASS: &str = "minecraft:glass";

/// `width` levers stacked at 2y pitch from `y0`, connection dust one step
/// toward the field. Returns the bit-0 connection cell.
fn lever_bank(s: &mut UniversalSchematic, x: i32, y0: i32, z: i32, dx: i32, dz: i32, width: u8) -> P3 {
    for i in 0..width as i32 {
        let y = y0 + 2 * i;
        s.set_block_from_string(x, y - 1, z, STONE).unwrap();
        s.set_block_from_string(x, y, z, LEVER).unwrap();
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE).unwrap();
        s.set_block_from_string(x + dx, y, z + dz, DUST).unwrap();
    }
    (x + dx, y0, z + dz)
}

/// `width` lamps stacked at 2y pitch from `y0`, each supporting its own dust.
fn lamp_bank(s: &mut UniversalSchematic, x: i32, y0: i32, z: i32, width: u8) -> P3 {
    for i in 0..width as i32 {
        let y = y0 + 2 * i;
        s.set_block_from_string(x, y - 1, z, LAMP).unwrap();
        s.set_block_from_string(x, y, z, DUST).unwrap();
    }
    (x, y0, z)
}

/// A one-bus design: driver bank at `a`, sink bank at `b`, `width` bits.
fn leveled_design(a: P3, b: P3, width: u8) -> Design {
    let mut s = UniversalSchematic::new("lvl".to_string());
    // The lever bank stands one cell behind its connection dust, on the axis
    // the bus will leave along.
    let (dx, dz) = if (b.0 - a.0).abs() >= (b.2 - a.2).abs() {
        ((b.0 - a.0).signum(), 0)
    } else {
        (0, (b.2 - a.2).signum())
    };
    let drv = lever_bank(&mut s, a.0 - dx, a.1, a.2 - dz, dx, dz, width);
    let snk = lamp_bank(&mut s, b.0, b.1, b.2, width);
    assert_eq!(drv, a, "driver anchor");
    assert_eq!(snk, b, "sink anchor");
    let mut d = Design::for_schematic("lvl", s);
    let ty = IoType::UnsignedInt { bits: width as usize };
    d.declare_input("din", drv, (0, 2, 0), width, ty.clone()).unwrap();
    d.declare_output("dout", snk, (0, 2, 0), width, ty).unwrap();
    d
}

/// Gating DRC violation count (the `drc` array; `cells` is informational).
fn violations(json: &str) -> usize {
    let Some(start) = json.find("\"drc\":[") else {
        return 0;
    };
    let rest = &json[start + 7..];
    let end = rest.find("],\"cells\"").unwrap_or(rest.len());
    rest[..end].matches("\"kind\"").count()
}

fn assert_clean(d: &Design, what: &str) {
    let c = d.check().unwrap_or_else(|e| panic!("{what}: check() failed: {e}"));
    assert!(
        c.clean,
        "{what}: not DRC/LVS clean ({} gating violation(s)): {}",
        violations(&c.json),
        c.json
    );
}

fn route(d: &mut Design, name: &str) -> BusState {
    d.route_bus(name, "din", &["dout"], vec![], BusStyle::default())
        .unwrap_or_else(|e| panic!("declaration refused: {e}"))
}

// ----------------------------------------------------------------------
// The user's case
// ----------------------------------------------------------------------

/// THE REPORTED BUG, exactly: 8 bits, driver bit 0 at y=5, sink bit 0 at
/// y=2, (55,5,22) -> (67,2,23). It must route, with no user action.
#[test]
fn the_reported_case_routes_automatically() {
    let mut d = leveled_design((55, 5, 22), (67, 2, 23), 8);
    let state = route(&mut d, "bus2");
    assert_eq!(
        state,
        BusState::Routed,
        "the reported geometry still fails: {:?}",
        d.bus_state("bus2")
    );
    assert_clean(&d, "reported case");

    // It really did change level: the fragment carries dust at BOTH bit-0
    // levels, and the alternation's transparent supports are present.
    let frag = &d.bus("bus2").expect("routed bus").fragment;
    let has_dust_at = |y: i32| {
        frag.iter()
            .any(|(p, b)| p.1 == y && b.contains("redstone_wire"))
    };
    assert!(has_dust_at(5), "no dust at the driver's level");
    assert!(has_dust_at(2), "no dust at the sink's level");
    let glass = frag.values().filter(|b| b.as_str() == GLASS).count();
    assert!(
        glass > 0,
        "a level shift with no transparent support cannot have alternated"
    );
    // Bit 0 is unconstrained (nothing below it to protect), so its supports
    // are solid: 3 levels x 7 upper bits.
    assert_eq!(glass, 3 * 7, "one transparent support per level per bit > 0");
}

/// The failure message must never tell the user to move an instance for
/// something the router can do. When a shift genuinely does not fit, the
/// reason names the tile's cost and the span actually available.
#[test]
fn an_impossible_shift_names_what_it_tried_instead_of_blaming_the_user() {
    // 8 levels apart with only 4 cells of span: no placement can fit.
    let mut d = leveled_design((10, 10, 8), (14, 2, 8), 8);
    let state = route(&mut d, "tight");
    let BusState::Failed(reason) = state else {
        panic!("a 4-cell span cannot host a 20-cell shift, but it routed");
    };
    assert!(
        !reason.to_lowercase().contains("move one endpoint")
            && !reason.to_lowercase().contains("move ")
            || reason.contains("Lengthen"),
        "the reason still blames the user: {reason}"
    );
    assert!(
        reason.contains("level-shift tile needs") && reason.contains("cells of straight run"),
        "the reason does not name the tile's cost: {reason}"
    );
    assert!(
        reason.contains("only spans"),
        "the reason does not name the span available: {reason}"
    );
}

// ----------------------------------------------------------------------
// K sweep, both directions
// ----------------------------------------------------------------------

/// Every K the Python gate verified, both directions, 8 bits: routed and
/// clean. Spans are generous so the placement ladder is not the thing under
/// test — the tile is.
#[test]
fn level_shifts_route_for_every_verified_k_both_directions() {
    for k in [1i32, 2, 3, 5, 8] {
        for down in [true, false] {
            let (ya, yb) = if down { (2 + k, 2) } else { (2, 2 + k) };
            let mut d = leveled_design((1, ya, 8), (48, yb, 8), 8);
            let name = format!("k{k}{}", if down { "d" } else { "u" });
            let state = route(&mut d, &name);
            assert_eq!(
                state,
                BusState::Routed,
                "K={k} {} failed: {:?}",
                if down { "down" } else { "up" },
                d.bus_state(&name)
            );
            assert_clean(&d, &format!("K={k} {}", if down { "down" } else { "up" }));
            let frag = &d.bus(&name).unwrap().fragment;
            let glass = frag.values().filter(|b| b.as_str() == GLASS).count();
            assert_eq!(
                glass, (k as usize) * 7,
                "K={k}: one transparent support per level per bit > 0"
            );
        }
    }
}

/// A level shift on the Z axis, and one where the pair also doglegs: the
/// placement ladder must pick an axis with room, not just X.
#[test]
fn level_shifts_work_on_either_axis_and_through_a_dogleg() {
    for (a, b, what) in [
        ((8, 6, 1), (8, 2, 40), "z axis"),
        ((1, 2, 8), (40, 8, 36), "x axis with a dogleg"),
        ((1, 8, 8), (40, 2, -24), "x axis, descending, negative dogleg"),
    ] {
        let mut d = leveled_design(a, b, 8);
        let state = route(&mut d, "b");
        assert_eq!(state, BusState::Routed, "{what}: {:?}", d.bus_state("b"));
        assert_clean(&d, what);
    }
}

/// A gate declared at the target level splits the run: the leg before it
/// shifts, the leg after is flat. Declared gates stay honoured.
#[test]
fn a_gate_at_the_target_level_still_splits_the_run() {
    let mut d = leveled_design((1, 6, 8), (48, 2, 8), 8);
    let gates = vec![nucleation::design::Gate {
        name: "g".to_string(),
        anchor: (24, 2, 8),
        step: (0, 2, 0),
    }];
    let state = d
        .route_bus("g_bus", "din", &["dout"], gates, BusStyle::default())
        .unwrap();
    assert_eq!(state, BusState::Routed, "{:?}", d.bus_state("g_bus"));
    assert_clean(&d, "gated level shift");
}

// ----------------------------------------------------------------------
// Typed, in-sim
// ----------------------------------------------------------------------

/// The words really arrive: the reported geometry, driven and read purely
/// through the embedded contract in the vanilla-accurate engine.
#[cfg(all(feature = "simulation", feature = "bridge", feature = "mc-tick"))]
#[test]
fn the_reported_case_delivers_its_word_in_sim() {
    use nucleation::io_contract::Value;
    use nucleation::simulation::typed_executor::BackendCircuitExecutor;

    let mut d = leveled_design((55, 5, 22), (67, 2, 23), 8);
    assert_eq!(route(&mut d, "bus2"), BusState::Routed);

    let baked = d.bake(4000).unwrap();
    let contract = baked.embedded_cell_contract().unwrap().unwrap();
    let extra = nucleation::design::executor_extra_states();
    let refs: Vec<&str> = extra.iter().map(String::as_str).collect();
    let mut cell = BackendCircuitExecutor::for_cell(baked, &contract, &refs).unwrap();
    cell.settle(4000);
    let read = |c: &mut BackendCircuitExecutor| match c.read_output("dout").unwrap() {
        Value::U32(v) => v,
        other => panic!("unexpected {other:?}"),
    };
    // Walking ones prove per-bit isolation across the shift; all-on and the
    // alternating pairs prove the interleave holds under load.
    for i in 0..8u32 {
        cell.set_input("din", &Value::U32(1 << i)).unwrap();
        cell.settle(800);
        assert_eq!(read(&mut cell), 1 << i, "walking one, bit {i}");
    }
    for w in [0xFFu32, 0xAA, 0x55, 0x00] {
        cell.set_input("din", &Value::U32(w)).unwrap();
        cell.settle(800);
        assert_eq!(read(&mut cell), w, "word {w:#x}");
    }
}

/// Ascending and descending shifts both conduct in-sim (4 bits, to keep the
/// engine's work down — the 8-bit case above covers width).
#[cfg(all(feature = "simulation", feature = "bridge", feature = "mc-tick"))]
#[test]
fn shifts_conduct_in_sim_in_both_directions() {
    use nucleation::io_contract::Value;
    use nucleation::simulation::typed_executor::BackendCircuitExecutor;

    for (ya, yb, what) in [(6, 2, "down 4"), (2, 7, "up 5")] {
        let mut d = leveled_design((1, ya, 8), (32, yb, 8), 4);
        assert_eq!(route(&mut d, "b"), BusState::Routed, "{what}: {:?}", d.bus_state("b"));
        let baked = d.bake(4000).unwrap();
        let contract = baked.embedded_cell_contract().unwrap().unwrap();
        let extra = nucleation::design::executor_extra_states();
        let refs: Vec<&str> = extra.iter().map(String::as_str).collect();
        let mut cell = BackendCircuitExecutor::for_cell(baked, &contract, &refs).unwrap();
        cell.settle(4000);
        for w in [0x1u32, 0x2, 0x4, 0x8, 0xF, 0xA, 0x5, 0x0] {
            cell.set_input("din", &Value::U32(w)).unwrap();
            cell.settle(800);
            let got = match cell.read_output("dout").unwrap() {
                Value::U32(v) => v,
                other => panic!("unexpected {other:?}"),
            };
            assert_eq!(got, w, "{what}: word {w:#x}");
        }
    }
}
