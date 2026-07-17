use crate::blockpedia::{color::ExtendedColorData, BlockFacts, BLOCKS};
use std::collections::HashSet;

/// Main entry point for block queries - works with BlockFacts throughout
#[derive(Debug, Clone)]
pub struct BlockQuery {
    blocks: Vec<&'static BlockFacts>,
}

/// Color sampling methods for palette generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSamplingMethod {
    /// Use the dominant color from the block texture
    Dominant,
    /// Use the average color from the block texture
    Average,
    /// Use color clustering to find representative colors
    Clustering { k: usize },
    /// Weight colors by edge detection (emphasize textures)
    EdgeWeighted,
    /// Use the most frequent color in quantized space
    MostFrequent { bins: usize },
}

/// Color space for gradient interpolation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    /// RGB color space (simple linear interpolation)
    Rgb,
    /// HSL color space (good for hue transitions)
    Hsl,
    /// Oklab color space (perceptually uniform)
    Oklab,
    /// CIE Lab color space (professional color work)
    Lab,
}

/// Easing functions for gradient generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingFunction {
    /// Linear interpolation (constant speed)
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    EaseOut,
    /// Ease in-out (slow start and end)
    EaseInOut,
    /// Cubic bezier curve
    CubicBezier { p1: (f32, f32), p2: (f32, f32) },
    /// Sine wave easing
    Sine,
    /// Exponential easing
    Exponential,
}

/// Gradient configuration
#[derive(Debug, Clone)]
pub struct GradientConfig {
    pub steps: usize,
    pub color_space: ColorSpace,
    pub sampling_method: ColorSamplingMethod,
    pub easing: EasingFunction,
}

impl Default for GradientConfig {
    fn default() -> Self {
        Self {
            steps: 10,
            color_space: ColorSpace::Oklab,
            sampling_method: ColorSamplingMethod::Dominant,
            easing: EasingFunction::Linear,
        }
    }
}

/// Main entry point - all blocks
pub struct AllBlocks;

impl AllBlocks {
    /// Start a new block query with all available blocks
    #[allow(clippy::new_ret_no_self)] // AllBlocks is just a namespace
    pub fn new() -> BlockQuery {
        BlockQuery {
            blocks: BLOCKS.values().copied().collect(),
        }
    }
}

impl BlockQuery {
    // === FILTERING METHODS (return BlockQuery) ===

    /// Only include solid blocks (exclude partial blocks, stairs, slabs, etc.)
    pub fn only_solid(mut self) -> Self {
        self.blocks.retain(|block| Self::is_solid_block(block));
        self
    }

    /// Exclude blocks that are tile entities (chests, furnaces, etc.)
    pub fn exclude_tile_entities(mut self) -> Self {
        self.blocks.retain(|block| !Self::is_tile_entity(block));
        self
    }

    /// Exclude blocks that fall due to gravity
    pub fn exclude_falling(mut self) -> Self {
        self.blocks.retain(|block| !Self::is_falling_block(block));
        self
    }

    /// Exclude transparent blocks (glass, water, etc.)
    pub fn exclude_transparent(mut self) -> Self {
        self.blocks.retain(|block| !Self::is_transparent(block));
        self
    }

    /// Exclude blocks that emit light
    pub fn exclude_light_sources(mut self) -> Self {
        self.blocks.retain(|block| !Self::is_light_source(block));
        self
    }

    /// Only include blocks that require no support
    pub fn exclude_needs_support(mut self) -> Self {
        self.blocks.retain(|block| !Self::needs_support(block));
        self
    }

    /// Only include blocks obtainable in survival mode
    pub fn survival_only(mut self) -> Self {
        self.blocks
            .retain(|block| Self::is_survival_obtainable(block));
        self
    }

    /// Only include blocks that have color data
    pub fn with_color(mut self) -> Self {
        self.blocks.retain(|block| block.extras.color.is_some());
        self
    }

    /// Filter by property existence
    pub fn with_property(mut self, property: &str) -> Self {
        let property = property.to_string();
        self.blocks.retain(|block| block.has_property(&property));
        self
    }

