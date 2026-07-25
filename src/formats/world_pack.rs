//! Pack schematics into a Minecraft world via the streaming [`WorldSink`].
//!
//! This is the inverse of world *extraction* (`world → builds`): given a set of
//! schematics plus placements (`builds → world`), stream each schematic's
//! non-air blocks into a world at its placement, producing a persisted world
//! directory without ever materialising the whole world in RAM.
//!
//! ## Generality
//!
//! The API is deliberately domain-agnostic (per the standing Generality Law):
//! inputs are opaque `(key, offset, bbox)` placements plus a caller-supplied
//! lazy `load` closure. There are no query / tag / ORE / "wol" concepts here —
//! a tag-query result is just one possible *producer* of the placement list.
//!
//! ## Streaming model
//!
//! [`pack`] processes placements in a deterministic order and holds **at most
//! one schematic in memory at a time** (loaded on demand through the `load`
//! closure, dropped before the next). Chunk views are flushed to the sink as
//! soon as the last placement that can touch them has been processed, so the
//! live working set is bounded by the *overlap span* of the layout — for the
//! default non-overlapping grid layout that is exactly one schematic's chunks.
//!
//! ## Overlap rule
//!
//! Placements are totally ordered by `(key, offset)` ascending and processed in
//! that order. When two placements write the same world cell, the **later**
//! placement wins ("last placement wins, per block"). This is deterministic and
//! independent of the order the placements were supplied in.
//!
//! ## Capability note
//!
//! [`WorldSink`]/[`WorldChunkView`] express a pure block translation only: there
//! is no block-state *rotation* primitive at the chunk-view level, so placements
//! carry an offset (translation) but **not** a rotation. Rotating a build must
//! be done upstream (rotate the `UniversalSchematic` before layout). This is a
//! flagged gap, intentionally left out of scope.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;

use crate::bounding_box::BoundingBox;
use crate::formats::world_stream::{WorldChunkView, WorldSink};
use crate::universal_schematic::UniversalSchematic;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const CHUNK: i32 = 16;

/// Integer floor-division (matches `world_stream`'s chunk math for negatives).
fn floor_div(a: i32, b: i32) -> i32 {
    let d = a / b;
    let r = a % b;
    if r != 0 && (r < 0) != (b < 0) {
        d - 1
    } else {
        d
    }
}

fn is_air(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

/// One placement of a schematic into the world.
///
/// `offset` is a pure world-space translation added to every block position of
/// the schematic. `local_bbox` is the schematic's own bounding box
/// (`UniversalSchematic::get_bounding_box`) and lets [`pack`] plan chunk
/// coverage *without* loading the schematic's blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    /// Stable, opaque identifier — the primary key for the total ordering and
    /// the overlap tie-break. Should be unique per placement.
    pub key: String,
    /// World-space translation added to every block of the schematic.
    pub offset: (i32, i32, i32),
    /// The schematic's local bounding box (used for coverage planning only).
    pub local_bbox: BoundingBox,
}

impl Placement {
    /// World-space bounding box occupied by this placement.
    pub fn world_bbox(&self) -> BoundingBox {
        let (ox, oy, oz) = self.offset;
        BoundingBox::new(
            (
                self.local_bbox.min.0 + ox,
                self.local_bbox.min.1 + oy,
                self.local_bbox.min.2 + oz,
            ),
            (
                self.local_bbox.max.0 + ox,
                self.local_bbox.max.1 + oy,
                self.local_bbox.max.2 + oz,
            ),
        )
    }

    /// Inclusive chunk-column range `(cx0, cz0, cx1, cz1)` this placement covers.
    fn chunk_span(&self) -> (i32, i32, i32, i32) {
        let wb = self.world_bbox();
        (
            floor_div(wb.min.0, CHUNK),
            floor_div(wb.min.2, CHUNK),
            floor_div(wb.max.0, CHUNK),
            floor_div(wb.max.2, CHUNK),
        )
    }
}

/// Outcome statistics for a pack run.
#[derive(Debug, Clone, PartialEq)]
pub struct PackStats {
    /// Number of placements processed.
    pub schematics: usize,
    /// Non-air blocks written to the world (post-overlap-resolution, so a cell
    /// overwritten by a later placement is counted once).
    pub blocks_written: u64,
    /// Distinct chunk columns written.
    pub chunks_written: usize,
    /// World-space bounding box of all placed non-air blocks (`None` if empty).
    pub bounds: Option<BoundingBox>,
    /// High-water mark of simultaneously-live chunk views — a memory proxy that
    /// demonstrates the packer never buffers the whole world.
    pub peak_live_chunks: usize,
}

