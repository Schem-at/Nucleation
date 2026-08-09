//! Routability harness: how often does `Design::route_bus` actually route?
//!
//! The EDA Studio reports "a lot of invalid busses". This harness mimics
//! realistic studio usage — endpoint hardware on the loose layer, real cells
//! from `computational_schematics/enhanced/` placed at assorted positions and
//! rotations, buses declared between them across many geometries — and prints
//! a PASS/FAIL line per bus WITH THE FAILURE REASON, then a routability rate
//! and a reason histogram.
//!
//! Run it as a report:
//!
//! ```text
//! cargo test --features routing --test design_routability -- --nocapture
//! ```
//!
//! `routability_rate_holds_the_line` is the regression gate; the rest of the
//! file is the measurement instrument.

#![cfg(feature = "routing")]

use nucleation::design::{BusState, BusStyle, Design};
use nucleation::io_contract::IoType;
use nucleation::UniversalSchematic;
use std::collections::BTreeMap;

type P3 = (i32, i32, i32);

const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";

/// `width` levers stacked at 2y pitch, each with its connection dust one step
/// toward the field. Returns the bit-0 connection cell.
fn lever_bank(s: &mut UniversalSchematic, x: i32, y0: i32, z: i32, dx: i32, dz: i32, width: u8) -> P3 {
    for i in 0..width as i32 {
        let y = y0 + 2 * i;
        s.set_block_from_string(x, y - 1, z, STONE).unwrap();
        s.set_block_from_string(x, y, z, LEVER).unwrap();
        s.set_block_from_string(x + dx, y - 1, z + dz, STONE).unwrap();
        s.set_block_from_string(x + dx, y, z + dz, DUST).unwrap();
    }
    (x + dx, y0, z + dz)
}

/// `width` lamps stacked at 2y pitch, each lamp supporting its own dust.
fn lamp_bank(s: &mut UniversalSchematic, x: i32, y0: i32, z: i32, width: u8) -> P3 {
    for i in 0..width as i32 {
        let y = y0 + 2 * i;
        s.set_block_from_string(x, y - 1, z, LAMP).unwrap();
        s.set_block_from_string(x, y, z, DUST).unwrap();
    }
    (x, y0, z)
}

// ----------------------------------------------------------------------
// Recording
// ----------------------------------------------------------------------

struct Attempt {
    scenario: &'static str,
    bus: String,
    reason: Option<String>,
}

#[derive(Default)]
struct Report {
    attempts: Vec<Attempt>,
    /// Scenarios whose routed result did NOT come back DRC/LVS clean. A
    /// router that "routes" by laying illegal geometry is not a fix, so this
    /// list has to stay empty.
    dirty: Vec<(&'static str, String)>,
}

impl Report {
    /// Verify the whole design after a scenario's buses are in. Routability
    /// means nothing if the geometry does not pass the rule checks.
    ///
    /// Compared against `baseline`, the violation count BEFORE any bus was
    /// routed: several enhanced-corpus cells are not DRC-clean standing alone
    /// (floating dust inside their own bodies), and those are cell-authoring
    /// defects, not router output. Only violations the router ADDS count.
    fn verify(&mut self, d: &Design, scenario: &'static str, baseline: usize) {
        match d.check() {
            Ok(c) if c.clean => {}
            Ok(c) => {
                let n = violations(&c.json);
                if n > baseline {
                    self.dirty.push((
                        scenario,
                        format!("{} new violation(s) over baseline {baseline}: {}", n - baseline, one_line(&c.json)),
                    ));
                }
            }
            Err(e) => self.dirty.push((scenario, format!("check() failed: {e}"))),
        }
    }

