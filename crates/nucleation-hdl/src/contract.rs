//! Compile-time cell-contract derivation: the compiled PLA as a TYPED CELL.
//!
//! Groups the BLIF's flattened vector ports (`a[0] a[1] ...` bracket form or
//! `a0 a1 ...` digit-suffix form) back into word-valued bus ports, maps input
//! bits to their lever cells and output bits to their probe cells, and emits
//! the whole thing as a `CellContract` JSON document — the exact serde shape
//! of nucleation's `io_contract::CellContract` (name + IoLayout +
//! PhysicalContract), so the consumer side can `CellContract::from_json` it
//! directly. The crate stays dependency-free: the JSON is hand-rolled with
//! stable key order, like [`crate::Compiled::report_json`].
//!
//! Physical sidecar contents:
//! - one keepout box = the build bounds (no foreign routing through the PLA),
//! - a delay table with one entry per (input port, output port) pair, using
//!   the levelization depth of the output's prim times 2 redstone ticks.
//!   These are ESTIMATES from logic depth, not measured characterization —
//!   good for ordering/budgeting, not for cycle-exact timing,
//! - `paste_safe: false` (unknown until proven).

use std::collections::BTreeMap;

use crate::{Compiled, Value};

/// A typed port derived from the BLIF's flattened net names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortGroup {
    /// Port name: the vector base (`a` for `a[0]..a[3]`) or the original
    /// scalar net name.
    pub name: String,
    /// Member net names in bit order, index = bit significance (LSB first).
    pub bits: Vec<String>,
    /// True = single-bit boolean port; false = unsigned word of `bits.len()`.
    pub boolean: bool,
}

/// Split a net name into `(base, bit index)` if it follows either vector
/// convention: `base[idx]` (yosys `write_blif`) or `baseIdx` (trailing
/// digits).
fn vector_index(name: &str) -> Option<(String, usize)> {
    if let Some(stripped) = name.strip_suffix(']') {
        if let Some((base, idx)) = stripped.rsplit_once('[') {
            if !base.is_empty() && !idx.is_empty() && idx.bytes().all(|b| b.is_ascii_digit()) {
                if let Ok(i) = idx.parse::<usize>() {
                    return Some((base.to_string(), i));
                }
            }
        }
        return None;
    }
    let trimmed = name.trim_end_matches(|c: char| c.is_ascii_digit());
    if trimmed.is_empty() || trimmed.len() == name.len() {
        return None;
    }
    let idx = &name[trimmed.len()..];
    idx.parse::<usize>().ok().map(|i| (trimmed.to_string(), i))
}

/// Group an ordered list of net names into typed ports.
///
/// Rules (conservative — a group only forms when it is unambiguous):
/// - Bracket-form members (`a[i]`) group when their indices are unique and
///   contiguous from 0. A lone `a[0]` becomes a boolean port named `a`.
/// - Digit-suffix members (`a0`, `a1`, ...) group only when there are at
///   least two, unique and contiguous from 0 — a lone `a0` keeps its name
///   (it is probably just a name, not a vector).
/// - A base that collides with a scalar net name, or has duplicate or
///   non-contiguous indices, degrades: every member stays a scalar boolean
///   under its original name.
/// - Everything else is a scalar boolean port.
pub fn group_ports(names: &[String]) -> Vec<PortGroup> {
    #[derive(Default)]
    struct Cand {
        /// (bit index, member name), in appearance order.
        members: Vec<(usize, String)>,
        /// Any member used the bracket convention.
        bracket: bool,
    }
    let scalars: Vec<&String> = names.iter().filter(|n| vector_index(n).is_none()).collect();
    let mut cands: BTreeMap<String, Cand> = BTreeMap::new();
    let mut order: Vec<Item> = Vec::new();
    enum Item {
        Scalar(String),
        Base(String),
    }
    for name in names.iter() {
        match vector_index(name) {
            Some((base, idx)) => {
                let c = cands.entry(base.clone()).or_default();
                if c.members.is_empty() {
                    order.push(Item::Base(base.clone()));
                }
                c.bracket |= name.ends_with(']');
                c.members.push((idx, name.clone()));
            }
            None => order.push(Item::Scalar(name.clone())),
        }
    }
    let mut out = Vec::new();
    for item in order {
        match item {
            Item::Scalar(name) => out.push(PortGroup {
                name: name.clone(),
                bits: vec![name],
                boolean: true,
            }),
            Item::Base(base) => {
                let c = &cands[&base];
                let mut sorted = c.members.clone();
                sorted.sort_by_key(|(i, _)| *i);
                let unique = sorted.windows(2).all(|w| w[0].0 != w[1].0);
                let contiguous = sorted
                    .iter()
                    .enumerate()
                    .all(|(want, (have, _))| want == *have);
                let collides = scalars.iter().any(|s| **s == base);
                let big_enough = sorted.len() >= 2 || c.bracket;
                if unique && contiguous && !collides && big_enough {
                    let boolean = sorted.len() == 1;
                    out.push(PortGroup {
                        name: base,
                        bits: sorted.into_iter().map(|(_, n)| n).collect(),
                        boolean,
                    });
                } else {
                    for (_, name) in c.members.iter() {
                        out.push(PortGroup {
                            name: name.clone(),
                            bits: vec![name.clone()],
                            boolean: true,
                        });
                    }
                }
            }
        }
    }
    out
}

