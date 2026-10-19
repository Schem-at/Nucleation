use crate::block_entity::BlockEntity;
use crate::block_entity_store::BlockEntityStore;
use crate::block_position::BlockPosition;
use crate::bounding_box::BoundingBox;
use crate::entity::Entity;
use crate::BlockState;
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Re-exported alias kept for downstream Rust crates that referenced the
/// previous concrete map type. New code should use `BlockEntityStore`.
pub type BlockEntityMap = BlockEntityStore;

/// Serde form of `Region::blocks`. The field was `Vec<usize>` when snapshot
/// versions 1–3 were written, and bincode encodes it positionally as u64
/// elements, so keep that wire shape for compatibility.
mod blocks_wire {
    use serde::de::{Error, SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(blocks: &[u32], serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(blocks.len()))?;
        for &b in blocks {
            seq.serialize_element(&(b as u64))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u32>, D::Error> {
        struct BlocksVisitor;
        impl<'de> Visitor<'de> for BlocksVisitor {
            type Value = Vec<u32>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a sequence of palette indices")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<u32>, A::Error> {
                // Cap the pre-allocation: the length prefix is untrusted.
                let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(1 << 24));
                while let Some(v) = seq.next_element::<u64>()? {
                    out.push(
                        u32::try_from(v)
                            .map_err(|_| A::Error::custom("palette index exceeds u32"))?,
                    );
                }
                Ok(out)
            }
        }
        deserializer.deserialize_seq(BlocksVisitor)
    }
}

