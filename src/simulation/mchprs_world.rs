use crate::block_entity::BlockEntity as UtilBlockEntity;
use crate::UniversalSchematic;
use mchprs_blocks::{block_entities::BlockEntity, blocks::Block, BlockPos};
use mchprs_redpiler::{BackendVariant, Compiler, CompilerOptions};
use mchprs_world::{storage::Chunk, TickEntry, TickPriority, World};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MchprsWorldError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),
    #[error("Invalid block operation: {0}")]
    InvalidOperation(String),
}

/// Configuration options for MCHPRS simulation
#[derive(Debug, Clone)]
pub struct SimulationOptions {
    /// Enable optimization in the compiler
    pub optimize: bool,
    /// When true, only inputs/outputs are tracked (faster but no intermediate wire states)
    /// When false, all redstone wire states are updated (slower but shows wire power levels)
    pub io_only: bool,
    /// List of positions to designate as custom IO nodes for signal injection/monitoring
    pub custom_io: Vec<BlockPos>,
}

impl Default for SimulationOptions {
    fn default() -> Self {
        Self {
            optimize: true, // Default to true for performance
            io_only: false, // Default to false so wire states are visible when optimize=false
            custom_io: Vec::new(),
        }
    }
}

/// Custom IO state change event
#[derive(Debug, Clone)]
pub struct CustomIoChange {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub old_power: u8,
    pub new_power: u8,
}

pub struct MchprsWorld {
    pub(crate) schematic: UniversalSchematic,
    chunks: HashMap<(i32, i32), Chunk>,
    to_be_ticked: Vec<TickEntry>,
    compiler: Compiler,
    options: SimulationOptions,
    /// Previous state of custom IO positions for change detection
    custom_io_states: HashMap<BlockPos, u8>,
    /// Queue of custom IO changes since last poll
    custom_io_changes: Vec<CustomIoChange>,
    /// Minimum coordinates of the schematic for coordinate normalization
    /// MCHPRS expects coordinates starting from 0, so we normalize schematic coords
    min_coords: (i32, i32, i32),
}

impl MchprsWorld {
    /// Creates a new MchprsWorld from a UniversalSchematic with default options
    ///
    /// # Errors
    /// Returns an error if chunk initialization or compilation fails
    pub fn new(schematic: UniversalSchematic) -> Result<Self, String> {
        Self::with_options(schematic, SimulationOptions::default())
    }

    /// Creates a new MchprsWorld from a UniversalSchematic with custom options
    ///
    /// # Errors
    /// Returns an error if chunk initialization or compilation fails
    pub fn with_options(
        schematic: UniversalSchematic,
        options: SimulationOptions,
    ) -> Result<Self, String> {
        let min_coords = schematic.get_bounding_box().min;
        let mut world = MchprsWorld {
            schematic,
            chunks: HashMap::new(),
            to_be_ticked: Vec::new(),
            compiler: {
                let c = Compiler::default();
                // Initialize backend for WASM
                #[cfg(target_arch = "wasm32")]
                {
                    use mchprs_redpiler::backend::BackendDispatcher;
                    let mut c = c;
                    c.use_jit(BackendDispatcher::DirectBackend(Default::default()));
                    c
                }
                #[cfg(not(target_arch = "wasm32"))]
                c
            },
            options,
            custom_io_states: HashMap::new(),
            custom_io_changes: Vec::new(),
            min_coords,
        };

        world.initialize_chunks()?;
        world.populate_chunks();
        // NOTE: Don't replace custom IO wires with redstone blocks - that makes them always powered
        // Instead, we keep them as wires and modify the identify_nodes pass to convert them to Constants
        world.update_redstone();

        // Process any scheduled ticks (like comparator updates) before compilation
        // This ensures comparators have read their input containers and set their output strength
        world.process_scheduled_ticks();

        world.initialize_compiler()?;
        Ok(world)
    }

