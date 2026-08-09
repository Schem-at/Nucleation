//! LVS v1: layout-versus-schematic netlist comparison.
//!
//! The intent side is a list of named nets, each a set of terminal
//! positions that must be one electrical signal. The extracted side is a
//! static conduction graph over the block states (the design doc's
//! conduction model): dust adjacency including cut diagonals
//! ([`crate::nets`]), PLUS through-component edges — repeater/comparator
//! in→out, torch anchor→torch→outputs — because a routed net legitimately
//! passes through refresh repeaters and ladder torches while remaining ONE
//! intent net.
//!
//! The comparison closes the two blind spots simulation-plus-shorts
//! checking has:
//!
//! - **opens**: an intent net whose terminals land in different conduction
//!   components (a dead route settles quiescent and reads as "fine").
//! - **shorts**: one conduction component carrying terminals of two intent
//!   nets (adjacent lanes touching).
//! - **cycles**: directed repeater/comparator rings ([`crate::drc::
//!   repeater_cycles`]) — the cout/cin ring latch passed every other check.
//!
//! Deliberate conservatism, documented: through-component edges track
//! signal *flow*, not equipotential — an inverter's input and output merge
//! into one conduction component. LVS therefore compares routed wiring and
//! cell ports; gate internals belong inside cell keepouts, not in the
//! intent netlist.

use crate::blocks::{self, is_comparator, is_dust, is_repeater, is_solid_block, is_torch};
use crate::drc::{repeater_cycles, Violation};
use pnr_core::unionfind::UnionFind;
use pnr_core::Pos;
use std::collections::BTreeMap;

/// One intended net: a name and the terminals that must be connected.
#[derive(Clone, Debug)]
pub struct IntentNet {
    /// Net name.
    pub name: String,
    /// Terminal positions (dust cells, ports, torch anchors...).
    pub terminals: Vec<Pos>,
}

/// An intent net whose terminals are NOT all in one conduction component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LvsOpen {
    /// The broken net.
    pub net: String,
    /// Terminal groups by conduction component; two or more groups = open.
    pub fragments: Vec<Vec<Pos>>,
}

/// Two intent nets sharing one conduction component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LvsShort {
    /// First net.
    pub net_a: String,
    /// Second net.
    pub net_b: String,
    /// Witness terminal of the first net.
    pub at_a: Pos,
    /// Witness terminal of the second net.
    pub at_b: Pos,
}

/// The LVS verdict.
#[derive(Clone, Debug, Default)]
pub struct LvsReport {
    /// Nets whose terminals are connected and not merged with another net.
    pub matched: Vec<String>,
    /// Intent connected, extracted not.
    pub opens: Vec<LvsOpen>,
    /// Extracted merges two intents.
    pub shorts: Vec<LvsShort>,
    /// Directed repeater/comparator rings (accidental latches), as the
    /// diode positions on each ring.
    pub cycles: Vec<Vec<Pos>>,
}

impl LvsReport {
    /// No opens, no shorts, no rings.
    pub fn clean(&self) -> bool {
        self.opens.is_empty() && self.shorts.is_empty() && self.cycles.is_empty()
    }
}

/// The input and output cells of a repeater/comparator: `facing` names the
/// INPUT side (verified: `repeater[facing=west]` conducts toward +X).
fn diode_io(p: Pos, block: &str) -> Option<(Pos, Pos)> {
    let f = blocks::facing_of(block)?;
    let (dx, dy, dz) = blocks::facing_vec(f)?;
    Some((p.offset(dx, dy, dz), p.offset(-dx, -dy, -dz)))
}

/// The block a torch is attached to: below for a standing torch, one step
/// opposite `facing` for a wall torch.
fn torch_anchor(p: Pos, block: &str) -> Pos {
    if block.contains("redstone_wall_torch") {
        if let Some((dx, dy, dz)) = blocks::facing_of(block).and_then(blocks::facing_vec) {
            return p.offset(-dx, -dy, -dz);
        }
    }
    p.offset(0, -1, 0)
}

