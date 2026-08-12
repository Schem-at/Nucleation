//! Negotiated-congestion `route_all` (PathFinder): route every net, let
//! paths overlap, then iterate rip-up-and-reroute with escalating history
//! costs on contested cells until the solution is conflict-free.
//!
//! The Python prototype routed nets one by one in a hand-tuned order and
//! paid for it across three separate debugging sessions ("tightest nets
//! first" whack-a-mole); negotiation is the scalable replacement and the
//! default entry point here.

use crate::astar::{route, PathStep, RouteRequest};
use crate::fabric::{Fabric, RouteCtx};
use crate::grid::Pos;
use std::collections::{BTreeMap, BTreeSet};

/// One net to route.
#[derive(Clone, Debug)]
pub struct NetReq {
    /// Caller-assigned net id, handed to the fabric via [`RouteCtx`].
    pub net: usize,
    /// The A* request (source, targets, weights).
    pub req: RouteRequest,
}

/// Options for the negotiation loop.
#[derive(Clone, Debug)]
pub struct CongestionOpts {
    /// Maximum rip-up/reroute rounds before giving up.
    pub max_rounds: usize,
    /// History-cost increment added to every contested cell after a round.
    pub history_increment: u32,
    /// Cost added per cell currently claimed by another net's tentative
    /// path (the "present sharing" term).
    pub present_penalty: u32,
}

impl Default for CongestionOpts {
    fn default() -> Self {
        CongestionOpts {
            max_rounds: 40,
            history_increment: 4,
            present_penalty: 6,
        }
    }
}

/// Failure report: which nets could not be routed, and which cells were
/// still contested when the round budget ran out.
#[derive(Clone, Debug)]
pub struct RouteAllError {
    /// Nets for which A* found no path in the final round.
    pub unrouted: Vec<usize>,
    /// Cells still claimed by more than one net after the last round.
    pub contested: Vec<Pos>,
}

