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
    /// Minimum fraction of the footprint a Y level must fill to count as slab.
    pub min_slab_coverage: f32,
    /// Inclusive Y range to scan for the slab.
    pub y_scan: (i32, i32),
}

impl Default for ProfileParams {
    fn default() -> Self {
        ProfileParams { sample_stride: 1, min_slab_coverage: 0.9, y_scan: (-64, 320) }
    }
}

impl WorldProfile {
    /// Derive a pinnable profile from sample tiles by locating the near-solid
    /// ground slab. Pure and order-independent: samples are processed in sorted
    /// id order and the result depends only on their contents and `params`.
    pub fn derive(samples: &[VoxelTile], params: &ProfileParams) -> WorldProfile {
        // Sort sample references by tile id for order-independence.
        let mut ordered: Vec<&VoxelTile> = samples.iter().collect();
        ordered.sort_by_key(|t| t.id());
        let stride = params.sample_stride.max(1);
        let chosen: Vec<&VoxelTile> = ordered.into_iter().step_by(stride).collect();

        if chosen.is_empty() {
            return WorldProfile::new(BTreeSet::new(), (0, 0));
        }

        // Per-Y distinct occupied columns, and per-Y block-name set.
        let mut cols_at_y: BTreeMap<i32, BTreeSet<(i32, i32)>> = BTreeMap::new();
        let mut names_at_y: BTreeMap<i32, BTreeSet<String>> = BTreeMap::new();
        // Footprint = distinct (x,z) columns seen anywhere in the samples.
        let mut footprint: BTreeSet<(i32, i32)> = BTreeSet::new();

        for tile in &chosen {
            for ((x, y, z), state) in tile.blocks() {
                if y < params.y_scan.0 || y > params.y_scan.1 {
                    continue;
                }
                footprint.insert((x, z));
                cols_at_y.entry(y).or_default().insert((x, z));
                names_at_y.entry(y).or_default().insert(state.get_name().to_string());
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

        // Palette = block names appearing within the band.
        let mut palette: BTreeSet<String> = BTreeSet::new();
        for y in lo..=hi {
            if let Some(names) = names_at_y.get(&y) {
                palette.extend(names.iter().cloned());
            }
        }

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
        let params = ProfileParams { sample_stride: 1, min_slab_coverage: 0.9, y_scan: (-64, 63) };
        let profile = WorldProfile::derive(&[flat_world_tile()], &params);
        // Band starts at the bottom and covers the four solid layers.
        assert_eq!(profile.substrate_y_band, (-64, -61));
        // Palette is exactly the ground material, not the build blocks.
        assert!(profile.substrate_palette.contains("minecraft:stone"));
        assert!(!profile.substrate_palette.contains("minecraft:redstone_wire"));
        assert!(!profile.substrate_palette.contains("minecraft:repeater"));
    }

    #[test]
    fn derivation_is_independent_of_sample_order() {
        let params = ProfileParams::default();
        let a = flat_world_tile();
        let b = flat_world_tile();
        // Two identical samples in either order must give the same profile hash.
        let p1 = WorldProfile::derive(&[a], &params);
        let p2 = WorldProfile::derive(&[b], &params);
        assert_eq!(p1.profile_hash(), p2.profile_hash());
    }

    #[test]
    fn empty_samples_yield_an_empty_profile() {
        let profile = WorldProfile::derive(&[], &ProfileParams::default());
        assert!(profile.substrate_palette.is_empty());
    }
}
