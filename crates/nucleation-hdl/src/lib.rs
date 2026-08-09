//! HDL -> redstone: compile a combinational BLIF into a sim-verified
//! dual-rail PLA build.
//!
//! Rust port of `redstone-eda/hdl/hdl2redstone.py` + `redstone-eda/build_ppa.py`
//! (the verified Python pipeline is the spec; its test vectors — seg7 16/16,
//! cmp4 256/256, popcnt4 16/16 — are this crate's integration tests):
//!
//! - [`blif`]: BLIF subset parse (`.model`/`.inputs`/`.outputs`/`.names`;
//!   `.latch`/`.subckt` rejected) + constant folding.
//! - [`logic`]: dual-rail values with Quine-McCluskey off-set covers,
//!   peephole double-inversion collapse, levelization with buffer insertion,
//!   2-per-slice stage packing.
//! - [`pla`]: the PLA geometry compiler — rails with station-pitch
//!   repeaters, input-stage inverter torches, AND columns, inter-stage
//!   corridors — emitting cells + probe/lever metadata.
//! - [`contract`]: compile-time typed-cell derivation — vector ports grouped
//!   into word buses, levers/probes mapped to named ports, emitted as a
//!   `CellContract`-shaped JSON document.
//! - [`verify`] (feature `mc-tick`): exhaustive/sampled in-sim verification
//!   against the pure-Rust prim-graph eval, with lever discipline.
//!
//! The core compile path has zero dependencies and is wasm-clean.

pub mod blif;
pub mod contract;
pub mod logic;
pub mod pla;
pub mod seq;
#[cfg(feature = "mc-tick")]
pub mod verify;

use std::collections::HashMap;

pub use logic::Value;
pub use pla::Build;

/// Everything that can go wrong between BLIF text and placed geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HdlError {
    /// The BLIF uses a feature outside the supported combinational subset.
    Unsupported(String),
    /// The BLIF text is malformed or inconsistent.
    Parse(String),
    /// The plan cannot be made physical (a Python-side assert).
    Layout(String),
}

impl std::fmt::Display for HdlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HdlError::Unsupported(m) => write!(f, "unsupported BLIF: {m}"),
            HdlError::Parse(m) => write!(f, "BLIF parse: {m}"),
            HdlError::Layout(m) => write!(f, "layout: {m}"),
        }
    }
}

impl std::error::Error for HdlError {}

/// Compile statistics, for reports.
#[derive(Debug, Clone)]
pub struct Stats {
    /// Prim nodes placed (incl. complements/buffers/splits).
    pub prims: usize,
    /// Logic depth in stages.
    pub levels: i32,
    /// Buffer/double-inversion nodes the peephole collapsed.
    pub peephole_removed: usize,
    /// Nets that folded to constants.
    pub const_nets: usize,
    /// Authored (non-air) cells.
    pub blocks: usize,
    /// `(min, max)` corners of the build.
    pub bounds: ((i32, i32, i32), (i32, i32, i32)),
}

/// One compiled latch: nets, resolved D value, baked init and port cells.
#[derive(Debug, Clone)]
pub struct LatchSpec {
    /// The Q net (BLIF `.latch` output).
    pub q: String,
    /// The D net (BLIF `.latch` input).
    pub d: String,
    /// Baked initial Q (0/1; BLIF don't-care/unknown bake at 0).
    pub init: u8,
    /// D as the reference model computes it: a constant or a resolved vid.
    pub d_val: Value,
    /// The stage-0 rail this latch's Q drives (`<q vid>.lv`) — its probe
    /// reads Q as the fabric sees it, after the wrap corridor.
    pub q_rail: String,
    /// The DFF's D port dust cell (probe key `dff<k>.d`).
    pub d_port: (i32, i32, i32),
    /// The DFF's Q port dust cell (probe key `dff<k>.q`).
    pub q_port: (i32, i32, i32),
}

/// The clock of a sequential design: one net, one lever, characterization.
#[derive(Debug, Clone)]
pub struct Clock {
    /// The `.latch` control net.
    pub net: String,
    /// The spine's drive lever.
    pub lever: (i32, i32, i32),
    /// ESTIMATED minimum period in game ticks (DFF floor + depth + wrap
    /// allowance). [`verify::verify_clocked`] measures the real margins.
    pub est_min_period_gt: u32,
}

