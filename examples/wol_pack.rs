//! wol_pack — the **query → pack → world** direction of WoL (inverse of
//! `wol_extract`). Given a set of `.schem` files (the result of a tag query)
//! plus optional placements, stream each schematic's non-air blocks into a
//! Minecraft world via Nucleation's streaming `WorldSink`.
//!
//! The core primitive (`nucleation::formats::world_pack`) is domain-agnostic:
//! it knows nothing about queries, tags, or ORE — it consumes opaque
//! `(key, offset, bbox)` placements and a lazy `load` closure. This driver is
//! the thin WoL-flavoured shell that turns `.schem` paths into placements and
//! reports on the run, mirroring `wol_extract`'s provenance-style output.
//!
//! ## Usage
//!
//! ```text
//! wol_pack --out <world_dir> [--spacing N] [--base-y Y] <path.schem[@x,y,z]> ...
//! ```
//!
//! * A bare `path.schem` is laid out automatically on a deterministic grid.
//! * `path.schem@x,y,z` pins an explicit world offset (skips layout for that one).
//! * If *any* explicit offset is given, all inputs must be explicit (mixed mode
//!   is rejected to keep placement provenance unambiguous).
//!
//! Emits a standard Anvil world directory (`region/*.mca` + `level.dat`) at
//! `--out`, then re-reads it back and asserts the placed blocks are present at
//! their expected world coordinates (the round-trip proof).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use nucleation::formats::schematic::from_schematic;
use nucleation::formats::world_pack::{grid_layout, pack, Placement};
use nucleation::formats::world_stream::{WorldSink, WorldSource};
use nucleation::{BoundingBox, UniversalSchematic};

struct Cli {
    out: PathBuf,
    spacing: i32,
    base_y: i32,
    /// (key, path, explicit offset)
    inputs: Vec<(String, PathBuf, Option<(i32, i32, i32)>)>,
}

fn parse_args() -> Cli {
    let mut out = PathBuf::from("wol_pack_world");
    let mut spacing = 1;
    let mut base_y = 64;
    let mut inputs = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out = PathBuf::from(args.next().expect("--out needs a value")),
            "--spacing" => spacing = args.next().expect("--spacing needs a value").parse().unwrap(),
            "--base-y" => base_y = args.next().expect("--base-y needs a value").parse().unwrap(),
            other => {
                // path[@x,y,z]
                let (path_str, offset) = match other.split_once('@') {
                    Some((p, coords)) => {
                        let nums: Vec<i32> = coords
                            .split(',')
                            .map(|s| s.trim().parse().expect("offset must be x,y,z ints"))
                            .collect();
                        assert_eq!(nums.len(), 3, "offset must be x,y,z");
                        (p.to_string(), Some((nums[0], nums[1], nums[2])))
                    }
                    None => (other.to_string(), None),
                };
                let path = PathBuf::from(&path_str);
                // Stable key: file stem (falls back to the full path).
                let key = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&path_str)
                    .to_string();
                inputs.push((key, path, offset));
            }
        }
    }
    if inputs.is_empty() {
        eprintln!(
            "usage: wol_pack --out <dir> [--spacing N] [--base-y Y] <path.schem[@x,y,z]> ..."
        );
        std::process::exit(2);
    }
    Cli {
        out,
        spacing,
        base_y,
        inputs,
    }
}

