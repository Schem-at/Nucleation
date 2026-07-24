//! World-level constants, derived once and then pinned.
//!
//! Anything that needs world-wide statistics belongs here and nowhere else:
//! a heuristic that needed global knowledge at segment time would silently
//! break shardability. Derive it once, pin it, commit it.
//!
//! Derivation from real world data is Plan 2. This module defines the pinned
//! artifact and its hash.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::world_segment::ids::ContentId;
use crate::world_segment::tile::VoxelTile;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct WorldProfile {
    /// Block names considered natural terrain. `BTreeSet` so iteration — and
    /// therefore the profile hash — is order-independent.
    pub substrate_palette: BTreeSet<String>,
    /// Inclusive `(min_y, max_y)` band within which natural blocks are ground.
    pub substrate_y_band: (i32, i32),
}

impl WorldProfile {
    pub fn new(substrate_palette: BTreeSet<String>, substrate_y_band: (i32, i32)) -> Self {
        WorldProfile { substrate_palette, substrate_y_band }
    }

    /// Stable hash of the pinned profile. Recorded on every build so a run can
    /// prove which constants produced it.
    pub fn profile_hash(&self) -> ContentId {
        let mut parts: Vec<Vec<u8>> = vec![b"profile.v1".to_vec()];
        for name in &self.substrate_palette {
            parts.push(name.as_bytes().to_vec());
        }
        parts.push(self.substrate_y_band.0.to_le_bytes().to_vec());
        parts.push(self.substrate_y_band.1.to_le_bytes().to_vec());
        let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
        ContentId::of(&refs)
    }
}

/// Parameters for empirical profile derivation.
#[derive(Clone, Debug)]
pub struct ProfileParams {
    /// Use every Nth sample tile (in sorted id order). 1 = all.
    pub sample_stride: usize,
    /// Minimum fraction of the footprint a Y level must fill to count as
    /// slab. Inclusive: a level whose coverage is at least
    /// `min_slab_coverage` (coverage == threshold counts as slab) qualifies.
    pub min_slab_coverage: f32,
    /// Inclusive Y range to scan for the slab.
    pub y_scan: (i32, i32),
    /// Minimum fraction of in-band block count a name must reach to be kept
    /// in the derived palette. `0.0` (default) keeps every name that appears
    /// at all within the band — byte-identical to the pre-dominance-filter
    /// behavior. On real-world data, a wide-enough band to cover ground
    /// naturally also covers the bottom of player builds, so raising this
    /// filters out rare non-substrate names (redstone, repeaters, etc.)
    /// while keeping the truly dominant ground materials.
    pub palette_min_share: f32,
}

impl Default for ProfileParams {
    fn default() -> Self {
        ProfileParams {
            sample_stride: 1,
            min_slab_coverage: 0.9,
            y_scan: (-64, 320),
            palette_min_share: 0.0,
        }
    }
}

/// Content key for a tile: `ContentId::of` folded over the tile's blocks in
/// their canonical ascending-position order (see `VoxelTile::blocks`).
///
/// Used purely to break ties between samples that share a `TileId` but differ
/// in contents, so sorting by `(TileId, content_key)` is a total order over
/// content rather than an order that depends on input position.
fn content_key(tile: &VoxelTile) -> ContentId {
    let mut parts: Vec<Vec<u8>> = Vec::new();
    for ((x, y, z), state) in tile.blocks() {
        parts.push(x.to_le_bytes().to_vec());
        parts.push(y.to_le_bytes().to_vec());
        parts.push(z.to_le_bytes().to_vec());
        parts.push(state.get_name().as_bytes().to_vec());
    }
    let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
    ContentId::of(&refs)
}