/// Route all nets with negotiated congestion.
///
/// `extra_conflicts` lets the caller declare additional conflicts between
/// tentative paths that plain cell-sharing cannot see — the redstone fabric
/// uses it for electrical adjacency (two dust runs one cell apart short
/// without ever sharing a cell). It receives every net's footprint and
/// returns the cells to penalize.
pub fn route_all<F: Fabric>(
    fabric: &F,
    nets: &[NetReq],
    opts: &CongestionOpts,
    extra_conflicts: &dyn Fn(&BTreeMap<usize, Vec<Pos>>) -> Vec<Pos>,
) -> Result<BTreeMap<usize, Vec<PathStep<F::Tag>>>, RouteAllError> {
    let mut history: BTreeMap<Pos, u32> = BTreeMap::new();
    let mut paths: BTreeMap<usize, Vec<PathStep<F::Tag>>> = BTreeMap::new();
    let mut footprints: BTreeMap<usize, Vec<Pos>> = BTreeMap::new();

    for _round in 0..opts.max_rounds {
        let mut unrouted = Vec::new();
        for nr in nets {
            // Rip up this net's previous path before rerouting it.
            footprints.remove(&nr.net);
            paths.remove(&nr.net);

            // Present usage by every OTHER net's tentative path.
            let mut present: BTreeMap<Pos, u32> = BTreeMap::new();
            for cells in footprints.values() {
                for p in cells {
                    *present.entry(*p).or_insert(0) += 1;
                }
            }
            let extra = |p: Pos| -> u32 {
                history.get(&p).copied().unwrap_or(0)
                    + present.get(&p).copied().unwrap_or(0) * opts.present_penalty
            };
            let ctx = RouteCtx { net: nr.net };
            match route(fabric, &nr.req, &ctx, &extra) {
                Some(path) => {
                    let cells: Vec<Pos> = path.iter().map(|s| s.pos).collect();
                    footprints.insert(nr.net, cells);
                    paths.insert(nr.net, path);
                }
                None => unrouted.push(nr.net),
            }
        }

        // Conflicts: cells shared by 2+ nets, plus caller-declared ones.
        let mut users: BTreeMap<Pos, BTreeSet<usize>> = BTreeMap::new();
        for (net, cells) in &footprints {
            for p in cells {
                users.entry(*p).or_default().insert(*net);
            }
        }
        let mut contested: BTreeSet<Pos> = users
            .iter()
            .filter(|(_, u)| u.len() > 1)
            .map(|(p, _)| *p)
            .collect();
        for p in extra_conflicts(&footprints) {
            contested.insert(p);
        }

        if contested.is_empty() && unrouted.is_empty() {
            return Ok(paths);
        }
        if contested.is_empty() && !unrouted.is_empty() {
            // No congestion left to negotiate away: genuinely unroutable.
            return Err(RouteAllError {
                unrouted,
                contested: Vec::new(),
            });
        }
        for p in &contested {
            *history.entry(*p).or_insert(0) += opts.history_increment;
        }
    }

    // Round budget exhausted: report the residual state.
    let mut users: BTreeMap<Pos, BTreeSet<usize>> = BTreeMap::new();
    for (net, cells) in &footprints {
        for p in cells {
            users.entry(*p).or_default().insert(*net);
        }
    }
    let contested: Vec<Pos> = users
        .iter()
        .filter(|(_, u)| u.len() > 1)
        .map(|(p, _)| *p)
        .collect();
    let routed: BTreeSet<usize> = paths.keys().copied().collect();
    let unrouted: Vec<usize> = nets
        .iter()
        .map(|n| n.net)
        .filter(|n| !routed.contains(n))
        .collect();
    Err(RouteAllError {
        unrouted,
        contested,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fabric::{Budget, Candidate, State};
    use std::collections::BTreeSet;

    /// Flat fabric with walls; overlap between nets is allowed at search
    /// time (negotiation handles it), walls are hard-illegal.
    struct Flat {
        blocked: BTreeSet<Pos>,
    }
    impl Fabric for Flat {
        type Memory = ();
        type Tag = char;
        fn start_memory(&self) {}
        fn moves(&self, from: &State<()>, _ctx: &RouteCtx) -> Vec<Candidate<(), char>> {
            [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .map(|(dx, dz)| {
                    let to = from.pos.offset(*dx, 0, *dz);
                    Candidate {
                        to: State { pos: to, mem: () },
                        base_cost: 1,
                        tag: 'h',
                        footprint: vec![to],
                    }
                })
                .collect()
        }
        fn legal(&self, _f: &State<()>, c: &Candidate<(), char>, _ctx: &RouteCtx) -> bool {
            !self.blocked.contains(&c.to.pos)
                && c.to.pos.x.abs() <= 30
                && c.to.pos.z.abs() <= 30
                && c.to.pos.y == 0
        }
        fn budget(&self) -> Budget {
            Budget::default()
        }
    }

    #[test]
    fn two_nets_negotiate_two_gaps() {
        // A wall with two one-cell gaps; both nets' shortest paths prefer
        // the same (nearer) gap, negotiation must split them.
        let mut blocked = BTreeSet::new();
        for z in -10..=10 {
            if z != 0 && z != 5 {
                blocked.insert(Pos::new(5, 0, z));
            }
        }
        let f = Flat { blocked };
        let nets = vec![
            NetReq {
                net: 0,
                req: RouteRequest::new(Pos::new(0, 0, 0), Pos::new(10, 0, 0)),
            },
            NetReq {
                net: 1,
                req: RouteRequest::new(Pos::new(0, 0, 1), Pos::new(10, 0, 1)),
            },
        ];
        let paths = route_all(&f, &nets, &CongestionOpts::default(), &|_| Vec::new())
            .expect("both nets route");
        let a: BTreeSet<Pos> = paths[&0].iter().map(|s| s.pos).collect();
        let b: BTreeSet<Pos> = paths[&1].iter().map(|s| s.pos).collect();
        assert!(a.is_disjoint(&b), "negotiation left shared cells");
        // One through each gap.
        let gaps_a = a.contains(&Pos::new(5, 0, 0)) as u8 + a.contains(&Pos::new(5, 0, 5)) as u8;
        let gaps_b = b.contains(&Pos::new(5, 0, 0)) as u8 + b.contains(&Pos::new(5, 0, 5)) as u8;
        assert_eq!(gaps_a, 1);
        assert_eq!(gaps_b, 1);
    }

    #[test]
    fn extra_conflicts_forces_spacing() {
        // No walls; declare a conflict whenever two paths run adjacent in z.
        // (This is the electrical-clearance hook the redstone fabric uses.)
        let f = Flat {
            blocked: BTreeSet::new(),
        };
        let nets = vec![
            NetReq {
                net: 0,
                req: RouteRequest::new(Pos::new(0, 0, 0), Pos::new(8, 0, 0)),
            },
            NetReq {
                net: 1,
                req: RouteRequest::new(Pos::new(0, 0, 1), Pos::new(8, 0, 1)),
            },
        ];
        let adjacency = |fp: &BTreeMap<usize, Vec<Pos>>| -> Vec<Pos> {
            let mut out = Vec::new();
            let nets: Vec<(&usize, &Vec<Pos>)> = fp.iter().collect();
            for i in 0..nets.len() {
                for j in i + 1..nets.len() {
                    for a in nets[i].1 {
                        for b in nets[j].1 {
                            // endpoints are pinned; only flag interior cells
                            if a.manhattan(*b) <= 1 && a.x > 0 && a.x < 8 && b.x > 0 && b.x < 8 {
                                out.push(*a);
                                out.push(*b);
                            }
                        }
                    }
                }
            }
            out
        };
        let paths = route_all(&f, &nets, &CongestionOpts::default(), &adjacency).expect("routes");
        // Interior cells of the two paths are never z-adjacent.
        for a in paths[&0]
            .iter()
            .map(|s| s.pos)
            .filter(|p| p.x > 0 && p.x < 8)
        {
            for b in paths[&1]
                .iter()
                .map(|s| s.pos)
                .filter(|p| p.x > 0 && p.x < 8)
            {
                assert!(a.manhattan(b) > 1, "paths {a:?} and {b:?} touch");
            }
        }
    }

    #[test]
    fn truly_unroutable_reports_unrouted() {
        let mut blocked = BTreeSet::new();
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            blocked.insert(Pos::new(20 + dx, 0, dz));
        }
        let f = Flat { blocked };
        let nets = vec![NetReq {
            net: 7,
            req: RouteRequest::new(Pos::new(0, 0, 0), Pos::new(20, 0, 0)),
        }];
        let err = route_all(&f, &nets, &CongestionOpts::default(), &|_| Vec::new())
            .expect_err("unroutable");
        assert_eq!(err.unrouted, vec![7]);
    }
}
