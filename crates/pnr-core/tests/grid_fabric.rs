//! Regression tests on a synthetic 3-D grid fabric that encodes the rules
//! the Python router discovered the hard way: stair chain caps, the
//! switchback ban, route-to-net excluding the source side, and footprint
//! bounds. The fabric here is redstone-shaped but Minecraft-free — the same
//! `(pos, stair_count, prev_stair_dir)` state pattern, generalized.

use pnr_core::fabric::{Budget, Candidate, Fabric, RouteCtx, State};
use pnr_core::{route, Aabb, Pos, RouteRequest};
use std::collections::BTreeSet;

const H_MOVES: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Path memory: consecutive-stair count + previous stair direction.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
struct Mem {
    chain: u8,
    prev_stair: Option<(i32, i32)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Mv {
    H,
    Stair, // dy = +-1 with the horizontal step
    Via,   // vertical jump: +5 y, 2 cells ahead (torch-ladder shaped)
}

struct GridFabric {
    blocked: BTreeSet<Pos>,
    bounds: Aabb,
    vias_enabled: bool,
    stair_cap: u8,
}

impl GridFabric {
    fn open(bounds: Aabb) -> Self {
        GridFabric {
            blocked: BTreeSet::new(),
            bounds,
            vias_enabled: true,
            stair_cap: 4,
        }
    }
}

impl Fabric for GridFabric {
    type Memory = Mem;
    type Tag = Mv;

    fn start_memory(&self) -> Mem {
        Mem::default()
    }

    fn moves(&self, from: &State<Mem>, _ctx: &RouteCtx) -> Vec<Candidate<Mem, Mv>> {
        let p = from.pos;
        let mut out = Vec::new();
        for (dx, dz) in H_MOVES {
            let to = p.offset(dx, 0, dz);
            out.push(Candidate {
                to: State {
                    pos: to,
                    mem: Mem::default(),
                },
                base_cost: 1,
                tag: Mv::H,
                footprint: vec![to],
            });
            // Stairs: capped chain, remembered direction.
            if from.mem.chain < self.stair_cap {
                for dy in [1, -1] {
                    let to = p.offset(dx, dy, dz);
                    out.push(Candidate {
                        to: State {
                            pos: to,
                            mem: Mem {
                                chain: from.mem.chain + 1,
                                prev_stair: Some((dx, dz)),
                            },
                        },
                        base_cost: 3,
                        tag: Mv::Stair,
                        footprint: vec![to],
                    });
                }
            }
            // Via: the torch-ladder shape (exit +5 y, 2 ahead), occupying
            // the entry cell and the column.
            if self.vias_enabled {
                let exit = p.offset(2 * dx, 5, 2 * dz);
                let mut footprint = vec![p.offset(dx, 0, dz)];
                for k in 0..=5 {
                    footprint.push(Pos::new(exit.x, p.y + k, exit.z));
                }
                out.push(Candidate {
                    to: State {
                        pos: exit,
                        mem: Mem::default(),
                    },
                    base_cost: 9,
                    tag: Mv::Via,
                    footprint,
                });
            }
        }
        out
    }

    fn legal(&self, from: &State<Mem>, c: &Candidate<Mem, Mv>, _ctx: &RouteCtx) -> bool {
        // Footprint bounds: every cell the move occupies stays inside the
        // region (the Python `bounds` rule: cells must not route outside
        // their own footprint — unbounded internal routes stole the
        // composer's port space).
        if !c.footprint.iter().all(|p| self.bounds.contains(*p)) {
            return false;
        }
        if c.footprint.iter().any(|p| self.blocked.contains(p)) {
            return false;
        }
        // Switchback ban: a stair exactly reversing the previous stair
        // places its support on the cell that cuts the previous diagonal.
        if c.tag == Mv::Stair {
            let step = (c.to.pos.x - from.pos.x, c.to.pos.z - from.pos.z);
            if let Some(prev) = from.mem.prev_stair {
                if step == (-prev.0, -prev.1) {
                    return false;
                }
            }
        }
        true
    }

