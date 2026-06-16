//! Canonical, fast graph-analysis kernels over [`RedstoneGraph`] (Phase 4 Part A
//! of the redstone-graph effort).
//!
//! These kernels return **data, not verdicts**: counts, components, depths and
//! fan metrics. They deliberately contain NO classification/naming logic (e.g.
//! "this is a 4-bit adder") — that interpretation lives in downstream Python.
//! Everything is pure Rust over the `Vec<RedstoneNode>` adjacency; no petgraph.
//!
//! The graph stores INCOMING edges on each node (`node.inputs`, where
//! `link.from` is the source node id). Several kernels need out-adjacency, so we
//! invert `inputs` into an out-adjacency list exactly once and reuse it.

use super::graph::{RedstoneGraph, RedstoneNodeKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Aggregate, serde-serializable feature vector for a [`RedstoneGraph`].
///
/// Computed by [`RedstoneGraph::features`], which derives the SCC decomposition
/// once and reuses it for every cycle/depth metric.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphFeatures {
    pub node_count: usize,
    pub edge_count: usize,
    pub node_kind_counts: BTreeMap<String, usize>,
    pub has_cycles: bool,
    pub is_combinational: bool,
    pub critical_path: u32,
    pub delay_weighted_depth: u32,
    pub scc_count: usize,
    pub largest_scc: usize,
    pub weakly_connected_components: usize,
    pub max_fan_in: usize,
    pub max_fan_out: usize,
    /// Heuristic input/output node counts by component kind. NOTE: the mchprs
    /// lowering drops the real is_input/is_output flags, so these are inferred
    /// from kind (inputs ≈ Lever/Button/PressurePlate; outputs ≈ Lamp/Trapdoor/
    /// NoteBlock) and are approximate.
    pub approx_input_count: usize,
    pub approx_output_count: usize,
}

impl GraphFeatures {
    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// Deserialize from JSON.
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}

/// Stable discriminant name for a node kind (payload such as repeater delay or
/// comparator mode is ignored — one bucket per discriminant).
fn kind_name(kind: &RedstoneNodeKind) -> &'static str {
    match kind {
        RedstoneNodeKind::Repeater { .. } => "Repeater",
        RedstoneNodeKind::Comparator { .. } => "Comparator",
        RedstoneNodeKind::Torch => "Torch",
        RedstoneNodeKind::Lamp => "Lamp",
        RedstoneNodeKind::Button => "Button",
        RedstoneNodeKind::Lever => "Lever",
        RedstoneNodeKind::PressurePlate => "PressurePlate",
        RedstoneNodeKind::Trapdoor => "Trapdoor",
        RedstoneNodeKind::Wire => "Wire",
        RedstoneNodeKind::Constant => "Constant",
        RedstoneNodeKind::NoteBlock => "NoteBlock",
    }
}

/// Per-node weight used by `delay_weighted_depth`: a Repeater contributes its
/// configured `delay` (1..=4 redstone ticks); every other node contributes 1.
fn delay_weight(kind: &RedstoneNodeKind) -> u32 {
    match kind {
        RedstoneNodeKind::Repeater { delay } => u32::from(*delay),
        _ => 1,
    }
}

impl RedstoneGraph {
    /// Out-adjacency list (node id -> ids it feeds), inverted from `inputs`.
    fn out_adjacency(&self) -> Vec<Vec<usize>> {
        let n = self.nodes.len();
        let mut out: Vec<Vec<usize>> = vec![Vec::new(); n];
        for node in &self.nodes {
            for link in &node.inputs {
                // `link.from` feeds `node.id`.
                if link.from < n {
                    out[link.from].push(node.id);
                }
            }
        }
        out
    }

    /// Stable kind name -> count. One bucket per discriminant; kind-specific
    /// payload (repeater delay, comparator mode) is ignored.
    pub fn node_kind_counts(&self) -> BTreeMap<String, usize> {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for node in &self.nodes {
            *counts.entry(kind_name(&node.kind).to_string()).or_insert(0) += 1;
        }
        counts
    }

    /// Strongly connected components via an **iterative** Tarjan's algorithm
    /// (explicit work stack — safe on very large/deep graphs). Each inner vec is
    /// the set of node ids in one SCC. Component order is unspecified but the
    /// decomposition is deterministic for a given graph.
    pub fn strongly_connected_components(&self) -> Vec<Vec<usize>> {
        let n = self.nodes.len();
        let out = self.out_adjacency();
        self.tarjan_scc(&out, n)
    }