    /// Attempt one bus and record the outcome. Declaration errors (`Err`) are
    /// recorded too — from the studio's point of view a bus that refuses to
    /// exist is just as invalid as one that fails to route.
    fn bus(
        &mut self,
        d: &mut Design,
        scenario: &'static str,
        name: &str,
        driver: &str,
        sinks: &[&str],
    ) {
        let reason = match d.route_bus(name, driver, sinks, vec![], BusStyle::default()) {
            Ok(BusState::Routed) => None,
            Ok(BusState::Failed(r)) => Some(r),
            Ok(other) => Some(format!("unexpected state {other:?}")),
            Err(e) => Some(format!("declaration refused: {e}")),
        };
        self.attempts.push(Attempt {
            scenario,
            bus: name.to_string(),
            reason,
        });
    }

    fn routed(&self) -> usize {
        self.attempts.iter().filter(|a| a.reason.is_none()).count()
    }

    fn print(&self) {
        println!("\n=== per-bus outcomes ===");
        let mut scenario = "";
        for a in &self.attempts {
            if a.scenario != scenario {
                scenario = a.scenario;
                println!("\n-- {scenario}");
            }
            match &a.reason {
                None => println!("  PASS  {}", a.bus),
                Some(r) => println!("  FAIL  {}  :: {}", a.bus, one_line(r)),
            }
        }

        let mut hist: BTreeMap<&'static str, usize> = BTreeMap::new();
        for a in &self.attempts {
            if let Some(r) = &a.reason {
                *hist.entry(category(r)).or_default() += 1;
            }
        }
        let total = self.attempts.len();
        let ok = self.routed();
        println!("\n=== failure categories ===");
        let mut rows: Vec<_> = hist.iter().collect();
        rows.sort_by_key(|(cat, n)| (std::cmp::Reverse(**n), **cat));
        for (cat, n) in rows {
            println!("  {n:3}  {cat}");
        }
        println!("\n=== ROUTABILITY ===");
        println!(
            "  {ok}/{total} routed = {:.1}%",
            100.0 * ok as f64 / total.max(1) as f64
        );
        if self.dirty.is_empty() {
            println!("  DRC + LVS clean on every scenario");
        } else {
            println!("\n=== NOT CLEAN ===");
            for (s, why) in &self.dirty {
                println!("  {s}: {why}");
            }
        }

        // Every remaining failure must be ACTIONABLE: the reason has to name
        // a cause and a location, not a bare "no path".
        let vague: Vec<&Attempt> = self
            .attempts
            .iter()
            .filter(|a| a.reason.as_deref().is_some_and(|r| category(r) == "other"))
            .collect();
        if !vague.is_empty() {
            println!("\n=== UNCATEGORIZED (reason strings needing work) ===");
            for a in vague {
                println!("  {}  :: {}", a.bus, one_line(a.reason.as_deref().unwrap()));
            }
        }
    }
}

/// Number of GATING rule violations a `check()` report carries — the `drc`
/// array only. Findings inside a placed library cell live under `cells` and are
/// informational: a cell is a verified black box, and hand-built community
/// redstone breaks the route-oriented conventions by design.
fn violations(json: &str) -> usize {
    let Some(start) = json.find("\"drc\":[") else {
        return 0;
    };
    let rest = &json[start + 7..];
    let end = rest.find("],\"cells\"").unwrap_or(rest.len());
    rest[..end].matches("\"kind\"").count()
}

/// The gating violation count a design already carries before any bus is routed.
fn drc_baseline(d: &Design) -> usize {
    d.check().map(|c| violations(&c.json)).unwrap_or(0)
}

fn one_line(s: &str) -> String {
    let flat: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.len() > 200 {
        format!("{}...", &flat[..200])
    } else {
        flat
    }
}

/// Bucket a failure reason. Anything landing in `other` is a reason string
/// that does not yet tell the user what to do.
fn category(r: &str) -> &'static str {
    const RULES: &[(&str, &str)] = &[
        ("endpoint approach blocked", "endpoint escape: port has no free lane"),
        ("no corridor", "no corridor (blocked after search)"),
        ("collision", "collision: obstacle in the corridor"),
        ("influence halo", "halo/keepout rejection"),
        ("SINGLE-LEVEL", "endpoints on different levels"),
        ("levels/widths differ", "crossing level/width mismatch"),
        ("crossing windows", "crossing windows too close"),
        ("no trunk run aligns", "branch: no perpendicular shot"),
        ("did not survive as plain dust", "branch: junction eaten"),
        ("unsupported bus form", "port form/step mismatch"),
        ("does not match the bus form", "gate form/step mismatch"),
        ("zero length", "degenerate geometry"),
        ("internal plan conflict", "planner bug: plan conflict"),
        ("width", "width/type mismatch"),
        ("type", "width/type mismatch"),
        ("cannot drive", "endpoint direction misuse"),
        ("cannot be a sink", "endpoint direction misuse"),
        ("no dust", "endpoint not connectable"),
        ("unknown port", "unknown port"),
    ];
    for (needle, cat) in RULES {
        if r.contains(needle) {
            return cat;
        }
    }
    "other"
}

