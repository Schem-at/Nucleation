//! Graph-based (diagonal-capable) period detection.
//!
//! The pure-voxel detector in the parent module only proposes axis-aligned
//! periods. Diagonal datapaths (e.g. a diagonal carry-chain adder) repeat along
//! a non-axis vector that voxel self-overlap can't easily find. This module
//! recovers that vector from the redstone **logic graph** — the same approach
//! the Python prototype used:
//!
//! 1. `wl_labels`     — a 3-round Weisfeiler-Lehman structural label per node.
//! 2. `dominant_dir`  — the most common displacement between same-label nodes,
//!                      gcd-reduced to a primitive lattice direction (diagonal-
//!                      friendly: `(2,2,0) -> (1,1,0)`).
//! 3. `true_period`   — the smallest multiple of that direction whose phase-
//!                      bucket label-histograms are translation-invariant.
//!
//! The resulting vector is fed through the parent module's pure-voxel lattice
//! machinery (`region_set`/`best_phase`/`lattice_slabs`), so resizing is
//! identical to the axis case — only detection differs.
//!
//! Requires the `simulation` feature (graph extraction runs the redpiler).

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::{bbox_of, best_phase, build_grid, grid_origin, lattice_slabs, proj, region_set};
use super::{Grid, Structure, Vec3};
use crate::simulation::graph::{RedstoneGraph, RedstoneNode, RedstoneNodeKind};
use crate::simulation::MchprsWorld;
use crate::UniversalSchematic;

const WL_ROUNDS: usize = 3;
const MAX_MULT: i32 = 12;
const MIN_RUN: usize = 2;
const MIN_NODES: usize = 4;

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

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// 3-round direction-aware Weisfeiler-Lehman labels keyed by node id.
fn wl_labels(graph: &RedstoneGraph) -> HashMap<usize, u64> {
    let mut out_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut in_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for node in &graph.nodes {
        for link in &node.inputs {
            // edge: link.from -> node.id
            out_adj.entry(link.from).or_default().push(node.id);
            in_adj.entry(node.id).or_default().push(link.from);
        }
    }

    let mut lab: HashMap<usize, u64> = graph
        .nodes
        .iter()
        .map(|n| (n.id, hash_str(kind_name(&n.kind))))
        .collect();

    for _ in 0..WL_ROUNDS {
        let mut next: HashMap<usize, u64> = HashMap::with_capacity(lab.len());
        for node in &graph.nodes {
            let mut outs: Vec<u64> = out_adj
                .get(&node.id)
                .map(|ts| ts.iter().filter_map(|t| lab.get(t).copied()).collect())
                .unwrap_or_default();
            let mut ins: Vec<u64> = in_adj
                .get(&node.id)
                .map(|ts| ts.iter().filter_map(|t| lab.get(t).copied()).collect())
                .unwrap_or_default();
            outs.sort_unstable();
            ins.sort_unstable();

            let mut h = DefaultHasher::new();
            lab[&node.id].hash(&mut h);
            b'O'.hash(&mut h);
            outs.hash(&mut h);
            b'I'.hash(&mut h);
            ins.hash(&mut h);
            next.insert(node.id, h.finish());
        }
        lab = next;
    }
    lab
}

/// Sign-canonicalize a displacement so `+v` and `-v` collapse to one key.
fn canon(d: Vec3) -> Vec3 {
    for c in d {
        if c > 0 {
            return d;
        }
        if c < 0 {
            return [-d[0], -d[1], -d[2]];
        }
    }
    d
}

fn gcd(a: i32, b: i32) -> i32 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Primitive lattice direction: gcd-reduce the most common same-label
/// displacement. Returns `None` if no two same-label nodes are placed.
fn dominant_dir(graph: &RedstoneGraph, lab: &HashMap<usize, u64>) -> Option<Vec3> {
    let mut by_lab: HashMap<u64, Vec<Vec3>> = HashMap::new();
    for n in &graph.nodes {
        if let Some((x, y, z)) = n.pos {
            by_lab.entry(lab[&n.id]).or_default().push([x, y, z]);
        }
    }
    let mut disp: HashMap<Vec3, usize> = HashMap::new();
    for ps in by_lab.values() {
        for i in 0..ps.len() {
            for j in (i + 1)..ps.len() {
                let d = [
                    ps[j][0] - ps[i][0],
                    ps[j][1] - ps[i][1],
                    ps[j][2] - ps[i][2],
                ];
                if d != [0, 0, 0] {
                    *disp.entry(canon(d)).or_default() += 1;
                }
            }
        }
    }
    // Most common displacement; deterministic tie-break: higher count, then
    // shorter vector, then lexicographic.
    let v = disp.into_iter().max_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| super::norm2(b.0).cmp(&super::norm2(a.0)))
            .then_with(|| a.0.cmp(&b.0))
    })?;
    let v = v.0;
    let g = gcd(gcd(v[0], v[1]), v[2]).max(1);
    Some([v[0] / g, v[1] / g, v[2] / g])
}

