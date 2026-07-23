//! Sources of voxel tiles.
//!
//! A tile is an axis-aligned voxel block a source can yield independently. This
//! is the boundary between the pure segmentation core and the outside world:
//! all I/O lives in `TileSource` implementations, and the core downstream of a
//! yielded `VoxelTile` remains order-independent and side-effect-free.

use crate::world_segment::ids::TileId;
use crate::world_segment::tile::{TileBounds, VoxelTile};

/// Whether a source supports random tile access or only a single forward pass.
///
/// A `.tar.gz` cannot be seeked, so it is `Forward`: `tile()` is unavailable and
/// callers must use `for_each_tile`. A world directory is `Random`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Access {
    Random,
    Forward,
}

#[derive(thiserror::Error, Debug)]
pub enum TileError {
    #[error("io: {0}")]
    Io(String),
    #[error("source does not support random access; use for_each_tile")]
    NotRandomAccess,
    #[error("malformed source: {0}")]
    Malformed(String),
}

/// A Minecraft region spans 512 blocks per axis (32 chunks x 16 blocks).
pub const REGION_BLOCKS: i32 = 512;

/// World-space tile id and bounds for a region at region-coordinates
/// `(region_x, region_z)`, clamped vertically to `[min_y, max_y]` inclusive.
pub fn region_tile_bounds(
    region_x: i32,
    region_z: i32,
    min_y: i32,
    max_y: i32,
) -> (TileId, TileBounds) {
    let x0 = region_x * REGION_BLOCKS;
    let z0 = region_z * REGION_BLOCKS;
    (
        TileId { x: region_x, z: region_z },
        TileBounds {
            min: (x0, min_y, z0),
            max: (x0 + REGION_BLOCKS - 1, max_y, z0 + REGION_BLOCKS - 1),
        },
    )
}

/// A source of voxel tiles.
///
/// Every source implements `for_each_tile` (the streaming path that works for
/// both access kinds). Random-access sources additionally support `tile_ids` +
/// `tile` for pull scheduling; forward sources return `NotRandomAccess` from
/// `tile`.
pub trait TileSource {
    fn access(&self) -> Access;

    /// Tile ids available, ascending and deduplicated. Forward sources may
    /// return an error or an empty list if ids are not known before streaming.
    fn tile_ids(&self) -> Result<Vec<TileId>, TileError>;

    /// A single tile by id. `Ok(None)` means "no data for this id"; random
    /// sources only. Forward sources return `NotRandomAccess`.
    fn tile(&self, id: TileId) -> Result<Option<VoxelTile>, TileError>;

    /// Stream every non-empty tile through `f`, in the source's natural order.
    /// This is the entry point that works for both `Random` and `Forward`.
    fn for_each_tile(
        &self,
        f: &mut dyn FnMut(VoxelTile) -> Result<(), TileError>,
    ) -> Result<(), TileError> {
        for id in self.tile_ids()? {
            if let Some(t) = self.tile(id)? {
                f(t)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_tile_bounds_maps_region_to_512_block_span() {
        // Region (0,0) covers world blocks x,z in [0,511]; region (-1,0) covers [-512,-1].
        let (id, b) = region_tile_bounds(0, 0, -64, 319);
        assert_eq!(id, crate::world_segment::ids::TileId { x: 0, z: 0 });
        assert_eq!(b.min, (0, -64, 0));
        assert_eq!(b.max, (511, 319, 511));

        let (id2, b2) = region_tile_bounds(-1, 0, -64, 319);
        assert_eq!(id2, crate::world_segment::ids::TileId { x: -1, z: 0 });
        assert_eq!(b2.min, (-512, -64, 0));
        assert_eq!(b2.max, (-1, 319, 511));
    }

    #[test]
    fn access_is_copy_and_comparable() {
        assert_eq!(Access::Random, Access::Random);
        assert_ne!(Access::Random, Access::Forward);
        let a = Access::Forward;
        let _b = a; // Copy
        assert_eq!(a, Access::Forward);
    }
}
