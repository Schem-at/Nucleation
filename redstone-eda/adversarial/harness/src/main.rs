//! ADVERSARIAL SOLVE + VERIFY HARNESS.
//!
//! One process, one problem: read a problem spec (JSON), build the endpoint
//! hardware and the obstacle field, ask the Rust bus router to solve it, MEASURE
//! the solution, and then prove it in the vanilla-accurate engine across many
//! words -- never one, because a quiet value validates a leaking pitch.
//!
//! Everything it knows about the solver is the public `Design` surface, so this
//! file touches nothing another agent owns.
//!
//! Usage:
//!   adv-harness <problem.json> [--work-dir DIR]
//! Prints ONE json object on stdout (the result record); diagnostics on stderr.

use std::time::Instant;

use nucleation::design::{BusState, BusStyle, Design, Gate};
use nucleation::io_contract::{IoType, Value};
use nucleation::UniversalSchematic;
use serde_json::{json, Map, Value as J};

type P3 = (i32, i32, i32);

const STONE: &str = "minecraft:stone";
const LAMP: &str = "minecraft:redstone_lamp[lit=false]";
const LEVER: &str = "minecraft:lever[face=floor,facing=north,powered=false]";
const DUST: &str = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";

fn p3(v: &J) -> P3 {
    (
        v[0].as_i64().unwrap_or(0) as i32,
        v[1].as_i64().unwrap_or(0) as i32,
        v[2].as_i64().unwrap_or(0) as i32,
    )
}

fn add(a: P3, b: P3) -> P3 {
    (a.0 + b.0, a.1 + b.1, a.2 + b.2)
}

fn scale(a: P3, k: i32) -> P3 {
    (a.0 * k, a.1 * k, a.2 * k)
}

/// Gating DRC violation count (`drc` array; `cells` is informational).
fn drc_violations(json: &str) -> usize {
    let Some(start) = json.find("\"drc\":[") else {
        return 0;
    };
    let rest = &json[start + 7..];
    let end = rest.find("],\"cells\"").unwrap_or(rest.len());
    rest[..end].matches("\"kind\"").count()
}

struct PortSpec {
    name: String,
    dir: String,
    anchor: P3,
    step: P3,
    width: u8,
    out: P3,
}

struct BusSpecJ {
    name: String,
    driver: String,
    sinks: Vec<String>,
    gates: Vec<Gate>,
}