    /// Filter by property value
    pub fn with_property_value(mut self, property: &str, value: &str) -> Self {
        let property = property.to_string();
        let value = value.to_string();
        self.blocks.retain(|block| {
            block.get_property(&property) == Some(value.as_str())
                || block
                    .get_property_values(&property)
                    .map(|values| values.contains(&value))
                    .unwrap_or(false)
        });
        self
    }

    /// Filter by block name pattern (supports wildcards)
    pub fn matching(mut self, pattern: &str) -> Self {
        let pattern = pattern.to_lowercase();
        self.blocks.retain(|block| {
            let id = block.id().to_lowercase();
            if pattern.contains('*') {
                Self::matches_pattern(&id, &pattern)
            } else {
                id.contains(&pattern)
            }
        });
        self
    }

    /// Include only blocks from specific families
    pub fn from_families(mut self, families: &[&str]) -> Self {
        let family_set: HashSet<String> = families.iter().map(|f| f.to_lowercase()).collect();
        self.blocks.retain(|block| {
            let family = Self::get_block_family(block);
            family_set.contains(&family.to_lowercase())
        });
        self
    }

    /// Exclude blocks from specific families
    pub fn exclude_families(mut self, families: &[&str]) -> Self {
        let family_set: HashSet<String> = families.iter().map(|f| f.to_lowercase()).collect();
        self.blocks.retain(|block| {
            let family = Self::get_block_family(block);
            !family_set.contains(&family.to_lowercase())
        });
        self
    }

    /// Filter by color similarity to a target color
    pub fn similar_to_color(mut self, target_color: ExtendedColorData, tolerance: f32) -> Self {
        self.blocks.retain(|block| {
            if let Some(color) = block.extras.color {
                color.to_extended().distance_oklab(&target_color) <= tolerance
            } else {
                false
            }
        });
        self
    }

    /// Limit the number of results
    pub fn limit(mut self, count: usize) -> Self {
        self.blocks.truncate(count);
        self
    }

    /// Sort blocks by name
    pub fn sort_by_name(mut self) -> Self {
        self.blocks.sort_by(|a, b| a.id().cmp(b.id()));
        self
    }

    /// Sort blocks by color similarity to a reference color
    pub fn sort_by_color_similarity(mut self, reference: ExtendedColorData) -> Self {
        self.blocks.sort_by(|a, b| {
            let dist_a = a
                .extras
                .color
                .map(|c| c.to_extended().distance_oklab(&reference))
                .unwrap_or(f32::INFINITY);
            let dist_b = b
                .extras
                .color
                .map(|c| c.to_extended().distance_oklab(&reference))
                .unwrap_or(f32::INFINITY);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self
    }

    // === TERMINAL METHODS (return Vec<BlockFacts> or other types) ===

    /// Get the blocks as a vector
    pub fn collect(self) -> Vec<&'static BlockFacts> {
        self.blocks
    }

    /// Get the count of matching blocks (consumes the query)
    pub fn count(self) -> usize {
        self.blocks.len()
    }

    /// Get the length of matching blocks (non-consuming)
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Check if the query is empty (non-consuming)
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Get the first block (if any)
    pub fn first(self) -> Option<&'static BlockFacts> {
        self.blocks.into_iter().next()
    }

    /// Check if any blocks match
    pub fn any(self) -> bool {
        !self.blocks.is_empty()
    }

    /// Generate a gradient between blocks (returns blocks that match the gradient colors)
    pub fn generate_gradient(self, config: GradientConfig) -> Self {
        // Need at least 2 blocks with colors to generate a gradient
        let colored_blocks: Vec<_> = self
            .blocks
            .iter()
            .filter(|block| block.extras.color.is_some())
            .copied()
            .collect();

        if colored_blocks.len() < 2 {
            return BlockQuery {
                blocks: colored_blocks,
            };
        }

        let start_color = colored_blocks
            .first()
            .unwrap()
            .extras
            .color
            .unwrap()
            .to_extended();
        let end_color = colored_blocks
            .last()
            .unwrap()
            .extras
            .color
            .unwrap()
            .to_extended();

        Self::generate_gradient_between_colors_static(start_color, end_color, config)
    }

