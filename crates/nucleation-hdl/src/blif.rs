//! BLIF subset parser + constant folding.
//!
//! Port of `redstone-eda/hdl/hdl2redstone.py` (`parse_blif`, `tt_of`, `fold`)
//! — that file is the verified spec; every rule here mirrors it line for line.
//!
//! Combinational only: `.latch` / `.subckt` / `.gate` / `.mlatch` are rejected
//! with a clear error. `.names` covers become truth tables; constants are
//! propagated and duplicate/constant inputs restricted away.

use std::collections::HashMap;

use crate::HdlError;

/// One `.names` node as read from the file: input nets and cover rows.
#[derive(Debug, Clone)]
pub struct RawNode {
    /// Input net names, in file order (`tok[1..-1]`).
    pub inputs: Vec<String>,
    /// Cover rows `(pattern, output-bit)`. A 0-input node's row is `("", bit)`.
    pub rows: Vec<(String, String)>,
}

/// A parsed BLIF model: primary inputs/outputs plus `.names` nodes in file order.
#[derive(Debug, Clone)]
pub struct Blif {
    /// `.inputs`, in file order.
    pub inputs: Vec<String>,
    /// `.outputs`, in file order.
    pub outputs: Vec<String>,
    /// `.names` nodes keyed by output net, preserving file order.
    pub nodes: Vec<(String, RawNode)>,
}

impl Blif {
    fn node_index(&self) -> HashMap<&str, usize> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, (n, _))| (n.as_str(), i))
            .collect()
    }
}

/// Parse a BLIF text into inputs, outputs and raw `.names` nodes.
///
/// Comment stripping (`#`), line continuations (`\`), and the "any other dot
/// directive ends the current cover" rule match the Python parser exactly.
pub fn parse_blif(text: &str) -> Result<Blif, HdlError> {
    let mut lines: Vec<String> = Vec::new();
    let mut pend = String::new();
    for raw in text.lines() {
        let raw = raw.split('#').next().unwrap_or("").trim_end();
        if let Some(stripped) = raw.strip_suffix('\\') {
            pend.push_str(stripped);
            pend.push(' ');
            continue;
        }
        let line = format!("{pend}{raw}").trim().to_string();
        pend.clear();
        if !line.is_empty() {
            lines.push(line);
        }
    }

    let mut inputs: Vec<String> = Vec::new();
    let mut outputs: Vec<String> = Vec::new();
    let mut nodes: Vec<(String, RawNode)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut cur: Option<usize> = None;
    for line in &lines {
        let tok: Vec<&str> = line.split_whitespace().collect();
        match tok[0] {
            ".model" | ".end" => cur = None,
            ".inputs" => {
                inputs.extend(tok[1..].iter().map(|s| s.to_string()));
                cur = None;
            }
            ".outputs" => {
                outputs.extend(tok[1..].iter().map(|s| s.to_string()));
                cur = None;
            }
            ".names" => {
                let out = tok[tok.len() - 1].to_string();
                let node = RawNode {
                    inputs: tok[1..tok.len() - 1].iter().map(|s| s.to_string()).collect(),
                    rows: Vec::new(),
                };
                if let Some(&i) = index.get(&out) {
                    nodes[i] = (out, node); // same overwrite the Python dict does
                    cur = Some(i);
                } else {
                    index.insert(out.clone(), nodes.len());
                    cur = Some(nodes.len());
                    nodes.push((out, node));
                }
            }
            ".latch" | ".subckt" | ".gate" | ".mlatch" => {
                return Err(HdlError::Unsupported(format!(
                    "combinational only: found {}",
                    tok[0]
                )));
            }
            t if t.starts_with('.') => cur = None, // .default_input_arrival etc
            _ => {
                if let Some(i) = cur {
                    if tok.len() == 1 {
                        // 0-input node: row is just the out bit
                        nodes[i].1.rows.push((String::new(), tok[0].to_string()));
                    } else {
                        nodes[i].1.rows.push((tok[0].to_string(), tok[1].to_string()));
                    }
                }
            }
        }
    }
    Ok(Blif { inputs, outputs, nodes })
}

/// Truth table over `2**k` input assignments; entry `m` is the value when bit
/// `i` of `m` is the value of `ins[i]`.
pub fn tt_of(ins_len: usize, rows: &[(String, String)]) -> Result<(Vec<bool>, usize), HdlError> {
    let k = ins_len;
    if k > 16 {
        return Err(HdlError::Unsupported(format!(
            ".names with {k} inputs (max 16)"
        )));
    }
    let size = 1usize << k;
    if rows.is_empty() {
        return Ok((vec![false; size], k));
    }
    let mut outs: Vec<&str> = rows.iter().map(|(_p, o)| o.as_str()).collect();
    outs.sort_unstable();
    outs.dedup();
    if outs.len() != 1 {
        return Err(HdlError::Parse("mixed-polarity .names cover".into()));
    }
    let onset = outs == ["1"];
    let mut tt = vec![false; size];
    for (m, slot) in tt.iter_mut().enumerate() {
        for (pat, _o) in rows {
            let ok = pat.chars().enumerate().all(|(i, c)| {
                c == '-' || (c.to_digit(10).unwrap_or(2) as usize) == ((m >> i) & 1)
            });
            if ok {
                *slot = true;
                break;
            }
        }
    }
    if !onset {
        for v in tt.iter_mut() {
            *v = !*v;
        }
    }
    Ok((tt, k))
}

