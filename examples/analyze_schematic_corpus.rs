//! Stream a tar archive of schematics and emit one JSON record per file.
//!
//! This is intended for network-backed corpora: the archive can arrive over
//! SSH, while parsing and metric calculation happen on the local machine.  At
//! most one schematic is held in memory at a time.
//!
//! ```text
//! ssh storage-host 'tar -C /data/builds -cf - schematics' \
//!   | cargo run --release --features world-segment \
//!       --example analyze_schematic_corpus -- metrics.jsonl run-summary.json
//! ```

use nucleation::formats::manager::get_manager;
use nucleation::BlockState;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

#[derive(Debug, Serialize)]
struct ComponentMetrics {
    count: usize,
    substantial_32: usize,
    largest_blocks: usize,
    largest_share: f64,
    singleton_count: usize,
}

#[derive(Debug, Serialize)]
struct SchematicMetrics {
    id: String,
    path: String,
    file_bytes: u64,
    block_count: usize,
    size_x: i32,
    size_y: i32,
    size_z: i32,
    bounding_volume: u64,
    density: f64,
    bytes_per_block: f64,
    palette_names: usize,
    palette_states: usize,
    palette_entropy_bits: f64,
    dominant_block: String,
    dominant_share: f64,
    redstone_blocks: usize,
    redstone_share: f64,
    face_components: ComponentMetrics,
    corner_components: ComponentMetrics,
}

#[derive(Default)]
struct RunStats {
    archives_seen: usize,
    schematics_written: usize,
    parse_failures: usize,
    total_file_bytes: u64,
    total_blocks: u64,
    global_names: FxHashMap<String, u64>,
}