/// Load a `.schem` file into a `UniversalSchematic`.
fn load_schem(path: &Path) -> UniversalSchematic {
    let bytes = std::fs::read(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    from_schematic(&bytes)
        .unwrap_or_else(|e| panic!("failed to parse {} as .schem: {e}", path.display()))
}

fn main() {
    let cli = parse_args();
    let started = Instant::now();

    println!("wol_pack: {} input schematic(s)", cli.inputs.len());
    let any_explicit = cli.inputs.iter().any(|(_, _, o)| o.is_some());
    let all_explicit = cli.inputs.iter().all(|(_, _, o)| o.is_some());
    if any_explicit && !all_explicit {
        panic!("wol_pack: mixed explicit/auto placement is not allowed — give every input an @offset or none");
    }

    // key -> path, so the pack `load` closure can lazily re-read one at a time.
    let paths: HashMap<String, PathBuf> = cli
        .inputs
        .iter()
        .map(|(k, p, _)| (k.clone(), p.clone()))
        .collect();

    // Build placements. Auto mode: read each schematic's bbox once (blocks
    // dropped immediately), then grid-lay-out. Explicit mode: use given offsets.
    let placements: Vec<Placement> = if all_explicit {
        cli.inputs
            .iter()
            .map(|(key, path, off)| {
                let s = load_schem(path);
                Placement {
                    key: key.clone(),
                    offset: off.unwrap(),
                    local_bbox: s.get_bounding_box(),
                }
            })
            .collect()
    } else {
        let items: Vec<(String, BoundingBox)> = cli
            .inputs
            .iter()
            .map(|(key, path, _)| {
                let s = load_schem(path);
                (key.clone(), s.get_bounding_box())
            })
            .collect();
        grid_layout(&items, cli.spacing, cli.base_y)
    };

    for p in &placements {
        println!("wol_pack:   placement key={:?} offset={:?}", p.key, p.offset);
    }

    // --- pack: stream one schematic at a time into the sink ---
    std::fs::create_dir_all(&cli.out)
        .unwrap_or_else(|e| panic!("failed to create out dir {}: {e}", cli.out.display()));
    let mut sink = WorldSink::create(&cli.out, None)
        .unwrap_or_else(|e| panic!("failed to create WorldSink at {}: {e}", cli.out.display()));

    let stats = pack(
        &placements,
        |p| {
            let path = paths.get(&p.key).expect("placement key has a path");
            Ok(load_schem(path))
        },
        &mut sink,
    )
    .expect("pack failed");
    sink.finish().expect("WorldSink::finish failed");

    let elapsed = started.elapsed();
    println!();
    println!("wol_pack: pack complete.");
    println!("wol_pack:   schematics      = {}", stats.schematics);
    println!("wol_pack:   blocks written  = {}", stats.blocks_written);
    println!("wol_pack:   chunks written  = {}", stats.chunks_written);
    match &stats.bounds {
        Some(bb) => println!(
            "wol_pack:   world bounds     = x[{}..{}] y[{}..{}] z[{}..{}]",
            bb.min.0, bb.max.0, bb.min.1, bb.max.1, bb.min.2, bb.max.2
        ),
        None => println!("wol_pack:   world bounds     = (empty)"),
    }
    println!(
        "wol_pack:   peak live chunks = {}  (memory proxy: never buffers the whole world)",
        stats.peak_live_chunks
    );
    println!("wol_pack:   duration        = {:.3?}", elapsed);
    println!("wol_pack:   world written to {}", cli.out.display());

    // --- round-trip proof: re-read the world and check every placed block ---
    println!();
    println!("wol_pack: round-trip verification (re-reading {})...", cli.out.display());
    let world = read_world(&cli.out);
    let mut checked: u64 = 0;
    let mut missing: u64 = 0;
    for p in &placements {
        let s = load_schem(paths.get(&p.key).unwrap());
        let (ox, oy, oz) = p.offset;
        for (bp, block) in s.iter_blocks() {
            let name = block.name.as_str();
            if matches!(name, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air") {
                continue;
            }
            let coord = (bp.x + ox, bp.y + oy, bp.z + oz);
            match world.get(&coord) {
                Some(got) if got == name => checked += 1,
                other => {
                    // In auto (non-overlapping) mode this must never happen; in
                    // explicit overlap mode a cell may be legitimately won by a
                    // later placement, so only count genuine absences.
                    if !all_explicit || other.is_none() {
                        missing += 1;
                        if missing <= 5 {
                            eprintln!(
                                "wol_pack:   MISMATCH at {:?}: expected {name}, got {:?}",
                                coord, other
                            );
                        }
                    } else {
                        checked += 1; // overwritten by a later placement — expected
                    }
                }
            }
        }
    }
    println!("wol_pack:   verified {checked} placed cells present, {missing} missing");
    if missing == 0 {
        println!("wol_pack: ROUND-TRIP OK");
    } else {
        eprintln!("wol_pack: ROUND-TRIP FAILED ({missing} cells missing)");
        std::process::exit(1);
    }
}

/// Read a packed world into a map of world-coord -> block name.
fn read_world(dir: &Path) -> HashMap<(i32, i32, i32), String> {
    let source = WorldSource::open_dir(dir).expect("open packed world");
    let mut out = HashMap::new();
    for chunk in source.chunks().expect("iterate chunks") {
        let chunk = chunk.expect("decode chunk");
        for (x, y, z, state) in chunk.blocks() {
            out.insert((x, y, z), state.name.to_string());
        }
    }
    out
}