/// Phase-bucket nodes along `period` (floor division → contiguous domains).
fn phase_buckets<'a>(
    nodes: &[&'a RedstoneNode],
    period: Vec3,
    origin: Vec3,
) -> HashMap<i64, Vec<&'a RedstoneNode>> {
    let den = super::norm2(period).max(1);
    let mut cells: HashMap<i64, Vec<&RedstoneNode>> = HashMap::new();
    for n in nodes {
        if let Some(p) = n.pos {
            let k = proj(p, period, origin).div_euclid(den);
            cells.entry(k).or_default().push(n);
        }
    }
    cells
}

fn label_hist(cell: &[&RedstoneNode], lab: &HashMap<usize, u64>) -> Vec<(u64, usize)> {
    let mut counts: HashMap<u64, usize> = HashMap::new();
    for n in cell {
        *counts.entry(lab[&n.id]).or_default() += 1;
    }
    let mut v: Vec<(u64, usize)> = counts.into_iter().collect();
    v.sort_unstable();
    v
}

/// Smallest `m*direction` whose interior phase-buckets are translation-invariant
/// (>= MIN_RUN consecutive non-empty buckets with identical label-histograms).
fn true_period(
    nodes: &[&RedstoneNode],
    lab: &HashMap<usize, u64>,
    direction: Vec3,
    origin: Vec3,
) -> Option<Vec3> {
    for m in 1..=MAX_MULT {
        let period = [direction[0] * m, direction[1] * m, direction[2] * m];
        if period == [0, 0, 0] {
            continue;
        }
        let cells = phase_buckets(nodes, period, origin);
        let mut ks: Vec<i64> = cells.keys().copied().collect();
        ks.sort_unstable();
        let hists: Vec<Vec<(u64, usize)>> = ks.iter().map(|k| label_hist(&cells[k], lab)).collect();
        let mut run = 1usize;
        let mut best = 1usize;
        for i in 1..hists.len() {
            if hists[i] == hists[i - 1] && !cells[&ks[i]].is_empty() {
                run += 1;
                best = best.max(run);
            } else {
                run = 1;
            }
        }
        if best >= MIN_RUN {
            return Some(period);
        }
    }
    None
}

fn node_origin(nodes: &[&RedstoneNode]) -> Vec3 {
    let mut o = [i32::MAX; 3];
    for n in nodes {
        if let Some((x, y, z)) = n.pos {
            o[0] = o[0].min(x);
            o[1] = o[1].min(y);
            o[2] = o[2].min(z);
        }
    }
    if o[0] == i32::MAX {
        [0, 0, 0]
    } else {
        o
    }
}

/// Bounding box of one representative unit cell for `v` (the first cell of the
/// longest identical run), in world coordinates.
fn cell_bbox(grid: &Grid, v: Vec3) -> (Vec3, Vec3) {
    let o = grid_origin(grid);
    let (phase, ks, run_start, _) = best_phase(grid, v, o);
    let byk = lattice_slabs(grid, v, o, phase);
    if let Some(cell) = byk.get(&ks[run_start]) {
        let k = ks[run_start] as i32;
        let mut mn = [i32::MAX; 3];
        let mut mx = [i32::MIN; 3];
        for loc in cell.keys() {
            let p = [loc[0] + k * v[0], loc[1] + k * v[1], loc[2] + k * v[2]];
            for i in 0..3 {
                mn[i] = mn[i].min(p[i]);
                mx[i] = mx[i].max(p[i]);
            }
        }
        if mn[0] != i32::MAX {
            return (mn, mx);
        }
    }
    (o, o)
}

