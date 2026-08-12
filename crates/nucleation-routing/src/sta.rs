//! Static timing analysis for routed redstone (port of `timing.py`, on top
//! of `pnr_core::sta`).
//!
//! Delay model, in redstone ticks (1 rt = 2 game ticks): every torch
//! inverts in 1 rt, every repeater adds its delay, dust is free. Repeaters
//! are attributed per NET from the built geometry — every repeater is
//! charged to the signal whose dust it sits beside — so a consumer of a net
//! is charged the whole net's repeaters. The estimate is an upper bound
//! (validated within ~1.4x of measured on the 4-bit adder), which is the
//! right direction to be wrong in.

use crate::blocks::{is_repeater, repeater_delay};
use crate::workspace::Workspace;
use pnr_core::sta::DelayGraph;
use pnr_core::Pos;
use std::collections::BTreeMap;

/// A combinational gate: `out = f(ins)` with an intrinsic delay.
#[derive(Clone, Debug)]
pub struct Gate {
    /// Output signal name.
    pub out: String,
    /// Input signal names.
    pub ins: Vec<String>,
    /// Intrinsic delay in redstone ticks (e.g. 2 for a tap-torch +
    /// gate-torch column, 1 for a comparator).
    pub delay_rt: u32,
}

/// Timing analysis result.
#[derive(Clone, Debug)]
pub struct TimingReport {
    /// Arrival time per signal, redstone ticks.
    pub arrival_rt: BTreeMap<String, u64>,
    /// The worst signal and its critical path, source first.
    pub critical: Vec<String>,
}

/// STA failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaError {
    /// The netlist has a combinational loop through the listed signals.
    CombinationalLoop(Vec<String>),
    /// A gate input is neither a primary input nor another gate's output.
    UnknownSignal(String),
}

/// Repeaters attributed per net from geometry: a repeater belongs to the
/// first labelled dust beside it whose label carries no `#` (pre-gate
/// collectors like `sig#13` are internal). Port of `timing.py`'s
/// `net_repeaters`.
pub fn net_repeaters(ws: &Workspace) -> BTreeMap<String, Vec<Pos>> {
    let mut reps: BTreeMap<String, Vec<Pos>> = BTreeMap::new();
    for (p, blk) in ws.cells() {
        if !is_repeater(blk) {
            continue;
        }
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            if let Some(lab) = ws.label(p.offset(dx, 0, dz)) {
                if !lab.contains('#') {
                    reps.entry(lab.to_string()).or_default().push(*p);
                    break;
                }
            }
        }
    }
    reps
}

/// Total repeater delay (rt) charged to a net.
pub fn net_repeater_delay_rt(ws: &Workspace, reps: &BTreeMap<String, Vec<Pos>>, net: &str) -> u64 {
    reps.get(net)
        .map(|v| {
            v.iter()
                .map(|p| ws.get(*p).map_or(1, repeater_delay) as u64)
                .sum()
        })
        .unwrap_or(0)
}

