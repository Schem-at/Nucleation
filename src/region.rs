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
            tight_bounds: None, // Will be set when first non-air block is placed
        };
        region.rebuild_bbox();
        region
    }

    #[inline(always)]
    pub fn rebuild_bbox(&mut self) {
        self.bbox = BoundingBox::from_position_and_size(self.position, self.size);
    }

    /// Rebuild tight bounds by scanning all blocks
    /// This is typically called after deserialization
    pub fn rebuild_tight_bounds(&mut self) {
        self.tight_bounds = None;

        // Scan all blocks to find non-air blocks
        for index in 0..self.blocks.len() {
            let palette_idx = self.blocks[index];
            if palette_idx > 0 {
                // palette_idx 0 is always air, so > 0 means non-air
                let (x, y, z) = self.index_to_coords(index);
                self.update_tight_bounds(x, y, z);
            }
        }
    }

    pub fn get_or_insert_in_palette(&mut self, block: BlockState) -> usize {
        match self.palette_index.get(&block) {
            Some(&index) => index,
            None => {
                let index = self.palette.len();
                self.palette.push(block.clone());
                self.palette_index.insert(block, index);
                index
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        // Check if all blocks in the region are air (typically index 0 in palette)
        self.blocks
            .iter()
            .all(|&block_index| self.palette[block_index as usize].name == "minecraft:air")
    }

    /// Alternative implementation checking for non-air blocks
    pub fn has_non_air_blocks(&self) -> bool {
        self.blocks
            .iter()
            .any(|&block_index| self.palette[block_index as usize].name != "minecraft:air")
    }

    /// Count non-air blocks (if this method doesn't exist already)
    pub fn count_non_air_blocks(&self) -> usize {
        self.blocks
            .iter()
            .filter(|&&block_index| self.palette[block_index as usize].name != "minecraft:air")
            .count()
    }

    // Add this after from_nbt deserialization
    fn rebuild_palette_index(&mut self) {
        self.palette_index = HashMap::with_capacity(self.palette.len());
        for (index, block) in self.palette.iter().enumerate() {
            self.palette_index.insert(block.clone(), index);
        }
    }

    pub fn get_block_entities_as_list(&self) -> Vec<BlockEntity> {
        self.block_entities.values().cloned().collect()
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockState) -> bool {
        if !self.is_in_region(x, y, z) {
            self.expand_to_fit(x, y, z);
        }

        let index = self.coords_to_index(x, y, z);
        let palette_index = self.get_or_insert_in_palette(block.clone());
        self.blocks[index] = palette_index;

        // Update tight bounds if this is a non-air block
        if !block.name.contains("air") {
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
        self.bbox.coords_to_index(x, y, z)
    }

    #[inline(always)]
    pub fn index_to_coords(&self, index: usize) -> (i32, i32, i32) {
        self.bbox.index_to_coords(index)
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
        match &self.tight_bounds {
            None => {
                // First non-air block - initialize tight bounds to this single point
                self.tight_bounds = Some(BoundingBox::new((x, y, z), (x, y, z)));
            }
            Some(bounds) => {
                // Expand tight bounds to include this point
                let new_bounds = BoundingBox::new(
                    (
                        bounds.min.0.min(x),
                        bounds.min.1.min(y),
                        bounds.min.2.min(z),
                    ),
                    (
                        bounds.max.0.max(x),
                        bounds.max.1.max(y),
                        bounds.max.2.max(z),
                    ),
                );
                self.tight_bounds = Some(new_bounds);
            }
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
            // Empty region - return minimal region
            let mut empty = Region::new(self.name.clone(), (0, 0, 0), (1, 1, 1));
            empty.palette = self.palette.clone();
            return empty;
        };

        let tight_dims = tight_bounds.get_dimensions();
        let tight_pos = tight_bounds.min;

        // Create new region with exact tight bounds
        let mut compact = Region::new(self.name.clone(), tight_pos, tight_dims);

        // Build palette mapping (same as current palette)
        compact.palette = self.palette.clone();
        compact.rebuild_palette_index();

        // Copy blocks within tight bounds
        for index in 0..self.blocks.len() {
            let palette_idx = self.blocks[index];
            if palette_idx == 0 {
                continue; // Skip air blocks
            }

            let (x, y, z) = self.index_to_coords(index);

            // Only copy if within tight bounds
            if tight_bounds.contains((x, y, z)) {
                let block = self.palette[palette_idx].clone();
                compact.set_block(x, y, z, block);
            }
        }

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

    fn expand_to_bounding_box(&mut self, new_bounding_box: BoundingBox) {
        let new_size = new_bounding_box.get_dimensions();
        let new_position = new_bounding_box.min;

        if new_size == self.size && new_position == self.position {
            return;
        }

        let air_id = self
            .palette
            .iter()
            .position(|b| b.name == "minecraft:air")
            .unwrap();
        let mut new_blocks = vec![air_id; new_bounding_box.volume() as usize];

        // Copy existing blocks efficiently
        for index in 0..self.blocks.len() {
            let (x, y, z) = self.index_to_coords(index);
            let new_index = new_bounding_box.coords_to_index(x, y, z);
            new_blocks[new_index] = self.blocks[index];
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
        let bounding_box = self.get_bounding_box().union(&other.get_bounding_box());
        let other_bounding_box = other.get_bounding_box();

        let combined_bounding_box = bounding_box.union(&other_bounding_box);
        let new_size = combined_bounding_box.get_dimensions();
        let new_position = combined_bounding_box.min;

        let mut new_blocks = vec![0; (combined_bounding_box.volume()) as usize];
        let mut new_palette = self.palette.clone();
        let mut reverse_new_palette: HashMap<BlockState, usize> = HashMap::new();
        for (index, block) in self.palette.iter().enumerate() {
            reverse_new_palette.insert(block.clone(), index);
        }

        // Process blocks from current region
        for index in 0..self.blocks.len() {
            let (x, y, z) = self.index_to_coords(index);
            let new_index = combined_bounding_box.coords_to_index(x, y, z);
            let block_index = self.blocks[index];
            let block = &self.palette[block_index];
            if let Some(palette_index) = reverse_new_palette.get(block) {
                new_blocks[new_index] = *palette_index;
            } else {
                new_blocks[new_index] = new_palette.len();
                new_palette.push(block.clone());
                reverse_new_palette.insert(block.clone(), new_palette.len() - 1);
            }
        }

        // Process blocks from other region
        for index in 0..other.blocks.len() {
            let (x, y, z) = other.index_to_coords(index);
            let new_index = combined_bounding_box.coords_to_index(x, y, z);
            let block_palette_index = other.blocks[index];
            let block = &other.palette[block_palette_index];
            if let Some(palette_index) = reverse_new_palette.get(block) {
                if block.name == "minecraft:air" {
                    continue;
                }
                new_blocks[new_index] = *palette_index;
            } else {
                new_palette.push(block.clone());
                reverse_new_palette.insert(block.clone(), new_palette.len() - 1);
                if block.name == "minecraft:air" {
                    continue;
                }
                new_blocks[new_index] = new_palette.len() - 1;
            }
        }

        // Update region properties
        self.position = new_position;
        self.size = new_size;
        self.blocks = new_blocks;
        self.palette = new_palette;

        // CRUCIAL: Rebuild the cached bounding box after changing position and size
        self.rebuild_bbox();

        // Rebuild palette index for performance
        self.rebuild_palette_index();

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
                    if let block_entity = BlockEntity::from_nbt(be_compound) {
                        block_entities.insert((coords[0], coords[1], coords[2]), block_entity);
                    }
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
            tight_bounds: None, // Will be rebuilt by scanning blocks
        };

        // Rebuild palette index, bbox, and tight bounds after deserialization
        region.rebuild_palette_index();
        region.rebuild_bbox();
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
        self.blocks
            .iter()
            .filter(|&&block_index| block_index != 0)
            .count()
    }

    pub fn get_palette_index(&self, block: &BlockState) -> Option<usize> {
        self.palette.iter().position(|b| b == block)
    }

    // Transformation methods

    /// Flip the region along the X axis
    pub fn flip_x(&mut self) {
        use crate::transforms::{transform_block_state_flip, Axis};

        let (size_x, size_y, size_z) = self.size;
        let volume = self.volume();
        let mut new_blocks = vec![0; volume];

        // Create new transformed blocks array
        for index in 0..volume {
            let (x, y, z) = self.index_to_coords(index);
            let new_x = self.bbox.max.0 - (x - self.bbox.min.0);
            let new_index = self.coords_to_index(new_x, y, z);
            new_blocks[new_index] = self.blocks[index];
        }

        // Transform block state properties
        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_flip(block_state, Axis::X));
        }

        self.blocks = new_blocks;
        self.palette = new_palette;
        self.rebuild_palette_index();

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

        let volume = self.volume();
        let mut new_blocks = vec![0; volume];

        for index in 0..volume {
            let (x, y, z) = self.index_to_coords(index);
            let new_y = self.bbox.max.1 - (y - self.bbox.min.1);
            let new_index = self.coords_to_index(x, new_y, z);
            new_blocks[new_index] = self.blocks[index];
        }

        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_flip(block_state, Axis::Y));
        }

        self.blocks = new_blocks;
        self.palette = new_palette;
        self.rebuild_palette_index();

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

        let volume = self.volume();
        let mut new_blocks = vec![0; volume];

        for index in 0..volume {
            let (x, y, z) = self.index_to_coords(index);
            let new_z = self.bbox.max.2 - (z - self.bbox.min.2);
            let new_index = self.coords_to_index(x, y, new_z);
            new_blocks[new_index] = self.blocks[index];
        }

        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_flip(block_state, Axis::Z));
        }

        self.blocks = new_blocks;
        self.palette = new_palette;
        self.rebuild_palette_index();

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
        use crate::transforms::{transform_block_state_rotate, Axis};

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
        let air_index = self
            .palette
            .iter()
            .position(|b| b.name == "minecraft:air")
            .unwrap_or(0);
        let mut new_blocks = vec![air_index; new_volume];

        // Transform each block position
        for index in 0..self.blocks.len() {
            let (x, y, z) = old_bbox.index_to_coords(index);

            // Calculate relative position in old space
            let rel_x = x - old_bbox.min.0;
            let rel_z = z - old_bbox.min.2;

            // 90-degree rotation: (x, z) -> (z, size_x - 1 - x)
            let new_rel_x = rel_z;
            let new_rel_z = old_size_x - 1 - rel_x;

            // Calculate absolute position in new space
            let new_x = new_bbox.min.0 + new_rel_x;
            let new_z = new_bbox.min.2 + new_rel_z;

            let new_index = new_bbox.coords_to_index(new_x, y, new_z);
            new_blocks[new_index] = self.blocks[index];
        }

        // Transform block state properties
        let mut new_palette = Vec::with_capacity(self.palette.len());
        for block_state in &self.palette {
            new_palette.push(transform_block_state_rotate(block_state, Axis::Y, 90));
        }

        let new_bbox_clone = new_bbox.clone();

        self.position = new_bbox.to_position_and_size().0;
        self.size = new_bbox.to_position_and_size().1;
        self.blocks = new_blocks;
        self.palette = new_palette;
        self.bbox = new_bbox;
        self.rebuild_palette_index();

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
        use crate::transforms::{transform_block_state_rotate, Axis};

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
        let air_index = self
            .palette
            .iter()
            .position(|b| b.name == "minecraft:air")
            .unwrap_or(0);
        let mut new_blocks = vec![air_index; new_volume];

        for index in 0..self.blocks.len() {
            let (x, y, z) = old_bbox.index_to_coords(index);

            let rel_y = y - old_bbox.min.1;
            let rel_z = z - old_bbox.min.2;

            // 90-degree rotation around X: (y, z) -> (z, size_y - 1 - y)
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
        self.bbox = new_bbox;
        self.rebuild_palette_index();

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
        use crate::transforms::{transform_block_state_rotate, Axis};

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
        let air_index = self
            .palette
            .iter()
            .position(|b| b.name == "minecraft:air")
            .unwrap_or(0);
        let mut new_blocks = vec![air_index; new_volume];

        for index in 0..self.blocks.len() {
            let (x, y, z) = old_bbox.index_to_coords(index);

            let rel_x = x - old_bbox.min.0;
            let rel_y = y - old_bbox.min.1;

            // 90-degree rotation around Z: (x, y) -> (y, size_x - 1 - x)
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
        self.bbox = new_bbox;
        self.rebuild_palette_index();

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
        let mut region = Region {
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

        assert!(region.set_block(0, 0, 0, stone.clone()));
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

        region.set_block(0, 0, 0, stone.clone());
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
        region.set_block(0, 0, 0, stone.clone());

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
        region.set_block(0, 0, 0, stone.clone());

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

        region.set_block(0, 0, 0, stone.clone());
        region.set_block(1, 1, 1, stone.clone());

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

            region1.set_block(0, 0, 0, stone.clone());
            region2.set_block(2, 2, 2, stone.clone());

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

            region1.set_block(0, 0, 0, stone.clone());
            region2.set_block(2, 2, 2, dirt.clone());

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

            region1.set_block(0, 0, 0, stone.clone());
            region1.set_block(1, 1, 1, dirt.clone());

            region2.set_block(2, 2, 2, dirt.clone());

            region1.merge(&region2);

            assert_eq!(region1.size, (3, 3, 3));
            assert_eq!(region1.get_block(0, 0, 0), Some(&stone));
            assert_eq!(region1.get_block(1, 1, 1), Some(&dirt));
            assert_eq!(region1.get_block(2, 2, 2), Some(&dirt));
        }
        region1.set_block(0, 0, 0, stone.clone());
        region2.set_block(2, 2, 2, stone.clone());

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

        region1.set_block(0, 0, 0, stone.clone());
        region2.set_block(2, 2, 2, dirt.clone());

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

        region1.set_block(0, 0, 0, stone.clone());
        region1.set_block(1, 1, 1, dirt.clone());

        region2.set_block(2, 2, 2, dirt.clone());

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
        region.set_block(3, 3, 3, stone.clone());

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
        region.set_block(-1, -1, -1, dirt.clone());

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
        region.set_block(10, 10, 10, stone.clone());

        assert_eq!(region.position, (0, 0, 0));
        assert_eq!(region.get_block(10, 10, 10), Some(&stone));
    }

    #[test]
    fn test_expand_to_fit_corner_to_corner() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        // Place a block at one corner
        region.set_block(0, 0, 0, stone.clone());

        // Place another block far from the first to trigger resizing
        region.set_block(4, 4, 4, dirt.clone());

        assert_eq!(region.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region.get_block(4, 4, 4), Some(&dirt));
    }

    #[test]
    fn test_expand_to_fit_multiple_expansions() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        // Perform multiple expansions
        region.set_block(3, 3, 3, stone.clone());
        region.set_block(7, 7, 7, stone.clone());
        region.set_block(-2, -2, -2, stone.clone());

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
        region.set_block(0, 0, 0, stone.clone());
        region.set_block(2, 2, 2, dirt.clone());

        // Trigger expansion
        region.set_block(5, 5, 5, stone.clone());

        assert_eq!(region.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region.get_block(2, 2, 2), Some(&dirt));
        assert_eq!(region.get_block(5, 5, 5), Some(&stone));
    }

    #[test]
    fn test_incremental_expansion_in_x() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        for x in 0..32 {
            region.set_block(x, 0, 0, stone.clone());
            assert_eq!(region.get_block(x, 0, 0), Some(&stone));
        }
    }

    #[test]
    fn test_incremental_expansion_in_y() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        for y in 0..32 {
            region.set_block(0, y, 0, stone.clone());
            assert_eq!(region.get_block(0, y, 0), Some(&stone));
        }
    }

    #[test]
    fn test_incremental_expansion_in_z() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        for z in 0..32 {
            region.set_block(0, 0, z, stone.clone());
            assert_eq!(region.get_block(0, 0, z), Some(&stone));
        }
    }

    #[test]
    fn test_incremental_expansion_in_x_y_z() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));
        let stone = BlockState::new("minecraft:stone".to_string());

        for i in 0..32 {
            region.set_block(i, i, i, stone.clone());
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
                        region.set_block(x, y, z, stone.clone());
                    } else {
                        region.set_block(x, y, z, dirt.clone());
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

        let mut volume1 = region.volume();
        for i in 0..8 {
            let coords = region.index_to_coords(i);
            let index = region.coords_to_index(coords.0, coords.1, coords.2);
            assert!(index < volume1);
            assert_eq!(index, i);
        }

        let region2 = Region::new("Test".to_string(), (0, 0, 0), (-2, -2, -2));

        let mut volume2 = region2.volume();
        for i in 0..8 {
            let coords = region2.index_to_coords(i);
            let index = region2.coords_to_index(coords.0, coords.1, coords.2);
            assert!(index >= 0 && index < volume2);
            assert_eq!(index, i);
        }

        region.merge(&region2);

        let mut volume3 = region.volume();
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

        region1.set_block(0, 0, 0, stone.clone());
        region2.set_block(-2, -2, -2, stone.clone());

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
        region.set_block(1, 0, 1, stone.clone());
        region.set_block(0, 1, 0, stone.clone());

        // Expand the region by setting a block outside the current bounds
        region.set_block(1, 2, 1, diamond.clone());

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

        region.set_block(5, 3, 7, stone);

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
        region.set_block(2, 1, 3, stone.clone());
        region.set_block(5, 4, 8, dirt.clone());
        region.set_block(3, 2, 5, stone.clone());

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
        region.set_block(1, 1, 1, stone.clone());
        region.set_block(3, 3, 3, stone.clone());

        // Place air blocks (should not affect tight bounds)
        region.set_block(0, 0, 0, air.clone());
        region.set_block(5, 5, 5, air.clone());

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
        region.set_block(2, 2, 2, stone.clone());

        let bounds1 = region.get_tight_bounds().unwrap();
        assert_eq!(bounds1.min, (2, 2, 2));
        assert_eq!(bounds1.max, (2, 2, 2));

        // Place block that requires region expansion
        region.set_block(10, 10, 10, stone.clone());

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
        region.set_block(0, 0, 0, stone.clone());
        region.set_block(1, 0, 0, stone.clone());
        region.set_block(0, 1, 0, stone.clone());
        region.set_block(1, 1, 0, stone.clone());

        // Force expansion by placing a block far away
        region.set_block(10, 10, 10, stone.clone());

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
        region.set_block(1, 1, 1, stone.clone());
        region.set_block(5, 5, 5, stone.clone());

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
}