    /// Iterative Tarjan SCC over a precomputed out-adjacency.
    fn tarjan_scc(&self, out: &[Vec<usize>], n: usize) -> Vec<Vec<usize>> {
        const UNVISITED: usize = usize::MAX;
        let mut index = vec![UNVISITED; n];
        let mut lowlink = vec![0usize; n];
        let mut on_stack = vec![false; n];
        let mut stack: Vec<usize> = Vec::new();
        let mut next_index = 0usize;
        let mut sccs: Vec<Vec<usize>> = Vec::new();

        // Each work-stack frame tracks a node and how far through its successors
        // we've recursed (the position resumes the "for child" loop iteratively).
        let mut work: Vec<(usize, usize)> = Vec::new();

        for root in 0..n {
            if index[root] != UNVISITED {
                continue;
            }
            work.push((root, 0));

            while let Some(&(v, child_pos)) = work.last() {
                if child_pos == 0 {
                    // First time we touch `v`: assign its index/lowlink.
                    index[v] = next_index;
                    lowlink[v] = next_index;
                    next_index += 1;
                    stack.push(v);
                    on_stack[v] = true;
                }

                let mut recursed = false;
                let mut pos = child_pos;
                while pos < out[v].len() {
                    let w = out[v][pos];
                    if index[w] == UNVISITED {
                        // Descend into w; resume v's loop at pos+1 afterwards.
                        *work.last_mut().unwrap() = (v, pos + 1);
                        work.push((w, 0));
                        recursed = true;
                        break;
                    } else if on_stack[w] {
                        lowlink[v] = lowlink[v].min(index[w]);
                    }
                    pos += 1;
                }
                if recursed {
                    continue;
                }

                // Done with all of v's successors.
                if lowlink[v] == index[v] {
                    // v is an SCC root: pop until v.
                    let mut component = Vec::new();
                    loop {
                        let w = stack.pop().unwrap();
                        on_stack[w] = false;
                        component.push(w);
                        if w == v {
                            break;
                        }
                    }
                    sccs.push(component);
                }

                // Pop v; propagate its lowlink to its parent (if any).
                work.pop();
                if let Some(&(parent, _)) = work.last() {
                    lowlink[parent] = lowlink[parent].min(lowlink[v]);
                }
            }
        }

        sccs
    }

    /// True if the graph contains a directed cycle: any SCC of size > 1, OR any
    /// node that lists itself among its own inputs (a self-loop).
    pub fn has_cycles(&self) -> bool {
        // Self-loop check is cheap; do it first.
        for node in &self.nodes {
            if node.inputs.iter().any(|l| l.from == node.id) {
                return true;
            }
        }
        self.strongly_connected_components()
            .iter()
            .any(|scc| scc.len() > 1)
    }

    /// A graph is combinational iff it has no cycles.
    pub fn is_combinational(&self) -> bool {
        !self.has_cycles()
    }

    /// Number of weakly connected components: connected components when every
    /// directed edge is treated as undirected. Computed with union-find.
    pub fn weakly_connected_components(&self) -> usize {
        let n = self.nodes.len();
        if n == 0 {
            return 0;
        }
        let mut uf = UnionFind::new(n);
        for node in &self.nodes {
            for link in &node.inputs {
                if link.from < n {
                    uf.union(node.id, link.from);
                }
            }
        }
        let mut roots = std::collections::HashSet::new();
        for i in 0..n {
            roots.insert(uf.find(i));
        }
        roots.len()
    }

    /// Longest path through the graph measured in **node count**, computed over
    /// the condensation (the DAG of SCCs) so it terminates even on cyclic
    /// graphs.
    ///
    /// Exact definition: collapse each SCC to a single super-node whose weight is
    /// the number of original nodes in that SCC. `critical_path` is the maximum,
    /// over all paths in the condensation DAG, of the sum of super-node weights
    /// along the path. For an acyclic graph (every SCC has size 1) this is simply
    /// the number of nodes on the longest directed path. The value is 0 for an
    /// empty graph and 1 for a single isolated node.
    pub fn critical_path(&self) -> u32 {
        self.longest_weighted_path(WeightMode::NodeCount)
    }

    /// Same longest-path-over-condensation metric as [`Self::critical_path`], but
    /// each original node contributes a **delay weight** instead of 1: a Repeater
    /// contributes its `delay` (1..=4), every other node contributes 1. An SCC's
    /// super-node weight is the sum of its members' delay weights. This
    /// approximates redstone-tick propagation delay along the deepest chain.
    pub fn delay_weighted_depth(&self) -> u32 {
        self.longest_weighted_path(WeightMode::Delay)
    }