    fn initialize_compiler(&mut self) -> Result<(), String> {
        let bounding_box = self.schematic.get_bounding_box();
        // Normalize coordinates to start from (0, 0, 0) for MCHPRS
        let bounds = (
            BlockPos::new(0, 0, 0),
            BlockPos::new(
                bounding_box.max.0 - self.min_coords.0,
                bounding_box.max.1 - self.min_coords.1,
                bounding_box.max.2 - self.min_coords.2,
            ),
        );

        // Normalize custom IO positions for the compiler
        let normalized_custom_io: Vec<BlockPos> = self
            .options
            .custom_io
            .iter()
            .map(|&pos| self.normalize_pos(pos))
            .collect();

        let compiler_options = CompilerOptions {
            optimize: self.options.optimize,
            io_only: self.options.io_only,
            wire_dot_out: true,
            backend_variant: BackendVariant::Direct,
            custom_io: normalized_custom_io,
            ..Default::default()
        };

        let ticks = self.to_be_ticked.drain(..).collect();
        let monitor = Default::default();

        let mut temp_compiler = std::mem::take(&mut self.compiler);

        // Try compilation with error handling
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            temp_compiler.compile(self, bounds, compiler_options, ticks, monitor)
        })) {
            Ok(_) => {
                self.compiler = temp_compiler;
                Ok(())
            }
            Err(e) => {
                self.compiler = temp_compiler; // Restore compiler even on error
                let error_msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown compilation error".to_string()
                };
                Err(error_msg)
            }
        }
    }

    fn initialize_chunks(&mut self) -> Result<(), String> {
        let bounding_box = self.schematic.get_bounding_box();
        let (min_x, min_y, min_z) = (bounding_box.min.0, bounding_box.min.1, bounding_box.min.2);
        let (max_x, max_y, max_z) = (bounding_box.max.0, bounding_box.max.1, bounding_box.max.2);

        // Normalize coordinates to MCHPRS space (0-based)
        let norm_min_x = 0;
        let norm_min_z = 0;
        let norm_max_x = max_x - min_x;
        let norm_max_z = max_z - min_z;

        for chunk_x in (norm_min_x >> 4)..=((norm_max_x >> 4) + 1) {
            for chunk_z in (norm_min_z >> 4)..=((norm_max_z >> 4) + 1) {
                let chunk = Chunk::empty(chunk_x, chunk_z, ((max_y - min_y) / 16 + 1) as usize);
                self.chunks.insert((chunk_x, chunk_z), chunk);
            }
        }

        if self.chunks.is_empty() {
            return Err("No chunks initialized".to_string());
        }

        Ok(())
    }

    fn populate_chunks(&mut self) {
        // Collect all block data first to avoid borrow issues
        let block_data: Vec<_> = self
            .schematic
            .iter_blocks()
            .map(|(pos, block_state)| {
                let name = block_state
                    .name
                    .strip_prefix("minecraft:")
                    .unwrap_or(&block_state.name)
                    .to_string();
                let properties = block_state.properties.clone();
                let block_entity =
                    if Block::from_name(&name).map_or(false, |b| b.has_block_entity()) {
                        self.schematic.get_block_entity(pos.clone()).cloned()
                    } else {
                        None
                    };
                (
                    // Normalize coordinates relative to schematic minimum
                    BlockPos::new(
                        pos.x - self.min_coords.0,
                        pos.y - self.min_coords.1,
                        pos.z - self.min_coords.2,
                    ),
                    name,
                    properties,
                    block_entity,
                )
            })
            .collect();

        // Now populate the world with the collected data
        for (pos, name, properties, maybe_block_entity) in block_data {
            if let Some(mut block) = Block::from_name(&name) {
                let properties_ref: HashMap<&str, &str> = properties
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();

                block.set_properties(properties_ref);
                self.set_block_raw(pos, block.get_id());

                if let Some(util_block_entity) = maybe_block_entity {
                    if let Some(block_entity) = self.convert_block_entity(util_block_entity) {
                        self.set_block_entity(pos, block_entity);
                    }
                }
            } else {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::warn_1(&format!("Unknown block '{}' at {:?}", name, pos).into());
                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("Warning: Unknown block '{}' at position {:?}", name, pos);
            }
        }
    }

    fn convert_block_entity(&self, block_entity: UtilBlockEntity) -> Option<BlockEntity> {
        // Convert our NbtValue HashMap to mchprs's nbt::Value HashMap
        use crate::utils::NbtValue;
        use nbt;
        use std::collections::HashMap as StdHashMap;

        let nbt_hashmap = block_entity.to_hashmap();
        let mut converted: StdHashMap<String, nbt::Value> = StdHashMap::new();

        for (key, value) in nbt_hashmap {
            // Convert to legacy format for MCHPRS compatibility (Count: Byte instead of count: Int)
            let legacy_value = value.to_legacy_item_format();
            // Convert NbtValue to hematite_nbt::Value
            if let Some(converted_value) = Self::convert_nbt_value(legacy_value) {
                converted.insert(key, converted_value);
            }
        }

        BlockEntity::from_nbt(&block_entity.id, &converted)
    }

    fn replace_custom_io_wires_with_blocks(&mut self) {
        // Replace redstone wires at custom IO positions with redstone blocks
        // This is necessary because the Redpiler's input_search pass only creates
        // outgoing edges for blocks that unconditionally provide power (like redstone blocks)
        // whereas wires only conditionally provide power based on their current level
        use mchprs_blocks::blocks::Block;

        // Clone to avoid borrow checker issues
        let custom_io_positions = self.options.custom_io.clone();
        for pos in custom_io_positions {
            let block = self.get_block(pos);
            if matches!(block, Block::RedstoneWire { .. }) {
                // Replace with redstone block so it can act as a power source
                self.set_block(pos, Block::RedstoneBlock {});
            }
        }
    }

    fn convert_nbt_value(value: crate::utils::NbtValue) -> Option<nbt::Value> {
        use crate::utils::NbtValue;
        use nbt::Value;

        match value {
            NbtValue::Byte(b) => Some(Value::Byte(b)),
            NbtValue::Short(s) => Some(Value::Short(s)),
            NbtValue::Int(i) => Some(Value::Int(i)),
            NbtValue::Long(l) => Some(Value::Long(l)),
            NbtValue::Float(f) => Some(Value::Float(f)),
            NbtValue::Double(d) => Some(Value::Double(d)),
            NbtValue::String(s) => Some(Value::String(s)),
            NbtValue::ByteArray(ba) => Some(Value::ByteArray(ba)),
            NbtValue::IntArray(ia) => Some(Value::IntArray(ia)),
            NbtValue::LongArray(la) => Some(Value::LongArray(la)),
            NbtValue::List(list) => {
                let converted: Vec<_> = list
                    .into_iter()
                    .filter_map(Self::convert_nbt_value)
                    .collect();
                Some(Value::List(converted))
            }
            NbtValue::Compound(map) => {
                let mut converted = std::collections::HashMap::new();
                for (k, v) in map {
                    if let Some(cv) = Self::convert_nbt_value(v) {
                        converted.insert(k, cv);
                    }
                }
                Some(Value::Compound(converted))
            }
        }
    }

    fn get_chunk_key(&self, pos: BlockPos) -> (i32, i32) {
        (pos.x >> 4, pos.z >> 4)
    }

    /// Normalizes a position from schematic coordinates to MCHPRS coordinates
    /// MCHPRS expects coordinates starting from 0, while schematics may have negative coords
    fn normalize_pos(&self, pos: BlockPos) -> BlockPos {
        BlockPos::new(
            pos.x - self.min_coords.0,
            pos.y - self.min_coords.1,
            pos.z - self.min_coords.2,
        )
    }

    /// Denormalizes a position from MCHPRS coordinates back to schematic coordinates
    fn denormalize_pos(&self, pos: BlockPos) -> BlockPos {
        BlockPos::new(
            pos.x + self.min_coords.0,
            pos.y + self.min_coords.1,
            pos.z + self.min_coords.2,
        )
    }

    /// Updates redstone state for all blocks in the schematic
    pub fn update_redstone(&mut self) {
        let bounding_box = self.schematic.get_bounding_box();
        let (min_x, min_y, min_z) = self.min_coords;
        let (max_x, max_y, max_z) = bounding_box.max;

        // Iterate using actual schematic coordinates, then normalize for MCHPRS
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let schematic_pos = BlockPos::new(x, y, z);
                    let normalized_pos = self.normalize_pos(schematic_pos);
                    let block = self.get_block(normalized_pos);
                    mchprs_redstone::update(block, self, normalized_pos);
                }
            }
        }
    }

    /// Processes all scheduled ticks until none remain
    /// This is useful during initialization to let comparators and other blocks update
    fn process_scheduled_ticks(&mut self) {
        // Process ticks in multiple passes to handle cascading updates
        // Limit to prevent infinite loops
        for _ in 0..100 {
            if self.to_be_ticked.is_empty() {
                break;
            }

            // Sort by priority (higher priority first)
            self.to_be_ticked
                .sort_by(|a, b| b.tick_priority.cmp(&a.tick_priority));

            // Take all ticks to process this pass
            let ticks_to_process: Vec<_> = self.to_be_ticked.drain(..).collect();

            for tick in ticks_to_process {
                let block = self.get_block(tick.pos);
                mchprs_redstone::tick(block, self, tick.pos);
            }
        }
    }

    /// Gets the redstone power level at a position (in schematic coordinates)
    pub fn get_redstone_power(&self, pos: BlockPos) -> u8 {
        let normalized_pos = self.normalize_pos(pos);
        self.get_block(normalized_pos)
            .properties()
            .get("power")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Sets the redstone power level at a position (for redstone wire, in schematic coordinates)
    pub fn set_redstone_power(&mut self, pos: BlockPos, power: u8) {
        let normalized_pos = self.normalize_pos(pos);
        let mut block = self.get_block(normalized_pos);
        if block.get_name() == "redstone_wire" {
            let mut properties: HashMap<&str, String> = block
                .properties()
                .iter()
                .map(|(k, v)| (*k, v.to_string()))
                .collect();
            properties.insert("power", power.to_string());
            block.set_properties(properties.iter().map(|(k, v)| (*k, v.as_str())).collect());
            self.set_block_raw(normalized_pos, block.get_id());
        }
    }

    /// Sets the signal strength at a specific block position (for custom IO nodes, in schematic coordinates)
    ///
    /// # Arguments
    /// * `pos` - The block position in schematic coordinates
    /// * `strength` - The signal strength (0-15)
    pub fn set_signal_strength(&mut self, pos: BlockPos, strength: u8) {
        let normalized_pos = self.normalize_pos(pos);
        self.compiler.set_signal_strength(normalized_pos, strength);
    }

    /// Gets the signal strength at a specific block position (for custom IO nodes, in schematic coordinates)
    ///
    /// # Arguments
    /// * `pos` - The block position in schematic coordinates
    ///
    /// # Returns
    /// The current signal strength (0-15), or 0 if not a valid component
    pub fn get_signal_strength(&self, pos: BlockPos) -> u8 {
        let normalized_pos = self.normalize_pos(pos);
        self.compiler
            .get_signal_strength(normalized_pos)
            .unwrap_or(0)
    }

    /// Sets signal strength for multiple positions at once (batch operation)
    ///
    /// This is more efficient than calling set_signal_strength multiple times
    /// as it reduces the number of function calls across the WASM boundary.
    ///
    /// # Arguments
    /// * `positions` - Array of (x, y, z) tuples in schematic coordinates
    /// * `strengths` - Array of signal strengths (0-15), must match positions length
    ///
    /// # Errors
    /// Returns an error if positions and strengths arrays have different lengths
    pub fn set_signals_batch(
        &mut self,
        positions: &[(i32, i32, i32)],
        strengths: &[u8],
    ) -> Result<(), String> {
        if positions.len() != strengths.len() {
            return Err(format!(
                "Position count ({}) doesn't match strength count ({})",
                positions.len(),
                strengths.len()
            ));
        }

        for (i, &(x, y, z)) in positions.iter().enumerate() {
            let pos = BlockPos::new(x, y, z);
            let strength = strengths[i];
            self.set_signal_strength(pos, strength);
        }

        Ok(())
    }

    /// Gets signal strength for multiple positions at once (batch operation)
    ///
    /// This is more efficient than calling get_signal_strength multiple times
    /// as it reduces the number of function calls across the WASM boundary.
    ///
    /// # Arguments
    /// * `positions` - Array of (x, y, z) tuples in schematic coordinates
    ///
    /// # Returns
    /// Array of signal strengths (0-15) corresponding to each position
    pub fn get_signals_batch(&self, positions: &[(i32, i32, i32)]) -> Vec<u8> {
        positions
            .iter()
            .map(|&(x, y, z)| {
                let pos = BlockPos::new(x, y, z);
                self.get_signal_strength(pos)
            })
            .collect()
    }

    /// Checks if a position has a node in the redpiler graph (DEBUG)
    #[cfg(test)]
    pub fn has_node(&self, pos: BlockPos) -> bool {
        self.compiler.get_signal_strength(pos).is_some()
    }

    /// Simulates a right-click on a block (typically a lever, in schematic coordinates)
    ///
    /// # Errors
    /// Logs a warning if the block is not a lever
    pub fn on_use_block(&mut self, pos: BlockPos) {
        let normalized_pos = self.normalize_pos(pos);
        let block = self.get_block(normalized_pos);
        if block.get_name() == "lever" {
            let current_state = self.get_lever_power(pos);
            self.set_lever_power(pos, !current_state);
            self.compiler.on_use_block(normalized_pos);
            return;
        }

        #[cfg(target_arch = "wasm32")]
        web_sys::console::error_1(&format!("Tried to use non-lever block at {:?}", pos).into());
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!(
            "Error: Tried to use block at {:?} which is not a lever",
            pos
        );
    }

    /// Advances the simulation by the specified number of ticks
    pub fn tick(&mut self, number_of_ticks: u32) {
        for _ in 0..number_of_ticks {
            self.compiler.tick();
        }
    }

    /// Flushes pending changes from the compiler to the world
    pub fn flush(&mut self) {
        let mut temp_compiler = std::mem::take(&mut self.compiler);
        temp_compiler.flush(self);
        self.compiler = temp_compiler;

        // Only call update_redstone() for non-custom-IO circuits
        // For custom IO circuits, update_redstone() would recalculate wire power from inputs,
        // overwriting the redpiler's flushed state (including custom IO injected values)
        if self.options.custom_io.is_empty() {
            self.update_redstone();
        }
    }

    /// Checks if a redstone lamp is lit at the given position (in schematic coordinates)
    pub fn is_lit(&self, pos: BlockPos) -> bool {
        let normalized_pos = self.normalize_pos(pos);
        self.get_block(normalized_pos)
            .properties()
            .get("lit")
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    /// Sets the power state of a lever (in schematic coordinates)
    pub fn set_lever_power(&mut self, pos: BlockPos, powered: bool) {
        let normalized_pos = self.normalize_pos(pos);
        let mut block = self.get_block(normalized_pos);
        let mut properties: HashMap<&str, String> = block
            .properties()
            .iter()
            .map(|(k, v)| (*k, v.to_string()))
            .collect();
        properties.insert("powered", powered.to_string());
        block.set_properties(properties.iter().map(|(k, v)| (*k, v.as_str())).collect());
        self.set_block_raw(normalized_pos, block.get_id());
    }

    /// Gets the power state of a lever (in schematic coordinates)
    pub fn get_lever_power(&self, pos: BlockPos) -> bool {
        let normalized_pos = self.normalize_pos(pos);
        self.get_block(normalized_pos)
            .properties()
            .get("powered")
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    /// Syncs the current simulation state back to the UniversalSchematic
    ///
    /// This updates all block states (including redstone power levels, lever states, lamp states, etc.)
    /// from the MCHPRS simulation back to the schematic.
    ///
    /// Call this after running simulation if you want to export the resulting state.
    pub fn sync_to_schematic(&mut self) {
        self.flush();
        let bounding_box = self.schematic.get_bounding_box();
        let (min_x, min_y, min_z) = self.min_coords;
        let (max_x, max_y, max_z) = bounding_box.max;
        let custom_io_set: std::collections::HashSet<_> =
            self.options.custom_io.iter().cloned().collect();

        // Iterate using actual schematic coordinates
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    // Normalize position for MCHPRS access
                    let normalized_pos = self.normalize_pos(BlockPos::new(x, y, z));
                    let raw_id = self.get_block_raw(normalized_pos);
                    let block = self.get_block(normalized_pos);

                    // Skip air blocks
                    if block.get_id() == 0 {
                        continue;
                    }

                    // Get block name with minecraft: prefix
                    let name = format!("minecraft:{}", block.get_name());

                    // Get all properties from the MCHPRS block
                    let properties: std::collections::HashMap<String, String> = block
                        .properties()
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect();

                    // Create BlockState and update schematic
                    let mut block_state = crate::BlockState::new(name);
                    block_state.properties = properties;

                    self.schematic.set_block(x, y, z, block_state);
                }
            }
        }
    }

    /// Gets a reference to the underlying schematic
    ///
    /// Note: This returns the schematic in its current state.
    /// Call `sync_to_schematic()` first if you want the latest simulation state.
    pub fn get_schematic(&self) -> &UniversalSchematic {
        &self.schematic
    }

    /// Consumes the MchprsWorld and returns the schematic with the current simulation state
    ///
    /// This automatically syncs the simulation state before returning the schematic.
    pub fn into_schematic(mut self) -> UniversalSchematic {
        self.sync_to_schematic();
        self.schematic
    }

    /// Check for custom IO state changes and queue them
    /// Call this after tick() or set_signal_strength() to detect changes
    ///
    /// **Performance**: This is a no-op if there are no custom IO positions,
    /// so it has zero overhead for circuits without custom IO.
    pub fn check_custom_io_changes(&mut self) {
        // Early return if no custom IO - zero overhead!
        if self.options.custom_io.is_empty() {
            return;
        }

        // Check each custom IO position for changes
        for pos in &self.options.custom_io {
            let current_power = self.get_signal_strength(*pos);
            let previous_power = self.custom_io_states.get(pos).copied().unwrap_or(255); // 255 = uninitialized

            if current_power != previous_power && previous_power != 255 {
                // State changed, queue the change
                self.custom_io_changes.push(CustomIoChange {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                    old_power: previous_power,
                    new_power: current_power,
                });
            }

            // Update tracked state
            self.custom_io_states.insert(*pos, current_power);
        }
    }

    /// Get and clear all custom IO changes since last poll
    /// Returns a vector of changes in the order they occurred
    ///
    /// # Example
    /// ```ignore
    /// // After ticking
    /// world.tick(5);
    /// world.check_custom_io_changes();
    ///
    /// // Poll for changes
    /// let changes = world.poll_custom_io_changes();
    /// for change in changes {
    ///     println!("IO at ({},{},{}) changed: {} -> {}",
    ///         change.x, change.y, change.z, change.old_power, change.new_power);
    /// }
    /// ```
    pub fn poll_custom_io_changes(&mut self) -> Vec<CustomIoChange> {
        std::mem::take(&mut self.custom_io_changes)
    }

    /// Get custom IO changes without clearing the queue
    /// Useful for inspecting changes without consuming them
    pub fn peek_custom_io_changes(&self) -> &[CustomIoChange] {
        &self.custom_io_changes
    }

    /// Clear all queued custom IO changes without returning them
    pub fn clear_custom_io_changes(&mut self) {
        self.custom_io_changes.clear();
    }
}