/// A folded logic node: unique non-constant inputs, truth table, arity.
pub type Node = (Vec<String>, Vec<bool>, usize);

/// Topo-order nodes, collapse duplicate inputs, propagate constants.
///
/// Returns surviving nodes (in topo order) and the constant nets.
pub fn fold(
    blif: &Blif,
) -> Result<(Vec<(String, Node)>, HashMap<String, u8>), HdlError> {
    let index = blif.node_index();
    // Topo order over raw nodes, in file order, with cycle detection.
    let mut order: Vec<usize> = Vec::new();
    let mut seen: Vec<u8> = vec![0; blif.nodes.len()];
    fn visit(
        i: usize,
        blif: &Blif,
        index: &HashMap<&str, usize>,
        seen: &mut Vec<u8>,
        order: &mut Vec<usize>,
    ) -> Result<(), HdlError> {
        if seen[i] == 2 {
            return Ok(());
        }
        if seen[i] == 1 {
            return Err(HdlError::Parse(format!(
                "combinational loop through {}",
                blif.nodes[i].0
            )));
        }
        seen[i] = 1;
        for s in &blif.nodes[i].1.inputs {
            if let Some(&j) = index.get(s.as_str()) {
                visit(j, blif, index, seen, order)?;
            }
        }
        seen[i] = 2;
        order.push(i);
        Ok(())
    }
    for i in 0..blif.nodes.len() {
        visit(i, blif, &index, &mut seen, &mut order)?;
    }

    let pis: std::collections::HashSet<&str> = blif.inputs.iter().map(|s| s.as_str()).collect();
    let mut consts: HashMap<String, u8> = HashMap::new();
    let mut nodes: Vec<(String, Node)> = Vec::new();
    for &i in &order {
        let (name, raw) = &blif.nodes[i];
        for s in &raw.inputs {
            if !pis.contains(s.as_str()) && !index.contains_key(s.as_str()) {
                return Err(HdlError::Unsupported(format!(
                    "undriven net {s} feeding {name}"
                )));
            }
        }
        let (mut tt, mut k) = tt_of(raw.inputs.len(), &raw.rows)?;
        let mut ins: Vec<String> = raw.inputs.clone();

        // collapse duplicate inputs
        let mut uniq: Vec<String> = Vec::new();
        for s in &ins {
            if !uniq.contains(s) {
                uniq.push(s.clone());
            }
        }
        if uniq.len() < ins.len() {
            let pos: Vec<usize> = ins
                .iter()
                .map(|s| uniq.iter().position(|u| u == s).unwrap())
                .collect();
            let mut ntt = vec![false; 1 << uniq.len()];
            for (m, slot) in ntt.iter_mut().enumerate() {
                let mut full = 0usize;
                for (i, p) in pos.iter().enumerate().take(k) {
                    full |= ((m >> p) & 1) << i;
                }
                *slot = tt[full];
            }
            ins = uniq;
            k = ins.len();
            tt = ntt;
        }

        // restrict away constant inputs
        let free: Vec<usize> = (0..ins.len())
            .filter(|&i| !consts.contains_key(&ins[i]))
            .collect();
        if free.len() < k {
            let mut base = 0usize;
            for (i, s) in ins.iter().enumerate().take(k) {
                if let Some(&c) = consts.get(s) {
                    base += (c as usize) << i;
                }
            }
            let mut ntt = vec![false; 1 << free.len()];
            for (m, slot) in ntt.iter_mut().enumerate() {
                let mut full = base;
                for (j, f) in free.iter().enumerate() {
                    full += ((m >> j) & 1) << f;
                }
                *slot = tt[full];
            }
            ins = free.iter().map(|&i| ins[i].clone()).collect();
            k = free.len();
            tt = ntt;
        }

        if k == 0 {
            consts.insert(name.clone(), u8::from(tt[0]));
        } else if tt.iter().all(|&b| !b) {
            consts.insert(name.clone(), 0);
        } else if tt.iter().all(|&b| b) {
            consts.insert(name.clone(), 1);
        } else {
            nodes.push((name.clone(), (ins, tt, k)));
        }
    }
    Ok((nodes, consts))
}
