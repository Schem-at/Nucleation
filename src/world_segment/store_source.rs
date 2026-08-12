//! Random-access region tiles read from a pluggable [`Store`](crate::store::Store).
//!
//! A storage node can expose an extracted Anvil dimension while a separate
//! machine performs compute. Only the `.mca` files intersecting the requested
//! rectangle cross the network, and at most one region is buffered at a time.

use crate::formats::world_stream::WorldSource;
use crate::store::Store;
use crate::world_segment::ids::TileId;
use crate::world_segment::source::{Access, TileError, TileSource};
use crate::world_segment::targz_source::{parse_region_coords, WorldRect};
use crate::world_segment::tile::VoxelTile;
use crate::world_segment::world_source::WorldSourceTiles;

/// An Anvil `region/` directory whose files live in a [`Store`].
pub struct StoreRegionTiles {
    store: Box<dyn Store>,
    region_prefix: String,
    min_y: i32,
    max_y: i32,
    world_rect: Option<WorldRect>,
}

impl StoreRegionTiles {
    /// `region_prefix` is the store key of the dimension's `region` directory,
    /// without a trailing slash (for example `world/region`).
    pub fn new(
        store: Box<dyn Store>,
        region_prefix: impl Into<String>,
        min_y: i32,
        max_y: i32,
    ) -> Result<Self, TileError> {
        let region_prefix = region_prefix.into().trim_matches('/').to_string();
        if region_prefix.is_empty() || region_prefix.split('/').any(|part| part == "..") {
            return Err(TileError::Io("invalid region store prefix".into()));
        }
        Ok(Self {
            store,
            region_prefix,
            min_y,
            max_y,
            world_rect: None,
        })
    }

    /// Restrict network reads and decoded blocks to an inclusive XZ rectangle.
    pub fn with_world_rect(mut self, min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> Self {
        self.world_rect = Some(WorldRect::new(min_x, min_z, max_x, max_z));
        self
    }

    fn key(&self, id: TileId) -> String {
        format!("{}/r.{}.{}.mca", self.region_prefix, id.x, id.z)
    }
}

impl TileSource for StoreRegionTiles {
    fn access(&self) -> Access {
        Access::Random
    }

    fn tile_ids(&self) -> Result<Vec<TileId>, TileError> {
        let mut ids = self
            .store
            .list(&format!("{}/", self.region_prefix))
            .map_err(|error| TileError::Io(error.to_string()))?
            .into_iter()
            .filter_map(|key| parse_region_coords(&key))
            .filter(|(x, z)| {
                self.world_rect
                    .map_or(true, |rect| rect.intersects_region(*x, *z))
            })
            .map(|(x, z)| TileId { x, z })
            .collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        Ok(ids)
    }

    fn tile(&self, id: TileId) -> Result<Option<VoxelTile>, TileError> {
        if self
            .world_rect
            .is_some_and(|rect| !rect.intersects_region(id.x, id.z))
        {
            return Ok(None);
        }
        let key = self.key(id);
        let Some(bytes) = self
            .store
            .get(&key)
            .map_err(|error| TileError::Io(error.to_string()))?
        else {
            return Ok(None);
        };
        let source = WorldSource::from_mca_bytes(bytes)
            .map_err(|error| TileError::Malformed(error.to_string()))?;
        let mut tiles = WorldSourceTiles::new(source, self.min_y, self.max_y);
        if let Some(rect) = self.world_rect {
            tiles = tiles.with_world_rect(rect.min_x, rect.min_z, rect.max_x, rect.max_z);
        }
        tiles.tile(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{MemStore, Store};

    #[test]
    fn listing_is_sorted_deduped_and_rectangle_filtered() {
        let store = MemStore::new();
        store.put("map/region/r.0.0.mca", b"x").unwrap();
        store.put("map/region/r.2.-1.mca", b"x").unwrap();
        store.put("map/entities/r.0.0.mca", b"x").unwrap();
        let source = StoreRegionTiles::new(Box::new(store), "map/region", -64, 320)
            .unwrap()
            .with_world_rect(0, 0, 511, 511);
        assert_eq!(source.tile_ids().unwrap(), vec![TileId { x: 0, z: 0 }]);
    }
}
