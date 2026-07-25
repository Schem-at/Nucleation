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
    /// Not an error: a callback returns this from `for_each_tile` to request
    /// early termination. Implementations MUST stop iterating and return
    /// `Ok(())` — never propagate `Stop` to the caller.
    #[error("iteration stopped by callback")]
    Stop,
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
    ///
    /// Early termination: if `f` returns `Err(TileError::Stop)`, this method
    /// stops iterating and returns `Ok(())` — `Stop` is a sentinel, not a
    /// real error, and must never be propagated to the caller. Any other
    /// `Err` returned by `f` propagates as before.
    fn for_each_tile(
        &self,
        f: &mut dyn FnMut(VoxelTile) -> Result<(), TileError>,
    ) -> Result<(), TileError> {
        for id in self.tile_ids()? {
            if let Some(t) = self.tile(id)? {
                match f(t) {
                    Ok(()) => {}
                    Err(TileError::Stop) => return Ok(()),
                    Err(e) => return Err(e),
                }
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

    /// A random-access source with 3 tiles that does NOT override
    /// `for_each_tile`, so the default trait impl is exercised directly.
    struct ThreeTileSource;

    impl TileSource for ThreeTileSource {
        fn access(&self) -> Access {
            Access::Random
        }

        fn tile_ids(&self) -> Result<Vec<TileId>, TileError> {
            Ok(vec![
                TileId { x: 0, z: 0 },
                TileId { x: 1, z: 0 },
                TileId { x: 2, z: 0 },
            ])
        }

        fn tile(&self, id: TileId) -> Result<Option<VoxelTile>, TileError> {
            Ok(Some(VoxelTile::from_blocks(
                id,
                TileBounds { min: (0, 0, 0), max: (15, 15, 15) },
                std::iter::once((
                    (0, 0, 0),
                    crate::block_state::BlockState::new("minecraft:stone"),
                )),
            )))
        }
    }

    /// Callback returns `Stop` after the first tile of three. The default
    /// impl must stop iterating right there and report `Ok(())`, not
    /// propagate `Stop` as an error. If `Stop` were (mis)propagated as a
    /// plain `Err`, `result.is_ok()` below would be false and this test
    /// would fail — so this is not a vacuous assertion.
    #[test]
    fn default_for_each_tile_stops_on_stop_sentinel_and_reports_ok() {
        let source = ThreeTileSource;
        let mut call_count = 0usize;
        let result = source.for_each_tile(&mut |_tile| {
            call_count += 1;
            Err(TileError::Stop)
        });

        assert!(
            result.is_ok(),
            "Stop must not propagate as an error: got {result:?}"
        );
        assert_eq!(call_count, 1, "iteration must stop right after the Stop-returning call");
    }
}