/// Build a [`Structure`] for a graph-detected period vector, using the voxel
/// grid for coverage/region.
fn structure_for(s: &UniversalSchematic, period: Vec3) -> Option<Structure> {
    let grid = build_grid(s);
    if grid.is_empty() {
        return None;
    }
    let region = region_set(&grid, period);
    if region.is_empty() {
        return None;
    }
    let coverage = region.len() as f32 / grid.len().max(1) as f32;
    let (rmn, rmx) = bbox_of(&region);
    let (cmn, cmx) = cell_bbox(&grid, period);
    let diagonal = period.iter().filter(|&&c| c != 0).count() > 1;
    let dir = if diagonal {
        format!("diagonal {:?}", (period[0], period[1], period[2]))
    } else {
        let ax = (0..3).find(|&i| period[i] != 0).unwrap_or(0);
        ["X", "Y", "Z"][ax].to_string()
    };
    Some(Structure {
        mode: "1d".to_string(),
        vectors: vec![period],
        coverage,
        region_min: rmn,
        region_max: rmx,
        cell_min: cmn,
        cell_max: cmx,
        label: format!(
            "1D run · {} · {}% of build (graph)",
            dir,
            (coverage * 100.0) as i32
        ),
    })
}

/// Detect repeating structure(s) from the redstone logic graph — recovers
/// diagonal lattices the pure-voxel detector misses. Returns an empty vec if the
/// build isn't redstone-computational or has no detectable graph period.
pub fn detect_structures_graph(s: &UniversalSchematic) -> Vec<Structure> {
    let world = match MchprsWorld::new(s.clone()) {
        Ok(w) => w,
        Err(_) => return Vec::new(),
    };
    let graph = match world.export_graph_structural() {
        Ok(g) => g,
        Err(_) => return Vec::new(),
    };
    if graph.nodes.len() < MIN_NODES {
        return Vec::new();
    }

    let lab = wl_labels(&graph);
    let dir = match dominant_dir(&graph, &lab) {
        Some(d) => d,
        None => return Vec::new(),
    };
    let nodes: Vec<&RedstoneNode> = graph.nodes.iter().filter(|n| n.pos.is_some()).collect();
    if nodes.len() < MIN_NODES {
        return Vec::new();
    }
    let origin = node_origin(&nodes);
    let period = match true_period(&nodes, &lab, dir, origin) {
        Some(p) => p,
        None => return Vec::new(),
    };
    structure_for(s, period).into_iter().collect()
}

/// JSON array of graph-detected structures (binding-friendly entry point).
pub fn detect_structures_graph_json(s: &UniversalSchematic) -> String {
    serde_json::to_string(&detect_structures_graph(s)).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockState;

    /// A long lever → redstone-wire → lamp line: a clean 1-D periodic datapath
    /// whose graph period the detector should recover without panicking.
    fn redstone_line(len: i32) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("line".to_string());
        for x in 0..len {
            s.set_block(
                x,
                0,
                0,
                &BlockState::new("minecraft:gray_concrete".to_string()),
            );
        }
        let mut lever = BlockState::new("minecraft:lever".to_string());
        lever.set_property("facing", "east");
        lever.set_property("powered", "false");
        lever.set_property("face", "floor");
        s.set_block(0, 1, 0, &lever);
        for x in 1..(len - 1) {
            let mut w = BlockState::new("minecraft:redstone_wire".to_string());
            w.set_property("power", "0");
            w.set_property("east", "side");
            w.set_property("west", "side");
            s.set_block(x, 1, 0, &w);
        }
        let mut lamp = BlockState::new("minecraft:redstone_lamp".to_string());
        lamp.set_property("lit", "false");
        s.set_block(len - 1, 1, 0, &lamp);
        s
    }

    #[test]
    fn graph_detect_runs_and_is_axis_consistent() {
        let s = redstone_line(16);
        // Must not panic; JSON entry point must produce valid JSON.
        let structs = detect_structures_graph(&s);
        let json = detect_structures_graph_json(&s);
        assert!(json.starts_with('['), "json array");

        // If a period is found, its vector is non-zero and the structure is 1d.
        for st in &structs {
            assert_eq!(st.mode, "1d");
            let v = st.vectors[0];
            assert_ne!(v, [0, 0, 0], "period vector must be non-zero");
            assert!(st.coverage > 0.0 && st.coverage <= 1.0);
        }
    }

    #[test]
    fn non_redstone_build_yields_no_graph_structures() {
        // A plain stone cube has (essentially) no redstone graph.
        let mut s = UniversalSchematic::new("cube".to_string());
        for x in 0..4 {
            for y in 0..4 {
                for z in 0..4 {
                    s.set_block(x, y, z, &BlockState::new("minecraft:stone".to_string()));
                }
            }
        }
        assert!(detect_structures_graph(&s).is_empty());
    }
}
