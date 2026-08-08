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
//! - [`verify`] (feature `mc-tick`): exhaustive/sampled in-sim verification
//!   against the pure-Rust prim-graph eval, with lever discipline.
//!
//! The core compile path has zero dependencies and is wasm-clean.

pub mod blif;
pub mod logic;
pub mod pla;
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
    /// Primary inputs, in `.inputs` order (= lever order).
    pub inputs: Vec<String>,
    /// Primary outputs with their resolved value: a constant or a probed vid.
    pub outputs: Vec<(String, Value)>,
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

    /// A JSON report: stats, probes, levers, outputs. Hand-rolled (the crate
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
        let ((x0, y0, z0), (x1, y1, z1)) = self.stats.bounds;
        format!(
            "{{\"name\":\"{}\",\"prims\":{},\"levels\":{},\"peephole_removed\":{},\
             \"const_nets\":{},\"blocks\":{},\
             \"bounds\":[[{x0},{y0},{z0}],[{x1},{y1},{z1}]],\
             \"inputs\":[{}],\"outputs\":[{}],\"levers\":[{}],\"probes\":{{{}}}}}",
            esc(&self.name),
            self.stats.prims,
            self.stats.levels,
            self.stats.peephole_removed,
            self.stats.const_nets,
            self.stats.blocks,
            inputs.join(","),
            outputs.join(","),
            levers.join(","),
            probes.join(",")
        )
    }
}

/// Compile BLIF text into a placed, probed, lever-driven PLA build.
///
/// The whole verified pipeline: parse -> fold constants -> dual-rail with QM
/// off-set covers -> peephole -> levelise + buffers -> pack slices -> place
/// rails/inverters/columns/routes/lids.
pub fn compile_blif(text: &str, name: &str) -> Result<Compiled, HdlError> {
    let parsed = blif::parse_blif(text)?;
    if parsed.inputs.is_empty() {
        return Err(HdlError::Unsupported("a design with no inputs".into()));
    }
    if parsed.outputs.is_empty() {
        return Err(HdlError::Unsupported("a design with no outputs".into()));
    }
    let (nodes, consts) = blif::fold(&parsed)?;
    let const_nets = consts.len();
    let mut comp = logic::Compiler::new(&parsed.inputs, &parsed.outputs, nodes, consts);
    let mut po_val: Vec<(String, Value)> = Vec::new();
    for po in parsed.outputs.clone() {
        let v = comp.value(&po, 1)?; // ('const', b) or a vid
        po_val.push((po, v));
    }
    let removed = comp.peephole();
    let po_val: Vec<(String, Value)> = po_val
        .into_iter()
        .map(|(po, v)| match v {
            Value::Vid(vid) => (po, Value::Vid(comp.resolve(&vid))),
            c => (po, c),
        })
        .collect();
    comp.levelise();
    let stages = comp.stages();
    let prims = comp.val_terms.len();
    let levels = comp.level.values().max().copied().unwrap_or(0);

    let mut pla = pla::Pla::new(stages)?;
    pla.build()?;
    let bounds = pla.b.bounds();
    let blocks = pla.b.cells.len();
    Ok(Compiled {
        name: name.to_string(),
        build: pla.b,
        probes: pla.probe,
        levers: pla.levers,
        inputs: parsed.inputs,
        outputs: po_val,
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