/// Union `p` with every dust cell a powered solid block at `p` conducts to:
/// the four horizontal neighbours at its own level plus the dust on top.
fn union_block_outputs(uf: &mut UnionFind<Pos>, cells: &BTreeMap<Pos, String>, p: Pos) {
    for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1), (0, 1, 0)] {
        let q = p.offset(dx, dy, dz);
        if cells.get(&q).is_some_and(|b| is_dust(b)) {
            uf.union(&p, &q);
        }
    }
}

/// Build conduction components over the cells (see module docs for the
/// exact edge set). Returns a union-find whose roots identify components.
pub fn conduction_components(cells: &BTreeMap<Pos, String>) -> UnionFind<Pos> {
    let mut uf: UnionFind<Pos> = UnionFind::new();
    for (p, b) in cells {
        if is_dust(b) {
            // Dust electrical adjacency including cut diagonals.
            uf.find(p);
            for q in crate::nets::neighbours(cells, *p) {
                uf.union(p, &q);
            }
        } else if is_repeater(b) || is_comparator(b) {
            // Through-diode edge: in-cell, diode, out-cell are one signal.
            let Some((inp, outp)) = diode_io(*p, b) else {
                continue;
            };
            if cells.get(&inp).is_some_and(|b| is_dust(b)) {
                uf.union(p, &inp);
            } else if cells.get(&inp).is_some_and(|b| is_solid_block(b)) {
                // Reading through a block: the BLOCK is the junction. Dust
                // sitting on it, dust beside it (weak-powering it — the
                // bus8 station: dust → entry block → repeater) and any
                // diode strongly powering it all feed this diode; unioning
                // through the block also chains station → refresh repeater
                // (block-fed repeater directly after an exit block).
                uf.union(p, &inp);
                let above = inp.offset(0, 1, 0);
                if cells.get(&above).is_some_and(|b| is_dust(b)) {
                    uf.union(p, &above);
                }
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let q = inp.offset(dx, 0, dz);
                    if q != *p && cells.get(&q).is_some_and(|b| is_dust(b)) {
                        uf.union(p, &q);
                    }
                }
            }
            if cells.get(&outp).is_some_and(|b| is_dust(b)) {
                uf.union(p, &outp);
            } else if cells.get(&outp).is_some_and(|b| is_solid_block(b)) {
                // Strong-powering the block it faces: that block conducts
                // to its adjacent dust.
                uf.union(p, &outp);
                union_block_outputs(&mut uf, cells, outp);
            }
        } else if is_torch(b) {
            // Torch in-out: what powers the anchor flows to the torch's
            // outputs (inverted, but the same routed net — ladder vias).
            let anchor = torch_anchor(*p, b);
            if cells.get(&anchor).is_some_and(|b| is_solid_block(b)) {
                uf.union(p, &anchor);
                // Anchor inputs: pointing dust beside it, dust on top.
                union_block_outputs(&mut uf, cells, anchor);
            }
            // Torch outputs: adjacent dust, dust directly above, and the
            // block above (strong-powered), which conducts onward.
            union_block_outputs(&mut uf, cells, *p);
            let above = p.offset(0, 1, 0);
            if cells.get(&above).is_some_and(|b| is_solid_block(b)) {
                uf.union(p, &above);
                union_block_outputs(&mut uf, cells, above);
            }
        }
    }
    uf
}