// ----------------------------------------------------------------------
// Scenarios
// ----------------------------------------------------------------------

fn ty8() -> IoType {
    IoType::UnsignedInt { bits: 8 }
}

/// Straight and dogleg pairs over assorted spans on one level — the
/// baseline "it should obviously work" set.
fn scenario_open_field(rep: &mut Report) {
    for (i, (span, dz)) in [(8i32, 0i32), (24, 0), (64, 0), (120, 0), (24, 12), (64, -20), (16, 40)]
        .iter()
        .enumerate()
    {
        let mut s = UniversalSchematic::new("open".to_string());
        let drv = lever_bank(&mut s, 0, 2, 8, 1, 0, 8);
        let snk = lamp_bank(&mut s, span + 4, 2, 8 + dz, 8);
        let mut d = Design::for_schematic("open", s);
        let step = (0, 2, 0);
        d.declare_input("din", drv, step, 8, ty8()).unwrap();
        d.declare_output("dout", snk, step, 8, ty8()).unwrap();
        rep.bus(&mut d, "open field (straight + dogleg spans)", &format!("open{i}"), "din", &["dout"]);
    }
}

/// A wall across the corridor with a gap: the classic "route around it" case.
/// The gap is always wide enough for the bus form, so a real router finds it.
fn scenario_walled(rep: &mut Report) {
    for (i, gap_z) in [20i32, -12, 30].iter().enumerate() {
        let mut s = UniversalSchematic::new("wall".to_string());
        let drv = lever_bank(&mut s, 0, 2, 8, 1, 0, 8);
        let snk = lamp_bank(&mut s, 40, 2, 8, 8);
        // A tall stone wall at x=20 spanning z, with a 12-cell gap at gap_z.
        for z in -40..=60 {
            if (z - gap_z).abs() <= 6 {
                continue;
            }
            for y in 0..=20 {
                s.set_block_from_string(20, y, z, STONE).unwrap();
            }
        }
        let mut d = Design::for_schematic("wall", s);
        let step = (0, 2, 0);
        d.declare_input("din", drv, step, 8, ty8()).unwrap();
        d.declare_output("dout", snk, step, 8, ty8()).unwrap();
        rep.bus(&mut d, "wall with a gap (detour required)", &format!("wall{i}"), "din", &["dout"]);
        rep.verify(&d, "wall with a gap (detour required)", 0);
    }
}