/// Pick the horizontal bounds face the port's cells sit nearest to.
/// (PLA IO presents laterally; up/down are never port faces here.)
fn nearest_face(
    positions: &[(i32, i32, i32)],
    bounds: ((i32, i32, i32), (i32, i32, i32)),
) -> &'static str {
    let ((x0, _, z0), (x1, _, z1)) = bounds;
    let n = positions.len().max(1) as i64;
    let (sx, sz) = positions.iter().fold((0i64, 0i64), |(ax, az), &(x, _, z)| {
        (ax + i64::from(x), az + i64::from(z))
    });
    let (mx, mz) = (sx / n, sz / n);
    // (distance to plane, tie-break order, face name)
    let faces = [
        (mx - i64::from(x0), 0, "west"),
        (i64::from(x1) - mx, 1, "east"),
        (mz - i64::from(z0), 2, "north"),
        (i64::from(z1) - mz, 3, "south"),
    ];
    faces.iter().min_by_key(|(d, t, _)| (*d, *t)).unwrap().2
}

/// If the bit positions advance by a constant nonzero step along exactly one
/// axis, return `(axis, spacing)` for a bus pitch.
fn uniform_pitch(positions: &[(i32, i32, i32)]) -> Option<(&'static str, i32)> {
    if positions.len() < 2 {
        return None;
    }
    let d = (
        positions[1].0 - positions[0].0,
        positions[1].1 - positions[0].1,
        positions[1].2 - positions[0].2,
    );
    for w in positions.windows(2) {
        let step = (w[1].0 - w[0].0, w[1].1 - w[0].1, w[1].2 - w[0].2);
        if step != d {
            return None;
        }
    }
    match d {
        (s, 0, 0) if s != 0 => Some(("X", s)),
        (0, s, 0) if s != 0 => Some(("Y", s)),
        (0, 0, s) if s != 0 => Some(("Z", s)),
        _ => None,
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn pos_json(p: (i32, i32, i32)) -> String {
    format!("[{},{},{}]", p.0, p.1, p.2)
}

fn ty_json(boolean: bool, bits: usize) -> String {
    if boolean {
        "\"Boolean\"".to_string()
    } else {
        format!("{{\"UnsignedInt\":{{\"bits\":{bits}}}}}")
    }
}

/// One resolved port: name, member positions in bit order, type flag.
struct ResolvedPort {
    name: String,
    boolean: bool,
    positions: Vec<(i32, i32, i32)>,
    /// Levelization depth of the deepest member prim (outputs only).
    depth: i32,
}

impl ResolvedPort {
    fn mapping_json(&self, direction: &str, bounds: ((i32, i32, i32), (i32, i32, i32))) -> String {
        let positions: Vec<String> = self.positions.iter().map(|&p| pos_json(p)).collect();
        format!(
            "{{\"io_type\":{},\"layout\":\"OneToOne\",\"positions\":[{}],\"face\":\"{}\",\"direction\":\"{}\"}}",
            ty_json(self.boolean, self.positions.len()),
            positions.join(","),
            nearest_face(&self.positions, bounds),
            direction
        )
    }

    fn bus_json(
        &self,
        direction: &str,
        bounds: ((i32, i32, i32), (i32, i32, i32)),
    ) -> Option<String> {
        if self.boolean {
            return None;
        }
        let (axis, spacing) = uniform_pitch(&self.positions)?;
        Some(format!(
            "{{\"spec\":{{\"width\":{},\"ty\":{},\"pitch\":{{\"axis\":\"{axis}\",\"spacing\":{spacing}}},\"face\":\"{}\",\"encoding\":\"binary1_per_wire\"}},\"bit0\":{},\"direction\":\"{}\"}}",
            self.positions.len(),
            ty_json(false, self.positions.len()),
            nearest_face(&self.positions, bounds),
            pos_json(self.positions[0]),
            direction
        ))
    }
}

impl Compiled {
    /// The input ports, grouped and typed, each bit resolved to its lever
    /// cell (bit order = vector index, LSB first).
    fn input_ports(&self) -> Vec<ResolvedPort> {
        let lever_of: BTreeMap<&str, (i32, i32, i32)> = self
            .inputs
            .iter()
            .zip(self.levers.iter())
            .map(|(pi, (_, cell))| (pi.as_str(), *cell))
            .collect();
        group_ports(&self.inputs)
            .into_iter()
            .map(|g| ResolvedPort {
                positions: g.bits.iter().map(|b| lever_of[b.as_str()]).collect(),
                name: g.name,
                boolean: g.boolean,
                depth: 0,
            })
            .collect()
    }

    /// The output ports, grouped and typed, each bit resolved to its probe
    /// cell. Constant-folded outputs have no probe and no physical port: a
    /// constant scalar is dropped, and a group with any constant bit
    /// degrades to per-bit scalars (the constant bits dropped) so every
    /// emitted position is real.
    fn output_ports(&self) -> Vec<ResolvedPort> {
        let value_of: BTreeMap<&str, &Value> = self
            .outputs
            .iter()
            .map(|(po, v)| (po.as_str(), v))
            .collect();
        let names: Vec<String> = self.outputs.iter().map(|(po, _)| po.clone()).collect();
        let mut groups = Vec::new();
        for g in group_ports(&names) {
            let any_const = g
                .bits
                .iter()
                .any(|b| matches!(value_of[b.as_str()], Value::Const(_)));
            if any_const && g.bits.len() > 1 {
                for b in g.bits {
                    groups.push(PortGroup {
                        name: b.clone(),
                        bits: vec![b],
                        boolean: true,
                    });
                }
            } else {
                groups.push(g);
            }
        }
        let mut out = Vec::new();
        for g in groups {
            let mut positions = Vec::new();
            let mut depth = 0i32;
            let mut skip = false;
            for b in &g.bits {
                match value_of[b.as_str()] {
                    Value::Const(_) => skip = true,
                    Value::Vid(vid) => {
                        positions.push(self.probes[vid]);
                        depth = depth.max(self.comp.level.get(vid).copied().unwrap_or(0));
                    }
                }
            }
            if skip || positions.is_empty() {
                continue;
            }
            out.push(ResolvedPort {
                name: g.name,
                boolean: g.boolean,
                positions,
                depth,
            });
        }
        out
    }

    /// The cell contract as JSON, in the exact serde shape of nucleation's
    /// `io_contract::CellContract` (see module docs for what the physical
    /// sidecar carries and the ESTIMATED nature of the delay table).
    pub fn cell_contract_json(&self) -> String {
        let bounds = self.stats.bounds;
        let inputs = self.input_ports();
        let outputs = self.output_ports();

        let mut in_entries = Vec::new();
        let mut bus_entries = Vec::new();
        for p in &inputs {
            in_entries.push(format!(
                "\"{}\":{}",
                esc(&p.name),
                p.mapping_json("input", bounds)
            ));
            if let Some(b) = p.bus_json("input", bounds) {
                bus_entries.push(format!("\"{}\":{}", esc(&p.name), b));
            }
        }
        // The clock is a real input port (its position is the spine lever),
        // flagged as such in the `sequential` sidecar below — the shared
        // CellContract parser keeps the port and ignores the sidecar.
        if let Some(ck) = &self.clock {
            in_entries.push(format!(
                "\"{}\":{{\"io_type\":\"Boolean\",\"layout\":\"OneToOne\",\
                 \"positions\":[{}],\"face\":\"{}\",\"direction\":\"input\"}}",
                esc(&ck.net),
                pos_json(ck.lever),
                nearest_face(&[ck.lever], bounds)
            ));
        }
        let mut out_entries = Vec::new();
        for p in &outputs {
            out_entries.push(format!(
                "\"{}\":{}",
                esc(&p.name),
                p.mapping_json("output", bounds)
            ));
            if let Some(b) = p.bus_json("output", bounds) {
                bus_entries.push(format!("\"{}\":{}", esc(&p.name), b));
            }
        }

        // Estimated delay table: levelization depth * 2rt from every input
        // port to each output port (depth is a property of the output cone).
        let mut delays = Vec::new();
        for o in &outputs {
            let rt = u32::try_from(o.depth.max(1)).unwrap_or(1) * 2;
            for i in &inputs {
                delays.push(format!(
                    "{{\"from\":\"{}\",\"to\":\"{}\",\"delay_rt\":{rt}}}",
                    esc(&i.name),
                    esc(&o.name)
                ));
            }
        }

        let ((x0, y0, z0), (x1, y1, z1)) = bounds;
        let buses = if bus_entries.is_empty() {
            String::new()
        } else {
            format!(",\"buses\":{{{}}}", bus_entries.join(","))
        };
        // Sequential sidecar: names the clock port (clock=true, in effect),
        // carries the characterized period floor + the estimated compiled
        // period, and the baked initial state — init-by-construction means
        // the schematic ITSELF is the reset state. Unknown to the shared
        // CellContract type (serde ignores it); documented in hdl/README.
        let sequential = self.clock.as_ref().map_or(String::new(), |ck| {
            let inits: Vec<String> = self
                .latches
                .iter()
                .map(|l| format!("\"{}\":{}", esc(&l.q), l.init))
                .collect();
            format!(
                ",\"sequential\":{{\"clock_port\":\"{}\",\"clock\":true,\
                 \"est_min_period_gt\":{},\
                 \"dff\":{{\"setup_gt\":{},\"hold_gt\":{},\"min_pulse_gt\":{},\
                 \"clk_to_q_gt\":{},\"min_period_gt\":{},\
                 \"spine_skew_per_repeater_gt\":{}}},\
                 \"initial_state\":{{{}}}}}",
                esc(&ck.net),
                ck.est_min_period_gt,
                crate::seq::DFF_SETUP_GT,
                crate::seq::DFF_HOLD_GT,
                crate::seq::DFF_MIN_PULSE_GT,
                crate::seq::DFF_CLK_TO_Q_GT,
                crate::seq::DFF_MIN_PERIOD_GT,
                crate::seq::SPINE_SKEW_PER_REPEATER_GT,
                inits.join(",")
            )
        });
        format!(
            "{{\"name\":\"{}\",\"io\":{{\"inputs\":{{{}}},\"outputs\":{{{}}}{}}},\
             \"physical\":{{\"keepouts\":[{{\"min\":[{x0},{y0},{z0}],\"max\":[{x1},{y1},{z1}]}}],\
             \"delays_rt\":[{}],\"paste_safe\":false}}{}}}",
            esc(&self.name),
            in_entries.join(","),
            out_entries.join(","),
            buses,
            delays.join(","),
            sequential
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn bracket_vectors_group_into_words() {
        let g = group_ports(&names(&["a[0]", "a[1]", "a[2]", "a[3]", "cin"]));
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].name, "a");
        assert_eq!(g[0].bits, names(&["a[0]", "a[1]", "a[2]", "a[3]"]));
        assert!(!g[0].boolean);
        assert_eq!(g[1].name, "cin");
        assert!(g[1].boolean);
    }

    #[test]
    fn digit_suffix_vectors_group_too() {
        let g = group_ports(&names(&["d0", "d1", "d2"]));
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].name, "d");
        assert_eq!(g[0].bits, names(&["d0", "d1", "d2"]));
    }

    #[test]
    fn file_order_does_not_matter_bit_order_does() {
        let g = group_ports(&names(&["a[2]", "a[0]", "a[1]"]));
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].bits, names(&["a[0]", "a[1]", "a[2]"]));
    }

    #[test]
    fn lone_digit_suffix_stays_a_scalar_name() {
        let g = group_ports(&names(&["a0", "b"]));
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].name, "a0");
        assert!(g[0].boolean);
    }

    #[test]
    fn lone_bracket_bit_becomes_boolean_base() {
        let g = group_ports(&names(&["en[0]"]));
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].name, "en");
        assert!(g[0].boolean);
    }

    #[test]
    fn non_contiguous_indices_degrade_to_scalars() {
        let g = group_ports(&names(&["a[0]", "a[2]"]));
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].name, "a[0]");
        assert_eq!(g[1].name, "a[2]");
        assert!(g.iter().all(|p| p.boolean));
    }

    #[test]
    fn base_colliding_with_scalar_degrades() {
        let g = group_ports(&names(&["a", "a0", "a1"]));
        assert_eq!(g.len(), 3);
        assert!(g.iter().all(|p| p.boolean));
    }

    #[test]
    fn pitch_detection_needs_one_constant_axis() {
        assert_eq!(
            uniform_pitch(&[(0, 1, 0), (2, 1, 0), (4, 1, 0)]),
            Some(("X", 2))
        );
        assert_eq!(uniform_pitch(&[(0, 1, 0), (0, 1, 3)]), Some(("Z", 3)));
        assert_eq!(uniform_pitch(&[(0, 1, 0), (2, 1, 1)]), None);
        assert_eq!(uniform_pitch(&[(0, 1, 0), (2, 1, 0), (5, 1, 0)]), None);
        assert_eq!(uniform_pitch(&[(0, 1, 0)]), None);
    }

    fn fixture(name: &str) -> String {
        let path = format!("{}/tests/data/{name}.blif", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(path).expect("checked-in BLIF fixture")
    }

    #[test]
    fn cmp4_contract_carries_typed_buses_and_estimated_delays() {
        let c = crate::compile_blif(&fixture("cmp4"), "cmp4").unwrap();
        let json = c.cell_contract_json();
        // a and b group to 4-bit unsigned input words; eq/lt/gt stay boolean.
        assert!(
            json.contains("\"a\":{\"io_type\":{\"UnsignedInt\":{\"bits\":4}}"),
            "{json}"
        );
        assert!(
            json.contains("\"b\":{\"io_type\":{\"UnsignedInt\":{\"bits\":4}}"),
            "{json}"
        );
        assert!(json.contains("\"eq\":{\"io_type\":\"Boolean\""), "{json}");
        assert!(json.contains("\"direction\":\"output\""), "{json}");
        // physical sidecar: bounds keepout, estimated delays, paste_safe.
        assert!(json.contains("\"keepouts\":[{\"min\":["), "{json}");
        assert!(json.contains("\"to\":\"lt\",\"delay_rt\":"), "{json}");
        assert!(json.contains("\"paste_safe\":false"), "{json}");
        // every input bit resolved to a distinct lever cell
        let a_positions = json.split("\"a\":{").nth(1).unwrap();
        assert!(a_positions.contains("\"positions\":[["), "{json}");
    }

    #[test]
    fn counter4_contract_flags_the_clock_and_bakes_initial_state() {
        let c = crate::compile_blif(&fixture("counter4"), "counter4").unwrap();
        let json = c.cell_contract_json();
        // the clock is a real Boolean input port at the spine lever...
        assert!(json.contains("\"clk\":{\"io_type\":\"Boolean\""), "{json}");
        // ...flagged in the sequential sidecar with the characterized DFF
        // table and the estimated compiled period
        assert!(
            json.contains("\"sequential\":{\"clock_port\":\"clk\",\"clock\":true"),
            "{json}"
        );
        assert!(json.contains("\"clk_to_q_gt\":10"), "{json}");
        assert!(json.contains("\"est_min_period_gt\":"), "{json}");
        // init-by-construction: the baked state ships in the contract
        assert!(
            json.contains("\"initial_state\":{\"q[0]\":0,\"q[1]\":0"),
            "{json}"
        );
        // the state word still groups into a typed output bus
        assert!(
            json.contains("\"q\":{\"io_type\":{\"UnsignedInt\":{\"bits\":4}}"),
            "{json}"
        );
    }

    #[test]
    fn seg7_groups_the_display_word() {
        let c = crate::compile_blif(&fixture("seg7"), "seg7").unwrap();
        let json = c.cell_contract_json();
        assert!(
            json.contains("\"d\":{\"io_type\":{\"UnsignedInt\":{\"bits\":4}}"),
            "{json}"
        );
        assert!(
            json.contains("\"seg\":{\"io_type\":{\"UnsignedInt\":{\"bits\":7}}"),
            "{json}"
        );
    }
}
