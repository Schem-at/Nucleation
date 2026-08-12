//! Random-access tile source over a `WorldSource` (directory / zip / mca).

use std::collections::BTreeMap;

use crate::block_entity::BlockEntity;
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
    world_rect: Option<(i32, i32, i32, i32)>,
}

impl WorldSourceTiles {
    pub fn new(source: WorldSource, min_y: i32, max_y: i32) -> Self {
        WorldSourceTiles {
            source,
            min_y,
            max_y,
            world_rect: None,
        }
    }

    /// Restrict block and block-entity collection to an inclusive world-space
    /// XZ rectangle while retaining the enclosing region's tile identity.
    pub fn with_world_rect(mut self, min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> Self {
        self.world_rect = Some((
            min_x.min(max_x),
            min_z.min(max_z),
            min_x.max(max_x),
            min_z.max(max_z),
        ));
        self
    }

    fn collect_tile(&self, region_x: i32, region_z: i32) -> Result<Option<VoxelTile>, TileError> {
        let (tile_id, bounds) = region_tile_bounds(region_x, region_z, self.min_y, self.max_y);
        let (query_min, query_max) = match self.world_rect {
            Some((min_x, min_z, max_x, max_z)) => {
                let min = (
                    bounds.min.0.max(min_x),
                    bounds.min.1,
                    bounds.min.2.max(min_z),
                );
                let max = (
                    bounds.max.0.min(max_x),
                    bounds.max.1,
                    bounds.max.2.min(max_z),
                );
                if min.0 > max.0 || min.2 > max.2 {
                    return Ok(None);
                }
                (min, max)
            }
            None => (bounds.min, bounds.max),
        };
        // Bounded chunk iteration over exactly this region's block span.
        let iter = self
            .source
            .chunks_bounded(query_min, query_max)
            .map_err(|e| TileError::Io(e.to_string()))?;
        // Gather blocks deterministically: BTreeMap keyed by position.
        let mut blocks: BTreeMap<(i32, i32, i32), crate::BlockState> = BTreeMap::new();
        let mut block_entities: BTreeMap<(i32, i32, i32), BlockEntity> = BTreeMap::new();
        for view in iter {
            // ChunkIter yields Result<WorldChunkView>: a corrupt chunk is one
            // error item, then iteration continues. Propagate as a TileError.
            let view = view.map_err(|e| TileError::Malformed(e.to_string()))?;
            for (x, y, z, state) in view.blocks() {
                if y < self.min_y || y > self.max_y {
                    continue;
                }
                if x < query_min.0 || x > query_max.0 || z < query_min.2 || z > query_max.2 {
                    continue;
                }
                if chunk_region(view.cx(), view.cz()) != (region_x, region_z) {
                    continue;
                }
                blocks.insert((x, y, z), state.clone());
            }
            for block_entity in view.block_entities() {
                let pos = block_entity.position;
                if pos.0 >= query_min.0
                    && pos.0 <= query_max.0
                    && pos.1 >= query_min.1
                    && pos.1 <= query_max.1
                    && pos.2 >= query_min.2
                    && pos.2 <= query_max.2
                    && chunk_region(view.cx(), view.cz()) == (region_x, region_z)
                {
                    block_entities.insert(pos, block_entity.clone());
                }
            }
        }
        if blocks.is_empty() {
            return Ok(None);
        }
        Ok(Some(VoxelTile::from_blocks_and_block_entities(
            tile_id,
            bounds,
            blocks.into_iter(),
            block_entities.into_values(),
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
        if let Some((min_x, min_z, max_x, max_z)) = self.world_rect {
            let min_region_x = min_x.div_euclid(512);
            let max_region_x = max_x.div_euclid(512);
            let min_region_z = min_z.div_euclid(512);
            let max_region_z = max_z.div_euclid(512);
            ids.retain(|id| {
                id.x >= min_region_x
                    && id.x <= max_region_x
                    && id.z >= min_region_z
                    && id.z <= max_region_z
            });
        }
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