/// A compiled design: geometry, I/O metadata, and the reference model.
pub struct Compiled {
    /// Design name.
    pub name: String,
    /// The authored voxels + net labels.
    pub build: pla::Build,
    /// signal -> a dust cell that reads its settled value.
    pub probes: std::collections::BTreeMap<String, (i32, i32, i32)>,
    /// `(rail signal, lever cell)` in drive order (PI order).
    pub levers: Vec<(String, (i32, i32, i32))>,
    /// Primary inputs, in `.inputs` order (= lever order; the clock net and
    /// latch state nets are NOT in this list).
    pub inputs: Vec<String>,
    /// Primary outputs with their resolved value: a constant or a probed vid.
    pub outputs: Vec<(String, Value)>,
    /// Latches, in `.latch` file order. Empty for combinational designs.
    pub latches: Vec<LatchSpec>,
    /// The clock, when the design is sequential.
    pub clock: Option<Clock>,
    /// The prim-graph compiler, kept for [`logic::Compiler::eval`].
    pub comp: logic::Compiler,
    /// Compile statistics.
    pub stats: Stats,
}

impl Compiled {
    /// Evaluate the reference model for one input assignment (`bits[i]` is
    /// the value of `inputs[i]`).
    pub fn eval(&self, bits: &[u8]) -> HashMap<String, u8> {
        self.comp.eval(bits)
    }

    /// The expected primary-output values for one input assignment, in
    /// `.outputs` order.
    pub fn eval_outputs(&self, bits: &[u8]) -> Vec<(String, u8)> {
        let val = self.eval(bits);
        self.outputs
            .iter()
            .map(|(po, v)| {
                let b = match v {
                    Value::Const(c) => *c,
                    Value::Vid(vid) => val[vid],
                };
                (po.clone(), b)
            })
            .collect()
    }

    /// Reference model, sequential: evaluate the fabric for one input
    /// assignment and one latch-state assignment (`state[k]` = Q of
    /// `latches[k]`).
    pub fn seq_eval(&self, pi_bits: &[u8], state: &[u8]) -> HashMap<String, u8> {
        let bits: Vec<u8> = pi_bits.iter().chain(state.iter()).copied().collect();
        self.comp.eval(&bits)
    }

    /// The next latch state after a rising edge, from a [`Self::seq_eval`]
    /// valuation.
    pub fn latch_next(&self, val: &HashMap<String, u8>) -> Vec<u8> {
        self.latches
            .iter()
            .map(|l| match &l.d_val {
                Value::Const(c) => *c,
                Value::Vid(v) => val[v],
            })
            .collect()
    }

    /// The expected primary-output values from a [`Self::seq_eval`]
    /// valuation, in `.outputs` order.
    pub fn outputs_from(&self, val: &HashMap<String, u8>) -> Vec<(String, u8)> {
        self.outputs
            .iter()
            .map(|(po, v)| {
                let b = match v {
                    Value::Const(c) => *c,
                    Value::Vid(vid) => val[vid],
                };
                (po.clone(), b)
            })
            .collect()
    }

    /// A JSON report: stats, probes, levers, outputs — plus `latches` and
    /// `clock` for sequential designs. Hand-rolled (the crate
    /// is dependency-free); keys are sorted and stable.
    pub fn report_json(&self) -> String {
        fn esc(s: &str) -> String {
            s.replace('\\', "\\\\").replace('"', "\\\"")
        }
        fn pos(p: (i32, i32, i32)) -> String {
            format!("[{},{},{}]", p.0, p.1, p.2)
        }
        let probes: Vec<String> = self
            .probes
            .iter()
            .map(|(s, p)| format!("\"{}\":{}", esc(s), pos(*p)))
            .collect();
        let levers: Vec<String> = self
            .levers
            .iter()
            .map(|(s, p)| format!("{{\"signal\":\"{}\",\"pos\":{}}}", esc(s), pos(*p)))
            .collect();
        let inputs: Vec<String> = self.inputs.iter().map(|s| format!("\"{}\"", esc(s))).collect();
        let outputs: Vec<String> = self
            .outputs
            .iter()
            .map(|(po, v)| match v {
                Value::Const(c) => format!("{{\"name\":\"{}\",\"const\":{c}}}", esc(po)),
                Value::Vid(vid) => {
                    format!("{{\"name\":\"{}\",\"probe\":\"{}\"}}", esc(po), esc(vid))
                }
            })
            .collect();
        let latches: Vec<String> = self
            .latches
            .iter()
            .enumerate()
            .map(|(k, l)| {
                let d = match &l.d_val {
                    Value::Const(c) => format!("\"d_const\":{c}"),
                    Value::Vid(v) => format!("\"d_vid\":\"{}\"", esc(v)),
                };
                format!(
                    "{{\"q\":\"{}\",\"d\":\"{}\",\"init\":{},{d},\
                     \"q_rail\":\"{}\",\"d_probe\":\"dff{k}.d\",\
                     \"q_probe\":\"dff{k}.q\",\"d_port\":{},\"q_port\":{}}}",
                    esc(&l.q),
                    esc(&l.d),
                    l.init,
                    esc(&l.q_rail),
                    pos(l.d_port),
                    pos(l.q_port)
                )
            })
            .collect();
        let clock = self.clock.as_ref().map_or(String::new(), |c| {
            format!(
                ",\"clock\":{{\"net\":\"{}\",\"lever\":{},\"est_min_period_gt\":{}}}",
                esc(&c.net),
                pos(c.lever),
                c.est_min_period_gt
            )
        });
        let ((x0, y0, z0), (x1, y1, z1)) = self.stats.bounds;
        format!(
            "{{\"name\":\"{}\",\"prims\":{},\"levels\":{},\"peephole_removed\":{},\
             \"const_nets\":{},\"blocks\":{},\
             \"bounds\":[[{x0},{y0},{z0}],[{x1},{y1},{z1}]],\
             \"inputs\":[{}],\"outputs\":[{}],\"levers\":[{}],\
             \"latches\":[{}]{},\"probes\":{{{}}}}}",
            esc(&self.name),
            self.stats.prims,
            self.stats.levels,
            self.stats.peephole_removed,
            self.stats.const_nets,
            self.stats.blocks,
            inputs.join(","),
            outputs.join(","),
            levers.join(","),
            latches.join(","),
            clock,
            probes.join(",")
        )
    }
}

