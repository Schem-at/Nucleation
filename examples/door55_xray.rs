//! Forensic x-ray of the record 3x3 piston door: where does the causal chain
//! stop, and what was next to what when it stopped.
//!
//! ```text
//! cargo run --release --example door55_xray --features bridge,mc-tick -- \
//!     <world.zip> [press_tick[,press_tick...]] [ticks]
//! ```
//!
//! Prints machine-readable lines (`BLOCK`, `BODY`, `TOUCH`, `CHG`, `UPD`,
//! `SEEN`) so the analysis can happen outside the binary — the point is to
//! avoid a summary that quietly hides the thing being looked for.

use nucleation::bridge::mc_tick::ffi::{TickSettleMode, TickSimulation};
use nucleation::bridge::schematic::ffi::Schematic;
use std::collections::{BTreeMap, BTreeSet};

fn read_out(f: impl FnOnce(&mut diplomat_runtime::DiplomatWrite)) -> String {
    unsafe {
        let write = diplomat_runtime::diplomat_buffer_write_create(0);
        f(&mut *write);
        let text = String::from_utf8_lossy((*write).as_bytes()).into_owned();
        diplomat_runtime::diplomat_buffer_write_destroy(write);
        text
    }
}

/// `{"id":N,"kind":"...","pos":[x,y,z]...}` without a JSON parser, because the
/// values may be `NaN` and that is the thing being preserved.
fn bodies(json: &str) -> Vec<(u32, String, [f64; 3])> {
    let mut out = Vec::new();
    for chunk in json.split("{\"id\":").skip(1) {
        let Some(id) = chunk.split(',').next().and_then(|t| t.trim().parse::<u32>().ok()) else {
            continue;
        };
        let kind = chunk
            .split("\"kind\":\"")
            .nth(1)
            .and_then(|t| t.split('"').next())
            .unwrap_or("?")
            .to_string();
        if kind == "?" {
            continue;
        }
        let pos = chunk
            .split("\"pos\":[")
            .nth(1)
            .and_then(|t| t.split(']').next())
            .map(|t| {
                let v: Vec<f64> = t.split(',').filter_map(|p| p.trim().parse().ok()).collect();
                [
                    v.first().copied().unwrap_or(f64::NAN),
                    v.get(1).copied().unwrap_or(f64::NAN),
                    v.get(2).copied().unwrap_or(f64::NAN),
                ]
            })
            .unwrap_or([f64::NAN; 3]);
        out.push((id, kind, pos));
    }
    out
}

/// `BasePressurePlateBlock.TOUCH_AABB` at a cell — duplicated from
/// `mc_tick::components` (private there) so the geometry printed here is the
/// geometry the engine actually tests against.
fn touch_aabb(p: (i32, i32, i32)) -> ([f64; 3], [f64; 3]) {
    (
        [f64::from(p.0) + 0.0625, f64::from(p.1), f64::from(p.2) + 0.0625],
        [f64::from(p.0) + 0.9375, f64::from(p.1) + 0.25, f64::from(p.2) + 0.9375],
    )
}

/// `DetectorRailBlock`'s search box: the cell inset 0.2 on every side but the
/// bottom.
fn rail_aabb(p: (i32, i32, i32)) -> ([f64; 3], [f64; 3]) {
    (
        [f64::from(p.0) + 0.2, f64::from(p.1), f64::from(p.2) + 0.2],
        [f64::from(p.0) + 0.8, f64::from(p.1) + 0.8, f64::from(p.2) + 0.8],
    )
}

/// Signed clearance per axis: negative means the boxes overlap on that axis,
/// positive is the gap that would have to be closed.
fn gaps(a: ([f64; 3], [f64; 3]), b: ([f64; 3], [f64; 3])) -> [f64; 3] {
    let mut out = [0.0; 3];
    for i in 0..3 {
        out[i] = (b.0[i] - a.1[i]).max(a.0[i] - b.1[i]);
    }
    out
}

