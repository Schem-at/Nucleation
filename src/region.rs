use crate::block_entity::BlockEntity;
use crate::block_position::BlockPosition;
use crate::bounding_box::BoundingBox;
use crate::entity::Entity;
use crate::BlockState;
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub name: String,
    pub position: (i32, i32, i32),
    pub size: (i32, i32, i32),
    pub blocks: Vec<usize>,
    pub(crate) palette: Vec<BlockState>,
    pub entities: Vec<Entity>,
    #[serde(
        serialize_with = "serialize_block_entities",
        deserialize_with = "deserialize_block_entities"
    )]
    pub block_entities: HashMap<(i32, i32, i32), BlockEntity>,
    #[serde(skip, default = "HashMap::new")]
    palette_index: HashMap<BlockState, usize>,

    #[serde(skip)]
    bbox: BoundingBox,

    /// Tracks the actual min/max coordinates of non-air blocks placed in this region
    /// Unlike bbox (which is pre-allocated for performance), tight_bounds only covers
    /// the space that actually contains blocks
    #[serde(skip)]
    tight_bounds: Option<BoundingBox>,

    // ── Cached fields for hot-path performance ──
    #[serde(skip)]
    cached_width: i32,
    #[serde(skip)]
    cached_length: i32,
    #[serde(skip)]
    cached_width_x_length: i32,
    #[serde(skip)]
    cached_air_index: usize,
    #[serde(skip)]
    non_air_count: usize,
}

fn serialize_block_entities<S>(
    block_entities: &HashMap<(i32, i32, i32), BlockEntity>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let block_entities_vec: Vec<&BlockEntity> = block_entities.values().collect();
    block_entities_vec.serialize(serializer)
}

fn deserialize_block_entities<'de, D>(
    deserializer: D,
) -> Result<HashMap<(i32, i32, i32), BlockEntity>, D::Error>
where
    D: Deserializer<'de>,
{
    let block_entities_vec: Vec<BlockEntity> = Vec::deserialize(deserializer)?;
    Ok(block_entities_vec
        .into_iter()
        .map(|be| {
            let pos = (
                be.position.0 as i32,
                be.position.1 as i32,
                be.position.2 as i32,
            );
            (pos, be)
        })
        .collect())
}