/// Place one port's hardware. Inputs get a lever bank one cell outward, outputs
/// get a lamp as their own support -- the `design_level_shift.rs` convention.
fn place_port(s: &mut UniversalSchematic, p: &PortSpec) -> Result<(), String> {
    for k in 0..p.width as i32 {
        let cell = add(p.anchor, scale(p.step, k));
        let below = add(cell, (0, -1, 0));
        if p.dir == "in" {
            s.set_block_from_string(below.0, below.1, below.2, STONE)
                .map_err(|e| e.to_string())?;
            s.set_block_from_string(cell.0, cell.1, cell.2, DUST)
                .map_err(|e| e.to_string())?;
            let lv = add(cell, p.out);
            let lvb = add(lv, (0, -1, 0));
            s.set_block_from_string(lvb.0, lvb.1, lvb.2, STONE)
                .map_err(|e| e.to_string())?;
            s.set_block_from_string(lv.0, lv.1, lv.2, LEVER)
                .map_err(|e| e.to_string())?;
        } else {
            s.set_block_from_string(below.0, below.1, below.2, LAMP)
                .map_err(|e| e.to_string())?;
            s.set_block_from_string(cell.0, cell.1, cell.2, DUST)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn parse_ports(spec: &J) -> Vec<PortSpec> {
    spec["ports"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|p| PortSpec {
            name: p["name"].as_str().unwrap_or("?").to_string(),
            dir: p["dir"].as_str().unwrap_or("in").to_string(),
            anchor: p3(&p["anchor"]),
            step: p3(&p["step"]),
            width: p["width"].as_u64().unwrap_or(1) as u8,
            out: p3(&p["out"]),
        })
        .collect()
}

fn parse_buses(spec: &J) -> Vec<BusSpecJ> {
    spec["buses"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|b| BusSpecJ {
            name: b["name"].as_str().unwrap_or("b").to_string(),
            driver: b["driver"].as_str().unwrap_or("din").to_string(),
            sinks: b["sinks"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            gates: b["gates"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|g| Gate {
                            name: g["name"].as_str().unwrap_or("g").to_string(),
                            anchor: p3(&g["anchor"]),
                            step: p3(&g["step"]),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect()
}

/// Default test vectors: walking ones (per-bit isolation), the all-on and the
/// alternating pairs (interleave under load), plus seeded pseudorandom words.
/// One quiet value proves nothing, so the set is never short.
fn default_words(width: u8, seed: u64) -> Vec<u32> {
    let mask: u32 = if width >= 32 {
        u32::MAX
    } else {
        (1u32 << width) - 1
    };
    let mut v: Vec<u32> = Vec::new();
    for i in 0..width.min(32) {
        v.push(1u32 << i);
    }
    for w in [0u32, mask, 0xAAAA_AAAA, 0x5555_5555, 0x3333_3333, 0x0F0F_0F0F] {
        v.push(w & mask);
    }
    // xorshift, seeded: deterministic per problem.
    let mut x = seed | 1;
    for _ in 0..4 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        v.push((x as u32) & mask);
    }
    let mut out = Vec::new();
    for w in v {
        if !out.contains(&w) {
            out.push(w);
        }
    }
    out
}

fn read_u(v: &Value) -> u64 {
    match v {
        Value::U32(x) => *x as u64,
        Value::U64(x) => *x,
        Value::I32(x) => *x as u64,
        Value::I64(x) => *x as u64,
        Value::Bool(b) => *b as u64,
        Value::BitArray(bits) => bits
            .iter()
            .enumerate()
            .fold(0u64, |a, (i, b)| a | ((*b as u64) << i)),
        _ => u64::MAX,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: adv-harness <problem.json> [--work-dir DIR]");
        std::process::exit(2);
    }
    let mut work_dir = String::from(".");
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--work-dir" && i + 1 < args.len() {
            work_dir = args[i + 1].clone();
            i += 2;
        } else {
            i += 1;
        }
    }
    let raw = std::fs::read_to_string(&args[1]).expect("read problem");
    let spec: J = serde_json::from_str(&raw).expect("parse problem");
    let out = run(&spec, &work_dir);
    println!("{}", serde_json::to_string(&out).unwrap());
}

fn run(spec: &J, work_dir: &str) -> J {
    let id = spec["id"].as_str().unwrap_or("anon").to_string();
    let tier = spec["tier"].as_u64().unwrap_or(0);
    let seed = spec["seed"].as_u64().unwrap_or(1);
    let carrier = spec["carrier"].as_str().unwrap_or("binary").to_string();
    let mut rec = Map::new();
    rec.insert("id".into(), json!(id));
    rec.insert("tier".into(), json!(tier));
    rec.insert("seed".into(), json!(seed));
    rec.insert("carrier".into(), json!(carrier));
    rec.insert("family".into(), spec["family"].clone());

    // A carrier the router has no knob for at all is a CAPABILITY GAP, and
    // saying so is more honest than routing binary and calling it a hex bus.
    if carrier != "binary" {
        rec.insert("solved".into(), json!(false));
        rec.insert(
            "unsupported".into(),
            json!(format!(
                "carrier `{carrier}`: Design::route_bus has no encoding knob \
                 (routing::bus::Encoding::HexAnalog exists but is unreachable \
                 from the design surface, and no analog-preserving tile exists)"
            )),
        );
        return J::Object(rec);
    }

    let style = BusStyle {
        bus_block: spec["style"]["bus_block"]
            .as_str()
            .unwrap_or("minecraft:gray_concrete")
            .to_string(),
        transparent_block: spec["style"]["transparent_block"]
            .as_str()
            .unwrap_or("minecraft:glass")
            .to_string(),
    };

    // ---- geometry ----
    let mut s = UniversalSchematic::new(id.clone());
    let mut obstacle_cells = 0usize;
    if let Some(obs) = spec["obstacles"].as_array() {
        for o in obs {
            let (x, y, z) = p3(o);
            let b = o[3].as_str().unwrap_or(STONE);
            if s.set_block_from_string(x, y, z, b).is_ok() {
                obstacle_cells += 1;
            }
        }
    }
    let ports = parse_ports(spec);
    for p in &ports {
        if let Err(e) = place_port(&mut s, p) {
            rec.insert("solved".into(), json!(false));
            rec.insert("error".into(), json!(format!("hardware: {e}")));
            return J::Object(rec);
        }
    }
    rec.insert("obstacle_cells".into(), json!(obstacle_cells));

    let mut d = Design::for_schematic(id.clone(), s);
    for p in &ports {
        let ty = IoType::UnsignedInt {
            bits: p.width as usize,
        };
        let r = if p.dir == "in" {
            d.declare_input(p.name.clone(), p.anchor, p.step, p.width, ty)
        } else {
            d.declare_output(p.name.clone(), p.anchor, p.step, p.width, ty)
        };
        if let Err(e) = r {
            rec.insert("solved".into(), json!(false));
            rec.insert("error".into(), json!(format!("declare `{}`: {e}", p.name)));
            return J::Object(rec);
        }
    }

    // ---- solve ----
    let buses = parse_buses(spec);
    let mut bus_recs = Vec::new();
    let mut all_routed = true;
    let mut total_ms = 0f64;
    for b in &buses {
        let sinks: Vec<&str> = b.sinks.iter().map(String::as_str).collect();
        let t0 = Instant::now();
        let st = d.route_bus(
            b.name.clone(),
            &b.driver,
            &sinks,
            b.gates.clone(),
            style.clone(),
        );
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        total_ms += ms;
        let mut br = Map::new();
        br.insert("name".into(), json!(b.name));
        br.insert("solve_ms".into(), json!((ms * 100.0).round() / 100.0));
        match st {
            Err(e) => {
                all_routed = false;
                br.insert("state".into(), json!("Refused"));
                br.insert("reason".into(), json!(e));
            }
            Ok(BusState::Failed(reason)) => {
                all_routed = false;
                br.insert("state".into(), json!("Failed"));
                br.insert("reason".into(), json!(reason));
            }
            Ok(BusState::Intended) => {
                all_routed = false;
                br.insert("state".into(), json!("Intended"));
            }
            Ok(BusState::Routed) => {
                br.insert("state".into(), json!("Routed"));
                if let Some(layer) = d.bus(&b.name) {
                    let cost = d.bus_cost(layer);
                    br.insert(
                        "cost".into(),
                        json!({
                            "length": cost.length,
                            "delay_rt": cost.delay_rt,
                            "skew_rt": cost.skew_rt,
                            "coherence": cost.coherence,
                            "footprint": cost.footprint,
                        }),
                    );
                    let frag = &layer.fragment;
                    br.insert("cells".into(), json!(frag.len()));
                    let mut lo = (i32::MAX, i32::MAX, i32::MAX);
                    let mut hi = (i32::MIN, i32::MIN, i32::MIN);
                    let mut kinds: Map<String, J> = Map::new();
                    for (p, blk) in frag.iter() {
                        lo = (lo.0.min(p.0), lo.1.min(p.1), lo.2.min(p.2));
                        hi = (hi.0.max(p.0), hi.1.max(p.1), hi.2.max(p.2));
                        let base = blk.split('[').next().unwrap_or(blk).to_string();
                        let e = kinds.entry(base).or_insert(json!(0));
                        let n = e.as_u64().unwrap_or(0) + 1;
                        *e = json!(n);
                    }
                    br.insert("bbox".into(), json!([[lo.0, lo.1, lo.2], [hi.0, hi.1, hi.2]]));
                    br.insert("block_kinds".into(), J::Object(kinds));
                    br.insert("runs".into(), json!(layer.runs.len()));
                    br.insert("segments".into(), json!(layer.segments.len()));
                    // Full geometry to disk: the critic reads the file, so the
                    // prompt stays small.
                    let cells: Vec<J> = frag
                        .iter()
                        .map(|(p, blk)| json!([p.0, p.1, p.2, blk]))
                        .collect();
                    let path = format!("{work_dir}/{id}.{}.geom.json", b.name);
                    let _ = std::fs::write(
                        &path,
                        serde_json::to_string(&json!({"bus": b.name, "cells": cells})).unwrap(),
                    );
                    br.insert("geom_file".into(), json!(path));
                }
            }
        }
        bus_recs.push(J::Object(br));
    }
    rec.insert("buses".into(), J::Array(bus_recs));
    rec.insert(
        "solve_ms".into(),
        json!((total_ms * 100.0).round() / 100.0),
    );
    rec.insert("routed".into(), json!(all_routed));

    // ---- static checks ----
    match d.check() {
        Ok(c) => {
            rec.insert("drc_lvs_clean".into(), json!(c.clean));
            rec.insert("drc_violations".into(), json!(drc_violations(&c.json)));
            if !c.clean {
                let path = format!("{work_dir}/{id}.check.json");
                let _ = std::fs::write(&path, &c.json);
                rec.insert("check_file".into(), json!(path));
            }
        }
        Err(e) => {
            rec.insert("drc_lvs_clean".into(), json!(false));
            rec.insert("check_error".into(), json!(e));
        }
    }
    if let Ok(bytes) = d.to_schem_bytes() {
        let path = format!("{work_dir}/{id}.schem");
        let _ = std::fs::write(&path, bytes);
        rec.insert("schem_file".into(), json!(path));
    }

    if !all_routed {
        rec.insert("solved".into(), json!(false));
        return J::Object(rec);
    }

    // ---- in-sim proof ----
    if spec["sim"].as_bool() == Some(false) {
        rec.insert("solved".into(), json!(true));
        rec.insert("sim".into(), json!("skipped"));
        return J::Object(rec);
    }
    let budget = spec["settle"].as_u64().unwrap_or(4000) as u32;
    let sim = simulate(&d, &buses, &ports, seed, budget);
    let pass = sim["pass"].as_bool().unwrap_or(false);
    rec.insert("sim".into(), sim);
    rec.insert("solved".into(), json!(pass));
    J::Object(rec)
}

/// Drive every bus with many words and read every sink, in the engine.
///
/// Two phases: ISOLATION (one bus hot, the rest at zero -- catches leaks INTO a
/// quiet neighbour, which a single all-on vector hides) and LOADED (every bus
/// carrying a different word at once).
fn simulate(d: &Design, buses: &[BusSpecJ], ports: &[PortSpec], seed: u64, budget: u32) -> J {
    use nucleation::simulation::typed_executor::BackendCircuitExecutor;

    let baked = match d.bake(budget) {
        Ok(b) => b,
        Err(e) => return json!({"pass": false, "error": format!("bake: {e}")}),
    };
    let contract = match baked.embedded_cell_contract() {
        Ok(Some(c)) => c,
        Ok(None) => return json!({"pass": false, "error": "no embedded contract"}),
        Err(e) => return json!({"pass": false, "error": format!("contract: {e}")}),
    };
    let extra = nucleation::design::executor_extra_states();
    let refs: Vec<&str> = extra.iter().map(String::as_str).collect();
    let mut cell = match BackendCircuitExecutor::for_cell(baked, &contract, &refs) {
        Ok(c) => c,
        Err(e) => return json!({"pass": false, "error": format!("executor: {e}")}),
    };
    cell.settle(budget);

    let width_of = |name: &str| -> u8 {
        ports
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.width)
            .unwrap_or(1)
    };
    let mut checks = 0usize;
    let mut fails: Vec<J> = Vec::new();

    // Phase 1: isolation + crosstalk.
    for b in buses {
        let w = width_of(&b.driver);
        for word in default_words(w, seed ^ (b.name.len() as u64) << 8) {
            // Every driver low, then this one hot.
            for other in buses {
                let ow = width_of(&other.driver);
                let v = if other.name == b.name { word } else { 0 };
                let _ = cell.set_input(
                    &other.driver,
                    &Value::U32(v & mask_of(ow)),
                );
            }
            cell.settle(budget.min(1200));
            for other in buses {
                let raw = if other.name == b.name { word } else { 0 };
                let expect = raw & mask_of(width_of(&other.driver));
                for sink in &other.sinks {
                    checks += 1;
                    match cell.read_output(sink) {
                        Ok(v) => {
                            let got = read_u(&v);
                            if got != expect as u64 {
                                if fails.len() < 24 {
                                    fails.push(json!({
                                        "phase": "isolation",
                                        "hot_bus": b.name,
                                        "word": word,
                                        "sink": sink,
                                        "expect": expect,
                                        "got": got,
                                    }));
                                }
                            }
                        }
                        Err(e) => {
                            if fails.len() < 24 {
                                fails.push(json!({
                                    "phase": "isolation", "sink": sink,
                                    "error": e.to_string()
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    // Phase 2: loaded -- every bus a different word at once.
    if buses.len() > 1 {
        let mut x = seed | 1;
        for _ in 0..6 {
            let mut want: Vec<(String, u32)> = Vec::new();
            for b in buses {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                let w = (x as u32) & mask_of(width_of(&b.driver));
                let _ = cell.set_input(&b.driver, &Value::U32(w));
                want.push((b.name.clone(), w));
            }
            cell.settle(budget.min(1200));
            for b in buses {
                let expect = want
                    .iter()
                    .find(|(n, _)| n == &b.name)
                    .map(|(_, w)| *w)
                    .unwrap_or(0);
                for sink in &b.sinks {
                    checks += 1;
                    if let Ok(v) = cell.read_output(sink) {
                        let got = read_u(&v);
                        if got != expect as u64 && fails.len() < 24 {
                            fails.push(json!({
                                "phase": "loaded", "bus": b.name, "sink": sink,
                                "expect": expect, "got": got,
                            }));
                        }
                    }
                }
            }
        }
    }

    json!({
        "pass": fails.is_empty(),
        "checks": checks,
        "failures": fails.len(),
        "sample_failures": fails,
    })
}

fn mask_of(width: u8) -> u32 {
    if width >= 32 {
        u32::MAX
    } else {
        (1u32 << width) - 1
    }
}
