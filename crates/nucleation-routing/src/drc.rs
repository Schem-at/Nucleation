//! Design-rule checking: shorts, support, decay, and directed cycles
//! through repeaters.
//!
//! The cycle check is the NEW tool the rca_cells session was missing: two
//! cells' port approaches formed opposite-facing repeaters in a ring on an
//! aliased net — no short, passed every static check, settled quiescent,
//! and latched the placement transient. A directed cycle through repeaters
//! on a net is a storage element; unless you built a latch on purpose, it
//! is a bug.

use crate::audit;
use crate::blocks::{self, is_comparator, is_dust, is_repeater};
use crate::nets;
use crate::workspace::Workspace;
use pnr_core::unionfind::UnionFind;
use pnr_core::Pos;
use std::collections::{BTreeMap, BTreeSet};

/// A DRC violation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Violation {
    /// Two distinct labels share an electrical net.
    Short {
        /// First label.
        label_a: String,
        /// Second label.
        label_b: String,
        /// Witness cell for the first label.
        at_a: Pos,
        /// Witness cell for the second label.
        at_b: Pos,
    },
    /// A block needing a floor has none.
    Floating {
        /// Where.
        at: Pos,
        /// Block name (without properties).
        block: String,
    },
    /// A wall torch's anchor block is missing or not solid.
    UnattachedWallTorch {
        /// The torch.
        at: Pos,
        /// The missing anchor cell.
        anchor: Pos,
    },
    /// A directed cycle through repeaters/comparators — an accidental
    /// latch. Lists the diodes on the cycle.
    RepeaterCycle {
        /// Repeater/comparator positions forming the ring.
        diodes: Vec<Pos>,
    },
    /// Dust farther from every driver than a full signal reaches.
    PowerStarved {
        /// The starved dust cell.
        at: Pos,
        /// Its distance from the nearest driver.
        distance: u32,
    },
}

/// DRC options.
#[derive(Clone, Debug, Default)]
pub struct DrcOptions {
    /// Label pairs that are deliberately the same electrical net.
    pub aliases: Vec<(String, String)>,
    /// Skip the decay (PowerStarved) check.
    pub skip_decay: bool,
}

/// Run all checks over a workspace.
pub fn drc(ws: &Workspace, opts: &DrcOptions) -> Vec<Violation> {
    let mut out = Vec::new();
    for s in nets::check(ws.cells(), ws.labels(), &opts.aliases) {
        out.push(Violation::Short {
            label_a: s.label_a,
            label_b: s.label_b,
            at_a: s.at_a,
            at_b: s.at_b,
        });
    }
    let a = audit::audit(ws.cells());
    for (at, block, _below) in a.floating {
        out.push(Violation::Floating { at, block });
    }
    for (at, _face, anchor) in a.unattached_wall_torch {
        out.push(Violation::UnattachedWallTorch { at, anchor });
    }
    out.extend(repeater_cycles(ws.cells()));
    if !opts.skip_decay {
        out.extend(decay_check(ws.cells(), 15));
    }
    out
}

/// The input and output cells of a repeater/comparator: `facing` names the
/// INPUT side (verified: `repeater[facing=west]` conducts toward +X).
fn diode_io(p: Pos, block: &str) -> Option<(Pos, Pos)> {
    let f = blocks::facing_of(block)?;
    let (dx, dy, dz) = blocks::facing_vec(f)?;
    Some((p.offset(dx, dy, dz), p.offset(-dx, -dy, -dz)))
}

