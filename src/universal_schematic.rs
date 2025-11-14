use crate::block_entity::BlockEntity;
use crate::block_position::BlockPosition;
use crate::bounding_box::BoundingBox;
use crate::chunk::Chunk;
use crate::entity::Entity;
use crate::metadata::Metadata;
use crate::region::Region;
use crate::utils::NbtValue;
use crate::utils::{parse_custom_name, parse_items_array, NbtMap};
use crate::BlockState;
use quartz_nbt::{NbtCompound, NbtTag};
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct UniversalSchematic {
    pub metadata: Metadata,
    pub default_region: Region,
    pub other_regions: HashMap<String, Region>,
    pub default_region_name: String,
    #[serde(skip, default = "HashMap::new")]
    block_state_cache: HashMap<String, BlockState>,
}

#[derive(Debug, Clone)]
pub struct ChunkIndices {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub chunk_z: i32,
    pub blocks: Vec<(BlockPosition, usize)>, // (position, palette_index)
}

#[derive(Debug, Clone)]
pub struct AllPalettes {
    pub default_palette: Vec<BlockState>,
    pub region_palettes: HashMap<String, Vec<BlockState>>,
}

pub enum ChunkLoadingStrategy {
    Default,
    DistanceToCamera(f32, f32, f32), // Camera position
    TopDown,
    BottomUp,
    CenterOutward,
    Random,
}
pub type SimpleBlockMapping = (&'static str, Vec<(&'static str, &'static str)>);

impl UniversalSchematic {
    pub fn new(name: String) -> Self {
        let default_region_name = "Main".to_string();
        UniversalSchematic {
            metadata: Metadata {
                name: Some(name),
                ..Metadata::default()
            },
            default_region: Region::new(default_region_name.clone(), (0, 0, 0), (1, 1, 1)),
            other_regions: HashMap::new(),
            default_region_name,
            block_state_cache: HashMap::new(),
        }
    }

    pub fn get_all_regions(&self) -> HashMap<String, &Region> {
        let mut all_regions = HashMap::new();
        all_regions.insert(self.default_region_name.clone(), &self.default_region);
        all_regions.extend(
            self.other_regions
                .iter()
                .map(|(name, region)| (name.clone(), region)),
        );
        all_regions
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockState) -> bool {
        // Check if the default region is empty and needs repositioning
        if self.default_region.is_empty() {
            // Reposition the default region to the first block's location
            self.default_region =
                Region::new(self.default_region_name.clone(), (x, y, z), (1, 1, 1));
        }

        self.default_region.set_block(x, y, z, block)
    }

    pub fn set_block_str(&mut self, x: i32, y: i32, z: i32, block_name: &str) -> bool {
        // Check if string contains properties (bracket notation) or NBT data (braces)
        if block_name.contains('[') || block_name.ends_with('}') {
            self.set_block_from_string(x, y, z, block_name).unwrap()
        } else {
            let block_state = match self.block_state_cache.get(block_name) {
                Some(cached) => cached.clone(),
                None => {
                    let new_block = BlockState::new(block_name.to_string());
                    self.block_state_cache
                        .insert(block_name.to_string(), new_block.clone());
                    new_block
                }
            };

            self.set_block(x, y, z, block_state)
        }
    }

    pub fn set_block_in_region(
        &mut self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
        block: BlockState,
    ) -> bool {
        if region_name == self.default_region_name {
            self.default_region.set_block(x, y, z, block)
        } else {
            let region = self
                .other_regions
                .entry(region_name.to_string())
                .or_insert_with(|| Region::new(region_name.to_string(), (x, y, z), (1, 1, 1)));
            region.set_block(x, y, z, block)
        }
    }

    pub fn get_palette_from_region(&self, region_name: &str) -> Option<Vec<BlockState>> {
        if region_name == self.default_region_name {
            Some(self.default_region.get_palette())
        } else {
            self.other_regions
                .get(region_name)
                .map(|region| region.get_palette())
        }
    }

    pub fn get_default_region_palette(&self) -> Vec<BlockState> {
        let default_region_name = self.default_region_name.clone();
        self.get_palette_from_region(&default_region_name)
            .unwrap_or_else(Vec::new)
    }

    pub fn set_block_in_region_str(
        &mut self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
    ) -> bool {
        // Get cached block state
        let block_state = match self.block_state_cache.get(block_name) {
            Some(cached) => cached.clone(),
            None => {
                let new_block = BlockState::new(block_name.to_string());
                self.block_state_cache
                    .insert(block_name.to_string(), new_block.clone());
                new_block
            }
        };

        self.set_block_in_region(region_name, x, y, z, block_state)
    }

    pub fn from_layers(
        name: String,
        block_mappings: &[(&'static char, SimpleBlockMapping)],
        layers: &str,
    ) -> Self {
        let mut schematic = UniversalSchematic::new(name);
        let full_mappings = Self::convert_to_full_mappings(block_mappings);

        let layers: Vec<&str> = layers
            .split("\n\n")
            .map(|layer| layer.trim())
            .filter(|layer| !layer.is_empty())
            .collect();

        for (y, layer) in layers.iter().enumerate() {
            let rows: Vec<&str> = layer
                .lines()
                .map(|row| row.trim())
                .filter(|row| !row.is_empty())
                .collect();

            for (z, row) in rows.iter().enumerate() {
                for (x, c) in row.chars().enumerate() {
                    if let Some(block_state) = full_mappings.get(&c) {
                        schematic.set_block(x as i32, y as i32, z as i32, block_state.clone());
                    } else if c != ' ' {
                        println!(
                            "Warning: Unknown character '{}' at position ({}, {}, {})",
                            c, x, y, z
                        );
                    }
                }
            }
        }

        schematic
    }

    fn convert_to_full_mappings(
        simple_mappings: &[(&'static char, SimpleBlockMapping)],
    ) -> HashMap<char, BlockState> {
        simple_mappings
            .iter()
            .map(|(&c, (name, props))| {
                let block_state = BlockState::new(format!("minecraft:{}", name)).with_properties(
                    props
                        .iter()
                        .map(|&(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                );
                (c, block_state)
            })
            .collect()
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<&BlockState> {
        // Check default region first
        if self.default_region.get_bounding_box().contains((x, y, z)) {
            return self.default_region.get_block(x, y, z);
        }

        // Check other regions
        for region in self.other_regions.values() {
            if region.get_bounding_box().contains((x, y, z)) {
                return region.get_block(x, y, z);
            }
        }
        None
    }

    pub fn get_block_entity(&self, position: BlockPosition) -> Option<&BlockEntity> {
        // Check default region first
        if self
            .default_region
            .get_bounding_box()
            .contains((position.x, position.y, position.z))
        {
            if let Some(entity) = self.default_region.get_block_entity(position) {
                return Some(entity);
            }
        }

        // Check other regions
        for region in self.other_regions.values() {
            if region
                .get_bounding_box()
                .contains((position.x, position.y, position.z))
            {
                if let Some(entity) = region.get_block_entity(position) {
                    return Some(entity);
                }
            }
        }
        None
    }

    pub fn get_block_entities_as_list(&self) -> Vec<BlockEntity> {
        let mut block_entities = Vec::new();
        block_entities.extend(self.default_region.get_block_entities_as_list());
        for region in self.other_regions.values() {
            block_entities.extend(region.get_block_entities_as_list());
        }
        block_entities
    }

    pub fn get_entities_as_list(&self) -> Vec<Entity> {
        let mut entities = Vec::new();
        entities.extend(self.default_region.entities.clone());
        for region in self.other_regions.values() {
            entities.extend(region.entities.clone());
        }
        entities
    }

    pub fn set_block_entity(&mut self, position: BlockPosition, block_entity: BlockEntity) -> bool {
        self.default_region.set_block_entity(position, block_entity)
    }

    /// Sets a block with NBT data in one convenient call
    ///
    /// # Arguments
    /// * `x`, `y`, `z` - Block coordinates
    /// * `block_name` - Block name with optional properties (e.g., "minecraft:sign[rotation=0]")
    /// * `nbt_data` - NBT data as a HashMap (keys and values as strings for JSON compatibility)
    ///
    /// # Examples
    /// ```
    /// use nucleation::UniversalSchematic;
    /// use std::collections::HashMap;
    ///
    /// let mut schematic = UniversalSchematic::new("test".to_string());
    /// let mut nbt = HashMap::new();
    /// nbt.insert("Text1".to_string(), r#"{"text":"Hello"}"#.to_string());
    /// schematic.set_block_with_nbt(0, 0, 0, "minecraft:sign", nbt).unwrap();
    /// ```
    pub fn set_block_with_nbt(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
        nbt_data: std::collections::HashMap<String, String>,
    ) -> Result<bool, String> {
        // Parse block name (may include properties like [rotation=0])
        let (block_state, _) = Self::parse_block_string(block_name)?;

        // Set the basic block first
        if !self.set_block(x, y, z, block_state.clone()) {
            return Ok(false);
        }

        // Create block entity with NBT data
        let mut block_entity = BlockEntity::new(block_state.name.clone(), (x, y, z));

        for (key, value) in nbt_data {
            // Try to parse value as NbtValue
            let nbt_value = Self::parse_nbt_value(&value);
            block_entity = block_entity.with_nbt_data(key, nbt_value);
        }

        self.set_block_entity(BlockPosition { x, y, z }, block_entity);
        Ok(true)
    }

    /// Helper function to parse a string value into an appropriate NbtValue
    fn parse_nbt_value(value: &str) -> NbtValue {
        // If it's a JSON string (for Text components), keep as string
        if value.starts_with('{') && value.ends_with('}') {
            return NbtValue::String(value.to_string());
        }

        // Try to parse as integer
        if let Ok(i) = value.parse::<i32>() {
            return NbtValue::Int(i);
        }

        // Try to parse as float
        if let Ok(f) = value.parse::<f32>() {
            return NbtValue::Float(f);
        }

        // Try to parse as boolean
        if let Ok(b) = value.parse::<bool>() {
            return NbtValue::Byte(if b { 1 } else { 0 });
        }

        // Default to string
        NbtValue::String(value.to_string())
    }

    pub fn set_block_entity_in_region(
        &mut self,
        region_name: &str,
        position: BlockPosition,
        block_entity: BlockEntity,
    ) -> bool {
        if region_name == self.default_region_name {
            self.default_region.set_block_entity(position, block_entity)
        } else {
            let region = self
                .other_regions
                .entry(region_name.to_string())
                .or_insert_with(|| {
                    Region::new(
                        region_name.to_string(),
                        (position.x, position.y, position.z),
                        (1, 1, 1),
                    )
                });
            region.set_block_entity(position, block_entity)
        }
    }

    pub fn get_blocks(&self) -> Vec<BlockState> {
        let mut blocks: Vec<BlockState> = Vec::new();

        // Add blocks from default region
        let default_palette = self.default_region.get_palette();
        for block_index in &self.default_region.blocks {
            blocks.push(default_palette[*block_index as usize].clone());
        }

        // Add blocks from other regions
        for region in self.other_regions.values() {
            let region_palette = region.get_palette();
            for block_index in &region.blocks {
                blocks.push(region_palette[*block_index as usize].clone());
            }
        }
        blocks
    }

    pub fn get_region_names(&self) -> Vec<String> {
        let mut names = vec![self.default_region_name.clone()];
        names.extend(self.other_regions.keys().cloned());
        names
    }

    pub fn get_region_from_index(&self, index: usize) -> Option<&Region> {
        if index == 0 {
            Some(&self.default_region)
        } else {
            self.other_regions.values().nth(index - 1)
        }
    }

    pub fn get_block_from_region(
        &self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<&BlockState> {
        if region_name == self.default_region_name {
            self.default_region.get_block(x, y, z)
        } else {
            self.other_regions
                .get(region_name)
                .and_then(|region| region.get_block(x, y, z))
        }
    }

    pub fn get_dimensions(&self) -> (i32, i32, i32) {
        let bounding_box = self.get_bounding_box();
        bounding_box.get_dimensions()
    }

    /// Get the tight bounding box (actual min/max coordinates of placed non-air blocks)
    /// This only considers the default region. Returns None if no non-air blocks exist.
    pub fn get_tight_bounds(&self) -> Option<BoundingBox> {
        self.default_region.get_tight_bounds()
    }

    /// Get the tight dimensions (width, height, length) of actual block content
    /// Returns (0, 0, 0) if no non-air blocks have been placed yet
    pub fn get_tight_dimensions(&self) -> (i32, i32, i32) {
        self.default_region.get_tight_dimensions()
    }

    pub fn get_json_string(&self) -> Result<String, String> {
        // Attempt to serialize the metadata
        let metadata_json = serde_json::to_string(&self.metadata).map_err(|e| {
            format!(
                "Failed to serialize 'metadata' in UniversalSchematic: {}",
                e
            )
        })?;

        // Create a temporary combined regions map for serialization
        let mut combined_regions = HashMap::new();
        combined_regions.insert(
            self.default_region_name.clone(),
            self.default_region.clone(),
        );
        combined_regions.extend(self.other_regions.clone());

        // Attempt to serialize the combined regions
        let regions_json = serde_json::to_string(&combined_regions)
            .map_err(|e| format!("Failed to serialize 'regions' in UniversalSchematic: {}", e))?;

        // Combine everything into a single JSON object manually
        let combined_json = format!(
            "{{\"metadata\":{},\"regions\":{}}}",
            metadata_json, regions_json
        );

        Ok(combined_json)
    }

    pub(crate) fn total_blocks(&self) -> i32 {
        let mut total = self.default_region.count_blocks() as i32;
        total += self
            .other_regions
            .values()
            .map(|r| r.count_blocks() as i32)
            .sum::<i32>();
        total
    }

    pub(crate) fn total_volume(&self) -> i32 {
        let mut total = self.default_region.volume() as i32;
        total += self
            .other_regions
            .values()
            .map(|r| r.volume() as i32)
            .sum::<i32>();
        total
    }

    pub fn get_region_bounding_box(&self, region_name: &str) -> Option<BoundingBox> {
        if region_name == self.default_region_name {
            Some(self.default_region.get_bounding_box())
        } else {
            self.other_regions
                .get(region_name)
                .map(|region| region.get_bounding_box())
        }
    }

    pub fn get_schematic_bounding_box(&self) -> Option<BoundingBox> {
        let mut bounding_box = self.default_region.get_bounding_box();

        for region in self.other_regions.values() {
            bounding_box = bounding_box.union(&region.get_bounding_box());
        }

        Some(bounding_box)
    }

    pub fn add_region(&mut self, region: Region) -> bool {
        if region.name == self.default_region_name {
            self.default_region = region;
            true
        } else if self.other_regions.contains_key(&region.name) {
            false
        } else {
            self.other_regions.insert(region.name.clone(), region);
            true
        }
    }

    pub fn remove_region(&mut self, name: &str) -> Option<Region> {
        if name == self.default_region_name {
            None // Cannot remove the default region
        } else {
            self.other_regions.remove(name)
        }
    }

    pub fn get_region(&self, name: &str) -> Option<&Region> {
        if name == self.default_region_name {
            Some(&self.default_region)
        } else {
            self.other_regions.get(name)
        }
    }

    pub fn get_region_mut(&mut self, name: &str) -> Option<&mut Region> {
        if name == self.default_region_name {
            Some(&mut self.default_region)
        } else {
            self.other_regions.get_mut(name)
        }
    }

    pub fn get_merged_region(&self) -> Region {
        let mut merged_region = self.default_region.clone();

        for region in self.other_regions.values() {
            merged_region.merge(region);
        }

        merged_region
    }

    pub fn add_block_entity_in_region(
        &mut self,
        region_name: &str,
        block_entity: BlockEntity,
    ) -> bool {
        if region_name == self.default_region_name {
            self.default_region.add_block_entity(block_entity);
            true
        } else {
            let region = self
                .other_regions
                .entry(region_name.to_string())
                .or_insert_with(|| {
                    Region::new(region_name.to_string(), block_entity.position, (1, 1, 1))
                });
            region.add_block_entity(block_entity);
            true
        }
    }

    pub fn remove_block_entity_in_region(
        &mut self,
        region_name: &str,
        position: (i32, i32, i32),
    ) -> Option<BlockEntity> {
        if region_name == self.default_region_name {
            self.default_region.remove_block_entity(position)
        } else {
            self.other_regions
                .get_mut(region_name)?
                .remove_block_entity(position)
        }
    }

    pub fn add_block_entity(&mut self, block_entity: BlockEntity) -> bool {
        self.default_region.add_block_entity(block_entity);
        true
    }

    pub fn remove_block_entity(&mut self, position: (i32, i32, i32)) -> Option<BlockEntity> {
        self.default_region.remove_block_entity(position)
    }

    pub fn add_entity_in_region(&mut self, region_name: &str, entity: Entity) -> bool {
        if region_name == self.default_region_name {
            self.default_region.add_entity(entity);
            true
        } else {
            let region = self
                .other_regions
                .entry(region_name.to_string())
                .or_insert_with(|| {
                    let rounded_position = (
                        entity.position.0.round() as i32,
                        entity.position.1.round() as i32,
                        entity.position.2.round() as i32,
                    );
                    Region::new(region_name.to_string(), rounded_position, (1, 1, 1))
                });
            region.add_entity(entity);
            true
        }
    }

    pub fn remove_entity_in_region(&mut self, region_name: &str, index: usize) -> Option<Entity> {
        if region_name == self.default_region_name {
            self.default_region.remove_entity(index)
        } else {
            self.other_regions
                .get_mut(region_name)?
                .remove_entity(index)
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> bool {
        self.default_region.add_entity(entity);
        true
    }

    pub fn remove_entity(&mut self, index: usize) -> Option<Entity> {
        self.default_region.remove_entity(index)
    }

    pub fn to_nbt(&self) -> NbtCompound {
        let mut root = NbtCompound::new();

        root.insert("Metadata", self.metadata.to_nbt());

        // Create combined regions for NBT
        let mut regions_tag = NbtCompound::new();
        regions_tag.insert(&self.default_region_name, self.default_region.to_nbt());
        for (name, region) in &self.other_regions {
            regions_tag.insert(name, region.to_nbt());
        }
        root.insert("Regions", NbtTag::Compound(regions_tag));

        root.insert(
            "DefaultRegion",
            NbtTag::String(self.default_region_name.clone()),
        );

        root
    }

    pub fn from_nbt(nbt: NbtCompound) -> Result<Self, String> {
        let metadata = Metadata::from_nbt(
            nbt.get::<_, &NbtCompound>("Metadata")
                .map_err(|e| format!("Failed to get Metadata: {}", e))?,
        )?;

        let regions_tag = nbt
            .get::<_, &NbtCompound>("Regions")
            .map_err(|e| format!("Failed to get Regions: {}", e))?;

        let default_region_name = nbt
            .get::<_, &str>("DefaultRegion")
            .map_err(|e| format!("Failed to get DefaultRegion: {}", e))?
            .to_string();

        let mut default_region = None;
        let mut other_regions = HashMap::new();

        for (region_name, region_tag) in regions_tag.inner() {
            if let NbtTag::Compound(region_compound) = region_tag {
                let region = Region::from_nbt(&region_compound.clone())?;
                if region_name == &default_region_name {
                    default_region = Some(region);
                } else {
                    other_regions.insert(region_name.to_string(), region);
                }
            }
        }

        let default_region = default_region.ok_or("Default region not found in NBT")?;

        Ok(UniversalSchematic {
            metadata,
            default_region,
            other_regions,
            default_region_name,
            block_state_cache: HashMap::new(),
        })
    }

    pub fn get_default_region_mut(&mut self) -> &mut Region {
        &mut self.default_region
    }

    /// Swap the default region with another region by name
    pub fn swap_default_region(&mut self, region_name: &str) -> Result<(), String> {
        if region_name == self.default_region_name {
            return Ok(()); // Already the default region
        }

        if let Some(new_default) = self.other_regions.remove(region_name) {
            let old_default = std::mem::replace(&mut self.default_region, new_default);
            let old_default_name = self.default_region_name.clone();

            // Update the default region name
            self.default_region_name = region_name.to_string();

            // Put the old default into other_regions
            self.other_regions.insert(old_default_name, old_default);

            Ok(())
        } else {
            Err(format!("Region '{}' not found", region_name))
        }
    }

    /// Set a new default region directly
    pub fn set_default_region(&mut self, region: Region) -> Region {
        let old_default = std::mem::replace(&mut self.default_region, region);
        self.default_region_name = self.default_region.name.clone();
        old_default
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        let mut bounding_box = self.default_region.get_bounding_box();

        for region in self.other_regions.values() {
            let region_bb = region.get_bounding_box();
            bounding_box = bounding_box.union(&region_bb);
        }

        bounding_box
    }

    /// Stack/repeat this schematic multiple times along an axis, returning a new schematic
    ///
    /// # Arguments
    /// * `count` - Number of additional copies (total instances will be count + 1, including original)
    /// * `axis` - Which axis to stack along ('x', 'y', or 'z')
    /// * `spacing` - Spacing between instances (0 = touching)
    ///
    /// # Example
    /// ```ignore
    /// // Create a 1-bit adder, then stack it 3 times along X axis for 4-bit adder
    /// let single_bit = create_1bit_adder();
    /// let four_bit = single_bit.stack(3, 'x', 0)?;
    /// ```
    pub fn stack(&self, count: usize, axis: char, spacing: i32) -> Result<Self, String> {
        if count == 0 {
            return Ok(self.clone());
        }

        let bbox = self.get_bounding_box();
        let size = bbox.get_dimensions();

        // Calculate step size based on axis
        let (step_x, step_y, step_z) = match axis.to_lowercase().next().unwrap() {
            'x' => (size.0 + spacing, 0, 0),
            'y' => (0, size.1 + spacing, 0),
            'z' => (0, 0, size.2 + spacing),
            _ => return Err(format!("Invalid axis '{}', must be 'x', 'y', or 'z'", axis)),
        };

        let mut result = UniversalSchematic::new(format!(
            "{}_stacked",
            self.metadata
                .name
                .as_ref()
                .unwrap_or(&"schematic".to_string())
        ));

        // Copy all blocks from each instance
        for instance in 0..=count {
            let offset_x = step_x * instance as i32;
            let offset_y = step_y * instance as i32;
            let offset_z = step_z * instance as i32;

            // Copy all blocks from this schematic
            for (pos, block_state) in self.iter_blocks() {
                result.set_block(
                    pos.x + offset_x,
                    pos.y + offset_y,
                    pos.z + offset_z,
                    block_state.clone(),
                );
            }

            // Copy block entities from default region
            for block_entity in self.default_region.get_block_entities_as_list() {
                let pos = block_entity.position;
                result.set_block_entity(
                    BlockPosition {
                        x: pos.0 + offset_x,
                        y: pos.1 + offset_y,
                        z: pos.2 + offset_z,
                    },
                    block_entity.clone(),
                );
            }
        }

        Ok(result)
    }

    /// Stack/repeat this schematic in-place, modifying the current schematic
    ///
    /// This is more memory-efficient than `stack()` if you don't need the original.
    ///
    /// # Arguments
    /// * `count` - Number of additional copies to add
    /// * `axis` - Which axis to stack along ('x', 'y', or 'z')
    /// * `spacing` - Spacing between instances (0 = touching)
    pub fn stack_in_place(&mut self, count: usize, axis: char, spacing: i32) -> Result<(), String> {
        if count == 0 {
            return Ok(());
        }

        let bbox = self.get_bounding_box();
        let size = bbox.get_dimensions();

        // Calculate step size based on axis
        let (step_x, step_y, step_z) = match axis.to_lowercase().next().unwrap() {
            'x' => (size.0 + spacing, 0, 0),
            'y' => (0, size.1 + spacing, 0),
            'z' => (0, 0, size.2 + spacing),
            _ => return Err(format!("Invalid axis '{}', must be 'x', 'y', or 'z'", axis)),
        };

        // Collect all blocks and entities from the original (instance 0)
        let original_blocks: Vec<_> = self
            .iter_blocks()
            .map(|(pos, block)| (pos.clone(), block.clone()))
            .collect();

        let original_entities: Vec<_> = self.default_region.get_block_entities_as_list();

        // Add copies for instances 1..=count
        for instance in 1..=count {
            let offset_x = step_x * instance as i32;
            let offset_y = step_y * instance as i32;
            let offset_z = step_z * instance as i32;

            for (pos, block_state) in &original_blocks {
                self.set_block(
                    pos.x + offset_x,
                    pos.y + offset_y,
                    pos.z + offset_z,
                    block_state.clone(),
                );
            }

            for block_entity in &original_entities {
                let pos = block_entity.position;
                self.set_block_entity(
                    BlockPosition {
                        x: pos.0 + offset_x,
                        y: pos.1 + offset_y,
                        z: pos.2 + offset_z,
                    },
                    block_entity.clone(),
                );
            }
        }

        Ok(())
    }

    pub fn to_schematic(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        crate::formats::schematic::to_schematic(self)
    }

    pub fn from_schematic(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        crate::formats::schematic::from_schematic(data)
    }

    pub fn count_block_types(&self) -> HashMap<BlockState, usize> {
        let mut block_counts = HashMap::new();

        // Count blocks in default region
        let default_block_counts = self.default_region.count_block_types();
        for (block, count) in default_block_counts {
            *block_counts.entry(block).or_insert(0) += count;
        }

        // Count blocks in other regions
        for region in self.other_regions.values() {
            let region_block_counts = region.count_block_types();
            for (block, count) in region_block_counts {
                *block_counts.entry(block).or_insert(0) += count;
            }
        }
        block_counts
    }

    pub fn copy_region(
        &mut self,
        from_schematic: &UniversalSchematic,
        bounds: &BoundingBox,
        target_position: (i32, i32, i32),
        excluded_blocks: &[BlockState],
    ) -> Result<(), String> {
        let offset = (
            target_position.0 - bounds.min.0,
            target_position.1 - bounds.min.1,
            target_position.2 - bounds.min.2,
        );

        let air_block = BlockState::new("minecraft:air".to_string());

        // Copy blocks
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    if let Some(block) = from_schematic.get_block(x, y, z) {
                        let new_x = x + offset.0;
                        let new_y = y + offset.1;
                        let new_z = z + offset.2;

                        if excluded_blocks.contains(block) {
                            // Set air block instead of skipping
                            self.set_block(new_x, new_y, new_z, air_block.clone());
                        } else {
                            self.set_block(new_x, new_y, new_z, block.clone());
                        }
                    }
                }
            }
        }

        // Rest of the method remains the same...
        // Copy block entities
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    let pos = BlockPosition { x, y, z };
                    if let Some(block_entity) = from_schematic.get_block_entity(pos) {
                        let mut new_block_entity = block_entity.clone();
                        new_block_entity.position = (
                            block_entity.position.0 + offset.0,
                            block_entity.position.1 + offset.1,
                            block_entity.position.2 + offset.2,
                        );
                        self.set_block_entity(
                            BlockPosition {
                                x: x + offset.0,
                                y: y + offset.1,
                                z: z + offset.2,
                            },
                            new_block_entity,
                        );
                    }
                }
            }
        }

        // Copy entities that are within the bounds
        let mut entities_to_copy = Vec::new();

        // Collect entities from default region
        for entity in &from_schematic.default_region.entities {
            let entity_pos = (
                entity.position.0.floor() as i32,
                entity.position.1.floor() as i32,
                entity.position.2.floor() as i32,
            );

            if bounds.contains(entity_pos) {
                let mut new_entity = entity.clone();
                new_entity.position = (
                    entity.position.0 + offset.0 as f64,
                    entity.position.1 + offset.1 as f64,
                    entity.position.2 + offset.2 as f64,
                );
                entities_to_copy.push(new_entity);
            }
        }

        // Collect entities from other regions
        for region in from_schematic.other_regions.values() {
            for entity in &region.entities {
                let entity_pos = (
                    entity.position.0.floor() as i32,
                    entity.position.1.floor() as i32,
                    entity.position.2.floor() as i32,
                );

                if bounds.contains(entity_pos) {
                    let mut new_entity = entity.clone();
                    new_entity.position = (
                        entity.position.0 + offset.0 as f64,
                        entity.position.1 + offset.1 as f64,
                        entity.position.2 + offset.2 as f64,
                    );
                    entities_to_copy.push(new_entity);
                }
            }
        }

        // Add all collected entities
        for entity in entities_to_copy {
            self.add_entity(entity);
        }

        Ok(())
    }

    pub fn split_into_chunks(
        &self,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
    ) -> Vec<Chunk> {
        use std::collections::HashMap;
        let mut chunk_map: HashMap<(i32, i32, i32), Vec<BlockPosition>> = HashMap::new();

        // Helper function to get chunk coordinate
        let get_chunk_coord = |pos: i32, chunk_size: i32| -> i32 {
            let offset = if pos < 0 { chunk_size - 1 } else { 0 };
            (pos - offset) / chunk_size
        };

        // Process default region - skip air blocks for consistency with split_into_chunks_indices
        for (index, &palette_index) in self.default_region.blocks.iter().enumerate() {
            if palette_index == 0 {
                continue; // Skip air blocks
            }

            let (x, y, z) = self.default_region.index_to_coords(index);
            let chunk_x = get_chunk_coord(x, chunk_width);
            let chunk_y = get_chunk_coord(y, chunk_height);
            let chunk_z = get_chunk_coord(z, chunk_length);
            let chunk_key = (chunk_x, chunk_y, chunk_z);

            chunk_map
                .entry(chunk_key)
                .or_insert_with(Vec::new)
                .push(BlockPosition { x, y, z });
        }

        // Process other regions - skip air blocks for consistency with split_into_chunks_indices
        for region in self.other_regions.values() {
            for (index, &palette_index) in region.blocks.iter().enumerate() {
                if palette_index == 0 {
                    continue; // Skip air blocks
                }

                let (x, y, z) = region.index_to_coords(index);
                let chunk_x = get_chunk_coord(x, chunk_width);
                let chunk_y = get_chunk_coord(y, chunk_height);
                let chunk_z = get_chunk_coord(z, chunk_length);
                let chunk_key = (chunk_x, chunk_y, chunk_z);

                chunk_map
                    .entry(chunk_key)
                    .or_insert_with(Vec::new)
                    .push(BlockPosition { x, y, z });
            }
        }

        chunk_map
            .into_iter()
            .map(|((chunk_x, chunk_y, chunk_z), positions)| Chunk {
                chunk_x,
                chunk_y,
                chunk_z,
                positions,
            })
            .collect()
    }

    pub fn iter_blocks(&self) -> impl Iterator<Item = (BlockPosition, &BlockState)> {
        // Create an iterator that chains default region and other regions
        let default_iter = self.default_region.blocks.iter().enumerate().filter_map(
            move |(index, block_index)| {
                let (x, y, z) = self.default_region.index_to_coords(index);
                Some((
                    BlockPosition { x, y, z },
                    &self.default_region.palette[*block_index as usize],
                ))
            },
        );

        let other_iter = self.other_regions.values().flat_map(|region| {
            region
                .blocks
                .iter()
                .enumerate()
                .filter_map(move |(index, block_index)| {
                    let (x, y, z) = region.index_to_coords(index);
                    Some((
                        BlockPosition { x, y, z },
                        &region.palette[*block_index as usize],
                    ))
                })
        });

        default_iter.chain(other_iter)
    }

    pub fn iter_blocks_indices(&self) -> impl Iterator<Item = (BlockPosition, usize)> + '_ {
        // Iterator for default region - returns palette indices directly
        let default_iter = self.default_region.blocks.iter().enumerate().filter_map(
            move |(index, &palette_index)| {
                // Skip air blocks (usually index 0) to reduce data transfer
                if palette_index == 0 {
                    return None;
                }
                let (x, y, z) = self.default_region.index_to_coords(index);
                Some((BlockPosition { x, y, z }, palette_index))
            },
        );

        // Iterator for other regions
        let other_iter = self.other_regions.values().flat_map(|region| {
            region
                .blocks
                .iter()
                .enumerate()
                .filter_map(move |(index, &palette_index)| {
                    if palette_index == 0 {
                        return None;
                    }
                    let (x, y, z) = region.index_to_coords(index);
                    Some((BlockPosition { x, y, z }, palette_index))
                })
        });

        default_iter.chain(other_iter)
    }

    pub fn iter_chunks_indices(
        &self,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
        strategy: Option<ChunkLoadingStrategy>,
    ) -> impl Iterator<Item = ChunkIndices> + '_ {
        let chunks = self.split_into_chunks_indices(chunk_width, chunk_height, chunk_length);

        // Apply sorting based on strategy (same logic as before)
        let mut ordered_chunks = chunks;
        if let Some(strategy) = strategy {
            match strategy {
                ChunkLoadingStrategy::Default => {
                    // Default order - no sorting needed
                }
                ChunkLoadingStrategy::DistanceToCamera(cam_x, cam_y, cam_z) => {
                    ordered_chunks.sort_by(|a, b| {
                        let a_center_x = (a.chunk_x * chunk_width) + (chunk_width / 2);
                        let a_center_y = (a.chunk_y * chunk_height) + (chunk_height / 2);
                        let a_center_z = (a.chunk_z * chunk_length) + (chunk_length / 2);

                        let b_center_x = (b.chunk_x * chunk_width) + (chunk_width / 2);
                        let b_center_y = (b.chunk_y * chunk_height) + (chunk_height / 2);
                        let b_center_z = (b.chunk_z * chunk_length) + (chunk_length / 2);

                        let a_dist = (a_center_x as f32 - cam_x).powi(2)
                            + (a_center_y as f32 - cam_y).powi(2)
                            + (a_center_z as f32 - cam_z).powi(2);

                        let b_dist = (b_center_x as f32 - cam_x).powi(2)
                            + (b_center_y as f32 - cam_y).powi(2)
                            + (b_center_z as f32 - cam_z).powi(2);

                        a_dist
                            .partial_cmp(&b_dist)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
                ChunkLoadingStrategy::TopDown => {
                    ordered_chunks.sort_by(|a, b| b.chunk_y.cmp(&a.chunk_y));
                }
                ChunkLoadingStrategy::BottomUp => {
                    ordered_chunks.sort_by(|a, b| a.chunk_y.cmp(&b.chunk_y));
                }
                ChunkLoadingStrategy::CenterOutward => {
                    let (width, height, depth) = self.get_dimensions();
                    let center_x = (width / 2) / chunk_width;
                    let center_y = (height / 2) / chunk_height;
                    let center_z = (depth / 2) / chunk_length;

                    ordered_chunks.sort_by(|a, b| {
                        let a_dist = (a.chunk_x - center_x).pow(2)
                            + (a.chunk_y - center_y).pow(2)
                            + (a.chunk_z - center_z).pow(2);

                        let b_dist = (b.chunk_x - center_x).pow(2)
                            + (b.chunk_y - center_y).pow(2)
                            + (b.chunk_z - center_z).pow(2);

                        a_dist.cmp(&b_dist)
                    });
                }
                ChunkLoadingStrategy::Random => {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};

                    let mut hasher = DefaultHasher::new();
                    if let Some(name) = &self.metadata.name {
                        name.hash(&mut hasher);
                    } else {
                        "Default".hash(&mut hasher);
                    }
                    let seed = hasher.finish();

                    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
                    use rand::seq::SliceRandom;
                    ordered_chunks.shuffle(&mut rng);
                }
            }
        }

        ordered_chunks.into_iter()
    }

    fn split_into_chunks_indices(
        &self,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
    ) -> Vec<ChunkIndices> {
        use std::collections::HashMap;
        let mut chunk_map: HashMap<(i32, i32, i32), Vec<(BlockPosition, usize)>> = HashMap::new();

        // Helper function to get chunk coordinate
        let get_chunk_coord = |pos: i32, chunk_size: i32| -> i32 {
            let offset = if pos < 0 { chunk_size - 1 } else { 0 };
            (pos - offset) / chunk_size
        };

        // Process default region
        for (index, &palette_index) in self.default_region.blocks.iter().enumerate() {
            if palette_index == 0 {
                continue; // Skip air blocks
            }

            let (x, y, z) = self.default_region.index_to_coords(index);
            let chunk_x = get_chunk_coord(x, chunk_width);
            let chunk_y = get_chunk_coord(y, chunk_height);
            let chunk_z = get_chunk_coord(z, chunk_length);
            let chunk_key = (chunk_x, chunk_y, chunk_z);

            chunk_map
                .entry(chunk_key)
                .or_insert_with(Vec::new)
                .push((BlockPosition { x, y, z }, palette_index));
        }

        // Process other regions
        for region in self.other_regions.values() {
            for (index, &palette_index) in region.blocks.iter().enumerate() {
                if palette_index == 0 {
                    continue; // Skip air blocks
                }

                let (x, y, z) = region.index_to_coords(index);
                let chunk_x = get_chunk_coord(x, chunk_width);
                let chunk_y = get_chunk_coord(y, chunk_height);
                let chunk_z = get_chunk_coord(z, chunk_length);
                let chunk_key = (chunk_x, chunk_y, chunk_z);

                chunk_map
                    .entry(chunk_key)
                    .or_insert_with(Vec::new)
                    .push((BlockPosition { x, y, z }, palette_index));
            }
        }

        chunk_map
            .into_iter()
            .map(|((chunk_x, chunk_y, chunk_z), blocks)| ChunkIndices {
                chunk_x,
                chunk_y,
                chunk_z,
                blocks,
            })
            .collect()
    }
    pub fn get_all_palettes(&self) -> AllPalettes {
        let mut all_palettes = AllPalettes {
            default_palette: self.default_region.palette.clone(),
            region_palettes: HashMap::new(),
        };

        for (region_name, region) in &self.other_regions {
            all_palettes
                .region_palettes
                .insert(region_name.clone(), region.palette.clone());
        }

        all_palettes
    }

    pub fn get_chunk_blocks_indices(
        &self,
        offset_x: i32,
        offset_y: i32,
        offset_z: i32,
        width: i32,
        height: i32,
        length: i32,
    ) -> Vec<(BlockPosition, usize)> {
        let mut blocks = Vec::new();

        // Check default region
        if self.default_region.get_bounding_box().intersects_range(
            offset_x,
            offset_y,
            offset_z,
            offset_x + width,
            offset_y + height,
            offset_z + length,
        ) {
            for (index, &palette_index) in self.default_region.blocks.iter().enumerate() {
                if palette_index == 0 {
                    continue; // Skip air
                }

                let (x, y, z) = self.default_region.index_to_coords(index);
                if x >= offset_x
                    && x < offset_x + width
                    && y >= offset_y
                    && y < offset_y + height
                    && z >= offset_z
                    && z < offset_z + length
                {
                    blocks.push((BlockPosition { x, y, z }, palette_index));
                }
            }
        }

        // Check other regions
        for region in self.other_regions.values() {
            if region.get_bounding_box().intersects_range(
                offset_x,
                offset_y,
                offset_z,
                offset_x + width,
                offset_y + height,
                offset_z + length,
            ) {
                for (index, &palette_index) in region.blocks.iter().enumerate() {
                    if palette_index == 0 {
                        continue; // Skip air
                    }

                    let (x, y, z) = region.index_to_coords(index);
                    if x >= offset_x
                        && x < offset_x + width
                        && y >= offset_y
                        && y < offset_y + height
                        && z >= offset_z
                        && z < offset_z + length
                    {
                        blocks.push((BlockPosition { x, y, z }, palette_index));
                    }
                }
            }
        }

        blocks
    }

    pub fn iter_chunks(
        &self,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
        strategy: Option<ChunkLoadingStrategy>,
    ) -> impl Iterator<Item = Chunk> + '_ {
        let chunks = self.split_into_chunks(chunk_width, chunk_height, chunk_length);

        // Apply sorting based on strategy
        let mut ordered_chunks = chunks;
        if let Some(strategy) = strategy {
            match strategy {
                ChunkLoadingStrategy::Default => {
                    // Default order - no sorting needed
                }
                ChunkLoadingStrategy::DistanceToCamera(cam_x, cam_y, cam_z) => {
                    // Sort by distance to camera
                    ordered_chunks.sort_by(|a, b| {
                        let a_center_x = (a.chunk_x * chunk_width) + (chunk_width / 2);
                        let a_center_y = (a.chunk_y * chunk_height) + (chunk_height / 2);
                        let a_center_z = (a.chunk_z * chunk_length) + (chunk_length / 2);

                        let b_center_x = (b.chunk_x * chunk_width) + (chunk_width / 2);
                        let b_center_y = (b.chunk_y * chunk_height) + (chunk_height / 2);
                        let b_center_z = (b.chunk_z * chunk_length) + (chunk_length / 2);

                        let a_dist = (a_center_x as f32 - cam_x).powi(2)
                            + (a_center_y as f32 - cam_y).powi(2)
                            + (a_center_z as f32 - cam_z).powi(2);

                        let b_dist = (b_center_x as f32 - cam_x).powi(2)
                            + (b_center_y as f32 - cam_y).powi(2)
                            + (b_center_z as f32 - cam_z).powi(2);

                        // Sort by ascending distance (closest first)
                        a_dist
                            .partial_cmp(&b_dist)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
                ChunkLoadingStrategy::TopDown => {
                    // Sort by y-coordinate, highest first
                    ordered_chunks.sort_by(|a, b| b.chunk_y.cmp(&a.chunk_y));
                }
                ChunkLoadingStrategy::BottomUp => {
                    // Sort by y-coordinate, lowest first
                    ordered_chunks.sort_by(|a, b| a.chunk_y.cmp(&b.chunk_y));
                }
                ChunkLoadingStrategy::CenterOutward => {
                    // Calculate schematic center in chunk coordinates
                    let (width, height, depth) = self.get_dimensions();
                    let center_x = (width / 2) / chunk_width;
                    let center_y = (height / 2) / chunk_height;
                    let center_z = (depth / 2) / chunk_length;

                    // Sort by distance from center
                    ordered_chunks.sort_by(|a, b| {
                        let a_dist = (a.chunk_x - center_x).pow(2)
                            + (a.chunk_y - center_y).pow(2)
                            + (a.chunk_z - center_z).pow(2);

                        let b_dist = (b.chunk_x - center_x).pow(2)
                            + (b.chunk_y - center_y).pow(2)
                            + (b.chunk_z - center_z).pow(2);

                        a_dist.cmp(&b_dist)
                    });
                }
                ChunkLoadingStrategy::Random => {
                    // Shuffle the chunks using a deterministic seed
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};

                    let mut hasher = DefaultHasher::new();
                    if let Some(name) = &self.metadata.name {
                        name.hash(&mut hasher);
                    } else {
                        "Default".hash(&mut hasher);
                    }
                    let seed = hasher.finish();

                    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
                    use rand::seq::SliceRandom;
                    ordered_chunks.shuffle(&mut rng);
                }
            }
        }

        // Process each chunk like in the original implementation
        ordered_chunks.into_iter().map(move |chunk| {
            let positions = chunk.positions;
            let blocks = positions
                .into_iter()
                .filter_map(|pos| {
                    self.get_block(pos.x, pos.y, pos.z)
                        .map(|block| (pos, block))
                })
                .collect::<Vec<_>>();

            Chunk {
                chunk_x: chunk.chunk_x,
                chunk_y: chunk.chunk_y,
                chunk_z: chunk.chunk_z,
                positions: blocks.iter().map(|(pos, _)| *pos).collect(),
            }
        })
    }

    // Keep the original method for backward compatibility
    pub fn iter_chunks_original(
        &self,
        chunk_width: i32,
        chunk_height: i32,
        chunk_length: i32,
    ) -> impl Iterator<Item = Chunk> + '_ {
        self.iter_chunks(chunk_width, chunk_height, chunk_length, None)
    }

    pub fn set_block_from_string(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_string: &str,
    ) -> Result<bool, String> {
        let (mut block_state, nbt_data) = Self::parse_block_string(block_string)?;

        // Special handling for jukebox: set has_record blockstate if RecordItem exists
        if block_state.name.contains("jukebox") {
            if let Some(ref nbt) = nbt_data {
                let has_record = nbt.contains_key("RecordItem");
                block_state
                    .properties
                    .insert("has_record".to_string(), has_record.to_string());
            }
        }

        // Set the basic block first
        if !self.set_block(x, y, z, block_state.clone()) {
            return Ok(false);
        }

        // If we have NBT data, create and set the block entity
        if let Some(nbt_data) = nbt_data {
            let mut block_entity = BlockEntity::new(block_state.name.clone(), (x, y, z));

            // Add NBT data
            for (key, value) in nbt_data {
                block_entity = block_entity.with_nbt_data(key, value);
            }

            self.set_block_entity(BlockPosition { x, y, z }, block_entity);
        }

        Ok(true)
    }

    /// Parses a block string into its components (block state and optional NBT data)
    fn calculate_items_for_signal(signal_strength: u8) -> u32 {
        if signal_strength == 0 {
            return 0;
        }

        const BARREL_SLOTS: u32 = 27;
        const MAX_STACK: u32 = 64;
        const MAX_SIGNAL: u32 = 14;

        let calculated = ((BARREL_SLOTS * MAX_STACK) as f64 / MAX_SIGNAL as f64)
            * (signal_strength as f64 - 1.0);
        let items_needed = calculated.ceil() as u32;

        std::cmp::max(signal_strength as u32, items_needed)
    }

    /// Creates Items NBT data for a barrel to achieve desired signal strength
    /// Uses modern format (1.20.5+): lowercase 'count' as Int
    fn create_barrel_items_nbt(signal_strength: u8) -> Vec<NbtValue> {
        let total_items = Self::calculate_items_for_signal(signal_strength);
        let mut items = Vec::new();
        let mut remaining_items = total_items;
        let mut slot: u8 = 0;

        while remaining_items > 0 {
            let stack_size = std::cmp::min(remaining_items, 64);
            let mut item_nbt = NbtMap::new(); // Using NbtMap instead of HashMap
                                              // Modern format (1.20.5+): lowercase 'count' as Int
            item_nbt.insert("count".to_string(), NbtValue::Int(stack_size as i32));
            item_nbt.insert("Slot".to_string(), NbtValue::Byte(slot as i8));
            item_nbt.insert(
                "id".to_string(),
                NbtValue::String("minecraft:redstone_block".to_string()),
            );

            items.push(NbtValue::Compound(item_nbt));

            remaining_items -= stack_size;
            slot += 1;
        }

        items
    }
    /// Parse a block string into its components, handling special signal strength case
    pub fn parse_block_string(
        block_string: &str,
    ) -> Result<(BlockState, Option<HashMap<String, NbtValue>>), String> {
        let mut parts = block_string.splitn(2, '{');
        let block_state_str = parts.next().unwrap().trim();
        let nbt_str = parts.next().map(|s| s.trim_end_matches('}'));

        // Parse block state
        let block_state = if block_state_str.contains('[') {
            let mut state_parts = block_state_str.splitn(2, '[');
            let block_name = state_parts.next().unwrap();
            let properties_str = state_parts
                .next()
                .ok_or("Missing properties closing bracket")?
                .trim_end_matches(']');

            let mut properties = HashMap::new();
            for prop in properties_str.split(',') {
                let mut kv = prop.split('=');
                let key = kv.next().ok_or("Missing property key")?.trim();
                let value = kv
                    .next()
                    .ok_or("Missing property value")?
                    .trim()
                    .trim_matches(|c| c == '\'' || c == '"');
                properties.insert(key.to_string(), value.to_string());
            }

            BlockState::new(block_name.to_string()).with_properties(properties)
        } else {
            BlockState::new(block_state_str.to_string())
        };

        // Parse NBT data if present using enhanced parser
        let nbt_data = if let Some(nbt_str) = nbt_str {
            let parsed = crate::utils::parse_enhanced_nbt(block_state.get_name(), nbt_str)?;
            if parsed.is_empty() {
                None
            } else {
                Some(parsed)
            }
        } else {
            None
        };

        Ok((block_state, nbt_data))
    }

    pub fn create_schematic_from_region(&self, bounds: &BoundingBox) -> Self {
        let mut new_schematic =
            UniversalSchematic::new(format!("Region_{}", self.default_region_name));

        // Normalize coordinates to start at 0,0,0 in the new schematic
        let offset = (-bounds.min.0, -bounds.min.1, -bounds.min.2);

        // Copy blocks
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    if let Some(block) = self.get_block(x, y, z) {
                        let new_x = x + offset.0;
                        let new_y = y + offset.1;
                        let new_z = z + offset.2;
                        new_schematic.set_block(new_x, new_y, new_z, block.clone());
                    }
                }
            }
        }

        // Copy block entities
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    let pos = BlockPosition { x, y, z };
                    if let Some(block_entity) = self.get_block_entity(pos) {
                        let mut new_block_entity = block_entity.clone();
                        new_block_entity.position = (
                            block_entity.position.0 + offset.0,
                            block_entity.position.1 + offset.1,
                            block_entity.position.2 + offset.2,
                        );
                        new_schematic.set_block_entity(
                            BlockPosition {
                                x: x + offset.0,
                                y: y + offset.1,
                                z: z + offset.2,
                            },
                            new_block_entity,
                        );
                    }
                }
            }
        }

        // Copy entities that are within the bounds
        let mut entities_to_copy = Vec::new();

        // Check default region
        for entity in &self.default_region.entities {
            let entity_pos = (
                entity.position.0.floor() as i32,
                entity.position.1.floor() as i32,
                entity.position.2.floor() as i32,
            );

            if bounds.contains(entity_pos) {
                let mut new_entity = entity.clone();
                new_entity.position = (
                    entity.position.0 + offset.0 as f64,
                    entity.position.1 + offset.1 as f64,
                    entity.position.2 + offset.2 as f64,
                );
                entities_to_copy.push(new_entity);
            }
        }

        // Check other regions
        for region in self.other_regions.values() {
            for entity in &region.entities {
                let entity_pos = (
                    entity.position.0.floor() as i32,
                    entity.position.1.floor() as i32,
                    entity.position.2.floor() as i32,
                );

                if bounds.contains(entity_pos) {
                    let mut new_entity = entity.clone();
                    new_entity.position = (
                        entity.position.0 + offset.0 as f64,
                        entity.position.1 + offset.1 as f64,
                        entity.position.2 + offset.2 as f64,
                    );
                    entities_to_copy.push(new_entity);
                }
            }
        }

        // Add all collected entities
        for entity in entities_to_copy {
            new_schematic.add_entity(entity);
        }

        new_schematic
    }

    pub fn clear_block_state_cache(&mut self) {
        self.block_state_cache.clear();
    }

    /// Get cache statistics for debugging
    pub fn cache_stats(&self) -> (usize, usize) {
        (
            self.block_state_cache.len(),
            self.block_state_cache.capacity(),
        )
    }

    // Transformation methods (convenience wrappers for default region)

    /// Flip the default region along the X axis
    pub fn flip_x(&mut self) {
        self.default_region.flip_x();
    }

    /// Flip the default region along the Y axis
    pub fn flip_y(&mut self) {
        self.default_region.flip_y();
    }

    /// Flip the default region along the Z axis
    pub fn flip_z(&mut self) {
        self.default_region.flip_z();
    }

    /// Rotate the default region around the Y axis (horizontal plane)
    pub fn rotate_y(&mut self, degrees: i32) {
        self.default_region.rotate_y(degrees);
    }

    /// Rotate the default region around the X axis
    pub fn rotate_x(&mut self, degrees: i32) {
        self.default_region.rotate_x(degrees);
    }

    /// Rotate the default region around the Z axis
    pub fn rotate_z(&mut self, degrees: i32) {
        self.default_region.rotate_z(degrees);
    }

    /// Flip a specific region along the X axis
    pub fn flip_region_x(&mut self, region_name: &str) -> Result<(), String> {
        if region_name == self.default_region_name {
            self.default_region.flip_x();
            Ok(())
        } else if let Some(region) = self.other_regions.get_mut(region_name) {
            region.flip_x();
            Ok(())
        } else {
            Err(format!("Region '{}' not found", region_name))
        }
    }

    /// Flip a specific region along the Y axis
    pub fn flip_region_y(&mut self, region_name: &str) -> Result<(), String> {
        if region_name == self.default_region_name {
            self.default_region.flip_y();
            Ok(())
        } else if let Some(region) = self.other_regions.get_mut(region_name) {
            region.flip_y();
            Ok(())
        } else {
            Err(format!("Region '{}' not found", region_name))
        }
    }

    /// Flip a specific region along the Z axis
    pub fn flip_region_z(&mut self, region_name: &str) -> Result<(), String> {
        if region_name == self.default_region_name {
            self.default_region.flip_z();
            Ok(())
        } else if let Some(region) = self.other_regions.get_mut(region_name) {
            region.flip_z();
            Ok(())
        } else {
            Err(format!("Region '{}' not found", region_name))
        }
    }

    /// Rotate a specific region around the Y axis
    pub fn rotate_region_y(&mut self, region_name: &str, degrees: i32) -> Result<(), String> {
        if region_name == self.default_region_name {
            self.default_region.rotate_y(degrees);
            Ok(())
        } else if let Some(region) = self.other_regions.get_mut(region_name) {
            region.rotate_y(degrees);
            Ok(())
        } else {
            Err(format!("Region '{}' not found", region_name))
        }
    }

    /// Rotate a specific region around the X axis
    pub fn rotate_region_x(&mut self, region_name: &str, degrees: i32) -> Result<(), String> {
        if region_name == self.default_region_name {
            self.default_region.rotate_x(degrees);
            Ok(())
        } else if let Some(region) = self.other_regions.get_mut(region_name) {
            region.rotate_x(degrees);
            Ok(())
        } else {
            Err(format!("Region '{}' not found", region_name))
        }
    }

    /// Rotate a specific region around the Z axis
    pub fn rotate_region_z(&mut self, region_name: &str, degrees: i32) -> Result<(), String> {
        if region_name == self.default_region_name {
            self.default_region.rotate_z(degrees);
            Ok(())
        } else if let Some(region) = self.other_regions.get_mut(region_name) {
            region.rotate_z(degrees);
            Ok(())
        } else {
            Err(format!("Region '{}' not found", region_name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_entity;
    use crate::item::ItemStack;
    use quartz_nbt::io::{read_nbt, write_nbt};
    use std::io::Cursor;

    #[test]
    fn test_schematic_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        // Test automatic region creation and expansion
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        assert!(schematic.set_block(0, 0, 0, stone.clone()));
        assert_eq!(schematic.get_block(0, 0, 0), Some(&stone));

        assert!(schematic.set_block(5, 5, 5, dirt.clone()));
        assert_eq!(schematic.get_block(5, 5, 5), Some(&dirt));

        // Check that the default region was expanded
        assert_eq!(schematic.get_region("Main").unwrap().name, "Main");

        // Test explicit region creation and manipulation
        let obsidian = BlockState::new("minecraft:obsidian".to_string());
        assert!(schematic.set_block_in_region("Custom", 10, 10, 10, obsidian.clone()));
        assert_eq!(
            schematic.get_block_from_region("Custom", 10, 10, 10),
            Some(&obsidian)
        );

        // Check that the custom region was created
        let custom_region = schematic.get_region("Custom").unwrap();
        assert_eq!(custom_region.position, (10, 10, 10));

        // Test manual region addition
        let region2 = Region::new("Region2".to_string(), (20, 0, 0), (5, 5, 5));
        assert!(schematic.add_region(region2));
        assert!(!schematic.add_region(Region::new("Region2".to_string(), (0, 0, 0), (1, 1, 1))));

        // Test getting non-existent blocks
        assert_eq!(schematic.get_block(100, 100, 100), None);
        assert_eq!(
            schematic.get_block_from_region("NonexistentRegion", 0, 0, 0),
            None
        );

        // Test removing regions
        assert!(schematic.remove_region("Region2").is_some());
        assert!(schematic.remove_region("Region2").is_none());

        // Test that we cannot remove the default region
        assert!(schematic.remove_region("Main").is_none());

        // Test that removed region's blocks are no longer accessible
        assert_eq!(schematic.get_block_from_region("Region2", 20, 0, 0), None);
    }

    #[test]
    fn test_swap_default_region() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        // Add a block to the default region
        let stone = BlockState::new("minecraft:stone".to_string());
        schematic.set_block(0, 0, 0, stone.clone());

        // Create and add another region
        let mut custom_region = Region::new("Custom".to_string(), (10, 10, 10), (5, 5, 5));
        let dirt = BlockState::new("minecraft:dirt".to_string());
        custom_region.set_block(10, 10, 10, dirt.clone());
        schematic.add_region(custom_region);

        // Test swapping default region
        assert!(schematic.swap_default_region("Custom").is_ok());
        assert_eq!(schematic.default_region_name, "Custom");

        // Verify the swap worked
        assert_eq!(schematic.get_block(10, 10, 10), Some(&dirt));
        assert_eq!(
            schematic.get_block_from_region("Main", 0, 0, 0),
            Some(&stone)
        );

        // Test swapping with non-existent region
        assert!(schematic.swap_default_region("NonExistent").is_err());
    }

    #[test]
    fn test_set_default_region() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        // Create a new region
        let mut new_region = Region::new("NewDefault".to_string(), (5, 5, 5), (3, 3, 3));
        let gold = BlockState::new("minecraft:gold_block".to_string());
        new_region.set_block(5, 5, 5, gold.clone());

        // Set it as the default
        let old_default = schematic.set_default_region(new_region);

        // Check that the default region name was updated
        assert_eq!(schematic.default_region_name, "NewDefault");

        // Check that the new default region is working
        assert_eq!(schematic.get_block(5, 5, 5), Some(&gold));

        // Check that the old default was returned
        assert_eq!(old_default.name, "Main");
    }

    #[test]
    fn test_bounding_box_and_dimensions() {
        let mut schematic = UniversalSchematic::new("Test Bounding Box".to_string());

        schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
        schematic.set_block(
            4,
            4,
            4,
            BlockState::new("minecraft:sea_lantern".to_string()),
        );

        let bbox = schematic.get_bounding_box();

        // With hybrid approach, expect aggressive expansion
        assert_eq!(bbox.min, (0, 0, 0));
        assert_eq!(bbox.max, (68, 68, 68)); // Now expects 68 instead of 4

        // Don't test exact dimensions as they depend on expansion strategy
        let dimensions = schematic.get_dimensions();
        assert!(dimensions.0 >= 5 && dimensions.1 >= 5 && dimensions.2 >= 5);
    }
    #[test]
    fn test_schematic_large_coordinates() {
        let mut schematic = UniversalSchematic::new("Large Schematic".to_string());

        let far_block = BlockState::new("minecraft:diamond_block".to_string());
        assert!(schematic.set_block(1000, 1000, 1000, far_block.clone()));
        assert_eq!(schematic.get_block(1000, 1000, 1000), Some(&far_block));

        let main_region = schematic.default_region.clone();
        assert_eq!(main_region.position, (1000, 1000, 1000));
        assert_eq!(main_region.size, (1, 1, 1));

        // Test that blocks outside the region are not present
        assert_eq!(schematic.get_block(999, 1000, 1000), None);
        assert_eq!(schematic.get_block(1002, 1000, 1000), None);
    }

    #[test]
    fn test_schematic_region_expansion() {
        let mut schematic = UniversalSchematic::new("Expanding Schematic".to_string());

        let block1 = BlockState::new("minecraft:stone".to_string());
        let block2 = BlockState::new("minecraft:dirt".to_string());

        assert!(schematic.set_block(0, 0, 0, block1.clone()));
        assert!(schematic.set_block(10, 20, 30, block2.clone()));

        let main_region = schematic.get_region("Main").unwrap();
        assert_eq!(main_region.position, (0, 0, 0));

        assert_eq!(schematic.get_block(0, 0, 0), Some(&block1));
        assert_eq!(schematic.get_block(10, 20, 30), Some(&block2));
        assert_eq!(
            schematic.get_block(5, 10, 15),
            Some(&BlockState::new("minecraft:air".to_string()))
        );
    }

    #[test]
    fn test_copy_bounded_region() {
        // Create source schematic
        let mut source = UniversalSchematic::new("Source".to_string());

        // Add some blocks in a pattern
        source.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
        source.set_block(1, 1, 1, BlockState::new("minecraft:dirt".to_string()));
        source.set_block(
            2,
            2,
            2,
            BlockState::new("minecraft:diamond_block".to_string()),
        );

        // Add a block entity
        let chest = BlockEntity::create_chest(
            (1, 1, 1),
            vec![ItemStack::new("minecraft:diamond", 64).with_slot(0)],
        );
        source.set_block_entity(BlockPosition { x: 1, y: 1, z: 1 }, chest);

        // Add an entity
        let entity = Entity::new("minecraft:creeper".to_string(), (1.5, 1.0, 1.5));
        source.add_entity(entity);

        // Create target schematic
        let mut target = UniversalSchematic::new("Target".to_string());

        // Define a bounding box that includes part of the pattern
        let bounds = BoundingBox::new((0, 0, 0), (1, 1, 1));

        // Copy to new position
        assert!(target
            .copy_region(&source, &bounds, (10, 10, 10), &[])
            .is_ok());

        // Verify copied blocks
        assert_eq!(
            target.get_block(10, 10, 10).unwrap().get_name(),
            "minecraft:stone"
        );
        assert_eq!(
            target.get_block(11, 11, 11).unwrap().get_name(),
            "minecraft:dirt"
        );

        // Block at (2, 2, 2) should not have been copied as it's outside bounds
        assert!(target.get_block(12, 12, 12).is_none());

        // Verify block entity was copied and moved
        assert!(target
            .get_block_entity(BlockPosition {
                x: 11,
                y: 11,
                z: 11
            })
            .is_some());

        // Verify entity was copied and moved
        assert_eq!(target.default_region.entities.len(), 1);
        assert_eq!(
            target.default_region.entities[0].position,
            (11.5, 11.0, 11.5)
        );
    }

    #[test]
    fn test_copy_region_excluded_blocks() {
        // Create source schematic
        let mut source = UniversalSchematic::new("Source".to_string());

        // Add blocks in a pattern including blocks we'll want to exclude
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());
        let diamond = BlockState::new("minecraft:diamond_block".to_string());
        let air = BlockState::new("minecraft:air".to_string());

        // Create a 2x2x2 cube with different blocks
        source.set_block(0, 0, 0, stone.clone());
        source.set_block(0, 1, 0, dirt.clone());
        source.set_block(1, 0, 0, diamond.clone());
        source.set_block(1, 1, 0, dirt.clone());

        // Create target schematic
        let mut target = UniversalSchematic::new("Target".to_string());

        // Define bounds that include all blocks
        let bounds = BoundingBox::new((0, 0, 0), (1, 1, 0));

        // List of blocks to exclude (stone and diamond)
        let excluded_blocks = vec![stone.clone(), diamond.clone()];

        // Copy region with exclusions to position (10, 10, 10)
        assert!(target
            .copy_region(&source, &bounds, (10, 10, 10), &excluded_blocks)
            .is_ok());

        // Test some specific positions
        // Where dirt blocks were in source (should be copied)
        assert_eq!(
            target.get_block(10, 11, 10),
            Some(&dirt),
            "Dirt block should be copied at (10, 11, 10)"
        );
        assert_eq!(
            target.get_block(11, 11, 10),
            Some(&dirt),
            "Dirt block should be copied at (11, 11, 10)"
        );

        // Check that excluded blocks were not copied (they should be air within the expanded region)
        assert_eq!(
            target.get_block(10, 10, 10),
            Some(&air),
            "Stone block should not be copied at (10, 10, 10) - should be air"
        );
        assert_eq!(
            target.get_block(11, 10, 10),
            Some(&air),
            "Diamond block should not be copied at (11, 10, 10) - should be air"
        );

        // Count the total number of dirt blocks
        let dirt_blocks: Vec<_> = target
            .get_blocks()
            .into_iter()
            .filter(|b| b == &dirt)
            .collect();

        assert_eq!(dirt_blocks.len(), 2, "Should have exactly 2 dirt blocks");
    }
    #[test]
    fn test_schematic_negative_coordinates() {
        let mut schematic = UniversalSchematic::new("Negative Coordinates Schematic".to_string());

        let neg_block = BlockState::new("minecraft:emerald_block".to_string());
        assert!(schematic.set_block(-10, -10, -10, neg_block.clone()));
        assert_eq!(schematic.get_block(-10, -10, -10), Some(&neg_block));

        let main_region = schematic.get_region("Main").unwrap();
        assert!(
            main_region.position.0 <= -10
                && main_region.position.1 <= -10
                && main_region.position.2 <= -10
        );
    }

    #[test]
    fn test_entity_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let entity = Entity::new("minecraft:creeper".to_string(), (10.5, 65.0, 20.5))
            .with_nbt_data("Fuse".to_string(), "30".to_string());

        assert!(schematic.add_entity(entity.clone()));

        assert_eq!(schematic.default_region.entities.len(), 1);
        assert_eq!(schematic.default_region.entities[0], entity);

        let removed_entity = schematic.remove_entity(0).unwrap();
        assert_eq!(removed_entity, entity);

        assert_eq!(schematic.default_region.entities.len(), 0);
    }

    #[test]
    fn test_block_entity_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let chest = BlockEntity::create_chest(
            (5, 10, 15),
            vec![ItemStack::new("minecraft:diamond", 64).with_slot(0)],
        );

        assert!(schematic.add_block_entity(chest.clone()));

        assert_eq!(schematic.default_region.block_entities.len(), 1);
        assert_eq!(
            schematic.default_region.block_entities.get(&(5, 10, 15)),
            Some(&chest)
        );

        let removed_block_entity = schematic.remove_block_entity((5, 10, 15)).unwrap();
        assert_eq!(removed_block_entity, chest);

        assert_eq!(schematic.default_region.block_entities.len(), 0);
    }

    #[test]
    fn test_block_entity_helper_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let diamond = ItemStack::new("minecraft:diamond", 64).with_slot(0);
        let chest = BlockEntity::create_chest((5, 10, 15), vec![diamond]);

        assert!(schematic.add_block_entity(chest.clone()));

        assert_eq!(schematic.default_region.block_entities.len(), 1);
        assert_eq!(
            schematic.default_region.block_entities.get(&(5, 10, 15)),
            Some(&chest)
        );

        let removed_block_entity = schematic.remove_block_entity((5, 10, 15)).unwrap();
        assert_eq!(removed_block_entity, chest);

        assert_eq!(schematic.default_region.block_entities.len(), 0);
    }

    #[test]
    fn test_block_entity_in_region_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let chest = BlockEntity::create_chest(
            (5, 10, 15),
            vec![ItemStack::new("minecraft:diamond", 64).with_slot(0)],
        );
        assert!(schematic.add_block_entity_in_region("Main", chest.clone()));

        assert_eq!(schematic.default_region.block_entities.len(), 1);
        assert_eq!(
            schematic.default_region.block_entities.get(&(5, 10, 15)),
            Some(&chest)
        );

        let removed_block_entity = schematic
            .remove_block_entity_in_region("Main", (5, 10, 15))
            .unwrap();
        assert_eq!(removed_block_entity, chest);

        assert_eq!(schematic.default_region.block_entities.len(), 0);
    }

    #[test]
    fn test_set_block_from_string() {
        let mut schematic = UniversalSchematic::new("Test".to_string());

        // Test simple block
        assert!(schematic
            .set_block_from_string(0, 0, 0, "minecraft:stone")
            .unwrap());

        // Test block with properties
        assert!(schematic
            .set_block_from_string(1, 0, 0, "minecraft:chest[facing=north]")
            .unwrap());

        // Test container with items
        let barrel_str = r#"minecraft:barrel[facing=up]{CustomName:'{"text":"Storage"}',Items:[{Count:64b,Slot:0b,id:"minecraft:redstone"}]}"#;
        assert!(schematic
            .set_block_from_string(2, 0, 0, barrel_str)
            .unwrap());

        // Verify the blocks were set correctly
        assert_eq!(
            schematic.get_block(0, 0, 0).unwrap().get_name(),
            "minecraft:stone"
        );
        assert_eq!(
            schematic.get_block(1, 0, 0).unwrap().get_name(),
            "minecraft:chest"
        );
        assert_eq!(
            schematic.get_block(2, 0, 0).unwrap().get_name(),
            "minecraft:barrel"
        );

        // Verify container contents
        let barrel_entity = schematic
            .get_block_entity(BlockPosition { x: 2, y: 0, z: 0 })
            .unwrap();
        let items = barrel_entity.nbt.get("Items").unwrap();
        if let NbtValue::List(items) = items {
            assert_eq!(items.len(), 1);
            if let NbtValue::Compound(item) = &items[0] {
                assert_eq!(
                    item.get("id").unwrap(),
                    &NbtValue::String("minecraft:redstone".to_string())
                );
                // Modern format uses lowercase 'count' as Int
                assert_eq!(item.get("count").unwrap(), &NbtValue::Int(64));
                assert_eq!(item.get("Slot").unwrap(), &NbtValue::Byte(0));
            } else {
                panic!("Expected compound NBT value");
            }
        } else {
            panic!("Expected list of items");
        }
    }

    #[test]
    fn test_region_palette_operations() {
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (2, 2, 2));

        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        region.set_block(0, 0, 0, stone.clone());
        region.set_block(0, 1, 0, dirt.clone());
        region.set_block(1, 0, 0, stone.clone());

        assert_eq!(region.get_block(0, 0, 0), Some(&stone));
        assert_eq!(region.get_block(0, 1, 0), Some(&dirt));
        assert_eq!(region.get_block(1, 0, 0), Some(&stone));
        assert_eq!(
            region.get_block(1, 1, 1),
            Some(&BlockState::new("minecraft:air".to_string()))
        );

        // Check the palette size
        assert_eq!(region.palette.len(), 3); // air, stone, dirt
    }

    #[test]
    fn test_nbt_serialization_deserialization() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        // Add some blocks and entities
        schematic.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
        schematic.set_block(1, 1, 1, BlockState::new("minecraft:dirt".to_string()));
        schematic.add_entity(Entity::new(
            "minecraft:creeper".to_string(),
            (0.5, 0.0, 0.5),
        ));

        // Serialize to NBT
        let nbt = schematic.to_nbt();

        // Write NBT to a buffer
        let mut buffer = Vec::new();
        write_nbt(
            &mut buffer,
            None,
            &nbt,
            quartz_nbt::io::Flavor::Uncompressed,
        )
        .unwrap();

        // Read NBT from the buffer
        let (read_nbt, _) = read_nbt(
            &mut Cursor::new(buffer),
            quartz_nbt::io::Flavor::Uncompressed,
        )
        .unwrap();

        // Deserialize from NBT
        let deserialized_schematic = UniversalSchematic::from_nbt(read_nbt).unwrap();

        // Compare original and deserialized schematics
        assert_eq!(schematic.metadata, deserialized_schematic.metadata);
        assert_eq!(
            schematic.other_regions.len(),
            deserialized_schematic.other_regions.len()
        );

        // Check if blocks are correctly deserialized
        assert_eq!(
            schematic.get_block(0, 0, 0),
            deserialized_schematic.get_block(0, 0, 0)
        );
        assert_eq!(
            schematic.get_block(1, 1, 1),
            deserialized_schematic.get_block(1, 1, 1)
        );

        // Check if entities are correctly deserialized
        let original_entities = schematic.default_region.entities.clone();
        let deserialized_entities = deserialized_schematic.default_region.entities.clone();
        assert_eq!(original_entities, deserialized_entities);

        // Check if palettes are correctly deserialized
        let original_palette = schematic.default_region.get_palette_nbt().clone();
        let deserialized_palette = deserialized_schematic
            .default_region
            .get_palette_nbt()
            .clone();
        assert_eq!(original_palette, deserialized_palette);
    }

    #[test]
    fn test_multiple_region_merging() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        let mut region1 = Region::new("Region1".to_string(), (0, 0, 0), (2, 2, 2));
        let mut region2 = Region::new("Region4".to_string(), (0, 0, 0), (-2, -2, -2));

        // Add some blocks to the regions
        region1.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
        region1.set_block(1, 1, 1, BlockState::new("minecraft:dirt".to_string()));
        region2.set_block(
            0,
            -1,
            -1,
            BlockState::new("minecraft:gold_block".to_string()),
        );

        // Add a block to the default region
        schematic.set_block(
            2,
            2,
            2,
            BlockState::new("minecraft:diamond_block".to_string()),
        );

        schematic.add_region(region1);
        schematic.add_region(region2);

        let merged_region = schematic.get_merged_region();

        assert_eq!(merged_region.count_blocks(), 4); // 3 from added regions + 1 from default
        assert_eq!(
            merged_region.get_block(0, 0, 0),
            Some(&BlockState::new("minecraft:stone".to_string()))
        );
        assert_eq!(
            merged_region.get_block(1, 1, 1),
            Some(&BlockState::new("minecraft:dirt".to_string()))
        );
        assert_eq!(
            merged_region.get_block(2, 2, 2),
            Some(&BlockState::new("minecraft:diamond_block".to_string()))
        );
    }

    #[test]
    fn test_calculate_items_for_signal() {
        assert_eq!(UniversalSchematic::calculate_items_for_signal(0), 0);
        assert_eq!(UniversalSchematic::calculate_items_for_signal(1), 1);
        assert_eq!(UniversalSchematic::calculate_items_for_signal(15), 1728); // Full barrel
    }

    #[test]
    fn test_barrel_signal_strength() {
        let mut schematic = UniversalSchematic::new("Test".to_string());

        // Test simple signal strength
        let barrel_str = "minecraft:barrel{signal=13}";
        assert!(schematic
            .set_block_from_string(0, 0, 0, barrel_str)
            .unwrap());

        // log the palette for debugging
        println!("Palette: {:?}", schematic.default_region.palette);
        println!(
            "Block Entities: {:?}",
            schematic.default_region.block_entities
        );

        let barrel_entity = schematic
            .get_block_entity(BlockPosition { x: 0, y: 0, z: 0 })
            .unwrap();
        let items = barrel_entity.nbt.get("Items").unwrap();
        println!("Items NBT: {:?}", items);
        if let NbtValue::List(items) = items {
            // Calculate expected total items
            let mut total_items = 0;
            for item in items {
                if let NbtValue::Compound(item_map) = item {
                    // Check for modern format: lowercase 'count' as Int (1.20.5+)
                    if let Some(NbtValue::Int(count)) = item_map.get("count") {
                        total_items += *count as u32;
                    }
                    // Also check for legacy format: uppercase 'Count' as Byte (backward compatibility)
                    else if let Some(NbtValue::Byte(count)) = item_map.get("Count") {
                        total_items += *count as u32;
                    }
                }
            }

            // Verify the total items matches what's needed for signal strength 13
            let expected_items = UniversalSchematic::calculate_items_for_signal(13);
            assert_eq!(total_items as u32, expected_items);
        }

        // Test invalid signal strength
        let invalid_barrel = "minecraft:barrel{signal=16}";
        assert!(schematic
            .set_block_from_string(1, 0, 0, invalid_barrel)
            .is_err());
    }

    #[test]
    fn test_barrel_with_properties_and_signal() {
        let mut schematic = UniversalSchematic::new("Test".to_string());

        let barrel_str = "minecraft:barrel[facing=up]{signal=7}";
        assert!(schematic
            .set_block_from_string(0, 0, 0, barrel_str)
            .unwrap());

        // Verify the block state properties
        let block = schematic.get_block(0, 0, 0).unwrap();
        assert_eq!(block.get_property("facing"), Some(&"up".to_string()));

        // Verify the signal strength items
        let barrel_entity = schematic
            .get_block_entity(BlockPosition { x: 0, y: 0, z: 0 })
            .unwrap();
        let items = barrel_entity.nbt.get("Items").unwrap();
        if let NbtValue::List(items) = items {
            let mut total_items = 0;
            for item in items {
                if let NbtValue::Compound(item_map) = item {
                    // Check for modern format: lowercase 'count' as Int (1.20.5+)
                    if let Some(NbtValue::Int(count)) = item_map.get("count") {
                        total_items += *count as u32;
                    }
                    // Also check for legacy format: uppercase 'Count' as Byte (backward compatibility)
                    else if let Some(NbtValue::Byte(count)) = item_map.get("Count") {
                        total_items += *count as u32;
                    }
                }
            }
            let expected_items = UniversalSchematic::calculate_items_for_signal(7);
            assert_eq!(total_items as u32, expected_items);
        }
    }

    #[test]
    fn test_chunk_consistency() {
        let mut schematic = UniversalSchematic::new("Chunk Test".to_string());

        // Add some non-air blocks in a pattern
        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    if (x + y + z) % 3 == 0 {
                        // Only set some blocks, not all
                        schematic.set_block(
                            x,
                            y,
                            z,
                            BlockState::new("minecraft:stone".to_string()),
                        );
                    }
                }
            }
        }

        let chunk_width = 16;
        let chunk_height = 16;
        let chunk_length = 16;

        // Count chunks using both methods
        let chunks: Vec<_> = schematic
            .iter_chunks(
                chunk_width,
                chunk_height,
                chunk_length,
                Some(ChunkLoadingStrategy::BottomUp),
            )
            .collect();

        let chunks_indices: Vec<_> = schematic
            .iter_chunks_indices(
                chunk_width,
                chunk_height,
                chunk_length,
                Some(ChunkLoadingStrategy::BottomUp),
            )
            .collect();

        // They should now be equal since both exclude air blocks
        assert_eq!(
            chunks.len(),
            chunks_indices.len(),
            "Chunk counts should be consistent between iter_chunks and iter_chunks_indices"
        );

        // Verify both methods return the same number of non-empty chunks
        assert!(
            chunks.len() > 0,
            "Should have at least one chunk with blocks"
        );
        assert_eq!(
            chunks.len(),
            1,
            "Should have exactly one chunk for this small test case"
        );
    }

    #[test]
    fn test_exact_chunk_dimensions() {
        // Test case 1: 16x16x16 cube with 16x16x16 chunks should produce exactly 1 chunk
        let mut schematic = UniversalSchematic::new("Exact Chunk Test".to_string());

        // Fill a 16x16x16 cube with blocks
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    schematic.set_block(x, y, z, BlockState::new("minecraft:stone".to_string()));
                }
            }
        }

        let chunks: Vec<_> = schematic.iter_chunks(16, 16, 16, None).collect();
        let chunks_indices: Vec<_> = schematic.iter_chunks_indices(16, 16, 16, None).collect();

        assert_eq!(
            chunks.len(),
            1,
            "16x16x16 cube with 16x16x16 chunks should produce exactly 1 chunk"
        );
        assert_eq!(
            chunks_indices.len(),
            1,
            "iter_chunks_indices should also produce exactly 1 chunk"
        );
        assert_eq!(
            chunks[0].chunk_x, 0,
            "Chunk should be at coordinate (0, 0, 0)"
        );
        assert_eq!(
            chunks[0].chunk_y, 0,
            "Chunk should be at coordinate (0, 0, 0)"
        );
        assert_eq!(
            chunks[0].chunk_z, 0,
            "Chunk should be at coordinate (0, 0, 0)"
        );

        // Test case 2: 64x16x16 cube with 16x16x16 chunks should produce exactly 4 chunks
        let mut schematic2 = UniversalSchematic::new("4 Chunk Test".to_string());

        // Fill a 64x16x16 cube with blocks
        for x in 0..64 {
            for y in 0..16 {
                for z in 0..16 {
                    schematic2.set_block(x, y, z, BlockState::new("minecraft:stone".to_string()));
                }
            }
        }

        let chunks2: Vec<_> = schematic2.iter_chunks(16, 16, 16, None).collect();
        let chunks_indices2: Vec<_> = schematic2.iter_chunks_indices(16, 16, 16, None).collect();

        assert_eq!(
            chunks2.len(),
            4,
            "64x16x16 cube with 16x16x16 chunks should produce exactly 4 chunks"
        );
        assert_eq!(
            chunks_indices2.len(),
            4,
            "iter_chunks_indices should also produce exactly 4 chunks"
        );

        // Verify chunk coordinates are correct (should be at x=0,1,2,3 and y=0, z=0)
        let mut chunk_x_coords: Vec<i32> = chunks2.iter().map(|c| c.chunk_x).collect();
        chunk_x_coords.sort();
        assert_eq!(
            chunk_x_coords,
            vec![0, 1, 2, 3],
            "Chunks should be at x coordinates 0, 1, 2, 3"
        );

        // All chunks should be at y=0, z=0
        for chunk in &chunks2 {
            assert_eq!(chunk.chunk_y, 0, "All chunks should be at y=0");
            assert_eq!(chunk.chunk_z, 0, "All chunks should be at z=0");
        }

        // Test case 3: 32x32x32 cube with 16x16x16 chunks should produce exactly 8 chunks
        let mut schematic3 = UniversalSchematic::new("8 Chunk Test".to_string());

        // Fill a 32x32x32 cube with blocks
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    schematic3.set_block(x, y, z, BlockState::new("minecraft:stone".to_string()));
                }
            }
        }

        let chunks3: Vec<_> = schematic3.iter_chunks(16, 16, 16, None).collect();
        let chunks_indices3: Vec<_> = schematic3.iter_chunks_indices(16, 16, 16, None).collect();

        assert_eq!(
            chunks3.len(),
            8,
            "32x32x32 cube with 16x16x16 chunks should produce exactly 8 chunks"
        );
        assert_eq!(
            chunks_indices3.len(),
            8,
            "iter_chunks_indices should also produce exactly 8 chunks"
        );

        // Test case 4: Sparse blocks should still chunk correctly
        let mut schematic4 = UniversalSchematic::new("Sparse Chunk Test".to_string());

        // Place blocks only at corners of a 32x32x32 space
        schematic4.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
        schematic4.set_block(31, 0, 0, BlockState::new("minecraft:stone".to_string()));
        schematic4.set_block(0, 31, 0, BlockState::new("minecraft:stone".to_string()));
        schematic4.set_block(0, 0, 31, BlockState::new("minecraft:stone".to_string()));
        schematic4.set_block(31, 31, 31, BlockState::new("minecraft:stone".to_string()));

        let chunks4: Vec<_> = schematic4.iter_chunks(16, 16, 16, None).collect();
        let chunks_indices4: Vec<_> = schematic4.iter_chunks_indices(16, 16, 16, None).collect();

        // Should have chunks at different coordinates due to sparse placement
        assert_eq!(
            chunks4.len(),
            chunks_indices4.len(),
            "Both methods should produce same number of chunks for sparse blocks"
        );
        assert!(
            chunks4.len() <= 8,
            "Should not exceed 8 chunks for blocks in 32x32x32 space"
        );
        assert!(
            chunks4.len() > 0,
            "Should have at least one chunk with blocks"
        );
    }

    #[test]
    fn test_set_block_with_nbt_sign() {
        let mut schematic = UniversalSchematic::new("test_schematic".to_string());
        let mut nbt = HashMap::new();
        nbt.insert("Text1".to_string(), r#"{"text":"Hello"}"#.to_string());
        nbt.insert("Text2".to_string(), r#"{"text":"World"}"#.to_string());
        nbt.insert("Text3".to_string(), r#"{"text":"Line 3"}"#.to_string());
        nbt.insert("Text4".to_string(), r#"{"text":"Line 4"}"#.to_string());

        let result = schematic.set_block_with_nbt(0, 1, 0, "minecraft:oak_sign[rotation=0]", nbt);
        assert!(result.is_ok());

        // Verify block was set
        let block = schematic.get_block(0, 1, 0);
        assert!(block.is_some());
        assert_eq!(block.unwrap().name, "minecraft:oak_sign");

        // Verify block entity was created
        let entity = schematic.get_block_entity(BlockPosition { x: 0, y: 1, z: 0 });
        assert!(entity.is_some());

        let entity = entity.unwrap();
        assert_eq!(entity.id, "minecraft:oak_sign");
        assert!(entity.nbt.get("Text1").is_some());
        assert!(entity.nbt.get("Text2").is_some());
    }

    #[test]
    fn test_set_block_with_nbt_chest() {
        let mut schematic = UniversalSchematic::new("test_schematic".to_string());
        let mut nbt = HashMap::new();
        nbt.insert(
            "CustomName".to_string(),
            r#"{"text":"My Chest"}"#.to_string(),
        );
        nbt.insert("Lock".to_string(), "secret_key".to_string());

        let result = schematic.set_block_with_nbt(5, 2, 3, "minecraft:chest[facing=north]", nbt);
        assert!(result.is_ok());

        // Verify block was set
        let block = schematic.get_block(5, 2, 3);
        assert!(block.is_some());
        assert_eq!(block.unwrap().name, "minecraft:chest");

        // Verify block entity
        let entity = schematic.get_block_entity(BlockPosition { x: 5, y: 2, z: 3 });
        assert!(entity.is_some());

        let entity = entity.unwrap();
        assert_eq!(entity.id, "minecraft:chest");
        assert!(entity.nbt.get("CustomName").is_some());
        assert!(entity.nbt.get("Lock").is_some());
    }

    #[test]
    fn test_set_block_with_nbt_furnace() {
        let mut schematic = UniversalSchematic::new("test_schematic".to_string());
        let mut nbt = HashMap::new();
        nbt.insert("BurnTime".to_string(), "200".to_string());
        nbt.insert("CookTime".to_string(), "100".to_string());

        let result = schematic.set_block_with_nbt(10, 5, 10, "minecraft:furnace[lit=true]", nbt);
        assert!(result.is_ok());

        // Verify block entity has numeric NBT values
        let entity = schematic.get_block_entity(BlockPosition { x: 10, y: 5, z: 10 });
        assert!(entity.is_some());

        let entity = entity.unwrap();
        assert!(entity.nbt.get("BurnTime").is_some());
        assert!(entity.nbt.get("CookTime").is_some());
    }

    #[test]
    fn test_set_block_with_nbt_empty_nbt() {
        let mut schematic = UniversalSchematic::new("test_schematic".to_string());
        let nbt = HashMap::new();

        let result = schematic.set_block_with_nbt(0, 0, 0, "minecraft:stone", nbt);
        assert!(result.is_ok());

        // Verify block was set
        let block = schematic.get_block(0, 0, 0);
        assert!(block.is_some());
        assert_eq!(block.unwrap().name, "minecraft:stone");

        // Should still create a block entity (even if empty)
        let entity = schematic.get_block_entity(BlockPosition { x: 0, y: 0, z: 0 });
        assert!(entity.is_some());
    }

    #[test]
    fn test_set_block_with_nbt_multiple_blocks() {
        let mut schematic = UniversalSchematic::new("test_schematic".to_string());

        // Set multiple signs with different NBT data
        for i in 0..3 {
            let mut nbt = HashMap::new();
            nbt.insert("Text1".to_string(), format!(r#"{{"text":"Sign {i}"}}"#));

            let result = schematic.set_block_with_nbt(i, 0, 0, "minecraft:oak_sign", nbt);
            assert!(result.is_ok());
        }

        // Verify all blocks and entities were created
        for i in 0..3 {
            let block = schematic.get_block(i, 0, 0);
            assert!(block.is_some());

            let entity = schematic.get_block_entity(BlockPosition { x: i, y: 0, z: 0 });
            assert!(entity.is_some());
            assert!(entity.unwrap().nbt.get("Text1").is_some());
        }
    }

    #[test]
    fn test_parse_nbt_value() {
        // Test JSON string (should stay as string)
        let json_value = UniversalSchematic::parse_nbt_value(r#"{"text":"Hello"}"#);
        match json_value {
            NbtValue::String(s) => assert!(s.contains("Hello")),
            _ => panic!("Expected String variant for JSON"),
        }

        // Test integer
        let int_value = UniversalSchematic::parse_nbt_value("42");
        match int_value {
            NbtValue::Int(i) => assert_eq!(i, 42),
            _ => panic!("Expected Int variant"),
        }

        // Test float
        let float_value = UniversalSchematic::parse_nbt_value("3.14");
        match float_value {
            NbtValue::Float(f) => assert!((f - 3.14).abs() < 0.01),
            _ => panic!("Expected Float variant"),
        }

        // Test boolean
        let bool_value = UniversalSchematic::parse_nbt_value("true");
        match bool_value {
            NbtValue::Byte(b) => assert_eq!(b, 1),
            _ => panic!("Expected Byte variant for boolean"),
        }

        // Test plain string
        let string_value = UniversalSchematic::parse_nbt_value("plain text");
        match string_value {
            NbtValue::String(s) => assert_eq!(s, "plain text"),
            _ => panic!("Expected String variant"),
        }
    }
}
