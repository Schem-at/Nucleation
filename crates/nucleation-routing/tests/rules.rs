//! The design-rule table, one regression test per row. Every rule was
//! mandated by a real in-sim bug; the provenance is quoted from
//! `ROUTING_CRATE_DESIGN.md` at each test.

use nucleation_routing::blocks::{self, DUST};
use nucleation_routing::fabric::{NetSpec, RMove, RedstoneFabric, StairMem};
use nucleation_routing::nets;
use nucleation_routing::via::ViaRegistry;
use nucleation_routing::{Aabb, Pos, RedstoneRouter, SignalBudget, Workspace};
use pnr_core::fabric::{Candidate, Fabric, RouteCtx, State};

fn fabric<'a>(
    ws: &'a Workspace,
    specs: &'a [NetSpec],
    vias: &'a ViaRegistry,
) -> RedstoneFabric<'a> {
    RedstoneFabric {
        ws,
        nets: specs,
        bounds: None,
        region: None,
        budget: SignalBudget::default(),
        vias,
    }
}

fn specs(label: &str) -> Vec<NetSpec> {
    vec![NetSpec::new(label)]
}

fn ctx() -> RouteCtx {
    RouteCtx { net: 0 }
}

/// RULE: electrical clearance (dust adjacency incl. cut diagonals).
/// Provenance: "every braid short".
#[test]
fn rule_electrical_clearance() {
    let mut ws = Workspace::new();
    ws.dust(Pos::new(0, 1, 1), "b").unwrap();
    let s = specs("a");
    let vias = ViaRegistry::default();
    let f = fabric(&ws, &s, &vias);
    let ok = s[0].ok_labels();
    // Side-adjacent to foreign dust: short.
    assert!(!f.dust_ok(Pos::new(1, 1, 1), &ok));
    // Diagonally adjacent (one step up): still a short.
    assert!(!f.dust_ok(Pos::new(1, 2, 1), &ok));
    // Two cells away: clear.
    assert!(f.dust_ok(Pos::new(2, 1, 1), &ok));
    // Friendly labels are allowed to touch (multi-terminal joins).
    let mut friendly = NetSpec::new("a");
    friendly.friendly.insert("b".to_string());
    assert!(f.dust_ok(Pos::new(1, 1, 1), &friendly.ok_labels()));
    // Reuse: own-net dust may be re-entered, foreign may not.
    let mut ws2 = Workspace::new();
    ws2.dust(Pos::new(0, 1, 0), "a").unwrap();
    ws2.dust(Pos::new(0, 1, 4), "b").unwrap();
    let f2 = fabric(&ws2, &s, &vias);
    assert!(f2.dust_ok(Pos::new(0, 1, 0), &ok));
    assert!(!f2.dust_ok(Pos::new(0, 1, 4), &ok));
}

/// RULE: a support may not cap diagonal-using dust.
/// Provenance: "broken neighbours' stairs" — a block above dust cuts that
/// dust's up-diagonals, changing someone else's circuit. Flat-run dust caps
/// harmlessly, which is what makes y+1 bridges over existing lanes legal
/// (the HA cell's designed flyover).
#[test]
fn rule_support_may_not_cap_diagonal_dust() {
    // Someone else's stair: dust at y1 connected diagonally to dust at y2.
    let mut ws = Workspace::new();
    ws.dust(Pos::new(0, 1, 0), "b").unwrap();
    ws.dust(Pos::new(1, 2, 0), "b").unwrap();
    let s = specs("a");
    let vias = ViaRegistry::default();
    let f = fabric(&ws, &s, &vias);
    let ok = s[0].ok_labels();
    // New dust at (0,3,0) would place its support at (0,2,0), capping the
    // stair's lower dust: forbidden.
    assert!(!f.dust_ok(Pos::new(0, 3, 0), &ok));

    // Flat run below: capping is harmless, the bridge is legal.
    let mut ws2 = Workspace::new();
    ws2.dust(Pos::new(0, 1, 0), "b").unwrap();
    ws2.dust(Pos::new(1, 1, 0), "b").unwrap();
    ws2.dust(Pos::new(-1, 1, 0), "b").unwrap();
    let f2 = fabric(&ws2, &s, &vias);
    // (0,3,0) sits 2 above the flat b-lane; support at (0,2,0) caps it —
    // but the lane uses no diagonals there. Clearance also passes ((0,3,0)
    // is diagonal to (±1,1,0)? No: dy=2. (±1,2,0) hold no dust.)
    assert!(f2.dust_ok(Pos::new(0, 3, 0), &ok));
}