    /// Generate a gradient between two specific blocks
    pub fn generate_gradient_between_blocks(
        start_block_id: &str,
        end_block_id: &str,
        config: GradientConfig,
    ) -> Self {
        let start_block = BLOCKS.get(start_block_id);
        let end_block = BLOCKS.get(end_block_id);

        match (start_block, end_block) {
            (Some(start), Some(end)) => {
                if let (Some(start_color), Some(end_color)) = (start.extras.color, end.extras.color)
                {
                    Self::generate_gradient_between_colors_static(
                        start_color.to_extended(),
                        end_color.to_extended(),
                        config,
                    )
                } else {
                    BlockQuery { blocks: Vec::new() }
                }
            }
            _ => BlockQuery { blocks: Vec::new() },
        }
    }

    /// Generate a gradient between two specific colors (returns blocks that best match)
    pub fn generate_gradient_between_colors(
        self,
        start_color: ExtendedColorData,
        end_color: ExtendedColorData,
        config: GradientConfig,
    ) -> Self {
        Self::generate_gradient_between_colors_static(start_color, end_color, config)
    }

    /// Generate a multi-color gradient through all available block colors
    pub fn generate_multi_gradient(self, config: GradientConfig) -> Self {
        let colored_blocks: Vec<_> = self
            .blocks
            .iter()
            .filter(|block| block.extras.color.is_some())
            .copied()
            .collect();

        if colored_blocks.is_empty() {
            return BlockQuery { blocks: Vec::new() };
        }

        if colored_blocks.len() == 1 {
            return BlockQuery {
                blocks: vec![colored_blocks[0]; config.steps.min(1)],
            };
        }

        let colors: Vec<ExtendedColorData> = colored_blocks
            .iter()
            .map(|block| block.extras.color.unwrap().to_extended())
            .collect();

        // Create a dummy instance to call the method
        let dummy = BlockQuery { blocks: vec![] };
        let gradient_colors = dummy.create_multi_gradient_colors(colors, config);

        // Find blocks that best match each gradient color
        let mut gradient_blocks = Vec::new();
        for target_color in gradient_colors {
            // Use static method to find closest block
            let mut best_block = None;
            let mut best_distance = f32::INFINITY;

            for block in BLOCKS.values() {
                if let Some(block_color) = block.extras.color {
                    let distance = block_color.to_extended().distance_oklab(&target_color);
                    if distance < best_distance {
                        best_distance = distance;
                        best_block = Some(*block);
                    }
                }
            }

            if let Some(block) = best_block {
                gradient_blocks.push(block);
            }
        }

        BlockQuery {
            blocks: gradient_blocks,
        }
    }

    /// Sort blocks to create a smooth color transition
    pub fn sort_by_color_gradient(self) -> Self {
        if self.blocks.len() <= 1 {
            return self;
        }

        // Only consider blocks with colors
        let mut colored_blocks: Vec<_> = self
            .blocks
            .into_iter()
            .filter(|block| block.extras.color.is_some())
            .collect();

        if colored_blocks.len() <= 1 {
            return BlockQuery {
                blocks: colored_blocks,
            };
        }

        // Use traveling salesman-like approach to create smooth color transitions
        let mut result = Vec::new();
        result.push(colored_blocks.remove(0)); // Start with first block

        while !colored_blocks.is_empty() {
            let current_color = result.last().unwrap().extras.color.unwrap().to_extended();

            // Find the closest remaining color
            let mut best_index = 0;
            let mut best_distance = f32::INFINITY;

            for (i, block) in colored_blocks.iter().enumerate() {
                let distance = block
                    .extras
                    .color
                    .unwrap()
                    .to_extended()
                    .distance_oklab(&current_color);
                if distance < best_distance {
                    best_distance = distance;
                    best_index = i;
                }
            }

            result.push(colored_blocks.remove(best_index));
        }

        BlockQuery { blocks: result }
    }

    // === HELPER METHODS ===

    fn is_solid_block(block: &BlockFacts) -> bool {
        // Official model geometry (cube-family template / full 16^3 element)
        // instead of the old name-substring guess.
        block.is_full_cube()
    }