impl Region {
    pub fn new(name: String, position: (i32, i32, i32), size: (i32, i32, i32)) -> Self {
        let bounding_box = BoundingBox::from_position_and_size(position, size);
        let volume = bounding_box.volume() as usize;
        let position_and_size = bounding_box.to_position_and_size();

        let mut palette = Vec::new();
        let mut palette_index = HashMap::new();

        let air = BlockState::new("minecraft:air".to_string());
        palette.push(air.clone());
        palette_index.insert(air, 0);

        let mut region = Region {
            name,
            position: position_and_size.0,
            size: position_and_size.1,
            blocks: vec![0; volume],
            palette,
            palette_index,
            entities: Vec::new(),
            block_entities: HashMap::new(),
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
        region
    }

    #[inline(always)]
    pub fn rebuild_bbox(&mut self) {
        self.bbox = BoundingBox::from_position_and_size(self.position, self.size);
        let (w, _, l) = self.bbox.get_dimensions();
        self.cached_width = w;
        self.cached_length = l;
        self.cached_width_x_length = w * l;
    }

    /// Recompute cached_air_index from the palette.
    pub(crate) fn rebuild_air_index(&mut self) {
        self.cached_air_index = self
            .palette
            .iter()
            .position(|b| b.name == "minecraft:air")
            .unwrap_or(usize::MAX);
    }

    /// Recount non-air blocks by scanning the blocks array.
    pub(crate) fn rebuild_non_air_count(&mut self) {
        let air = self.cached_air_index;
        self.non_air_count = self.blocks.iter().filter(|&&b| b != air).count();
    }

    /// Rebuild tight bounds by scanning all blocks
    /// This is typically called after deserialization
    pub fn rebuild_tight_bounds(&mut self) {
        self.tight_bounds = None;
        let air = self.cached_air_index;

        for index in 0..self.blocks.len() {
            if self.blocks[index] != air {
                let (x, y, z) = self.index_to_coords(index);
                self.update_tight_bounds(x, y, z);
            }
        }
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

    pub fn has_non_air_blocks(&self) -> bool {
        self.non_air_count > 0
    }

    pub fn count_non_air_blocks(&self) -> usize {
        self.non_air_count
    }

    pub(crate) fn rebuild_palette_index(&mut self) {
        self.palette_index = HashMap::with_capacity(self.palette.len());
        for (index, block) in self.palette.iter().enumerate() {
            self.palette_index.insert(block.clone(), index);
        }
    }

    pub fn get_block_entities_as_list(&self) -> Vec<BlockEntity> {
        self.block_entities.values().cloned().collect()
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: &BlockState) -> bool {
        if !self.is_in_region(x, y, z) {
            self.expand_to_fit(x, y, z);
        }

        let index = self.coords_to_index(x, y, z);
        let palette_index = self.get_or_insert_in_palette(block);
        let old_palette_index = self.blocks[index];
        self.blocks[index] = palette_index;

        // Update non_air_count based on old vs new block
        let old_is_air = old_palette_index == self.cached_air_index;
        let new_is_air = palette_index == self.cached_air_index;
        if old_is_air && !new_is_air {
            self.non_air_count += 1;
        } else if !old_is_air && new_is_air {
            self.non_air_count -= 1;
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
        let dx = x - self.bbox.min.0;
        let dy = y - self.bbox.min.1;
        let dz = z - self.bbox.min.2;
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

    /// Create a compacted region containing only the non-air blocks within tight bounds.
    /// This is ideal for exporting schematics as it removes all allocated but empty space.
    /// Block entities and entities are preserved at their correct positions.
    /// Returns a clone if no tight bounds exist (empty region).
    pub fn to_compact(&self) -> Region {
        // If no tight bounds, return a minimal clone
        let Some(tight_bounds) = &self.tight_bounds else {
            let mut empty = Region::new(self.name.clone(), (0, 0, 0), (1, 1, 1));
            empty.palette = self.palette.clone();
            empty.rebuild_palette_index();
            empty.rebuild_air_index();
            return empty;
        };

        let tight_dims = tight_bounds.get_dimensions();
        let tight_pos = tight_bounds.min;

        // Create new region with exact tight bounds
        let mut compact = Region::new(self.name.clone(), tight_pos, tight_dims);
        compact.palette = self.palette.clone();
        compact.rebuild_palette_index();
        compact.rebuild_air_index();

        // Phase 4: Row-level copy instead of per-block set_block
        let old_w = self.cached_width as usize;
        let old_l = self.cached_length as usize;
        let (compact_w, _, compact_l) = compact.bbox.get_dimensions();
        let compact_w = compact_w as usize;
        let compact_l = compact_l as usize;

        // Tight bounds relative to source region
        let tb_min_x = tight_bounds.min.0;
        let tb_min_y = tight_bounds.min.1;
        let tb_min_z = tight_bounds.min.2;
        let tb_max_y = tight_bounds.max.1;
        let tb_max_z = tight_bounds.max.2;

        let x_off_src = (tb_min_x - self.bbox.min.0) as usize;
        let row_len = compact_w; // tight width = compact width

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
            // The compact array is structured such that each chunk of `compact_w * compact_l` elements
            // is exactly one slice parallel to XZ plane (a Y slice).
            // We can par_chunks_mut over the compact blocks by the Y slice size.
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

                    // Within this Y slice, iterate serially over Z rows
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

        // Rebuild non_air_count and tight_bounds for compact region
        compact.rebuild_non_air_count();
        compact.tight_bounds = Some(tight_bounds.clone());

        // Copy block entities within tight bounds
        for (&pos, be) in &self.block_entities {
            if tight_bounds.contains(pos) {
                compact.block_entities.insert(pos, be.clone());
            }
        }

        // Copy entities within tight bounds
        for entity in &self.entities {
            let entity_pos = (
                entity.position.0.floor() as i32,
                entity.position.1.floor() as i32,
                entity.position.2.floor() as i32,
            );
            if tight_bounds.contains(entity_pos) {
                compact.entities.push(entity.clone());
            }
        }

        compact
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<&BlockState> {
        if !self.is_in_region(x, y, z) {
            return None;
        }

        let index = self.coords_to_index(x, y, z);
        let block_index = self.blocks[index];
        let palette_index = self.palette.get(block_index);
        palette_index
    }

    pub fn get_block_index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if !self.is_in_region(x, y, z) {
            return None;
        }

        let index = self.coords_to_index(x, y, z);
        let block_index = self.blocks[index];
        Some(block_index)
    }

    pub fn volume(&self) -> usize {
        self.size.0 as usize * self.size.1 as usize * self.size.2 as usize
    }

    pub fn expand_to_fit(&mut self, x: i32, y: i32, z: i32) {
        let current_bounding_box = self.get_bounding_box();

        if current_bounding_box.contains((x, y, z)) {
            return;
        }

        let current_volume = current_bounding_box.volume();
        let current_size = current_bounding_box.get_dimensions();

        // Choose expansion strategy based on current size
        let expansion_size = if current_volume < 1000 {
            // Small regions: fixed large expansion
            (64, 64, 64)
        } else if current_volume < 100_000 {
            // Medium regions: proportional expansion
            (current_size.0 / 2, current_size.1 / 2, current_size.2 / 2)
        } else {
            // Large regions: chunk-based expansion
            (128, 128, 128)
        };

        let new_min = (
            if x < current_bounding_box.min.0 {
                x - expansion_size.0
            } else {
                current_bounding_box.min.0
            },
            if y < current_bounding_box.min.1 {
                y - expansion_size.1
            } else {
                current_bounding_box.min.1
            },
            if z < current_bounding_box.min.2 {
                z - expansion_size.2
            } else {
                current_bounding_box.min.2
            },
        );

        let new_max = (
            if x > current_bounding_box.max.0 {
                x + expansion_size.0
            } else {
                current_bounding_box.max.0
            },
            if y > current_bounding_box.max.1 {
                y + expansion_size.1
            } else {
                current_bounding_box.max.1
            },
            if z > current_bounding_box.max.2 {
                z + expansion_size.2
            } else {
                current_bounding_box.max.2
            },
        );

        let new_bounding_box = BoundingBox::new(new_min, new_max);
        self.expand_to_bounding_box(new_bounding_box);
    }

    /// Ensure the region covers the given bounds, expanding if necessary.
    /// This avoids multiple reallocations when filling a known shape.
    pub fn ensure_bounds(&mut self, min: (i32, i32, i32), max: (i32, i32, i32)) {
        let current_bbox = self.get_bounding_box();
        let mut new_min = current_bbox.min;
        let mut new_max = current_bbox.max;
        let mut expanded = false;

        if min.0 < new_min.0 {
            new_min.0 = min.0;
            expanded = true;
        }
        if min.1 < new_min.1 {
            new_min.1 = min.1;
            expanded = true;
        }
        if min.2 < new_min.2 {
            new_min.2 = min.2;
            expanded = true;
        }

        if max.0 > new_max.0 {
            new_max.0 = max.0;
            expanded = true;
        }
        if max.1 > new_max.1 {
            new_max.1 = max.1;
            expanded = true;
        }
        if max.2 > new_max.2 {
            new_max.2 = max.2;
            expanded = true;
        }

        if expanded {
            let new_bbox = BoundingBox::new(new_min, new_max);
            self.expand_to_bounding_box(new_bbox);
        }
    }

    fn expand_to_bounding_box(&mut self, new_bounding_box: BoundingBox) {
        let new_size = new_bounding_box.get_dimensions();
        let new_position = new_bounding_box.min;

        if new_size == self.size && new_position == self.position {
            return;
        }

        let air_id = self.cached_air_index;
        let mut new_blocks = vec![air_id; new_bounding_box.volume() as usize];

        // Phase 2: Row-level copy instead of per-element coords
        let old_w = self.cached_width as usize;
        let old_l = self.cached_length as usize;
        let (_, old_h, _) = self.bbox.get_dimensions();
        let old_h = old_h as usize;

        let (new_w, _, new_l) = new_bounding_box.get_dimensions();
        let new_w = new_w as usize;
        let new_l = new_l as usize;

        // Offset of old min within new coordinate system
        let x_off = (self.bbox.min.0 - new_bounding_box.min.0) as usize;
        let y_off = (self.bbox.min.1 - new_bounding_box.min.1) as usize;
        let z_off = (self.bbox.min.2 - new_bounding_box.min.2) as usize;

        for dy in 0..old_h {
            for dz in 0..old_l {
                let src_start = dy * old_w * old_l + dz * old_w;
                let dst_start = (dy + y_off) * new_w * new_l + (dz + z_off) * new_w + x_off;
                new_blocks[dst_start..dst_start + old_w]
                    .copy_from_slice(&self.blocks[src_start..src_start + old_w]);
            }
        }

        self.position = new_position;
        self.size = new_size;
        self.blocks = new_blocks;
        self.rebuild_bbox();
    }
    fn calculate_bits_per_block(&self) -> usize {
        let palette_size = self.palette.len();
        let bits_per_block = std::cmp::max((palette_size as f64).log2().ceil() as usize, 2);
        bits_per_block
    }

    pub fn merge(&mut self, other: &Region) {
        let combined_bounding_box = self.get_bounding_box().union(&other.get_bounding_box());
        let new_size = combined_bounding_box.get_dimensions();
        let new_position = combined_bounding_box.min;

        let (new_w, _, new_l) = new_size;
        let new_w = new_w as usize;
        let new_l = new_l as usize;

        let mut new_blocks = vec![0usize; combined_bounding_box.volume() as usize];
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
        let mut other_remap: Vec<usize> = Vec::with_capacity(other.palette.len());
        for (idx, block) in other.palette.iter().enumerate() {
            if let Some(&existing) = reverse_new_palette.get(block) {
                other_remap.push(existing);
            } else {
                let new_idx = new_palette.len();
                new_palette.push(block.clone());
                reverse_new_palette.insert(block.clone(), new_idx);
                other_remap.push(new_idx);
            }
            // Mark air indices in remap for skip check
            let _ = idx;
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
                        new_blocks[dst_start + dx] = other_remap[src_idx];
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
        self.rebuild_non_air_count();

        // Merge entities and block entities
        self.merge_entities(other);
        self.merge_block_entities(other);
    }

    fn merge_entities(&mut self, other: &Region) {
        self.entities.extend(other.entities.iter().cloned());
    }

    fn merge_block_entities(&mut self, other: &Region) {
        self.block_entities.extend(
            other
                .block_entities
                .iter()
                .map(|(&pos, be)| (pos, be.clone())),
        );
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
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
                &format!("{},{},{}", x, y, z),
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
        for ((x, y, z), block_entity) in &self.block_entities {
            block_entities_tag.insert(&format!("{},{},{}", x, y, z), block_entity.to_nbt());
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
        let mut blocks = vec![0; (size.0 * size.1 * size.2) as usize];
        for (key, value) in blocks_tag.inner() {
            if let NbtTag::Int(index) = value {
                let coords: Vec<i32> = key.split(',').map(|s| s.parse::<i32>().unwrap()).collect();
                if coords.len() == 3 {
                    let block_index =
                        (coords[1] * size.0 * size.2 + coords[2] * size.0 + coords[0]) as usize;
                    blocks[block_index] = *index as usize;
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
        let mut block_entities = HashMap::new();
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
            palette_index: HashMap::new(),
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
        region.rebuild_non_air_count();
        region.rebuild_tight_bounds();

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

    pub fn unpack_block_states(&self, packed_states: &[i64]) -> Vec<usize> {
        let bits_per_block = self.calculate_bits_per_block();
        let mask = (1 << bits_per_block) - 1;
        let volume = self.volume();

        let mut blocks = Vec::with_capacity(volume);

        for index in 0..volume {
            let bit_index = index * bits_per_block;
            let start_long_index = bit_index / 64;
            let start_offset = bit_index % 64;

            let value = if start_offset + bits_per_block <= 64 {
                // Block is entirely within one long
                ((packed_states[start_long_index] as u64) >> start_offset) & (mask as u64)
            } else {
                // Block spans two longs
                let low_bits = ((packed_states[start_long_index] as u64) >> start_offset)
                    & ((1 << (64 - start_offset)) - 1);
                let high_bits = (packed_states[start_long_index + 1] as u64)
                    & ((1 << (bits_per_block - (64 - start_offset))) - 1);
                low_bits | (high_bits << (64 - start_offset))
            };

            blocks.push(value as usize);
        }

        blocks
    }

    pub(crate) fn create_packed_block_states(&self) -> Vec<i64> {
        let bits_per_block = self.calculate_bits_per_block();
        let size = self.blocks.len();
        let expected_len = (size * bits_per_block + 63) / 64; // Equivalent to ceil(size * bits_per_block / 64)

        let mut packed_states = vec![0i64; expected_len];
        let mask = (1i64 << bits_per_block) - 1;

        for (index, &block_state) in self.blocks.iter().enumerate() {
            let bit_index = index * bits_per_block;
            let start_long_index = bit_index / 64;
            let end_long_index = (bit_index + bits_per_block - 1) / 64;
            let start_offset = bit_index % 64;

            let value = (block_state as i64) & mask;

            if start_long_index == end_long_index {
                packed_states[start_long_index] |= value << start_offset;
            } else {
                packed_states[start_long_index] |= value << start_offset;
                packed_states[end_long_index] |= value >> (64 - start_offset);
            }
        }

        // Handle negative numbers
        packed_states.iter_mut().for_each(|x| *x = *x as u64 as i64);

        packed_states
    }

    pub fn get_palette(&self) -> Vec<BlockState> {
        self.palette.clone()
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
            let block_state = &self.palette[*block_index];
            *block_counts.entry(block_state.clone()).or_insert(0) += 1;
        }
        block_counts
    }

    pub fn count_blocks(&self) -> usize {
        self.non_air_count
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
        let mut new_blocks = self.blocks.clone();

        for dy in 0..h {
            for dz in 0..l {
                let start = dy * wl + dz * w;
                new_blocks[start..start + w].reverse();
            }
        }

        // Transform block state properties
        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_flip(block_state, Axis::X));
        }

        self.blocks = new_blocks;
        self.palette = new_palette;
        self.rebuild_palette_index();
        self.rebuild_air_index();
        self.rebuild_non_air_count();

        // Transform block entities
        let mut new_block_entities = HashMap::new();
        for ((x, y, z), mut be) in self.block_entities.drain() {
            let new_x = self.bbox.max.0 - (x - self.bbox.min.0);
            be.position = (new_x, y, z);
            new_block_entities.insert((new_x, y, z), be);
        }
        self.block_entities = new_block_entities;

        // Transform entities
        for entity in &mut self.entities {
            let rel_x = entity.position.0 - self.bbox.min.0 as f64;
            entity.position.0 = self.bbox.max.0 as f64 - rel_x;
        }
    }

    /// Flip the region along the Y axis
    pub fn flip_y(&mut self) {
        use crate::transforms::{transform_block_state_flip, Axis};

        // Phase 3: Swap Y-layers using swap_nonoverlapping
        let wl = self.cached_width_x_length as usize;
        let (_, h, _) = self.bbox.get_dimensions();
        let h = h as usize;
        let mut new_blocks = self.blocks.clone();

        // Swap layer dy with layer (h-1-dy)
        let ptr = new_blocks.as_mut_ptr();
        for dy in 0..h / 2 {
            let mirror_dy = h - 1 - dy;
            let off_a = dy * wl;
            let off_b = mirror_dy * wl;
            // Safety: non-overlapping because dy < h/2 < mirror_dy
            unsafe {
                std::ptr::swap_nonoverlapping(ptr.add(off_a), ptr.add(off_b), wl);
            }
        }

        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_flip(block_state, Axis::Y));
        }

        self.blocks = new_blocks;
        self.palette = new_palette;
        self.rebuild_palette_index();
        self.rebuild_air_index();
        self.rebuild_non_air_count();

        let mut new_block_entities = HashMap::new();
        for ((x, y, z), mut be) in self.block_entities.drain() {
            let new_y = self.bbox.max.1 - (y - self.bbox.min.1);
            be.position = (x, new_y, z);
            new_block_entities.insert((x, new_y, z), be);
        }
        self.block_entities = new_block_entities;

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
        let mut new_blocks = self.blocks.clone();

        let ptr = new_blocks.as_mut_ptr();
        for dy in 0..h {
            let layer_off = dy * wl;
            for dz in 0..l / 2 {
                let mirror_dz = l - 1 - dz;
                let off_a = layer_off + dz * w;
                let off_b = layer_off + mirror_dz * w;
                unsafe {
                    std::ptr::swap_nonoverlapping(ptr.add(off_a), ptr.add(off_b), w);
                }
            }
        }

        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_flip(block_state, Axis::Z));
        }

        self.blocks = new_blocks;
        self.palette = new_palette;
        self.rebuild_palette_index();
        self.rebuild_air_index();
        self.rebuild_non_air_count();

        let mut new_block_entities = HashMap::new();
        for ((x, y, z), mut be) in self.block_entities.drain() {
            let new_z = self.bbox.max.2 - (z - self.bbox.min.2);
            be.position = (x, y, new_z);
            new_block_entities.insert((x, y, new_z), be);
        }
        self.block_entities = new_block_entities;

        for entity in &mut self.entities {
            let rel_z = entity.position.2 - self.bbox.min.2 as f64;
            entity.position.2 = self.bbox.max.2 as f64 - rel_z;
        }
    }

    /// Rotate the region around the Y axis (horizontal plane)
    /// Degrees must be 90, 180, or 270
    pub fn rotate_y(&mut self, degrees: i32) {
        if degrees % 90 != 0 || degrees == 0 {
            return; // Only support 90-degree rotations
        }

        let normalized_degrees = ((degrees % 360 + 360) % 360) as i32;
        let rotations = normalized_degrees / 90;

        for _ in 0..rotations {
            self.rotate_y_90();
        }
    }

    fn rotate_y_90(&mut self) {
        use crate::transforms::{transform_block_state_rotate, Axis};

        // For 90-degree rotation around Y:
        // X and Z swap and dimensions change: (sx, sy, sz) -> (sz, sy, sx)
        // Keep the origin at min corner

        let old_bbox = self.bbox.clone();
        let (old_size_x, size_y, old_size_z) = self.size;

        // New dimensions after rotation
        let new_size = (old_size_z, size_y, old_size_x);
        let new_bbox = BoundingBox::from_position_and_size(self.position, new_size);

        let new_volume = new_bbox.volume() as usize;
        let air_index = self.cached_air_index;
        let mut new_blocks = vec![air_index; new_volume];

        // Transform each block position
        for index in 0..self.blocks.len() {
            let (x, y, z) = old_bbox.index_to_coords(index);

            let rel_x = x - old_bbox.min.0;
            let rel_z = z - old_bbox.min.2;

            let new_rel_x = rel_z;
            let new_rel_z = old_size_x - 1 - rel_x;

            let new_x = new_bbox.min.0 + new_rel_x;
            let new_z = new_bbox.min.2 + new_rel_z;

            let new_index = new_bbox.coords_to_index(new_x, y, new_z);
            new_blocks[new_index] = self.blocks[index];
        }

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
        self.rebuild_non_air_count();

        // Transform block entities
        let old_bbox_for_be = old_bbox.clone();
        let mut new_block_entities = HashMap::new();
        for ((x, y, z), mut be) in self.block_entities.drain() {
            let rel_x = x - old_bbox_for_be.min.0;
            let rel_z = z - old_bbox_for_be.min.2;
            let new_rel_x = rel_z;
            let new_rel_z = old_size_x - 1 - rel_x;
            let new_x = new_bbox_clone.min.0 + new_rel_x;
            let new_z = new_bbox_clone.min.2 + new_rel_z;
            be.position = (new_x, y, new_z);
            new_block_entities.insert((new_x, y, new_z), be);
        }
        self.block_entities = new_block_entities;

        // Transform entities
        for entity in &mut self.entities {
            let rel_x = entity.position.0 - old_bbox.min.0 as f64;
            let rel_z = entity.position.2 - old_bbox.min.2 as f64;
            let new_rel_x = rel_z;
            let new_rel_z = old_size_x as f64 - 1.0 - rel_x;
            entity.position.0 = new_bbox_clone.min.0 as f64 + new_rel_x;
            entity.position.2 = new_bbox_clone.min.2 as f64 + new_rel_z;
        }
    }

    /// Rotate the region around the X axis
    /// Degrees must be 90, 180, or 270
    pub fn rotate_x(&mut self, degrees: i32) {
        if degrees % 90 != 0 || degrees == 0 {
            return;
        }

        let normalized_degrees = ((degrees % 360 + 360) % 360) as i32;
        let rotations = normalized_degrees / 90;

        for _ in 0..rotations {
            self.rotate_x_90();
        }
    }

    fn rotate_x_90(&mut self) {
        use crate::transforms::{transform_block_state_rotate, Axis};

        let old_bbox = self.bbox.clone();
        let (size_x, old_size_y, old_size_z) = self.size;

        // New dimensions after rotation around X
        let new_size = (size_x, old_size_z, old_size_y);
        let new_bbox = BoundingBox::from_position_and_size(self.position, new_size);

        let new_volume = new_bbox.volume() as usize;
        let air_index = self.cached_air_index;
        let mut new_blocks = vec![air_index; new_volume];

        for index in 0..self.blocks.len() {
            let (x, y, z) = old_bbox.index_to_coords(index);

            let rel_y = y - old_bbox.min.1;
            let rel_z = z - old_bbox.min.2;

            let new_rel_y = rel_z;
            let new_rel_z = old_size_y - 1 - rel_y;

            let new_y = new_bbox.min.1 + new_rel_y;
            let new_z = new_bbox.min.2 + new_rel_z;

            let new_index = new_bbox.coords_to_index(x, new_y, new_z);
            new_blocks[new_index] = self.blocks[index];
        }

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
        self.rebuild_non_air_count();

        let mut new_block_entities = HashMap::new();
        for ((x, y, z), mut be) in self.block_entities.drain() {
            let rel_y = y - old_bbox.min.1;
            let rel_z = z - old_bbox.min.2;
            let new_rel_y = rel_z;
            let new_rel_z = old_size_y - 1 - rel_y;
            let new_y = new_bbox_clone.min.1 + new_rel_y;
            let new_z = new_bbox_clone.min.2 + new_rel_z;
            be.position = (x, new_y, new_z);
            new_block_entities.insert((x, new_y, new_z), be);
        }
        self.block_entities = new_block_entities;

        for entity in &mut self.entities {
            let rel_y = entity.position.1 - old_bbox.min.1 as f64;
            let rel_z = entity.position.2 - old_bbox.min.2 as f64;
            let new_rel_y = rel_z;
            let new_rel_z = old_size_y as f64 - 1.0 - rel_y;
            entity.position.1 = new_bbox_clone.min.1 as f64 + new_rel_y;
            entity.position.2 = new_bbox_clone.min.2 as f64 + new_rel_z;
        }
    }

    /// Rotate the region around the Z axis
    /// Degrees must be 90, 180, or 270
    pub fn rotate_z(&mut self, degrees: i32) {
        if degrees % 90 != 0 || degrees == 0 {
            return;
        }

        let normalized_degrees = ((degrees % 360 + 360) % 360) as i32;
        let rotations = normalized_degrees / 90;

        for _ in 0..rotations {
            self.rotate_z_90();
        }
    }

    fn rotate_z_90(&mut self) {
        use crate::transforms::{transform_block_state_rotate, Axis};

        let old_bbox = self.bbox.clone();
        let (old_size_x, old_size_y, size_z) = self.size;

        // New dimensions after rotation around Z
        let new_size = (old_size_y, old_size_x, size_z);
        let new_bbox = BoundingBox::from_position_and_size(self.position, new_size);

        let new_volume = new_bbox.volume() as usize;
        let air_index = self.cached_air_index;
        let mut new_blocks = vec![air_index; new_volume];

        for index in 0..self.blocks.len() {
            let (x, y, z) = old_bbox.index_to_coords(index);

            let rel_x = x - old_bbox.min.0;
            let rel_y = y - old_bbox.min.1;

            let new_rel_x = rel_y;
            let new_rel_y = old_size_x - 1 - rel_x;

            let new_x = new_bbox.min.0 + new_rel_x;
            let new_y = new_bbox.min.1 + new_rel_y;

            let new_index = new_bbox.coords_to_index(new_x, new_y, z);
            new_blocks[new_index] = self.blocks[index];
        }

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
        self.rebuild_non_air_count();

        let mut new_block_entities = HashMap::new();
        for ((x, y, z), mut be) in self.block_entities.drain() {
            let rel_x = x - old_bbox.min.0;
            let rel_y = y - old_bbox.min.1;
            let new_rel_x = rel_y;
            let new_rel_y = old_size_x - 1 - rel_x;
            let new_x = new_bbox_clone.min.0 + new_rel_x;
            let new_y = new_bbox_clone.min.1 + new_rel_y;
            be.position = (new_x, new_y, z);
            new_block_entities.insert((new_x, new_y, z), be);
        }
        self.block_entities = new_block_entities;

        for entity in &mut self.entities {
            let rel_x = entity.position.0 - old_bbox.min.0 as f64;
            let rel_y = entity.position.1 - old_bbox.min.1 as f64;
            let new_rel_x = rel_y;
            let new_rel_y = old_size_x as f64 - 1.0 - rel_x;
            entity.position.0 = new_bbox_clone.min.0 as f64 + new_rel_x;
            entity.position.1 = new_bbox_clone.min.1 as f64 + new_rel_y;
        }
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

    /// Direct array write with air-count and tight-bounds bookkeeping.
    /// Caller must ensure (x, y, z) is within region bounds.
    #[inline(always)]
    pub fn set_block_at_index_unchecked(&mut self, palette_index: usize, x: i32, y: i32, z: i32) {
        let index = self.coords_to_index(x, y, z);
        let old_palette_index = self.blocks[index];
        self.blocks[index] = palette_index;

        let old_is_air = old_palette_index == self.cached_air_index;
        let new_is_air = palette_index == self.cached_air_index;
        if old_is_air && !new_is_air {
            self.non_air_count += 1;
        } else if !old_is_air && new_is_air {
            self.non_air_count -= 1;
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
        let block_index = self.blocks[index];
        self.palette.get(block_index).map(|bs| bs.name.as_str())
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
        let new_is_air = palette_index == self.cached_air_index;
        let w = self.cached_width;
        let wl = self.cached_width_x_length;
        let base_x = self.bbox.min.0;
        let base_y = self.bbox.min.1;
        let base_z = self.bbox.min.2;

        let dx_min = min.0 - base_x;
        let dx_max = max.0 - base_x;
        let row_len = (dx_max - dx_min + 1) as usize;

        let mut air_delta: i64 = 0;

        for y in min.1..=max.1 {
            let dy = y - base_y;
            for z in min.2..=max.2 {
                let dz = z - base_z;
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

        // Update tight bounds once for the entire fill
        if !new_is_air {
            self.update_tight_bounds(min.0, min.1, min.2);
            self.update_tight_bounds(max.0, max.1, max.2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockState;

    #[test]
    fn test_pack_block_states_to_long_array() {
        //array from 1 to 16
        let blocks = (1..=16).collect::<Vec<usize>>();
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
            block_entities: HashMap::new(),
            palette_index: HashMap::new(),
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

        // With hybrid approach, expect aggressive expansion
        assert_eq!(region.position, (-65, -65, -65)); // Now expects -65 instead of -1
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
            assert!(index >= 0 && index < volume2);
            assert_eq!(index, i);
        }

        region.merge(&region2);

        let volume3 = region.volume();
        for i in 0..27 {
            let coords = region.index_to_coords(i);
            let index = region.coords_to_index(coords.0, coords.1, coords.2);
            assert!(index >= 0 && index < volume3);
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

        // Allocated dimensions should now be much larger due to pre-allocation
        let allocated_dims = region.get_dimensions();
        assert!(allocated_dims.0 >= 64); // At least 64 due to expansion strategy
        assert!(allocated_dims.1 >= 64);
        assert!(allocated_dims.2 >= 64);

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