/// RULE: stair corner cells must be clear.
/// Provenance: "silent opens" — a solid corner above the lower dust cuts
/// the diagonal without any checker noticing.
#[test]
fn rule_stair_corner_must_be_clear() {
    let mut ws = Workspace::new();
    ws.stone(Pos::new(0, 2, 0), "plain").unwrap(); // corner above the lower dust
    let s = specs("a");
    let vias = ViaRegistry::default();
    let f = fabric(&ws, &s, &vias);
    let from = State {
        pos: Pos::new(0, 1, 0),
        mem: StairMem::default(),
    };
    let up = Candidate {
        to: State {
            pos: Pos::new(1, 2, 0),
            mem: StairMem {
                chain: 1,
                prev: Some((1, 0)),
            },
        },
        base_cost: 3,
        tag: RMove::Up,
        footprint: vec![Pos::new(1, 2, 0), Pos::new(1, 1, 0)],
    };
    assert!(!f.legal(&from, &up, &ctx()), "corner-blocked stair passed");
    // Clear workspace: same stair is legal.
    let ws2 = Workspace::new();
    let f2 = fabric(&ws2, &s, &vias);
    assert!(f2.legal(&from, &up, &ctx()));
    // Descending mirror: solid at the corner (q.x, from.y, q.z).
    let mut ws3 = Workspace::new();
    ws3.stone(Pos::new(1, 1, 0), "plain").unwrap();
    let f3 = fabric(&ws3, &s, &vias);
    let down = Candidate {
        to: State {
            pos: Pos::new(1, 0, 0),
            mem: StairMem {
                chain: 1,
                prev: Some((1, 0)),
            },
        },
        base_cost: 3,
        tag: RMove::Down,
        footprint: vec![Pos::new(1, 0, 0), Pos::new(1, -1, 0)],
    };
    assert!(
        !f3.legal(&from, &down, &ctx()),
        "corner-blocked descent passed"
    );
}

/// RULE: stair chains <= 4 (stairs cannot host repeaters).
/// Provenance: "15-cell staircase decayed to 0".
#[test]
fn rule_stair_chain_cap() {
    let ws = Workspace::new();
    let s = specs("a");
    let vias = ViaRegistry::empty();
    let f = fabric(&ws, &s, &vias);
    let capped = State {
        pos: Pos::new(0, 4, 0),
        mem: StairMem {
            chain: 4,
            prev: Some((1, 0)),
        },
    };
    let moves = f.moves(&capped, &ctx());
    assert!(
        moves
            .iter()
            .all(|c| !matches!(c.tag, RMove::Up | RMove::Down)),
        "a 5th consecutive stair was generated"
    );
    // Below the cap, stairs are offered.
    let fresh = State {
        pos: Pos::new(0, 4, 0),
        mem: StairMem {
            chain: 3,
            prev: Some((1, 0)),
        },
    };
    assert!(f
        .moves(&fresh, &ctx())
        .iter()
        .any(|c| matches!(c.tag, RMove::Up)));
}

