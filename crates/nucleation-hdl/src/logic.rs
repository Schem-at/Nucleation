//! Dual-rail logic preparation: Quine-McCluskey off-set covers, peephole
//! double-inversion collapse, levelization with buffer insertion, slice
//! packing into PLA stage dicts.
//!
//! Line-for-line port of the `qm` function and `Compiler` class in
//! `redstone-eda/hdl/hdl2redstone.py` — the Python is sim-verified
//! (seg7 16/16, cmp4 256/256, popcnt4 16/16) and is the spec.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::blif::Node;
use crate::pla::Stage;
use crate::HdlError;

/// A lone gi=0 node owns COL_A+COL_B = 3 columns.
pub const MAX_TERMS: usize = 3;

/// Minimal-ish SOP cover of `minterms` over `k` variables.
///
/// Returns `[[(bit, polarity), ...], ...]`; deterministic.
pub fn qm(minterms: &[usize], k: usize) -> Vec<Vec<(usize, u8)>> {
    let mut cur: HashSet<(usize, usize)> = minterms.iter().map(|&m| (m, 0)).collect();
    let mut primes: HashSet<(usize, usize)> = HashSet::new();
    while !cur.is_empty() {
        let mut nxt: HashSet<(usize, usize)> = HashSet::new();
        let mut merged: HashSet<(usize, usize)> = HashSet::new();
        let mut by: HashMap<usize, HashSet<usize>> = HashMap::new();
        for &(v, mask) in &cur {
            by.entry(mask).or_default().insert(v);
        }
        for (&mask, vs) in &by {
            for &v in vs {
                for b in 0..k {
                    if mask & (1 << b) != 0 || (v >> b) & 1 != 0 {
                        continue;
                    }
                    if vs.contains(&(v | (1 << b))) {
                        nxt.insert((v, mask | (1 << b)));
                        merged.insert((v, mask));
                        merged.insert((v | (1 << b), mask));
                    }
                }
            }
        }
        for p in cur.difference(&merged) {
            primes.insert(*p);
        }
        cur = nxt;
    }

    fn covers(p: (usize, usize), m: usize) -> bool {
        let (v, mask) = p;
        (m & !mask) == (v & !mask)
    }

    let mut remaining: HashSet<usize> = minterms.iter().copied().collect();
    let mut chosen: Vec<(usize, usize)> = Vec::new();
    let mut plist: Vec<(usize, usize)> = primes.into_iter().collect();
    plist.sort_unstable_by_key(|&(v, mask)| (mask, v));
    while !remaining.is_empty() {
        // The key is injective over distinct primes (it embeds v and mask), so
        // Python-max-vs-Rust-max tie behaviour cannot differ.
        let (bi, best) = plist
            .iter()
            .enumerate()
            .max_by_key(|&(_, &p)| {
                (
                    remaining.iter().filter(|&&m| covers(p, m)).count(),
                    p.1.count_ones(),
                    Reverse(p.0),
                    Reverse(p.1),
                )
            })
            .map(|(i, &p)| (i, p))
            .expect("primes always cover every minterm");
        chosen.push(best);
        remaining.retain(|&m| !covers(best, m));
        plist.remove(bi);
    }
    chosen
        .iter()
        .map(|&(v, mask)| {
            (0..k)
                .filter(|b| mask & (1 << b) == 0)
                .map(|b| (b, ((v >> b) & 1) as u8))
                .collect()
        })
        .collect()
}

/// A dual-rail value: a compile-time constant or a named prim-graph node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// The net folds to this bit.
    Const(u8),
    /// The prim-graph node (vid) that computes the net at this polarity.
    Vid(String),
}

/// BLIF -> prim graph: dual-rail on demand, buffer levelization, slice packing.
pub struct Compiler {
    /// Primary inputs, in `.inputs` order (this is the lever order).
    pub pis: Vec<String>,
    /// Primary outputs, in `.outputs` order.
    pub pos: Vec<String>,
    nodes: HashMap<String, Node>,
    consts: HashMap<String, u8>,
    base: HashMap<String, String>,
    /// vid -> OR of AND terms (each term a sorted list of vids).
    pub val_terms: BTreeMap<String, Vec<Vec<String>>>,
    /// vid -> level (0 = input-stage rails).
    pub level: HashMap<String, i32>,
    memo: HashMap<(String, u8), Value>,
    pi_slice: HashMap<String, i32>,
    alias: HashMap<String, String>,
}

