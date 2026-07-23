//! Random-access tile source over a `WorldSource` (directory / zip / mca).

use std::collections::BTreeMap;

use crate::formats::world_stream::WorldSource;
use crate::world_segment::ids::TileId;
use crate::world_segment::source::{region_tile_bounds, Access, TileError, TileSource};
use crate::world_segment::tile::VoxelTile;

/// Region containing a chunk. Floor division: chunk -1 is in region -1.
pub fn chunk_region(cx: i32, cz: i32) -> (i32, i32) {
    (cx.div_euclid(32), cz.div_euclid(32))
}

pub struct WorldSourceTiles {
    source: WorldSource,
    min_y: i32,
    max_y: i32,
}

impl WorldSourceTiles {
    pub fn new(source: WorldSource, min_y: i32, max_y: i32) -> Self {
        WorldSourceTiles { source, min_y, max_y }
    }

    fn collect_tile(&self, region_x: i32, region_z: i32) -> Result<Option<VoxelTile>, TileError> {
        let (tile_id, bounds) = region_tile_bounds(region_x, region_z, self.min_y, self.max_y);
        // Bounded chunk iteration over exactly this region's block span.
        let iter = self
            .source
            .chunks_bounded(bounds.min, bounds.max)
            .map_err(|e| TileError::Io(e.to_string()))?;
        // Gather blocks deterministically: BTreeMap keyed by position.
        let mut blocks: BTreeMap<(i32, i32, i32), crate::BlockState> = BTreeMap::new();
        for view in iter {
            // ChunkIter yields Result<WorldChunkView>: a corrupt chunk is one
            // error item, then iteration continues. Propagate as a TileError.
            let view = view.map_err(|e| TileError::Malformed(e.to_string()))?;
            for (x, y, z, state) in view.blocks() {
                if y < self.min_y || y > self.max_y {
                    continue;
                }
                if chunk_region(view.cx(), view.cz()) != (region_x, region_z) {
                    continue;
                }
                blocks.insert((x, y, z), state.clone());
            }
        }
        if blocks.is_empty() {
            return Ok(None);
        }
        Ok(Some(VoxelTile::from_blocks(
            tile_id,
            bounds,
            blocks.into_iter(),
        )))
    }
}

impl TileSource for WorldSourceTiles {
    fn access(&self) -> Access {
        Access::Random
    }

    fn tile_ids(&self) -> Result<Vec<TileId>, TileError> {
        let mut ids: Vec<TileId> = self
            .source
            .region_positions()
            .map_err(|e| TileError::Io(e.to_string()))?
            .into_iter()
            .map(|(x, z)| TileId { x, z })
            .collect();
        ids.sort();
        ids.dedup();
        Ok(ids)
    }

    fn tile(&self, id: TileId) -> Result<Option<VoxelTile>, TileError> {
        self.collect_tile(id.x, id.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_maps_to_its_region_tile() {
        // Chunk (0,0)..(31,31) -> region (0,0). Chunk (32,0) -> region (1,0).
        // Chunk (-1,0) -> region (-1,0) (floor division).
        assert_eq!(chunk_region(0, 0), (0, 0));
        assert_eq!(chunk_region(31, 31), (0, 0));
        assert_eq!(chunk_region(32, 0), (1, 0));
        assert_eq!(chunk_region(-1, 0), (-1, 0));
        assert_eq!(chunk_region(-32, 0), (-1, 0));
        assert_eq!(chunk_region(-33, 0), (-2, 0));
    }
}