/// Detect directed cycles through repeaters/comparators over the dust-net
/// graph: dust components are nodes, each diode is a directed edge from the
/// component at its input to the component at its output.
pub fn repeater_cycles(cells: &BTreeMap<Pos, String>) -> Vec<Violation> {
    // Dust components.
    let mut uf: UnionFind<Pos> = UnionFind::new();
    let dust: Vec<Pos> = cells
        .iter()
        .filter(|(_, b)| is_dust(b))
        .map(|(p, _)| *p)
        .collect();
    for p in &dust {
        uf.find(p);
        for q in nets::neighbours(cells, *p) {
            uf.union(p, &q);
        }
    }
    // Directed edges: component -> component, tagged with the diode.
    let mut edges: BTreeMap<Pos, Vec<(Pos, Pos)>> = BTreeMap::new(); // from-comp -> [(to-comp, diode)]
    for (p, b) in cells {
        if !(is_repeater(b) || is_comparator(b)) {
            continue;
        }
        let Some((inp, outp)) = diode_io(*p, b) else {
            continue;
        };
        let in_dust = cells.get(&inp).is_some_and(|b| is_dust(b));
        let out_dust = cells.get(&outp).is_some_and(|b| is_dust(b));
        if in_dust && out_dust {
            let a = uf.find(&inp);
            let bcomp = uf.find(&outp);
            edges.entry(a).or_default().push((bcomp, *p));
        }
    }
    // DFS cycle detection over component nodes (deterministic order).
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Grey,
        Black,
    }
    let nodes: BTreeSet<Pos> = edges
        .iter()
        .flat_map(|(from, tos)| std::iter::once(*from).chain(tos.iter().map(|(t, _)| *t)))
        .collect();
    let mut color: BTreeMap<Pos, Color> = nodes.iter().map(|n| (*n, Color::White)).collect();
    let mut found: Vec<Violation> = Vec::new();
    let mut reported: BTreeSet<Vec<Pos>> = BTreeSet::new();

    fn dfs(
        node: Pos,
        edges: &BTreeMap<Pos, Vec<(Pos, Pos)>>,
        color: &mut BTreeMap<Pos, Color>,
        stack: &mut Vec<(Pos, Pos)>, // (component, diode entered through)
        found: &mut Vec<Violation>,
        reported: &mut BTreeSet<Vec<Pos>>,
    ) {
        color.insert(node, Color::Grey);
        if let Some(succs) = edges.get(&node) {
            for (to, diode) in succs {
                match color.get(to).copied().unwrap_or(Color::White) {
                    Color::Grey => {
                        // Cycle: the diodes are the edges taken from `to`
                        // onward (skip `to`'s own entry edge — it enters
                        // the cycle from outside) plus the closing edge.
                        let mut diodes: Vec<Pos> = stack
                            .iter()
                            .skip_while(|(c, _)| c != to)
                            .skip(1)
                            .map(|(_, d)| *d)
                            .collect();
                        diodes.push(*diode);
                        diodes.sort_unstable();
                        if reported.insert(diodes.clone()) {
                            found.push(Violation::RepeaterCycle { diodes });
                        }
                    }
                    Color::White => {
                        stack.push((*to, *diode));
                        dfs(*to, edges, color, stack, found, reported);
                        stack.pop();
                    }
                    Color::Black => {}
                }
            }
        }
        color.insert(node, Color::Black);
    }

    for n in &nodes {
        if color[n] == Color::White {
            let mut stack = vec![(*n, Pos::new(0, 0, 0))];
            // The root's entry diode is a placeholder; cycles never include
            // it because cycle extraction starts at the revisited node.
            dfs(*n, &edges, &mut color, &mut stack, &mut found, &mut reported);
        }
    }
    found
}