/// Orthogonal bus pairs that must cross, several at a time.
fn scenario_crossings(rep: &mut Report) {
    let mut s = UniversalSchematic::new("cross".to_string());
    let mut names = Vec::new();
    for k in 0..3i32 {
        let z = 8 + 16 * k;
        let a_in = lever_bank(&mut s, 0, 2, z, 1, 0, 8);
        let a_out = lamp_bank(&mut s, 56, 2, z, 8);
        names.push((format!("row{k}"), a_in, a_out));
    }
    for k in 0..3i32 {
        let x = 12 + 16 * k;
        let b_in = lever_bank(&mut s, x, 2, -8, 0, 1, 8);
        let b_out = lamp_bank(&mut s, x, 2, 48, 8);
        names.push((format!("col{k}"), b_in, b_out));
    }
    let mut d = Design::for_schematic("cross", s);
    let step = (0, 2, 0);
    for (n, i, o) in &names {
        d.declare_input(format!("{n}_in"), *i, step, 8, ty8()).unwrap();
        d.declare_output(format!("{n}_out"), *o, step, 8, ty8()).unwrap();
    }
    for (n, _, _) in &names {
        let din = format!("{n}_in");
        let dout = format!("{n}_out");
        rep.bus(&mut d, "3x3 orthogonal crossings", n, &din, &[dout.as_str()]);
    }
    rep.verify(&d, "3x3 orthogonal crossings", 0);
}

/// One driver, several sinks (fanout branches).
fn scenario_fanout(rep: &mut Report) {
    let mut s = UniversalSchematic::new("fan".to_string());
    let drv = lever_bank(&mut s, 0, 2, 8, 1, 0, 8);
    let s1 = lamp_bank(&mut s, 48, 2, 8, 8);
    let s2 = lamp_bank(&mut s, 20, 2, 28, 8);
    let s3 = lamp_bank(&mut s, 32, 2, -16, 8);
    let mut d = Design::for_schematic("fan", s);
    let step = (0, 2, 0);
    d.declare_input("din", drv, step, 8, ty8()).unwrap();
    d.declare_output("o1", s1, step, 8, ty8()).unwrap();
    d.declare_output("o2", s2, step, 8, ty8()).unwrap();
    d.declare_output("o3", s3, step, 8, ty8()).unwrap();
    rep.bus(&mut d, "fanout (2 and 3 sinks)", "fan2", "din", &["o1", "o2"]);
    let mut d2 = {
        let mut s = UniversalSchematic::new("fan".to_string());
        let drv = lever_bank(&mut s, 0, 2, 8, 1, 0, 8);
        let s1 = lamp_bank(&mut s, 48, 2, 8, 8);
        let s2 = lamp_bank(&mut s, 20, 2, 28, 8);
        let s3 = lamp_bank(&mut s, 32, 2, -16, 8);
        let mut d = Design::for_schematic("fan", s);
        d.declare_input("din", drv, step, 8, ty8()).unwrap();
        d.declare_output("o1", s1, step, 8, ty8()).unwrap();
        d.declare_output("o2", s2, step, 8, ty8()).unwrap();
        d.declare_output("o3", s3, step, 8, ty8()).unwrap();
        d
    };
    rep.bus(&mut d2, "fanout (2 and 3 sinks)", "fan3", "din", &["o1", "o2", "o3"]);
}

/// Endpoints on different bit-0 levels — the studio hits this whenever two
/// cells sit at different heights.
fn scenario_levels(rep: &mut Report) {
    for (i, (y_a, y_b)) in [(2i32, 4i32), (2, 10), (6, 2)].iter().enumerate() {
        let mut s = UniversalSchematic::new("lvl".to_string());
        let drv = lever_bank(&mut s, 0, *y_a, 8, 1, 0, 8);
        let snk = lamp_bank(&mut s, 32, *y_b, 8, 8);
        let mut d = Design::for_schematic("lvl", s);
        let step = (0, 2, 0);
        d.declare_input("din", drv, step, 8, ty8()).unwrap();
        d.declare_output("dout", snk, step, 8, ty8()).unwrap();
        rep.bus(&mut d, "endpoints on different levels", &format!("lvl{i}"), "din", &["dout"]);
    }
}