/// Analyze a gate netlist over routed geometry. Arrival of a gate output =
/// worst input arrival + that input net's repeater delay + the gate's
/// intrinsic delay; primary inputs arrive at 0.
pub fn sta(ws: &Workspace, inputs: &[String], gates: &[Gate]) -> Result<TimingReport, StaError> {
    // Signal name -> node id.
    let mut ids: BTreeMap<&str, usize> = BTreeMap::new();
    for s in inputs {
        let n = ids.len();
        ids.entry(s.as_str()).or_insert(n);
    }
    for g in gates {
        let n = ids.len();
        ids.entry(g.out.as_str()).or_insert(n);
    }
    let names: BTreeMap<usize, &str> = ids.iter().map(|(s, i)| (*i, *s)).collect();

    let reps = net_repeaters(ws);
    let mut graph = DelayGraph::new(ids.len());
    for g in gates {
        let out = ids[g.out.as_str()];
        for input in &g.ins {
            let Some(&iid) = ids.get(input.as_str()) else {
                return Err(StaError::UnknownSignal(input.clone()));
            };
            let d = g.delay_rt as u64 + net_repeater_delay_rt(ws, &reps, input);
            graph.edge(iid, out, d as u32);
        }
    }
    let sources: Vec<(usize, u64)> = inputs.iter().map(|s| (ids[s.as_str()], 0)).collect();
    let res = graph.analyze(&sources).map_err(|e| {
        StaError::CombinationalLoop(e.nodes.iter().map(|n| names[n].to_string()).collect())
    })?;

    let mut arrival_rt = BTreeMap::new();
    for (name, id) in &ids {
        if let Some(t) = res.arrival[*id] {
            arrival_rt.insert(name.to_string(), t);
        }
    }
    // Critical path to the worst-arriving signal.
    let worst = arrival_rt
        .iter()
        .max_by_key(|(name, t)| (**t, std::cmp::Reverse(name.as_str())))
        .map(|(n, _)| n.clone());
    let critical = match worst {
        Some(w) => res
            .critical_path(ids[w.as_str()])
            .into_iter()
            .map(|i| names[&i].to_string())
            .collect(),
        None => Vec::new(),
    };
    Ok(TimingReport {
        arrival_rt,
        critical,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::repeater;

    #[test]
    fn repeaters_charge_their_net_and_arrivals_add_up() {
        let mut ws = Workspace::new();
        // Net "a": dust either side of a repeater at (1,1,0).
        ws.dust(Pos::new(0, 1, 0), "a").unwrap();
        ws.stone(Pos::new(1, 0, 0), "plain").unwrap();
        ws.put(Pos::new(1, 1, 0), &repeater("west", 1)).unwrap();
        ws.dust(Pos::new(2, 1, 0), "a").unwrap();
        // Internal collector label is never charged.
        ws.dust(Pos::new(0, 1, 4), "x#3").unwrap();
        ws.stone(Pos::new(1, 0, 4), "plain").unwrap();
        ws.put(Pos::new(1, 1, 4), &repeater("west", 1)).unwrap();

        let reps = net_repeaters(&ws);
        assert_eq!(reps.get("a").map(Vec::len), Some(1));
        assert!(reps.get("x#3").is_none());

        // a --(gate 2rt)--> y --(gate 2rt)--> z ; b joins at z.
        let gates = vec![
            Gate {
                out: "y".into(),
                ins: vec!["a".into()],
                delay_rt: 2,
            },
            Gate {
                out: "z".into(),
                ins: vec!["y".into(), "b".into()],
                delay_rt: 2,
            },
        ];
        let r = sta(&ws, &["a".into(), "b".into()], &gates).unwrap();
        // y = a(0) + rep(1) + 2 = 3 ; z = y(3) + 2 = 5 (b path is 2).
        assert_eq!(r.arrival_rt["y"], 3);
        assert_eq!(r.arrival_rt["z"], 5);
        assert_eq!(r.critical, vec!["a", "y", "z"]);
    }

    #[test]
    fn combinational_loop_is_an_error() {
        let ws = Workspace::new();
        let gates = vec![
            Gate {
                out: "p".into(),
                ins: vec!["q".into()],
                delay_rt: 1,
            },
            Gate {
                out: "q".into(),
                ins: vec!["p".into()],
                delay_rt: 1,
            },
        ];
        let err = sta(&ws, &[], &gates).unwrap_err();
        match err {
            StaError::CombinationalLoop(names) => {
                assert!(names.contains(&"p".to_string()) && names.contains(&"q".to_string()));
            }
            e => panic!("unexpected {e:?}"),
        }
    }

    #[test]
    fn unknown_signal_is_an_error() {
        let ws = Workspace::new();
        let gates = vec![Gate {
            out: "y".into(),
            ins: vec!["ghost".into()],
            delay_rt: 1,
        }];
        assert_eq!(
            sta(&ws, &[], &gates).unwrap_err(),
            StaError::UnknownSignal("ghost".to_string())
        );
    }
}
