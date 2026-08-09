//! In-sim verification against the mc-tick engine (feature `mc-tick`).
//!
//! Port of the harness in `redstone-eda/rs.py` + `hdl2redstone.py`'s driver:
//! wire the compiled cells into a [`mc_tick::Simulation`] exactly the way
//! nucleation's bridge does (placement pass + ordered settle), then drive the
//! input levers with toggle-to-target discipline — levers only respond to
//! `use_block`, never to a direct state write, and they are flipped one at a
//! time with a settle after each, because a player flips levers one by one —
//! and compare every probe against the pure-Rust prim-graph eval.

use std::collections::{BTreeMap, HashMap};

use mc_tick::{Pos, Simulation};

use crate::pla::Build;
use crate::Compiled;

/// Every state the sim may need to intern beyond the build's own palette.
/// mc-tick binds behaviour to interned states at construction time; a state
/// that only shows up later (a toggled lever, an unlit torch) sits inert.
pub fn extra_states() -> Vec<String> {
    const DIRS: [&str; 4] = ["north", "south", "east", "west"];
    let mut out: Vec<String> = Vec::new();
    for d in DIRS {
        for p in ["true", "false"] {
            out.push(format!("minecraft:lever[face=floor,facing={d},powered={p}]"));
        }
    }
    out.push("minecraft:redstone_torch[lit=true]".into());
    out.push("minecraft:redstone_torch[lit=false]".into());
    for d in DIRS {
        for p in ["true", "false"] {
            out.push(format!("minecraft:redstone_wall_torch[facing={d},lit={p}]"));
        }
    }
    out.push("minecraft:redstone_lamp[lit=true]".into());
    out.push("minecraft:redstone_lamp[lit=false]".into());
    for d in DIRS {
        for dl in [1, 2] {
            for lk in ["true", "false"] {
                for pw in ["true", "false"] {
                    out.push(format!(
                        "minecraft:repeater[facing={d},delay={dl},locked={lk},powered={pw}]"
                    ));
                }
            }
        }
    }
    out
}

/// A simulation addressable in the build's own coordinates.
///
/// The world is placed with the build's min corner at the origin, so probe
/// and lever cells from the compiler address it after offsetting.
pub struct Sim {
    /// The wired engine.
    pub sim: Simulation,
    off: (i32, i32, i32),
}

impl Sim {
    fn p(&self, x: i32, y: i32, z: i32) -> Pos {
        Pos::new(x - self.off.0, y - self.off.1, z - self.off.2)
    }

    /// The block-state descriptor at a build-coordinate cell.
    pub fn block(&self, x: i32, y: i32, z: i32) -> &str {
        let id = self.sim.world().get(self.p(x, y, z));
        self.sim.registry().descriptor(id).unwrap_or("minecraft:air")
    }

    /// The `power=` level at a cell, or -1 if the state has none.
    pub fn power(&self, x: i32, y: i32, z: i32) -> i32 {
        let digits: Option<String> = prop(self.block(x, y, z), "power=").map(|v| {
            v.chars().take_while(char::is_ascii_digit).collect()
        });
        digits.filter(|d| !d.is_empty()).and_then(|d| d.parse().ok()).unwrap_or(-1)
    }

    /// Whether a dust cell reads powered.
    pub fn on(&self, x: i32, y: i32, z: i32) -> bool {
        self.power(x, y, z) > 0
    }

    /// The `powered=` flag at a cell (levers), or None.
    pub fn powered(&self, x: i32, y: i32, z: i32) -> Option<bool> {
        prop(self.block(x, y, z), "powered=").map(|v| v.starts_with("true"))
    }

    /// Right-click a cell (toggle a lever).
    pub fn use_block(&mut self, x: i32, y: i32, z: i32) {
        self.sim.use_block(self.p(x, y, z));
    }

    /// Run until quiet or `budget` ticks pass; true when quiet.
    pub fn settle(&mut self, budget: u64) -> bool {
        self.sim.run_until_quiescent(budget);
        self.sim.is_quiescent()
    }

    /// Run exactly `gt` game ticks — the ONLY stepping clocked verification
    /// uses after the initial placement settle (a clocked design may
    /// oscillate; quiescence is not a fixpoint to wait for).
    pub fn run_gt(&mut self, gt: u64) {
        self.sim.run(gt);
    }
}

fn prop<'a>(descriptor: &'a str, key: &str) -> Option<&'a str> {
    descriptor.split(key).nth(1)
}

