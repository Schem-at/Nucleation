use crate::block_entity::BlockEntity;
use crate::block_position::BlockPosition;
use crate::bounding_box::BoundingBox;
use crate::chunk::Chunk;
use crate::definition_region::DefinitionRegion;
use crate::entity::Entity;
use crate::metadata::Metadata;
use crate::region::Region;
// use crate::utils::block_string::{parse_custom_name, parse_items_array};
// use crate::utils::enhanced_nbt_parser::parse_enhanced_nbt;
use crate::utils::NbtMap;
use crate::utils::NbtValue;
use crate::BlockState;
use quartz_nbt::{NbtCompound, NbtTag};
use rand::SeedableRng;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct UniversalSchematic {
    pub metadata: Metadata,
    pub default_region: Region,
    pub other_regions: HashMap<String, Region>,
    pub default_region_name: String,
    #[serde(default = "HashMap::new")]
    pub definition_regions: HashMap<String, DefinitionRegion>,
    /// String-keyed cache used by `set_block_str` to skip BlockState
    /// allocation on repeated placements of the same plain id. FxHashMap is
    /// the right hasher here — keys are short ASCII strings and we hit this
    /// on every set_block call.
    #[serde(skip, default = "FxHashMap::default")]
    block_state_cache: FxHashMap<String, BlockState>,
    /// Full-block-string cache used by `set_block_from_string` to skip
    /// property and NBT parsing on repeated placements of the same string
    /// (e.g. filling 100k identical chests). The NbtMap is Arc-shared with
    /// every BlockEntity placed from it; `BlockEntity::nbt_mut` copies on
    /// write, so sharing is invisible to callers.
    #[serde(skip, default = "FxHashMap::default")]
    block_string_cache: FxHashMap<String, (BlockState, Option<std::sync::Arc<NbtMap>>)>,
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
            definition_regions: HashMap::new(),
            block_state_cache: FxHashMap::default(),
            block_string_cache: FxHashMap::default(),
        }
    }

    /// Convert this schematic's block states, block entities, and items to
    /// `target_data_version`, returning a [`LossReport`](crate::dataconverter::LossReport)
    /// describing any data loss (empty when lossless).
    ///
    /// The `from` version is `metadata.source_data_version` (captured by
    /// importers), falling back to `mc_version`, then the canonical version.
    /// Forward (`target >= from`) is faithful and lossless; reverse
    /// (`target < from`, i.e. saving for an older version) may approximate and
    /// reports every loss so callers can warn before writing. The metadata
    /// version fields are updated to `target` so a subsequent export stamps it.
    pub fn convert_to_data_version(
        &mut self,
        target_data_version: i32,
    ) -> crate::dataconverter::LossReport {
        use crate::dataconverter::{
            convert_schematic, convert_schematic_reverse, LossReport, CANONICAL_DATA_VERSION,
        };
        let from = self
            .metadata
            .source_data_version
            .or(self.metadata.mc_version)
            .unwrap_or(CANONICAL_DATA_VERSION);

        let report = if target_data_version == from {
            LossReport::default()
        } else if target_data_version > from {
            convert_schematic(self, from, target_data_version);
            LossReport::default()
        } else {
            convert_schematic_reverse(self, from, target_data_version)
        };

        self.metadata.mc_version = Some(target_data_version);
        self.metadata.source_data_version = Some(target_data_version);
        report
    }

    /// Forward-convert to the canonical (in-memory target) data version. A
    /// convenience for load-time normalization; lossless.
    pub fn convert_to_canonical(&mut self) {
        let _ = self.convert_to_data_version(crate::dataconverter::CANONICAL_DATA_VERSION);
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

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: &BlockState) -> bool {
        // Check if the default region is empty and needs repositioning
        // Optimization: Only check if region is small (1x1x1), otherwise assume it's initialized
        if self.default_region.size == (1, 1, 1) && self.default_region.is_empty() {
            // Reposition the default region to the first block's location
            self.default_region =
                Region::new(self.default_region_name.clone(), (x, y, z), (1, 1, 1));
        }

        self.default_region.set_block(x, y, z, block)
    }

    pub fn set_block_str(&mut self, x: i32, y: i32, z: i32, block_name: &str) -> bool {
        self.try_set_block_str(x, y, z, block_name).unwrap_or(false)
    }

    /// Fallible string-based block placement for language adapters and callers
    /// that need parser errors instead of the boolean convenience fallback.
    pub fn try_set_block_str(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
    ) -> Result<bool, String> {
        // Any delimiter routes through the strict parser, including unmatched
        // closing delimiters that must fail instead of becoming part of an id.
        if block_name
            .chars()
            .any(|c| matches!(c, '[' | ']' | '{' | '}'))
        {
            self.set_block_from_string(x, y, z, block_name)
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

            let placed = self.set_block(x, y, z, &block_state);
            if placed {
                self.remove_block_entity((x, y, z));
            }
            Ok(placed)
        }
    }

    pub fn set_block_in_region(
        &mut self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
        block: &BlockState,
    ) -> bool {
        if region_name == self.default_region_name {
            if self.default_region.size == (1, 1, 1) && self.default_region.is_empty() {
                self.default_region =
                    Region::new(self.default_region_name.clone(), (x, y, z), (1, 1, 1));
            }
            self.default_region.set_block(x, y, z, block)
        } else {
            let region = self
                .other_regions
                .entry(region_name.to_string())
                .or_insert_with(|| Region::new(region_name.to_string(), (x, y, z), (1, 1, 1)));
            if region.size == (1, 1, 1) && region.is_empty() {
                *region = Region::new(region_name.to_string(), (x, y, z), (1, 1, 1));
            }
            region.set_block(x, y, z, block)
        }
    }

    /// Ensure the default region covers the given bounds.
    pub fn ensure_bounds(&mut self, min: (i32, i32, i32), max: (i32, i32, i32)) {
        if self.default_region.is_empty() {
            // If empty, just set it to the bounds
            let size = (
                (max.0 - min.0 + 1).max(1),
                (max.1 - min.1 + 1).max(1),
                (max.2 - min.2 + 1).max(1),
            );
            self.default_region = Region::new(self.default_region_name.clone(), min, size);
        } else {
            self.default_region.ensure_bounds(min, max);
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
            .unwrap_or_default()
    }

    pub fn try_set_block_in_region_str(
        &mut self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
    ) -> Result<bool, String> {
        if region_name.is_empty() {
            return Err("Region name cannot be empty".to_string());
        }

        let (block_state, nbt) = self.parse_block_string_cached(block_name)?;
        if !self.set_block_in_region(region_name, x, y, z, &block_state) {
            return Ok(false);
        }

        self.remove_block_entity_in_region(region_name, (x, y, z));
        if let Some(nbt) = nbt {
            let block_entity = BlockEntity {
                id: block_state.name.to_string(),
                position: (x, y, z),
                nbt,
            };
            self.set_block_entity_in_region(region_name, BlockPosition { x, y, z }, block_entity);
        }

        Ok(true)
    }

    pub fn set_block_in_region_str(
        &mut self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
        block_name: &str,
    ) -> bool {
        self.try_set_block_in_region_str(region_name, x, y, z, block_name)
            .unwrap_or(false)
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
                        schematic.set_block(x as i32, y as i32, z as i32, block_state);
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
                        .map(|&(k, v)| (SmolStr::from(k), SmolStr::from(v)))
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

        // Check named regions in stable lexicographic precedence.
        for region in Self::sorted_named_regions(self) {
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

        // Check named regions in stable lexicographic precedence.
        for region in Self::sorted_named_regions(self) {
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
        for region in Self::sorted_named_regions(self) {
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
        if !self.set_block(x, y, z, &block_state) {
            return Ok(false);
        }

        // Create block entity with NBT data
        let mut block_entity = BlockEntity::new(block_state.name.to_string(), (x, y, z));

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
            blocks.push(default_palette[*block_index].clone());
        }

        // Add blocks from named regions in stable order.
        for region in Self::sorted_named_regions(self) {
            let region_palette = region.get_palette();
            for block_index in &region.blocks {
                blocks.push(region_palette[*block_index].clone());
            }
        }
        blocks
    }

    pub fn get_region_names(&self) -> Vec<String> {
        let mut names = vec![self.default_region_name.clone()];
        let mut named: Vec<_> = self.other_regions.keys().cloned().collect();
        named.sort();
        names.extend(named);
        names
    }

    pub fn get_region_from_index(&self, index: usize) -> Option<&Region> {
        if index == 0 {
            Some(&self.default_region)
        } else {
            Self::sorted_named_regions(self).get(index - 1).copied()
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

    /// Read a block from exactly one region, without composite overlap lookup.
    pub fn get_block_in_region(
        &self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<&BlockState> {
        self.get_block_from_region(region_name, x, y, z)
    }

    /// Read a block-state string from exactly one region.
    pub fn get_block_string_in_region(
        &self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<String> {
        self.get_block_in_region(region_name, x, y, z)
            .map(ToString::to_string)
    }

    /// Read a position-aware owned block entity from exactly one region.
    ///
    /// Block-entity templates are internally deduplicated, so borrowed template
    /// positions are not authoritative after transforms. This accessor always
    /// materializes the queried store coordinate.
    pub fn get_block_entity_in_region(
        &self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<BlockEntity> {
        let position = BlockPosition { x, y, z };
        let mut entity = self
            .get_region(region_name)?
            .get_block_entity(position)?
            .clone();
        entity.position = (x, y, z);
        Some(entity)
    }

    /// Position-aware owned counterpart to the historical borrowed accessor.
    pub fn get_block_entity_owned(&self, position: BlockPosition) -> Option<BlockEntity> {
        let mut entity = self.get_block_entity(position)?.clone();
        entity.position = (position.x, position.y, position.z);
        Some(entity)
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

    /// Get the bounding box covering all content (non-air blocks, entities,
    /// block entities) across every region. This is what exporters use so
    /// that Litematica (and other formats) keep entities/block entities that
    /// sit outside the block bounds.
    pub fn get_content_bounds(&self) -> Option<BoundingBox> {
        let mut result = self.default_region.get_content_bounds();
        for region in self.other_regions.values() {
            match (result.take(), region.get_content_bounds()) {
                (Some(a), Some(b)) => result = Some(a.union(&b)),
                (Some(a), None) => result = Some(a),
                (None, Some(b)) => result = Some(b),
                (None, None) => {}
            }
        }
        result
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

    pub fn total_blocks(&self) -> i32 {
        let mut total = self.default_region.count_blocks() as i32;
        total += self
            .other_regions
            .values()
            .map(|r| r.count_blocks() as i32)
            .sum::<i32>();
        total
    }

    pub fn total_volume(&self) -> i32 {
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

    pub fn has_region(&self, name: &str) -> bool {
        name == self.default_region_name || self.other_regions.contains_key(name)
    }

    pub fn create_schematic_region(&mut self, name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Region name cannot be empty".to_string());
        }
        if self.has_region(name) {
            return Err(format!("Region '{name}' already exists"));
        }

        self.other_regions.insert(
            name.to_string(),
            Region::new(name.to_string(), (0, 0, 0), (1, 1, 1)),
        );
        Ok(())
    }

    pub fn remove_schematic_region(&mut self, name: &str) -> Result<Region, String> {
        if name == self.default_region_name {
            return Err("The default region cannot be removed".to_string());
        }
        self.other_regions
            .remove(name)
            .ok_or_else(|| format!("Region '{name}' not found"))
    }

    pub fn rename_schematic_region(
        &mut self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), String> {
        if old_name == self.default_region_name {
            return Err("The default region cannot be renamed".to_string());
        }
        if new_name.trim().is_empty() {
            return Err("Region name cannot be empty".to_string());
        }
        if self.has_region(new_name) {
            return Err(format!("Region '{new_name}' already exists"));
        }

        let mut region = self
            .other_regions
            .remove(old_name)
            .ok_or_else(|| format!("Region '{old_name}' not found"))?;
        region.name = new_name.to_string();
        self.other_regions.insert(new_name.to_string(), region);
        Ok(())
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

    pub fn fix_redstone_connectivity(&mut self) {
        let regions: Vec<String> = self.get_all_regions().keys().cloned().collect();
        for region_name in regions {
            self.fix_redstone_connectivity_for_region(&region_name);
        }
    }

    pub fn fix_redstone_connectivity_for_region(&mut self, region_name: &str) {
        let (min, max) = {
            let region = match self.get_region(region_name) {
                Some(r) => r,
                None => return,
            };
            let (width, height, length) = region.size;
            let (pos_x, pos_y, pos_z) = region.position;
            (
                (pos_x, pos_y, pos_z),
                (pos_x + width, pos_y + height, pos_z + length),
            )
        };

        for y in min.1..max.1 {
            for x in min.0..max.0 {
                for z in min.2..max.2 {
                    let block = match self.get_block_from_region(region_name, x, y, z) {
                        Some(b) if b.name == "minecraft:redstone_wire" => b.clone(),
                        _ => continue,
                    };

                    let mut new_block = block.clone();
                    let directions = [
                        ("north", 0, -1),
                        ("south", 0, 1),
                        ("east", 1, 0),
                        ("west", -1, 0),
                    ];

                    let mut connection_states = Vec::new();
                    let mut connected_count = 0;

                    for (dir, dx, dz) in directions {
                        let side_val = if self.should_connect_redstone(region_name, x, y, z, dx, dz)
                        {
                            "side"
                        } else if self.should_connect_redstone_up(region_name, x, y, z, dx, dz) {
                            "up"
                        } else {
                            "none"
                        };

                        if side_val != "none" {
                            connected_count += 1;
                        }
                        connection_states.push((dir, side_val.to_string()));
                    }

                    // Fix single connection issue: if only 1 connected, opposite becomes side
                    if connected_count == 1 {
                        // Find the connected one
                        if let Some(idx) =
                            connection_states.iter().position(|(_, val)| val != "none")
                        {
                            // 0=north, 1=south, 2=east, 3=west
                            // Opposites: 0<->1, 2<->3
                            let opposite_idx = match idx {
                                0 => 1,
                                1 => 0,
                                2 => 3,
                                3 => 2,
                                _ => unreachable!(),
                            };

                            // Force opposite to be "side"
                            connection_states[opposite_idx].1 = "side".to_string();
                        }
                    }

                    for (dir, val) in connection_states {
                        new_block.set_property(dir, val);
                    }

                    self.set_block_in_region(region_name, x, y, z, &new_block);
                }
            }
        }
    }

    fn should_connect_redstone(
        &self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
        dx: i32,
        dz: i32,
    ) -> bool {
        // Same level
        if let Some(neighbor) = self.get_block_from_region(region_name, x + dx, y, z + dz) {
            if is_redstone_connectable(neighbor) {
                return true;
            }
        }
        // One level down
        if let Some(neighbor_down) = self.get_block_from_region(region_name, x + dx, y - 1, z + dz)
        {
            if neighbor_down.name == "minecraft:redstone_wire" {
                // Only if the block above neighbor_down is air or non-opaque
                if let Some(above_neighbor_down) =
                    self.get_block_from_region(region_name, x + dx, y, z + dz)
                {
                    if !is_opaque(above_neighbor_down) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn should_connect_redstone_up(
        &self,
        region_name: &str,
        x: i32,
        y: i32,
        z: i32,
        dx: i32,
        dz: i32,
    ) -> bool {
        // One level up
        // Only if the block above the current wire is not opaque
        if let Some(above_current) = self.get_block_from_region(region_name, x, y + 1, z) {
            if is_opaque(above_current) {
                return false;
            }
        }

        if let Some(neighbor_up) = self.get_block_from_region(region_name, x + dx, y + 1, z + dz) {
            if neighbor_up.name == "minecraft:redstone_wire" {
                return true;
            }
        }
        false
    }

    pub fn get_merged_region(&self) -> Region {
        let named_regions = Self::sorted_named_regions(self);
        let mut merged_region = match named_regions.last() {
            Some(region) => (*region).clone(),
            None => return self.default_region.clone(),
        };

        for region in named_regions[..named_regions.len() - 1].iter().rev() {
            merged_region.merge_with_precedence(region);
        }
        merged_region.merge_with_precedence(&self.default_region);
        merged_region.name = self.default_region_name.clone();
        merged_region.entities = self
            .default_region
            .entities
            .iter()
            .chain(
                named_regions
                    .iter()
                    .flat_map(|region| region.entities.iter()),
            )
            .cloned()
            .collect();
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

        let mut metadata_tag = self.metadata.to_nbt();

        // Serialize definition regions to JSON string and store in Metadata
        if !self.definition_regions.is_empty() {
            if let NbtTag::Compound(ref mut metadata_compound) = metadata_tag {
                if let Ok(json) = serde_json::to_string(&self.definition_regions) {
                    metadata_compound.insert("NucleationDefinitions", NbtTag::String(json));
                }
            }
        }

        root.insert("Metadata", metadata_tag);

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
        let metadata_tag = nbt
            .get::<_, &NbtCompound>("Metadata")
            .map_err(|e| format!("Failed to get Metadata: {}", e))?;

        let metadata = Metadata::from_nbt(metadata_tag)?;

        // Try to parse definition regions from Metadata
        let mut definition_regions = HashMap::new();
        if let Ok(json) = metadata_tag.get::<_, &str>("NucleationDefinitions") {
            if let Ok(regions) = serde_json::from_str(json) {
                definition_regions = regions;
            }
        }

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
            definition_regions,
            block_state_cache: FxHashMap::default(),
            block_string_cache: FxHashMap::default(),
        })
    }

    pub fn import_insign_regions(&mut self) -> Result<(), String> {
        let json_value = crate::insign::compile_schematic_insign(self)
            .map_err(|e| format!("Insign compilation error: {}", e))?;

        if let Some(regions_map) = json_value.as_object() {
            for (name, data) in regions_map {
                let mut def_region = DefinitionRegion::new();

                // Parse bounding boxes
                if let Some(boxes) = data.get("bounding_boxes").and_then(|v| v.as_array()) {
                    for bbox_json in boxes {
                        if let Some(coords) = bbox_json.as_array() {
                            if coords.len() >= 2 {
                                let min_arr = coords[0].as_array();
                                let max_arr = coords[1].as_array();

                                if let (Some(min), Some(max)) = (min_arr, max_arr) {
                                    if min.len() == 3 && max.len() == 3 {
                                        let min_tuple = (
                                            min[0].as_i64().unwrap_or(0) as i32,
                                            min[1].as_i64().unwrap_or(0) as i32,
                                            min[2].as_i64().unwrap_or(0) as i32,
                                        );
                                        let max_tuple = (
                                            max[0].as_i64().unwrap_or(0) as i32,
                                            max[1].as_i64().unwrap_or(0) as i32,
                                            max[2].as_i64().unwrap_or(0) as i32,
                                        );
                                        def_region.add_bounds(min_tuple, max_tuple);
                                    }
                                }
                            }
                        }
                    }
                }

                // Parse metadata
                if let Some(meta) = data.get("metadata").and_then(|v| v.as_object()) {
                    for (key, value) in meta {
                        // Convert value to string
                        let val_str = match value {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            _ => value.to_string(),
                        };
                        def_region.set_metadata(key, val_str);
                    }
                }

                self.definition_regions.insert(name.clone(), def_region);
            }
        }

        Ok(())
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
                    &block_state.clone(),
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
            .map(|(pos, block)| (pos, block.clone()))
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
                    &block_state.clone(),
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
        Ok(crate::formats::schematic::to_schematic(self)?)
    }

    pub fn from_schematic(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(crate::formats::schematic::from_schematic(data)?)
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

    fn sorted_named_regions(source: &UniversalSchematic) -> Vec<&Region> {
        let mut regions: Vec<_> = source.other_regions.iter().collect();
        regions.sort_by(|(left, _), (right, _)| left.cmp(right));
        regions.into_iter().map(|(_, region)| region).collect()
    }

    fn source_region_at<'a>(
        source: &'a UniversalSchematic,
        named_regions: &[&'a Region],
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<&'a Region> {
        fn contains_visible_content(region: &Region, x: i32, y: i32, z: i32) -> bool {
            region
                .get_tight_bounds()
                .is_some_and(|bounds| bounds.contains((x, y, z)))
        }

        if contains_visible_content(&source.default_region, x, y, z) {
            return Some(&source.default_region);
        }
        named_regions
            .iter()
            .copied()
            .find(|region| contains_visible_content(region, x, y, z))
    }

    fn region_stamp_bounds(region: &Region) -> Option<BoundingBox> {
        let mut bounds = region.get_tight_bounds();
        for entity in &region.entities {
            let position = (
                entity.position.0.floor() as i32,
                entity.position.1.floor() as i32,
                entity.position.2.floor() as i32,
            );
            match &mut bounds {
                Some(bounds) => {
                    bounds.min.0 = bounds.min.0.min(position.0);
                    bounds.min.1 = bounds.min.1.min(position.1);
                    bounds.min.2 = bounds.min.2.min(position.2);
                    bounds.max.0 = bounds.max.0.max(position.0);
                    bounds.max.1 = bounds.max.1.max(position.1);
                    bounds.max.2 = bounds.max.2.max(position.2);
                }
                None => bounds = Some(BoundingBox::new(position, position)),
            }
        }
        bounds
    }

    fn stamp_offset(bounds: &BoundingBox, target: (i32, i32, i32)) -> (i64, i64, i64) {
        (
            target.0 as i64 - bounds.min.0 as i64,
            target.1 as i64 - bounds.min.1 as i64,
            target.2 as i64 - bounds.min.2 as i64,
        )
    }

    fn stamp_destination(
        source: (i32, i32, i32),
        offset: (i64, i64, i64),
    ) -> Result<(i32, i32, i32), String> {
        Ok((
            Self::checked_coord(source.0 as i64 + offset.0)?,
            Self::checked_coord(source.1 as i64 + offset.1)?,
            Self::checked_coord(source.2 as i64 + offset.2)?,
        ))
    }

    fn stamp_cell_from_region(
        &mut self,
        source_region: &Region,
        source: (i32, i32, i32),
        destination: (i32, i32, i32),
        excluded_blocks: &[BlockState],
    ) {
        let Some(block) = source_region.get_block(source.0, source.1, source.2) else {
            return;
        };
        if excluded_blocks.contains(block) {
            return;
        }
        let destination = BlockPosition {
            x: destination.0,
            y: destination.1,
            z: destination.2,
        };
        self.set_block(destination.x, destination.y, destination.z, block);
        self.remove_block_entity((destination.x, destination.y, destination.z));
        let source = BlockPosition {
            x: source.0,
            y: source.1,
            z: source.2,
        };
        if let Some(block_entity) = source_region.get_block_entity(source) {
            let mut copied = block_entity.clone();
            copied.position = (destination.x, destination.y, destination.z);
            self.set_block_entity(destination, copied);
        }
    }

    /// Stamp a merged schematic box into the default region. Excluded source blocks are skipped,
    /// preserving the destination. Written cells, including air, clear stale block entities.
    pub fn stamp_box(
        &mut self,
        source: &UniversalSchematic,
        bounds: &BoundingBox,
        target: (i32, i32, i32),
        excluded_blocks: &[BlockState],
    ) -> Result<(), String> {
        let offset = Self::stamp_offset(bounds, target);
        Self::stamp_destination(bounds.min, offset)?;
        Self::stamp_destination(bounds.max, offset)?;
        let named_regions = Self::sorted_named_regions(source);
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    let Some(source_region) =
                        Self::source_region_at(source, &named_regions, x, y, z)
                    else {
                        continue;
                    };
                    let destination = Self::stamp_destination((x, y, z), offset)?;
                    self.stamp_cell_from_region(
                        source_region,
                        (x, y, z),
                        destination,
                        excluded_blocks,
                    );
                }
            }
        }
        for entity in source.default_region.entities.iter().chain(
            named_regions
                .iter()
                .flat_map(|region| region.entities.iter()),
        ) {
            let position = (
                entity.position.0.floor() as i32,
                entity.position.1.floor() as i32,
                entity.position.2.floor() as i32,
            );
            if bounds.contains(position) {
                let mut copied = entity.clone();
                copied.position = (
                    entity.position.0 + offset.0 as f64,
                    entity.position.1 + offset.1 as f64,
                    entity.position.2 + offset.2 as f64,
                );
                self.add_entity(copied);
            }
        }
        Ok(())
    }

    /// Stamp one explicitly named source region into the default region.
    pub fn stamp_region(
        &mut self,
        source: &UniversalSchematic,
        region_name: &str,
        target: (i32, i32, i32),
        excluded_blocks: &[BlockState],
    ) -> Result<(), String> {
        let source_region = source
            .get_region(region_name)
            .ok_or_else(|| format!("Region '{region_name}' not found"))?;
        let Some(bounds) = Self::region_stamp_bounds(source_region) else {
            return Ok(());
        };
        let offset = Self::stamp_offset(&bounds, target);
        Self::stamp_destination(bounds.min, offset)?;
        Self::stamp_destination(bounds.max, offset)?;
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    let destination = Self::stamp_destination((x, y, z), offset)?;
                    self.stamp_cell_from_region(
                        source_region,
                        (x, y, z),
                        destination,
                        excluded_blocks,
                    );
                }
            }
        }
        for entity in &source_region.entities {
            let mut copied = entity.clone();
            copied.position = (
                entity.position.0 + offset.0 as f64,
                entity.position.1 + offset.1 as f64,
                entity.position.2 + offset.2 as f64,
            );
            self.add_entity(copied);
        }
        Ok(())
    }

    /// Compatibility alias for `stamp_box`.
    pub fn copy_region(
        &mut self,
        source: &UniversalSchematic,
        bounds: &BoundingBox,
        target: (i32, i32, i32),
        excluded_blocks: &[BlockState],
    ) -> Result<(), String> {
        self.stamp_box(source, bounds, target, excluded_blocks)
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
                .or_default()
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
                    .or_default()
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
                    &self.default_region.palette[*block_index],
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
                    Some((BlockPosition { x, y, z }, &region.palette[*block_index]))
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
                .or_default()
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
                    .or_default()
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
        let mut blocks = Vec::with_capacity((width * height * length) as usize);

        // Helper to process a region
        let mut process_region = |region: &Region| {
            let region_bbox = region.get_bounding_box();

            // Calculate intersection between chunk and region
            // Note: region_bbox.max is INCLUSIVE, but Rust ranges are EXCLUSIVE on the end
            // So we need +1 to include blocks at the maximum boundary
            let start_x = std::cmp::max(offset_x, region_bbox.min.0);
            let end_x = std::cmp::min(offset_x + width, region_bbox.max.0 + 1);

            let start_y = std::cmp::max(offset_y, region_bbox.min.1);
            let end_y = std::cmp::min(offset_y + height, region_bbox.max.1 + 1);

            let start_z = std::cmp::max(offset_z, region_bbox.min.2);
            let end_z = std::cmp::min(offset_z + length, region_bbox.max.2 + 1);

            // Find air index for this region to correctly skip air blocks
            let air_index = region
                .palette
                .iter()
                .position(|b| b.name == "minecraft:air");

            // If there is an intersection volume
            if start_x < end_x && start_y < end_y && start_z < end_z {
                for y in start_y..end_y {
                    for z in start_z..end_z {
                        for x in start_x..end_x {
                            let index = region.coords_to_index(x, y, z);
                            if let Some(&palette_index) = region.blocks.get(index) {
                                // Skip if it matches the air index
                                let is_air = match air_index {
                                    Some(idx) => palette_index == idx,
                                    None => false, // If no air in palette, assume no blocks are air
                                };

                                if !is_air {
                                    blocks.push((BlockPosition { x, y, z }, palette_index));
                                }
                            }
                        }
                    }
                }
            }
        };

        // Check default region
        process_region(&self.default_region);

        // Check other regions
        for region in self.other_regions.values() {
            process_region(region);
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

    fn parse_block_string_cached(
        &mut self,
        block_string: &str,
    ) -> Result<(BlockState, Option<std::sync::Arc<NbtMap>>), String> {
        if let Some((state, nbt)) = self.block_string_cache.get(block_string) {
            return Ok((state.clone(), nbt.clone()));
        }

        let (mut block_state, nbt_data) = Self::parse_block_string(block_string)?;
        if block_state.name == "minecraft:jukebox" {
            let has_record = nbt_data
                .as_ref()
                .is_some_and(|nbt| nbt.contains_key("RecordItem"));
            block_state.set_property("has_record", has_record.to_string());
        }

        let nbt = nbt_data.map(|data| {
            let mut map = NbtMap::new();
            for (key, value) in data {
                map.insert(key, value);
            }
            std::sync::Arc::new(map)
        });
        self.block_string_cache
            .insert(block_string.to_string(), (block_state.clone(), nbt.clone()));
        Ok((block_state, nbt))
    }

    pub fn set_block_from_string(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        block_string: &str,
    ) -> Result<bool, String> {
        let (block_state, nbt) = self.parse_block_string_cached(block_string)?;

        // Set the basic block first
        if !self.set_block(x, y, z, &block_state) {
            return Ok(false);
        }

        // Replacement is transactional with respect to block-entity state:
        // a new block string with no NBT must not retain the previous entity.
        self.remove_block_entity((x, y, z));

        // If we have NBT data, create and set the block entity. The NbtMap
        // stays Arc-shared with the cache (copy-on-write via nbt_mut).
        if let Some(nbt) = nbt {
            let block_entity = BlockEntity {
                id: block_state.name.to_string(),
                position: (x, y, z),
                nbt,
            };
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
        Self::validate_block_string_delimiters(block_string)?;
        let (block_state_str, nbt_str) = match block_string.split_once('{') {
            Some((block_state, nbt)) => {
                let nbt = nbt
                    .strip_suffix('}')
                    .ok_or("Missing block entity closing brace")?;
                (block_state.trim(), Some(nbt))
            }
            None => (block_string.trim(), None),
        };

        // Parse block state
        let block_state = if block_state_str.contains('[') {
            let mut state_parts = block_state_str.splitn(2, '[');
            let block_name = state_parts.next().unwrap();
            let properties_str = state_parts
                .next()
                .ok_or("Missing properties closing bracket")?
                .strip_suffix(']')
                .ok_or("Missing properties closing bracket")?;

            let mut properties = Vec::new();
            for prop in properties_str.split(',') {
                let (key, value) = prop
                    .split_once('=')
                    .ok_or("Missing property key or value")?;
                let key = key.trim();
                let value = value.trim().trim_matches(|c| c == '\'' || c == '"');
                if key.is_empty() || value.is_empty() || value.contains('=') {
                    return Err("Malformed block property".to_string());
                }
                properties.push((SmolStr::from(key), SmolStr::from(value)));
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

    fn validate_block_string_delimiters(block_string: &str) -> Result<(), String> {
        let mut stack = Vec::new();
        let mut quote = None;
        let mut escaped = false;

        for c in block_string.chars() {
            if quote.is_some() && c == '\\' && !escaped {
                escaped = true;
                continue;
            }
            if matches!(c, '\'' | '"') && !escaped {
                match quote {
                    Some(active) if active == c => quote = None,
                    None => quote = Some(c),
                    _ => {}
                }
                continue;
            }
            if quote.is_none() {
                match (c, stack.last().copied()) {
                    ('[' | '{', _) => stack.push(c),
                    (']', Some('[')) | ('}', Some('{')) => {
                        stack.pop();
                    }
                    (']', _) => {
                        return Err("Unmatched or misordered ']' in block string".to_string());
                    }
                    ('}', _) => {
                        return Err("Unmatched or misordered '}' in block string".to_string());
                    }
                    _ => {}
                }
            }
            escaped = false;
        }

        if quote.is_some() {
            return Err("Unterminated quoted string in block string".to_string());
        }
        if !stack.is_empty() {
            return Err("Unclosed delimiter in block string".to_string());
        }
        Ok(())
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
                        new_schematic.set_block(new_x, new_y, new_z, block);
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
        self.block_string_cache.clear();
    }

    /// Get cache statistics for debugging
    pub fn cache_stats(&self) -> (usize, usize) {
        (
            self.block_state_cache.len(),
            self.block_state_cache.capacity(),
        )
    }

    // Transformation methods (convenience wrappers for the default region)

    fn normalized_quarter_turn(degrees: i32) -> Result<i32, String> {
        if degrees % 90 != 0 {
            return Err(format!(
                "Rotation must be a multiple of 90 degrees, got {degrees}"
            ));
        }
        Ok(degrees.rem_euclid(360))
    }

    fn checked_coord(value: i64) -> Result<i32, String> {
        i32::try_from(value).map_err(|_| "Transform exceeds the i32 coordinate range".to_string())
    }

    fn checked_delta(target: i32, current: i32) -> Result<i32, String> {
        Self::checked_coord(target as i64 - current as i64)
    }

    /// Flip the default region along the X axis.
    pub fn flip_x(&mut self) {
        self.default_region.flip_x();
    }

    /// Flip the default region along the Y axis.
    pub fn flip_y(&mut self) {
        self.default_region.flip_y();
    }

    /// Flip the default region along the Z axis.
    pub fn flip_z(&mut self) {
        self.default_region.flip_z();
    }

    /// Rotate the default region around the Y axis (horizontal plane).
    pub fn rotate_y(&mut self, degrees: i32) -> Result<(), String> {
        self.default_region
            .rotate_y(Self::normalized_quarter_turn(degrees)?);
        Ok(())
    }

    /// Rotate the default region around the X axis.
    pub fn rotate_x(&mut self, degrees: i32) -> Result<(), String> {
        self.default_region
            .rotate_x(Self::normalized_quarter_turn(degrees)?);
        Ok(())
    }

    /// Rotate the default region around the Z axis.
    pub fn rotate_z(&mut self, degrees: i32) -> Result<(), String> {
        self.default_region
            .rotate_z(Self::normalized_quarter_turn(degrees)?);
        Ok(())
    }

    /// Translate the default region.
    pub fn translate(&mut self, dx: i32, dy: i32, dz: i32) -> Result<(), String> {
        self.default_region.translate(dx, dy, dz)
    }

    /// Flip a specific region along the X axis.
    pub fn flip_region_x(&mut self, region_name: &str) -> Result<(), String> {
        self.get_region_mut(region_name)
            .ok_or_else(|| format!("Region '{region_name}' not found"))?
            .flip_x();
        Ok(())
    }

    /// Flip a specific region along the Y axis.
    pub fn flip_region_y(&mut self, region_name: &str) -> Result<(), String> {
        self.get_region_mut(region_name)
            .ok_or_else(|| format!("Region '{region_name}' not found"))?
            .flip_y();
        Ok(())
    }

    /// Flip a specific region along the Z axis.
    pub fn flip_region_z(&mut self, region_name: &str) -> Result<(), String> {
        self.get_region_mut(region_name)
            .ok_or_else(|| format!("Region '{region_name}' not found"))?
            .flip_z();
        Ok(())
    }

    /// Rotate a specific region around the Y axis.
    pub fn rotate_region_y(&mut self, region_name: &str, degrees: i32) -> Result<(), String> {
        let degrees = Self::normalized_quarter_turn(degrees)?;
        self.get_region_mut(region_name)
            .ok_or_else(|| format!("Region '{region_name}' not found"))?
            .rotate_y(degrees);
        Ok(())
    }

    /// Rotate a specific region around the X axis.
    pub fn rotate_region_x(&mut self, region_name: &str, degrees: i32) -> Result<(), String> {
        let degrees = Self::normalized_quarter_turn(degrees)?;
        self.get_region_mut(region_name)
            .ok_or_else(|| format!("Region '{region_name}' not found"))?
            .rotate_x(degrees);
        Ok(())
    }

    /// Rotate a specific region around the Z axis.
    pub fn rotate_region_z(&mut self, region_name: &str, degrees: i32) -> Result<(), String> {
        let degrees = Self::normalized_quarter_turn(degrees)?;
        self.get_region_mut(region_name)
            .ok_or_else(|| format!("Region '{region_name}' not found"))?
            .rotate_z(degrees);
        Ok(())
    }

    /// Translate a specific region without affecting its siblings.
    pub fn translate_region(
        &mut self,
        region_name: &str,
        dx: i32,
        dy: i32,
        dz: i32,
    ) -> Result<(), String> {
        self.get_region_mut(region_name)
            .ok_or_else(|| format!("Region '{region_name}' not found"))?
            .translate(dx, dy, dz)
    }

    /// Translate every schematic region as one rigid object. The operation is
    /// transactional: coordinate overflow leaves the schematic unchanged.
    pub fn translate_schematic(&mut self, dx: i32, dy: i32, dz: i32) -> Result<(), String> {
        let mut transformed = self.clone();
        transformed.default_region.translate(dx, dy, dz)?;
        for region in transformed.other_regions.values_mut() {
            region.translate(dx, dy, dz)?;
        }
        *self = transformed;
        Ok(())
    }

    fn rotate_schematic_x_90(&mut self) -> Result<(), String> {
        let overall = self
            .get_schematic_bounding_box()
            .ok_or_else(|| "Schematic has no bounds".to_string())?;
        let global_size_z = overall.max.2 as i64 - overall.min.2 as i64 + 1;
        let (global_min_y, global_min_z) = (overall.min.1 as i64, overall.min.2 as i64);
        let rotate = |region: &mut Region| -> Result<(), String> {
            let old = region.get_bounding_box();
            let old_size_z = old.get_dimensions().2 as i64;
            let desired_y =
                global_min_y + global_size_z - (old.min.2 as i64 - global_min_z) - old_size_z;
            let desired_z = global_min_z + (old.min.1 as i64 - global_min_y);
            region.rotate_x_90_at((
                region.position.0,
                Self::checked_coord(desired_y)?,
                Self::checked_coord(desired_z)?,
            ))
        };
        rotate(&mut self.default_region)?;
        for region in self.other_regions.values_mut() {
            rotate(region)?;
        }
        Ok(())
    }

    fn rotate_schematic_y_90(&mut self) -> Result<(), String> {
        let overall = self
            .get_schematic_bounding_box()
            .ok_or_else(|| "Schematic has no bounds".to_string())?;
        let global_size_z = overall.max.2 as i64 - overall.min.2 as i64 + 1;
        let (global_min_x, global_min_z) = (overall.min.0 as i64, overall.min.2 as i64);
        let rotate = |region: &mut Region| -> Result<(), String> {
            let old = region.get_bounding_box();
            let old_size_z = old.get_dimensions().2 as i64;
            let desired_x =
                global_min_x + global_size_z - (old.min.2 as i64 - global_min_z) - old_size_z;
            let desired_z = global_min_z + (old.min.0 as i64 - global_min_x);
            region.rotate_y_90_at((
                Self::checked_coord(desired_x)?,
                region.position.1,
                Self::checked_coord(desired_z)?,
            ))
        };
        rotate(&mut self.default_region)?;
        for region in self.other_regions.values_mut() {
            rotate(region)?;
        }
        Ok(())
    }

    fn rotate_schematic_z_90(&mut self) -> Result<(), String> {
        let overall = self
            .get_schematic_bounding_box()
            .ok_or_else(|| "Schematic has no bounds".to_string())?;
        let global_size_y = overall.max.1 as i64 - overall.min.1 as i64 + 1;
        let (global_min_x, global_min_y) = (overall.min.0 as i64, overall.min.1 as i64);
        let rotate = |region: &mut Region| -> Result<(), String> {
            let old = region.get_bounding_box();
            let old_size_y = old.get_dimensions().1 as i64;
            let desired_x =
                global_min_x + global_size_y - (old.min.1 as i64 - global_min_y) - old_size_y;
            let desired_y = global_min_y + (old.min.0 as i64 - global_min_x);
            region.rotate_z_90_at((
                Self::checked_coord(desired_x)?,
                Self::checked_coord(desired_y)?,
                region.position.2,
            ))
        };
        rotate(&mut self.default_region)?;
        for region in self.other_regions.values_mut() {
            rotate(region)?;
        }
        Ok(())
    }

    fn rotate_schematic_with(
        &mut self,
        degrees: i32,
        quarter_turn: fn(&mut Self) -> Result<(), String>,
    ) -> Result<(), String> {
        let rotations = Self::normalized_quarter_turn(degrees)? / 90;
        let mut transformed = self.clone();
        for _ in 0..rotations {
            quarter_turn(&mut transformed)?;
        }
        *self = transformed;
        Ok(())
    }

    pub fn rotate_schematic_x(&mut self, degrees: i32) -> Result<(), String> {
        self.rotate_schematic_with(degrees, Self::rotate_schematic_x_90)
    }

    pub fn rotate_schematic_y(&mut self, degrees: i32) -> Result<(), String> {
        self.rotate_schematic_with(degrees, Self::rotate_schematic_y_90)
    }

    pub fn rotate_schematic_z(&mut self, degrees: i32) -> Result<(), String> {
        self.rotate_schematic_with(degrees, Self::rotate_schematic_z_90)
    }

    fn flip_schematic_axis(&mut self, axis: char) -> Result<(), String> {
        let overall = self
            .get_schematic_bounding_box()
            .ok_or_else(|| "Schematic has no bounds".to_string())?;
        let mut transformed = self.clone();
        let flip = |region: &mut Region| -> Result<(), String> {
            let old = region.get_bounding_box();
            let (target, current) = match axis {
                'x' => {
                    region.flip_x();
                    (
                        Self::checked_coord(
                            overall.min.0 as i64 + overall.max.0 as i64 - old.max.0 as i64,
                        )?,
                        region.position.0,
                    )
                }
                'y' => {
                    region.flip_y();
                    (
                        Self::checked_coord(
                            overall.min.1 as i64 + overall.max.1 as i64 - old.max.1 as i64,
                        )?,
                        region.position.1,
                    )
                }
                'z' => {
                    region.flip_z();
                    (
                        Self::checked_coord(
                            overall.min.2 as i64 + overall.max.2 as i64 - old.max.2 as i64,
                        )?,
                        region.position.2,
                    )
                }
                _ => unreachable!(),
            };
            let delta = Self::checked_delta(target, current)?;
            match axis {
                'x' => region.translate(delta, 0, 0),
                'y' => region.translate(0, delta, 0),
                'z' => region.translate(0, 0, delta),
                _ => unreachable!(),
            }
        };
        flip(&mut transformed.default_region)?;
        for region in transformed.other_regions.values_mut() {
            flip(region)?;
        }
        *self = transformed;
        Ok(())
    }

    pub fn flip_schematic_x(&mut self) -> Result<(), String> {
        self.flip_schematic_axis('x')
    }

    pub fn flip_schematic_y(&mut self) -> Result<(), String> {
        self.flip_schematic_axis('y')
    }

    pub fn flip_schematic_z(&mut self) -> Result<(), String> {
        self.flip_schematic_axis('z')
    }

    /// Create a new definition region and return a mutable reference to it for chaining
    pub fn create_region(
        &mut self,
        name: String,
        min: (i32, i32, i32),
        max: (i32, i32, i32),
    ) -> &mut DefinitionRegion {
        let mut region = DefinitionRegion::new();
        region.add_bounds(min, max);
        self.definition_regions.insert(name.clone(), region);
        self.definition_regions.get_mut(&name).unwrap()
    }

    /// Get a mutable reference to a definition region for chaining
    pub fn get_definition_region_mut(&mut self, name: &str) -> Option<&mut DefinitionRegion> {
        self.definition_regions.get_mut(name)
    }
}

pub fn is_redstone_connectable(block: &BlockState) -> bool {
    let name = block.name.as_str();
    name == "minecraft:redstone_wire"
        || name == "minecraft:repeater"
        || name == "minecraft:comparator"
        || name == "minecraft:observer"
        || name == "minecraft:target"
}

pub fn is_opaque(block: &BlockState) -> bool {
    let name = block.name.as_str();
    // Simplified opaque check - most common non-opaque blocks
    !(name == "minecraft:air"
        || name == "minecraft:cave_air"
        || name == "minecraft:void_air"
        || name.contains("glass")
        || name.contains("slab")
        || name.contains("stairs")
        || name.contains("fence")
        || name.contains("wall")
        || name.contains("iron_bars")
        || name.contains("door")
        || name.contains("trapdoor")
        || name.contains("torch")
        || name.contains("button")
        || name.contains("pressure_plate")
        || name.contains("sign")
        || name == "minecraft:redstone_wire"
        || name == "minecraft:repeater"
        || name == "minecraft:comparator"
        || name == "minecraft:lever"
        || name == "minecraft:hopper")
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::item::ItemStack;
    use quartz_nbt::io::{read_nbt, write_nbt};
    use std::io::Cursor;

    #[test]
    fn named_region_block_strings_parse_and_replace_block_entities() {
        let mut schematic = UniversalSchematic::new("regions".to_string());

        schematic
            .try_set_block_in_region_str(
                "wing",
                10,
                0,
                0,
                "minecraft:chest[facing=east]{items=[diamond]}",
            )
            .unwrap();

        let state = schematic
            .get_block_from_region("wing", 10, 0, 0)
            .expect("named-region block should exist");
        assert_eq!(state.name, "minecraft:chest");
        assert_eq!(
            state.get_property("facing").map(|value| value.as_str()),
            Some("east")
        );
        assert!(schematic
            .get_region("wing")
            .unwrap()
            .get_block_entity(BlockPosition { x: 10, y: 0, z: 0 })
            .is_some());

        schematic.rotate_region_y("wing", 90).unwrap();
        let rotated = schematic
            .get_block_from_region("wing", 10, 0, 0)
            .expect("rotated block should remain at the one-cell region anchor");
        assert_eq!(
            rotated.get_property("facing").map(|value| value.as_str()),
            Some("south")
        );

        schematic
            .try_set_block_in_region_str("wing", 10, 0, 0, "minecraft:stone")
            .unwrap();
        assert!(schematic
            .get_region("wing")
            .unwrap()
            .get_block_entity(BlockPosition { x: 10, y: 0, z: 0 })
            .is_none());
    }

    #[test]
    fn schematic_region_lifecycle_is_explicit_and_safe() {
        let mut schematic = UniversalSchematic::new("regions".to_string());

        schematic.create_schematic_region("wing").unwrap();
        assert!(schematic.has_region("wing"));
        assert!(schematic.create_schematic_region("wing").is_err());
        assert!(schematic.create_schematic_region("").is_err());
        assert!(schematic.create_schematic_region("Main").is_err());

        schematic
            .try_set_block_in_region_str("wing", 4, 5, 6, "minecraft:copper_block")
            .unwrap();
        let initial_bounds = schematic.get_region_bounding_box("wing").unwrap();
        assert_eq!(initial_bounds.min, (4, 5, 6));
        assert_eq!(initial_bounds.max, (4, 5, 6));
        schematic
            .rename_schematic_region("wing", "east_wing")
            .unwrap();
        assert!(!schematic.has_region("wing"));
        assert!(schematic.has_region("east_wing"));
        assert_eq!(
            schematic
                .get_block_from_region("east_wing", 4, 5, 6)
                .unwrap()
                .name,
            "minecraft:copper_block"
        );
        assert_eq!(schematic.get_region("east_wing").unwrap().name, "east_wing");

        assert!(schematic
            .rename_schematic_region("east_wing", "Main")
            .is_err());
        assert!(schematic.remove_schematic_region("Main").is_err());
        schematic.remove_schematic_region("east_wing").unwrap();
        assert!(!schematic.has_region("east_wing"));
        assert!(schematic.has_region("Main"));
    }

    #[test]
    fn rotations_reject_non_quarter_turns_without_mutation() {
        let mut schematic = UniversalSchematic::new("rotations".to_string());
        schematic.set_block_str(1, 2, 3, "minecraft:stone");
        schematic
            .try_set_block_in_region_str("wing", 10, 0, 0, "minecraft:gold_block")
            .unwrap();

        assert!(schematic.rotate_y(45).is_err());
        assert!(schematic.rotate_region_y("wing", -45).is_err());
        assert_eq!(
            schematic.get_block(1, 2, 3).unwrap().name,
            "minecraft:stone"
        );
        assert_eq!(
            schematic
                .get_block_from_region("wing", 10, 0, 0)
                .unwrap()
                .name,
            "minecraft:gold_block"
        );
    }

    #[test]
    fn translation_moves_region_content_and_attachments() {
        let mut schematic = UniversalSchematic::new("translation".to_string());
        schematic
            .set_block_from_string(1, 2, 3, "minecraft:chest{items=[diamond]}")
            .unwrap();
        schematic.add_entity(Entity::new(
            "minecraft:armor_stand".to_string(),
            (1.5, 2.0, 3.5),
        ));

        schematic.translate(10, -2, 4).unwrap();

        assert!(schematic.get_block(1, 2, 3).is_none());
        assert_eq!(
            schematic.get_block(11, 0, 7).unwrap().name,
            "minecraft:chest"
        );
        assert!(schematic
            .get_block_entity(BlockPosition { x: 11, y: 0, z: 7 })
            .is_some());
        assert_eq!(
            schematic.get_block_entities_as_list()[0].position,
            (11, 0, 7)
        );
        assert_eq!(
            schematic.default_region.entities[0].position,
            (11.5, 0.0, 7.5)
        );
    }

    #[test]
    fn translation_overflow_is_transactional() {
        let mut schematic = UniversalSchematic::new("overflow".to_string());
        schematic.set_block_str(i32::MAX - 1, 0, 0, "minecraft:stone");
        schematic
            .try_set_block_in_region_str("wing", 0, 0, 0, "minecraft:gold_block")
            .unwrap();

        assert!(schematic.translate_schematic(2, 0, 0).is_err());
        assert_eq!(
            schematic.get_block(i32::MAX - 1, 0, 0).unwrap().name,
            "minecraft:stone"
        );
        assert_eq!(
            schematic
                .get_block_from_region("wing", 0, 0, 0)
                .unwrap()
                .name,
            "minecraft:gold_block"
        );
    }

    #[test]
    fn named_and_schematic_transforms_have_distinct_scope() {
        let mut schematic = UniversalSchematic::new("multi".to_string());
        schematic.set_block_str(0, 0, 0, "minecraft:stone");
        schematic
            .try_set_block_in_region_str("wing", 2, 0, 0, "minecraft:oak_stairs[facing=east]")
            .unwrap();

        schematic.translate_region("wing", 3, 0, 0).unwrap();
        assert_eq!(
            schematic.get_block(0, 0, 0).unwrap().name,
            "minecraft:stone"
        );
        assert_eq!(
            schematic
                .get_block_from_region("wing", 5, 0, 0)
                .unwrap()
                .name,
            "minecraft:oak_stairs"
        );

        schematic.translate_schematic(-3, 0, 0).unwrap();
        assert_eq!(
            schematic.get_block(-3, 0, 0).unwrap().name,
            "minecraft:stone"
        );
        assert_eq!(
            schematic
                .get_block_from_region("wing", 2, 0, 0)
                .unwrap()
                .name,
            "minecraft:oak_stairs"
        );

        schematic.rotate_schematic_y(90).unwrap();
        assert_eq!(
            schematic.get_block(-3, 0, 0).unwrap().name,
            "minecraft:stone"
        );
        let wing = schematic.get_block_from_region("wing", -3, 0, 5).unwrap();
        assert_eq!(wing.name, "minecraft:oak_stairs");
        assert_eq!(
            wing.get_property("facing").map(|value| value.as_str()),
            Some("south")
        );

        schematic.flip_schematic_z().unwrap();
        assert_eq!(
            schematic.get_block(-3, 0, 5).unwrap().name,
            "minecraft:stone"
        );
        assert_eq!(
            schematic
                .get_block_from_region("wing", -3, 0, 0)
                .unwrap()
                .name,
            "minecraft:oak_stairs"
        );
    }

    #[test]
    fn quarter_turn_coordinates_and_facing_use_the_same_matrix() {
        let cases = [
            ('y', (1, 0, 0), (0, 0, 1), "east", "south"),
            ('x', (0, 0, 1), (0, 0, 0), "south", "down"),
            ('z', (0, 1, 0), (0, 0, 0), "up", "west"),
        ];

        for (axis, source, expected, facing, expected_facing) in cases {
            let mut schematic = UniversalSchematic::new(format!("axis-{axis}"));
            let size = match axis {
                'x' => (1, 1, 2),
                'y' => (2, 1, 1),
                'z' => (1, 2, 1),
                _ => unreachable!(),
            };
            schematic.default_region = Region::new("Main".to_string(), (0, 0, 0), size);
            schematic.set_block_str(0, 0, 0, "minecraft:stone");
            schematic
                .set_block_from_string(
                    source.0,
                    source.1,
                    source.2,
                    &format!("minecraft:observer[facing={facing}]"),
                )
                .unwrap();

            match axis {
                'x' => schematic.rotate_x(90).unwrap(),
                'y' => schematic.rotate_y(90).unwrap(),
                'z' => schematic.rotate_z(90).unwrap(),
                _ => unreachable!(),
            }

            let rotated = schematic
                .get_block(expected.0, expected.1, expected.2)
                .unwrap();
            assert_eq!(rotated.name, "minecraft:observer");
            assert_eq!(
                rotated.get_property("facing").map(|value| value.as_str()),
                Some(expected_facing)
            );
        }
    }

    #[test]
    fn whole_rotations_reject_signed_range_spans_without_mutation() {
        for axis in ['x', 'y', 'z'] {
            let (low, high) = match axis {
                'x' => ((0, i32::MIN, 0), (0, i32::MAX, 0)),
                'y' | 'z' => ((i32::MIN, 0, 0), (i32::MAX, 0, 0)),
                _ => unreachable!(),
            };
            let mut schematic = UniversalSchematic::new(format!("span-{axis}"));
            schematic.default_region = Region::new("Main".to_string(), low, (1, 1, 1));
            schematic.other_regions.insert(
                "far".to_string(),
                Region::new("far".to_string(), high, (1, 1, 1)),
            );

            let result = match axis {
                'x' => schematic.rotate_schematic_x(90),
                'y' => schematic.rotate_schematic_y(90),
                'z' => schematic.rotate_schematic_z(90),
                _ => unreachable!(),
            };

            assert!(result.is_err());
            assert_eq!(schematic.default_region.position, low);
            assert_eq!(schematic.other_regions["far"].position, high);
        }

        let endpoint_cases = [
            ('x', (0, i32::MAX, 0), (1, 1, 2)),
            ('y', (i32::MAX, 0, 0), (1, 1, 2)),
            ('z', (i32::MAX, 0, 0), (1, 2, 1)),
        ];
        for (axis, position, size) in endpoint_cases {
            let mut schematic = UniversalSchematic::new(format!("endpoint-{axis}"));
            schematic.default_region = Region::new("Main".to_string(), position, size);
            let before = schematic.default_region.get_bounding_box();

            let result = match axis {
                'x' => schematic.rotate_schematic_x(90),
                'y' => schematic.rotate_schematic_y(90),
                'z' => schematic.rotate_schematic_z(90),
                _ => unreachable!(),
            };

            assert!(result.is_err());
            assert_eq!(schematic.default_region.get_bounding_box(), before);
        }
    }

    #[test]
    fn stamp_exclusions_preserve_destination_and_written_cells_replace_block_entities() {
        let mut source = UniversalSchematic::new("source".to_string());
        source
            .set_block_from_string(
                0,
                0,
                0,
                r#"minecraft:chest[facing=east]{CustomName:'"source"'}"#,
            )
            .unwrap();
        source.set_block_str(1, 0, 0, "minecraft:stone");

        let mut destination = UniversalSchematic::new("destination".to_string());
        destination.set_block_str(10, 0, 0, "minecraft:barrel");
        destination.set_block_entity(
            BlockPosition { x: 10, y: 0, z: 0 },
            BlockEntity::new("minecraft:barrel".to_string(), (10, 0, 0)),
        );
        destination.set_block_str(11, 0, 0, "minecraft:gold_block");
        destination.set_block_entity(
            BlockPosition { x: 11, y: 0, z: 0 },
            BlockEntity::new("minecraft:barrel".to_string(), (11, 0, 0)),
        );

        destination
            .stamp_box(
                &source,
                &BoundingBox::new((0, 0, 0), (1, 0, 0)),
                (10, 0, 0),
                &[BlockState::new("minecraft:stone".to_string())],
            )
            .unwrap();

        assert_eq!(
            destination.get_block(10, 0, 0).unwrap().name,
            "minecraft:chest"
        );
        assert!(destination
            .get_block_entity(BlockPosition { x: 10, y: 0, z: 0 })
            .is_some());
        assert_eq!(
            destination.get_block(11, 0, 0).unwrap().name,
            "minecraft:gold_block"
        );
        assert!(destination
            .get_block_entity(BlockPosition { x: 11, y: 0, z: 0 })
            .is_some());

        destination
            .stamp_box(
                &source,
                &BoundingBox::new((1, 0, 0), (1, 0, 0)),
                (10, 0, 0),
                &[],
            )
            .unwrap();
        assert_eq!(
            destination.get_block(10, 0, 0).unwrap().name,
            "minecraft:stone"
        );
        assert!(destination
            .get_block_entity(BlockPosition { x: 10, y: 0, z: 0 })
            .is_none());
    }

    #[test]
    fn stamp_box_writes_source_air_and_stamp_region_selects_one_region() {
        let mut source = UniversalSchematic::new("source".to_string());
        source.set_block_str(0, 0, 0, "minecraft:stone");
        source.set_block_str(2, 0, 0, "minecraft:stone");
        source
            .try_set_block_in_region_str("wing", 4, 0, 0, "minecraft:diamond_block")
            .unwrap();

        let mut destination = UniversalSchematic::new("destination".to_string());
        destination.set_block_str(10, 0, 0, "minecraft:barrel");
        destination.set_block_entity(
            BlockPosition { x: 10, y: 0, z: 0 },
            BlockEntity::new("minecraft:barrel".to_string(), (10, 0, 0)),
        );

        destination
            .stamp_box(
                &source,
                &BoundingBox::new((1, 0, 0), (1, 0, 0)),
                (10, 0, 0),
                &[],
            )
            .unwrap();
        assert_eq!(
            destination.get_block(10, 0, 0).unwrap().name,
            "minecraft:air"
        );
        assert!(destination
            .get_block_entity(BlockPosition { x: 10, y: 0, z: 0 })
            .is_none());

        destination
            .stamp_region(&source, "wing", (20, 0, 0), &[])
            .unwrap();
        assert_eq!(
            destination.get_block(20, 0, 0).unwrap().name,
            "minecraft:diamond_block"
        );
        assert!(destination
            .stamp_region(&source, "missing", (30, 0, 0), &[])
            .is_err());
    }

    #[test]
    fn stamping_overflow_is_transactional() {
        let mut source = UniversalSchematic::new("source".to_string());
        source.set_block_str(0, 0, 0, "minecraft:stone");
        source.set_block_str(1, 0, 0, "minecraft:gold_block");
        let mut destination = UniversalSchematic::new("destination".to_string());
        destination.set_block_str(0, 0, 0, "minecraft:diamond_block");

        let result = destination.stamp_box(
            &source,
            &BoundingBox::new((0, 0, 0), (1, 0, 0)),
            (i32::MAX, 0, 0),
            &[],
        );

        assert!(result.is_err());
        assert_eq!(
            destination.get_block(0, 0, 0).unwrap().name,
            "minecraft:diamond_block"
        );
        assert!(destination.get_block(i32::MAX, 0, 0).is_none());
        assert!(destination.default_region.entities.is_empty());
    }

    #[test]
    fn stamp_region_uses_tight_bounds_after_negative_expansion() {
        let mut source = UniversalSchematic::new("source".to_string());
        source
            .try_set_block_in_region_str("wing", 10, 0, 0, "minecraft:gold_block")
            .unwrap();
        source
            .try_set_block_in_region_str("wing", -1, 0, 0, "minecraft:stone")
            .unwrap();
        let mut destination = UniversalSchematic::new("destination".to_string());
        destination.set_block_str(36, 0, 0, "minecraft:diamond_block");

        destination
            .stamp_region(&source, "wing", (100, 0, 0), &[])
            .unwrap();

        assert_eq!(
            destination.get_block(100, 0, 0).unwrap().name,
            "minecraft:stone"
        );
        assert_eq!(
            destination.get_block(111, 0, 0).unwrap().name,
            "minecraft:gold_block"
        );
        assert_eq!(
            destination.get_block(36, 0, 0).unwrap().name,
            "minecraft:diamond_block"
        );
    }

    #[test]
    fn overlapping_named_regions_have_stable_lexicographic_precedence() {
        let build_source = |reverse: bool| {
            let mut source = UniversalSchematic::new("source".to_string());
            let entries = if reverse {
                [
                    ("zeta", "minecraft:gold_block"),
                    ("alpha", "minecraft:stone"),
                ]
            } else {
                [
                    ("alpha", "minecraft:stone"),
                    ("zeta", "minecraft:gold_block"),
                ]
            };
            for (name, block) in entries {
                source
                    .try_set_block_in_region_str(name, 10, 0, 0, block)
                    .unwrap();
            }
            source
        };

        for source in [build_source(false), build_source(true)] {
            assert_eq!(source.get_region_names(), ["Main", "alpha", "zeta"]);
            assert_eq!(source.get_block(10, 0, 0).unwrap().name, "minecraft:stone");
            assert_eq!(
                source.get_merged_region().get_block(10, 0, 0).unwrap().name,
                "minecraft:stone"
            );
            let mut destination = UniversalSchematic::new("destination".to_string());
            destination
                .stamp_box(
                    &source,
                    &BoundingBox::new((10, 0, 0), (10, 0, 0)),
                    (0, 0, 0),
                    &[],
                )
                .unwrap();
            assert_eq!(
                destination.get_block(0, 0, 0).unwrap().name,
                "minecraft:stone"
            );
        }

        let mut main_wins = build_source(false);
        main_wins.set_block_str(10, 0, 0, "minecraft:diamond_block");
        assert_eq!(
            main_wins
                .get_merged_region()
                .get_block(10, 0, 0)
                .unwrap()
                .name,
            "minecraft:diamond_block"
        );

        let mut main_air_masks = build_source(false);
        main_air_masks.set_block_str(9, 0, 0, "minecraft:stone");
        main_air_masks.set_block_str(11, 0, 0, "minecraft:stone");
        assert_eq!(
            main_air_masks.get_block(10, 0, 0).unwrap().name,
            "minecraft:air"
        );
        assert_eq!(
            main_air_masks
                .get_merged_region()
                .get_block(10, 0, 0)
                .unwrap()
                .name,
            "minecraft:air"
        );

        let mut named_air_masks = UniversalSchematic::new("named-air".to_string());
        named_air_masks.set_block_str(0, 0, 0, "minecraft:stone");
        named_air_masks
            .try_set_block_in_region_str("alpha", 9, 0, 0, "minecraft:stone")
            .unwrap();
        named_air_masks
            .try_set_block_in_region_str("alpha", 11, 0, 0, "minecraft:stone")
            .unwrap();
        named_air_masks
            .try_set_block_in_region_str("zeta", 10, 0, 0, "minecraft:gold_block")
            .unwrap();
        assert_eq!(
            named_air_masks.get_block(10, 0, 0).unwrap().name,
            "minecraft:air"
        );
        assert_eq!(
            named_air_masks
                .get_merged_region()
                .get_block(10, 0, 0)
                .unwrap()
                .name,
            "minecraft:air"
        );

        let mut padded_main = UniversalSchematic::new("padded-main".to_string());
        padded_main.set_block_str(0, 0, 0, "minecraft:stone");
        padded_main.set_block_str(-1, 0, 0, "minecraft:stone");
        padded_main
            .try_set_block_in_region_str("alpha", -10, 0, 0, "minecraft:diamond_block")
            .unwrap();

        let mut destination = UniversalSchematic::new("destination".to_string());
        destination
            .stamp_box(
                &padded_main,
                &BoundingBox::new((-10, 0, 0), (-10, 0, 0)),
                (0, 0, 0),
                &[],
            )
            .unwrap();
        assert_eq!(
            destination.get_block(0, 0, 0).unwrap().name,
            "minecraft:diamond_block"
        );
    }

    #[test]
    fn copy_region_fast_path_matches_slow_path() {
        // The single-region source takes the palette-mapped fast path; adding
        // a dummy other-region forces the per-block slow path. Both must
        // produce identical targets, including exclusions, air overwrites,
        // and cells outside the source region staying untouched.
        let build_source = |extra_region: bool| {
            let mut src = UniversalSchematic::new("src".to_string());
            src.set_block_str(0, 0, 0, "minecraft:stone");
            src.set_block_str(1, 0, 0, "minecraft:dirt");
            src.set_block_str(2, 1, 1, "minecraft:stone");
            let _ = src.set_block_from_string(1, 1, 0, "minecraft:repeater[delay=2,facing=east]");
            if extra_region {
                src.set_block_in_region_str("Extra", 50, 50, 50, "minecraft:gold_block");
            }
            src
        };

        let build_target = || {
            let mut t = UniversalSchematic::new("target".to_string());
            // Pre-existing block that the copy must overwrite with air:
            // (12,2,7) maps back to source cell (2,0,0), which is air inside
            // the source region (offset = target_position - bounds.min).
            t.set_block_str(12, 2, 7, "minecraft:diamond_block");
            // Pre-existing block outside the copied box: must survive.
            t.set_block_str(-5, -5, -5, "minecraft:emerald_block");
            t
        };

        // Bounds deliberately larger than the source region on every side.
        let bounds = BoundingBox::new((-2, -2, -2), (6, 6, 6));
        let excluded = vec![BlockState::new("minecraft:dirt".to_string())];

        let mut fast = build_target();
        fast.copy_region(&build_source(false), &bounds, (8, 0, 5), &excluded)
            .unwrap();

        let mut slow = build_target();
        slow.copy_region(&build_source(true), &bounds, (8, 0, 5), &excluded)
            .unwrap();

        // Normalize unallocated (None) and allocated-air to the same value:
        // the two paths allocate different padding around the copy, which is
        // an implementation detail, not content.
        let norm = |b: Option<&BlockState>| match b {
            Some(b) if b.name != "minecraft:air" => Some(b.to_string()),
            _ => None,
        };
        for x in -8..20 {
            for y in -8..10 {
                for z in -8..10 {
                    let f = norm(fast.get_block(x, y, z));
                    let s = norm(slow.get_block(x, y, z));
                    assert_eq!(f, s, "mismatch at ({}, {}, {})", x, y, z);
                }
            }
        }
        // Sanity: the copy actually landed and exclusion produced air.
        // Source (0,0,0) + offset (10,2,7) = target (10,2,7).
        assert_eq!(
            fast.get_block(10, 2, 7).map(|b| b.name.as_str()),
            Some("minecraft:stone")
        );
        assert_ne!(
            fast.get_block(11, 2, 7).map(|b| b.name.as_str()),
            Some("minecraft:dirt")
        );
        assert_ne!(
            fast.get_block(12, 2, 7).map(|b| b.name.as_str()),
            Some("minecraft:diamond_block")
        );
        assert_eq!(
            fast.get_block(-5, -5, -5).map(|b| b.name.as_str()),
            Some("minecraft:emerald_block")
        );
    }

    #[test]
    fn litematic_load_captures_source_data_version() {
        // Stamp a known data version, round-trip through litematic, and confirm
        // the importer captures it as the source version for conversion.
        let mut schematic = UniversalSchematic::new("VersionTest".to_string());
        schematic.metadata.mc_version = Some(1343);
        schematic.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));

        let bytes = crate::formats::litematic::to_litematic(&schematic).expect("write litematic");
        let loaded = crate::formats::litematic::from_litematic(&bytes).expect("read litematic");
        assert_eq!(loaded.metadata.source_data_version, Some(1343));
    }

    #[test]
    fn convert_to_data_version_forward_then_reverse_round_trips() {
        // melon_block -> melon at V1490; forward to canonical-ish, then back.
        let mut schematic = UniversalSchematic::new("ConvTest".to_string());
        schematic.metadata.source_data_version = Some(1489);
        schematic.set_block(
            0,
            0,
            0,
            &BlockState::new("minecraft:melon_block".to_string()),
        );

        let report = schematic.convert_to_data_version(1490);
        assert!(report.is_empty(), "forward rename is lossless");
        assert_eq!(
            schematic.get_block(0, 0, 0).unwrap().get_name(),
            "minecraft:melon"
        );
        assert_eq!(schematic.metadata.source_data_version, Some(1490));

        let report = schematic.convert_to_data_version(1489);
        assert!(report.is_empty(), "reverse rename is lossless");
        assert_eq!(
            schematic.get_block(0, 0, 0).unwrap().get_name(),
            "minecraft:melon_block"
        );
    }

    #[test]
    fn test_schematic_operations() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());

        // Test automatic region creation and expansion
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        assert!(schematic.set_block(0, 0, 0, &stone));
        assert_eq!(schematic.get_block(0, 0, 0), Some(&stone));

        assert!(schematic.set_block(5, 5, 5, &dirt));
        assert_eq!(schematic.get_block(5, 5, 5), Some(&dirt));

        // Check that the default region was expanded
        assert_eq!(schematic.get_region("Main").unwrap().name, "Main");

        // Test explicit region creation and manipulation
        let obsidian = BlockState::new("minecraft:obsidian".to_string());
        assert!(schematic.set_block_in_region("Custom", 10, 10, 10, &obsidian));
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
        schematic.set_block(0, 0, 0, &stone);

        // Create and add another region
        let mut custom_region = Region::new("Custom".to_string(), (10, 10, 10), (5, 5, 5));
        let dirt = BlockState::new("minecraft:dirt".to_string());
        custom_region.set_block(10, 10, 10, &dirt);
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
        new_region.set_block(5, 5, 5, &gold);

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

        schematic.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));
        schematic.set_block(
            4,
            4,
            4,
            &BlockState::new("minecraft:sea_lantern".to_string()),
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
        assert!(schematic.set_block(1000, 1000, 1000, &far_block));
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

        assert!(schematic.set_block(0, 0, 0, &block1));
        assert!(schematic.set_block(10, 20, 30, &block2));

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
        source.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));
        source.set_block(1, 1, 1, &BlockState::new("minecraft:dirt".to_string()));
        source.set_block(
            2,
            2,
            2,
            &BlockState::new("minecraft:diamond_block".to_string()),
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
        let gold = BlockState::new("minecraft:gold_block".to_string());

        // Create a 2x2x2 cube with different blocks
        source.set_block(0, 0, 0, &stone);
        source.set_block(0, 1, 0, &dirt);
        source.set_block(1, 0, 0, &diamond);
        source.set_block(1, 1, 0, &dirt);

        // Create target schematic and preload the cells covered by exclusions.
        let mut target = UniversalSchematic::new("Target".to_string());
        target.set_block(10, 10, 10, &gold);
        target.set_block(11, 10, 10, &gold);

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

        // Excluded source cells preserve existing destination content.
        assert_eq!(
            target.get_block(10, 10, 10),
            Some(&gold),
            "Stone exclusion should preserve the destination"
        );
        assert_eq!(
            target.get_block(11, 10, 10),
            Some(&gold),
            "Diamond exclusion should preserve the destination"
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
        assert!(schematic.set_block(-10, -10, -10, &neg_block));
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
    fn get_entities_as_list_includes_non_default_regions() {
        // Regression: a multi-region .litematic can put its entity in a region
        // other than the (HashMap-arbitrary) default one. get_entities_as_list —
        // which the WASM `get_entities()` exporter relies on — must enumerate
        // every region, or the entity is silently (and non-deterministically)
        // dropped on load. See formats/litematic.rs region parsing.
        let mut schematic = UniversalSchematic::new("Multi".to_string());
        // Default region stays empty (like a "lever" sub-region with no entities).
        let mut other = Region::new("with_entity".to_string(), (0, 0, 0), (4, 4, 4));
        other.add_entity(Entity::new("minecraft:boat".to_string(), (1.5, 0.0, 1.5)));
        assert!(schematic.add_region(other));

        assert_eq!(schematic.default_region.entities.len(), 0);
        let all = schematic.get_entities_as_list();
        assert_eq!(
            all.len(),
            1,
            "entity in a non-default region must be listed"
        );
        assert_eq!(all[0].id, "minecraft:boat");
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

        region.set_block(0, 0, 0, &stone);
        region.set_block(0, 1, 0, &dirt);
        region.set_block(1, 0, 0, &stone);

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
        schematic.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));
        schematic.set_block(1, 1, 1, &BlockState::new("minecraft:dirt".to_string()));
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
        region1.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));
        region1.set_block(1, 1, 1, &BlockState::new("minecraft:dirt".to_string()));
        region2.set_block(
            0,
            -1,
            -1,
            &BlockState::new("minecraft:gold_block".to_string()),
        );

        // Add a block to the default region
        schematic.set_block(
            2,
            2,
            2,
            &BlockState::new("minecraft:diamond_block".to_string()),
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
    fn test_content_shorthands_create_block_entities_and_jukebox_state() {
        assert!(UniversalSchematic::parse_block_string("minecraft:chest{items=[diamond]").is_err());
        assert!(UniversalSchematic::parse_block_string(
            "minecraft:chest[facing=north{items=[diamond]}"
        )
        .is_err());
        assert!(UniversalSchematic::parse_block_string("minecraft:stone}").is_err());
        assert!(UniversalSchematic::parse_block_string("minecraft:stone[=north]").is_err());
        assert!(
            UniversalSchematic::parse_block_string("minecraft:stone[facing=north=extra]").is_err()
        );

        let mut invalid = UniversalSchematic::new("Invalid".to_string());
        assert!(invalid
            .try_set_block_str(0, 0, 0, "minecraft:chest{signal=bogus}")
            .is_err());
        assert!(!invalid.set_block_str(0, 0, 0, "minecraft:chest{signal=bogus}"));
        assert_ne!(
            invalid.get_block(0, 0, 0).map(BlockState::get_name),
            Some("minecraft:chest")
        );
        assert!(invalid
            .get_block_entity(BlockPosition { x: 0, y: 0, z: 0 })
            .is_none());

        let mut schematic = UniversalSchematic::new("Contents".to_string());
        schematic
            .set_block_from_string(0, 0, 0, "minecraft:chest{items=[diamond*64,emerald*12]}")
            .unwrap();
        schematic
            .set_block_from_string(1, 0, 0, "minecraft:jukebox{record=pigstep}")
            .unwrap();

        let chest = schematic
            .get_block_entity(BlockPosition { x: 0, y: 0, z: 0 })
            .unwrap();
        let Some(NbtValue::List(items)) = chest.nbt.get("Items") else {
            panic!("chest must contain an Items list");
        };
        assert_eq!(items.len(), 2);

        let jukebox = schematic.get_block(1, 0, 0).unwrap();
        assert_eq!(
            jukebox.get_property("has_record"),
            Some(&SmolStr::from("true"))
        );
        let jukebox_entity = schematic
            .get_block_entity(BlockPosition { x: 1, y: 0, z: 0 })
            .unwrap();
        let Some(NbtValue::Compound(record)) = jukebox_entity.nbt.get("RecordItem") else {
            panic!("jukebox must contain a RecordItem compound");
        };
        assert_eq!(
            record.get("id"),
            Some(&NbtValue::String(
                "minecraft:music_disc_pigstep".to_string()
            ))
        );

        schematic
            .set_block_from_string(2, 0, 0, "minecraft:jukebox[has_record=true]{signal=0}")
            .unwrap();
        assert_eq!(
            schematic
                .get_block(2, 0, 0)
                .unwrap()
                .get_property("has_record"),
            Some(&SmolStr::from("false"))
        );
        assert!(schematic
            .get_block_entity(BlockPosition { x: 2, y: 0, z: 0 })
            .is_none());

        schematic
            .set_block_from_string(5, 0, 0, "minecraft:jukebox{record=pigstep}")
            .unwrap();
        assert!(schematic
            .get_block_entity(BlockPosition { x: 5, y: 0, z: 0 })
            .is_some());
        schematic
            .set_block_from_string(5, 0, 0, "minecraft:jukebox{signal=0}")
            .unwrap();
        assert_eq!(
            schematic
                .get_block(5, 0, 0)
                .unwrap()
                .get_property("has_record"),
            Some(&SmolStr::from("false"))
        );
        assert!(schematic
            .get_block_entity(BlockPosition { x: 5, y: 0, z: 0 })
            .is_none());

        schematic
            .set_block_from_string(6, 0, 0, "minecraft:chest{items=[diamond]}")
            .unwrap();
        assert!(schematic
            .get_block_entity(BlockPosition { x: 6, y: 0, z: 0 })
            .is_some());
        assert!(schematic.set_block_str(6, 0, 0, "minecraft:stone"));
        assert!(schematic
            .get_block_entity(BlockPosition { x: 6, y: 0, z: 0 })
            .is_none());

        schematic
            .set_block_from_string(3, 0, 0, "mod:jukebox[has_record=true]")
            .unwrap();
        assert_eq!(
            schematic
                .get_block(3, 0, 0)
                .unwrap()
                .get_property("has_record"),
            Some(&SmolStr::from("true")),
            "custom ids containing jukebox must not receive vanilla synchronization"
        );
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
            assert_eq!(total_items, expected_items);
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
        assert_eq!(block.get_property("facing"), Some(&SmolStr::from("up")));

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
            assert_eq!(total_items, expected_items);
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
                            &BlockState::new("minecraft:stone".to_string()),
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
            !chunks.is_empty(),
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
                    schematic.set_block(x, y, z, &BlockState::new("minecraft:stone".to_string()));
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
                    schematic2.set_block(x, y, z, &BlockState::new("minecraft:stone".to_string()));
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
                    schematic3.set_block(x, y, z, &BlockState::new("minecraft:stone".to_string()));
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
        schematic4.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));
        schematic4.set_block(31, 0, 0, &BlockState::new("minecraft:stone".to_string()));
        schematic4.set_block(0, 31, 0, &BlockState::new("minecraft:stone".to_string()));
        schematic4.set_block(0, 0, 31, &BlockState::new("minecraft:stone".to_string()));
        schematic4.set_block(31, 31, 31, &BlockState::new("minecraft:stone".to_string()));

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
            !chunks4.is_empty(),
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
    #[allow(clippy::approx_constant)]
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
