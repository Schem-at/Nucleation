//! What the record 3x3 piston door's *doorway* holds, tick by tick.
//!
//! ```text
//! cargo run --release --example door55_doorway --features bridge,mc-tick -- \
//!     <world.zip> [press_tick[,press_tick...]] [ticks]
//! ```
//!
//! `door55_sim` reports the entities and `door55_xray` reports the causal chain;
//! neither answers the only question that decides whether the door works, which
//! is whether the nine cells of the hole fill and then empty again. This reads
//! those nine cells with `get_block` **every tick** rather than reconstructing
//! them from `changes_json` — which is known to report a push past the region
//! edge as a success, and so cannot be trusted to say a cell filled.
//!
//! It also prints the whole-world block state at the start and at the end, so
//! "the world returned to its start state" is a claim with a number behind it.

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

/// The hole the door is supposed to open and close: three wide, three tall, one
/// deep, in the wall's own plane.
const DOORWAY_X: [i32; 3] = [67, 68, 69];
const DOORWAY_Y: [i32; 3] = [0, 1, 2];
const DOORWAY_Z: i32 = 20;

/// The whole build, so a return to the start state is checkable.
///
/// `y` starts at `-2` on purpose: two of the down-facing pistons push a quartz
/// block to `y = -1`, below the floor the extraction gave us, and a scan that
/// started at 0 would report those blocks as having vanished.
const WORLD_X: std::ops::RangeInclusive<i32> = 55..=90;
const WORLD_Y: std::ops::RangeInclusive<i32> = -2..=12;
const WORLD_Z: std::ops::RangeInclusive<i32> = 14..=26;

/// Everything the extracted region declares, for the census. `door55_xray`
/// only ever looked at the slab around the mechanism, which is how "there is no
/// 3x3 panel in this sample" stayed invisible for so long.
const CENSUS_X: std::ops::RangeInclusive<i32> = -2..=104;
const CENSUS_Y: std::ops::RangeInclusive<i32> = -2..=8;
const CENSUS_Z: std::ops::RangeInclusive<i32> = -2..=67;

fn snapshot(sim: &TickSimulation) -> BTreeMap<(i32, i32, i32), String> {
    let mut out = BTreeMap::new();
    for x in WORLD_X {
        for y in WORLD_Y.clone() {
            for z in WORLD_Z.clone() {
                let s = read_out(|w| sim.get_block(x, y, z, w));
                if s != "minecraft:air" && !s.is_empty() {
                    out.insert((x, y, z), s);
                }
            }
        }
    }
    out
}

/// The doorway as a 3x3 picture, top row first, plus the filled count.
fn doorway(sim: &TickSimulation) -> (usize, String, Vec<String>) {
    let mut filled = 0;
    let mut picture = String::new();
    let mut names = Vec::new();
    for y in DOORWAY_Y.iter().rev() {
        for x in DOORWAY_X {
            let s = read_out(|w| sim.get_block(x, *y, DOORWAY_Z, w));
            let solid = s != "minecraft:air" && !s.is_empty();
            if solid {
                filled += 1;
            }
            picture.push(if solid { '#' } else { '.' });
            names.push(format!("({x},{y})={}", s.replace("minecraft:", "")));
        }
        picture.push('/');
    }
    (filled, picture, names)
}

fn main() {
    let path = std::env::args().nth(1).expect("a world zip");
    let presses: Vec<u32> = std::env::args()
        .nth(2)
        .map(|s| s.split(',').filter_map(|p| p.parse().ok()).collect())
        .unwrap_or_else(|| vec![5]);
    let ticks: u32 = std::env::args().nth(3).and_then(|t| t.parse().ok()).unwrap_or(120);

    let bytes = std::fs::read(&path).expect("read the zip");
    let schem = Schematic::from_data(&bytes).expect("parse the world");
    let mut sim = TickSimulation::from_schematic(&schem, TickSettleMode::InWorld, 0, 0, 0, b"")
        .unwrap_or_else(|_| {
            eprintln!("refused: {}", read_out(TickSimulation::last_error_detail));
            std::process::exit(2)
        });

    let start_world = snapshot(&sim);
    println!("# blocks at rest: {}", start_world.len());
    // The census: is there a 3x3 panel in this sample at all? A doorway count
    // is meaningless if the cells being counted are not a door.
    if std::env::var("CENSUS").is_ok() {
        let mut all: Vec<((i32, i32, i32), String)> = Vec::new();
        for x in CENSUS_X {
            for y in CENSUS_Y.clone() {
                for z in CENSUS_Z.clone() {
                    let s = read_out(|w| sim.get_block(x, y, z, w));
                    if s != "minecraft:air" && !s.is_empty() {
                        all.push(((x, y, z), s));
                    }
                }
            }
        }
        println!("# CENSUS: {} blocks in the whole declared region", all.len());
        let mut by_z: BTreeMap<i32, usize> = BTreeMap::new();
        for ((_, _, z), _) in &all {
            *by_z.entry(*z).or_default() += 1;
        }
        println!("# CENSUS by z: {by_z:?}");
        for (p, s) in &all {
            if !(WORLD_Z.contains(&p.2)) {
                println!("CENSUS-OUTSIDE {} {} {} {}", p.0, p.1, p.2, s.replace("minecraft:", ""));
            }
        }
    }
    let button = start_world
        .iter()
        .find(|(_, s)| s.contains("_button"))
        .map(|(p, _)| *p)
        .expect("a button");
    println!("# button at {button:?}, pressing on {presses:?}, running {ticks} ticks");

    let (filled, picture, names) = doorway(&sim);
    println!("t0   filled={filled}/9  {picture}");
    println!("     {}", names.join(" "));
    let mut previous = picture;
    let mut best = filled;
    let mut best_tick = 0;
    let mut closed_tick = None;
    let mut reopened_tick = None;

    for tick in 1..=ticks {
        if presses.contains(&tick) {
            println!("t{tick}   pressing the button");
            sim.use_block(button.0, button.1, button.2);
        }
        sim.step();
        let (filled, picture, names) = doorway(&sim);
        if picture != previous {
            println!("t{tick}   filled={filled}/9  {picture}");
            println!("     {}", names.join(" "));
            previous = picture;
        }
        if filled > best {
            best = filled;
            best_tick = tick;
        }
        if filled == 9 && closed_tick.is_none() {
            closed_tick = Some(tick);
        }
        if filled == 0 && closed_tick.is_some() && reopened_tick.is_none() {
            reopened_tick = Some(tick);
        }
    }

    println!("\n# best fill: {best}/9 on tick {best_tick}");
    println!("# fully closed on: {closed_tick:?}");
    println!("# fully open again on: {reopened_tick:?}");
    println!("# block changes: {}", sim.changes_count());
    println!("# quiescent: {}", sim.is_quiescent());
    println!("# retract contacts: {}", sim.piston_retract_contacts());

    let end_world = snapshot(&sim);
    let mut away = 0;
    for (p, s) in &start_world {
        if end_world.get(p) != Some(s) {
            away += 1;
            if away <= 30 {
                println!(
                    "AWAY {} {} {} was={} now={}",
                    p.0,
                    p.1,
                    p.2,
                    s.replace("minecraft:", ""),
                    end_world.get(p).map(|s| s.replace("minecraft:", "")).unwrap_or("air".into())
                );
            }
        }
    }
    for (p, s) in &end_world {
        if !start_world.contains_key(p) {
            away += 1;
            if away <= 30 {
                println!("AWAY {} {} {} was=air now={}", p.0, p.1, p.2, s.replace("minecraft:", ""));
            }
        }
    }
    println!("# cells away from the start state: {away}");
}
