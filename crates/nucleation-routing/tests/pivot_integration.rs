//! Bus planner + pivot stamping, statically verified: a vertical-port bus
//! routed to a horizontal-port rail stamps the v2h tile implicitly, routes
//! every bit, and leaves a short-free, floating-free, DRC-clean workspace.
//! Conduction is the sim-backed test's job (`pivot_sim.rs`, feature
//! `mc-tick`).

mod common;

use common::{build_scene, PIVOT_AT};
use nucleation_routing::drc::{drc, DrcOptions};
use nucleation_routing::pivot::{route_bus, PivotKind};
use nucleation_routing::{audit, nets, Aabb, Pos, RedstoneRouter};

fn router() -> RedstoneRouter {
    let mut r = RedstoneRouter::new();
    r.bounds = Some(Aabb::new(Pos::new(0, 0, -3), Pos::new(46, 17, 17)));
    r
}

#[test]
fn vertical_to_horizontal_stamps_the_pivot_implicitly() {
    let mut scene = build_scene();
    let report = route_bus(
        &mut scene.ws,
        &router(),
        &scene.from,
        &scene.to,
        PIVOT_AT,
        "bus",
        None,
    )
    .expect("bus routes");
    assert_eq!(report.pivot, Some(PivotKind::V2H));
    assert_eq!(report.routes.len(), 16, "8 bits x (port->pivot, pivot->rail)");
    // The stamped v-ports sit against the connection dusts; the h-ports at
    // the tile's east face.
    for n in 0..8 {
        assert_eq!(report.pivot_in[n as usize], Pos::new(6, 1 + 2 * n, 0));
        assert_eq!(report.pivot_out[n as usize], Pos::new(25, 1, 2 * n));
    }

    // No two labels share an electrical component.
    let shorts = nets::check(scene.ws.cells(), scene.ws.labels(), &[]);
    assert!(shorts.is_empty(), "shorts: {shorts:?}");

    // Nothing floats: the tile carries its own supports, the router lays
    // the rest.
    let report_a = audit::audit(scene.ws.cells());
    assert!(report_a.is_clean(), "audit: {report_a:?}");

    // DRC (decay is verified in-sim; see router_integration.rs for why the
    // static check skips it).
    let violations = drc(
        &scene.ws,
        &DrcOptions {
            aliases: vec![],
            skip_decay: true,
        },
    );
    assert!(violations.is_empty(), "DRC: {violations:?}");
}

#[test]
fn bus_route_is_deterministic() {
    let mut a = build_scene();
    let mut b = build_scene();
    let ra = route_bus(&mut a.ws, &router(), &a.from, &a.to, PIVOT_AT, "bus", None).unwrap();
    let rb = route_bus(&mut b.ws, &router(), &b.from, &b.to, PIVOT_AT, "bus", None).unwrap();
    assert_eq!(a.ws.cells(), b.ws.cells());
    for (x, y) in ra.routes.iter().zip(&rb.routes) {
        assert_eq!(x.path, y.path);
    }
}

#[test]
fn same_form_endpoints_route_without_a_pivot() {
    // Two flat rails on the same axis: plain per-bit routes, no tile.
    let mut ws = nucleation_routing::Workspace::new();
    for n in 0..4 {
        ws.dust(Pos::new(0, 1, 3 * n), &format!("w{n}")).unwrap();
        ws.dust(Pos::new(10, 1, 3 * n), &format!("w{n}")).unwrap();
    }
    let spec = |face| nucleation_routing::BusSpec {
        width: 4,
        pitch: nucleation_routing::bus::Pitch {
            axis: nucleation_routing::Axis::Z,
            spacing: 3,
        },
        face,
        encoding: nucleation_routing::Encoding::Binary1PerWire,
    };
    let from = nucleation_routing::BusPort {
        spec: spec(nucleation_routing::Face::East),
        bit0: Pos::new(0, 1, 0),
        dir: nucleation_routing::InOut::Out,
    };
    let to = nucleation_routing::BusPort {
        spec: spec(nucleation_routing::Face::West),
        bit0: Pos::new(10, 1, 0),
        dir: nucleation_routing::InOut::In,
    };
    let mut r = RedstoneRouter::new();
    r.bounds = Some(Aabb::new(Pos::new(-2, 0, -2), Pos::new(14, 4, 12)));
    let report = route_bus(&mut ws, &r, &from, &to, Pos::new(0, 0, 0), "w", None).unwrap();
    assert_eq!(report.pivot, None);
    assert_eq!(report.routes.len(), 4);
    let shorts = nets::check(ws.cells(), ws.labels(), &[]);
    assert!(shorts.is_empty(), "shorts: {shorts:?}");
}