/// Many parallel buses sharing one narrow channel: the contested-corridor
/// case negotiation is supposed to handle.
fn scenario_congested(rep: &mut Report) {
    let mut s = UniversalSchematic::new("cong".to_string());
    let n = 5;
    for k in 0..n {
        let z = 4 + 3 * k;
        lever_bank(&mut s, 0, 2, z, 1, 0, 4);
        lamp_bank(&mut s, 40, 2, z, 4);
    }
    // Pinch the channel: solid walls above and below a 15-cell band at x=20.
    for z in -20..4 {
        for y in 0..=12 {
            s.set_block_from_string(20, y, z, STONE).unwrap();
        }
    }
    let mut d = Design::for_schematic("cong", s);
    let step = (0, 2, 0);
    let ty = IoType::UnsignedInt { bits: 4 };
    for k in 0..n {
        let z = 4 + 3 * k;
        d.declare_input(format!("i{k}"), (1, 2, z), step, 4, ty.clone()).unwrap();
        d.declare_output(format!("o{k}"), (40, 2, z), step, 4, ty.clone()).unwrap();
    }
    for k in 0..n {
        let din = format!("i{k}");
        let dout = format!("o{k}");
        rep.bus(&mut d, "congested parallel channel", &format!("cong{k}"), &din, &[dout.as_str()]);
    }
}

// ----------------------------------------------------------------------
// Real cells from computational_schematics/enhanced
// ----------------------------------------------------------------------

fn enhanced_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("computational_schematics/enhanced")
}

fn load_enhanced(file: &str) -> Option<UniversalSchematic> {
    let path = enhanced_dir().join(file);
    let bytes = std::fs::read(path).ok()?;
    nucleation::formats::schematic::from_schematic(&bytes).ok()
}

/// What the enhanced corpus exposes — printed so the harness documents the
/// real port geometry it is routing against.
#[test]
fn probe_enhanced_cell_ports() {
    let files = [
        "ADD005_8bit_cle_enhanced.schem",
        "ADD007_8bit_cca_matt_enhanced.schem",
        "REGISTER001_8bit_register_enhanced.schem",
        "MULTIPY003_8bit_comb_multiplier_enhanced.schem",
    ];
    for f in files {
        let Some(sch) = load_enhanced(f) else {
            println!("{f}: NOT LOADABLE");
            continue;
        };
        let bbox = sch.get_bounding_box();
        print!("{f}: bbox {:?}..{:?}", bbox.min, bbox.max);
        match sch.resolve_cell_contract() {
            Ok(Some((c, w))) => {
                println!(" contract `{}` warnings={}", c.name, w.len());
                for (n, m) in c.io.inputs.iter().chain(c.io.outputs.iter()) {
                    println!("    port {n}: {} bits, pos[0]={:?}", m.positions.len(), m.positions.first());
                }
            }
            Ok(None) => println!(" NO CONTRACT"),
            Err(e) => println!(" contract error: {e}"),
        }
    }
}

/// How much of the enhanced corpus a bus can even terminate on. Recorded as
/// a metric rather than an attempt: a port whose hardware is a lever or a
/// lamp is legitimately executor-only, and that is a CELL-AUTHORING finding,
/// not a router defect. It is the dominant reason a studio user cannot wire
/// two library cells together at all.
fn probe_corpus_routability() {
    let files = [
        "ADD005_8bit_cle_enhanced.schem",
        "ADD007_8bit_cca_matt_enhanced.schem",
        "BINTOBCD001_8bit_comb_binary_to_bcd_enhanced.schem",
        "MULTIPY003_8bit_comb_multiplier_enhanced.schem",
        "NUMDISPLAY001_fast_bcd_to_7seg_enhanced.schem",
        "REGISTER001_8bit_register_enhanced.schem",
        "REGISTERFILE001_register_file_enhanced.schem",
    ];
    let mut d = Design::new("corpus");
    let mut x = 0i32;
    for (i, f) in files.iter().enumerate() {
        let Some(sch) = load_enhanced(f) else { continue };
        let w = sch.get_bounding_box().max.0 + 8;
        let cellname = format!("c{i}");
        if d.add_cell(&cellname, sch).is_err() {
            continue;
        }
        let _ = d.place(format!("u{i}"), &cellname, (x, 0, 0), 0);
        x += w;
    }
    if let Ok(ports) = d.instance_ports() {
        let total = ports.len();
        let ok = ports.iter().filter(|p| p.routable()).count();
        println!(
            "\n[corpus] enhanced cells: {ok}/{total} ports can terminate a bus ({:.0}%) — \
             the rest are executor-only lever/lamp hardware",
            100.0 * ok as f64 / total.max(1) as f64
        );
    }
}