/// A self-contained control for the one mechanism this x-ray accuses.
///
/// Vanilla: a button strongly powers the block it is attached to, whatever face
/// it is on, and a conductor re-emits that to everything touching it. So a lamp
/// above a block with a *wall* button on it lights, exactly as it does with a
/// floor button or a wall lever. Three lanes, identical but for the switch.
fn switch_probe() {
    for (label, switch) in [
        ("wall button", "{Name:\"minecraft:oak_button\",Properties:{face:\"wall\",facing:\"north\",powered:\"false\"}}"),
        ("floor button", "{Name:\"minecraft:oak_button\",Properties:{face:\"floor\",facing:\"north\",powered:\"false\"}}"),
        ("wall lever", "{Name:\"minecraft:lever\",Properties:{face:\"wall\",facing:\"north\",powered:\"false\"}}"),
    ] {
        // The switch sits at (1,1,0) for the wall lanes — north of the
        // conductor, so it is attached to it — and at (1,2,1) on the floor lane,
        // where the conductor *is* its floor. Both attach to (1,1,1).
        let (switch_pos, lamp_pos) = if label == "floor button" {
            ("[1,2,1]", "[0,1,1]")
        } else {
            ("[1,1,0]", "[1,2,1]")
        };
        let snbt = format!(
            "{{size:[3,3,3],entities:[],palette:[\
             {{Name:\"minecraft:quartz_block\"}},\
             {{Name:\"minecraft:note_block\",Properties:{{instrument:\"basedrum\",note:\"0\",powered:\"false\"}}}},\
             {{Name:\"minecraft:redstone_lamp\",Properties:{{lit:\"false\"}}}},\
             {switch}],\
             blocks:[{{pos:[1,0,1],state:0}},{{pos:[1,1,1],state:1}},\
             {{pos:{lamp_pos},state:2}},{{pos:{switch_pos},state:3}}]}}"
        );
        let mut sim = match TickSimulation::from_snbt(
            snbt.as_bytes(),
            TickSettleMode::InWorld,
            0,
            0,
            0,
            b"",
        ) {
            Ok(s) => s,
            Err(_) => {
                println!("PROBE {label}: refused: {}", read_out(TickSimulation::last_error_detail));
                continue;
            }
        };
        let (lx, ly, lz) = if label == "floor button" { (0, 1, 1) } else { (1, 2, 1) };
        let (sx, sy, sz) = if label == "floor button" { (1, 2, 1) } else { (1, 1, 0) };
        sim.run(2);
        let before = read_out(|w| sim.get_block(lx, ly, lz, w));
        sim.use_block(sx, sy, sz);
        sim.run(4);
        let after = read_out(|w| sim.get_block(lx, ly, lz, w));
        println!(
            "PROBE {label:<13} switch={} lamp {before} -> {after}  lit={}",
            read_out(|w| sim.get_block(sx, sy, sz, w)),
            after.contains("lit=true")
        );
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("a world zip");
    let presses: Vec<u32> = std::env::args()
        .nth(2)
        .map(|s| s.split(',').filter_map(|p| p.parse().ok()).collect())
        .unwrap_or_else(|| vec![5]);
    let ticks: u32 = std::env::args().nth(3).and_then(|t| t.parse().ok()).unwrap_or(80);

    let bytes = std::fs::read(&path).expect("read the zip");
    let schem = Schematic::from_data(&bytes).expect("parse the world");
    let mut sim = TickSimulation::from_schematic(&schem, TickSettleMode::InWorld, 0, 0, 0, b"")
        .unwrap_or_else(|_| {
            eprintln!("refused: {}", read_out(TickSimulation::last_error_detail));
            std::process::exit(2)
        });

    println!("# motion semantics: {}", read_out(|w| sim.motion_semantics(w)));
    println!("# block entities: {}", read_out(|w| TickSimulation::block_entity_audit_json(&schem, w)));
    // The dispenser is the build's only block entity; what it holds decides
    // what appears in the cell two observers are watching.
    let snbt = read_out(|w| TickSimulation::gametest_snbt(&schem, w));
    if let Ok(dest) = std::env::var("DUMP_SNBT") {
        let _ = std::fs::write(&dest, &snbt);
        println!("# snbt: {} bytes -> {dest}", snbt.len());
    }
    for (i, _) in snbt.match_indices("nbt:") {
        println!("# BE {}", &snbt[i..(i + 500).min(snbt.len())]);
    }
    switch_probe();

    // ---- the world, as the simulator holds it (not as a capture remembers it)
    let mut blocks: BTreeMap<(i32, i32, i32), String> = BTreeMap::new();
    for x in 55..=90 {
        for y in 0..=10 {
            for z in 14..=26 {
                let s = read_out(|w| sim.get_block(x, y, z, w));
                if s != "minecraft:air" && !s.is_empty() {
                    blocks.insert((x, y, z), s);
                }
            }
        }
    }
    for (p, s) in &blocks {
        println!("BLOCK {} {} {} {}", p.0, p.1, p.2, s);
    }

    // ---- every trigger surface in the build, and every body, and the geometry
    let triggers: Vec<((i32, i32, i32), String)> = blocks
        .iter()
        .filter(|(_, s)| {
            s.contains("pressure_plate") || s.contains("detector_rail") || s.contains("tripwire")
        })
        .map(|(p, s)| (*p, s.clone()))
        .collect();

    let ents = bodies(&read_out(|w| sim.item_entities_json(w)));
    println!("# bodies: {}", ents.len());
    for (id, kind, pos) in &ents {
        let bb = mc_tick::entity::body_aabb(kind, *pos);
        match bb {
            Some((min, max)) => println!(
                "BODY {id} {kind} pos={pos:?} min={min:?} max={max:?} in={}",
                read_out(|w| sim.get_block(
                    pos[0].floor() as i32,
                    pos[1].floor() as i32,
                    pos[2].floor() as i32,
                    w
                ))
            ),
            None => println!("BODY {id} {kind} pos={pos:?} NO-HITBOX"),
        }
    }
    for (p, state) in &triggers {
        let (tmin, tmax) = if state.contains("detector_rail") {
            rail_aabb(*p)
        } else {
            touch_aabb(*p)
        };
        println!("TRIGGER {} {} {} {state} box={tmin:?}..{tmax:?}", p.0, p.1, p.2);
        for (id, kind, pos) in &ents {
            let Some(bb) = mc_tick::entity::body_aabb(kind, *pos) else { continue };
            let g = gaps(bb, (tmin, tmax));
            // Only worth printing the neighbourhood, not the whole build.
            if g.iter().all(|v| *v < 1.5) {
                let touching = g.iter().all(|v| *v < 0.0);
                println!(
                    "TOUCH {} {} {} id={id} {kind} touching={touching} gap_x={:.6} gap_y={:.6} gap_z={:.6}",
                    p.0, p.1, p.2, g[0], g[1], g[2]
                );
            }
        }
    }

    // ---- record BEFORE the stimulus, or the log is empty and correctly so
    sim.record_updates(true);
    for tick in 1..=ticks {
        if presses.contains(&tick) {
            // The button is the one the door is actuated by; found rather than
            // assumed, and named so a wrong one is visible.
            let b = blocks
                .iter()
                .find(|(_, s)| s.contains("_button"))
                .map(|(p, _)| *p)
                .expect("a button");
            println!("# tick {tick}: pressing {b:?}");
            sim.use_block(b.0, b.1, b.2);
        }
        sim.step();
    }

    // Did anything move? Printed as the same lines, so a diff answers it.
    for (id, kind, pos) in bodies(&read_out(|w| sim.item_entities_json(w))) {
        println!("ENDBODY {id} {kind} pos={pos:?}");
    }
    println!("# updates recorded: {}", sim.updates_count());
    for c in read_out(|w| sim.changes_json(w)).split("{\"tick\":").skip(1) {
        let tick = c.split(',').next().unwrap_or("?");
        let pos = c.split("\"pos\":[").nth(1).and_then(|t| t.split(']').next()).unwrap_or("?");
        let from = c.split("\"from\":\"").nth(1).and_then(|t| t.split('"').next()).unwrap_or("?");
        let to = c.split("\"to\":\"").nth(1).and_then(|t| t.split('"').next()).unwrap_or("?");
        println!("CHG {tick} [{pos}] {from} -> {to}");
    }

    // ---- the dispatches themselves
    let raw = read_out(|w| sim.updates_json(w));
    let mut seen: BTreeSet<(i32, i32, i32)> = BTreeSet::new();
    for u in raw.split("{\"tick\":").skip(1) {
        let field = |k: &str| {
            u.split(k).nth(1).and_then(|t| t.split('"').next()).unwrap_or("?").to_string()
        };
        let tick = u.split(',').next().unwrap_or("?").to_string();
        let seq = u.split("\"seq\":").nth(1).and_then(|t| t.split(',').next()).unwrap_or("?");
        let pos = u.split("\"pos\":[").nth(1).and_then(|t| t.split(']').next()).unwrap_or("");
        let c: Vec<i32> = pos.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        if c.len() == 3 {
            seen.insert((c[0], c[1], c[2]));
        }
        println!(
            "UPD {tick} {seq} [{pos}] from={} kind={} phase={} state={}",
            field("\"from\":\""),
            field("\"kind\":\""),
            field("\"phase\":\""),
            field("\"state\":\"")
        );
    }

    // ---- which components never heard a thing. This is the whole question.
    for (p, s) in &blocks {
        let interesting = ["piston", "observer", "plate", "rail", "note_block", "repeater",
            "dispenser", "comparator", "redstone", "lever", "button", "target"]
            .iter()
            .any(|n| s.contains(n));
        if interesting {
            println!(
                "SEEN {} {} {} {} updated={}",
                p.0,
                p.1,
                p.2,
                s,
                seen.contains(p)
            );
        }
    }
    println!("# retract contacts: {}", sim.piston_retract_contacts());
    println!("# quiescent: {}", sim.is_quiescent());
}