/// RULE: refresh <= 5 — a repeater is inserted after 5 straight cells.
/// Provenance: the signal budget; decay is a checked invariant of the
/// emitted route, not a convention.
#[test]
fn rule_refresh_interval_inserts_repeaters() {
    let mut ws = Workspace::new();
    let router = RedstoneRouter::new();
    router
        .route(&mut ws, Pos::new(0, 1, 0), Pos::new(15, 1, 0), "n", &[])
        .unwrap();
    // Walk the straight run: no stretch of 5+ consecutive dust cells, and
    // every repeater's input faces the source (west).
    let mut run = 0;
    let mut repeaters = 0;
    for x in 0..=15 {
        let b = ws.get(Pos::new(x, 1, 0)).expect("cell emitted");
        if blocks::is_dust(b) {
            run += 1;
            assert!(run <= 5, "6 refreshless dust cells in a straight run");
        } else {
            assert!(blocks::is_repeater(b), "unexpected block {b}");
            assert_eq!(blocks::facing_of(b), Some("west"));
            repeaters += 1;
            run = 0;
        }
    }
    assert!(repeaters >= 2, "16-cell run needs at least 2 repeaters");
}

/// RULE: no switchback stairs — a stair exactly reversing the previous
/// stair places its support on the cell that cuts the previous diagonal.
/// Provenance: "route cut its own diagonal" (the FA cell's final bug).
#[test]
fn rule_no_switchback_stairs() {
    let ws = Workspace::new();
    let s = specs("a");
    let vias = ViaRegistry::default();
    let f = fabric(&ws, &s, &vias);
    let from = State {
        pos: Pos::new(5, 2, 5),
        mem: StairMem {
            chain: 1,
            prev: Some((1, 0)),
        },
    };
    for c in f.moves(&from, &ctx()) {
        if matches!(c.tag, RMove::Up | RMove::Down) && f.legal(&from, &c, &ctx()) {
            let step = (c.to.pos.x - from.pos.x, c.to.pos.z - from.pos.z);
            assert_ne!(step, (-1, 0), "switchback stair passed legality");
        }
    }
    // A perpendicular stair remains legal.
    let perp = Candidate {
        to: State {
            pos: Pos::new(5, 3, 6),
            mem: StairMem {
                chain: 2,
                prev: Some((0, 1)),
            },
        },
        base_cost: 3,
        tag: RMove::Up,
        footprint: vec![Pos::new(5, 3, 6), Pos::new(5, 2, 6)],
    };
    assert!(f.legal(&from, &perp, &ctx()));
}

/// RULE: torch/comparator base needs *pointing* dust; the climb lays its
/// own dead-end entry. Provenance: "dead ladder climbs" — a lane cell's
/// dust shape may run perpendicular, and dust only powers blocks it points
/// into. Also verifies the emitted ladder against the probe_vert template
/// exactly: base, torch, block, torch, cap, exit dust.
#[test]
fn rule_climb_lays_dead_end_entry_and_exact_ladder() {
    let mut ws = Workspace::new();
    // Ceiling y=2..5 over x -2..8, z -2..2, with a 1-wide chimney at x=4,z=0:
    // stairs cannot climb a 1-wide column (they need horizontal traversal),
    // so only the torch-ladder via reaches the ledge.
    for x in -2..=8 {
        for y in 2..=5 {
            for z in -2..=2 {
                if !(x == 4 && z == 0) {
                    ws.stone(Pos::new(x, y, z), "plain").unwrap();
                }
            }
        }
    }
    let mut router = RedstoneRouter::new();
    router.bounds = Some(Aabb::new(Pos::new(-2, 0, -2), Pos::new(8, 8, 2)));
    let res = router
        .route(&mut ws, Pos::new(0, 1, 0), Pos::new(4, 6, 0), "up", &[])
        .expect("via route");
    assert_eq!(*res.path.last().unwrap(), Pos::new(4, 6, 0));

    // The verified template, cell by cell (probe_vert.py / router.py emit).
    let exp_stone = [Pos::new(4, 1, 0), Pos::new(4, 3, 0), Pos::new(4, 5, 0)];
    for p in exp_stone {
        assert!(ws.solid_at(p), "ladder block missing at {p:?}");
    }
    for p in [Pos::new(4, 2, 0), Pos::new(4, 4, 0)] {
        assert_eq!(ws.get(p), Some(blocks::TORCH), "torch missing at {p:?}");
    }
    assert_eq!(ws.get(Pos::new(4, 6, 0)), Some(DUST), "exit dust missing");
    assert_eq!(ws.label(Pos::new(4, 6, 0)), Some("up"));

    // Entry contract: the entry dust at (3,1,0) is a fresh dead-end whose
    // single neighbour is straight behind — so the base is reliably
    // weak-powered.
    let entry = Pos::new(3, 1, 0);
    assert_eq!(ws.get(entry), Some(DUST));
    assert!(ws.solid_at(entry.offset(0, -1, 0)), "entry support missing");
    let n = nets::neighbours(ws.cells(), entry);
    assert_eq!(n, vec![Pos::new(2, 1, 0)], "entry dust is not a dead end");
}