/// Pack `placements` into `sink`, loading each schematic lazily via `load`.
///
/// `load` is called at most once per placement and its result is dropped before
/// the next placement is loaded — so peak schematic memory is a single build.
/// Returns after the last chunk is flushed; the caller still owns `sink` and
/// must call [`WorldSink::finish`] to write `level.dat` and flush the final
/// region buffer.
pub fn pack<L>(placements: &[Placement], mut load: L, sink: &mut WorldSink) -> Result<PackStats>
where
    L: FnMut(&Placement) -> Result<UniversalSchematic>,
{
    // Total, order-independent processing order: sort indices by (key, offset).
    let mut order: Vec<usize> = (0..placements.len()).collect();
    order.sort_by(|&a, &b| {
        let pa = &placements[a];
        let pb = &placements[b];
        pa.key
            .cmp(&pb.key)
            .then_with(|| pa.offset.cmp(&pb.offset))
    });

    // Plan: for each chunk column, the *last* processing position that touches
    // it. A chunk can be flushed the moment we finish that position.
    let mut last_touch: HashMap<(i32, i32), usize> = HashMap::new();
    for (pos, &idx) in order.iter().enumerate() {
        let (cx0, cz0, cx1, cz1) = placements[idx].chunk_span();
        for cx in cx0..=cx1 {
            for cz in cz0..=cz1 {
                last_touch.insert((cx, cz), pos);
            }
        }
    }

    let mut live: HashMap<(i32, i32), WorldChunkView> = HashMap::new();
    let mut stats = PackStats {
        schematics: placements.len(),
        blocks_written: 0,
        chunks_written: 0,
        bounds: None,
        peak_live_chunks: 0,
    };

    for (pos, &idx) in order.iter().enumerate() {
        let placement = &placements[idx];
        let (ox, oy, oz) = placement.offset;

        // --- load ONE schematic, place its blocks, drop it ---
        let schematic = load(placement)?;
        for (bp, block) in schematic.iter_blocks() {
            if is_air(&block.name) {
                continue;
            }
            let (wx, wy, wz) = (bp.x + ox, bp.y + oy, bp.z + oz);
            let (cx, cz) = (floor_div(wx, CHUNK), floor_div(wz, CHUNK));
            let view = live
                .entry((cx, cz))
                .or_insert_with(|| WorldChunkView::new(cx, cz));
            view.set_block(wx, wy, wz, block);

            stats.bounds = Some(match stats.bounds.take() {
                None => BoundingBox::new((wx, wy, wz), (wx, wy, wz)),
                Some(bb) => BoundingBox::new(
                    (bb.min.0.min(wx), bb.min.1.min(wy), bb.min.2.min(wz)),
                    (bb.max.0.max(wx), bb.max.1.max(wy), bb.max.2.max(wz)),
                ),
            });
        }
        drop(schematic);

        stats.peak_live_chunks = stats.peak_live_chunks.max(live.len());

        // --- flush every chunk whose last toucher was this position ---
        let mut ready: Vec<(i32, i32)> = live
            .keys()
            .copied()
            .filter(|c| last_touch.get(c) == Some(&pos))
            .collect();
        ready.sort(); // deterministic write order
        for c in ready {
            let view = live.remove(&c).expect("ready chunk is live");
            stats.blocks_written += view.blocks().count() as u64;
            sink.write_chunk(&view)?;
            stats.chunks_written += 1;
        }
    }

    // Safety net: anything still live (should not happen — every chunk has a
    // last toucher) is flushed in deterministic order.
    let mut leftover: Vec<(i32, i32)> = live.keys().copied().collect();
    leftover.sort();
    for c in leftover {
        let view = live.remove(&c).unwrap();
        stats.blocks_written += view.blocks().count() as u64;
        sink.write_chunk(&view)?;
        stats.chunks_written += 1;
    }

    Ok(stats)
}