    /// Maximum incoming edge count over all nodes.
    pub fn max_fan_in(&self) -> usize {
        self.nodes.iter().map(|n| n.inputs.len()).max().unwrap_or(0)
    }

    /// Maximum outgoing edge count over all nodes.
    pub fn max_fan_out(&self) -> usize {
        self.out_adjacency()
            .iter()
            .map(|o| o.len())
            .max()
            .unwrap_or(0)
    }

    /// Aggregate feature vector. The SCC decomposition (and the out-adjacency it
    /// derives from) is computed once and reused for every cycle/depth metric.
    pub fn features(&self) -> GraphFeatures {
        let n = self.nodes.len();
        let out = self.out_adjacency();
        let sccs = self.tarjan_scc(&out, n);

        // Self-loop detection (Tarjan reports a singleton SCC for a self-looping
        // node, so check inputs explicitly).
        let has_self_loop = self
            .nodes
            .iter()
            .any(|node| node.inputs.iter().any(|l| l.from == node.id));
        let has_cycles = has_self_loop || sccs.iter().any(|scc| scc.len() > 1);

        let scc_count = sccs.len();
        let largest_scc = sccs.iter().map(|s| s.len()).max().unwrap_or(0);

        let critical_path = self.longest_path_with_sccs(&out, &sccs, WeightMode::NodeCount);
        let delay_weighted_depth = self.longest_path_with_sccs(&out, &sccs, WeightMode::Delay);

        let max_fan_in = self.max_fan_in();
        let max_fan_out = out.iter().map(|o| o.len()).max().unwrap_or(0);

        let mut approx_input_count = 0usize;
        let mut approx_output_count = 0usize;
        for node in &self.nodes {
            match node.kind {
                RedstoneNodeKind::Lever
                | RedstoneNodeKind::Button
                | RedstoneNodeKind::PressurePlate => approx_input_count += 1,
                RedstoneNodeKind::Lamp
                | RedstoneNodeKind::Trapdoor
                | RedstoneNodeKind::NoteBlock => approx_output_count += 1,
                _ => {}
            }
        }

        GraphFeatures {
            node_count: n,
            edge_count: self.edge_count(),
            node_kind_counts: self.node_kind_counts(),
            has_cycles,
            is_combinational: !has_cycles,
            critical_path,
            delay_weighted_depth,
            scc_count,
            largest_scc,
            weakly_connected_components: self.weakly_connected_components(),
            max_fan_in,
            max_fan_out,
            approx_input_count,
            approx_output_count,
        }
    }

    /// Convenience entry: derive SCCs then delegate to the shared longest-path
    /// routine.
    fn longest_weighted_path(&self, mode: WeightMode) -> u32 {
        let n = self.nodes.len();
        let out = self.out_adjacency();
        let sccs = self.tarjan_scc(&out, n);
        self.longest_path_with_sccs(&out, &sccs, mode)
    }

    /// Longest weighted path over the condensation DAG.
    ///
    /// Builds the SCC-component mapping, the per-component weight (sum of member
    /// node weights under `mode`), and the condensation's edges, then does a
    /// topological-order DP (the condensation is always acyclic).
    fn longest_path_with_sccs(
        &self,
        out: &[Vec<usize>],
        sccs: &[Vec<usize>],
        mode: WeightMode,
    ) -> u32 {
        let n = self.nodes.len();
        if n == 0 {
            return 0;
        }
        let c = sccs.len();

        // node id -> component index.
        let mut comp_of = vec![0usize; n];
        for (ci, scc) in sccs.iter().enumerate() {
            for &node_id in scc {
                comp_of[node_id] = ci;
            }
        }

        // Per-component weight under the requested mode.
        let mut weight = vec![0u32; c];
        for (ci, scc) in sccs.iter().enumerate() {
            let mut w = 0u32;
            for &node_id in scc {
                w = w.saturating_add(match mode {
                    WeightMode::NodeCount => 1,
                    WeightMode::Delay => delay_weight(&self.nodes[node_id].kind),
                });
            }
            weight[ci] = w;
        }

        // Condensation edges (dedup self-edges between same component).
        let mut cond_out: Vec<Vec<usize>> = vec![Vec::new(); c];
        let mut indeg = vec![0usize; c];
        // Track existing edges to avoid duplicate counting of indegree.
        let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
        for u in 0..n {
            let cu = comp_of[u];
            for &v in &out[u] {
                let cv = comp_of[v];
                if cu != cv && seen.insert((cu, cv)) {
                    cond_out[cu].push(cv);
                    indeg[cv] += 1;
                }
            }
        }

        // Kahn topological order over the condensation.
        let mut queue: Vec<usize> = (0..c).filter(|&i| indeg[i] == 0).collect();
        let mut topo: Vec<usize> = Vec::with_capacity(c);
        let mut indeg_mut = indeg.clone();
        let mut head = 0;
        while head < queue.len() {
            let u = queue[head];
            head += 1;
            topo.push(u);
            for &v in &cond_out[u] {
                indeg_mut[v] -= 1;
                if indeg_mut[v] == 0 {
                    queue.push(v);
                }
            }
        }

        // DP: best[u] = max path-weight ending at component u (inclusive).
        let mut best = weight.clone();
        for &u in &topo {
            let bu = best[u];
            for &v in &cond_out[u] {
                let cand = bu.saturating_add(weight[v]);
                if cand > best[v] {
                    best[v] = cand;
                }
            }
        }

        best.into_iter().max().unwrap_or(0)
    }
}