/// A synthetic library cell with real DUST bus ports: a solid body (so it has
/// a footprint and an influence halo like any placed cell) with an 8-bit
/// input port on its -X face and an 8-bit output port on its +X face, both on
/// the verified 2y-pitch stack. This is the shape a router-friendly cell has
/// — the `nucleation-hdl` PLA cells and the studio's own composites.
fn dust_cell(name: &str, sx: i32, sz: i32) -> (UniversalSchematic, nucleation::io_contract::CellContract) {
    use nucleation::io_contract::{CellContract, IoLayoutBuilder, LayoutFunction};
    let mut s = UniversalSchematic::new(name.to_string());
    for x in 0..sx {
        for z in 0..sz {
            for y in 0..18 {
                s.set_block_from_string(x, y, z, STONE).unwrap();
            }
        }
    }
    let mut ins = Vec::new();
    let mut outs = Vec::new();
    for k in 0..8i32 {
        let y = 2 + 2 * k;
        s.set_block_from_string(0, y, 1, DUST).unwrap();
        s.set_block_from_string(sx - 1, y, 1, DUST).unwrap();
        ins.push((0, y, 1));
        outs.push((sx - 1, y, 1));
    }
    let io = IoLayoutBuilder::new()
        .add_input("d", ty8(), LayoutFunction::OneToOne, ins)
        .unwrap()
        .add_output("q", ty8(), LayoutFunction::OneToOne, outs)
        .unwrap()
        .build();
    (s, CellContract::new(name, io))
}

/// Studio usage A: a row of real cells. Adjacent hops are easy; every
/// non-adjacent hop has another cell's body AND influence halo sitting in the
/// straight corridor, so it needs a detour.
fn scenario_cell_row(rep: &mut Report) {
    let n = 5;
    let mut d = Design::new("row");
    let (sch, contract) = dust_cell("blk", 10, 4);
    d.add_cell_with_contract("blk", sch, contract);
    for k in 0..n {
        d.place(format!("u{k}"), "blk", (24 * k, 0, 0), 0).unwrap();
    }
    // Adjacent hops.
    for k in 0..n - 1 {
        let dr = format!("u{k}.q");
        let sk = format!("u{}.d", k + 1);
        rep.bus(&mut d, "cell row: adjacent hops", &format!("hop{k}"), &dr, &[sk.as_str()]);
    }
    // Skip hops: the corridor crosses intervening cells. Every port is used
    // by exactly ONE net, as in a real netlist — a port's escape lane is one
    // column wide, so two nets on one port is a design error, not a router
    // failure.
    let mut d2 = Design::new("row2");
    let (sch, contract) = dust_cell("blk", 10, 4);
    d2.add_cell_with_contract("blk", sch, contract);
    for k in 0..n {
        d2.place(format!("u{k}"), "blk", (24 * k, 0, 0), 0).unwrap();
    }
    for (i, (a, b)) in [(0, 2), (1, 3), (2, 4), (3, 0), (4, 1)].iter().enumerate() {
        let dr = format!("u{a}.q");
        let sk = format!("u{b}.d");
        rep.bus(&mut d2, "cell row: skip hops (detour required)", &format!("skip{i}"), &dr, &[sk.as_str()]);
    }
    rep.verify(&d, "cell row: adjacent hops", 0);
    rep.verify(&d2, "cell row: skip hops (detour required)", 0);
}