    fn is_tile_entity(block: &BlockFacts) -> bool {
        let id = block.id().to_lowercase();
        matches!(id.as_str(),
            id if id.contains("chest") ||
                  id.contains("furnace") ||
                  id.contains("dispenser") ||
                  id.contains("dropper") ||
                  id.contains("hopper") ||
                  id.contains("beacon") ||
                  id.contains("brewing_stand") ||
                  id.contains("enchanting_table") ||
                  id.contains("ender_chest") ||
                  id.contains("shulker_box") ||
                  id.contains("barrel") ||
                  id.contains("smoker") ||
                  id.contains("blast_furnace") ||
                  id.contains("campfire") ||
                  id.contains("lectern") ||
                  id.contains("jukebox")
        )
    }

    fn is_falling_block(block: &BlockFacts) -> bool {
        let id = block.id().to_lowercase();
        matches!(id.as_str(),
            id if id.contains("sand") ||
                  id.contains("gravel") ||
                  id.contains("anvil") ||
                  id.contains("concrete_powder")
        )
    }

    fn is_transparent(block: &BlockFacts) -> bool {
        // Data field from the block report pipeline instead of the old
        // name-substring guess.
        block.transparent
    }

    fn is_light_source(block: &BlockFacts) -> bool {
        let id = block.id().to_lowercase();
        matches!(id.as_str(),
            id if id.contains("torch") ||
                  id.contains("lantern") ||
                  id.contains("glowstone") ||
                  id.contains("sea_lantern") ||
                  id.contains("beacon") ||
                  id.contains("campfire") ||
                  id.contains("fire") ||
                  id.contains("lava") ||
                  id.contains("magma_block") ||
                  id.contains("jack_o_lantern") ||
                  id.contains("redstone_lamp") ||
                  id.contains("shroomlight") ||
                  id.contains("crying_obsidian") ||
                  id.contains("respawn_anchor") ||
                  id.contains("candle") ||
                  id.contains("glow_lichen") ||
                  id.contains("amethyst_cluster")
        )
    }

    fn needs_support(block: &BlockFacts) -> bool {
        let id = block.id().to_lowercase();
        matches!(id.as_str(),
            id if id.contains("torch") ||
                  id.contains("flower") ||
                  (id.contains("grass") && !id.contains("grass_block")) ||
                  id.contains("fern") ||
                  id.contains("sapling") ||
                  (id.contains("mushroom") && !id.contains("mushroom_block")) ||
                  id.contains("wheat") ||
                  id.contains("carrot") ||
                  id.contains("potato") ||
                  id.contains("beetroot") ||
                  id.contains("sugar_cane") ||
                  id.contains("cactus") ||
                  id.contains("bamboo") ||
                  id.contains("vine") ||
                  id.contains("lily_pad") ||
                  id.contains("seagrass") ||
                  id.contains("kelp") ||
                  (id.contains("coral") && !id.contains("coral_block")) ||
                  id.contains("button") ||
                  id.contains("lever") ||
                  id.contains("sign") ||
                  id.contains("banner") ||
                  id.contains("painting")
        )
    }

    fn is_survival_obtainable(block: &BlockFacts) -> bool {
        let id = block.id().to_lowercase();
        !matches!(id.as_str(),
            id if id.contains("barrier") ||
                  id.contains("structure_void") ||
                  id.contains("structure_block") ||
                  id.contains("command_block") ||
                  id.contains("chain_command_block") ||
                  id.contains("repeating_command_block") ||
                  id.contains("jigsaw") ||
                  id.contains("debug_stick") ||
                  id.contains("knowledge_book") ||
                  id.contains("spawn_egg")
        )
    }