/// Decay check: BFS through dust adjacency from every driver-adjacent dust
/// cell; dust farther than `full` (15) from all drivers cannot be powered
/// at strength. Components with no driver at all are skipped — a dead net
/// is an *open*, which static checks cannot see (only sim-vs-model or LVS
/// catches those; that is exactly why they are on the roadmap).
pub fn decay_check(cells: &BTreeMap<Pos, String>, full: u32) -> Vec<Violation> {
    // Driver dust: adjacent to a diode output, torch, lever, or sitting on
    // a strongly-powered ladder cap (block directly above a torch).
    let mut drivers: BTreeSet<Pos> = BTreeSet::new();
    for (p, b) in cells {
        if is_repeater(b) || is_comparator(b) {
            if let Some((_inp, outp)) = diode_io(*p, b) {
                if cells.get(&outp).is_some_and(|b| is_dust(b)) {
                    drivers.insert(outp);
                }
            }
        } else if blocks::is_torch(b) || blocks::is_lever(b) {
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let q = p.offset(dx, 0, dz);
                if cells.get(&q).is_some_and(|b| is_dust(b)) {
                    drivers.insert(q);
                }
            }
            // Dust directly above (a torch powers the dust on top of it —
            // and a torch under a cap strongly powers the cap, whose top
            // dust is the ladder exit).
            for dy in [1, 2] {
                let q = p.offset(0, dy, 0);
                if cells.get(&q).is_some_and(|b| is_dust(b)) {
                    drivers.insert(q);
                }
            }
        }
    }
    // Multi-source BFS.
    let mut dist: BTreeMap<Pos, u32> = drivers.iter().map(|p| (*p, 0)).collect();
    let mut frontier: Vec<Pos> = drivers.iter().copied().collect();
    while !frontier.is_empty() {
        let mut next = Vec::new();
        for p in frontier {
            let d = dist[&p];
            for q in nets::neighbours(cells, p) {
                if !dist.contains_key(&q) {
                    dist.insert(q, d + 1);
                    next.push(q);
                }
            }
        }
        frontier = next;
    }
    let mut out = Vec::new();
    for (p, b) in cells {
        if !is_dust(b) {
            continue;
        }
        match dist.get(p) {
            Some(d) if *d > full => out.push(Violation::PowerStarved {
                at: *p,
                distance: *d,
            }),
            Some(_) => {}
            None => {} // no driver in the component: an open, not decay
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{repeater, DUST, LEVER_OFF, STONE};

    fn dust_line(ws: &mut Workspace, x0: i32, x1: i32, z: i32, label: &str) {
        for x in x0..=x1 {
            ws.dust(Pos::new(x, 1, z), label).unwrap();
        }
    }

    #[test]
    fn repeater_ring_latch_is_detected() {
        // The cout/cin ring, minimally: two dust runs joined by two
        // opposite-facing repeaters — aliased into one net, invisible to
        // the short checker, and it latches (bug provenance: rca_cells
        // 18/32). Repeater facing names its INPUT side.
        let mut ws = Workspace::new();
        dust_line(&mut ws, 0, 2, 0, "cout");
        dust_line(&mut ws, 0, 2, 2, "cout");
        // rep A: input z0 run (north side), output z2 run.
        ws.stone(Pos::new(0, 0, 1), "plain").unwrap();
        ws.put(Pos::new(0, 1, 1), &repeater("north", 1)).unwrap();
        // rep B: input z2 run (south side), output z0 run.
        ws.stone(Pos::new(2, 0, 1), "plain").unwrap();
        ws.put(Pos::new(2, 1, 1), &repeater("south", 1)).unwrap();
        let cycles = repeater_cycles(ws.cells());
        assert_eq!(cycles.len(), 1);
        match &cycles[0] {
            Violation::RepeaterCycle { diodes } => {
                assert_eq!(diodes, &vec![Pos::new(0, 1, 1), Pos::new(2, 1, 1)]);
            }
            v => panic!("unexpected: {v:?}"),
        }
    }

    #[test]
    fn forward_chain_is_not_a_cycle() {
        // dust -> rep -> dust -> rep -> dust, all one direction: clean.
        let mut ws = Workspace::new();
        dust_line(&mut ws, 0, 1, 0, "a");
        ws.stone(Pos::new(2, 0, 0), "plain").unwrap();
        ws.put(Pos::new(2, 1, 0), &repeater("west", 1)).unwrap();
        dust_line(&mut ws, 3, 4, 0, "a");
        ws.stone(Pos::new(5, 0, 0), "plain").unwrap();
        ws.put(Pos::new(5, 1, 0), &repeater("west", 1)).unwrap();
        dust_line(&mut ws, 6, 7, 0, "a");
        assert!(repeater_cycles(ws.cells()).is_empty());
    }

    #[test]
    fn self_loop_through_one_repeater_is_a_cycle() {
        // A repeater feeding its own input component — the unclocked
        // storage element (a deliberate latch would use exactly this).
        let mut ws = Workspace::new();
        // Ring of dust around (1,1,1)..(3,1,3) minus center, with a
        // repeater bridging a cut in the ring.
        for (x, z) in [(1, 1), (2, 1), (3, 1), (3, 2), (3, 3), (2, 3), (1, 3)] {
            ws.dust(Pos::new(x, 1, z), "loop").unwrap();
        }
        // Repeater at (1,1,2): input the (1,1,3) side (south), output (1,1,1).
        ws.stone(Pos::new(1, 0, 2), "plain").unwrap();
        ws.put(Pos::new(1, 1, 2), &repeater("south", 1)).unwrap();
        let cycles = repeater_cycles(ws.cells());
        assert_eq!(cycles.len(), 1);
    }

    #[test]
    fn decay_flags_a_17_cell_unrepeated_run() {
        let mut ws = Workspace::new();
        // Lever drives x=0; dust runs x=1..=17 (17 cells): the far cells
        // sit beyond strength 15.
        ws.stone(Pos::new(0, 0, 0), "plain").unwrap();
        ws.put(Pos::new(0, 1, 0), LEVER_OFF).unwrap();
        dust_line(&mut ws, 1, 17, 0, "n");
        let v = decay_check(ws.cells(), 15);
        assert_eq!(v.len(), 1);
        assert!(matches!(
            v[0],
            Violation::PowerStarved {
                at: Pos { x: 17, .. },
                distance: 16
            }
        ));
        // A repeater mid-run fixes it.
        let mut ws2 = Workspace::new();
        ws2.stone(Pos::new(0, 0, 0), "plain").unwrap();
        ws2.put(Pos::new(0, 1, 0), LEVER_OFF).unwrap();
        dust_line(&mut ws2, 1, 8, 0, "n");
        ws2.stone(Pos::new(9, 0, 0), "plain").unwrap();
        ws2.put(Pos::new(9, 1, 0), &repeater("west", 1)).unwrap();
        dust_line(&mut ws2, 10, 17, 0, "n");
        assert!(decay_check(ws2.cells(), 15).is_empty());
    }

    #[test]
    fn drc_aggregates_all_checks() {
        let mut ws = Workspace::new();
        // A floating dust cell labelled twice-over to also short.
        ws.put(Pos::new(0, 1, 0), DUST).unwrap(); // floating
        ws.set_label(Pos::new(0, 1, 0), "a");
        ws.stone(Pos::new(1, 0, 0), "plain").unwrap();
        ws.put(Pos::new(1, 1, 0), DUST).unwrap();
        ws.set_label(Pos::new(1, 1, 0), "b"); // shorts with a
        let vs = drc(&ws, &DrcOptions::default());
        assert!(vs.iter().any(|v| matches!(v, Violation::Short { .. })));
        assert!(vs.iter().any(|v| matches!(v, Violation::Floating { .. })));
        // Stone is not a NEEDS_FLOOR block.
        assert!(!vs
            .iter()
            .any(|v| matches!(v, Violation::Floating { at, .. } if *at == Pos::new(1, 0, 0))));
        let _ = STONE;
    }
}
