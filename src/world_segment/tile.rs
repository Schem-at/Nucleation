//! A tile: an axis-aligned voxel block that a source can yield independently.
//!
//! Blocks are stored as a deduplicated palette plus sorted `(position, index)`
//! cells. Sorting at construction is what makes downstream iteration
//! order-independent without every consumer having to re-sort.

use std::collections::BTreeMap;

use crate::block_state::BlockState;
use crate::world_segment::ids::TileId;

/// Inclusive world-space bounds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TileBounds {
    pub min: (i32, i32, i32),
    pub max: (i32, i32, i32),
}

impl TileBounds {
    pub fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        x >= self.min.0 && x <= self.max.0
            && y >= self.min.1 && y <= self.max.1
            && z >= self.min.2 && z <= self.max.2
    }
}

/// One tile's non-air blocks.
///
/// Memory note: this holds every non-air block in the tile, including
/// substrate. Substrate is dropped by `segment_tile`, not here, so that
/// classification stays a pure decision a test can drive directly.
pub struct VoxelTile {
    id: TileId,
    bounds: TileBounds,
    palette: Vec<BlockState>,
    /// Sorted by position. `(x, y, z, palette_index)`.
    cells: Vec<((i32, i32, i32), u32)>,
}

impl VoxelTile {
    pub fn from_blocks(
        id: TileId,
        bounds: TileBounds,
        blocks: impl Iterator<Item = ((i32, i32, i32), BlockState)>,
    ) -> Self {
        let mut palette: Vec<BlockState> = Vec::new();
        let mut lookup: BTreeMap<String, u32> = BTreeMap::new();
        // BTreeMap keyed by position: dedupes repeated positions and yields
        // sorted order for free.
        let mut cells: BTreeMap<(i32, i32, i32), u32> = BTreeMap::new();

        for (pos, state) in blocks {
            if !bounds.contains(pos.0, pos.1, pos.2) {
                continue;
            }
            let key = palette_key(&state);
            let idx = match lookup.get(&key) {
                Some(i) => *i,
                None => {
                    let i = palette.len() as u32;
                    palette.push(state);
                    lookup.insert(key, i);
                    i
                }
            };
            cells.insert(pos, idx);
        }

        VoxelTile { id, bounds, palette, cells: cells.into_iter().collect() }
    }

    pub fn id(&self) -> TileId {
        self.id
    }

    pub fn bounds(&self) -> &TileBounds {
        &self.bounds
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn palette_len(&self) -> usize {
        self.palette.len()
    }

    /// Blocks in ascending position order.
    pub fn blocks(&self) -> impl Iterator<Item = ((i32, i32, i32), &BlockState)> + '_ {
        self.cells.iter().map(move |(pos, idx)| (*pos, &self.palette[*idx as usize]))
    }
}

/// Canonical string for palette dedup: name plus sorted properties.
fn palette_key(state: &BlockState) -> String {
    let mut props: Vec<String> =
        state.properties.iter().map(|(k, v)| format!("{k}={v}")).collect();
    props.sort();
    format!("{}[{}]", state.get_name(), props.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_segment::ids::TileId;

    fn bs(name: &str) -> BlockState {
        BlockState::new(name)
    }

    fn bounds() -> TileBounds {
        TileBounds { min: (0, 0, 0), max: (15, 15, 15) }
    }

    #[test]
    fn bounds_contains_is_inclusive() {
        let b = bounds();
        assert!(b.contains(0, 0, 0));
        assert!(b.contains(15, 15, 15));
        assert!(!b.contains(16, 0, 0));
        assert!(!b.contains(-1, 0, 0));
    }

    #[test]
    fn tile_stores_blocks_and_dedupes_palette() {
        let tile = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            bounds(),
            vec![
                ((1, 2, 3), bs("minecraft:stone")),
                ((4, 5, 6), bs("minecraft:stone")),
                ((7, 8, 9), bs("minecraft:redstone_wire")),
            ]
            .into_iter(),
        );
        assert_eq!(tile.len(), 3);
        assert_eq!(tile.palette_len(), 2, "identical states share a palette entry");
        assert_eq!(tile.id(), TileId { x: 0, z: 0 });
    }

    #[test]
    fn tile_drops_out_of_bounds_blocks() {
        let tile = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            bounds(),
            vec![((1, 1, 1), bs("minecraft:stone")), ((99, 1, 1), bs("minecraft:stone"))]
                .into_iter(),
        );
        assert_eq!(tile.len(), 1, "out-of-bounds blocks are rejected");
    }

    #[test]
    fn blocks_iterate_in_sorted_order_regardless_of_insertion_order() {
        let forward = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            bounds(),
            vec![
                ((1, 1, 1), bs("minecraft:stone")),
                ((2, 2, 2), bs("minecraft:dirt")),
                ((3, 3, 3), bs("minecraft:redstone_wire")),
            ]
            .into_iter(),
        );
        let reverse = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            bounds(),
            vec![
                ((3, 3, 3), bs("minecraft:redstone_wire")),
                ((2, 2, 2), bs("minecraft:dirt")),
                ((1, 1, 1), bs("minecraft:stone")),
            ]
            .into_iter(),
        );
        let f: Vec<_> = forward.blocks().map(|(p, b)| (p, b.get_name().to_string())).collect();
        let r: Vec<_> = reverse.blocks().map(|(p, b)| (p, b.get_name().to_string())).collect();
        assert_eq!(f, r, "iteration order must not depend on insertion order");
    }
}