impl WorldProfile {
    /// Derive a pinnable profile from sample tiles by locating the near-solid
    /// ground slab. Pure and order-independent: samples are processed in a
    /// total order over `(TileId, content_key)` — not merely `TileId` — so two
    /// samples that happen to share a `TileId` but differ in contents are
    /// still ordered deterministically by their contents, regardless of the
    /// order they were supplied in. The result depends only on the samples'
    /// contents and `params`.
    pub fn derive(samples: &[VoxelTile], params: &ProfileParams) -> WorldProfile {
        // Sort sample references by (tile id, content key) for order-independence,
        // even when two samples share a tile id but differ in contents.
        let mut ordered: Vec<_> = samples.iter().map(|t| (t.id(), content_key(t), t)).collect();
        ordered.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        let stride = params.sample_stride.max(1);
        let chosen: Vec<&VoxelTile> =
            ordered.into_iter().step_by(stride).map(|(_, _, t)| t).collect();

        if chosen.is_empty() {
            return WorldProfile::new(BTreeSet::new(), (0, 0));
        }

        // Per-Y distinct occupied columns, and per-Y block-name counts (used
        // both for palette membership and, via `palette_min_share`, for
        // filtering out names that are rare within the band even though they
        // appear at all — e.g. a couple of stray redstone_wire blocks sitting
        // inside an otherwise-natural ground band).
        let mut cols_at_y: BTreeMap<i32, BTreeSet<(i32, i32)>> = BTreeMap::new();
        let mut counts_at_y: BTreeMap<i32, BTreeMap<String, u64>> = BTreeMap::new();
        // Footprint = distinct (x,z) columns seen anywhere in the samples.
        let mut footprint: BTreeSet<(i32, i32)> = BTreeSet::new();

        for tile in &chosen {
            for ((x, y, z), state) in tile.blocks() {
                if y < params.y_scan.0 || y > params.y_scan.1 {
                    continue;
                }
                footprint.insert((x, z));
                cols_at_y.entry(y).or_default().insert((x, z));
                *counts_at_y
                    .entry(y)
                    .or_default()
                    .entry(state.get_name().to_string())
                    .or_insert(0) += 1;
            }
        }

        let footprint_size = footprint.len().max(1) as f32;
        let threshold = params.min_slab_coverage;

        // Band: contiguous run of slab-dense Y levels from the lowest scanned Y.
        let mut band_lo: Option<i32> = None;
        let mut band_hi: Option<i32> = None;
        for (&y, cols) in &cols_at_y {
            let coverage = cols.len() as f32 / footprint_size;
            if coverage >= threshold {
                if band_lo.is_none() {
                    band_lo = Some(y);
                }
                // Only extend the band while it stays contiguous with the last.
                match band_hi {
                    Some(prev) if y == prev + 1 => band_hi = Some(y),
                    Some(_) => break, // gap: slab ended
                    None => band_hi = Some(y),
                }
            } else if band_lo.is_some() {
                break; // first non-slab level above the slab ends the band
            }
        }

        let (lo, hi) = match (band_lo, band_hi) {
            (Some(l), Some(h)) => (l, h),
            _ => return WorldProfile::new(BTreeSet::new(), (0, 0)),
        };

        // Palette = block names appearing within the band, filtered by
        // dominance: a name qualifies only if its share of the band's total
        // block count is at least `palette_min_share`. With the default
        // 0.0, every count is `>= 0.0`, so this is byte-identical to "every
        // name that appears at all within the band".
        let mut counts: BTreeMap<String, u64> = BTreeMap::new();
        let mut total_band_blocks: u64 = 0;
        for y in lo..=hi {
            if let Some(names) = counts_at_y.get(&y) {
                for (name, count) in names {
                    *counts.entry(name.clone()).or_insert(0) += count;
                    total_band_blocks += count;
                }
            }
        }

        let min_share = params.palette_min_share as f64;
        let palette: BTreeSet<String> = counts
            .into_iter()
            .filter(|(_, count)| {
                total_band_blocks == 0
                    || (*count as f64 / total_band_blocks as f64) >= min_share
            })
            .map(|(name, _)| name)
            .collect();

        WorldProfile::new(palette, (lo, hi))
    }
}

#[cfg(test)]
mod derive_tests {
    use super::*;
    use crate::world_segment::ids::TileId;
    use crate::world_segment::tile::{TileBounds, VoxelTile};
    use crate::BlockState;