fn sanitize(net: &str) -> String {
    let b: String = net
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    let b = b.trim_matches('_');
    let mut b = if b.is_empty() { "n".to_string() } else { b.to_string() };
    if b.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        b = format!("n{b}");
    }
    b
}

impl Compiler {
    /// Build a compiler over folded nodes and constants.
    ///
    /// `nodes` must be in deterministic (topo) order — name uniquification
    /// hands out `_2` suffixes in encounter order.
    pub fn new(
        inputs: &[String],
        outputs: &[String],
        nodes: Vec<(String, Node)>,
        consts: HashMap<String, u8>,
    ) -> Self {
        let mut ordered: Vec<&String> = Vec::new();
        ordered.extend(inputs.iter());
        ordered.extend(nodes.iter().map(|(n, _)| n));
        let mut const_names: Vec<&String> = consts.keys().collect();
        const_names.sort();
        ordered.extend(const_names);

        let mut used: HashSet<String> = HashSet::new();
        let mut base: HashMap<String, String> = HashMap::new();
        for net in ordered {
            let b = sanitize(net);
            let mut c = b.clone();
            let mut i = 2;
            while used.contains(&c) {
                c = format!("{b}_{i}");
                i += 1;
            }
            used.insert(c.clone());
            base.insert(net.clone(), c);
        }

        let mut level = HashMap::new();
        let mut pi_slice = HashMap::new();
        let node_map: HashMap<String, Node> = nodes.into_iter().collect();
        let mut comp = Compiler {
            pis: inputs.to_vec(),
            pos: outputs.to_vec(),
            nodes: node_map,
            consts,
            base,
            val_terms: BTreeMap::new(),
            level: HashMap::new(),
            memo: HashMap::new(),
            pi_slice: HashMap::new(),
            alias: HashMap::new(),
        };
        for (i, net) in comp.pis.clone().iter().enumerate() {
            for pol in [0u8, 1u8] {
                let vid = comp.vid(net, pol);
                level.insert(vid.clone(), 0);
                pi_slice.insert(vid, i as i32);
            }
        }
        comp.level = level;
        comp.pi_slice = pi_slice;
        comp
    }

    /// The prim-graph name of `net` at polarity `pol` (`.n` = complement rail).
    pub fn vid(&self, net: &str, pol: u8) -> String {
        let b = &self.base[net];
        if pol == 1 {
            b.clone()
        } else {
            format!("{b}.n")
        }
    }

    /// Construct (on demand) the dual-rail value of `net` at polarity `pol`.
    pub fn value(&mut self, net: &str, pol: u8) -> Result<Value, HdlError> {
        let key = (net.to_string(), pol);
        if let Some(v) = self.memo.get(&key) {
            return Ok(v.clone());
        }
        if let Some(&c) = self.consts.get(net) {
            let r = Value::Const(if pol == 1 { c } else { 1 - c });
            self.memo.insert(key, r.clone());
            return Ok(r);
        }
        if self.pis.iter().any(|p| p == net) {
            let r = Value::Vid(self.vid(net, pol));
            self.memo.insert(key, r.clone());
            return Ok(r);
        }
        let (ins, tt, k) = self
            .nodes
            .get(net)
            .cloned()
            .ok_or_else(|| HdlError::Parse(format!("no driver for net {net}")))?;
        let want: Vec<usize> = (0..1usize << k)
            .filter(|&m| u8::from(tt[m]) == pol)
            .collect();
        let cover = qm(&want, k);
        let vid = self.vid(net, pol);
        self.memo.insert(key, Value::Vid(vid.clone()));
        let mut terms: Vec<Vec<String>> = Vec::new();
        for term in cover {
            let mut lits: Vec<String> = Vec::new();
            for (b, p) in term {
                match self.value(&ins[b], p)? {
                    Value::Vid(u) => lits.push(u),
                    Value::Const(_) => {
                        return Err(HdlError::Parse(format!(
                            "constant literal survived folding in {net}"
                        )))
                    }
                }
            }
            lits.sort();
            lits.dedup();
            terms.push(lits);
        }
        self.emit(&vid, terms, 0);
        Ok(Value::Vid(vid))
    }

