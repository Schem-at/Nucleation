//! Refuse artificial extraction cuts before cluster-size filtering can hide them.
//!
//! Empty margins or complete caller-owned hard partitions certify the XZ cut.
//! This is deliberately conservative: widen/re-align a rejected rectangle; do
//! not interpret its partial catalogue as evidence that a build disappeared.

use super::{Access, PartitionIndex, TileError, TileId, TileSource, VoxelTile, WorldProfile};

pub struct CoverageCheckedTiles<'a> {
    pub source: &'a dyn TileSource,
    pub profile: &'a WorldProfile,
    pub partitions: &'a PartitionIndex,
    pub rect: (i32, i32, i32, i32),
    pub margin: i32,
    pub drop_unpartitioned: bool,
}

impl CoverageCheckedTiles<'_> {
    fn check(&self, tile: &VoxelTile) -> Result<(), TileError> {
        let (x0, z0, x1, z1) = self.rect;
        for ((x, y, z), state) in tile.blocks() {
            if x > x0.saturating_add(self.margin)
                && x < x1.saturating_sub(self.margin)
                && z > z0.saturating_add(self.margin)
                && z < z1.saturating_sub(self.margin)
            {
                continue;
            }
            let name = state.get_name();
            if matches!(
                name,
                "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
            ) || (y >= self.profile.substrate_y_band.0
                && y <= self.profile.substrate_y_band.1
                && self.profile.substrate_palette.contains(name))
            {
                continue;
            }
            if let Some(index) = self.partitions.id_index_at(x, y, z) {
                let hint = self.partitions.hint_of_index(index);
                let (px0, px1, pz0, pz1) = hint.bbox_xz;
                if px0 >= x0 && px1 <= x1 && pz0 >= z0 && pz1 <= z1 {
                    continue;
                }
            } else if self.drop_unpartitioned && !self.partitions.is_empty() {
                continue;
            }
            return Err(TileError::Malformed(format!(
                "uncertified extraction boundary near ({x},{y},{z}); widen the rectangle to leave an empty margin or use complete hard partition hints"
            )));
        }
        Ok(())
    }
}

impl TileSource for CoverageCheckedTiles<'_> {
    fn access(&self) -> Access {
        self.source.access()
    }
    fn tile_ids(&self) -> Result<Vec<TileId>, TileError> {
        self.source.tile_ids()
    }
    fn tile(&self, id: TileId) -> Result<Option<VoxelTile>, TileError> {
        let tile = self.source.tile(id)?;
        if let Some(ref tile) = tile {
            self.check(tile)?;
        }
        Ok(tile)
    }
    fn for_each_tile(
        &self,
        f: &mut dyn FnMut(VoxelTile) -> Result<(), TileError>,
    ) -> Result<(), TileError> {
        self.source.for_each_tile(&mut |tile| {
            self.check(&tile)?;
            f(tile)
        })
    }
}
