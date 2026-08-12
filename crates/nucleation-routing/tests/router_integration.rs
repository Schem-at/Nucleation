//! Static port of `test_router.py`: two nets, plane 0 -> plane 1 (+12 y),
//! past an obstacle slab. The Python original verified conduction in
//! mc-tick; here we verify everything the static toolchain can see — paths
//! found, geometry emitted, no shorts, nothing floating, DRC clean — and
//! leave conduction to the sim-backed harness (deferred to the verify()
//! phase per the design doc).

use nucleation_routing::blocks::LEVER_OFF;
use nucleation_routing::drc::{drc, DrcOptions};
use nucleation_routing::router::NetRoute;
use nucleation_routing::{audit, nets, Aabb, Pos, RedstoneRouter, Workspace};

fn build_scene() -> (Workspace, Vec<(Pos, Pos)>) {
    let mut ws = Workspace::new();
    let mut endpoints = Vec::new();
    // Plane 0: two levers with short dust tails at z = 0 and 4.
    for (i, z) in [0, 4].into_iter().enumerate() {
        ws.stone(Pos::new(0, 0, z), "plain").unwrap();
        ws.force(Pos::new(0, 1, z), LEVER_OFF);
        ws.dust(Pos::new(1, 1, z), &format!("net{i}")).unwrap();
    }
    // Obstacle slab between the planes with a gap at x 4..=9.
    for x in -2..=15 {
        for z in -2..=7 {
            if !(4..=9).contains(&x) {
                ws.stone(Pos::new(x, 7, z), "plain").unwrap();
            }
        }
    }
    // Plane 1: two target rails at y = 13, labelled like compiled rails.
    for (i, z) in [1, 5].into_iter().enumerate() {
        for x in 8..=13 {
            ws.stone(Pos::new(x, 12, z), "plain").unwrap();
            ws.dust(Pos::new(x, 13, z), &format!("net{i}")).unwrap();
        }
        endpoints.push((
            Pos::new(1, 1, if i == 0 { 0 } else { 4 }),
            Pos::new(8, 13, z),
        ));
    }
    (ws, endpoints)
}

#[test]
fn routes_two_nets_across_planes_without_shorts() {
    let (mut ws, endpoints) = build_scene();
    let mut router = RedstoneRouter::new();
    router.bounds = Some(Aabb::new(Pos::new(-3, 0, -3), Pos::new(16, 14, 8)));

    let nets_req: Vec<NetRoute> = endpoints
        .iter()
        .enumerate()
        .map(|(i, (src, dst))| NetRoute {
            src: *src,
            dsts: vec![*dst],
            label: format!("net{i}"),
            friendly: vec![],
        })
        .collect();
    let results = router
        .route_all(&mut ws, &nets_req)
        .expect("both nets route");
    for (i, r) in results.iter().enumerate() {
        // Climbs compress +5 y into a single path step, so the +12y route
        // legitimately needs only a handful of steps.
        assert!(r.cells >= 4, "net{i} suspiciously short: {} cells", r.cells);
        assert_eq!(*r.path.first().unwrap(), endpoints[i].0);
        assert_eq!(*r.path.last().unwrap(), endpoints[i].1);
    }

    // Net check: no two labels share an electrical component.
    let shorts = nets::check(ws.cells(), ws.labels(), &[]);
    assert!(shorts.is_empty(), "shorts: {shorts:?}");

    // Structural audit: nothing floats (the router lays every support).
    let report = audit::audit(ws.cells());
    assert!(report.is_clean(), "audit: {report:?}");

    // Full DRC (skip decay: a routed path refreshes on straight runs, but
    // turn-heavy segments are only decay-exact in-sim; the static decay
    // check is exercised in its own unit tests).
    let violations = drc(
        &ws,
        &DrcOptions {
            aliases: vec![],
            skip_decay: true,
        },
    );
    assert!(violations.is_empty(), "DRC: {violations:?}");
}

#[test]
fn single_net_route_is_deterministic() {
    let (mut a, endpoints) = build_scene();
    let (mut b, _) = build_scene();
    let mut router = RedstoneRouter::new();
    router.bounds = Some(Aabb::new(Pos::new(-3, 0, -3), Pos::new(16, 14, 8)));
    let (src, dst) = endpoints[0];
    let ra = router.route(&mut a, src, dst, "net0", &[]).unwrap();
    let rb = router.route(&mut b, src, dst, "net0", &[]).unwrap();
    assert_eq!(ra.path, rb.path, "same inputs must route identically");
    assert_eq!(a.cells(), b.cells(), "emitted geometry must be identical");
}