/// Validate the sequential structure and name the clock net (if any).
fn seq_checks(parsed: &blif::Blif) -> Result<Option<String>, HdlError> {
    if parsed.latches.is_empty() {
        return Ok(None);
    }
    let mut clock: Option<&str> = None;
    let mut q_seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for l in &parsed.latches {
        let c = l.control.as_deref().ok_or_else(|| {
            HdlError::Unsupported(".latch without a clock control net".into())
        })?;
        match clock {
            None => clock = Some(c),
            Some(k) if k != c => {
                return Err(HdlError::Unsupported(format!(
                    "multiple clock domains ({k} and {c})"
                )))
            }
            _ => {}
        }
        if !q_seen.insert(&l.output) {
            return Err(HdlError::Parse(format!(
                "two .latch lines drive {}",
                l.output
            )));
        }
    }
    let clock = clock.expect("latches non-empty").to_string();
    if !parsed.inputs.contains(&clock) {
        return Err(HdlError::Unsupported(format!(
            "clock net {clock} is not a primary input"
        )));
    }
    for (n, node) in &parsed.nodes {
        if q_seen.contains(n.as_str()) {
            return Err(HdlError::Parse(format!(
                "net {n} driven by both .names and .latch"
            )));
        }
        if node.inputs.contains(&clock) {
            return Err(HdlError::Unsupported(format!(
                "clock net {clock} feeds combinational logic ({n}) — the \
                 clock is a dedicated spine, not a data rail"
            )));
        }
    }
    if parsed.outputs.contains(&clock) {
        return Err(HdlError::Unsupported(format!(
            "clock net {clock} is a primary output"
        )));
    }
    if parsed.latches.iter().any(|l| l.input == clock) {
        return Err(HdlError::Unsupported(format!(
            "clock net {clock} feeds a latch D input"
        )));
    }
    Ok(Some(clock))
}