/// Fanout whose trunk has to detour: the extra sinks' junctions must land on
/// the corridor the planner really laid, not on the straight-line guess.
fn scenario_fanout_detour(rep: &mut Report) {
    let mut s = UniversalSchematic::new("fand".to_string());
    let drv = lever_bank(&mut s, 0, 2, 8, 1, 0, 8);
    let s1 = lamp_bank(&mut s, 60, 2, 8, 8);
    let s2 = lamp_bank(&mut s, 30, 2, 40, 8);
    // A wall across the direct line, with a gap well off-axis.
    for z in -40..=24 {
        for y in 0..=20 {
            s.set_block_from_string(20, y, z, STONE).unwrap();
        }
    }
    let mut d = Design::for_schematic("fand", s);
    let step = (0, 2, 0);
    d.declare_input("din", drv, step, 8, ty8()).unwrap();
    d.declare_output("o1", s1, step, 8, ty8()).unwrap();
    d.declare_output("o2", s2, step, 8, ty8()).unwrap();
    rep.bus(&mut d, "fanout over a detoured trunk", "fand", "din", &["o1", "o2"]);
    rep.verify(&d, "fanout over a detoured trunk", 0);
}

/// Studio usage B: cells on a grid at assorted rotations — the general
/// "drag a few blocks onto the canvas and wire them up" case.
fn scenario_cell_grid(rep: &mut Report) {
    let mut d = Design::new("grid");
    let (sch, contract) = dust_cell("blk", 10, 6);
    d.add_cell_with_contract("blk", sch, contract);
    let spots: [(P3, i32); 6] = [
        ((0, 0, 0), 0),
        ((30, 0, 0), 0),
        ((60, 0, 0), 180),
        ((0, 0, 30), 90),
        ((30, 0, 30), 270),
        ((60, 0, 30), 0),
    ];
    for (k, (at, rot)) in spots.iter().enumerate() {
        d.place(format!("u{k}"), "blk", *at, *rot).unwrap();
    }
    if let Ok(ports) = d.instance_ports() {
        println!("\n[grid] derived instance ports:");
        for p in &ports {
            println!(
                "  {:>8} rot?  wires[0]={:?} step={:?} routable={} {}",
                p.name,
                p.wires.as_ref().and_then(|w| w.first()),
                p.step,
                p.routable(),
                p.blocked.as_deref().unwrap_or("")
            );
        }
    }
    // A fixed-point-free permutation: every `q` and every `d` carries exactly
    // one net, like a real netlist.
    let pairs = [(0, 2), (1, 4), (2, 5), (3, 0), (4, 1), (5, 3)];
    for (i, (a, b)) in pairs.iter().enumerate() {
        let dr = format!("u{a}.q");
        let sk = format!("u{b}.d");
        rep.bus(&mut d, "cell grid, assorted rotations", &format!("g{i}"), &dr, &[sk.as_str()]);
    }
    rep.verify(&d, "cell grid, assorted rotations", 0);
}