impl World for MchprsWorld {
    fn get_block_raw(&self, pos: BlockPos) -> u32 {
        let chunk_key = self.get_chunk_key(pos);
        if let Some(chunk) = self.chunks.get(&chunk_key) {
            chunk.get_block((pos.x & 15) as u32, pos.y as u32, (pos.z & 15) as u32)
        } else {
            0 // Air
        }
    }

    fn set_block_raw(&mut self, pos: BlockPos, block: u32) -> bool {
        let chunk_key = self.get_chunk_key(pos);
        if let Some(chunk) = self.chunks.get_mut(&chunk_key) {
            chunk.set_block(
                (pos.x & 15) as u32,
                pos.y as u32,
                (pos.z & 15) as u32,
                block,
            )
        } else {
            false
        }
    }

    fn delete_block_entity(&mut self, pos: BlockPos) {
        let chunk_key = self.get_chunk_key(pos);
        if let Some(chunk) = self.chunks.get_mut(&chunk_key) {
            chunk.delete_block_entity(BlockPos::new(pos.x & 15, pos.y, pos.z & 15));
        }
    }

    fn get_block_entity(&self, pos: BlockPos) -> Option<&BlockEntity> {
        let chunk_key = self.get_chunk_key(pos);
        self.chunks
            .get(&chunk_key)?
            .get_block_entity(BlockPos::new(pos.x & 15, pos.y, pos.z & 15))
    }

    fn set_block_entity(&mut self, pos: BlockPos, block_entity: BlockEntity) {
        let chunk_key = self.get_chunk_key(pos);
        if let Some(chunk) = self.chunks.get_mut(&chunk_key) {
            chunk.set_block_entity(BlockPos::new(pos.x & 15, pos.y, pos.z & 15), block_entity);
        }
    }

    fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, z))
    }

    fn get_chunk_mut(&mut self, x: i32, z: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(x, z))
    }

    fn schedule_tick(&mut self, pos: BlockPos, delay: u32, priority: TickPriority) {
        self.to_be_ticked.push(TickEntry {
            pos,
            ticks_left: delay,
            tick_priority: priority,
        });
    }

    fn pending_tick_at(&mut self, _pos: BlockPos) -> bool {
        self.to_be_ticked.iter().any(|entry| entry.pos == _pos)
    }
}