/// Compile BLIF text into a placed, probed, lever-driven PLA build.
///
/// The whole verified pipeline: parse -> fold constants -> dual-rail with QM
/// off-set covers -> peephole -> levelise + buffers -> pack slices -> place
/// rails/inverters/columns/routes/lids.
///
/// Sequential designs (`.latch`, rising-edge, single clock) additionally get
/// a DFF bank stage (the verified master-slave repeater-lock cell), a clock
/// spine with its own lever, D routes raised to the top level, and Q wrap
/// corridors feeding stage-0 input rails; initial state is baked by
/// construction.
pub fn compile_blif(text: &str, name: &str) -> Result<Compiled, HdlError> {
    let parsed = blif::parse_blif(text)?;
    let clock_net = seq_checks(&parsed)?;
    let real_pis: Vec<String> = parsed
        .inputs
        .iter()
        .filter(|i| Some(i.as_str()) != clock_net.as_deref())
        .cloned()
        .collect();
    if real_pis.is_empty() && parsed.latches.is_empty() {
        return Err(HdlError::Unsupported("a design with no inputs".into()));
    }
    if parsed.outputs.is_empty() {
        return Err(HdlError::Unsupported("a design with no outputs".into()));
    }
    let (nodes, consts) = blif::fold(&parsed)?;
    let const_nets = consts.len();
    let q_nets: Vec<String> = parsed.latches.iter().map(|l| l.output.clone()).collect();
    let comp_inputs: Vec<String> = real_pis.iter().chain(q_nets.iter()).cloned().collect();
    let mut comp = logic::Compiler::new(&comp_inputs, &parsed.outputs, nodes, consts);
    comp.mark_external(&q_nets);
    let mut po_val: Vec<(String, Value)> = Vec::new();
    for po in parsed.outputs.clone() {
        let v = comp.value(&po, 1)?; // ('const', b) or a vid
        po_val.push((po, v));
    }
    let mut d_val: Vec<Value> = Vec::new();
    for l in &parsed.latches {
        d_val.push(comp.value(&l.input, 1)?);
    }
    let removed = comp.peephole();
    let resolve = |comp: &logic::Compiler, v: Value| match v {
        Value::Vid(vid) => Value::Vid(comp.resolve(&vid)),
        c => c,
    };
    let po_val: Vec<(String, Value)> = po_val
        .into_iter()
        .map(|(po, v)| (po, resolve(&comp, v)))
        .collect();
    let d_val: Vec<Value> = d_val.into_iter().map(|v| resolve(&comp, v)).collect();
    comp.levelise();
    // D producers rise to the top level: every bank delivery becomes an
    // ordinary next-stage route (far corridors don't exist for gi=1 nodes)
    let top = comp.max_level();
    let d_raised: Vec<Option<String>> = d_val
        .iter()
        .map(|v| match v {
            Value::Vid(vid) => Some(comp.raise_to_level(vid, top)),
            Value::Const(_) => None,
        })
        .collect();
    let mut stages = comp.stages();
    if !parsed.latches.is_empty() {
        let bank0 = stages
            .iter()
            .flat_map(|s| s.nodes.iter().map(|(sl, _)| *sl))
            .max()
            .unwrap_or(0)
            + 1;
        let seq_cells: Vec<seq::SeqCell> = parsed
            .latches
            .iter()
            .enumerate()
            .map(|(k, l)| seq::SeqCell {
                slice: bank0 + k as i32,
                d: d_raised[k].clone(),
                d_const: match &d_val[k] {
                    Value::Const(c) => *c,
                    Value::Vid(_) => 0,
                },
                q_rail: comp.rail_of(&l.output),
                q_slice: comp.slice_of_pi(&l.output),
                init: l.init_bit(),
                label: format!("dff{k}"),
            })
            .collect();
        stages.push(pla::Stage {
            name: "seq".to_string(),
            nodes: Vec::new(),
            local: false,
            inverters: Vec::new(),
            levers: Vec::new(),
            stride: None,
            ext: Vec::new(),
            seq: seq_cells,
        });
    }
    let prims = comp.val_terms.len();
    let levels = comp.level.values().max().copied().unwrap_or(0);

    let mut pla = pla::Pla::new(stages)?;
    pla.build()?;
    let bounds = pla.b.bounds();
    let blocks = pla.b.cells.len();
    let latches: Vec<LatchSpec> = parsed
        .latches
        .iter()
        .enumerate()
        .map(|(k, l)| LatchSpec {
            q: l.output.clone(),
            d: l.input.clone(),
            init: l.init_bit(),
            d_val: d_val[k].clone(),
            q_rail: comp.rail_of(&l.output),
            d_port: pla.seq_ports[k].0,
            q_port: pla.seq_ports[k].1,
        })
        .collect();
    let clock = clock_net.map(|net| {
        let ((bx0, _, bz0), (bx1, _, bz1)) = bounds;
        // DFF floor + comb depth + a wrap-corridor repeater allowance from
        // the build perimeter — an ESTIMATE; verify_clocked measures.
        let wrap = (bx1 - bx0) + 2 * (bz1 - bz0);
        let est = seq::DFF_MIN_PERIOD_GT
            + 4 * u32::try_from(levels.max(0)).unwrap_or(0)
            + 4 * u32::try_from(wrap.max(0)).unwrap_or(0) / 13;
        Clock {
            net,
            lever: pla.clock_lever.expect("a DFF bank always has a clock"),
            est_min_period_gt: est,
        }
    });
    Ok(Compiled {
        name: name.to_string(),
        build: pla.b,
        probes: pla.probe,
        levers: pla.levers,
        inputs: real_pis,
        outputs: po_val,
        latches,
        clock,
        comp,
        stats: Stats {
            prims,
            levels,
            peephole_removed: removed,
            const_nets,
            blocks,
            bounds,
        },
    })
}