/// RULE: footprint bounds / keepouts.
/// Provenance: "cell routed through composer's space" — unbounded internal
/// routes stole the composer's port space.
#[test]
fn rule_bounds_confine_routes() {
    let mut ws = Workspace::new();
    // Wall splitting the bounded corridor, gap only at z=2 (inside bounds).
    for z in 0..=1 {
        for y in 1..=3 {
            ws.stone(Pos::new(5, y, z), "plain").unwrap();
        }
    }
    let mut router = RedstoneRouter::new();
    // Bounds apply to routed dust cells (supports may sit one below, like
    // the Python router whose cells.py bounds started at y=1 with supports
    // at y=0).
    router.bounds = Some(Aabb::new(Pos::new(0, 1, 0), Pos::new(10, 3, 2)));
    let res = router
        .route(&mut ws, Pos::new(0, 1, 0), Pos::new(10, 1, 0), "n", &[])
        .expect("path");
    for p in &res.path {
        assert!(router.bounds.unwrap().contains(*p), "escaped bounds: {p:?}");
    }
    // Seal the in-bounds gap: unroutable even though free space exists at
    // z=3, just outside.
    let mut ws2 = Workspace::new();
    for z in 0..=2 {
        for y in 1..=3 {
            ws2.stone(Pos::new(5, y, z), "plain").unwrap();
        }
    }
    assert!(router
        .route(&mut ws2, Pos::new(0, 1, 0), Pos::new(10, 1, 0), "n", &[])
        .is_err());
}

/// RULE: route-to-net must exclude the source's own side.
/// Provenance: cells.py — "the source port is itself labelled cout, so a
/// naive route-to-net would 'succeed' instantly without laying a block."
#[test]
fn rule_route_to_net_excludes_source_side() {
    let mut ws = Workspace::new();
    // A cout net with a west stub (the source's side) and an east rail.
    ws.dust(Pos::new(0, 1, 0), "cout").unwrap();
    for x in 10..=12 {
        ws.dust(Pos::new(x, 1, 0), "cout").unwrap();
    }
    let router = RedstoneRouter::new();
    // Naive: include every cout cell (the source among them) — zero-length.
    let all: Vec<Pos> = ws
        .labels()
        .iter()
        .filter(|(_, l)| l.as_str() == "cout")
        .map(|(p, _)| *p)
        .collect();
    let res = router
        .route_to_net(&mut ws, Pos::new(0, 1, 0), &all, "cout", &[])
        .unwrap();
    assert_eq!(
        res.cells, 1,
        "expected the degenerate self-satisfying route"
    );
    // Excluding the source side forces a real join.
    let east: Vec<Pos> = all.iter().copied().filter(|p| p.x >= 10).collect();
    let res = router
        .route_to_net(&mut ws, Pos::new(0, 1, 0), &east, "cout", &[])
        .unwrap();
    assert!(res.cells > 1);
    assert!(res.path.last().unwrap().x >= 10);
}
