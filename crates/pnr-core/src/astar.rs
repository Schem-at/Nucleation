//! Weighted grid A* over a [`Fabric`], with route-to-net targets and
//! external per-cell costs (the hook negotiated congestion uses).

use crate::fabric::{Fabric, RouteCtx, State};
use crate::grid::Pos;
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap, HashMap};

/// A routing request: source cell, target cell set, weighting and budget.
#[derive(Clone, Debug)]
pub struct RouteRequest {
    /// Source cell.
    pub src: Pos,
    /// Target cells. Reaching ANY of them completes the route
    /// (multi-terminal / route-to-net joining).
    ///
    /// Route-to-net trap (paid for in the Python prototype): when joining a
    /// source to its OWN net, the caller must exclude the source's side of
    /// the net from this set, or the route "succeeds" instantly without
    /// laying a single cell. See [`RouteRequest::to_net_excluding`].
    pub dsts: BTreeSet<Pos>,
    /// Heuristic weight as a rational `num/den` (weighted A*; the prototype
    /// used 1.3 = 13/10: slightly suboptimal paths, much less exploration).
    pub eps_num: u32,
    /// Denominator of the heuristic weight.
    pub eps_den: u32,
    /// Iteration cap; the search reports failure when exhausted.
    pub max_iter: usize,
}

impl RouteRequest {
    /// A request with the prototype's defaults (eps = 1.3, 1.2M iterations).
    pub fn new(src: Pos, dst: Pos) -> Self {
        let mut dsts = BTreeSet::new();
        dsts.insert(dst);
        RouteRequest {
            src,
            dsts,
            eps_num: 13,
            eps_den: 10,
            max_iter: 1_200_000,
        }
    }

    /// Route-to-net: target every cell of `net_cells` that survives
    /// `keep`. Pass a predicate that excludes the source's own side of the
    /// net — the regression that mandated this: a source port already
    /// labelled with the target net "routed" in zero cells.
    pub fn to_net_excluding(
        src: Pos,
        net_cells: impl IntoIterator<Item = Pos>,
        keep: impl Fn(Pos) -> bool,
    ) -> Self {
        let dsts: BTreeSet<Pos> = net_cells.into_iter().filter(|p| keep(*p)).collect();
        RouteRequest {
            src,
            dsts,
            eps_num: 13,
            eps_den: 10,
            max_iter: 1_200_000,
        }
    }
}

/// One step of a found path: the cell plus the move tag that entered it.
/// The first step is the source and carries no tag.
#[derive(Clone, Debug)]
pub struct PathStep<T> {
    /// Cell position.
    pub pos: Pos,
    /// Move that entered this cell (`None` for the source).
    pub tag: Option<T>,
}

/// Weighted A* over the fabric. `extra_cost` is charged per footprint cell
/// of every candidate (history + present-sharing costs during negotiation;
/// pass `|_| 0` when routing alone). Returns `None` when no legal path
/// exists within the iteration budget.
pub fn route<F: Fabric>(
    fabric: &F,
    req: &RouteRequest,
    ctx: &RouteCtx,
    extra_cost: &dyn Fn(Pos) -> u32,
) -> Option<Vec<PathStep<F::Tag>>> {
    if req.dsts.is_empty() {
        return None;
    }
    let h = |p: Pos| -> u64 {
        req.dsts
            .iter()
            .map(|q| p.manhattan(*q) as u64)
            .min()
            .unwrap_or(0)
    };
    // Integer weighted A*: order by eps_den*g + eps_num*h, which is
    // order-equivalent to g + (eps_num/eps_den)*h. A monotone sequence
    // number breaks ties deterministically (FIFO among equals).
    let f = |g: u64, hp: u64| -> u64 { req.eps_den as u64 * g + req.eps_num as u64 * hp };

    let start = State {
        pos: req.src,
        mem: fabric.start_memory(),
    };
    // Trivial completion: the source is already on the target net. Callers
    // joining a net must exclude the source side (see RouteRequest docs).
    if req.dsts.contains(&req.src) {
        return Some(vec![PathStep {
            pos: req.src,
            tag: None,
        }]);
    }

    let mut seq: u64 = 0;
    let mut open: BinaryHeap<Reverse<(u64, u64, usize)>> = BinaryHeap::new();
    // Heap entries carry an id into `states`; states/came/gbest are parallel.
    let mut states: Vec<State<F::Memory>> = vec![start.clone()];
    let mut tags: Vec<Option<F::Tag>> = vec![None];
    let mut parent: Vec<Option<usize>> = vec![None];
    let mut gbest: HashMap<State<F::Memory>, (u64, usize)> = HashMap::new();
    gbest.insert(start.clone(), (0, 0));
    open.push(Reverse((f(0, h(req.src)), seq, 0)));

    let mut it = 0usize;
    while let Some(Reverse((_, _, id))) = open.pop() {
        it += 1;
        if it > req.max_iter {
            break;
        }
        let s = states[id].clone();
        let (g, best_id) = *gbest.get(&s).expect("popped state was inserted");
        if best_id != id {
            continue; // stale heap entry
        }
        if req.dsts.contains(&s.pos) {
            // Reconstruct.
            let mut path = Vec::new();
            let mut cur = Some(id);
            while let Some(i) = cur {
                path.push(PathStep {
                    pos: states[i].pos,
                    tag: tags[i].clone(),
                });
                cur = parent[i];
            }
            path.reverse();
            return Some(path);
        }
        let ctx = *ctx;
        for cand in fabric.moves(&s, &ctx) {
            if !fabric.legal(&s, &cand, &ctx) {
                continue;
            }
            let step: u64 = fabric.cost(&cand, &ctx) as u64
                + cand
                    .footprint
                    .iter()
                    .map(|p| extra_cost(*p) as u64)
                    .sum::<u64>();
            let ng = g + step;
            let better = match gbest.get(&cand.to) {
                Some((old, _)) => ng < *old,
                None => true,
            };
            if better {
                seq += 1;
                let nid = states.len();
                states.push(cand.to.clone());
                tags.push(Some(cand.tag.clone()));
                parent.push(Some(id));
                gbest.insert(cand.to.clone(), (ng, nid));
                open.push(Reverse((f(ng, h(cand.to.pos)), seq, nid)));
            }
        }
    }
    None
}