    fn matches_pattern(text: &str, pattern: &str) -> bool {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.is_empty() {
            return true;
        }

        let mut search_pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            if i == 0 {
                // First part - must be at the beginning
                if !text.starts_with(part) {
                    return false;
                }
                search_pos = part.len();
            } else if i == parts.len() - 1 {
                // Last part - must be at the end
                if !text.ends_with(part) {
                    return false;
                }
            } else {
                // Middle part - must exist after current position
                if let Some(pos) = text[search_pos..].find(part) {
                    search_pos += pos + part.len();
                } else {
                    return false;
                }
            }
        }
        true
    }

    fn get_block_family(block: &BlockFacts) -> String {
        let id = block.id();
        if let Some(colon_pos) = id.find(':') {
            let name_part = &id[colon_pos + 1..];

            // Enhanced family detection
            if name_part.ends_with("_stairs") {
                return "stairs".to_string();
            }
            if name_part.ends_with("_slab") {
                return "slab".to_string();
            }
            if name_part.ends_with("_wool") {
                return "wool".to_string();
            }
            if name_part.ends_with("_concrete") {
                return "concrete".to_string();
            }
            if name_part.ends_with("_log") {
                return "log".to_string();
            }
            if name_part.ends_with("_planks") {
                return "planks".to_string();
            }
            if name_part.ends_with("_leaves") {
                return "leaves".to_string();
            }
            if name_part.contains("stone") && !name_part.contains("redstone") {
                return "stone".to_string();
            }
            if name_part.contains("glass") {
                return "glass".to_string();
            }

            name_part.to_string()
        } else {
            id.to_string()
        }
    }

    #[allow(dead_code)] // Helper method for future use
    fn format_block_name(id: &str) -> String {
        id.strip_prefix("minecraft:")
            .unwrap_or(id)
            .replace('_', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Helper methods for gradient generation
    fn create_gradient_colors(
        &self,
        start_color: ExtendedColorData,
        end_color: ExtendedColorData,
        config: GradientConfig,
    ) -> Vec<ExtendedColorData> {
        let mut colors = Vec::with_capacity(config.steps);

        for i in 0..config.steps {
            let t = if config.steps == 1 {
                0.0
            } else {
                i as f32 / (config.steps - 1) as f32
            };
            let eased_t = Self::apply_easing(t, config.easing);
            let color =
                Self::interpolate_color(start_color, end_color, eased_t, config.color_space);
            colors.push(color);
        }

        colors
    }

    fn create_multi_gradient_colors(
        &self,
        colors: Vec<ExtendedColorData>,
        config: GradientConfig,
    ) -> Vec<ExtendedColorData> {
        if colors.is_empty() {
            return Vec::new();
        }

        if colors.len() == 1 {
            return vec![colors[0]; config.steps];
        }

        let segment_steps = config.steps / (colors.len() - 1);
        let mut result = Vec::new();

        for i in 0..colors.len() - 1 {
            let segment_gradient = self.create_gradient_colors(
                colors[i],
                colors[i + 1],
                GradientConfig {
                    steps: segment_steps,
                    ..config
                },
            );

            // Avoid duplicating colors at segment boundaries
            if i > 0 {
                result.extend(segment_gradient.into_iter().skip(1));
            } else {
                result.extend(segment_gradient);
            }
        }

        result
    }

    #[allow(dead_code)] // Helper method for future use
    fn find_closest_color_block(
        &self,
        target_color: &ExtendedColorData,
    ) -> Option<&'static BlockFacts> {
        let mut best_block = None;
        let mut best_distance = f32::INFINITY;

        for block in BLOCKS.values() {
            if let Some(block_color) = block.extras.color {
                let distance = block_color.to_extended().distance_oklab(target_color);
                if distance < best_distance {
                    best_distance = distance;
                    best_block = Some(*block);
                }
            }
        }

        best_block
    }

    fn apply_easing(t: f32, easing: EasingFunction) -> f32 {
        match easing {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
            EasingFunction::CubicBezier { .. } => {
                // Simplified cubic bezier - for full implementation would need more complex math
                t * t * (3.0 - 2.0 * t) // smoothstep approximation
            }
            EasingFunction::Sine => (t * std::f32::consts::PI / 2.0).sin(),
            EasingFunction::Exponential => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * (t - 1.0))
                }
            }
        }
    }

    fn interpolate_color(
        start: ExtendedColorData,
        end: ExtendedColorData,
        t: f32,
        color_space: ColorSpace,
    ) -> ExtendedColorData {
        match color_space {
            ColorSpace::Rgb => Self::interpolate_rgb(start, end, t),
            ColorSpace::Hsl => Self::interpolate_hsl(start, end, t),
            ColorSpace::Oklab => Self::interpolate_oklab(start, end, t),
            ColorSpace::Lab => Self::interpolate_lab(start, end, t),
        }
    }

    fn interpolate_rgb(
        start: ExtendedColorData,
        end: ExtendedColorData,
        t: f32,
    ) -> ExtendedColorData {
        let r = (start.rgb[0] as f32 * (1.0 - t) + end.rgb[0] as f32 * t) as u8;
        let g = (start.rgb[1] as f32 * (1.0 - t) + end.rgb[1] as f32 * t) as u8;
        let b = (start.rgb[2] as f32 * (1.0 - t) + end.rgb[2] as f32 * t) as u8;
        ExtendedColorData::from_rgb(r, g, b)
    }

    fn interpolate_hsl(
        start: ExtendedColorData,
        end: ExtendedColorData,
        t: f32,
    ) -> ExtendedColorData {
        let h = Self::interpolate_hue(start.hsl[0], end.hsl[0], t);
        let s = start.hsl[1] * (1.0 - t) + end.hsl[1] * t;
        let l = start.hsl[2] * (1.0 - t) + end.hsl[2] * t;

        // Convert back to RGB (simplified)
        let rgb = Self::hsl_to_rgb(h, s, l);
        ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2])
    }

    fn interpolate_oklab(
        start: ExtendedColorData,
        end: ExtendedColorData,
        t: f32,
    ) -> ExtendedColorData {
        let l = start.oklab[0] * (1.0 - t) + end.oklab[0] * t;
        let a = start.oklab[1] * (1.0 - t) + end.oklab[1] * t;
        let b = start.oklab[2] * (1.0 - t) + end.oklab[2] * t;

        // Convert back to RGB (simplified)
        let rgb = Self::oklab_to_rgb([l, a, b]);
        ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2])
    }

    fn interpolate_lab(
        start: ExtendedColorData,
        end: ExtendedColorData,
        t: f32,
    ) -> ExtendedColorData {
        let l = start.lab[0] * (1.0 - t) + end.lab[0] * t;
        let a = start.lab[1] * (1.0 - t) + end.lab[1] * t;
        let b = start.lab[2] * (1.0 - t) + end.lab[2] * t;

        // Convert back to RGB using proper Lab conversion
        let rgb = Self::lab_to_rgb([l, a, b]);
        ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2])
    }

    fn interpolate_hue(start_hue: f32, end_hue: f32, t: f32) -> f32 {
        let mut diff = end_hue - start_hue;

        // Take the shortest path around the color wheel
        if diff > 180.0 {
            diff -= 360.0;
        } else if diff < -180.0 {
            diff += 360.0;
        }

        let result = start_hue + diff * t;
        if result < 0.0 {
            result + 360.0
        } else if result >= 360.0 {
            result - 360.0
        } else {
            result
        }
    }

    // Simplified color space conversions (in production, use a proper color library)
    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
        let h = h / 360.0;
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = if h < 1.0 / 6.0 {
            (c, x, 0.0)
        } else if h < 2.0 / 6.0 {
            (x, c, 0.0)
        } else if h < 3.0 / 6.0 {
            (0.0, c, x)
        } else if h < 4.0 / 6.0 {
            (0.0, x, c)
        } else if h < 5.0 / 6.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        [
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        ]
    }

    #[allow(clippy::manual_clamp, clippy::excessive_precision)] // Scientific precision required
    fn oklab_to_rgb(oklab: [f32; 3]) -> [u8; 3] {
        // Simplified Oklab to RGB conversion
        let l = oklab[0].clamp(0.0, 1.0);
        let a = oklab[1];
        let b = oklab[2];

        // Approximate conversion
        let r = (l + 0.3963377774 * a + 0.2158037573 * b).max(0.0).min(1.0);
        let g = (l - 0.1055613458 * a - 0.0638541728 * b).max(0.0).min(1.0);
        let b_val = (l - 0.0894841775 * a - 1.2914855480 * b).max(0.0).min(1.0);

        [(r * 255.0) as u8, (g * 255.0) as u8, (b_val * 255.0) as u8]
    }

    fn lab_to_rgb(lab: [f32; 3]) -> [u8; 3] {
        // Proper Lab to RGB conversion via XYZ color space
        let l = lab[0];
        let a = lab[1];
        let b = lab[2];

        // Lab to XYZ conversion
        let fy = (l + 16.0) / 116.0;
        let fx = a / 500.0 + fy;
        let fz = fy - b / 200.0;

        let delta = 6.0 / 29.0;
        let delta_cubed = delta * delta * delta;

        let x = if fx.powi(3) > delta_cubed {
            fx.powi(3)
        } else {
            3.0 * delta * delta * (fx - 4.0 / 29.0)
        };

        let y = if l > 8.0 {
            ((l + 16.0) / 116.0).powi(3)
        } else {
            l / 903.3
        };

        let z = if fz.powi(3) > delta_cubed {
            fz.powi(3)
        } else {
            3.0 * delta * delta * (fz - 4.0 / 29.0)
        };

        // D65 illuminant normalization
        let x = x * 95.047;
        let y = y * 100.0;
        let z = z * 108.883;

        // XYZ to RGB conversion (sRGB)
        let mut r = x * 0.032406 + y * -0.015372 + z * -0.004986;
        let mut g = x * -0.009689 + y * 0.018758 + z * 0.000415;
        let mut b_val = x * 0.000557 + y * -0.002040 + z * 0.010570;

        // Gamma correction for sRGB
        let gamma_correct = |c: f32| -> f32 {
            if c > 0.0031308 {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            } else {
                12.92 * c
            }
        };

        r = gamma_correct(r / 100.0);
        g = gamma_correct(g / 100.0);
        b_val = gamma_correct(b_val / 100.0);

        // Clamp to [0, 1] and convert to [0, 255]
        let r = (r.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (g.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (b_val.clamp(0.0, 1.0) * 255.0) as u8;

        [r, g, b]
    }

    /// Static method for gradient generation (used internally)
    fn generate_gradient_between_colors_static(
        start_color: ExtendedColorData,
        end_color: ExtendedColorData,
        config: GradientConfig,
    ) -> Self {
        // Generate gradient colors, including the exact start and end points
        // This creates a complete gradient from start to end
        let mut colors = Vec::with_capacity(config.steps);

        if config.steps == 1 {
            // If only one step, use the start color
            colors.push(start_color);
        } else if config.steps == 2 {
            // If two steps, use start and end colors
            colors.push(start_color);
            colors.push(end_color);
        } else {
            // Generate colors including start and end points
            for i in 0..config.steps {
                // Map i from [0, steps-1] to [0, 1] inclusive of endpoints
                let t = if config.steps == 1 {
                    0.0
                } else {
                    i as f32 / (config.steps - 1) as f32
                };
                let eased_t = Self::apply_easing(t, config.easing);
                let color =
                    Self::interpolate_color(start_color, end_color, eased_t, config.color_space);
                colors.push(color);
            }
        }

        // Find blocks that best match each gradient color, avoiding duplicates
        let mut gradient_blocks = Vec::new();
        let mut used_blocks = HashSet::new();

        for target_color in colors {
            let mut candidates: Vec<(&BlockFacts, f32)> = BLOCKS
                .values()
                .filter_map(|block| {
                    if let Some(block_color) = block.extras.color {
                        if !used_blocks.contains(block.id()) {
                            let distance = block_color.to_extended().distance_oklab(&target_color);
                            Some((*block, distance))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by distance and pick the best unused block
            candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((best_block, _)) = candidates.first() {
                gradient_blocks.push(*best_block);
                used_blocks.insert(best_block.id().to_string());
            }
        }

        BlockQuery {
            blocks: gradient_blocks,
        }
    }
}

// === CONVENIENCE CONSTRUCTORS ===

impl GradientConfig {
    pub fn new(steps: usize) -> Self {
        Self {
            steps,
            ..Default::default()
        }
    }

    pub fn with_color_space(mut self, color_space: ColorSpace) -> Self {
        self.color_space = color_space;
        self
    }

    pub fn with_sampling(mut self, sampling: ColorSamplingMethod) -> Self {
        self.sampling_method = sampling;
        self
    }

    pub fn with_easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }
}