/// Weighting strategy for the longest-path kernels.
#[derive(Clone, Copy)]
enum WeightMode {
    /// Every node counts as 1 (path length in nodes).
    NodeCount,
    /// Repeaters count as their delay; everything else as 1.
    Delay,
}

/// Minimal union-find (disjoint set) with path compression + union by size.
struct UnionFind {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            size: vec![1; n],
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, a: usize, b: usize) {
        let (mut ra, mut rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        if self.size[ra] < self.size[rb] {
            std::mem::swap(&mut ra, &mut rb);
        }
        self.parent[rb] = ra;
        self.size[ra] += self.size[rb];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::graph::{
        ComparatorMode, LinkKind, RedstoneGraph, RedstoneLink, RedstoneNode, RedstoneNodeKind,
    };
    use crate::simulation::MchprsWorld;
    use crate::{BlockState, UniversalSchematic};

    // ---- helpers for hand-built graphs ----------------------------------

    fn node(id: usize, kind: RedstoneNodeKind, inputs: Vec<usize>) -> RedstoneNode {
        RedstoneNode {
            id,
            kind,
            pos: None,
            facing_diode: false,
            powered: false,
            repeater_locked: false,
            output_strength: 0,
            aliased_blocks: Vec::new(),
            inputs: inputs
                .into_iter()
                .map(|from| RedstoneLink {
                    from,
                    kind: LinkKind::Default,
                    strength: 0,
                })
                .collect(),
        }
    }

    /// Same known-good fixture used by Phase 2/3: lever -> wire -> lamp.
    fn create_simple_redstone_line() -> UniversalSchematic {
        let mut schematic = UniversalSchematic::new("Simple Redstone Line".to_string());

        for x in 0..16 {
            schematic.set_block(
                x,
                0,
                0,
                &BlockState::new("minecraft:gray_concrete".to_string()),
            );
        }
        for x in 1..15 {
            let mut wire = BlockState::new("minecraft:redstone_wire".to_string());
            wire.set_property("power", "0");
            wire.set_property("east", "side");
            wire.set_property("west", "side");
            wire.set_property("north", "none");
            wire.set_property("south", "none");
            schematic.set_block(x, 1, 0, &wire);
        }
        let mut lever = BlockState::new("minecraft:lever".to_string());
        lever.set_property("facing", "east");
        lever.set_property("powered", "false");
        lever.set_property("face", "floor");
        schematic.set_block(0, 1, 0, &lever);

        let mut lamp = BlockState::new("minecraft:redstone_lamp".to_string());
        lamp.set_property("lit", "false");
        schematic.set_block(15, 1, 0, &lamp);

        schematic
    }

    // ---- test 1: real lever -> lamp fixture -----------------------------

    #[test]
    fn test_real_fixture_combinational() {
        let schematic = create_simple_redstone_line();
        let world = MchprsWorld::new(schematic).expect("world creation should succeed");
        let graph = world
            .export_graph()
            .expect("graph extraction should succeed");

        assert!(
            graph.is_combinational(),
            "lever->lamp should be combinational"
        );
        assert!(!graph.has_cycles(), "lever->lamp should have no cycles");
        assert_eq!(
            graph.weakly_connected_components(),
            1,
            "the whole line is one weak component"
        );

        let counts = graph.node_kind_counts();
        assert!(counts.contains_key("Lever"), "kind counts include Lever");
        assert!(counts.contains_key("Lamp"), "kind counts include Lamp");

        assert!(
            graph.critical_path() >= 2,
            "critical path should span at least lever..lamp, got {}",
            graph.critical_path()
        );
    }

    // ---- test 2: 2-node cycle -------------------------------------------

    #[test]
    fn test_two_node_cycle() {
        // A(0).inputs = [B(1)], B(1).inputs = [A(0)]
        let graph = RedstoneGraph {
            nodes: vec![
                node(0, RedstoneNodeKind::Torch, vec![1]),
                node(1, RedstoneNodeKind::Torch, vec![0]),
            ],
        };

        assert!(graph.has_cycles());
        assert!(!graph.is_combinational());

        let sccs = graph.strongly_connected_components();
        let big: Vec<_> = sccs.iter().filter(|s| s.len() == 2).collect();
        assert_eq!(big.len(), 1, "exactly one SCC of size 2");
        assert_eq!(sccs.len(), 1, "two mutually-cyclic nodes form one SCC");

        let f = graph.features();
        assert_eq!(f.scc_count, 1);
        assert_eq!(f.largest_scc, 2);
        assert!(f.has_cycles);
    }

    // ---- test 3: self-loop ----------------------------------------------

    #[test]
    fn test_self_loop() {
        let graph = RedstoneGraph {
            nodes: vec![node(0, RedstoneNodeKind::Torch, vec![0])],
        };
        assert!(graph.has_cycles(), "a self-loop is a cycle");
        assert!(graph.features().has_cycles);
        assert!(!graph.is_combinational());
    }

    // ---- test 4: fan-in / fan-out ---------------------------------------

    #[test]
    fn test_fan_in() {
        // sink (3) <- 0,1,2
        let graph = RedstoneGraph {
            nodes: vec![
                node(0, RedstoneNodeKind::Lever, vec![]),
                node(1, RedstoneNodeKind::Lever, vec![]),
                node(2, RedstoneNodeKind::Lever, vec![]),
                node(3, RedstoneNodeKind::Lamp, vec![0, 1, 2]),
            ],
        };
        assert_eq!(graph.max_fan_in(), 3);
    }

    #[test]
    fn test_fan_out() {
        // source (0) -> 1,2,3
        let graph = RedstoneGraph {
            nodes: vec![
                node(0, RedstoneNodeKind::Lever, vec![]),
                node(1, RedstoneNodeKind::Lamp, vec![0]),
                node(2, RedstoneNodeKind::Lamp, vec![0]),
                node(3, RedstoneNodeKind::Lamp, vec![0]),
            ],
        };
        assert_eq!(graph.max_fan_out(), 3);
    }

    // ---- test 5: two disconnected pairs ---------------------------------

    #[test]
    fn test_weakly_connected_components_two_pairs() {
        // pair A: 0->1, pair B: 2->3
        let graph = RedstoneGraph {
            nodes: vec![
                node(0, RedstoneNodeKind::Lever, vec![]),
                node(1, RedstoneNodeKind::Lamp, vec![0]),
                node(2, RedstoneNodeKind::Lever, vec![]),
                node(3, RedstoneNodeKind::Lamp, vec![2]),
            ],
        };
        assert_eq!(graph.weakly_connected_components(), 2);
    }

    // ---- test 6: delay-weighted depth -----------------------------------

    #[test]
    fn test_delay_weighted_depth() {
        // chain: Lever(0) -> Repeater{delay:4}(1) -> Lamp(2)
        // node-count path = 3 nodes.
        // delay-weighted = 1 (lever) + 4 (repeater) + 1 (lamp) = 6.
        let graph = RedstoneGraph {
            nodes: vec![
                node(0, RedstoneNodeKind::Lever, vec![]),
                node(1, RedstoneNodeKind::Repeater { delay: 4 }, vec![0]),
                node(2, RedstoneNodeKind::Lamp, vec![1]),
            ],
        };
        assert_eq!(graph.critical_path(), 3, "3 nodes on the chain");
        assert_eq!(
            graph.delay_weighted_depth(),
            6,
            "1 + 4 (repeater delay) + 1 along the chain"
        );
    }

    // ---- test 7: features JSON round-trip --------------------------------

    #[test]
    fn test_features_json_round_trip() {
        let graph = RedstoneGraph {
            nodes: vec![
                node(0, RedstoneNodeKind::Lever, vec![]),
                node(
                    1,
                    RedstoneNodeKind::Comparator {
                        mode: ComparatorMode::Compare,
                        far_input: None,
                    },
                    vec![0],
                ),
                node(2, RedstoneNodeKind::Lamp, vec![1]),
            ],
        };
        let f = graph.features();
        let json = f.to_json().expect("serialize");
        let back = GraphFeatures::from_json(&json).expect("deserialize");
        assert_eq!(f, back, "GraphFeatures JSON round-trips");
    }
}