    fn flat_world_tile() -> VoxelTile {
        // A 16x16 footprint: solid stone slab at y=-64..-61, then a small build.
        let mut blocks = vec![];
        for x in 0..16 {
            for z in 0..16 {
                for y in -64..=-61 {
                    blocks.push(((x, y, z), BlockState::new("minecraft:stone")));
                }
            }
        }
        // a build well above the slab
        blocks.push(((3, 0, 3), BlockState::new("minecraft:redstone_wire")));
        blocks.push(((4, 0, 3), BlockState::new("minecraft:repeater")));
        VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            TileBounds { min: (0, -64, 0), max: (15, 63, 15) },
            blocks.into_iter(),
        )
    }

    #[test]
    fn derives_the_slab_band_and_palette() {
        let params = ProfileParams {
            sample_stride: 1,
            min_slab_coverage: 0.9,
            y_scan: (-64, 63),
            ..Default::default()
        };
        let profile = WorldProfile::derive(&[flat_world_tile()], &params);
        // Band starts at the bottom and covers the four solid layers.
        assert_eq!(profile.substrate_y_band, (-64, -61));
        // Palette is exactly the ground material, not the build blocks.
        assert!(profile.substrate_palette.contains("minecraft:stone"));
        assert!(!profile.substrate_palette.contains("minecraft:redstone_wire"));
        assert!(!profile.substrate_palette.contains("minecraft:repeater"));
    }

    /// A small slab tile at `id` whose blocks are all `material`, on a 4x4
    /// footprint. Used to build samples that share a `TileId` but differ in
    /// contents.
    fn slab_tile(id: TileId, material: &str) -> VoxelTile {
        let mut blocks = vec![];
        for x in 0..4 {
            for z in 0..4 {
                for y in -64..=-61 {
                    blocks.push(((x, y, z), BlockState::new(material)));
                }
            }
        }
        VoxelTile::from_blocks(
            id,
            TileBounds { min: (0, -64, 0), max: (3, 63, 3) },
            blocks.into_iter(),
        )
    }

    #[test]
    fn derivation_is_independent_of_sample_order() {
        // Two samples share TileId (0,0) but differ in contents (stone vs
        // dirt); a third sample has a different TileId. `sample_stride = 2`
        // makes sub-sampling active, so which of the two same-id samples
        // survives depends entirely on how ties are broken during sort: a
        // stable sort keyed only on TileId would let input order decide,
        // which of them is picked and therefore change the derived palette.
        let params = ProfileParams {
            sample_stride: 2,
            min_slab_coverage: 0.9,
            y_scan: (-64, 63),
            ..Default::default()
        };

        let forward = vec![
            slab_tile(TileId { x: 0, z: 0 }, "minecraft:stone"),
            slab_tile(TileId { x: 0, z: 0 }, "minecraft:dirt"),
            slab_tile(TileId { x: 1, z: 0 }, "minecraft:stone"),
        ];
        let backward = vec![
            slab_tile(TileId { x: 1, z: 0 }, "minecraft:stone"),
            slab_tile(TileId { x: 0, z: 0 }, "minecraft:dirt"),
            slab_tile(TileId { x: 0, z: 0 }, "minecraft:stone"),
        ];

        let p1 = WorldProfile::derive(&forward, &params);
        let p2 = WorldProfile::derive(&backward, &params);
        assert_eq!(
            p1.profile_hash(),
            p2.profile_hash(),
            "profile must not depend on input order, even when two samples share a TileId"
        );
    }

    #[test]
    fn coverage_exactly_at_threshold_counts_as_slab() {
        // 10 distinct footprint columns total; only 9 of them are present at
        // y = -64, so coverage there is exactly 9 / 10 = 0.9, equal to (not
        // greater than) `min_slab_coverage`. The 10th footprint column is
        // seeded far below the band so it pads the footprint without itself
        // qualifying as slab.
        let params = ProfileParams {
            sample_stride: 1,
            min_slab_coverage: 0.9,
            y_scan: (-70, -60),
            ..Default::default()
        };
        let mut blocks = vec![((9, -70, 0), BlockState::new("minecraft:bedrock"))];
        for x in 0..9 {
            blocks.push(((x, -64, 0), BlockState::new("minecraft:stone")));
        }
        let tile = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            TileBounds { min: (0, -70, 0), max: (9, 63, 0) },
            blocks.into_iter(),
        );

        let profile = WorldProfile::derive(&[tile], &params);
        assert_eq!(
            profile.substrate_y_band,
            (-64, -64),
            "a level whose coverage exactly equals min_slab_coverage must count as slab"
        );
        assert!(profile.substrate_palette.contains("minecraft:stone"));
    }

    #[test]
    fn empty_samples_yield_an_empty_profile() {
        let profile = WorldProfile::derive(&[], &ProfileParams::default());
        assert!(profile.substrate_palette.is_empty());
    }

    #[test]
    fn palette_dominance_filter_excludes_rare_names() {
        // 10x10 stone slab across y=-64..-61 (400 blocks). Plus 2
        // redstone_wire blocks at y=-63, on footprint columns that don't
        // overlap the stone columns (so they don't overwrite slab blocks),
        // but the columns are only occupied at that single Y — they don't
        // themselves reach slab coverage at other levels, so the band is
        // unaffected: it stays -64..-61, exactly mirroring the real-data
        // failure (player redstone sitting inside an otherwise-natural band).
        fn build_tile() -> VoxelTile {
            let mut blocks = vec![];
            for x in 0..10 {
                for z in 0..10 {
                    for y in -64..=-61 {
                        blocks.push(((x, y, z), BlockState::new("minecraft:stone")));
                    }
                }
            }
            blocks.push(((20, -63, 20), BlockState::new("minecraft:redstone_wire")));
            blocks.push(((21, -63, 20), BlockState::new("minecraft:redstone_wire")));
            VoxelTile::from_blocks(
                TileId { x: 0, z: 0 },
                TileBounds { min: (0, -64, 0), max: (21, 63, 21) },
                blocks.into_iter(),
            )
        }

        // total in-band blocks = 400 stone + 2 redstone = 402;
        // redstone share = 2/402 ~= 0.005 < 0.01 -> excluded.
        let filtered_params = ProfileParams {
            sample_stride: 1,
            min_slab_coverage: 0.9,
            y_scan: (-64, -61),
            palette_min_share: 0.01,
        };
        let filtered = WorldProfile::derive(&[build_tile()], &filtered_params);
        assert_eq!(filtered.substrate_y_band, (-64, -61));
        assert!(filtered.substrate_palette.contains("minecraft:stone"));
        assert!(
            !filtered.substrate_palette.contains("minecraft:redstone_wire"),
            "rare name below palette_min_share must be filtered out"
        );

        // Default (0.0) must preserve today's behavior: every present name
        // qualifies.
        let default_params = ProfileParams {
            sample_stride: 1,
            min_slab_coverage: 0.9,
            y_scan: (-64, -61),
            ..Default::default()
        };
        let unfiltered = WorldProfile::derive(&[build_tile()], &default_params);
        assert_eq!(unfiltered.substrate_y_band, (-64, -61));
        assert!(unfiltered.substrate_palette.contains("minecraft:stone"));
        assert!(unfiltered.substrate_palette.contains("minecraft:redstone_wire"));
    }
}
