//! Generic static timing analysis over a delay DAG.
//!
//! Nodes are caller-defined indices; edges carry integer delays. Arrival
//! times propagate from sources in topological order; the worst-arrival
//! predecessor is recorded so critical paths can be walked back. Cycles are
//! an error (a combinational loop — the redstone analogue is the repeater
//! ring latch, which DRC reports separately).

use std::collections::BTreeMap;

/// A delay graph under construction.
#[derive(Clone, Debug, Default)]
pub struct DelayGraph {
    n_nodes: usize,
    edges: Vec<(usize, usize, u32)>,
}

/// STA result: arrival per node (None = unreachable from any source) and
/// worst-arrival predecessor per node.
#[derive(Clone, Debug)]
pub struct StaResult {
    /// Arrival time per node.
    pub arrival: Vec<Option<u64>>,
    /// Predecessor on the worst path per node.
    pub pred: Vec<Option<usize>>,
}

/// Error: the graph has a directed cycle among the listed nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CycleError {
    /// Nodes that never became ready (a superset of the cycle members).
    pub nodes: Vec<usize>,
}

impl DelayGraph {
    /// A graph with `n_nodes` nodes and no edges.
    pub fn new(n_nodes: usize) -> Self {
        DelayGraph {
            n_nodes,
            edges: Vec::new(),
        }
    }

    /// Add a directed edge `from -> to` with the given delay.
    pub fn edge(&mut self, from: usize, to: usize, delay: u32) -> &mut Self {
        assert!(
            from < self.n_nodes && to < self.n_nodes,
            "node out of range"
        );
        self.edges.push((from, to, delay));
        self
    }

    /// Number of nodes.
    pub fn len(&self) -> usize {
        self.n_nodes
    }

    /// Whether the graph has no nodes.
    pub fn is_empty(&self) -> bool {
        self.n_nodes == 0
    }

    /// Propagate arrivals from `(node, time)` sources. Nodes not reachable
    /// from a source get `None`.
    pub fn analyze(&self, sources: &[(usize, u64)]) -> Result<StaResult, CycleError> {
        let mut indeg = vec![0usize; self.n_nodes];
        let mut out: BTreeMap<usize, Vec<(usize, u32)>> = BTreeMap::new();
        for (u, v, d) in &self.edges {
            indeg[*v] += 1;
            out.entry(*u).or_default().push((*v, *d));
        }
        let mut arrival: Vec<Option<u64>> = vec![None; self.n_nodes];
        let mut pred: Vec<Option<usize>> = vec![None; self.n_nodes];
        for (n, t) in sources {
            let e = arrival[*n].get_or_insert(0);
            *e = (*e).max(*t);
        }
        // Kahn over the whole graph (deterministic: ascending node order).
        let mut ready: Vec<usize> = (0..self.n_nodes).filter(|n| indeg[*n] == 0).collect();
        ready.reverse(); // pop from the back => ascending order
        let mut processed = 0usize;
        while let Some(u) = ready.pop() {
            processed += 1;
            if let Some(succs) = out.get(&u) {
                for (v, d) in succs {
                    if let Some(t) = arrival[u] {
                        let cand = t + *d as u64;
                        match arrival[*v] {
                            Some(cur) if cur >= cand => {}
                            _ => {
                                arrival[*v] = Some(cand);
                                pred[*v] = Some(u);
                            }
                        }
                    }
                    indeg[*v] -= 1;
                    if indeg[*v] == 0 {
                        ready.push(*v);
                    }
                }
            }
            ready.sort_unstable_by(|a, b| b.cmp(a)); // keep ascending pop order
        }
        if processed != self.n_nodes {
            let nodes: Vec<usize> = (0..self.n_nodes).filter(|n| indeg[*n] > 0).collect();
            return Err(CycleError { nodes });
        }
        Ok(StaResult { arrival, pred })
    }
}

impl StaResult {
    /// Walk the critical path back from `sink` (returned source-first).
    pub fn critical_path(&self, sink: usize) -> Vec<usize> {
        let mut path = vec![sink];
        let mut cur = sink;
        while let Some(p) = self.pred[cur] {
            path.push(p);
            cur = p;
        }
        path.reverse();
        path
    }

    /// Slack per queried sink: `required - arrival` (negative = violated).
    /// Unreachable sinks report `None`.
    pub fn slack(&self, required: &[(usize, u64)]) -> Vec<(usize, Option<i64>)> {
        required
            .iter()
            .map(|(n, req)| (*n, self.arrival[*n].map(|a| *req as i64 - a as i64)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diamond_takes_worst_arrival() {
        // 0 -> 1 (1), 0 -> 2 (5), 1 -> 3 (1), 2 -> 3 (1)
        let mut g = DelayGraph::new(4);
        g.edge(0, 1, 1).edge(0, 2, 5).edge(1, 3, 1).edge(2, 3, 1);
        let r = g.analyze(&[(0, 0)]).unwrap();
        assert_eq!(r.arrival[3], Some(6));
        assert_eq!(r.critical_path(3), vec![0, 2, 3]);
        let s = r.slack(&[(3, 10), (3, 5)]);
        assert_eq!(s[0].1, Some(4));
        assert_eq!(s[1].1, Some(-1));
    }

    #[test]
    fn cycle_is_an_error() {
        let mut g = DelayGraph::new(3);
        g.edge(0, 1, 1).edge(1, 2, 1).edge(2, 1, 1);
        let err = g.analyze(&[(0, 0)]).unwrap_err();
        assert!(err.nodes.contains(&1) && err.nodes.contains(&2));
    }

    #[test]
    fn unreachable_nodes_have_no_arrival() {
        let mut g = DelayGraph::new(3);
        g.edge(0, 1, 2);
        let r = g.analyze(&[(0, 3)]).unwrap();
        assert_eq!(r.arrival[1], Some(5));
        assert_eq!(r.arrival[2], None);
    }
}