    fn emit(&mut self, vid: &str, terms: Vec<Vec<String>>, depth: usize) {
        if terms.len() <= MAX_TERMS {
            self.val_terms.insert(vid.to_string(), terms);
            return;
        }
        let mut parts: Vec<String> = Vec::new();
        for (ci, chunk) in terms.chunks(MAX_TERMS).enumerate() {
            let pv = format!("{vid}.d{depth}p{ci}");
            self.val_terms.insert(pv.clone(), chunk.to_vec());
            parts.push(pv);
        }
        self.emit(vid, parts.into_iter().map(|p| vec![p]).collect(), depth + 1);
    }

    /// Alias away pure buffer nodes: `v = OR-of-AND over a single literal` is
    /// just that literal. This is where double inversions die — dual-rail
    /// construction turns a BLIF NOT into a buffer of the complement rail, so
    /// NOT(NOT(x)) chains arrive here as buffer chains and collapse to
    /// nothing. Must run BEFORE `levelise`: the buffers levelise inserts are
    /// structural, not redundant. Returns how many nodes were removed.
    pub fn peephole(&mut self) -> usize {
        self.alias.clear();
        for (v, terms) in &self.val_terms {
            if terms.len() == 1 && terms[0].len() == 1 {
                self.alias.insert(v.clone(), terms[0][0].clone());
            }
        }
        let dead: Vec<String> = self.alias.keys().cloned().collect();
        for v in &dead {
            self.val_terms.remove(v);
        }
        let keys: Vec<String> = self.val_terms.keys().cloned().collect();
        for v in keys {
            let terms: Vec<Vec<String>> = self.val_terms[&v]
                .iter()
                .map(|t| {
                    let mut s: Vec<String> = t.iter().map(|u| self.resolve(u)).collect();
                    s.sort();
                    s.dedup();
                    s
                })
                .collect();
            self.val_terms.insert(v, terms);
        }
        dead.len()
    }

    /// Follow buffer aliases to the surviving vid.
    pub fn resolve(&self, v: &str) -> String {
        let mut v = v.to_string();
        while let Some(n) = self.alias.get(&v) {
            v = n.clone();
        }
        v
    }

    /// Assign every prim `level = 1 + max(level of inputs)` and insert buffer
    /// nodes so every net crosses exactly ONE stage boundary.
    pub fn levelise(&mut self) {
        fn lv(
            v: &str,
            val_terms: &BTreeMap<String, Vec<Vec<String>>>,
            level: &mut HashMap<String, i32>,
        ) -> i32 {
            if let Some(&l) = level.get(v) {
                return l;
            }
            let m = val_terms[v]
                .iter()
                .flatten()
                .map(|u| lv(u, val_terms, level))
                .max()
                .expect("a prim node always has at least one literal");
            level.insert(v.to_string(), 1 + m);
            1 + m
        }
        let keys: Vec<String> = self.val_terms.keys().cloned().collect();
        for v in &keys {
            lv(v, &self.val_terms, &mut self.level);
        }
        // vid -> highest level a buffer is needed at
        let mut need: BTreeMap<String, i32> = BTreeMap::new();
        for (v, terms) in &self.val_terms {
            for t in terms {
                for u in t {
                    if self.level[u] < self.level[v] - 1 {
                        let e = need.entry(u.clone()).or_insert(0);
                        *e = (*e).max(self.level[v] - 1);
                    }
                }
            }
        }
        for (u, top) in need {
            let mut prev = u.clone();
            for l in self.level[&u] + 1..=top {
                let b = format!("{u}.b{l}");
                self.val_terms.insert(b.clone(), vec![vec![prev]]);
                self.level.insert(b.clone(), l);
                prev = b;
            }
        }
        let keys: Vec<String> = self.val_terms.keys().cloned().collect();
        for v in keys {
            let lvv = self.level[&v];
            let terms: Vec<Vec<String>> = self.val_terms[&v]
                .iter()
                .map(|t| {
                    t.iter()
                        .map(|u| {
                            if self.level[u] == lvv - 1 {
                                u.clone()
                            } else {
                                format!("{u}.b{}", lvv - 1)
                            }
                        })
                        .collect()
                })
                .collect();
            self.val_terms.insert(v, terms);
        }
    }

