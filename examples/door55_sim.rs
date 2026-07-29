//! Tick the record 3x3 piston door and report, per tick, what its entities do.
//!
//! The companion to `world_entity_audit`, which only counts what is in the
//! file. This one actually runs it:
//!
//! ```text
//! cargo run --release --example door55_sim --features bridge,mc-tick -- <world.zip> [ticks]
//! ```
//!
//! It prints which `Entity.load` Motion semantics the save selected, how many
//! of its entities carry a non-finite velocity *after* loading, and then the
//! first tick on which each entity moves — because "the door comes apart" is
//! only a useful answer if it says which piece went first.

use nucleation::bridge::mc_tick::ffi::{TickSettleMode, TickSimulation};
use nucleation::bridge::schematic::ffi::Schematic;
use std::collections::BTreeMap;

fn read_out(f: impl FnOnce(&mut diplomat_runtime::DiplomatWrite)) -> String {
    unsafe {
        let write = diplomat_runtime::diplomat_buffer_write_create(0);
        f(&mut *write);
        let text = String::from_utf8_lossy((*write).as_bytes()).into_owned();
        diplomat_runtime::diplomat_buffer_write_destroy(write);
        text
    }
}

/// `{"id":N,...,"pos":[x,y,z],"vel":[x,y,z]}` out of the entity JSON, without
/// pulling in a parser. Values may be `NaN`, which is not legal JSON — and
/// that is exactly the thing being looked for, so the text is read directly.
fn entities(json: &str) -> BTreeMap<u32, (String, [f64; 3], [f64; 3])> {
    let mut out = BTreeMap::new();
    for chunk in json.split("{\"id\":").skip(1) {
        let id: u32 = match chunk.split(',').next().and_then(|t| t.trim().parse().ok()) {
            Some(v) => v,
            None => continue,
        };
        let kind = chunk
            .split("\"kind\":\"")
            .nth(1)
            .and_then(|t| t.split('"').next())
            .unwrap_or("?")
            .to_string();
        let triple = |key: &str| -> [f64; 3] {
            chunk
                .split(key)
                .nth(1)
                .and_then(|t| t.split(']').next())
                .map(|t| {
                    let v: Vec<f64> =
                        t.split(',').filter_map(|n| n.trim().parse().ok()).collect();
                    [
                        v.first().copied().unwrap_or(f64::NAN),
                        v.get(1).copied().unwrap_or(f64::NAN),
                        v.get(2).copied().unwrap_or(f64::NAN),
                    ]
                })
                .unwrap_or([f64::NAN; 3])
        };
        // A frozen body carries no `vel` key at all — it has no velocity,
        // which is different from having an unreadable one. Reporting the
        // missing field as NaN would have this diagnostic invent three
        // non-finite entities the save does not contain, so absence is
        // reported as zero and only a *present* NaN counts.
        let vel = if chunk.contains("\"vel\":[") { triple("\"vel\":[") } else { [0.0; 3] };
        out.insert(id, (kind, triple("\"pos\":["), vel));
    }
    out
}