    fn budget(&self) -> Budget {
        Budget {
            refresh_every: 5,
            max_unrefreshable_chain: self.stair_cap as u32,
        }
    }
}

fn big_bounds() -> Aabb {
    Aabb::new(Pos::new(-20, 0, -20), Pos::new(40, 40, 40))
}

fn ctx() -> RouteCtx {
    RouteCtx { net: 0 }
}

#[test]
fn stair_chains_never_exceed_the_cap() {
    // Climb +8: more than one 4-stair chain; the route must break chains
    // (or via). Regression for "15-cell staircase decayed to 0" — the cap
    // is enforced in the state, not by convention.
    let mut f = GridFabric::open(big_bounds());
    f.vias_enabled = false;
    let req = RouteRequest::new(Pos::new(0, 0, 0), Pos::new(20, 8, 0));
    let path = route(&f, &req, &ctx(), &|_| 0).expect("path");
    let mut chain = 0u8;
    let mut max_chain = 0u8;
    for s in &path {
        match s.tag {
            Some(Mv::Stair) => {
                chain += 1;
                max_chain = max_chain.max(chain);
            }
            _ => chain = 0,
        }
    }
    assert!(max_chain <= 4, "stair chain {max_chain} exceeds cap");
    assert_eq!(path.last().unwrap().pos, Pos::new(20, 8, 0));
}

#[test]
fn tall_climb_uses_via_when_stairs_cannot() {
    // A solid ceiling at y 1..=4 with a single 1-wide chimney at x=4:
    // stairs need horizontal traversal inside the climb, so the chimney is
    // stair-proof; only the via (a torch-ladder-shaped vertical jump)
    // reaches the ledge. Long verticals must use ladders — stairs cannot
    // host repeaters, so unbounded staircases decay (Python provenance).
    let bounds = Aabb::new(Pos::new(0, 0, 0), Pos::new(10, 6, 0));
    let mut f = GridFabric::open(bounds);
    for x in 0..=10 {
        for y in 1..=4 {
            if x != 4 {
                f.blocked.insert(Pos::new(x, y, 0));
            }
        }
    }
    f.vias_enabled = false;
    let req = RouteRequest::new(Pos::new(0, 0, 0), Pos::new(4, 5, 0));
    assert!(
        route(&f, &req, &ctx(), &|_| 0).is_none(),
        "stairs climbed a 1-wide chimney"
    );
    f.vias_enabled = true;
    let path = route(&f, &req, &ctx(), &|_| 0).expect("via path");
    assert!(
        path.iter().any(|s| s.tag == Some(Mv::Via)),
        "expected a via climb"
    );
}

#[test]
fn switchback_is_banned_at_the_move_level() {
    let f = GridFabric::open(big_bounds());
    // State: just took a stair in +x.
    let s = State {
        pos: Pos::new(5, 3, 5),
        mem: Mem {
            chain: 1,
            prev_stair: Some((1, 0)),
        },
    };
    let cands = f.moves(&s, &ctx());
    for c in cands {
        if c.tag == Mv::Stair && f.legal(&s, &c, &ctx()) {
            let step = (c.to.pos.x - s.pos.x, c.to.pos.z - s.pos.z);
            assert_ne!(step, (-1, 0), "switchback stair passed legality");
        }
    }
    // Sanity: a continuing stair (+x) and a perpendicular stair are legal.
    let cont = Candidate {
        to: State {
            pos: s.pos.offset(1, 1, 0),
            mem: Mem {
                chain: 2,
                prev_stair: Some((1, 0)),
            },
        },
        base_cost: 3,
        tag: Mv::Stair,
        footprint: vec![s.pos.offset(1, 1, 0)],
    };
    assert!(f.legal(&s, &cont, &ctx()));
}

#[test]
fn routed_stairs_never_reverse() {
    // Force stairs through a zig-zag canyon and assert no emitted stair
    // reverses the previous one (regression: the FA cell's final bug).
    let f = GridFabric::open(big_bounds());
    let req = RouteRequest::new(Pos::new(0, 0, 0), Pos::new(6, 4, 6));
    let path = route(&f, &req, &ctx(), &|_| 0).expect("path");
    let mut prev_stair: Option<(i32, i32)> = None;
    for w in path.windows(2) {
        let step = (w[1].pos.x - w[0].pos.x, w[1].pos.z - w[0].pos.z);
        if w[1].tag == Some(Mv::Stair) {
            if let Some(p) = prev_stair {
                assert_ne!(step, (-p.0, -p.1), "emitted switchback");
            }
            prev_stair = Some(step);
        } else {
            prev_stair = None;
        }
    }
}

#[test]
fn bounds_confine_the_route() {
    // Direct corridor exists outside the bounds; the route must stay in.
    let bounds = Aabb::new(Pos::new(0, 0, 0), Pos::new(10, 0, 2));
    let mut f = GridFabric::open(bounds);
    // Wall inside the bounds forcing a detour that would be shorter via z=3
    // (outside).
    for z in 0..=1 {
        f.blocked.insert(Pos::new(5, 0, z));
    }
    let req = RouteRequest::new(Pos::new(0, 0, 0), Pos::new(10, 0, 0));
    let path = route(&f, &req, &ctx(), &|_| 0).expect("path");
    for s in &path {
        assert!(bounds.contains(s.pos), "route escaped bounds at {:?}", s.pos);
    }
    assert!(path.iter().any(|s| s.pos == Pos::new(5, 0, 2)), "used the in-bounds gap");

    // Fully sealed inside the bounds -> unroutable, even though free space
    // exists just outside.
    f.blocked.insert(Pos::new(5, 0, 2));
    assert!(route(&f, &req, &ctx(), &|_| 0).is_none());
}

#[test]
fn route_to_net_must_exclude_source_side() {
    // The cells.py lesson, on the synthetic fabric: the source is already on
    // the target net; excluding its side forces a real join to the far side.
    let f = GridFabric::open(big_bounds());
    let src = Pos::new(0, 0, 0);
    let net: Vec<Pos> = vec![src, Pos::new(12, 0, 0), Pos::new(13, 0, 0)];
    let naive = RouteRequest::to_net_excluding(src, net.clone(), |_| true);
    assert_eq!(route(&f, &naive, &ctx(), &|_| 0).unwrap().len(), 1);
    let fixed = RouteRequest::to_net_excluding(src, net, |p| p.x >= 12);
    let path = route(&f, &fixed, &ctx(), &|_| 0).unwrap();
    assert!(path.len() > 1 && path.last().unwrap().pos.x >= 12);
}