    /// Pack levelized prims into PLA stages: nodes go 2/slice (gi0 <= 2 terms
    /// + gi1 1 term) where possible; rails only conduct west->east, so a
    /// node's slice is forced >= the slice of every producer it taps.
    pub fn stages(&self) -> Vec<Stage> {
        let mut slice_of: HashMap<String, i32> = self.pi_slice.clone();
        let mut by_level: BTreeMap<i32, Vec<String>> = BTreeMap::new();
        for v in self.val_terms.keys() {
            by_level.entry(self.level[v]).or_default().push(v.clone());
        }

        let mut stages = vec![self.input_stage()];
        let max_l = by_level.keys().max().copied().unwrap_or(0);
        for l in 1..=max_l {
            let mut todo: Vec<String> = by_level.get(&l).cloned().unwrap_or_default();
            let minreq: HashMap<String, i32> = todo
                .iter()
                .map(|v| {
                    let m = self.val_terms[v]
                        .iter()
                        .flatten()
                        .map(|u| slice_of[u])
                        .max()
                        .unwrap_or(0);
                    (v.clone(), m)
                })
                .collect();
            todo.sort_by_key(|v| (minreq[v], Reverse(self.val_terms[v].len()), v.clone()));
            let mut nodes: Vec<(i32, Vec<(String, Vec<Vec<String>>)>)> = Vec::new();
            let mut cur = 0i32;
            while !todo.is_empty() {
                let a = todo.remove(0);
                let sl = minreq[&a].max(cur);
                let na = self.val_terms[&a].len();
                let mut partner: Option<String> = None;
                if na <= 2 {
                    // a rides gi0, find a 1-term gi1
                    let best = todo
                        .iter()
                        .filter(|b| self.val_terms[*b].len() == 1 && minreq[*b] <= sl)
                        .max_by_key(|b| (minreq[*b], (*b).clone()))
                        .cloned();
                    if let Some(b) = best {
                        todo.retain(|x| x != &b);
                        partner = Some(b);
                    }
                }
                let mut group = vec![(a.clone(), self.val_terms[&a].clone())];
                if let Some(p) = &partner {
                    group.push((p.clone(), self.val_terms[p].clone()));
                }
                for (v, _t) in &group {
                    slice_of.insert(v.clone(), sl);
                }
                nodes.push((sl, group));
                cur = sl + 1;
            }
            stages.push(Stage {
                name: format!("lv{l}"),
                nodes,
                local: false,
                inverters: Vec::new(),
                levers: Vec::new(),
                stride: None,
                ext: Vec::new(),
            });
        }
        stages
    }

    /// Stage 0: one slice per PI, lever-driven rail, inverter torch making the
    /// complement rail, and the two rail-buffer PLA nodes `p` / `p.n`.
    pub fn input_stage(&self) -> Stage {
        let mut nodes = Vec::new();
        let mut invs = Vec::new();
        let mut levers = Vec::new();
        for (i, net) in self.pis.iter().enumerate() {
            let p = self.vid(net, 1);
            let n = self.vid(net, 0);
            let rail = format!("{p}.lv");
            let railn = format!("{p}.lvn");
            nodes.push((
                i as i32,
                vec![
                    (p, vec![vec![rail.clone()]]),
                    (n, vec![vec![railn.clone()]]),
                ],
            ));
            invs.push((i as i32, rail.clone(), railn));
            levers.push((i as i32, rail, 0usize));
        }
        Stage {
            name: "in".to_string(),
            nodes,
            local: true,
            inverters: invs,
            levers,
            stride: None,
            ext: Vec::new(),
        }
    }

    /// Reference model: evaluate the prim graph for one input assignment.
    pub fn eval(&self, bits: &[u8]) -> HashMap<String, u8> {
        let mut val: HashMap<String, u8> = HashMap::new();
        for (i, net) in self.pis.iter().enumerate() {
            let p = self.vid(net, 1);
            val.insert(format!("{p}.n"), 1 - bits[i]);
            val.insert(format!("{p}.lv"), bits[i]);
            val.insert(format!("{p}.lvn"), 1 - bits[i]);
            val.insert(p, bits[i]);
        }
        let mut keys: Vec<&String> = self.val_terms.keys().collect();
        keys.sort_by_key(|v| (self.level[*v], (*v).clone()));
        for v in keys {
            let r = self.val_terms[v]
                .iter()
                .any(|t| t.iter().all(|u| val[u] == 1));
            val.insert(v.clone(), u8::from(r));
        }
        val
    }
}
