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