/// Deterministically lay out `items` into a non-overlapping grid of placements.
///
/// Each item is `(key, local_bbox)`. Items are sorted by `key` first, so the
/// result is **independent of input order**. Cells use a uniform stride equal to
/// the largest footprint (in chunks) plus `spacing_chunks`, and every schematic
/// is chunk-aligned at its cell origin — guaranteeing no two placements ever
/// share a chunk column (which keeps [`pack`]'s live set to a single schematic).
///
/// `base_y` is the world Y that every schematic's minimum-Y corner is mapped to.
pub fn grid_layout(
    items: &[(String, BoundingBox)],
    spacing_chunks: i32,
    base_y: i32,
) -> Vec<Placement> {
    if items.is_empty() {
        return Vec::new();
    }
    let mut sorted: Vec<&(String, BoundingBox)> = items.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    // Footprint in chunks for a bbox width (aligned min → span = ceil(w/16)).
    let chunks_of = |lo: i32, hi: i32| -> i32 { (hi - lo).max(0) / CHUNK + 1 };
    let max_w = sorted
        .iter()
        .map(|(_, bb)| chunks_of(bb.min.0, bb.max.0))
        .max()
        .unwrap_or(1);
    let max_l = sorted
        .iter()
        .map(|(_, bb)| chunks_of(bb.min.2, bb.max.2))
        .max()
        .unwrap_or(1);
    let stride_x = (max_w + spacing_chunks.max(0)) * CHUNK;
    let stride_z = (max_l + spacing_chunks.max(0)) * CHUNK;

    let cols = (sorted.len() as f64).sqrt().ceil() as i32;
    let cols = cols.max(1);

    sorted
        .iter()
        .enumerate()
        .map(|(i, (key, bb))| {
            let col = (i as i32) % cols;
            let row = (i as i32) / cols;
            // Cell origin is chunk-aligned; offset maps the schematic's local
            // min corner to (cell_x, base_y, cell_z).
            let cell_x = col * stride_x;
            let cell_z = row * stride_z;
            let offset = (cell_x - bb.min.0, base_y - bb.min.1, cell_z - bb.min.2);
            Placement {
                key: key.clone(),
                offset,
                local_bbox: bb.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::world_stream::WorldSource;

    fn schem(name: &str, cells: &[(i32, i32, i32, &str)]) -> UniversalSchematic {
        let mut s = UniversalSchematic::new(name.to_string());
        for &(x, y, z, b) in cells {
            s.set_block_str(x, y, z, b);
        }
        s
    }

    /// Read a packed world back into a map of world-coord -> block name.
    fn read_world(dir: &std::path::Path) -> HashMap<(i32, i32, i32), String> {
        let source = WorldSource::open_dir(dir).expect("open world");
        let mut out = HashMap::new();
        for chunk in source.chunks().expect("chunks") {
            let chunk = chunk.expect("chunk decode");
            for (x, y, z, state) in chunk.blocks() {
                out.insert((x, y, z), state.name.to_string());
            }
        }
        out
    }

    fn place(key: &str, off: (i32, i32, i32), s: &UniversalSchematic) -> Placement {
        Placement {
            key: key.to_string(),
            offset: off,
            local_bbox: s.get_bounding_box(),
        }
    }

    #[test]
    fn packs_blocks_at_placement_plus_local() {
        let a = schem("a", &[(0, 0, 0, "minecraft:stone"), (1, 0, 0, "minecraft:dirt")]);
        let b = schem("b", &[(0, 0, 0, "minecraft:gold_block")]);
        let placements = vec![
            place("a", (0, 70, 0), &a),
            place("b", (40, 70, 40), &b),
        ];

        let dir = tempdir();
        let mut sink = WorldSink::create(&dir, None).unwrap();
        let loads = [("a", &a), ("b", &b)];
        let stats = pack(
            &placements,
            |p| {
                let s = loads.iter().find(|(k, _)| *k == p.key).unwrap().1;
                Ok(s.clone())
            },
            &mut sink,
        )
        .unwrap();
        sink.finish().unwrap();

        assert_eq!(stats.blocks_written, 3);
        let world = read_world(&dir);
        assert_eq!(world.get(&(0, 70, 0)).map(String::as_str), Some("minecraft:stone"));
        assert_eq!(world.get(&(1, 70, 0)).map(String::as_str), Some("minecraft:dirt"));
        assert_eq!(world.get(&(40, 70, 40)).map(String::as_str), Some("minecraft:gold_block"));
        cleanup(&dir);
    }

    #[test]
    fn air_cells_are_not_written() {
        // A schematic where set_block leaves gaps (unset = air) and an explicit air block.
        let mut a = schem("a", &[(0, 0, 0, "minecraft:stone"), (2, 0, 0, "minecraft:stone")]);
        a.set_block_str(1, 0, 0, "minecraft:air");
        let placements = vec![place("a", (0, 64, 0), &a)];

        let dir = tempdir();
        let mut sink = WorldSink::create(&dir, None).unwrap();
        pack(&placements, |_| Ok(a.clone()), &mut sink).unwrap();
        sink.finish().unwrap();

        let world = read_world(&dir);
        assert_eq!(world.get(&(0, 64, 0)).map(String::as_str), Some("minecraft:stone"));
        assert_eq!(world.get(&(2, 64, 0)).map(String::as_str), Some("minecraft:stone"));
        assert!(world.get(&(1, 64, 0)).is_none(), "air must not be written");
        cleanup(&dir);
    }

    #[test]
    fn overlap_resolves_last_placement_wins() {
        // Two placements target the SAME world cell in the SAME chunk.
        let a = schem("a", &[(0, 0, 0, "minecraft:stone")]);
        let b = schem("b", &[(0, 0, 0, "minecraft:gold_block")]);
        // key "b" > "a" so b is processed later and must win.
        let placements = vec![
            place("b", (5, 64, 5), &b),
            place("a", (5, 64, 5), &a), // supplied first, but ordered first by key
        ];

        let dir = tempdir();
        let mut sink = WorldSink::create(&dir, None).unwrap();
        pack(
            &placements,
            |p| Ok(if p.key == "a" { a.clone() } else { b.clone() }),
            &mut sink,
        )
        .unwrap();
        sink.finish().unwrap();

        let world = read_world(&dir);
        assert_eq!(
            world.get(&(5, 64, 5)).map(String::as_str),
            Some("minecraft:gold_block"),
            "later placement (key b) must win"
        );
        cleanup(&dir);
    }

    #[test]
    fn determinism_same_inputs_same_world() {
        let a = schem("a", &[(0, 0, 0, "minecraft:stone")]);
        let b = schem("b", &[(0, 0, 0, "minecraft:dirt")]);
        let items = vec![
            ("a".to_string(), a.get_bounding_box()),
            ("b".to_string(), b.get_bounding_box()),
        ];
        let run = |placements: &[Placement]| -> HashMap<(i32, i32, i32), String> {
            let dir = tempdir();
            let mut sink = WorldSink::create(&dir, None).unwrap();
            pack(
                placements,
                |p| Ok(if p.key == "a" { a.clone() } else { b.clone() }),
                &mut sink,
            )
            .unwrap();
            sink.finish().unwrap();
            let w = read_world(&dir);
            cleanup(&dir);
            w
        };
        let p1 = grid_layout(&items, 1, 64);
        // Reversed input order must yield identical placements (order-independent).
        let mut rev = items.clone();
        rev.reverse();
        let p2 = grid_layout(&rev, 1, 64);
        assert_eq!(p1, p2, "layout must be independent of input order");
        assert_eq!(run(&p1), run(&p2), "same inputs -> identical world");
    }

    #[test]
    fn streaming_live_set_is_bounded() {
        // 9 single-chunk schematics laid out on a grid: peak live chunks must
        // stay ~1 (one schematic at a time), never 9.
        let mut items = Vec::new();
        let mut map = HashMap::new();
        for i in 0..9 {
            let key = format!("s{i}");
            let s = schem(&key, &[(0, 0, 0, "minecraft:stone")]);
            items.push((key.clone(), s.get_bounding_box()));
            map.insert(key, s);
        }
        let placements = grid_layout(&items, 1, 64);
        let dir = tempdir();
        let mut sink = WorldSink::create(&dir, None).unwrap();
        let stats = pack(&placements, |p| Ok(map.get(&p.key).unwrap().clone()), &mut sink).unwrap();
        sink.finish().unwrap();

        assert_eq!(stats.schematics, 9);
        assert_eq!(stats.blocks_written, 9);
        assert!(
            stats.peak_live_chunks <= 1,
            "grid layout must keep the live set to one schematic's chunks, got {}",
            stats.peak_live_chunks
        );
        cleanup(&dir);
    }

    // --- tiny temp-dir helpers (avoid a dev-dependency) ---
    fn tempdir() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        p.push(format!("nucleation_world_pack_{}_{:p}", n, &n as *const _));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
    fn cleanup(p: &std::path::Path) {
        let _ = std::fs::remove_dir_all(p);
    }
}