fn main() {
    let path = std::env::args().nth(1).expect("a world zip");
    let ticks: u32 = std::env::args().nth(2).and_then(|t| t.parse().ok()).unwrap_or(60);
    let bytes = std::fs::read(&path).expect("read the zip");
    let schem = match Schematic::from_world_zip(&bytes) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("load failed");
            std::process::exit(1);
        }
    };

    // The mode is the whole experiment, so it is an argument rather than a
    // constant. `InWorld` is the default because this sample **is** a world:
    // it was cut out of a save that was already running, so nothing about it
    // was placed and nothing about it should settle. Ticking it under `Quiet`
    // — which this diagnostic did until now — re-writes every block through
    // `place_on_place`, and a build whose every neighbour just appeared is a
    // build whose every observer sees an edge. That, and not the door, was
    // the source of the 68 "spontaneous" block changes.
    let mode = std::env::args().nth(3).unwrap_or_else(|| "in-world".to_string());
    let settle = match mode.as_str() {
        "in-world" => TickSettleMode::InWorld,
        "quiet" => TickSettleMode::Quiet,
        "placement" => TickSettleMode::Placement,
        other => {
            eprintln!("unknown settle mode {other:?} (in-world | quiet | placement)");
            std::process::exit(1);
        }
    };
    println!("settle mode: {mode}");

    let mut sim = match TickSimulation::from_schematic(&schem, settle, 0, 0, 0, b"")
    {
        Ok(s) => s,
        Err(_) => {
            eprintln!(
                "the engine refused this build:\n{}",
                read_out(|w| TickSimulation::last_error_detail(w))
            );
            std::process::exit(2);
        }
    };
    println!("Motion semantics: {}", read_out(|w| sim.motion_semantics(w)));

    let start = entities(&read_out(|w| sim.item_entities_json(w)));
    let non_finite = start.values().filter(|(_, _, v)| v.iter().any(|c| !c.is_finite())).count();
    println!("entities in the simulator: {} ({non_finite} with a non-finite velocity)", start.len());
    for (id, (kind, pos, vel)) in &start {
        // What holds it up, if anything. `floor` because an entity's feet are
        // its `y`, so the cell it stands *on* is the one below that.
        let cell = |dy: f64| {
            let (x, y, z) = (
                pos[0].floor() as i32,
                (pos[1] + dy).floor() as i32,
                pos[2].floor() as i32,
            );
            read_out(|w| sim.get_block(x, y, z, w))
        };
        println!(
            "  id={id:<3} {kind:<28} pos={pos:?} vel={vel:?} in={} below={}",
            cell(0.0),
            cell(-0.5)
        );
    }

    // Nothing is triggered: this is the warmup the door has to survive before
    // anyone touches it. Anything that moves here moved on its own.
    let mut first_move: BTreeMap<u32, u32> = BTreeMap::new();
    let mut lost_nan: BTreeMap<u32, u32> = BTreeMap::new();
    let mut previous = start.clone();
    let mut changes_before = sim.changes_count();
    for tick in 1..=ticks {
        sim.step();
        // A door nobody touched should not be changing blocks. Reporting the
        // tick each change lands on is the difference between "the entities
        // came apart" and "the door actuated itself and then they came apart".
        let changes_now = sim.changes_count();
        if changes_now != changes_before {
            println!("  tick {tick}: {} block change(s)", changes_now - changes_before);
            changes_before = changes_now;
        }
        let now = entities(&read_out(|w| sim.item_entities_json(w)));
        for (id, (_, pos, vel)) in &now {
            if let Some((_, was_pos, was_vel)) = previous.get(id) {
                if pos != was_pos {
                    first_move.entry(*id).or_insert(tick);
                }
                let had = was_vel.iter().any(|c| !c.is_finite());
                let has = vel.iter().any(|c| !c.is_finite());
                if had && !has {
                    lost_nan.entry(*id).or_insert(tick);
                }
            }
        }
        for id in previous.keys() {
            if !now.contains_key(id) {
                println!("  tick {tick}: entity {id} left the world");
            }
        }
        previous = now;
    }

    println!("\nafter {ticks} ticks with nothing triggered:");
    println!("  block changes: {}", sim.changes_count());
    println!("  quiescent: {}", sim.is_quiescent());
    println!("  entities in a retracting piston's sweep: {}", sim.piston_retract_contacts());
    if first_move.is_empty() {
        println!("  no entity moved");
    } else {
        println!("  entities that moved, by the tick they first moved on:");
        for (id, tick) in &first_move {
            let (kind, pos, vel) = &previous[id];
            println!("    tick {tick:<4} id={id:<3} {kind:<28} now pos={pos:?} vel={vel:?}");
        }
    }
    // Vanilla, ticking this same save in place, changes **no** blocks at all
    // (`tools/gametest/captures/door55_in_world.entities.log`). Any change here
    // is the engine actuating a door nobody touched, so name them rather than
    // count them.
    if std::env::var("DUMP_CHANGES").is_ok() {
        let changes = read_out(|w| sim.changes_json(w));
        println!("  changes: {}", &changes[..changes.len().min(4000)]);
    }
    if !lost_nan.is_empty() {
        println!("  entities that lost their non-finite velocity:");
        for (id, tick) in &lost_nan {
            println!("    tick {tick:<4} id={id}");
        }
    }
}