/// Convenience: total footprint of a path given the fabric's candidates is
/// not reconstructible after the fact, so paths report their cells directly.
pub fn path_cells<T>(path: &[PathStep<T>]) -> Vec<Pos> {
    path.iter().map(|s| s.pos).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fabric::{Budget, Candidate};
    use std::collections::BTreeSet;

    /// Minimal 2-D fabric: 4-neighbour moves on y=0, blocked set.
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
            !self.blocked.contains(&c.to.pos) && c.to.pos.x.abs() <= 20 && c.to.pos.z.abs() <= 20
        }
        fn budget(&self) -> Budget {
            Budget::default()
        }
    }

    #[test]
    fn routes_around_a_wall() {
        // Wall at x=2 for z in -3..=3 with a gap at z=4.
        let blocked: BTreeSet<Pos> = (-3..=3).map(|z| Pos::new(2, 0, z)).collect();
        let f = Flat { blocked };
        let req = RouteRequest::new(Pos::new(0, 0, 0), Pos::new(5, 0, 0));
        let path = route(&f, &req, &RouteCtx { net: 0 }, &|_| 0).expect("path");
        assert_eq!(path.first().unwrap().pos, Pos::new(0, 0, 0));
        assert_eq!(path.last().unwrap().pos, Pos::new(5, 0, 0));
        for s in &path {
            assert!(!f.blocked.contains(&s.pos), "path crossed the wall");
        }
    }

    #[test]
    fn route_to_net_including_source_self_satisfies_in_zero_cells() {
        // The trap the Python session hit verbatim: the source port is
        // itself labelled with the target net, so a naive route-to-net
        // returns a zero-length route.
        let f = Flat {
            blocked: BTreeSet::new(),
        };
        let src = Pos::new(0, 0, 0);
        let net = vec![src, Pos::new(6, 0, 0), Pos::new(7, 0, 0)];
        let naive = RouteRequest::to_net_excluding(src, net.clone(), |_| true);
        let path = route(&f, &naive, &RouteCtx { net: 0 }, &|_| 0).expect("path");
        assert_eq!(path.len(), 1, "self-satisfied without laying a cell");

        // Excluding the source's side forces a real route to the far side.
        let fixed = RouteRequest::to_net_excluding(src, net, |p| p.x >= 6);
        let path = route(&f, &fixed, &RouteCtx { net: 0 }, &|_| 0).expect("path");
        assert!(path.len() > 1);
        assert!(path.last().unwrap().pos.x >= 6);
    }

    #[test]
    fn unreachable_returns_none() {
        // Fully enclosed target.
        let mut blocked = BTreeSet::new();
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            blocked.insert(Pos::new(10 + dx, 0, dz));
        }
        let f = Flat { blocked };
        let req = RouteRequest::new(Pos::new(0, 0, 0), Pos::new(10, 0, 0));
        assert!(route(&f, &req, &RouteCtx { net: 0 }, &|_| 0).is_none());
    }

    #[test]
    fn deterministic_across_runs() {
        let blocked: BTreeSet<Pos> = (-2..=2).map(|z| Pos::new(3, 0, z)).collect();
        let f = Flat { blocked };
        let req = RouteRequest::new(Pos::new(0, 0, 0), Pos::new(6, 0, 1));
        let a = route(&f, &req, &RouteCtx { net: 0 }, &|_| 0).unwrap();
        let b = route(&f, &req, &RouteCtx { net: 0 }, &|_| 0).unwrap();
        let pa: Vec<Pos> = a.iter().map(|s| s.pos).collect();
        let pb: Vec<Pos> = b.iter().map(|s| s.pos).collect();
        assert_eq!(pa, pb);
    }
}
