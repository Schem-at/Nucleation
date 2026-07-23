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
    /// Build a tile from an arbitrary, arbitrarily-ordered block stream.
    ///
    /// # Duplicate positions
    ///
    /// The same position may legitimately arrive more than once — region and
    /// chunk readers overlap, and so do tile margins. The winner is
    /// **content-defined, never order-defined**: of the states offered for a
    /// position, the one with the lexicographically smallest canonical
    /// [`palette_key`] is kept, and the rest are discarded.
    ///
    /// A last-write-wins `insert` would make the surviving block a function of
    /// arrival order, and since classification can differ between the
    /// candidates (one substrate, one artificial) that would change the cluster
    /// set — breaking the module's central guarantee that identical input plus
    /// identical config yields byte-identical output.
    ///
    /// The tie-break is deliberately arbitrary but total: any rule works so
    /// long as it depends only on content, and `palette_key` is already the
    /// canonical string used for palette dedup, so no new notion of block
    /// identity is introduced.
    ///
    /// Blocks outside `bounds` are dropped.
    pub fn from_blocks(
        id: TileId,
        bounds: TileBounds,
        blocks: impl Iterator<Item = ((i32, i32, i32), BlockState)>,
    ) -> Self {
        let mut seen: Vec<BlockState> = Vec::new();
        // Canonical key of `seen[i]`, kept alongside so a duplicate can be
        // resolved by key without re-deriving it. One clone per distinct
        // state, not one per block.
        let mut seen_keys: Vec<String> = Vec::new();
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
                    let i = seen.len() as u32;
                    seen.push(state);
                    seen_keys.push(key.clone());
                    lookup.insert(key, i);
                    i
                }
            };
            match cells.entry(pos) {
                std::collections::btree_map::Entry::Vacant(e) => {
                    e.insert(idx);
                }
                std::collections::btree_map::Entry::Occupied(mut e) => {
                    // Smaller canonical key wins. Equal keys mean equal states,
                    // so the comparison never has to break a genuine tie.
                    if seen_keys[idx as usize] < seen_keys[*e.get() as usize] {
                        e.insert(idx);
                    }
                }
            }
        }

        // Compact: keep only states some surviving cell references, ordered by
        // canonical key. A state that lost every duplicate contest it entered
        // must not linger in the palette, or `palette_len` — and any future
        // consumer of palette order — would still depend on arrival order.
        let mut used: Vec<u32> = cells.values().copied().collect();
        used.sort_unstable();
        used.dedup();
        used.sort_by(|a, b| seen_keys[*a as usize].cmp(&seen_keys[*b as usize]));

        let mut remap: Vec<u32> = vec![u32::MAX; seen.len()];
        let mut palette: Vec<BlockState> = Vec::with_capacity(used.len());
        for (new_idx, old_idx) in used.iter().enumerate() {
            remap[*old_idx as usize] = new_idx as u32;
            palette.push(seen[*old_idx as usize].clone());
        }

        let cells: Vec<((i32, i32, i32), u32)> =
            cells.into_iter().map(|(pos, old)| (pos, remap[old as usize])).collect();

        VoxelTile { id, bounds, palette, cells }
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

    /// Names and their canonical `palette_key`s, so the expected winner below
    /// is arithmetic rather than intuition:
    ///   "minecraft:redstone_wire[]"
    ///   "minecraft:stone[]"
    /// `'r'` (0x72) < `'s'` (0x73) at the first differing byte, so
    /// `redstone_wire` is the lexicographically smaller key and must win
    /// whichever order the two are supplied in.
    #[test]
    fn duplicate_positions_resolve_to_the_smaller_palette_key_either_way() {
        let forward = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            bounds(),
            vec![((1, 1, 1), bs("minecraft:stone")), ((1, 1, 1), bs("minecraft:redstone_wire"))]
                .into_iter(),
        );
        let reverse = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            bounds(),
            vec![((1, 1, 1), bs("minecraft:redstone_wire")), ((1, 1, 1), bs("minecraft:stone"))]
                .into_iter(),
        );

        for (label, tile) in [("forward", &forward), ("reverse", &reverse)] {
            assert_eq!(tile.len(), 1, "{label}: one position, one cell");
            let got: Vec<_> = tile.blocks().map(|(p, b)| (p, b.get_name().to_string())).collect();
            assert_eq!(
                got,
                vec![((1, 1, 1), "minecraft:redstone_wire".to_string())],
                "{label}: the lexicographically smaller palette key must win"
            );
        }
    }

    #[test]
    fn the_palette_is_content_defined_not_insertion_ordered() {
        // A losing duplicate must not leave a phantom palette entry behind,
        // otherwise `palette_len` depends on input order even though `blocks`
        // does not.
        let forward = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            bounds(),
            vec![((1, 1, 1), bs("minecraft:stone")), ((1, 1, 1), bs("minecraft:redstone_wire"))]
                .into_iter(),
        );
        let reverse = VoxelTile::from_blocks(
            TileId { x: 0, z: 0 },
            bounds(),
            vec![((1, 1, 1), bs("minecraft:redstone_wire")), ((1, 1, 1), bs("minecraft:stone"))]
                .into_iter(),
        );
        assert_eq!(forward.palette_len(), 1, "the losing state is not retained");
        assert_eq!(forward.palette_len(), reverse.palette_len());
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