/// Compare an intended netlist against the conduction netlist extracted
/// from the cells.
pub fn lvs(cells: &BTreeMap<Pos, String>, intent: &[IntentNet]) -> LvsReport {
    let mut uf = conduction_components(cells);
    let mut report = LvsReport::default();

    // Component root of each terminal. A terminal on empty/air ground is
    // its own singleton (guaranteed open unless the net has one terminal).
    let mut net_roots: Vec<(usize, BTreeMap<Pos, Vec<Pos>>)> = Vec::new();
    for (i, net) in intent.iter().enumerate() {
        let mut groups: BTreeMap<Pos, Vec<Pos>> = BTreeMap::new();
        for t in &net.terminals {
            groups.entry(uf.find(t)).or_default().push(*t);
        }
        net_roots.push((i, groups));
    }

    // Opens: terminals split across components.
    let mut open_nets = vec![false; intent.len()];
    for (i, groups) in &net_roots {
        if groups.len() > 1 {
            open_nets[*i] = true;
            report.opens.push(LvsOpen {
                net: intent[*i].name.clone(),
                fragments: groups.values().cloned().collect(),
            });
        }
    }

    // Shorts: one component claimed by two intent nets.
    let mut shorted = vec![false; intent.len()];
    for a in 0..net_roots.len() {
        for b in a + 1..net_roots.len() {
            let (ia, ga) = &net_roots[a];
            let (ib, gb) = &net_roots[b];
            for (root, wa) in ga {
                if let Some(wb) = gb.get(root) {
                    shorted[*ia] = true;
                    shorted[*ib] = true;
                    report.shorts.push(LvsShort {
                        net_a: intent[*ia].name.clone(),
                        net_b: intent[*ib].name.clone(),
                        at_a: wa[0],
                        at_b: wb[0],
                    });
                }
            }
        }
    }

    for (i, net) in intent.iter().enumerate() {
        if !open_nets[i] && !shorted[i] {
            report.matched.push(net.name.clone());
        }
    }

    // Repeater/comparator rings: an accidental latch is an LVS-visible bug
    // even when every net matches (the cout/cin ring's provenance).
    for v in repeater_cycles(cells) {
        if let Violation::RepeaterCycle { diodes } = v {
            report.cycles.push(diodes);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{repeater, DUST, STONE, TORCH};

    fn dust_at(cells: &mut BTreeMap<Pos, String>, p: Pos) {
        cells.insert(p, DUST.to_string());
    }

    fn net(name: &str, terminals: &[(i32, i32, i32)]) -> IntentNet {
        IntentNet {
            name: name.to_string(),
            terminals: terminals.iter().map(|&(x, y, z)| Pos::new(x, y, z)).collect(),
        }
    }

    #[test]
    fn straight_route_with_refresh_repeater_matches() {
        let mut cells = BTreeMap::new();
        for x in 0..=6 {
            if x == 3 {
                // facing names the INPUT side: west = reads (2,1,0),
                // drives (4,1,0) — a refresh repeater in a 0→6 route.
                cells.insert(Pos::new(3, 1, 0), repeater("west", 1));
            } else {
                dust_at(&mut cells, Pos::new(x, 1, 0));
            }
        }
        let intent = [net("a", &[(0, 1, 0), (6, 1, 0)])];
        let r = lvs(&cells, &intent);
        assert_eq!(r.matched, vec!["a".to_string()], "{r:?}");
        assert!(r.clean(), "{r:?}");
    }

    #[test]
    fn station_dust_block_repeater_block_dust_is_one_net() {
        // Provenance: the bus8 station (dust → entry block → repeater →
        // exit block → dust). The dust BESIDE the in-block weak-powers it;
        // without that edge every station read as an open.
        let mut cells = BTreeMap::new();
        dust_at(&mut cells, Pos::new(0, 1, 0));
        dust_at(&mut cells, Pos::new(1, 1, 0));
        cells.insert(Pos::new(2, 1, 0), "minecraft:magenta_concrete".to_string());
        cells.insert(Pos::new(3, 1, 0), repeater("west", 1));
        cells.insert(Pos::new(4, 1, 0), "minecraft:magenta_concrete".to_string());
        // A refresh repeater directly block-fed by the exit block (the
        // amended through-bus produced exactly this chain).
        cells.insert(Pos::new(5, 1, 0), repeater("west", 1));
        dust_at(&mut cells, Pos::new(6, 1, 0));
        let intent = [net("a", &[(0, 1, 0), (6, 1, 0)])];
        let r = lvs(&cells, &intent);
        assert_eq!(r.matched, vec!["a".to_string()], "{r:?}");
        assert!(r.clean(), "{r:?}");
    }

    #[test]
    fn broken_route_reports_an_open_with_both_fragments() {
        let mut cells = BTreeMap::new();
        for x in 0..=6 {
            if x == 3 {
                continue; // the break
            }
            dust_at(&mut cells, Pos::new(x, 1, 0));
        }
        let intent = [net("a", &[(0, 1, 0), (6, 1, 0)])];
        let r = lvs(&cells, &intent);
        assert!(r.matched.is_empty());
        assert_eq!(r.opens.len(), 1);
        assert_eq!(r.opens[0].net, "a");
        assert_eq!(r.opens[0].fragments.len(), 2, "{r:?}");
        assert!(r.shorts.is_empty());
    }

    #[test]
    fn adjacent_lanes_report_a_short_with_positions() {
        let mut cells = BTreeMap::new();
        for x in 0..=3 {
            dust_at(&mut cells, Pos::new(x, 1, 0));
            dust_at(&mut cells, Pos::new(x, 1, 1)); // touching lane
        }
        let intent = [
            net("a", &[(0, 1, 0), (3, 1, 0)]),
            net("b", &[(0, 1, 1), (3, 1, 1)]),
        ];
        let r = lvs(&cells, &intent);
        assert!(r.matched.is_empty());
        assert_eq!(r.shorts.len(), 1);
        let s = &r.shorts[0];
        assert_eq!((s.net_a.as_str(), s.net_b.as_str()), ("a", "b"));
        // Witnesses come from the intent terminals of each net.
        assert_eq!(s.at_a.z, 0);
        assert_eq!(s.at_b.z, 1);
    }

    #[test]
    fn separated_lanes_do_not_short() {
        let mut cells = BTreeMap::new();
        for x in 0..=3 {
            dust_at(&mut cells, Pos::new(x, 1, 0));
            dust_at(&mut cells, Pos::new(x, 1, 2)); // one clear cell between
        }
        let intent = [
            net("a", &[(0, 1, 0), (3, 1, 0)]),
            net("b", &[(0, 1, 2), (3, 1, 2)]),
        ];
        let r = lvs(&cells, &intent);
        assert_eq!(r.matched.len(), 2, "{r:?}");
        assert!(r.clean(), "{r:?}");
    }

    #[test]
    fn repeater_ring_is_flagged_even_when_nets_match() {
        // Two dust runs joined into a directed ring by opposing diodes —
        // the accidental-latch shape from the adder session.
        let mut cells = BTreeMap::new();
        dust_at(&mut cells, Pos::new(0, 1, 0));
        dust_at(&mut cells, Pos::new(0, 1, 1));
        dust_at(&mut cells, Pos::new(0, 1, 2));
        cells.insert(Pos::new(1, 1, 0), repeater("west", 1)); // (0,z0) -> (2,z0)
        dust_at(&mut cells, Pos::new(2, 1, 0));
        dust_at(&mut cells, Pos::new(2, 1, 1));
        dust_at(&mut cells, Pos::new(2, 1, 2));
        cells.insert(Pos::new(1, 1, 2), repeater("east", 1)); // (2,z2) -> (0,z2)
        let intent = [net("loop", &[(0, 1, 0), (2, 1, 2)])];
        let r = lvs(&cells, &intent);
        assert_eq!(r.cycles.len(), 1, "{r:?}");
        assert!(!r.clean());
        // Both diodes sit on the reported ring.
        assert!(r.cycles[0].contains(&Pos::new(1, 1, 0)));
        assert!(r.cycles[0].contains(&Pos::new(1, 1, 2)));
    }

    #[test]
    fn torch_ladder_counts_as_one_net() {
        // Entry dust -> anchor stone -> torch -> cap stone -> exit dust:
        // the verified ladder via, one intent net end to end.
        let mut cells = BTreeMap::new();
        dust_at(&mut cells, Pos::new(0, 1, 0)); // entry
        cells.insert(Pos::new(1, 1, 0), STONE.to_string()); // anchor
        cells.insert(Pos::new(1, 2, 0), TORCH.to_string()); // torch
        cells.insert(Pos::new(1, 3, 0), STONE.to_string()); // strong-powered cap
        dust_at(&mut cells, Pos::new(1, 4, 0)); // exit
        let intent = [net("up", &[(0, 1, 0), (1, 4, 0)])];
        let r = lvs(&cells, &intent);
        assert_eq!(r.matched, vec!["up".to_string()], "{r:?}");
        assert!(r.clean(), "{r:?}");
    }
}