fn is_air(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

fn is_redstone(name: &str) -> bool {
    const TOKENS: &[&str] = &[
        "redstone",
        "repeater",
        "comparator",
        "piston",
        "observer",
        "dispenser",
        "dropper",
        "hopper",
        "target",
        "lever",
        "button",
        "pressure_plate",
        "tripwire",
        "sculk_sensor",
        "daylight_detector",
        "crafter",
        "rail",
        "tnt",
        "note_block",
    ];
    TOKENS.iter().any(|token| name.contains(token))
}

fn component_metrics(
    occupied: &FxHashSet<(i32, i32, i32)>,
    corner_connected: bool,
) -> ComponentMetrics {
    if occupied.is_empty() {
        return ComponentMetrics {
            count: 0,
            substantial_32: 0,
            largest_blocks: 0,
            largest_share: 0.0,
            singleton_count: 0,
        };
    }

    let mut remaining = occupied.clone();
    let mut sizes = Vec::new();
    let mut queue = VecDeque::new();

    while let Some(&seed) = remaining.iter().next() {
        remaining.remove(&seed);
        queue.push_back(seed);
        let mut size = 0usize;

        while let Some((x, y, z)) = queue.pop_front() {
            size += 1;
            for dx in -1i32..=1 {
                for dy in -1i32..=1 {
                    for dz in -1i32..=1 {
                        let manhattan = dx.abs() + dy.abs() + dz.abs();
                        if manhattan == 0 || (!corner_connected && manhattan != 1) {
                            continue;
                        }
                        let neighbour = (x + dx, y + dy, z + dz);
                        if remaining.remove(&neighbour) {
                            queue.push_back(neighbour);
                        }
                    }
                }
            }
        }
        sizes.push(size);
    }

    let largest_blocks = sizes.iter().copied().max().unwrap_or(0);
    ComponentMetrics {
        count: sizes.len(),
        substantial_32: sizes.iter().filter(|&&size| size >= 32).count(),
        largest_blocks,
        largest_share: largest_blocks as f64 / occupied.len() as f64,
        singleton_count: sizes.iter().filter(|&&size| size == 1).count(),
    }
}

fn analyze(
    path: &str,
    file_bytes: u64,
    bytes: &[u8],
) -> Result<(SchematicMetrics, FxHashMap<String, usize>), String> {
    let schematic = get_manager()
        .lock()
        .map_err(|_| "format manager lock poisoned".to_string())?
        .read(bytes)
        .map_err(|error| format!("parse failed: {error}"))?;

    let mut occupied = FxHashSet::default();
    let mut name_counts: FxHashMap<String, usize> = FxHashMap::default();
    let mut state_counts: FxHashMap<BlockState, usize> = FxHashMap::default();
    let mut min = (i32::MAX, i32::MAX, i32::MAX);
    let mut max = (i32::MIN, i32::MIN, i32::MIN);
    let mut redstone_blocks = 0usize;

    for (position, block) in schematic.iter_blocks() {
        let name = block.name.as_str();
        if is_air(name) {
            continue;
        }
        let point = (position.x, position.y, position.z);
        // A multi-region schematic may expose the same world coordinate more
        // than once. Metrics describe occupied coordinates, not region entries.
        if !occupied.insert(point) {
            continue;
        }
        *name_counts.entry(name.to_string()).or_default() += 1;
        *state_counts.entry(block.clone()).or_default() += 1;
        if is_redstone(name) {
            redstone_blocks += 1;
        }
        min.0 = min.0.min(point.0);
        min.1 = min.1.min(point.1);
        min.2 = min.2.min(point.2);
        max.0 = max.0.max(point.0);
        max.1 = max.1.max(point.1);
        max.2 = max.2.max(point.2);
    }

    let block_count = occupied.len();
    let (size_x, size_y, size_z) = if block_count == 0 {
        (0, 0, 0)
    } else {
        (max.0 - min.0 + 1, max.1 - min.1 + 1, max.2 - min.2 + 1)
    };
    let bounding_volume = (size_x as u64)
        .saturating_mul(size_y as u64)
        .saturating_mul(size_z as u64);
    let density = if bounding_volume == 0 {
        0.0
    } else {
        block_count as f64 / bounding_volume as f64
    };
    let bytes_per_block = if block_count == 0 {
        0.0
    } else {
        file_bytes as f64 / block_count as f64
    };

    let mut entropy = 0.0;
    let mut dominant_block = String::new();
    let mut dominant_count = 0usize;
    for (name, &count) in &name_counts {
        let p = count as f64 / block_count.max(1) as f64;
        entropy -= p * p.log2();
        if count > dominant_count || (count == dominant_count && name < &dominant_block) {
            dominant_count = count;
            dominant_block.clone_from(name);
        }
    }

    let id = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path)
        .to_string();

    let metric = SchematicMetrics {
        id,
        path: path.to_string(),
        file_bytes,
        block_count,
        size_x,
        size_y,
        size_z,
        bounding_volume,
        density,
        bytes_per_block,
        palette_names: name_counts.len(),
        palette_states: state_counts.len(),
        palette_entropy_bits: entropy,
        dominant_block,
        dominant_share: dominant_count as f64 / block_count.max(1) as f64,
        redstone_blocks,
        redstone_share: redstone_blocks as f64 / block_count.max(1) as f64,
        face_components: component_metrics(&occupied, false),
        corner_components: component_metrics(&occupied, true),
    };
    Ok((metric, name_counts))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .ok_or("usage: analyze_schematic_corpus <metrics.jsonl> [run-summary.json]")?;
    let summary_output = std::env::args().nth(2);
    let output_file = File::create(&output)?;
    let mut writer = BufWriter::new(output_file);
    let stdin = io::stdin();
    let mut archive = tar::Archive::new(stdin.lock());
    let mut stats = RunStats::default();

    for entry in archive.entries()? {
        stats.archives_seen += 1;
        let mut entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                stats.parse_failures += 1;
                eprintln!("archive entry error: {error}");
                continue;
            }
        };
        let path = entry.path()?.to_string_lossy().to_string();
        if !path.ends_with(".schem") {
            continue;
        }
        let file_bytes = entry.size();
        let mut bytes = Vec::with_capacity(file_bytes as usize);
        io::copy(&mut entry, &mut bytes)?;

        match analyze(&path, file_bytes, &bytes) {
            Ok((metric, name_counts)) => {
                stats.schematics_written += 1;
                stats.total_file_bytes += metric.file_bytes;
                stats.total_blocks += metric.block_count as u64;
                for (name, count) in name_counts {
                    *stats.global_names.entry(name).or_default() += count as u64;
                }
                serde_json::to_writer(&mut writer, &metric)?;
                writer.write_all(b"\n")?;
            }
            Err(error) => {
                stats.parse_failures += 1;
                eprintln!("{path}: {error}");
            }
        }

        if stats.schematics_written > 0 && stats.schematics_written % 1_000 == 0 {
            eprintln!("analyzed {} schematics", stats.schematics_written);
        }
    }
    writer.flush()?;

    let mut top_blocks: Vec<_> = stats.global_names.into_iter().collect();
    top_blocks.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_blocks.truncate(50);
    let run_summary = serde_json::json!({
        "output": output,
        "archive_entries_seen": stats.archives_seen,
        "schematics_written": stats.schematics_written,
        "parse_failures": stats.parse_failures,
        "total_file_bytes": stats.total_file_bytes,
        "total_blocks": stats.total_blocks,
        "top_blocks": top_blocks,
    });
    if let Some(summary_path) = summary_output {
        serde_json::to_writer_pretty(File::create(summary_path)?, &run_summary)?;
    }
    println!("{run_summary}");
    Ok(())
}