/// Allocate `len` zeroed (air) voxels without panicking on failure.
///
/// Uses `alloc_zeroed`, so large regions get lazily-zeroed pages from the OS
/// instead of an explicit memset of the whole volume.
pub(crate) fn try_zeroed_blocks(len: usize) -> Result<Vec<u32>, String> {
    if len == 0 {
        return Ok(Vec::new());
    }
    let layout = std::alloc::Layout::array::<u32>(len)
        .map_err(|_| format!("Cannot allocate region with {len} blocks: size overflow"))?;
    // SAFETY: `layout` has non-zero size. A non-null pointer from
    // `alloc_zeroed` with the layout of `[u32; len]` is exactly what
    // `Vec::from_raw_parts` requires for length and capacity `len`, and zero
    // is a valid `u32`.
    unsafe {
        let ptr = std::alloc::alloc_zeroed(layout) as *mut u32;
        if ptr.is_null() {
            return Err(format!(
                "Cannot allocate region with {len} blocks: out of memory"
            ));
        }
        Ok(Vec::from_raw_parts(ptr, len, len))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub name: String,
    pub position: (i32, i32, i32),
    pub size: (i32, i32, i32),
    /// Palette index per voxel, x fastest, then z, then y. `u32` rather than
    /// `usize`: half the memory and bandwidth for every fill, copy, transform
    /// and export, while still holding any palette a region can produce.
    #[serde(with = "blocks_wire")]
    pub blocks: Vec<u32>,
    pub(crate) palette: Vec<BlockState>,
    pub entities: Vec<Entity>,
    pub block_entities: BlockEntityStore,
    /// Reverse palette lookup keyed by BlockState. FxHashMap because the
    /// palette is hit on every `set_block` call and BlockState's std
    /// hash is wasted cycles on a small-cardinality set.
    #[serde(skip, default = "FxHashMap::default")]
    palette_index: FxHashMap<BlockState, usize>,

    #[serde(skip)]
    bbox: BoundingBox,

    /// Tracks the actual min/max coordinates of non-air blocks placed in this region
    /// Unlike bbox (which is pre-allocated for performance), tight_bounds only covers
    /// the space that actually contains blocks
    #[serde(skip)]
    tight_bounds: Option<BoundingBox>,

    // ── Cached fields for hot-path performance ──
    // i64 (not i32) so that index arithmetic doesn't overflow for large
    // regions. `cached_width_x_length` alone exceeds i32::MAX once a region
    // is roughly 50k × 50k, and multiplying it by a Y offset overflows much
    // sooner; the resulting wrap caused a `blocks[index]` OOB panic.
    #[serde(skip)]
    cached_width: i64,
    #[serde(skip)]
    cached_length: i64,
    #[serde(skip)]
    cached_width_x_length: i64,
    #[serde(skip)]
    cached_air_index: u32,
    #[serde(skip)]
    non_air_count: usize,
}

impl Region {
    pub fn new(name: String, position: (i32, i32, i32), size: (i32, i32, i32)) -> Self {
        Self::try_new(name, position, size)
            .expect("region bounds or allocation exceed supported limits")
    }

    /// Construct a region without panicking on invalid bounds, arithmetic overflow,
    /// or allocation failure.
    pub fn try_new(
        name: String,
        position: (i32, i32, i32),
        size: (i32, i32, i32),
    ) -> Result<Self, String> {
        let bounding_box = BoundingBox::try_from_position_and_size(position, size)?;
        let dimension = |min: i32, max: i32, axis: &str| {
            let value = i64::from(max) - i64::from(min) + 1;
            i32::try_from(value)
                .map_err(|_| format!("Region {axis} dimension {value} exceeds i32 limits"))
        };
        let dimensions = (
            dimension(bounding_box.min.0, bounding_box.max.0, "X")?,
            dimension(bounding_box.min.1, bounding_box.max.1, "Y")?,
            dimension(bounding_box.min.2, bounding_box.max.2, "Z")?,
        );
        let volume = usize::try_from(dimensions.0)
            .ok()
            .and_then(|x| x.checked_mul(dimensions.1 as usize))
            .and_then(|xy| xy.checked_mul(dimensions.2 as usize))
            .ok_or_else(|| "Region volume exceeds addressable memory".to_string())?;
        let position_and_size = (bounding_box.min, dimensions);

        let mut palette = Vec::new();
        let mut palette_index: FxHashMap<BlockState, usize> = FxHashMap::default();

        let air = BlockState::new("minecraft:air".to_string());
        palette.push(air.clone());
        palette_index.insert(air, 0);

        let blocks = try_zeroed_blocks(volume)?;

        let mut region = Region {
            name,
            position: position_and_size.0,
            size: position_and_size.1,
            blocks,
            palette,
            palette_index,
            entities: Vec::new(),
            block_entities: BlockEntityStore::default(),
            bbox: bounding_box,
            tight_bounds: None,
            cached_width: 0,
            cached_length: 0,
            cached_width_x_length: 0,
            cached_air_index: 0,
            non_air_count: 0,
        };
        region.rebuild_bbox();
        region.rebuild_air_index();
        Ok(region)
    }

    #[inline(always)]
    pub fn rebuild_bbox(&mut self) {
        self.bbox = BoundingBox::from_position_and_size(self.position, self.size);
        let (w, _, l) = self.bbox.get_dimensions();
        let w64 = w as i64;
        let l64 = l as i64;
        self.cached_width = w64;
        self.cached_length = l64;
        // i64 multiplication: w × l can exceed i32::MAX (e.g. 50k × 50k = 2.5e9).
        self.cached_width_x_length = w64 * l64;
    }

    /// Recompute cached_air_index from the palette.
    pub(crate) fn rebuild_air_index(&mut self) {
        self.cached_air_index = self
            .palette
            .iter()
            .position(|b| b.name == "minecraft:air")
            .map_or(u32::MAX, |i| i as u32);
    }

    /// Recount non-air blocks by scanning the blocks array.
    pub(crate) fn rebuild_non_air_count(&mut self) {
        self.rescan_content();
    }

    /// Rebuild tight bounds by scanning all blocks
    /// This is typically called after deserialization
    pub fn rebuild_tight_bounds(&mut self) {
        self.rescan_content();
    }

    /// Recompute `non_air_count` and `tight_bounds` together in one pass.
    ///
    /// Scans x-rows as slices, so there is no per-voxel coordinate division
    /// and the air comparisons vectorize; each occupied row touches the
    /// bounds once.
    pub(crate) fn rescan_content(&mut self) {
        let air = self.cached_air_index;
        let w = self.cached_width as usize;
        let l = self.cached_length as usize;
        let mut count = 0usize;
        let (mut lo_x, mut lo_y, mut lo_z) = (usize::MAX, usize::MAX, usize::MAX);
        let (mut hi_x, mut hi_y, mut hi_z) = (0usize, 0usize, 0usize);
        if w > 0 && l > 0 {
            for (row_index, row) in self.blocks.chunks_exact(w).enumerate() {
                let row_count = row.iter().filter(|&&b| b != air).count();
                if row_count == 0 {
                    continue;
                }
                count += row_count;
                let first = row.iter().position(|&b| b != air).unwrap_or(0);
                let last = row.iter().rposition(|&b| b != air).unwrap_or(0);
                let (dy, dz) = (row_index / l, row_index % l);
                lo_x = lo_x.min(first);
                hi_x = hi_x.max(last);
                lo_y = lo_y.min(dy);
                hi_y = hi_y.max(dy);
                lo_z = lo_z.min(dz);
                hi_z = hi_z.max(dz);
            }
        }
        self.non_air_count = count;
        self.tight_bounds = (count > 0).then(|| {
            let (bx, by, bz) = self.bbox.min;
            BoundingBox::new(
                (bx + lo_x as i32, by + lo_y as i32, bz + lo_z as i32),
                (bx + hi_x as i32, by + hi_y as i32, bz + hi_z as i32),
            )
        });
    }

    pub fn get_or_insert_in_palette(&mut self, block: &BlockState) -> usize {
        match self.palette_index.get(block) {
            Some(&index) => index,
            None => {
                let index = self.palette.len();
                self.palette.push(block.clone());
                self.palette_index.insert(block.clone(), index);
                index
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.non_air_count == 0
    }

    /// An empty 1×1×1 region has no meaningful position yet: move it to the
    /// first written cell so that growth starts there. Moves in place, so
    /// the palette, entities and block entities are kept.
    #[inline]
    pub(crate) fn recenter_if_empty_unit(&mut self, position: (i32, i32, i32)) {
        if self.size == (1, 1, 1) && self.position != position && self.is_empty() {
            self.position = position;
            self.rebuild_bbox();
        }
    }

    pub fn has_non_air_blocks(&self) -> bool {
        self.non_air_count > 0
    }

    pub fn count_non_air_blocks(&self) -> usize {
        self.non_air_count
    }

    pub(crate) fn rebuild_palette_index(&mut self) {
        self.palette_index =
            FxHashMap::with_capacity_and_hasher(self.palette.len(), Default::default());
        for (index, block) in self.palette.iter().enumerate() {
            self.palette_index.insert(block.clone(), index);
        }
    }

    pub fn get_block_entities_as_list(&self) -> Vec<BlockEntity> {
        self.block_entities
            .iter()
            .map(|(position, template)| {
                let mut entity = template.clone();
                entity.position = position;
                entity
            })
            .collect()
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: &BlockState) -> bool {
        if !self.is_in_region(x, y, z) {
            self.expand_to_fit(x, y, z);
        }

        let index = self.coords_to_index(x, y, z);
        let palette_index = self.get_or_insert_in_palette(block) as u32;
        let old_palette_index = self.blocks[index];
        self.blocks[index] = palette_index;

        // Update non_air_count based on old vs new block
        let old_is_air = old_palette_index == self.cached_air_index;
        let new_is_air = palette_index == self.cached_air_index;
        if old_is_air && !new_is_air {
            self.non_air_count += 1;
        } else if !old_is_air && new_is_air {
            self.non_air_count -= 1;
            self.shrink_tight_bounds_after_removal(x, y, z);
        }

        // Update tight bounds if this is a non-air block
        if !new_is_air {
            self.update_tight_bounds(x, y, z);
        }

        true
    }

    pub fn set_block_entity(&mut self, position: BlockPosition, block_entity: BlockEntity) -> bool {
        self.block_entities
            .insert((position.x, position.y, position.z), block_entity);
        true
    }

    pub fn get_block_entity(&self, position: BlockPosition) -> Option<&BlockEntity> {
        self.block_entities
            .get(&(position.x, position.y, position.z))
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        self.bbox.clone()
    }

    #[inline(always)]
    pub fn coords_to_index(&self, x: i32, y: i32, z: i32) -> usize {
        // Promote to i64 before the multiplications: for a 600M-block region
        // the Y-axis term `dy * cached_width_x_length` is ~5e11, which
        // overflows i32 silently and wraps to a negative — the resulting
        // `as usize` cast then produced a near-`usize::MAX` index and an OOB
        // panic in `set_block`. See nucleation issue #__ for the repro.
        let dx = (x - self.bbox.min.0) as i64;
        let dy = (y - self.bbox.min.1) as i64;
        let dz = (z - self.bbox.min.2) as i64;
        (dx + dz * self.cached_width + dy * self.cached_width_x_length) as usize
    }

    #[inline(always)]
    pub fn index_to_coords(&self, index: usize) -> (i32, i32, i32) {
        let w = self.cached_width as usize;
        let wl = self.cached_width_x_length as usize;
        let dx = (index % w) as i32;
        let dy = (index / wl) as i32;
        let dz = ((index / w) % self.cached_length as usize) as i32;
        (
            dx + self.bbox.min.0,
            dy + self.bbox.min.1,
            dz + self.bbox.min.2,
        )
    }

    #[inline(always)]
    pub fn is_in_region(&self, x: i32, y: i32, z: i32) -> bool {
        self.bbox.contains((x, y, z))
    }

    pub fn get_dimensions(&self) -> (i32, i32, i32) {
        self.bbox.get_dimensions()
    }

    /// Update the tight bounds to include the given coordinate
    fn update_tight_bounds(&mut self, x: i32, y: i32, z: i32) {
        if let Some(bounds) = &mut self.tight_bounds {
            if x < bounds.min.0 {
                bounds.min.0 = x;
            }
            if x > bounds.max.0 {
                bounds.max.0 = x;
            }
            if y < bounds.min.1 {
                bounds.min.1 = y;
            }
            if y > bounds.max.1 {
                bounds.max.1 = y;
            }
            if z < bounds.min.2 {
                bounds.min.2 = z;
            }
            if z > bounds.max.2 {
                bounds.max.2 = z;
            }
        } else {
            self.tight_bounds = Some(BoundingBox::new((x, y, z), (x, y, z)));
        }
    }

    /// Repair tight bounds after clearing an occupied cell.
    ///
    /// Additions can update the cached box in O(1), but a removal only changes
    /// the box when it touches one of its six faces. Scan in that case; interior
    /// removals retain the existing box without any extra work. Clearing the
    /// final block is handled directly so an empty region never reports stale
    /// content bounds.
    fn shrink_tight_bounds_after_removal(&mut self, x: i32, y: i32, z: i32) {
        if self.non_air_count == 0 {
            self.tight_bounds = None;
            return;
        }

        let touches_boundary = self.tight_bounds.as_ref().is_some_and(|bounds| {
            x == bounds.min.0
                || x == bounds.max.0
                || y == bounds.min.1
                || y == bounds.max.1
                || z == bounds.min.2
                || z == bounds.max.2
        });
        if touches_boundary {
            self.rebuild_tight_bounds();
        }
    }

    /// Get the tight bounding box (actual min/max coordinates of placed non-air blocks)
    /// Returns None if no non-air blocks have been placed yet
    pub fn get_tight_bounds(&self) -> Option<BoundingBox> {
        self.tight_bounds.clone()
    }

    /// Get the tight dimensions (width, height, length) of actual block content
    /// Returns (0, 0, 0) if no non-air blocks have been placed yet
    pub fn get_tight_dimensions(&self) -> (i32, i32, i32) {
        self.tight_bounds
            .as_ref()
            .map(|bounds| bounds.get_dimensions())
            .unwrap_or((0, 0, 0))
    }

    /// Whether the allocated region is already the minimal box containing all
    /// blocks and entities. Exporters use this to borrow the region directly
    /// instead of cloning and compacting an already-compact input.
    pub(crate) fn is_content_compact(&self) -> bool {
        self.get_content_bounds().as_ref() == Some(&self.bbox)
    }

    /// Create a compacted region sized to cover all content: non-air blocks,
    /// entities, and block entities. This is used for exporting schematics so
    /// that Litematica (which filters entities outside the region bbox) keeps
    /// every entity/block entity that was added.
    ///
    /// Returns a minimal 1x1x1 clone if the region has no content at all.
    pub fn to_compact(&self) -> Region {
        let Some(content_bounds) = self.get_content_bounds() else {
            let mut empty = Region::new(self.name.clone(), (0, 0, 0), (1, 1, 1));
            empty.palette = self.palette.clone();
            empty.rebuild_palette_index();
            empty.rebuild_air_index();
            return empty;
        };

        // Fast path: when content_bounds == tight_bounds (no entities or
        // block entities outside the block envelope — the common case), the
        // compact region is exactly tight_bounds and the block copy needs no
        // extra offsets. Preserves the original row-copy layout.
        if let Some(tight_bounds) = self.tight_bounds.clone() {
            if tight_bounds == content_bounds {
                return self.to_compact_tight(&tight_bounds);
            }
        }

        // General path: content_bounds may be larger than tight_bounds
        // because entities or block_entities sit outside the block envelope.
        // Compact is sized to content_bounds; block-copy rows are written at
        // (tight.min - content.min) offsets.
        self.to_compact_with_content_bounds(&content_bounds)
    }

    /// Fast-path compaction: tight_bounds and content_bounds are identical,
    /// so the compact region has the same dims as tight_bounds.
    fn to_compact_tight(&self, tight_bounds: &BoundingBox) -> Region {
        let tight_dims = tight_bounds.get_dimensions();
        let tight_pos = tight_bounds.min;

        let mut compact = Region::new(self.name.clone(), tight_pos, tight_dims);
        compact.palette = self.palette.clone();
        compact.rebuild_palette_index();
        compact.rebuild_air_index();

        let old_w = self.cached_width as usize;
        let old_l = self.cached_length as usize;
        let (compact_w, _, compact_l) = compact.bbox.get_dimensions();
        let compact_w = compact_w as usize;
        let compact_l = compact_l as usize;

        let tb_min_x = tight_bounds.min.0;
        let tb_min_y = tight_bounds.min.1;
        let tb_min_z = tight_bounds.min.2;
        // Only read on the wasm32 path below; keep the name so that path compiles.
        #[allow(unused_variables)]
        let tb_max_y = tight_bounds.max.1;
        let tb_max_z = tight_bounds.max.2;

        let x_off_src = (tb_min_x - self.bbox.min.0) as usize;
        let row_len = compact_w;

        #[cfg(target_arch = "wasm32")]
        {
            for y in tb_min_y..=tb_max_y {
                let dy_src = (y - self.bbox.min.1) as usize;
                let dy_dst = (y - tb_min_y) as usize;
                for z in tb_min_z..=tb_max_z {
                    let dz_src = (z - self.bbox.min.2) as usize;
                    let dz_dst = (z - tb_min_z) as usize;
                    let src_start = dy_src * old_w * old_l + dz_src * old_w + x_off_src;
                    let dst_start = dy_dst * compact_w * compact_l + dz_dst * compact_w;
                    compact.blocks[dst_start..dst_start + row_len]
                        .copy_from_slice(&self.blocks[src_start..src_start + row_len]);
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use rayon::prelude::*;
            use rayon::slice::ParallelSliceMut;
            let y_slice_size = compact_w * compact_l;
            let src_blocks = &self.blocks;
            let bbox_min_y = self.bbox.min.1;
            let bbox_min_z = self.bbox.min.2;

            compact
                .blocks
                .par_chunks_mut(y_slice_size)
                .enumerate()
                .for_each(|(dy_dst, dst_y_slice)| {
                    let y = tb_min_y + dy_dst as i32;
                    let dy_src = (y - bbox_min_y) as usize;

                    for z in tb_min_z..=tb_max_z {
                        let dz_src = (z - bbox_min_z) as usize;
                        let dz_dst = (z - tb_min_z) as usize;

                        let src_start = dy_src * old_w * old_l + dz_src * old_w + x_off_src;
                        let dst_start_in_slice = dz_dst * compact_w;

                        dst_y_slice[dst_start_in_slice..dst_start_in_slice + row_len]
                            .copy_from_slice(&src_blocks[src_start..src_start + row_len]);
                    }
                });
        }

        // Compaction copies every occupied cell, so the cached count carries
        // over unchanged.
        compact.non_air_count = self.non_air_count;
        compact.tight_bounds = Some(tight_bounds.clone());

        for (pos, be) in self.block_entities.iter() {
            compact.block_entities.insert(pos, be.clone());
        }
        for entity in &self.entities {
            compact.entities.push(entity.clone());
        }

        compact
    }

    /// General-path compaction: content_bounds is the union of tight_bounds
    /// plus entity and block-entity positions. The compact region is sized
    /// to content_bounds; block-copy rows are written at offsets relative to
    /// content_bounds.min. Cells outside tight_bounds stay at palette index 0
    /// (air — guaranteed by `Region::new`).
    fn to_compact_with_content_bounds(&self, content_bounds: &BoundingBox) -> Region {
        let content_dims = content_bounds.get_dimensions();
        let content_pos = content_bounds.min;

        let mut compact = Region::new(self.name.clone(), content_pos, content_dims);
        compact.palette = self.palette.clone();
        compact.rebuild_palette_index();
        compact.rebuild_air_index();

        let (compact_w, _, compact_l) = compact.bbox.get_dimensions();
        let compact_w = compact_w as usize;
        let compact_l = compact_l as usize;

        if let Some(tight_bounds) = &self.tight_bounds {
            let old_w = self.cached_width as usize;
            let old_l = self.cached_length as usize;

            let tb_min_x = tight_bounds.min.0;
            let tb_min_y = tight_bounds.min.1;
            let tb_min_z = tight_bounds.min.2;
            let tb_max_y = tight_bounds.max.1;
            let tb_max_z = tight_bounds.max.2;

            let (tight_w, _, _) = tight_bounds.get_dimensions();
            let row_len = tight_w as usize;

            let x_off_src = (tb_min_x - self.bbox.min.0) as usize;
            let x_off_dst = (tb_min_x - content_pos.0) as usize;

            #[cfg(target_arch = "wasm32")]
            {
                for y in tb_min_y..=tb_max_y {
                    let dy_src = (y - self.bbox.min.1) as usize;
                    let dy_dst = (y - content_pos.1) as usize;
                    for z in tb_min_z..=tb_max_z {
                        let dz_src = (z - self.bbox.min.2) as usize;
                        let dz_dst = (z - content_pos.2) as usize;
                        let src_start = dy_src * old_w * old_l + dz_src * old_w + x_off_src;
                        let dst_start =
                            dy_dst * compact_w * compact_l + dz_dst * compact_w + x_off_dst;
                        compact.blocks[dst_start..dst_start + row_len]
                            .copy_from_slice(&self.blocks[src_start..src_start + row_len]);
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                use rayon::prelude::*;
                use rayon::slice::ParallelSliceMut;
                let y_slice_size = compact_w * compact_l;
                let src_blocks = &self.blocks;
                let bbox_min_y = self.bbox.min.1;
                let bbox_min_z = self.bbox.min.2;
                let content_min_y = content_pos.1;
                let content_min_z = content_pos.2;

                compact
                    .blocks
                    .par_chunks_mut(y_slice_size)
                    .enumerate()
                    .for_each(|(dy_dst, dst_y_slice)| {
                        let y = content_min_y + dy_dst as i32;
                        if y < tb_min_y || y > tb_max_y {
                            return;
                        }
                        let dy_src = (y - bbox_min_y) as usize;

                        for z in tb_min_z..=tb_max_z {
                            let dz_src = (z - bbox_min_z) as usize;
                            let dz_dst = (z - content_min_z) as usize;

                            let src_start = dy_src * old_w * old_l + dz_src * old_w + x_off_src;
                            let dst_start_in_slice = dz_dst * compact_w + x_off_dst;

                            dst_y_slice[dst_start_in_slice..dst_start_in_slice + row_len]
                                .copy_from_slice(&src_blocks[src_start..src_start + row_len]);
                        }
                    });
            }
        }

        // Compaction copies every occupied cell, so the cached count carries
        // over unchanged.
        compact.non_air_count = self.non_air_count;
        compact.tight_bounds = self.tight_bounds.clone();

        for (pos, be) in self.block_entities.iter() {
            compact.block_entities.insert(pos, be.clone());
        }
        for entity in &self.entities {
            compact.entities.push(entity.clone());
        }

        compact
    }

    #[inline]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<&BlockState> {
        if !self.is_in_region(x, y, z) {
            return None;
        }

        let index = self.coords_to_index(x, y, z);
        self.palette.get(self.blocks[index] as usize)
    }

    pub fn get_block_index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if !self.is_in_region(x, y, z) {
            return None;
        }

        let index = self.coords_to_index(x, y, z);
        Some(self.blocks[index] as usize)
    }

    pub fn volume(&self) -> usize {
        self.size.0 as usize * self.size.1 as usize * self.size.2 as usize
    }

    /// Grow the allocation so it contains `(x, y, z)`. See [`Self::ensure_bounds`].
    pub fn expand_to_fit(&mut self, x: i32, y: i32, z: i32) {
        self.ensure_bounds((x, y, z), (x, y, z));
    }

    /// Ensure the region covers the given bounds, expanding if necessary.
    ///
    /// Growth is geometric and per axis, and only in the direction that
    /// overflowed: an axis of extent `e` grows by at least `max(e / 2, 16)`
    /// cells. A range request into an untouched region (a fill or stamp into
    /// a fresh schematic) allocates exactly its footprint instead. Reallocations are therefore O(log n) per axis however the
    /// region is grown (cell by cell, or by repeated stamps), and the
    /// allocation is never more than ~1.5× the touched extent on any axis.
    /// (A fixed ±64 pad used to turn a few hundred scattered blocks into a
    /// 1.6M-voxel region, while `size / 2` on a thin axis grew it one cell at
    /// a time.)
    pub fn ensure_bounds(&mut self, min: (i32, i32, i32), max: (i32, i32, i32)) {
        let current = self.get_bounding_box();
        if current.contains(min) && current.contains(max) {
            return;
        }
        let untouched = self.size == (1, 1, 1)
            && self.is_empty()
            && self.block_entities.is_empty()
            && self.entities.is_empty();
        let well_formed = min.0 <= max.0 && min.1 <= max.1 && min.2 <= max.2;
        if untouched && well_formed && min != max && self.cached_air_index == 0 {
            let bounds = BoundingBox::new(min, max);
            if let Ok(blocks) = try_zeroed_blocks(bounds.volume() as usize) {
                self.position = bounds.min;
                self.size = bounds.get_dimensions();
                self.blocks = blocks;
                self.rebuild_bbox();
                return;
            }
        }

        fn grow(cur_min: i32, cur_max: i32, req_min: i32, req_max: i32) -> (i32, i32) {
            const MIN_STEP: i64 = 16;
            let (cur_min, cur_max) = (cur_min as i64, cur_max as i64);
            let step = ((cur_max - cur_min + 1) / 2).max(MIN_STEP);
            let clamp = |v: i64| v.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
            let new_min = if (req_min as i64) < cur_min {
                (req_min as i64).min(cur_min - step)
            } else {
                cur_min
            };
            let new_max = if (req_max as i64) > cur_max {
                (req_max as i64).max(cur_max + step)
            } else {
                cur_max
            };
            (clamp(new_min), clamp(new_max))
        }

        let (min_x, max_x) = grow(current.min.0, current.max.0, min.0, max.0);
        let (min_y, max_y) = grow(current.min.1, current.max.1, min.1, max.1);
        let (min_z, max_z) = grow(current.min.2, current.max.2, min.2, max.2);
        self.expand_to_bounding_box(BoundingBox::new(
            (min_x, min_y, min_z),
            (max_x, max_y, max_z),
        ));
    }

    /// Grow the allocation to `new_bounding_box`, which must contain the
    /// current one.
    ///
    /// Works in place: the vector is extended (a realloc, which for large
    /// buffers remaps the existing pages instead of copying them), the old
    /// rows are moved back-to-front to their new offsets, and only the cells
    /// that were not part of the old allocation are set to air. Compared with
    /// allocating a second full buffer, this halves peak memory during growth
    /// and avoids faulting in a fresh copy of every resident page.
    fn expand_to_bounding_box(&mut self, new_bounding_box: BoundingBox) {
        let new_size = new_bounding_box.get_dimensions();
        let new_position = new_bounding_box.min;

        if new_size == self.size && new_position == self.position {
            return;
        }
        debug_assert!(
            new_bounding_box.contains(self.bbox.min) && new_bounding_box.contains(self.bbox.max)
        );

        let air_id = self.cached_air_index;
        let old_w = self.cached_width as usize;
        let old_l = self.cached_length as usize;
        let (_, old_h, _) = self.bbox.get_dimensions();
        let old_h = old_h as usize;

        let (new_w, new_h, new_l) = new_bounding_box.get_dimensions();
        let (new_w, new_h, new_l) = (new_w as usize, new_h as usize, new_l as usize);

        // Offset of old min within new coordinate system
        let x_off = (self.bbox.min.0 - new_bounding_box.min.0) as usize;
        let y_off = (self.bbox.min.1 - new_bounding_box.min.1) as usize;
        let z_off = (self.bbox.min.2 - new_bounding_box.min.2) as usize;

        let old_len = self.blocks.len();
        self.blocks
            .resize(new_bounding_box.volume() as usize, air_id);

        // Growing only towards +y appends whole layers: the old cells are
        // already in place.
        let in_place = x_off == 0 && y_off == 0 && z_off == 0 && new_w == old_w && new_l == old_l;
        if !in_place {
            // Every old row moves to an equal or higher index, and rows keep
            // their order, so moving the last row first never overwrites a
            // row that has not been moved yet.
            for dy in (0..old_h).rev() {
                for dz in (0..old_l).rev() {
                    let src_start = dy * old_w * old_l + dz * old_w;
                    let dst_start = (dy + y_off) * new_w * new_l + (dz + z_off) * new_w + x_off;
                    if dst_start != src_start {
                        self.blocks
                            .copy_within(src_start..src_start + old_w, dst_start);
                    }
                }
            }
            // Every cell outside the moved rows becomes air. Only cells below
            // the old length can hold a stale value: `resize` already wrote
            // air past it.
            'rows: for dy in 0..new_h {
                for dz in 0..new_l {
                    let row_start = (dy * new_l + dz) * new_w;
                    if row_start >= old_len {
                        break 'rows;
                    }
                    // Clip to the old length: cells past it are already air.
                    let row = &mut self.blocks[row_start..][..new_w.min(old_len - row_start)];
                    let holds_old_row =
                        (y_off..y_off + old_h).contains(&dy) && (z_off..z_off + old_l).contains(&dz);
                    if holds_old_row {
                        let end = row.len();
                        row[..x_off.min(end)].fill(air_id);
                        row[(x_off + old_w).min(end)..].fill(air_id);
                    } else {
                        row.fill(air_id);
                    }
                }
            }
        }

        self.position = new_position;
        self.size = new_size;
        self.rebuild_bbox();
    }
    fn calculate_bits_per_block(&self) -> usize {
        let palette_size = self.palette.len();
        if palette_size <= 1 {
            2
        } else {
            std::cmp::max(
                (usize::BITS - (palette_size - 1).leading_zeros()) as usize,
                2,
            )
        }
    }

    pub fn merge(&mut self, other: &Region) {
        let combined_bounding_box = self.get_bounding_box().union(&other.get_bounding_box());
        let new_size = combined_bounding_box.get_dimensions();
        let new_position = combined_bounding_box.min;

        let (new_w, _, new_l) = new_size;
        let new_w = new_w as usize;
        let new_l = new_l as usize;

        let mut new_blocks = vec![0u32; combined_bounding_box.volume() as usize];
        let mut new_palette = self.palette.clone();
        let mut reverse_new_palette: HashMap<BlockState, usize> = HashMap::new();
        for (index, block) in self.palette.iter().enumerate() {
            reverse_new_palette.insert(block.clone(), index);
        }

        // Phase 5: Pre-build remap table for self (identity since palette is same)
        // For self, palette indices are identity, so just row-copy with offset
        {
            let old_w = self.cached_width as usize;
            let old_l = self.cached_length as usize;
            let (_, old_h, _) = self.bbox.get_dimensions();
            let old_h = old_h as usize;

            let x_off = (self.bbox.min.0 - combined_bounding_box.min.0) as usize;
            let y_off = (self.bbox.min.1 - combined_bounding_box.min.1) as usize;
            let z_off = (self.bbox.min.2 - combined_bounding_box.min.2) as usize;

            for dy in 0..old_h {
                for dz in 0..old_l {
                    let src_start = dy * old_w * old_l + dz * old_w;
                    let dst_start = (dy + y_off) * new_w * new_l + (dz + z_off) * new_w + x_off;
                    new_blocks[dst_start..dst_start + old_w]
                        .copy_from_slice(&self.blocks[src_start..src_start + old_w]);
                }
            }
        }

        // Phase 5: Pre-build remap table for other palette
        let other_air_index = other.cached_air_index;
        let mut other_remap: Vec<u32> = Vec::with_capacity(other.palette.len());
        for block in &other.palette {
            if let Some(&existing) = reverse_new_palette.get(block) {
                other_remap.push(existing as u32);
            } else {
                let new_idx = new_palette.len();
                new_palette.push(block.clone());
                reverse_new_palette.insert(block.clone(), new_idx);
                other_remap.push(new_idx as u32);
            }
        }

        // Copy other blocks using remap table, row by row
        {
            let other_w = other.cached_width as usize;
            let other_l = other.cached_length as usize;
            let (_, other_h, _) = other.bbox.get_dimensions();
            let other_h = other_h as usize;

            let x_off = (other.bbox.min.0 - combined_bounding_box.min.0) as usize;
            let y_off = (other.bbox.min.1 - combined_bounding_box.min.1) as usize;
            let z_off = (other.bbox.min.2 - combined_bounding_box.min.2) as usize;

            for dy in 0..other_h {
                for dz in 0..other_l {
                    let src_start = dy * other_w * other_l + dz * other_w;
                    let dst_start = (dy + y_off) * new_w * new_l + (dz + z_off) * new_w + x_off;
                    for dx in 0..other_w {
                        let src_idx = other.blocks[src_start + dx];
                        if src_idx == other_air_index {
                            continue; // Skip air from other region
                        }
                        new_blocks[dst_start + dx] = other_remap[src_idx as usize];
                    }
                }
            }
        }

        // Update region properties
        self.position = new_position;
        self.size = new_size;
        self.blocks = new_blocks;
        self.palette = new_palette;

        // Rebuild all caches
        self.rebuild_bbox();
        self.rebuild_palette_index();
        self.rebuild_air_index();
        // Compaction must include blocks added outside the original region.
        self.rescan_content();

        // Merge entities and block entities
        self.merge_entities(other);
        self.merge_block_entities(other);
    }

    /// Merge `other` as a higher-precedence region. Unlike the general merge,
    /// air cells inside `other`'s allocated portion of its visible content
    /// bounds overwrite lower-precedence blocks, matching composite reads.
    pub(crate) fn merge_with_precedence(&mut self, other: &Region) {
        let visible_bounds = other.get_tight_bounds();
        self.merge(other);

        let Some(bounds) = visible_bounds else {
            return;
        };
        // `merge` sized `self` to cover `other`, so every cell is in range.
        // Write directly and rescan once: clearing cells through `set_block`
        // rescans the whole region whenever a cleared cell is on the edge.
        let air = BlockState::new("minecraft:air".to_string());
        let air_index = self.get_or_insert_in_palette(&air) as u32;
        self.rebuild_air_index();
        for y in bounds.min.1..=bounds.max.1 {
            for z in bounds.min.2..=bounds.max.2 {
                for x in bounds.min.0..=bounds.max.0 {
                    if other
                        .get_block(x, y, z)
                        .is_some_and(|block| block.name == "minecraft:air")
                    {
                        let index = self.coords_to_index(x, y, z);
                        self.blocks[index] = air_index;
                    }
                }
            }
        }
        self.rescan_content();
    }

    fn merge_entities(&mut self, other: &Region) {
        self.entities.extend(other.entities.iter().cloned());
    }

    fn merge_block_entities(&mut self, other: &Region) {
        self.block_entities.extend(
            other
                .block_entities
                .iter()
                .map(|(pos, be)| (pos, be.clone())),
        );
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Tight bounding box of all content (non-air blocks, entities, and block
    /// entities). Entity positions are floored. Returns `None` only if the
    /// region has no blocks, no entities, and no block entities.
    ///
    /// Exporters use this (not `tight_bounds`) to ensure entity and block
    /// entity positions are covered by the region size written to disk —
    /// Litematica filters out entities outside the region bounding box.
    pub fn get_content_bounds(&self) -> Option<BoundingBox> {
        let mut result = self.tight_bounds.clone();

        let mut expand = |p: (i32, i32, i32)| {
            result = Some(match result.take() {
                Some(bb) => BoundingBox::new(
                    (bb.min.0.min(p.0), bb.min.1.min(p.1), bb.min.2.min(p.2)),
                    (bb.max.0.max(p.0), bb.max.1.max(p.1), bb.max.2.max(p.2)),
                ),
                None => BoundingBox::new(p, p),
            });
        };

        for &pos in self.block_entities.keys() {
            expand(pos);
        }
        for entity in &self.entities {
            expand((
                entity.position.0.floor() as i32,
                entity.position.1.floor() as i32,
                entity.position.2.floor() as i32,
            ));
        }

        result
    }

    pub fn remove_entity(&mut self, index: usize) -> Option<Entity> {
        if index < self.entities.len() {
            Some(self.entities.remove(index))
        } else {
            None
        }
    }

    pub fn add_block_entity(&mut self, block_entity: BlockEntity) {
        self.block_entities
            .insert(block_entity.position, block_entity);
    }

    pub fn remove_block_entity(&mut self, position: (i32, i32, i32)) -> Option<BlockEntity> {
        self.block_entities.remove(&position)
    }

    pub fn to_nbt(&self) -> NbtTag {
        let mut tag = NbtCompound::new();
        tag.insert("Name", NbtTag::String(self.name.clone()));
        tag.insert(
            "Position",
            NbtTag::IntArray(vec![self.position.0, self.position.1, self.position.2]),
        );
        tag.insert(
            "Size",
            NbtTag::IntArray(vec![self.size.0, self.size.1, self.size.2]),
        );

        let mut blocks_tag = NbtCompound::new();
        for (index, &block_index) in self.blocks.iter().enumerate() {
            let (x, y, z) = self.index_to_coords(index);
            blocks_tag.insert(
                format!("{},{},{}", x, y, z),
                NbtTag::Int(block_index as i32),
            );
        }
        tag.insert("Blocks", NbtTag::Compound(blocks_tag));

        let palette_list = NbtList::from(
            self.palette
                .iter()
                .map(|b| b.to_nbt())
                .collect::<Vec<NbtTag>>(),
        );
        tag.insert("Palette", NbtTag::List(palette_list));

        let entities_list = NbtList::from(
            self.entities
                .iter()
                .map(|e| e.to_nbt())
                .collect::<Vec<NbtTag>>(),
        );
        tag.insert("Entities", NbtTag::List(entities_list));

        let mut block_entities_tag = NbtCompound::new();
        for ((x, y, z), block_entity) in self.block_entities.iter() {
            block_entities_tag.insert(format!("{},{},{}", x, y, z), block_entity.to_nbt());
        }
        tag.insert("BlockEntities", NbtTag::Compound(block_entities_tag));

        NbtTag::Compound(tag)
    }

    pub fn from_nbt(nbt: &NbtCompound) -> Result<Self, String> {
        let name = nbt
            .get::<_, &str>("Name")
            .map_err(|e| format!("Failed to get Region Name: {}", e))?
            .to_string();

        let position = match nbt.get::<_, &NbtTag>("Position") {
            Ok(NbtTag::IntArray(arr)) if arr.len() == 3 => (arr[0], arr[1], arr[2]),
            _ => return Err("Invalid Position tag".to_string()),
        };

        let size = match nbt.get::<_, &NbtTag>("Size") {
            Ok(NbtTag::IntArray(arr)) if arr.len() == 3 => (arr[0], arr[1], arr[2]),
            _ => return Err("Invalid Size tag".to_string()),
        };

        let palette_tag = nbt
            .get::<_, &NbtList>("Palette")
            .map_err(|e| format!("Failed to get Palette: {}", e))?;
        let palette: Vec<BlockState> = palette_tag
            .iter()
            .filter_map(|tag| {
                if let NbtTag::Compound(compound) = tag {
                    BlockState::from_nbt(compound).ok()
                } else {
                    None
                }
            })
            .collect();

        let blocks_tag = nbt
            .get::<_, &NbtCompound>("Blocks")
            .map_err(|e| format!("Failed to get Blocks: {}", e))?;
        let mut blocks = vec![0u32; (size.0 * size.1 * size.2) as usize];
        for (key, value) in blocks_tag.inner() {
            if let NbtTag::Int(index) = value {
                let coords: Vec<i32> = key.split(',').map(|s| s.parse::<i32>().unwrap()).collect();
                if coords.len() == 3 {
                    let block_index =
                        (coords[1] * size.0 * size.2 + coords[2] * size.0 + coords[0]) as usize;
                    blocks[block_index] = *index as u32;
                }
            }
        }

        let entities_tag = nbt
            .get::<_, &NbtList>("Entities")
            .map_err(|e| format!("Failed to get Entities: {}", e))?;
        let entities = entities_tag
            .iter()
            .filter_map(|tag| {
                if let NbtTag::Compound(compound) = tag {
                    Entity::from_nbt(compound).ok()
                } else {
                    None
                }
            })
            .collect();

        let block_entities_tag = nbt
            .get::<_, &NbtCompound>("BlockEntities")
            .map_err(|e| format!("Failed to get BlockEntities: {}", e))?;
        let mut block_entities = BlockEntityStore::default();
        for (key, value) in block_entities_tag.inner() {
            if let NbtTag::Compound(be_compound) = value {
                let coords: Vec<i32> = key.split(',').map(|s| s.parse::<i32>().unwrap()).collect();
                if coords.len() == 3 {
                    let block_entity = BlockEntity::from_nbt(be_compound);
                    block_entities.insert((coords[0], coords[1], coords[2]), block_entity);
                }
            }
        }

        let mut region = Region {
            name,
            position,
            size,
            blocks,
            palette,
            entities,
            block_entities,
            palette_index: FxHashMap::default(),
            bbox: BoundingBox::from_position_and_size(position, size),
            tight_bounds: None,
            cached_width: 0,
            cached_length: 0,
            cached_width_x_length: 0,
            cached_air_index: 0,
            non_air_count: 0,
        };

        region.rebuild_palette_index();
        region.rebuild_bbox();
        region.rebuild_air_index();
        region.rescan_content();

        Ok(region)
    }

    pub fn to_litematic_nbt(&self) -> NbtCompound {
        let mut region_nbt = NbtCompound::new();

        // 1. Position and Size
        region_nbt.insert(
            "Position",
            NbtTag::IntArray(vec![self.position.0, self.position.1, self.position.2]),
        );
        region_nbt.insert(
            "Size",
            NbtTag::IntArray(vec![self.size.0, self.size.1, self.size.2]),
        );

        // 2. BlockStatePalette
        let palette_nbt = NbtList::from(
            self.palette
                .iter()
                .map(|block_state| block_state.to_nbt())
                .collect::<Vec<NbtTag>>(),
        );
        region_nbt.insert("BlockStatePalette", NbtTag::List(palette_nbt));

        // 3. BlockStates (packed long array)
        let block_states = self.create_packed_block_states();
        region_nbt.insert("BlockStates", NbtTag::LongArray(block_states));

        // 4. Entities
        let entities_nbt = NbtList::from(
            self.entities
                .iter()
                .map(|entity| entity.to_nbt())
                .collect::<Vec<NbtTag>>(),
        );
        region_nbt.insert("Entities", NbtTag::List(entities_nbt));

        // 5. TileEntities
        region_nbt.insert("TileEntities", NbtTag::List(NbtList::new()));

        region_nbt
    }

    /// Unpack a Litematica `BlockStates` long array for this region's volume
    /// and palette. Missing longs read as zero; use
    /// [`Self::try_unpack_block_states`] for untrusted input.
    pub fn unpack_block_states(&self, packed_states: &[i64]) -> Vec<u32> {
        let bits_per_block = self.calculate_bits_per_block();
        let volume = self.volume();
        let needed = (volume * bits_per_block).div_ceil(64);
        if packed_states.len() >= needed {
            return Self::unpack_exact(packed_states, bits_per_block, volume);
        }
        let mut padded = packed_states.to_vec();
        padded.resize(needed, 0);
        Self::unpack_exact(&padded, bits_per_block, volume)
    }

    /// Like [`Self::unpack_block_states`], but rejects a long array of the
    /// wrong length for the volume, or cells that reference a missing palette
    /// entry.
    pub fn try_unpack_block_states(&self, packed_states: &[i64]) -> Result<Vec<u32>, String> {
        let bits_per_block = self.calculate_bits_per_block();
        let volume = self.volume();
        let needed = volume
            .checked_mul(bits_per_block)
            .map(|bits| bits.div_ceil(64))
            .ok_or("BlockStates volume overflow")?;
        if packed_states.len() != needed {
            return Err(format!(
                "BlockStates has {} longs, expected {needed} for the region volume",
                packed_states.len()
            ));
        }
        let blocks = Self::unpack_exact(packed_states, bits_per_block, volume);
        let palette_len = self.palette.len();
        if blocks.iter().any(|&b| b as usize >= palette_len) {
            return Err("BlockStates references a missing palette entry".to_string());
        }
        Ok(blocks)
    }

    /// Unpack `volume` cells; `packed_states` must hold at least
    /// `ceil(volume * bits / 64)` longs.
    fn unpack_exact(packed_states: &[i64], bits_per_block: usize, volume: usize) -> Vec<u32> {
        let mask: u64 = (1u64 << bits_per_block) - 1;
        let mut blocks = vec![0u32; volume];

        if 64 % bits_per_block == 0 {
            // Entries never cross i64 boundaries (bits_per_block = 2, 4, 8,
            // 16, 32): a fixed number of cells per long, so the inner loop
            // has a constant trip count.
            let per_long = 64 / bits_per_block;
            for (cells, &packed) in blocks.chunks_mut(per_long).zip(packed_states) {
                let packed = packed as u64;
                for (i, cell) in cells.iter_mut().enumerate() {
                    *cell = ((packed >> (i * bits_per_block)) & mask) as u32;
                }
            }
        } else {
            // Entries may cross i64 boundaries.
            for (index, cell) in blocks.iter_mut().enumerate() {
                let bit_index = index * bits_per_block;
                let start_long_index = bit_index / 64;
                let start_offset = bit_index % 64;

                let value = if start_offset + bits_per_block <= 64 {
                    ((packed_states[start_long_index] as u64) >> start_offset) & mask
                } else {
                    let low_bits = (packed_states[start_long_index] as u64) >> start_offset;
                    let high_bits = (packed_states[start_long_index + 1] as u64)
                        & ((1u64 << (bits_per_block - (64 - start_offset))) - 1);
                    low_bits | (high_bits << (64 - start_offset))
                };
                *cell = value as u32;
            }
        }

        blocks
    }

    pub(crate) fn create_packed_block_states(&self) -> Vec<i64> {
        let bits_per_block = self.calculate_bits_per_block();
        let size = self.blocks.len();
        let expected_len = (size * bits_per_block).div_ceil(64);

        let mut packed_states = vec![0i64; expected_len];
        let mask: u64 = (1u64 << bits_per_block) - 1;

        if 64 % bits_per_block == 0 {
            // Fast path: entries never cross i64 boundaries (bits_per_block = 2, 4, 8, 16, 32)
            let entries_per_long = 64 / bits_per_block;
            for (chunk_idx, chunk) in self.blocks.chunks(entries_per_long).enumerate() {
                let mut packed: u64 = 0;
                for (i, &block_state) in chunk.iter().enumerate() {
                    packed |= (block_state as u64 & mask) << (i * bits_per_block);
                }
                packed_states[chunk_idx] = packed as i64;
            }
        } else {
            // Slow path: entries may cross i64 boundaries
            for (index, &block_state) in self.blocks.iter().enumerate() {
                let bit_index = index * bits_per_block;
                let start_long_index = bit_index / 64;
                let end_long_index = (bit_index + bits_per_block - 1) / 64;
                let start_offset = bit_index % 64;
                let value = (block_state as u64) & mask;

                packed_states[start_long_index] |= (value << start_offset) as i64;
                if start_long_index != end_long_index {
                    packed_states[end_long_index] |= (value >> (64 - start_offset)) as i64;
                }
            }
        }

        packed_states
    }

    pub fn get_palette(&self) -> Vec<BlockState> {
        self.palette.clone()
    }

    /// Canonicalize this region's palette without changing any placed block.
    ///
    /// Property order is normalized, unused entries are removed, equivalent
    /// states are deduplicated, air is index zero, and remaining states are
    /// sorted by their canonical block string. Returns `(old_len, new_len,
    /// rewritten_cells)`.
    pub fn canonicalize_palette(&mut self) -> (usize, usize, usize) {
        let old_len = self.palette.len();
        let mut normalized = Vec::with_capacity(old_len);
        for state in &self.palette {
            let mut state = state.clone();
            state
                .properties
                .sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
            normalized.push(state);
        }

        let air = BlockState::new("minecraft:air");
        let mut used: Vec<BlockState> = self
            .blocks
            .iter()
            .filter_map(|&index| normalized.get(index as usize).cloned())
            .collect();
        used.push(air.clone());
        used.sort_by(|a, b| {
            let a_air = a.name == "minecraft:air";
            let b_air = b.name == "minecraft:air";
            b_air
                .cmp(&a_air)
                .then_with(|| a.to_string().cmp(&b.to_string()))
        });
        used.dedup();

        let lookup: FxHashMap<BlockState, u32> = used
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, state)| (state, index as u32))
            .collect();
        let mut rewritten = 0;
        for index in &mut self.blocks {
            if let Some(state) = normalized.get(*index as usize) {
                let next = lookup[state];
                if next != *index {
                    rewritten += 1;
                }
                *index = next;
            }
        }
        self.palette = used;
        self.rebuild_palette_index();
        self.rebuild_air_index();
        self.rebuild_non_air_count();
        (old_len, self.palette.len(), rewritten)
    }

    /// Rewrite every palette state through `map`, returning the number of
    /// placed cells whose state changed. Equivalent results are compacted by
    /// [`Region::canonicalize_palette`].
    pub fn transform_palette_states<F>(&mut self, mut map: F) -> usize
    where
        F: FnMut(&BlockState) -> BlockState,
    {
        let next: Vec<BlockState> = self.palette.iter().map(&mut map).collect();
        let changed_indices: Vec<bool> = self
            .palette
            .iter()
            .zip(&next)
            .map(|(before, after)| before != after)
            .collect();
        let changed_cells = self
            .blocks
            .iter()
            .filter(|&&index| {
                changed_indices
                    .get(index as usize)
                    .copied()
                    .unwrap_or(false)
            })
            .count();
        self.palette = next;
        self.rebuild_palette_index();
        self.rebuild_air_index();
        self.canonicalize_palette();
        changed_cells
    }

    pub(crate) fn get_palette_nbt(&self) -> NbtList {
        let mut palette = NbtList::new();
        for block in &self.palette {
            palette.push(block.to_nbt());
        }
        palette
    }

    pub fn count_block_types(&self) -> HashMap<BlockState, usize> {
        let mut block_counts = HashMap::new();
        for block_index in &self.blocks {
            let block_state = &self.palette[*block_index as usize];
            *block_counts.entry(block_state.clone()).or_insert(0) += 1;
        }
        block_counts
    }

    pub fn count_blocks(&self) -> usize {
        self.non_air_count
    }

    /// Number of canonical block states currently stored by this region.
    pub fn palette_len(&self) -> usize {
        self.palette.len()
    }

    pub fn get_palette_index(&self, block: &BlockState) -> Option<usize> {
        self.palette.iter().position(|b| b == block)
    }

    // Transformation methods

    /// Flip the region along the X axis
    pub fn flip_x(&mut self) {
        use crate::transforms::{transform_block_state_flip, Axis};

        // Phase 3: Reverse each X-row in-place
        let w = self.cached_width as usize;
        let l = self.cached_length as usize;
        let wl = self.cached_width_x_length as usize;
        let (_, h, _) = self.bbox.get_dimensions();
        let h = h as usize;
        for dy in 0..h {
            for dz in 0..l {
                let start = dy * wl + dz * w;
                self.blocks[start..start + w].reverse();
            }
        }

        // Transform block state properties
        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_flip(block_state, Axis::X));
        }

        self.palette = new_palette;
        self.rebuild_palette_index();
        self.rebuild_air_index();
        // A flip permutes cells: the air count is unchanged and the content
        // bounds mirror within the allocation.
        self.mirror_tight_bounds(0);

        // Transform block entities — pure position remap, no palette work.
        let max_x = self.bbox.max.0;
        let min_x = self.bbox.min.0;
        self.block_entities
            .remap_positions(|(x, y, z)| (max_x - (x - min_x), y, z));

        // Transform entities
        for entity in &mut self.entities {
            let rel_x = entity.position.0 - self.bbox.min.0 as f64;
            entity.position.0 = self.bbox.max.0 as f64 - rel_x;
        }
    }

    /// `(position, palette index)` for every cell inside the content (tight)
    /// bounds, x fastest, then z, then y. Cells outside the tight bounds are
    /// allocation padding and always air, so they are not visited; air inside
    /// is. Rows are walked as slices, with no per-cell coordinate division.
    pub fn content_cells(&self) -> impl Iterator<Item = ((i32, i32, i32), u32)> + '_ {
        self.tight_bounds.clone().into_iter().flat_map(move |b| {
            let (x0, x1) = (b.min.0, b.max.0);
            let z_range = b.min.2..=b.max.2;
            (b.min.1..=b.max.1).flat_map(move |y| {
                z_range.clone().flat_map(move |z| {
                    let start = self.coords_to_index(x0, y, z);
                    self.blocks[start..start + (x1 - x0 + 1) as usize]
                        .iter()
                        .zip(x0..)
                        .map(move |(&index, x)| ((x, y, z), index))
                })
            })
        })
    }

    /// Mirror the cached tight bounds within the allocation along `axis`
    /// (0 = x, 1 = y, 2 = z), as a flip does to the content.
    fn mirror_tight_bounds(&mut self, axis: usize) {
        let (lo, hi) = match axis {
            0 => (self.bbox.min.0, self.bbox.max.0),
            1 => (self.bbox.min.1, self.bbox.max.1),
            _ => (self.bbox.min.2, self.bbox.max.2),
        };
        if let Some(bounds) = &mut self.tight_bounds {
            let (min, max) = match axis {
                0 => (&mut bounds.min.0, &mut bounds.max.0),
                1 => (&mut bounds.min.1, &mut bounds.max.1),
                _ => (&mut bounds.min.2, &mut bounds.max.2),
            };
            let (old_min, old_max) = (*min, *max);
            *min = hi - (old_max - lo);
            *max = hi - (old_min - lo);
        }
    }

    /// Flip the region along the Y axis
    pub fn flip_y(&mut self) {
        use crate::transforms::{transform_block_state_flip, Axis};

        // Phase 3: Swap Y-layers using swap_nonoverlapping
        let wl = self.cached_width_x_length as usize;
        let (_, h, _) = self.bbox.get_dimensions();
        let h = h as usize;
        // Swap layer dy with layer (h-1-dy)
        for dy in 0..h / 2 {
            let off_a = dy * wl;
            let off_b = (h - 1 - dy) * wl;
            let (low, high) = self.blocks.split_at_mut(off_b);
            low[off_a..off_a + wl].swap_with_slice(&mut high[..wl]);
        }

        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_flip(block_state, Axis::Y));
        }

        self.palette = new_palette;
        self.rebuild_palette_index();
        self.rebuild_air_index();
        // A flip permutes cells: the air count is unchanged and the content
        // bounds mirror within the allocation.
        self.mirror_tight_bounds(1);

        let max_y = self.bbox.max.1;
        let min_y = self.bbox.min.1;
        self.block_entities
            .remap_positions(|(x, y, z)| (x, max_y - (y - min_y), z));

        for entity in &mut self.entities {
            let rel_y = entity.position.1 - self.bbox.min.1 as f64;
            entity.position.1 = self.bbox.max.1 as f64 - rel_y;
        }
    }

    /// Flip the region along the Z axis
    pub fn flip_z(&mut self) {
        use crate::transforms::{transform_block_state_flip, Axis};

        // Phase 3: Swap Z-rows within each Y-layer
        let w = self.cached_width as usize;
        let l = self.cached_length as usize;
        let wl = self.cached_width_x_length as usize;
        let (_, h, _) = self.bbox.get_dimensions();
        let h = h as usize;
        for dy in 0..h {
            let layer_off = dy * wl;
            for dz in 0..l / 2 {
                let off_a = layer_off + dz * w;
                let off_b = layer_off + (l - 1 - dz) * w;
                let (low, high) = self.blocks.split_at_mut(off_b);
                low[off_a..off_a + w].swap_with_slice(&mut high[..w]);
            }
        }

        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_flip(block_state, Axis::Z));
        }

        self.palette = new_palette;
        self.rebuild_palette_index();
        self.rebuild_air_index();
        // A flip permutes cells: the air count is unchanged and the content
        // bounds mirror within the allocation.
        self.mirror_tight_bounds(2);

        let max_z = self.bbox.max.2;
        let min_z = self.bbox.min.2;
        self.block_entities
            .remap_positions(|(x, y, z)| (x, y, max_z - (z - min_z)));

        for entity in &mut self.entities {
            let rel_z = entity.position.2 - self.bbox.min.2 as f64;
            entity.position.2 = self.bbox.max.2 as f64 - rel_z;
        }
    }

    /// Move this region and all attached content by an integer offset.
    pub fn translate(&mut self, dx: i32, dy: i32, dz: i32) -> Result<(), String> {
        let shifted = |value: i32, delta: i32| {
            value
                .checked_add(delta)
                .ok_or_else(|| "Region translation exceeds the i32 coordinate range".to_string())
        };

        let new_position = (
            shifted(self.position.0, dx)?,
            shifted(self.position.1, dy)?,
            shifted(self.position.2, dz)?,
        );
        shifted(self.bbox.max.0, dx)?;
        shifted(self.bbox.max.1, dy)?;
        shifted(self.bbox.max.2, dz)?;
        for &(x, y, z) in self.block_entities.keys() {
            shifted(x, dx)?;
            shifted(y, dy)?;
            shifted(z, dz)?;
        }

        let new_tight_bounds = self
            .tight_bounds
            .as_ref()
            .map(|bounds| {
                Ok::<BoundingBox, String>(BoundingBox::new(
                    (
                        shifted(bounds.min.0, dx)?,
                        shifted(bounds.min.1, dy)?,
                        shifted(bounds.min.2, dz)?,
                    ),
                    (
                        shifted(bounds.max.0, dx)?,
                        shifted(bounds.max.1, dy)?,
                        shifted(bounds.max.2, dz)?,
                    ),
                ))
            })
            .transpose()?;

        self.position = new_position;
        self.tight_bounds = new_tight_bounds;
        self.block_entities
            .remap_positions(|(x, y, z)| (x + dx, y + dy, z + dz));
        for entity in &mut self.entities {
            entity.position.0 += dx as f64;
            entity.position.1 += dy as f64;
            entity.position.2 += dz as f64;
        }
        self.rebuild_bbox();
        Ok(())
    }

    /// Rotate the region around the Y axis (horizontal plane)
    /// Degrees must be 90, 180, or 270
    pub fn rotate_y(&mut self, degrees: i32) {
        if degrees % 90 != 0 || degrees == 0 {
            return; // Only support 90-degree rotations
        }

        let normalized_degrees = degrees.rem_euclid(360);
        let rotations = normalized_degrees / 90;

        for _ in 0..rotations {
            if self.rotate_y_90_at(self.position).is_err() {
                return;
            }
        }
    }

    pub(crate) fn rotate_y_90_at(&mut self, position: (i32, i32, i32)) -> Result<(), String> {
        use crate::transforms::{transform_block_state_rotate, Axis};

        // For 90-degree rotation around Y:
        // X and Z swap and dimensions change: (sx, sy, sz) -> (sz, sy, sx)
        // Keep the origin at min corner

        let old_bbox = self.bbox.clone();
        let (old_size_x, size_y, old_size_z) = self.size;

        // New dimensions after rotation
        let new_size = (old_size_z, size_y, old_size_x);
        let new_bbox = BoundingBox::try_from_position_and_size(position, new_size)?;

        // (dx, dy, dz) -> (sz-1-dz, dy, dx). Every cell is written, so the
        // zeroed allocation's contents never survive.
        let (w, h, l) = (old_size_x as usize, size_y as usize, old_size_z as usize);
        let (new_w, new_l) = (l, w);
        let mut new_blocks = try_zeroed_blocks(w * h * l)?;
        let mut src = 0;
        for dy in 0..h {
            let layer = dy * new_w * new_l;
            for dz in 0..l {
                let column = layer + (l - 1 - dz);
                for (dx, &block) in self.blocks[src..src + w].iter().enumerate() {
                    new_blocks[column + dx * new_w] = block;
                }
                src += w;
            }
        }
        let new_tight = self.tight_bounds.as_ref().map(|b| {
            let map = |x: i32, z: i32| {
                (
                    new_bbox.min.0 + (old_size_z - 1 - (z - old_bbox.min.2)),
                    new_bbox.min.2 + (x - old_bbox.min.0),
                )
            };
            let ((ax, az), (bx, bz)) = (map(b.min.0, b.min.2), map(b.max.0, b.max.2));
            BoundingBox::new((ax.min(bx), b.min.1, az.min(bz)), (ax.max(bx), b.max.1, az.max(bz)))
        });

        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_rotate(block_state, Axis::Y, 90));
        }

        let new_bbox_clone = new_bbox.clone();

        self.position = new_bbox.to_position_and_size().0;
        self.size = new_bbox.to_position_and_size().1;
        self.blocks = new_blocks;
        self.palette = new_palette;
        self.rebuild_bbox();
        self.rebuild_palette_index();
        self.rebuild_air_index();
        // A rotation permutes cells: the air count is unchanged.
        self.tight_bounds = new_tight;

        // Transform block entities — pure position remap.
        let old_min_x = old_bbox.min.0;
        let old_min_z = old_bbox.min.2;
        let new_min_x = new_bbox_clone.min.0;
        let new_min_z = new_bbox_clone.min.2;
        let osize_z = old_size_z;
        self.block_entities.remap_positions(move |(x, y, z)| {
            let rel_x = x - old_min_x;
            let rel_z = z - old_min_z;
            let new_rel_x = osize_z - 1 - rel_z;
            let new_rel_z = rel_x;
            (new_min_x + new_rel_x, y, new_min_z + new_rel_z)
        });

        // Transform entities
        for entity in &mut self.entities {
            let rel_x = entity.position.0 - old_bbox.min.0 as f64;
            let rel_z = entity.position.2 - old_bbox.min.2 as f64;
            let new_rel_x = old_size_z as f64 - 1.0 - rel_z;
            let new_rel_z = rel_x;
            entity.position.0 = new_bbox_clone.min.0 as f64 + new_rel_x;
            entity.position.2 = new_bbox_clone.min.2 as f64 + new_rel_z;
        }
        Ok(())
    }

    /// Rotate the region around the X axis
    /// Degrees must be 90, 180, or 270
    pub fn rotate_x(&mut self, degrees: i32) {
        if degrees % 90 != 0 || degrees == 0 {
            return;
        }

        let normalized_degrees = degrees.rem_euclid(360);
        let rotations = normalized_degrees / 90;

        for _ in 0..rotations {
            if self.rotate_x_90_at(self.position).is_err() {
                return;
            }
        }
    }

    pub(crate) fn rotate_x_90_at(&mut self, position: (i32, i32, i32)) -> Result<(), String> {
        use crate::transforms::{transform_block_state_rotate, Axis};

        let old_bbox = self.bbox.clone();
        let (size_x, old_size_y, old_size_z) = self.size;

        // New dimensions after rotation around X
        let new_size = (size_x, old_size_z, old_size_y);
        let new_bbox = BoundingBox::try_from_position_and_size(position, new_size)?;

        // (dx, dy, dz) -> (dx, sz-1-dz, dy): x-rows stay contiguous.
        let (w, h, l) = (size_x as usize, old_size_y as usize, old_size_z as usize);
        let new_l = h;
        let mut new_blocks = try_zeroed_blocks(w * h * l)?;
        let mut src = 0;
        for dy in 0..h {
            for dz in 0..l {
                let dst = dy * w + (l - 1 - dz) * w * new_l;
                new_blocks[dst..dst + w].copy_from_slice(&self.blocks[src..src + w]);
                src += w;
            }
        }
        let new_tight = self.tight_bounds.as_ref().map(|b| {
            let map = |y: i32, z: i32| {
                (
                    new_bbox.min.1 + (old_size_z - 1 - (z - old_bbox.min.2)),
                    new_bbox.min.2 + (y - old_bbox.min.1),
                )
            };
            let ((ay, az), (by, bz)) = (map(b.min.1, b.min.2), map(b.max.1, b.max.2));
            BoundingBox::new((b.min.0, ay.min(by), az.min(bz)), (b.max.0, ay.max(by), az.max(bz)))
        });

        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_rotate(block_state, Axis::X, 90));
        }

        let new_bbox_clone = new_bbox.clone();

        self.position = new_bbox.to_position_and_size().0;
        self.size = new_bbox.to_position_and_size().1;
        self.blocks = new_blocks;
        self.palette = new_palette;
        self.rebuild_bbox();
        self.rebuild_palette_index();
        self.rebuild_air_index();
        // A rotation permutes cells: the air count is unchanged.
        self.tight_bounds = new_tight;

        let old_min_y = old_bbox.min.1;
        let old_min_z = old_bbox.min.2;
        let new_min_y = new_bbox_clone.min.1;
        let new_min_z = new_bbox_clone.min.2;
        let osize_z = old_size_z;
        self.block_entities.remap_positions(move |(x, y, z)| {
            let rel_y = y - old_min_y;
            let rel_z = z - old_min_z;
            let new_rel_y = osize_z - 1 - rel_z;
            let new_rel_z = rel_y;
            (x, new_min_y + new_rel_y, new_min_z + new_rel_z)
        });

        for entity in &mut self.entities {
            let rel_y = entity.position.1 - old_bbox.min.1 as f64;
            let rel_z = entity.position.2 - old_bbox.min.2 as f64;
            let new_rel_y = old_size_z as f64 - 1.0 - rel_z;
            let new_rel_z = rel_y;
            entity.position.1 = new_bbox_clone.min.1 as f64 + new_rel_y;
            entity.position.2 = new_bbox_clone.min.2 as f64 + new_rel_z;
        }
        Ok(())
    }

    /// Rotate the region around the Z axis
    /// Degrees must be 90, 180, or 270
    pub fn rotate_z(&mut self, degrees: i32) {
        if degrees % 90 != 0 || degrees == 0 {
            return;
        }

        let normalized_degrees = degrees.rem_euclid(360);
        let rotations = normalized_degrees / 90;

        for _ in 0..rotations {
            if self.rotate_z_90_at(self.position).is_err() {
                return;
            }
        }
    }

    pub(crate) fn rotate_z_90_at(&mut self, position: (i32, i32, i32)) -> Result<(), String> {
        use crate::transforms::{transform_block_state_rotate, Axis};

        let old_bbox = self.bbox.clone();
        let (old_size_x, old_size_y, size_z) = self.size;

        // New dimensions after rotation around Z
        let new_size = (old_size_y, old_size_x, size_z);
        let new_bbox = BoundingBox::try_from_position_and_size(position, new_size)?;

        // (dx, dy, dz) -> (sy-1-dy, dx, dz).
        let (w, h, l) = (old_size_x as usize, old_size_y as usize, size_z as usize);
        let (new_w, new_l) = (h, l);
        let mut new_blocks = try_zeroed_blocks(w * h * l)?;
        let mut src = 0;
        for dy in 0..h {
            for dz in 0..l {
                let base = (h - 1 - dy) + dz * new_w;
                for (dx, &block) in self.blocks[src..src + w].iter().enumerate() {
                    new_blocks[base + dx * new_w * new_l] = block;
                }
                src += w;
            }
        }
        let new_tight = self.tight_bounds.as_ref().map(|b| {
            let map = |x: i32, y: i32| {
                (
                    new_bbox.min.0 + (old_size_y - 1 - (y - old_bbox.min.1)),
                    new_bbox.min.1 + (x - old_bbox.min.0),
                )
            };
            let ((ax, ay), (bx, by)) = (map(b.min.0, b.min.1), map(b.max.0, b.max.1));
            BoundingBox::new((ax.min(bx), ay.min(by), b.min.2), (ax.max(bx), ay.max(by), b.max.2))
        });

        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_rotate(block_state, Axis::Z, 90));
        }

        let new_bbox_clone = new_bbox.clone();

        self.position = new_bbox.to_position_and_size().0;
        self.size = new_bbox.to_position_and_size().1;
        self.blocks = new_blocks;
        self.palette = new_palette;
        self.rebuild_bbox();
        self.rebuild_palette_index();
        self.rebuild_air_index();
        // A rotation permutes cells: the air count is unchanged.
        self.tight_bounds = new_tight;

        let old_min_x = old_bbox.min.0;
        let old_min_y = old_bbox.min.1;
        let new_min_x = new_bbox_clone.min.0;
        let new_min_y = new_bbox_clone.min.1;
        let osize_y = old_size_y;
        self.block_entities.remap_positions(move |(x, y, z)| {
            let rel_x = x - old_min_x;
            let rel_y = y - old_min_y;
            let new_rel_x = osize_y - 1 - rel_y;
            let new_rel_y = rel_x;
            (new_min_x + new_rel_x, new_min_y + new_rel_y, z)
        });

        for entity in &mut self.entities {
            let rel_x = entity.position.0 - old_bbox.min.0 as f64;
            let rel_y = entity.position.1 - old_bbox.min.1 as f64;
            let new_rel_x = old_size_y as f64 - 1.0 - rel_y;
            let new_rel_y = rel_x;
            entity.position.0 = new_bbox_clone.min.0 as f64 + new_rel_x;
            entity.position.1 = new_bbox_clone.min.1 as f64 + new_rel_y;
        }
        Ok(())
    }

    // ── Fast-path methods for Python binding performance ──

    /// Look up or insert a block in the palette by name only (no properties).
    /// Uses a linear scan of the palette (palettes are typically 2-20 entries).
    /// Avoids creating a full BlockState + HashMap lookup on cache miss.
    #[inline]
    pub fn get_or_insert_palette_by_name(&mut self, name: &str) -> usize {
        // Linear scan is faster than HashMap for small palettes
        for (i, bs) in self.palette.iter().enumerate() {
            if bs.name == name && bs.properties.is_empty() {
                return i;
            }
        }
        // Not found — insert new entry
        let index = self.palette.len();
        let block = BlockState::new(name.to_string());
        self.palette_index.insert(block.clone(), index);
        self.palette.push(block);
        index
    }

    /// Look up or insert a block in the palette by full BlockState
    /// (id + properties). Mirrors `get_or_insert_palette_by_name` but for the
    /// case where we have a complete state (e.g. from parsing a block string).
    #[inline]
    pub fn get_or_insert_palette_by_state(&mut self, state: &BlockState) -> usize {
        if let Some(&i) = self.palette_index.get(state) {
            return i;
        }
        let index = self.palette.len();
        self.palette_index.insert(state.clone(), index);
        self.palette.push(state.clone());
        index
    }

    /// Copy the cells of `src` inside `clip` into this region, translated by
    /// `offset`, skipping cells covered by any of `shadows` (higher-precedence
    /// layers) and cells whose state is in `excluded`.
    ///
    /// Every written cell takes the source state, air included, and its block
    /// entity: the destination's previous block entity is dropped and the
    /// source's (if any) is copied. `clip` must lie inside `src`'s allocation
    /// and its translation must fit in `i32`.
    ///
    /// Works on whole x-rows: the source palette is remapped once (lazily, so
    /// unused source states never enter this palette), the air count and
    /// tight bounds are maintained incrementally, and block entities are
    /// handled in one pass each instead of per cell.
    pub(crate) fn stamp_from(
        &mut self,
        src: &Region,
        clip: &BoundingBox,
        offset: (i64, i64, i64),
        excluded: &[BlockState],
        shadows: &[BoundingBox],
    ) {
        const UNRESOLVED: u32 = u32::MAX;
        const SKIP: u32 = u32::MAX - 1;
        let translate = |p: (i32, i32, i32)| {
            (
                (p.0 as i64 + offset.0) as i32,
                (p.1 as i64 + offset.1) as i32,
                (p.2 as i64 + offset.2) as i32,
            )
        };
        let shadowed = |p: (i32, i32, i32)| shadows.iter().any(|s| s.contains(p));

        let dest_min = translate(clip.min);
        self.recenter_if_empty_unit(dest_min);
        self.ensure_bounds(dest_min, translate(clip.max));
        if self.cached_air_index == u32::MAX {
            // Keep air comparisons meaningful if the source writes air.
            self.get_or_insert_in_palette(&BlockState::new("minecraft:air".to_string()));
            self.rebuild_air_index();
        }
        let air = self.cached_air_index;
        let old_bounds = self.tight_bounds.clone();
        let on_old_edge = |(x, y, z): (i32, i32, i32)| {
            old_bounds.as_ref().is_some_and(|b| {
                x == b.min.0 || x == b.max.0 || y == b.min.1 || y == b.max.1 || z == b.min.2
                    || z == b.max.2
            })
        };

        let mut remap = vec![UNRESOLVED; src.palette.len()];
        let mut air_delta: i64 = 0;
        let mut cleared_edge = false;
        let mut added: Option<BoundingBox> = None;
        let mut intervals: Vec<(i32, i32)> = Vec::new();

        for y in clip.min.1..=clip.max.1 {
            for z in clip.min.2..=clip.max.2 {
                // x-intervals of this row not owned by a higher layer.
                intervals.clear();
                intervals.push((clip.min.0, clip.max.0));
                for s in shadows {
                    if y < s.min.1 || y > s.max.1 || z < s.min.2 || z > s.max.2 {
                        continue;
                    }
                    let mut next = Vec::with_capacity(intervals.len() + 1);
                    for &(a, b) in &intervals {
                        if s.max.0 < a || s.min.0 > b {
                            next.push((a, b));
                            continue;
                        }
                        if a < s.min.0 {
                            next.push((a, s.min.0 - 1));
                        }
                        if b > s.max.0 {
                            next.push((s.max.0 + 1, b));
                        }
                    }
                    intervals = next;
                }

                for &(x0, x1) in &intervals {
                    let src_start = src.coords_to_index(x0, y, z);
                    let (dx0, dy, dz) = translate((x0, y, z));
                    let dst_start = self.coords_to_index(dx0, dy, dz);
                    let len = (x1 - x0 + 1) as usize;
                    let (mut lo, mut hi) = (usize::MAX, 0usize);
                    for i in 0..len {
                        let state = src.blocks[src_start + i] as usize;
                        let mut mapped = remap[state];
                        if mapped == UNRESOLVED {
                            let block = &src.palette[state];
                            mapped = if excluded.contains(block) {
                                SKIP
                            } else {
                                self.get_or_insert_in_palette(block) as u32
                            };
                            remap[state] = mapped;
                        }
                        if mapped == SKIP {
                            continue;
                        }
                        let cell = &mut self.blocks[dst_start + i];
                        let old = *cell;
                        *cell = mapped;
                        if mapped == air {
                            if old != air {
                                air_delta -= 1;
                                cleared_edge |= on_old_edge((dx0 + i as i32, dy, dz));
                            }
                        } else {
                            if old == air {
                                air_delta += 1;
                            }
                            lo = lo.min(i);
                            hi = hi.max(i);
                        }
                    }
                    if lo <= hi {
                        let row = BoundingBox::new(
                            (dx0 + lo as i32, dy, dz),
                            (dx0 + hi as i32, dy, dz),
                        );
                        added = Some(match added {
                            Some(bounds) => bounds.union(&row),
                            None => row,
                        });
                    }
                }
            }
        }

        self.non_air_count = (self.non_air_count as i64 + air_delta) as usize;
        if cleared_edge {
            // A cleared cell on the old edge may shrink the bounds.
            self.rescan_content();
        } else if let Some(added) = added {
            self.tight_bounds = Some(match self.tight_bounds.take() {
                Some(bounds) => bounds.union(&added),
                None => added,
            });
        }

        // Block entities: drop the destination's at every written cell, then
        // copy the source's.
        let written_from = |src_pos: (i32, i32, i32)| {
            clip.contains(src_pos)
                && !shadowed(src_pos)
                && remap[src.blocks[src.coords_to_index(src_pos.0, src_pos.1, src_pos.2)] as usize]
                    != SKIP
        };
        if !self.block_entities.is_empty() {
            self.block_entities.retain(|(x, y, z), _| {
                let src_pos = (
                    x as i64 - offset.0,
                    y as i64 - offset.1,
                    z as i64 - offset.2,
                );
                let in_range = |v: i64| v >= i32::MIN as i64 && v <= i32::MAX as i64;
                !(in_range(src_pos.0)
                    && in_range(src_pos.1)
                    && in_range(src_pos.2)
                    && written_from((src_pos.0 as i32, src_pos.1 as i32, src_pos.2 as i32)))
            });
        }
        for (pos, block_entity) in src.block_entities.iter() {
            if written_from(pos) {
                let dest = translate(pos);
                let mut copied = block_entity.clone();
                copied.position = dest;
                self.block_entities.set(dest, copied);
            }
        }
    }

    /// Write one palette index at many positions, all inside the allocation.
    ///
    /// The air count and tight bounds are maintained in bulk: clearing cells
    /// rescans at most once (only if a cleared cell was on the old edge),
    /// instead of once per edge cell as repeated single writes would.
    pub(crate) fn fill_positions(&mut self, palette_index: usize, positions: &[(i32, i32, i32)]) {
        let palette_index = palette_index as u32;
        let air = self.cached_air_index;
        let new_is_air = palette_index == air;
        let old_bounds = self.tight_bounds.clone();
        let mut air_delta: i64 = 0;
        let mut cleared_edge = false;
        let mut added: Option<BoundingBox> = None;
        for &(x, y, z) in positions {
            let index = self.coords_to_index(x, y, z);
            let old = std::mem::replace(&mut self.blocks[index], palette_index);
            if new_is_air {
                if old != air {
                    air_delta -= 1;
                    cleared_edge |= old_bounds.as_ref().is_some_and(|b| {
                        x == b.min.0 || x == b.max.0 || y == b.min.1 || y == b.max.1 || z == b.min.2
                            || z == b.max.2
                    });
                }
            } else {
                if old == air {
                    air_delta += 1;
                }
                added = Some(match added {
                    Some(b) => BoundingBox::new(
                        (b.min.0.min(x), b.min.1.min(y), b.min.2.min(z)),
                        (b.max.0.max(x), b.max.1.max(y), b.max.2.max(z)),
                    ),
                    None => BoundingBox::new((x, y, z), (x, y, z)),
                });
            }
        }
        self.non_air_count = (self.non_air_count as i64 + air_delta) as usize;
        if cleared_edge {
            self.rescan_content();
        } else if let Some(added) = added {
            self.tight_bounds = Some(match self.tight_bounds.take() {
                Some(bounds) => bounds.union(&added),
                None => added,
            });
        }
    }

    /// Direct array write with air-count and tight-bounds bookkeeping.
    /// Caller must ensure (x, y, z) is within region bounds.
    #[inline(always)]
    pub fn set_block_at_index_unchecked(&mut self, palette_index: usize, x: i32, y: i32, z: i32) {
        let palette_index = palette_index as u32;
        let index = self.coords_to_index(x, y, z);
        let old_palette_index = self.blocks[index];
        self.blocks[index] = palette_index;

        let old_is_air = old_palette_index == self.cached_air_index;
        let new_is_air = palette_index == self.cached_air_index;
        if old_is_air && !new_is_air {
            self.non_air_count += 1;
        } else if !old_is_air && new_is_air {
            self.non_air_count -= 1;
            self.shrink_tight_bounds_after_removal(x, y, z);
        }

        if !new_is_air {
            self.update_tight_bounds(x, y, z);
        }
    }

    /// Get the block name at a position without cloning anything.
    /// Returns None if out of bounds or if the block is air.
    #[inline]
    pub fn get_block_name(&self, x: i32, y: i32, z: i32) -> Option<&str> {
        if !self.is_in_region(x, y, z) {
            return None;
        }
        let index = self.coords_to_index(x, y, z);
        self.palette
            .get(self.blocks[index] as usize)
            .map(|bs| bs.name.as_str())
    }

    /// Fill a rectangular region with a single palette index.
    /// Uses row-level `fill()` for maximum throughput.
    /// Caller must ensure min/max are within region bounds and palette_index is valid.
    pub fn fill_uniform(
        &mut self,
        min: (i32, i32, i32),
        max: (i32, i32, i32),
        palette_index: usize,
    ) {
        let palette_index = palette_index as u32;
        let new_is_air = palette_index == self.cached_air_index;

        // Whole-region fills know the final count and bounds without reading
        // the previous contents. This is the common path for newly-created
        // cuboids and turns the operation into one contiguous slice fill.
        if min == self.bbox.min && max == self.bbox.max {
            self.blocks.fill(palette_index);
            if new_is_air {
                self.non_air_count = 0;
                self.tight_bounds = None;
            } else {
                self.non_air_count = self.blocks.len();
                self.tight_bounds = Some(self.bbox.clone());
            }
            return;
        }

        let w = self.cached_width;
        let wl = self.cached_width_x_length;
        let base_x = self.bbox.min.0;
        let base_y = self.bbox.min.1;
        let base_z = self.bbox.min.2;

        let dx_min = (min.0 - base_x) as i64;
        let dx_max = (max.0 - base_x) as i64;
        let row_len = (dx_max - dx_min + 1) as usize;

        let mut air_delta: i64 = 0;

        for y in min.1..=max.1 {
            let dy = (y - base_y) as i64;
            for z in min.2..=max.2 {
                let dz = (z - base_z) as i64;
                let row_start = (dx_min + dz * w + dy * wl) as usize;
                let row_end = row_start + row_len;
                let row = &mut self.blocks[row_start..row_end];

                // Count air blocks being replaced/created for delta tracking
                // Phase 2: Manual SIMD-style chunking (8 elements at a time)
                let mut chunks = row.chunks_exact(8);
                for chunk in &mut chunks {
                    let mut is_air_count = 0;
                    is_air_count += (chunk[0] == self.cached_air_index) as i64;
                    is_air_count += (chunk[1] == self.cached_air_index) as i64;
                    is_air_count += (chunk[2] == self.cached_air_index) as i64;
                    is_air_count += (chunk[3] == self.cached_air_index) as i64;
                    is_air_count += (chunk[4] == self.cached_air_index) as i64;
                    is_air_count += (chunk[5] == self.cached_air_index) as i64;
                    is_air_count += (chunk[6] == self.cached_air_index) as i64;
                    is_air_count += (chunk[7] == self.cached_air_index) as i64;

                    if !new_is_air {
                        air_delta += is_air_count;
                    } else {
                        air_delta -= 8 - is_air_count;
                    }
                }

                // Handle remainder
                for &old in chunks.remainder() {
                    let old_is_air = old == self.cached_air_index;
                    if old_is_air && !new_is_air {
                        air_delta += 1;
                    } else if !old_is_air && new_is_air {
                        air_delta -= 1;
                    }
                }

                row.fill(palette_index);
            }
        }

        // Batch update non_air_count
        self.non_air_count = (self.non_air_count as i64 + air_delta) as usize;

        // Update tight bounds once for the entire fill. A bulk clear may remove
        // several faces at once, so one post-write scan is both simpler and far
        // cheaper than repairing the cache for every cleared cell.
        if !new_is_air {
            self.update_tight_bounds(min.0, min.1, min.2);
            self.update_tight_bounds(max.0, max.1, max.2);
        } else if air_delta < 0 {
            self.rebuild_tight_bounds();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockState;

    // Litematica's EXACT continuous-stream packing (LitematicaBitArray.setAt) — the
    // on-disk BlockStates format. Proves nucleation's unpack_block_states reads a
    // genuine Litematica long array byte-for-byte, for EVERY bit width including the
    // power-of-two widths that use the "fast path".
    fn litematica_pack(values: &[usize], bits: usize) -> Vec<i64> {
        let len = (values.len() * bits).div_ceil(64).max(1);
        let mut longs = vec![0i64; len];
        let mask = (1u64 << bits) - 1;
        for (i, &v) in values.iter().enumerate() {
            let start_offset = i * bits;
            let start_arr = start_offset >> 6;
            let end_arr = ((i + 1) * bits - 1) >> 6;
            let start_bit = start_offset & 63;
            let mut s = longs[start_arr] as u64;
            s = (s & !(mask << start_bit)) | ((v as u64 & mask) << start_bit);
            longs[start_arr] = s as i64;
            if start_arr != end_arr {
                let end_offset = 64 - start_bit;
                let j1 = bits - end_offset;
                let mut e = longs[end_arr] as u64;
                e = ((e >> j1) << j1) | ((v as u64 & mask) >> end_offset);
                longs[end_arr] = e as i64;
            }
        }
        longs
    }

    #[test]
    fn unpack_matches_litematica_continuous_stream_all_bit_widths() {
        for (bits, palette_len) in [
            (2usize, 4usize),
            (3, 8),
            (4, 16),
            (5, 32),
            (6, 64),
            (7, 128),
            (8, 256),
        ] {
            let volume = 200usize; // spans many longs
            let values: Vec<usize> = (0..volume).map(|i| (i * 7 + 1) % palette_len).collect();
            let packed = litematica_pack(&values, bits);
            let mut region = Region::new("t".to_string(), (0, 0, 0), (volume as i32, 1, 1));
            region.palette = (0..palette_len)
                .map(|i| BlockState::new(format!("minecraft:b{}", i)))
                .collect();
            assert_eq!(
                region.calculate_bits_per_block(),
                bits,
                "bits setup for palette_len={}",
                palette_len
            );
            // Litematica long-count is ceil(volume*bits/64); confirm nucleation reads exactly that.
            assert_eq!(packed.len(), (volume * bits).div_ceil(64));
            let unpacked = region.unpack_block_states(&packed);
            let values: Vec<u32> = values.iter().map(|&v| v as u32).collect();
            assert_eq!(unpacked, values, "unpack mismatch at bits={}", bits);
        }
    }

    /// Flips and rotations update the air count and tight bounds
    /// analytically instead of rescanning; they must agree with a rescan, and
    /// four quarter turns (or two flips) must restore the cells.
    #[test]
    fn transforms_keep_cached_content_consistent() {
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        let mut next = |n: u64| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed % n) as i32
        };
        let states = [
            BlockState::new("minecraft:stone".to_string()),
            BlockState::new("minecraft:glass".to_string()),
        ];
        for _ in 0..200 {
            let origin = (next(9) - 4, next(9) - 4, next(9) - 4);
            let size = (next(7) + 1, next(7) + 1, next(7) + 1);
            let mut region = Region::new("T".to_string(), origin, size);
            for _ in 0..next(12) {
                let p = (
                    origin.0 + next(size.0 as u64),
                    origin.1 + next(size.1 as u64),
                    origin.2 + next(size.2 as u64),
                );
                region.set_block(p.0, p.1, p.2, &states[next(2) as usize]);
            }
            let before: Vec<u32> = region.blocks.clone();
            let check = |r: &Region| {
                let mut fresh = r.clone();
                fresh.rescan_content();
                assert_eq!(r.non_air_count, fresh.non_air_count);
                assert_eq!(r.tight_bounds, fresh.tight_bounds);
            };
            for flip in 0..3 {
                for _ in 0..2 {
                    match flip {
                        0 => region.flip_x(),
                        1 => region.flip_y(),
                        _ => region.flip_z(),
                    }
                    check(&region);
                }
                assert_eq!(region.blocks, before);
            }
            for axis in 0..3 {
                for _ in 0..4 {
                    let at = region.position;
                    match axis {
                        0 => region.rotate_x_90_at(at).unwrap(),
                        1 => region.rotate_y_90_at(at).unwrap(),
                        _ => region.rotate_z_90_at(at).unwrap(),
                    }
                    check(&region);
                }
                assert_eq!(region.blocks, before);
            }
        }
    }

    #[test]
    fn try_unpack_block_states_requires_the_exact_length() {
        // 4 cells at 2 bits each: exactly one long.
        let region = Region::new("t".to_string(), (0, 0, 0), (2, 1, 2));
        assert!(region.try_unpack_block_states(&[0]).is_ok());
        assert!(region.try_unpack_block_states(&[]).is_err());
        assert!(region.try_unpack_block_states(&[0, 0]).is_err());
    }

    #[test]
    fn test_pack_block_states_to_long_array() {
        //array from 1 to 16
        let blocks = (1..=16).collect::<Vec<u32>>();
        let mut palette = vec![BlockState::new("minecraft:air".to_string())];
        for i in 1..=16 {
            palette.push(BlockState::new(format!("minecraft:wool{}", i)));
        }
        let region = Region {
            name: "Test".to_string(),
            position: (0, 0, 0),
            size: (16, 1, 1),
            blocks: blocks.clone(),
            palette,
            entities: Vec::new(),
            block_entities: BlockEntityStore::default(),
            palette_index: FxHashMap::default(),
            bbox: BoundingBox::from_position_and_size((0, 0, 0), (16, 1, 1)),
            tight_bounds: None,
            cached_width: 16,
            cached_length: 1,
            cached_width_x_length: 16,
            cached_air_index: 0,
            non_air_count: 16,
        };
        let packed_states = region.create_packed_block_states();
        assert_eq!(packed_states.len(), 2);
        assert_eq!(packed_states, vec![-3013672028691362751, 33756]);

        let unpacked_blocks = region.unpack_block_states(&packed_states);
        assert_eq!(unpacked_blocks, blocks);
    }

    /// Regression: with the old i32-typed `cached_width_x_length` and
    /// i32 multiplication in `coords_to_index`, asking for an index inside
    /// a region whose footprint exceeded 50k × 19k blocks would silently
    /// overflow and wrap to a negative i32. `as usize` then produced a
    /// near-`usize::MAX` value and `blocks[index]` panicked OOB.
    ///
    /// We construct a `Region` with tiny `blocks` but cached fields that
    /// match a real >500M-block region observed in the wild. The math
    /// must produce a sane (large) index, not a wrapped one.
    #[test]
    fn coords_to_index_does_not_overflow_for_huge_regions() {
        // Dimensions from the build-extractor mega-terrain repro:
        // bounds (-47472, -64, -5216) .. (34975, 319, 13727)
        let width: i64 = 82448;
        let length: i64 = 18944;
        let wl: i64 = width * length; // 1_561_754_112 — just barely fits i32
        assert!(wl > 0 && wl < i64::from(i32::MAX));

        let region = Region {
            name: "huge".to_string(),
            position: (-47472, -64, -5216),
            size: (82448, 384, 18944),
            blocks: Vec::new(), // we never actually index into this
            palette: vec![BlockState::new("minecraft:air".to_string())],
            entities: Vec::new(),
            block_entities: BlockEntityStore::default(),
            palette_index: FxHashMap::default(),
            bbox: BoundingBox::new((-47472, -64, -5216), (34975, 319, 13727)),
            tight_bounds: None,
            cached_width: width,
            cached_length: length,
            cached_width_x_length: wl,
            cached_air_index: 0,
            non_air_count: 0,
        };

        // The problematic point: dy=300 makes `dy * wl ≈ 4.7e11` — well past
        // i32::MAX. Pre-fix this returned ~usize::MAX-N; post-fix the index
        // is the expected linear position.
        let idx = region.coords_to_index(-47472, 236, -5216);
        let expected: usize = (300i64 * wl) as usize;
        assert_eq!(
            idx, expected,
            "coords_to_index overflowed (got {idx}, expected {expected})"
        );

        // Spot-check the opposite corner of the bbox: every term contributes.
        let idx_far = region.coords_to_index(34975, 319, 13727);
        let expected_far: usize = ((width - 1) + (length - 1) * width + (384i64 - 1) * wl) as usize;
        assert_eq!(idx_far, expected_far);
    }

    #[test]
    fn test_region_creation() {
        let region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        assert_eq!(region.name, "Test");
        assert_eq!(region.position, (0, 0, 0));
        assert_eq!(region.size, (2, 2, 2));
        assert_eq!(region.blocks.len(), 8);
        assert_eq!(region.palette.len(), 1);
        assert_eq!(region.palette[0].name, "minecraft:air");
    }

    #[test]
    fn test_set_and_get_block() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        assert!(region.set_block(0, 0, 0, &stone));
        assert_eq!(region.get_block(0, 0, 0), Some(&stone));
        assert_eq!(
            region.get_block(1, 1, 1),
            Some(&BlockState::new("minecraft:air".to_string()))
        );
        assert_eq!(region.get_block(2, 2, 2), None);
    }

    #[test]
    fn test_expand_to_fit() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        region.set_block(0, 0, 0, &stone);
        let new_size = (3, 3, 3);
        region.expand_to_fit(new_size.0, new_size.1, new_size.2);

        assert_eq!(region.get_block(0, 0, 0), Some(&stone));
        assert_eq!(
            region.get_block(3, 3, 3),
            Some(&BlockState::new("minecraft:air".to_string()))
        );
    }

    #[test]
    fn test_entities() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let entity = Entity::new("minecraft:creeper".to_string(), (0.5, 0.0, 0.5));

        region.add_entity(entity.clone());
        assert_eq!(region.entities.len(), 1);

        let removed = region.remove_entity(0);
        assert_eq!(removed, Some(entity));
        assert_eq!(region.entities.len(), 0);
    }

    #[test]
    fn test_block_entities() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let block_entity = BlockEntity::new("minecraft:chest".to_string(), (0, 0, 0));

        region.add_block_entity(block_entity.clone());
        assert_eq!(region.block_entities.len(), 1);

        let removed = region.remove_block_entity((0, 0, 0));
        assert_eq!(removed, Some(block_entity));
        assert_eq!(region.block_entities.len(), 0);
    }

    #[test]
    fn test_to_and_from_nbt() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());
        region.set_block(0, 0, 0, &stone);

        let nbt = region.to_nbt();
        let deserialized_region = match nbt {
            NbtTag::Compound(compound) => Region::from_nbt(&compound).unwrap(),
            _ => panic!("Expected NbtTag::Compound"),
        };

        assert_eq!(region.name, deserialized_region.name);
        assert_eq!(region.position, deserialized_region.position);
        assert_eq!(region.size, deserialized_region.size);
        assert_eq!(
            region.get_block(0, 0, 0),
            deserialized_region.get_block(0, 0, 0)
        );
    }

    #[test]
    fn test_to_litematic_nbt() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());
        region.set_block(0, 0, 0, &stone);

        let nbt = region.to_litematic_nbt();

        assert!(nbt.contains_key("Position"));
        assert!(nbt.contains_key("Size"));
        assert!(nbt.contains_key("BlockStatePalette"));
        assert!(nbt.contains_key("BlockStates"));
        assert!(nbt.contains_key("Entities"));
        assert!(nbt.contains_key("TileEntities"));
    }

    #[test]
    fn test_count_blocks() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        assert_eq!(region.count_blocks(), 0);

        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 1, 1, &stone);

        assert_eq!(region.count_blocks(), 2);
    }

    #[test]
    fn test_region_merge() {
        let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (2, 2, 2));
        let mut region2 = Region::new("Test2".to_string(), (2, 2, 2), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());
        #[test]
        fn test_region_merge() {
            let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (2, 2, 2));
            let mut region2 = Region::new("Test2".to_string(), (2, 2, 2), (2, 2, 2));
            let stone = BlockState::new("minecraft:stone".to_string());

            region1.set_block(0, 0, 0, &stone);
            region2.set_block(2, 2, 2, &stone);

            region1.merge(&region2);

            assert_eq!(region1.size, (4, 4, 4));
            assert_eq!(region1.get_block(0, 0, 0), Some(&stone));
            assert_eq!(region1.get_block(2, 2, 2), Some(&stone));
        }

        #[test]
        fn test_region_merge_different_palettes() {
            let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (2, 2, 2));
            let mut region2 = Region::new("Test2".to_string(), (2, 2, 2), (2, 2, 2));
            let stone = BlockState::new("minecraft:stone".to_string());
            let dirt = BlockState::new("minecraft:dirt".to_string());

            region1.set_block(0, 0, 0, &stone);
            region2.set_block(2, 2, 2, &dirt);

            region1.merge(&region2);

            assert_eq!(region1.size, (4, 4, 4));
            assert_eq!(region1.get_block(0, 0, 0), Some(&stone));
            assert_eq!(region1.get_block(2, 2, 2), Some(&dirt));
        }

        #[test]
        fn test_region_merge_different_overlapping_palettes() {
            let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (2, 2, 2));
            let mut region2 = Region::new("Test2".to_string(), (1, 1, 1), (2, 2, 2));
            let stone = BlockState::new("minecraft:stone".to_string());
            let dirt = BlockState::new("minecraft:dirt".to_string());

            region1.set_block(0, 0, 0, &stone);
            region1.set_block(1, 1, 1, &dirt);

            region2.set_block(2, 2, 2, &dirt);

            region1.merge(&region2);

            assert_eq!(region1.size, (3, 3, 3));
            assert_eq!(region1.get_block(0, 0, 0), Some(&stone));
            assert_eq!(region1.get_block(1, 1, 1), Some(&dirt));
            assert_eq!(region1.get_block(2, 2, 2), Some(&dirt));
        }
        region1.set_block(0, 0, 0, &stone);
        region2.set_block(2, 2, 2, &stone);

        region1.merge(&region2);

        assert_eq!(region1.size, (4, 4, 4));
        assert_eq!(region1.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region1.get_block(2, 2, 2), Some(&stone));
    }

    #[test]
    fn test_region_merge_different_palettes() {
        let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (2, 2, 2));
        let mut region2 = Region::new("Test2".to_string(), (2, 2, 2), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region1.set_block(0, 0, 0, &stone);
        region2.set_block(2, 2, 2, &dirt);

        region1.merge(&region2);

        assert_eq!(region1.size, (4, 4, 4));
        assert_eq!(region1.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region1.get_block(2, 2, 2), Some(&dirt));
    }

    #[test]
    fn test_region_merge_different_overlapping_palettes() {
        let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (2, 2, 2));
        let mut region2 = Region::new("Test2".to_string(), (1, 1, 1), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region1.set_block(0, 0, 0, &stone);
        region1.set_block(1, 1, 1, &dirt);

        region2.set_block(2, 2, 2, &dirt);

        region1.merge(&region2);

        assert_eq!(region1.size, (3, 3, 3));
        assert_eq!(region1.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region1.get_block(1, 1, 1), Some(&dirt));
        assert_eq!(region1.get_block(2, 2, 2), Some(&dirt));
    }

    #[test]
    fn test_expand_to_fit_single_block() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        // Place a block at the farthest corner to trigger resizing
        region.set_block(3, 3, 3, &stone);

        assert_eq!(region.position, (0, 0, 0));
        assert_eq!(region.get_block(3, 3, 3), Some(&stone));
        assert_eq!(
            region.get_block(0, 0, 0),
            Some(&BlockState::new("minecraft:air".to_string()))
        );
    }

    #[test]
    fn test_expand_to_fit_negative_coordinates() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let dirt = BlockState::new("minecraft:dirt".to_string());

        // Place a block at a negative coordinate to trigger resizing
        region.set_block(-1, -1, -1, &dirt);

        // Geometric growth: an extent-2 axis grows by the 16-cell minimum step.
        assert_eq!(region.position, (-16, -16, -16));
        assert_eq!(region.get_block(-1, -1, -1), Some(&dirt));
        assert_eq!(
            region.get_block(0, 0, 0),
            Some(&BlockState::new("minecraft:air".to_string()))
        );
    }

    #[test]
    fn test_expand_to_fit_large_positive_coordinates() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        // Place a block far away to trigger significant resizing
        region.set_block(10, 10, 10, &stone);

        assert_eq!(region.position, (0, 0, 0));
        assert_eq!(region.get_block(10, 10, 10), Some(&stone));
    }

    #[test]
    fn test_expand_to_fit_corner_to_corner() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        // Place a block at one corner
        region.set_block(0, 0, 0, &stone);

        // Place another block far from the first to trigger resizing
        region.set_block(4, 4, 4, &dirt);

        assert_eq!(region.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region.get_block(4, 4, 4), Some(&dirt));
    }

    #[test]
    fn test_expand_to_fit_multiple_expansions() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        // Perform multiple expansions
        region.set_block(3, 3, 3, &stone);
        region.set_block(7, 7, 7, &stone);
        region.set_block(-2, -2, -2, &stone);

        // With hybrid approach, expect aggressive expansion
        // The exact value depends on your expansion logic, but should be around -66
        assert!(region.position.0 <= -2); // Position should shift to at least -2
        assert!(region.position.1 <= -2); // or more negative due to expansion margin
        assert!(region.position.2 <= -2);

        // These should all work regardless of expansion strategy
        assert_eq!(region.get_block(3, 3, 3), Some(&stone));
        assert_eq!(region.get_block(7, 7, 7), Some(&stone));
        assert_eq!(region.get_block(-2, -2, -2), Some(&stone));
    }

    #[test]
    fn test_expand_to_fit_with_existing_blocks() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 3, 3));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        // Place blocks in the initial region
        region.set_block(0, 0, 0, &stone);
        region.set_block(2, 2, 2, &dirt);

        // Trigger expansion
        region.set_block(5, 5, 5, &stone);

        assert_eq!(region.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region.get_block(2, 2, 2), Some(&dirt));
        assert_eq!(region.get_block(5, 5, 5), Some(&stone));
    }

    #[test]
    fn test_incremental_expansion_in_x() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        for x in 0..32 {
            region.set_block(x, 0, 0, &stone);
            assert_eq!(region.get_block(x, 0, 0), Some(&stone));
        }
    }

    #[test]
    fn test_incremental_expansion_in_y() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        for y in 0..32 {
            region.set_block(0, y, 0, &stone);
            assert_eq!(region.get_block(0, y, 0), Some(&stone));
        }
    }

    #[test]
    fn test_incremental_expansion_in_z() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        for z in 0..32 {
            region.set_block(0, 0, z, &stone);
            assert_eq!(region.get_block(0, 0, z), Some(&stone));
        }
    }

    #[test]
    fn test_incremental_expansion_in_x_y_z() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        for i in 0..32 {
            region.set_block(i, i, i, &stone);
            assert_eq!(region.get_block(i, i, i), Some(&stone));
        }
    }

    #[test]
    fn test_checkerboard_expansion() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    if (x + y + z) % 2 == 0 {
                        region.set_block(x, y, z, &stone);
                    } else {
                        region.set_block(x, y, z, &dirt);
                    }
                }
            }
        }

        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let expected = if (x + y + z) % 2 == 0 { &stone } else { &dirt };
                    assert_eq!(region.get_block(x, y, z), Some(expected));
                }
            }
        }
    }

    #[test]
    fn test_bounding_box() {
        let region = Region::new("Test".to_string(), (1, 0, 1), (-2, 2, -2));
        let bounding_box = region.get_bounding_box();

        assert_eq!(bounding_box.min, (0, 0, 0));
        assert_eq!(bounding_box.max, (1, 1, 1));

        let region = Region::new("Test".to_string(), (1, 0, 1), (-3, 3, -3));
        let bounding_box = region.get_bounding_box();

        assert_eq!(bounding_box.min, (-1, 0, -1));
        assert_eq!(bounding_box.max, (1, 2, 1));
    }

    #[test]
    fn test_coords_to_index() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));

        let volume1 = region.volume();
        for i in 0..8 {
            let coords = region.index_to_coords(i);
            let index = region.coords_to_index(coords.0, coords.1, coords.2);
            assert!(index < volume1);
            assert_eq!(index, i);
        }

        let region2 = Region::new("Test".to_string(), (0, 0, 0), (-2, -2, -2));

        let volume2 = region2.volume();
        for i in 0..8 {
            let coords = region2.index_to_coords(i);
            let index = region2.coords_to_index(coords.0, coords.1, coords.2);
            assert!(index < volume2);
            assert_eq!(index, i);
        }

        region.merge(&region2);

        let volume3 = region.volume();
        for i in 0..27 {
            let coords = region.index_to_coords(i);
            let index = region.coords_to_index(coords.0, coords.1, coords.2);
            assert!(index < volume3);
            assert_eq!(index, i);
        }
    }

    #[test]
    fn test_merge_negative_size() {
        let mut region1 = Region::new("Test1".to_string(), (0, 0, 0), (-2, -2, -2));
        let mut region2 = Region::new("Test2".to_string(), (-2, -2, -2), (-2, -2, -2));
        let stone = BlockState::new("minecraft:stone".to_string());

        region1.set_block(0, 0, 0, &stone);
        region2.set_block(-2, -2, -2, &stone);

        region1.merge(&region2);

        assert_eq!(region1.size, (4, 4, 4));
        assert_eq!(region1.get_bounding_box().min, (-3, -3, -3));
        assert_eq!(region1.get_bounding_box().max, (0, 0, 0));
        assert_eq!(region1.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region1.get_block(-2, -2, -2), Some(&stone));
    }

    #[test]
    fn test_expand_to_fit_preserve_blocks() {
        let mut region = Region::new("Test".to_string(), (1, 0, 1), (-2, 2, -2));
        let stone = BlockState::new("minecraft:stone".to_string());
        let diamond = BlockState::new("minecraft:diamond_block".to_string());

        // Set some initial blocks
        region.set_block(1, 0, 1, &stone);
        region.set_block(0, 1, 0, &stone);

        // Expand the region by setting a block outside the current bounds
        region.set_block(1, 2, 1, &diamond);

        // Check if the original blocks are preserved
        assert_eq!(region.get_block(1, 0, 1), Some(&stone));
        assert_eq!(region.get_block(0, 1, 0), Some(&stone));

        // Check if the new block is set correctly
        assert_eq!(region.get_block(1, 2, 1), Some(&diamond));
    }

    #[test]
    fn test_tight_bounds_empty_region() {
        let region = Region::new("Test".to_string(), (0, 0, 0), (10, 10, 10));

        // Empty region should have no tight bounds
        assert_eq!(region.get_tight_bounds(), None);
        assert_eq!(region.get_tight_dimensions(), (0, 0, 0));
    }

    #[test]
    fn test_tight_bounds_single_block() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (10, 10, 10));
        let stone = BlockState::new("minecraft:stone".to_string());

        region.set_block(5, 3, 7, &stone);

        let bounds = region.get_tight_bounds().unwrap();
        assert_eq!(bounds.min, (5, 3, 7));
        assert_eq!(bounds.max, (5, 3, 7));
        assert_eq!(region.get_tight_dimensions(), (1, 1, 1));
    }

    #[test]
    fn test_tight_bounds_multiple_blocks() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (10, 10, 10));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        // Place blocks at various positions
        region.set_block(2, 1, 3, &stone);
        region.set_block(5, 4, 8, &dirt);
        region.set_block(3, 2, 5, &stone);

        let bounds = region.get_tight_bounds().unwrap();
        assert_eq!(bounds.min, (2, 1, 3));
        assert_eq!(bounds.max, (5, 4, 8));
        assert_eq!(region.get_tight_dimensions(), (4, 4, 6));
    }

    #[test]
    fn test_tight_bounds_ignores_air() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (10, 10, 10));
        let stone = BlockState::new("minecraft:stone".to_string());
        let air = BlockState::new("minecraft:air".to_string());

        // Place stones
        region.set_block(1, 1, 1, &stone);
        region.set_block(3, 3, 3, &stone);

        // Place air blocks (should not affect tight bounds)
        region.set_block(0, 0, 0, &air);
        region.set_block(5, 5, 5, &air);

        let bounds = region.get_tight_bounds().unwrap();
        assert_eq!(bounds.min, (1, 1, 1));
        assert_eq!(bounds.max, (3, 3, 3));
        assert_eq!(region.get_tight_dimensions(), (3, 3, 3));
    }

    #[test]
    fn test_tight_bounds_shrink_when_blocks_are_cleared() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (10, 10, 10));
        let stone = BlockState::new("minecraft:stone".to_string());
        let air = BlockState::new("minecraft:air".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(2, 2, 2, &stone);
        assert_eq!(region.get_tight_dimensions(), (3, 3, 3));

        region.set_block(2, 2, 2, &air);
        let bounds = region.get_tight_bounds().unwrap();
        assert_eq!(bounds.min, (0, 0, 0));
        assert_eq!(bounds.max, (0, 0, 0));
        assert_eq!(region.get_tight_dimensions(), (1, 1, 1));

        region.set_block(0, 0, 0, &air);
        assert_eq!(region.count_non_air_blocks(), 0);
        assert_eq!(region.get_tight_bounds(), None);
        assert_eq!(region.get_tight_dimensions(), (0, 0, 0));
    }

    #[test]
    fn test_tight_bounds_shrink_for_unchecked_and_bulk_clears() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (5, 5, 5));
        let stone = BlockState::new("minecraft:stone".to_string());
        let stone_index = region.get_or_insert_palette_by_state(&stone);
        let air_index = region.cached_air_index as usize;

        region.set_block_at_index_unchecked(stone_index, 0, 0, 0);
        region.set_block_at_index_unchecked(stone_index, 2, 2, 2);
        region.set_block_at_index_unchecked(air_index, 2, 2, 2);
        assert_eq!(region.get_tight_dimensions(), (1, 1, 1));

        region.fill_uniform((0, 0, 0), (2, 2, 2), stone_index);
        assert_eq!(region.get_tight_dimensions(), (3, 3, 3));
        region.fill_uniform((2, 0, 0), (2, 2, 2), air_index);
        assert_eq!(region.get_tight_dimensions(), (2, 3, 3));
        region.fill_uniform((0, 0, 0), (1, 2, 2), air_index);
        assert_eq!(region.get_tight_bounds(), None);
        assert_eq!(region.get_tight_dimensions(), (0, 0, 0));
    }

    #[test]
    fn test_fill_uniform_whole_region_replaces_count_and_bounds() {
        let mut region = Region::new("Test".to_string(), (-2, 3, 5), (4, 3, 2));
        let stone = BlockState::new("minecraft:stone".to_string());
        let stone_index = region.get_or_insert_palette_by_state(&stone);
        let air_index = region.cached_air_index as usize;
        let bounds = region.get_bounding_box();

        region.fill_uniform(bounds.min, bounds.max, stone_index);
        assert_eq!(region.count_non_air_blocks(), 24);
        assert_eq!(region.get_tight_bounds(), Some(bounds.clone()));
        assert!(region.blocks.iter().all(|&index| index as usize == stone_index));

        region.fill_uniform(bounds.min, bounds.max, air_index);
        assert_eq!(region.count_non_air_blocks(), 0);
        assert_eq!(region.get_tight_bounds(), None);
        assert!(region.blocks.iter().all(|&index| index as usize == air_index));
    }

    #[test]
    fn test_tight_bounds_with_expansion() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (5, 5, 5));
        let stone = BlockState::new("minecraft:stone".to_string());

        // Place initial block
        region.set_block(2, 2, 2, &stone);

        let bounds1 = region.get_tight_bounds().unwrap();
        assert_eq!(bounds1.min, (2, 2, 2));
        assert_eq!(bounds1.max, (2, 2, 2));

        // Place block that requires region expansion
        region.set_block(10, 10, 10, &stone);

        let bounds2 = region.get_tight_bounds().unwrap();
        assert_eq!(bounds2.min, (2, 2, 2));
        assert_eq!(bounds2.max, (10, 10, 10));
        assert_eq!(region.get_tight_dimensions(), (9, 9, 9));
    }

    #[test]
    fn test_tight_bounds_vs_allocated_bounds() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        // Place a tiny 2x2x1 structure within initial bounds
        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 0, 0, &stone);
        region.set_block(0, 1, 0, &stone);
        region.set_block(1, 1, 0, &stone);

        // Force expansion by placing a block far away
        region.set_block(10, 10, 10, &stone);

        // Allocation grows past the request by the minimum step (16 cells),
        // so the next few writes in that direction do not reallocate.
        let allocated_dims = region.get_dimensions();
        assert_eq!(allocated_dims, (18, 18, 18));

        // Tight dimensions should reflect actual structure (min to max of placed blocks)
        let tight_dims = region.get_tight_dimensions();
        assert_eq!(tight_dims, (11, 11, 11)); // From (0,0,0) to (10,10,10) inclusive
    }

    #[test]
    fn test_rebuild_tight_bounds() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (10, 10, 10));
        let stone = BlockState::new("minecraft:stone".to_string());

        // Place some blocks
        region.set_block(1, 1, 1, &stone);
        region.set_block(5, 5, 5, &stone);

        let bounds_before = region.get_tight_bounds().unwrap();
        assert_eq!(bounds_before.min, (1, 1, 1));
        assert_eq!(bounds_before.max, (5, 5, 5));

        // Rebuild tight bounds (simulating deserialization)
        region.rebuild_tight_bounds();

        let bounds_after = region.get_tight_bounds().unwrap();
        assert_eq!(bounds_after.min, (1, 1, 1));
        assert_eq!(bounds_after.max, (5, 5, 5));
        assert_eq!(region.get_tight_dimensions(), (5, 5, 5));
    }

    // ══════════════════════════════════════════════════════════════════
    // Phase 1: non_air_count tracking tests
    // ══════════════════════════════════════════════════════════════════

    #[test]
    fn test_non_air_count_basic() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (4, 4, 4));
        assert_eq!(region.count_blocks(), 0);
        assert!(region.is_empty());

        let stone = BlockState::new("minecraft:stone".to_string());
        region.set_block(0, 0, 0, &stone);
        assert_eq!(region.count_blocks(), 1);
        assert!(!region.is_empty());

        region.set_block(1, 1, 1, &stone);
        assert_eq!(region.count_blocks(), 2);
    }

    #[test]
    fn test_non_air_count_overwrite_non_air_with_non_air() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (4, 4, 4));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region.set_block(0, 0, 0, &stone);
        assert_eq!(region.count_blocks(), 1);

        // Overwrite stone with dirt — count should stay 1
        region.set_block(0, 0, 0, &dirt);
        assert_eq!(region.count_blocks(), 1);
        assert_eq!(region.get_block(0, 0, 0), Some(&dirt));
    }

    #[test]
    fn test_non_air_count_overwrite_non_air_with_air() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (4, 4, 4));
        let stone = BlockState::new("minecraft:stone".to_string());
        let air = BlockState::new("minecraft:air".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 0, 0, &stone);
        assert_eq!(region.count_blocks(), 2);

        // Replace stone with air — count should decrement
        region.set_block(0, 0, 0, &air);
        assert_eq!(region.count_blocks(), 1);
        assert!(!region.is_empty());

        region.set_block(1, 0, 0, &air);
        assert_eq!(region.count_blocks(), 0);
        assert!(region.is_empty());
    }

    #[test]
    fn test_non_air_count_set_air_on_air() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (4, 4, 4));
        let air = BlockState::new("minecraft:air".to_string());

        // Setting air on already-air should not underflow
        region.set_block(0, 0, 0, &air);
        assert_eq!(region.count_blocks(), 0);
        assert!(region.is_empty());
    }

    #[test]
    fn test_non_air_count_survives_expansion() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 1, 1, &stone);
        assert_eq!(region.count_blocks(), 2);

        // Trigger expansion
        region.set_block(10, 10, 10, &stone);
        assert_eq!(region.count_blocks(), 3);

        // Original blocks should still be there
        assert_eq!(region.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region.get_block(1, 1, 1), Some(&stone));
    }

    #[test]
    fn test_non_air_count_after_merge() {
        let mut r1 = Region::new("R1".to_string(), (0, 0, 0), (4, 4, 4));
        let mut r2 = Region::new("R2".to_string(), (4, 0, 0), (4, 4, 4));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        r1.set_block(0, 0, 0, &stone);
        r1.set_block(1, 0, 0, &stone);
        r2.set_block(4, 0, 0, &dirt);
        r2.set_block(5, 0, 0, &dirt);
        r2.set_block(6, 0, 0, &dirt);

        r1.merge(&r2);
        assert_eq!(r1.count_blocks(), 5);
    }

    #[test]
    fn test_non_air_count_after_flip() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (4, 4, 4));
        let stone = BlockState::new("minecraft:stone".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 1, 1, &stone);
        region.set_block(2, 2, 2, &stone);
        assert_eq!(region.count_blocks(), 3);

        region.flip_x();
        assert_eq!(region.count_blocks(), 3);

        region.flip_y();
        assert_eq!(region.count_blocks(), 3);

        region.flip_z();
        assert_eq!(region.count_blocks(), 3);
    }

    #[test]
    fn test_non_air_count_after_rotate() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (4, 4, 4));
        let stone = BlockState::new("minecraft:stone".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 1, 1, &stone);
        assert_eq!(region.count_blocks(), 2);

        region.rotate_y(90);
        assert_eq!(region.count_blocks(), 2);

        region.rotate_x(90);
        assert_eq!(region.count_blocks(), 2);

        region.rotate_z(90);
        assert_eq!(region.count_blocks(), 2);
    }

    #[test]
    fn test_non_air_count_after_to_compact() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (10, 10, 10));
        let stone = BlockState::new("minecraft:stone".to_string());

        region.set_block(2, 2, 2, &stone);
        region.set_block(5, 5, 5, &stone);
        region.set_block(3, 3, 3, &stone);

        let compact = region.to_compact();
        assert_eq!(compact.count_blocks(), 3);
        assert!(!compact.is_empty());
    }

    // ══════════════════════════════════════════════════════════════════
    // Phase 2: expand_to_bounding_box row copy correctness
    // ══════════════════════════════════════════════════════════════════

    #[test]
    fn test_expand_preserves_all_blocks_3d() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 3, 3));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());
        let glass = BlockState::new("minecraft:glass".to_string());

        // Fill with a unique pattern per position
        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 0, 0, &dirt);
        region.set_block(2, 0, 0, &glass);
        region.set_block(0, 1, 1, &dirt);
        region.set_block(1, 2, 2, &glass);
        region.set_block(2, 2, 0, &stone);

        // Expand in negative direction
        region.set_block(-5, -5, -5, &stone);

        // Verify all original blocks survived the expansion
        assert_eq!(region.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region.get_block(1, 0, 0), Some(&dirt));
        assert_eq!(region.get_block(2, 0, 0), Some(&glass));
        assert_eq!(region.get_block(0, 1, 1), Some(&dirt));
        assert_eq!(region.get_block(1, 2, 2), Some(&glass));
        assert_eq!(region.get_block(2, 2, 0), Some(&stone));
        assert_eq!(region.get_block(-5, -5, -5), Some(&stone));
    }

    #[test]
    fn test_expand_with_negative_origin() {
        let mut region = Region::new("Test".to_string(), (-2, -2, -2), (4, 4, 4));
        let stone = BlockState::new("minecraft:stone".to_string());

        region.set_block(-2, -2, -2, &stone);
        region.set_block(1, 1, 1, &stone);

        // Expand further negative
        region.set_block(-10, -10, -10, &stone);

        assert_eq!(region.get_block(-2, -2, -2), Some(&stone));
        assert_eq!(region.get_block(1, 1, 1), Some(&stone));
        assert_eq!(region.get_block(-10, -10, -10), Some(&stone));
    }

    // ══════════════════════════════════════════════════════════════════
    // Phase 3: Flip correctness — double flip must restore original
    // ══════════════════════════════════════════════════════════════════

    #[test]
    fn test_flip_x_double_restores_original() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (5, 3, 4));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(4, 2, 3, &dirt);
        region.set_block(2, 1, 2, &stone);

        let original_blocks = region.blocks.clone();

        region.flip_x();
        region.flip_x();

        assert_eq!(region.blocks, original_blocks);
    }

    #[test]
    fn test_flip_y_double_restores_original() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (5, 3, 4));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(4, 2, 3, &dirt);
        region.set_block(2, 1, 2, &stone);

        let original_blocks = region.blocks.clone();

        region.flip_y();
        region.flip_y();

        assert_eq!(region.blocks, original_blocks);
    }

    #[test]
    fn test_flip_z_double_restores_original() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (5, 3, 4));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(4, 2, 3, &dirt);
        region.set_block(2, 1, 2, &stone);

        let original_blocks = region.blocks.clone();

        region.flip_z();
        region.flip_z();

        assert_eq!(region.blocks, original_blocks);
    }

    #[test]
    fn test_flip_x_odd_width() {
        // Odd width: middle column should stay in place
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 1, 1));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());
        let glass = BlockState::new("minecraft:glass".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 0, 0, &dirt);
        region.set_block(2, 0, 0, &glass);

        region.flip_x();

        // After flip_x: block at x=0 should now be at x=2 and vice versa
        // But palette is also transformed, so check via get_block
        // The block content at position 0 should now be what was at position 2
        let b0 = region.get_block(0, 0, 0).unwrap();
        let b2 = region.get_block(2, 0, 0).unwrap();
        // glass was at x=2, should now be at x=0
        assert!(b0.name.contains("glass"));
        // stone was at x=0, should now be at x=2
        assert!(b2.name.contains("stone"));
    }

    #[test]
    fn test_flip_y_odd_height() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 3, 1));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(0, 2, 0, &dirt);

        region.flip_y();

        let b0 = region.get_block(0, 0, 0).unwrap();
        let b2 = region.get_block(0, 2, 0).unwrap();
        assert!(b0.name.contains("dirt"));
        assert!(b2.name.contains("stone"));
    }

    #[test]
    fn test_flip_z_odd_length() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 1, 3));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(0, 0, 2, &dirt);

        region.flip_z();

        let b0 = region.get_block(0, 0, 0).unwrap();
        let b2 = region.get_block(0, 0, 2).unwrap();
        assert!(b0.name.contains("dirt"));
        assert!(b2.name.contains("stone"));
    }

    #[test]
    fn test_flip_dimension_1() {
        // Flipping along an axis where that dimension is 1 should be a no-op for blocks
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 4, 4));
        let stone = BlockState::new("minecraft:stone".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(0, 3, 3, &stone);

        let original_blocks = region.blocks.clone();
        region.flip_x();
        assert_eq!(region.blocks, original_blocks);
    }

    // ══════════════════════════════════════════════════════════════════
    // Phase 4: to_compact row copy correctness
    // ══════════════════════════════════════════════════════════════════

    #[test]
    fn test_to_compact_preserves_all_blocks() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (20, 20, 20));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());
        let glass = BlockState::new("minecraft:glass".to_string());

        // Place blocks at various positions
        region.set_block(5, 5, 5, &stone);
        region.set_block(10, 10, 10, &dirt);
        region.set_block(7, 8, 9, &glass);
        region.set_block(5, 10, 5, &stone);
        region.set_block(10, 5, 10, &dirt);

        let compact = region.to_compact();

        // All blocks should be retrievable at the same world coordinates
        assert_eq!(compact.get_block(5, 5, 5), Some(&stone));
        assert_eq!(compact.get_block(10, 10, 10), Some(&dirt));
        assert_eq!(compact.get_block(7, 8, 9), Some(&glass));
        assert_eq!(compact.get_block(5, 10, 5), Some(&stone));
        assert_eq!(compact.get_block(10, 5, 10), Some(&dirt));
        assert_eq!(compact.count_blocks(), 5);
    }

    #[test]
    fn test_to_compact_single_block() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (100, 100, 100));
        let stone = BlockState::new("minecraft:stone".to_string());

        region.set_block(50, 50, 50, &stone);

        let compact = region.to_compact();
        assert_eq!(compact.size, (1, 1, 1));
        assert_eq!(compact.get_block(50, 50, 50), Some(&stone));
        assert_eq!(compact.count_blocks(), 1);
    }

    #[test]
    fn test_to_compact_empty_region() {
        let region = Region::new("Test".to_string(), (0, 0, 0), (10, 10, 10));

        let compact = region.to_compact();
        assert_eq!(compact.count_blocks(), 0);
        assert!(compact.is_empty());
    }

    #[test]
    fn test_to_compact_air_not_counted() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (10, 10, 10));
        let stone = BlockState::new("minecraft:stone".to_string());

        // Place blocks in a line
        for x in 3..7 {
            region.set_block(x, 5, 5, &stone);
        }

        let compact = region.to_compact();
        assert_eq!(compact.count_blocks(), 4);

        // Air blocks between stone and edges should exist in compact but not be counted
        let air_pos = compact.get_block(3, 5, 5);
        assert_eq!(air_pos, Some(&stone));
    }

    // ══════════════════════════════════════════════════════════════════
    // Phase 5: Merge palette remap correctness
    // ══════════════════════════════════════════════════════════════════

    #[test]
    fn test_merge_palette_remap_correctness() {
        let mut r1 = Region::new("R1".to_string(), (0, 0, 0), (4, 4, 4));
        let mut r2 = Region::new("R2".to_string(), (4, 0, 0), (4, 4, 4));

        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());
        let glass = BlockState::new("minecraft:glass".to_string());
        let oak = BlockState::new("minecraft:oak_planks".to_string());

        // r1 has stone, dirt
        r1.set_block(0, 0, 0, &stone);
        r1.set_block(1, 0, 0, &dirt);

        // r2 has glass, oak (different palette)
        r2.set_block(4, 0, 0, &glass);
        r2.set_block(5, 0, 0, &oak);

        r1.merge(&r2);

        // All blocks should be correct after palette remapping
        assert_eq!(r1.get_block(0, 0, 0), Some(&stone));
        assert_eq!(r1.get_block(1, 0, 0), Some(&dirt));
        assert_eq!(r1.get_block(4, 0, 0), Some(&glass));
        assert_eq!(r1.get_block(5, 0, 0), Some(&oak));
    }

    #[test]
    fn test_merge_air_from_other_does_not_overwrite() {
        let mut r1 = Region::new("R1".to_string(), (0, 0, 0), (4, 4, 4));
        let r2 = Region::new("R2".to_string(), (0, 0, 0), (4, 4, 4));

        let stone = BlockState::new("minecraft:stone".to_string());

        // r1 has a stone block; r2 has air at the same position
        r1.set_block(0, 0, 0, &stone);

        r1.merge(&r2);

        // Air from r2 should NOT overwrite stone from r1
        assert_eq!(r1.get_block(0, 0, 0), Some(&stone));
    }

    #[test]
    fn test_merge_overlapping_non_air_overwrites() {
        let mut r1 = Region::new("R1".to_string(), (0, 0, 0), (4, 4, 4));
        let mut r2 = Region::new("R2".to_string(), (0, 0, 0), (4, 4, 4));

        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        r1.set_block(0, 0, 0, &stone);
        r2.set_block(0, 0, 0, &dirt);

        r1.merge(&r2);

        // Dirt from r2 should overwrite stone from r1
        assert_eq!(r1.get_block(0, 0, 0), Some(&dirt));
    }

    // ══════════════════════════════════════════════════════════════════
    // Cross-phase: cached fields consistency after complex operations
    // ══════════════════════════════════════════════════════════════════

    #[test]
    fn test_cached_dims_after_rotation() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 5, 7));
        let stone = BlockState::new("minecraft:stone".to_string());
        region.set_block(0, 0, 0, &stone);

        // After rotate_y(90): (3,5,7) -> (7,5,3)
        region.rotate_y(90);
        let dims = region.get_dimensions();
        assert_eq!(dims, (7, 5, 3));

        // coords_to_index should still work correctly
        let vol = region.volume();
        for i in 0..vol {
            let (x, y, z) = region.index_to_coords(i);
            let roundtrip = region.coords_to_index(x, y, z);
            assert_eq!(i, roundtrip, "roundtrip failed for index {}", i);
        }
    }

    #[test]
    fn test_index_roundtrip_after_expand() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (3, 3, 3));
        let stone = BlockState::new("minecraft:stone".to_string());
        region.set_block(0, 0, 0, &stone);

        // Trigger expansion
        region.set_block(20, 20, 20, &stone);

        let vol = region.volume();
        for i in 0..vol {
            let (x, y, z) = region.index_to_coords(i);
            let roundtrip = region.coords_to_index(x, y, z);
            assert_eq!(
                i, roundtrip,
                "roundtrip failed for index {} -> ({},{},{}) -> {}",
                i, x, y, z, roundtrip
            );
        }
    }

    #[test]
    fn test_nbt_roundtrip_preserves_count() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (4, 4, 4));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 1, 1, &dirt);
        region.set_block(2, 2, 2, &stone);

        let nbt = region.to_nbt();
        let deserialized = match nbt {
            NbtTag::Compound(compound) => Region::from_nbt(&compound).unwrap(),
            _ => panic!("Expected NbtTag::Compound"),
        };

        assert_eq!(deserialized.count_blocks(), 3);
        assert!(!deserialized.is_empty());
        assert_eq!(deserialized.get_block(0, 0, 0), Some(&stone));
        assert_eq!(deserialized.get_block(1, 1, 1), Some(&dirt));
        assert_eq!(deserialized.get_block(2, 2, 2), Some(&stone));
    }

    #[test]
    fn test_full_solid_fill_then_clear() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (4, 4, 4));
        let stone = BlockState::new("minecraft:stone".to_string());
        let air = BlockState::new("minecraft:air".to_string());

        // Fill entirely with stone
        for y in 0..4 {
            for z in 0..4 {
                for x in 0..4 {
                    region.set_block(x, y, z, &stone);
                }
            }
        }
        assert_eq!(region.count_blocks(), 64);

        // Clear every block back to air
        for y in 0..4 {
            for z in 0..4 {
                for x in 0..4 {
                    region.set_block(x, y, z, &air);
                }
            }
        }
        assert_eq!(region.count_blocks(), 0);
        assert!(region.is_empty());
    }
}