/// Studio usage C: buses routed across a field already populated with REAL
/// enhanced cells. The endpoints are loose-layer hardware (which is what a
/// user declares for the design's own IO), but every corridor has to get past
/// real cell bodies and their halos.
fn scenario_real_cell_field(rep: &mut Report) {
    let files = [
        "ADD005_8bit_cle_enhanced.schem",
        "ADD007_8bit_cca_matt_enhanced.schem",
        "REGISTER001_8bit_register_enhanced.schem",
    ];
    let mut s = UniversalSchematic::new("field".to_string());
    // Design IO on the loose layer, west and east, at three z lanes.
    let mut lanes = Vec::new();
    for k in 0..3i32 {
        let z = 6 + 14 * k;
        let i = lever_bank(&mut s, -20, 2, z, 1, 0, 8);
        let o = lamp_bank(&mut s, 80, 2, z, 8);
        lanes.push((i, o));
    }
    let mut d = Design::for_schematic("field", s);
    let step = (0, 2, 0);
    for (k, (i, o)) in lanes.iter().enumerate() {
        d.declare_input(format!("i{k}"), *i, step, 8, ty8()).unwrap();
        d.declare_output(format!("o{k}"), *o, step, 8, ty8()).unwrap();
    }
    // Real cells dropped into the middle of the field.
    let spots: [(P3, i32); 3] = [((10, 0, 2), 0), ((36, 0, 18), 90), ((54, 0, 4), 180)];
    for (j, f) in files.iter().enumerate() {
        let Some(sch) = load_enhanced(f) else { continue };
        let cellname = format!("c{j}");
        if d.add_cell(&cellname, sch).is_err() {
            continue;
        }
        let (at, rot) = spots[j];
        let _ = d.place(format!("u{j}"), &cellname, at, rot);
    }
    // Item (a): with REAL community cells placed and ZERO buses routed, is the
    // design clean? It used to report hundreds of violations from the cells'
    // own interiors, so `check()` could never come back green in the studio.
    let baseline = drc_baseline(&d);
    match d.check() {
        Ok(c) => println!(
            "\n[field] {} real cells placed, ZERO buses: clean={} — {} gating violation(s), \
             {} reported informationally from cell interiors",
            d.instance_ports().map(|p| p.len()).unwrap_or(0),
            c.clean,
            baseline,
            c.json.matches("\"kind\"").count() - baseline
        ),
        Err(e) => println!("\n[field] check() failed before routing: {e}"),
    }
    for k in 0..3 {
        let din = format!("i{k}");
        let dout = format!("o{k}");
        rep.bus(
            &mut d,
            "field populated with real cells",
            &format!("lane{k}"),
            &din,
            &[dout.as_str()],
        );
    }
    rep.verify(&d, "field populated with real cells", baseline);
}

// ----------------------------------------------------------------------
// The report
// ----------------------------------------------------------------------

fn measure() -> Report {
    probe_corpus_routability();
    let mut rep = Report::default();
    scenario_open_field(&mut rep);
    scenario_walled(&mut rep);
    scenario_crossings(&mut rep);
    scenario_fanout(&mut rep);
    scenario_fanout_detour(&mut rep);
    scenario_levels(&mut rep);
    scenario_congested(&mut rep);
    scenario_cell_row(&mut rep);
    scenario_cell_grid(&mut rep);
    scenario_real_cell_field(&mut rep);
    rep
}

#[test]
fn routability_rate_holds_the_line() {
    let rep = measure();
    rep.print();
    let ok = rep.routed();
    let total = rep.attempts.len();
    let pct = 100.0 * ok as f64 / total.max(1) as f64;
    // Baseline recorded in redstone-eda/IMPROVEMENTS.md. Ratchet this up as
    // the router improves; never down.
    assert!(
        pct >= ROUTABILITY_FLOOR,
        "routability regressed to {pct:.1}% ({ok}/{total}), floor is {ROUTABILITY_FLOOR}%"
    );
    // Every failure must be actionable.
    let vague: Vec<String> = rep
        .attempts
        .iter()
        .filter_map(|a| a.reason.as_ref())
        .filter(|r| category(r) == "other")
        .cloned()
        .collect();
    assert!(vague.is_empty(), "unactionable failure reasons: {vague:#?}");
    // Routing more buses is only progress if the geometry is still legal.
    assert!(rep.dirty.is_empty(), "routed designs failed DRC/LVS: {:#?}", rep.dirty);
}

/// Measured 2026-08-09. Baseline before the corridor router and the instance
/// transform fix was 52.1%; see `redstone-eda/IMPROVEMENTS.md`.
const ROUTABILITY_FLOOR: f64 = 84.0;