/// Build a simulation from authored cells, wired the way nucleation's bridge
/// wires a schematic: place, intern extras + companions, register vanilla
/// behaviours, vanilla placement order, `onPlace` pass, ordered settle, then
/// run to quiescence under `settle_budget`.
pub fn simulate(build: &Build, settle_budget: u64) -> Result<Sim, String> {
    const MARGIN: i32 = 4;
    let (lo, hi) = build.bounds();
    let size = (hi.0 - lo.0 + 1, hi.1 - lo.1 + 1, hi.2 - lo.2 + 1);

    // The cells as an mc_tick Structure, normalised to a zero min corner.
    let mut palette: Vec<String> = Vec::new();
    let mut index: HashMap<&str, usize> = HashMap::new();
    let mut blocks: Vec<(Pos, usize)> = Vec::new();
    for (&(x, y, z), state) in &build.cells {
        let entry = *index.entry(state.as_str()).or_insert_with(|| {
            palette.push(state.clone());
            palette.len() - 1
        });
        blocks.push((Pos::new(x - lo.0, y - lo.1, z - lo.2), entry));
    }
    let structure = mc_tick::Structure {
        size,
        data_version: None,
        palette,
        blocks,
        inventories: Vec::new(),
        inventory_blocked_slots: Vec::new(),
        comparator_outputs: Vec::new(),
        block_entities: Vec::new(),
        commands: Vec::new(),
        entities: Vec::new(),
        item_entities: Vec::new(),
    };

    let mut sim = Simulation::new(structure.bounds(MARGIN));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    let mut wanted: Vec<String> = vec!["minecraft:redstone_block".to_string()];
    wanted.extend(extra_states());
    for descriptor in &wanted {
        sim.registry_mut()
            .intern(descriptor)
            .map_err(|e| format!("interning {descriptor}: {e:?}"))?;
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
        *sim.behaviours_mut() = table;
    }
    if let Some(report) = sim.unknown_report() {
        return Err(format!("blocks without behaviour: {report}"));
    }
    {
        let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
        sim.set_physics_tables(solidity, frictions, heights, webs);
        let (water_kinds, bubble_kinds) = mc_tick::vanilla::fluid_tables(sim.registry());
        sim.set_fluid_tables(water_kinds, bubble_kinds);
        let (rails, conductors) = mc_tick::vanilla::rail_tables(sim.registry());
        sim.set_rail_tables(rails, conductors);
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    sim.settle_with_order(&order);
    sim.record();
    let mut wrapped = Sim { sim, off: lo };
    wrapped.settle(settle_budget);
    Ok(wrapped)
}

/// Toggle-to-target lever driver: track state, flip one lever at a time with
/// a settle after each — flipping several inside one tick injects transients
/// a chain can latch on to.
pub struct Levers {
    positions: Vec<(i32, i32, i32)>,
    state: Vec<bool>,
}

impl Levers {
    /// Read the current lever states out of the settled sim.
    pub fn new(sim: &Sim, positions: Vec<(i32, i32, i32)>) -> Self {
        let state = positions
            .iter()
            .map(|&(x, y, z)| sim.powered(x, y, z).unwrap_or(false))
            .collect();
        Levers { positions, state }
    }

    /// Drive the levers to `bits`; true when every settle went quiet.
    pub fn set(&mut self, sim: &mut Sim, bits: &[bool], settle: u64) -> bool {
        let mut ok = true;
        for (i, &b) in bits.iter().enumerate() {
            if self.state[i] != b {
                let (x, y, z) = self.positions[i];
                sim.use_block(x, y, z);
                self.state[i] = b;
                ok = sim.settle(settle) && ok;
            }
        }
        sim.settle(settle) && ok
    }
}

/// One verification run's outcome.
#[derive(Debug, Clone)]
pub struct VerifyReport {
    /// Cases driven.
    pub cases: usize,
    /// Cases whose primary outputs all matched the model.
    pub outputs_ok: usize,
    /// signal -> how many cases its probe disagreed with the model.
    pub sig_bad: BTreeMap<String, usize>,
}

impl VerifyReport {
    /// True when every case matched on every probe.
    pub fn pass(&self) -> bool {
        self.outputs_ok == self.cases && self.sig_bad.is_empty()
    }
}

/// Drive `cases` (bit `i` of a case = value of `inputs[i]`) through the
/// levers and compare every probe — primary outputs and internal signals —
/// against the prim-graph eval.
pub fn verify(c: &Compiled, cases: &[u64], settle: u64) -> Result<VerifyReport, String> {
    let mut sim = simulate(&c.build, 4000)?;
    let lever_names: Vec<String> = c.levers.iter().map(|(s, _)| s.clone()).collect();
    let mut lv = Levers::new(&sim, c.levers.iter().map(|(_, p)| *p).collect());
    // lever rail name -> PI bit index
    let pi_of_lever: HashMap<String, usize> = c
        .inputs
        .iter()
        .enumerate()
        .map(|(i, net)| (format!("{}.lv", c.comp.vid(net, 1)), i))
        .collect();

    let n = c.inputs.len();
    let mut bad = 0usize;
    let mut sig_bad: BTreeMap<String, usize> = BTreeMap::new();
    for &case in cases {
        let bits: Vec<u8> = (0..n).map(|i| ((case >> i) & 1) as u8).collect();
        let lever_bits: Vec<bool> = lever_names
            .iter()
            .map(|nm| bits[pi_of_lever[nm]] == 1)
            .collect();
        lv.set(&mut sim, &lever_bits, settle);
        let want = c.eval(&bits);
        let mut wrong_po = false;
        for (_po, v) in &c.outputs {
            match v {
                crate::Value::Const(_) => {}
                crate::Value::Vid(vid) => {
                    let w = want[vid];
                    let p = c.probes.get(vid).ok_or_else(|| format!("no probe for {vid}"))?;
                    let g = u8::from(sim.on(p.0, p.1, p.2));
                    if g != w {
                        wrong_po = true;
                    }
                }
            }
        }
        for (s, p) in &c.probes {
            if let Some(&w) = want.get(s) {
                if u8::from(sim.on(p.0, p.1, p.2)) != w {
                    *sig_bad.entry(s.clone()).or_insert(0) += 1;
                }
            }
        }
        if wrong_po {
            bad += 1;
        }
    }
    Ok(VerifyReport {
        cases: cases.len(),
        outputs_ok: cases.len() - bad,
        sig_bad,
    })
}

/// One clocked verification run's outcome.
#[derive(Debug, Clone)]
pub struct ClockedReport {
    /// Rising edges driven.
    pub steps: usize,
    /// Steps whose Q rails and primary outputs all matched the model.
    pub steps_ok: usize,
    /// The at-rest state matched the baked initial state.
    pub init_ok: bool,
    /// Max game ticks any step needed from the last input flip until every
    /// D port matched the model — the measured input-to-edge margin.
    pub measured_setup_gt: u64,
    /// Max game ticks any step needed from the falling edge until every Q
    /// rail and output matched — the measured state-propagation margin.
    pub measured_edge_gt: u64,
    /// The clock high phase used (covers min pulse + spine skew).
    pub high_gt: u64,
    /// Human-readable mismatch notes (first few only).
    pub mismatches: Vec<String>,
}

impl ClockedReport {
    /// True when the initial state and every step matched everywhere.
    pub fn pass(&self) -> bool {
        self.init_ok && self.steps_ok == self.steps && self.mismatches.is_empty()
    }

    /// The measured safe period: setup + high phase + post-edge margin.
    pub fn measured_min_period_gt(&self) -> u64 {
        self.measured_setup_gt + self.high_gt + self.measured_edge_gt
    }
}

/// Drive a SEQUENTIAL design: reset-by-bake (the placement settle converges
/// to the authored initial state — legal because a locked slave cuts every
/// feedback loop at rest), then for each case fixed-tick stepping only:
/// set the input levers, wait for the D ports to match the model (measuring
/// the real input-to-edge margin instead of assuming one), pulse the clock
/// lever high for `high_gt`, and after the falling edge wait for the Q rails
/// and primary outputs to match the stepped model (measuring clk->Q through
/// the wrap corridors). `cap_gt` bounds both waits; a design that cannot
/// settle under the cap fails loudly with a mismatch note.
pub fn verify_clocked(
    c: &Compiled,
    input_seq: &[u64],
    high_gt: u64,
    cap_gt: u64,
) -> Result<ClockedReport, String> {
    let clock = c
        .clock
        .as_ref()
        .ok_or_else(|| "verify_clocked needs a sequential design".to_string())?;
    let mut sim = simulate(&c.build, 4000)?;
    let lever_names: Vec<String> = c.levers.iter().map(|(s, _)| s.clone()).collect();
    let mut lv = Levers::new(&sim, c.levers.iter().map(|(_, p)| *p).collect());
    let pi_of_lever: HashMap<String, usize> = c
        .inputs
        .iter()
        .enumerate()
        .map(|(i, net)| (format!("{}.lv", c.comp.vid(net, 1)), i))
        .collect();
    let n = c.inputs.len();
    let mut mismatches: Vec<String> = Vec::new();
    let note = |m: String, mm: &mut Vec<String>| {
        if mm.len() < 8 {
            mm.push(m);
        }
    };

    // -- reset-by-bake: the settled world must sit at the declared state ----
    let mut state: Vec<u8> = c.latches.iter().map(|l| l.init).collect();
    let mut pi_bits: Vec<u8> = (0..n)
        .map(|i| {
            let (_, p) = c.levers[i];
            u8::from(sim.powered(p.0, p.1, p.2).unwrap_or(false))
        })
        .collect();
    let mut init_ok = true;
    for (k, l) in c.latches.iter().enumerate() {
        let p = c.probes[&l.q_rail];
        if u8::from(sim.on(p.0, p.1, p.2)) != state[k] {
            init_ok = false;
            note(format!("init: {} rail != {}", l.q, state[k]), &mut mismatches);
        }
    }
    let val0 = c.seq_eval(&pi_bits, &state);
    for (po, want) in c.outputs_from(&val0) {
        if let Some(crate::Value::Vid(vid)) = c
            .outputs
            .iter()
            .find(|(o, _)| *o == po)
            .map(|(_, v)| v.clone())
        {
            let p = c.probes[&vid];
            if u8::from(sim.on(p.0, p.1, p.2)) != want {
                init_ok = false;
                note(format!("init: output {po} != {want}"), &mut mismatches);
            }
        }
    }

    let d_expected = |c: &Compiled, val: &HashMap<String, u8>| -> Vec<Option<u8>> {
        c.latches
            .iter()
            .map(|l| match &l.d_val {
                crate::Value::Const(_) => None, // tied at the cell, not probed
                crate::Value::Vid(v) => Some(val[v]),
            })
            .collect()
    };

    let mut steps_ok = 0usize;
    let mut measured_setup_gt = 0u64;
    let mut measured_edge_gt = 0u64;
    for (si, &case) in input_seq.iter().enumerate() {
        // 1. inputs, one lever flip at a time, fixed 2 gt apart
        let bits: Vec<u8> = (0..n).map(|i| ((case >> i) & 1) as u8).collect();
        for (li, nm) in lever_names.iter().enumerate() {
            let want = bits[pi_of_lever[nm]] == 1;
            if lv.state[li] != want {
                let (x, y, z) = lv.positions[li];
                sim.use_block(x, y, z);
                lv.state[li] = want;
                sim.run_gt(2);
            }
        }
        pi_bits = bits;

        // 2. measured input-settle: step until every D port reads the model
        let val = c.seq_eval(&pi_bits, &state);
        let want_d = d_expected(c, &val);
        let mut t = 0u64;
        loop {
            let ok = c.latches.iter().zip(&want_d).all(|(l, w)| match w {
                None => true,
                Some(w) => u8::from(sim.on(l.d_port.0, l.d_port.1, l.d_port.2)) == *w,
            });
            if ok {
                break;
            }
            if t >= cap_gt {
                note(
                    format!("step {si}: D ports never settled under {cap_gt} gt"),
                    &mut mismatches,
                );
                break;
            }
            sim.run_gt(1);
            t += 1;
        }
        measured_setup_gt = measured_setup_gt.max(t);
        sim.run_gt(4); // hold margin past the probe point

        // 3. the edge: clock high for high_gt, then low
        let (cx, cy, cz) = clock.lever;
        sim.use_block(cx, cy, cz);
        sim.run_gt(high_gt);
        sim.use_block(cx, cy, cz);
        state = c.latch_next(&val);

        // 4. measured post-edge settle: Q rails + outputs match the stepped
        //    model
        let val2 = c.seq_eval(&pi_bits, &state);
        let want_po = c.outputs_from(&val2);
        let mut t = 0u64;
        let mut ok;
        loop {
            ok = c.latches.iter().enumerate().all(|(k, l)| {
                let p = c.probes[&l.q_rail];
                u8::from(sim.on(p.0, p.1, p.2)) == state[k]
            }) && want_po.iter().all(|(po, w)| {
                match c.outputs.iter().find(|(o, _)| o == po).map(|(_, v)| v) {
                    Some(crate::Value::Vid(vid)) => {
                        let p = c.probes[vid];
                        u8::from(sim.on(p.0, p.1, p.2)) == *w
                    }
                    _ => true,
                }
            });
            if ok || t >= cap_gt {
                break;
            }
            sim.run_gt(1);
            t += 1;
        }
        measured_edge_gt = measured_edge_gt.max(t);
        sim.run_gt(4);
        if ok {
            steps_ok += 1;
        } else {
            note(
                format!("step {si}: state/outputs diverged (want Q={state:?})"),
                &mut mismatches,
            );
        }
    }
    Ok(ClockedReport {
        steps: input_seq.len(),
        steps_ok,
        init_ok,
        measured_setup_gt,
        measured_edge_gt,
        high_gt,
        mismatches,
    })
}

/// Write every settled non-air state back into the build (the Python `bake`).
/// Returns how many cells changed.
pub fn bake(build: &mut Build, sim: &Sim) -> usize {
    let mut changed = 0;
    let cells: Vec<((i32, i32, i32), String)> = build
        .cells
        .iter()
        .map(|(p, s)| (*p, s.clone()))
        .collect();
    for ((x, y, z), authored) in cells {
        let state = sim.block(x, y, z).to_string();
        if state != authored && !state.contains("air") {
            build.force(x, y, z, &state);
            changed += 1;
        }
    }
    changed
}
