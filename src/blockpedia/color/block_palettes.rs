use super::{palettes::GradientMethod, ExtendedColorData};
use crate::blockpedia::BlockFacts;
use crate::blockpedia::BLOCKS;
/// Generate palettes of actual Minecraft blocks based on color relationships
pub struct BlockPaletteGenerator;

/// A palette of Minecraft blocks organized by color relationships
#[derive(Debug, Clone)]
pub struct BlockPalette {
    pub name: String,
    pub description: String,
    pub blocks: Vec<BlockRecommendation>,
    pub theme: PaletteTheme,
}

/// A recommended block with usage context
#[derive(Debug, Clone)]
pub struct BlockRecommendation {
    pub block: &'static BlockFacts,
    pub color: ExtendedColorData,
    pub role: BlockRole,
    pub usage_notes: String,
}

/// Different types of palettes for building
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteTheme {
    Monochrome,
    Gradient,
    Complementary,
    Analogous,
    Triadic,
    Natural,
    Architectural,
    Seasonal,
}

/// Role of a block in the palette
#[derive(Debug, Clone, PartialEq)]
pub enum BlockRole {
    Primary,    // Main building material
    Secondary,  // Supporting elements
    Accent,     // Detail and contrast
    Transition, // Smooth color flow
    Highlight,  // Eye-catching features
}

/// Filter configuration for block selection
#[derive(Debug, Clone, Default)]
pub struct BlockFilter {
    /// Exclude blocks that fall due to gravity (sand, gravel, etc.)
    pub exclude_falling: bool,
    /// Exclude blocks that have tile entities (chests, furnaces, etc.)
    pub exclude_tile_entities: bool,
    /// Only include full blocks (exclude slabs, stairs, etc.)
    pub full_blocks_only: bool,
    /// Exclude blocks that require support (torches, flowers, etc.)
    pub exclude_needs_support: bool,
    /// Exclude transparent blocks (glass, water, etc.)
    pub exclude_transparent: bool,
    /// Exclude blocks that emit light
    pub exclude_light_sources: bool,
    /// Only include blocks that can be obtained in survival
    pub survival_obtainable_only: bool,
    /// Custom block ID patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Custom block ID patterns to include (overrides excludes)
    pub include_patterns: Vec<String>,
    /// Vanilla block tags a block must ALL carry (`minecraft:wool` or the
    /// short `wool` form; empty = no requirement)
    pub required_tags: Vec<String>,
    /// Vanilla block tags that disqualify a block (any match excludes)
    pub excluded_tags: Vec<String>,
    /// Official block kinds to allow (`minecraft:stair`, short `stair`, ...;
    /// empty = all kinds allowed)
    pub kinds: Vec<String>,
}

/// Normalize a user-supplied tag/kind name to the stored
/// `minecraft:`-prefixed form.
fn normalize_mc_name(name: &str) -> String {
    if name.contains(':') {
        name.to_string()
    } else {
        format!("minecraft:{name}")
    }
}

impl BlockFilter {
    /// Create a filter for solid building blocks only
    pub fn solid_blocks_only() -> Self {
        BlockFilter {
            exclude_falling: true,
            exclude_tile_entities: true,
            full_blocks_only: true,
            exclude_needs_support: true,
            exclude_transparent: true,
            exclude_light_sources: false,
            survival_obtainable_only: true,
            exclude_patterns: vec![
                "_slab".to_string(),
                "_stairs".to_string(),
                "_fence".to_string(),
                "_gate".to_string(),
                "_wall".to_string(),
                "_button".to_string(),
                "_pressure_plate".to_string(),
                "_door".to_string(),
                "_trapdoor".to_string(),
            ],
            ..Default::default()
        }
    }

    /// Create a filter for decorative blocks (allows more variety)
    pub fn decorative_blocks() -> Self {
        BlockFilter {
            exclude_falling: true,
            exclude_tile_entities: true,
            full_blocks_only: false,
            exclude_needs_support: false,
            exclude_transparent: false,
            exclude_light_sources: false,
            survival_obtainable_only: true,
            ..Default::default()
        }
    }

    /// Create a filter for structural blocks (very conservative)
    pub fn structural_blocks_only() -> Self {
        BlockFilter {
            exclude_falling: true,
            exclude_tile_entities: true,
            full_blocks_only: true,
            exclude_needs_support: true,
            exclude_transparent: true,
            exclude_light_sources: true,
            survival_obtainable_only: true,
            exclude_patterns: vec![
                "_slab".to_string(),
                "_stairs".to_string(),
                "_fence".to_string(),
                "_gate".to_string(),
                "_wall".to_string(),
                "_button".to_string(),
                "_pressure_plate".to_string(),
                "_door".to_string(),
                "_trapdoor".to_string(),
                "glass".to_string(),
                "water".to_string(),
                "lava".to_string(),
                "air".to_string(),
            ],
            ..Default::default()
        }
    }

    /// Check if a block passes this filter
    pub fn allows_block(&self, block: &BlockFacts) -> bool {
        let id = block.id().to_lowercase();

        // Check include patterns first (overrides excludes)
        if !self.include_patterns.is_empty() {
            let included = self
                .include_patterns
                .iter()
                .any(|pattern| id.contains(&pattern.to_lowercase()));
            if !included {
                return false;
            }
        }

        // Check exclude patterns
        if self
            .exclude_patterns
            .iter()
            .any(|pattern| id.contains(&pattern.to_lowercase()))
        {
            return false;
        }

        // Check falling blocks
        if self.exclude_falling && Self::is_falling_block(&id) {
            return false;
        }

        // Check tile entities (official block_entity_type registry data)
        if self.exclude_tile_entities && block.has_block_entity {
            return false;
        }

        // Check full blocks only (official model geometry, not name guessing)
        if self.full_blocks_only && !block.is_full_cube() {
            return false;
        }

        // Check needs support
        if self.exclude_needs_support && Self::needs_support(&id) {
            return false;
        }

        // Check transparency (data field from the block report pipeline)
        if self.exclude_transparent && block.transparent {
            return false;
        }

        // Check light sources (default-state `emit_light` data field)
        if self.exclude_light_sources && block.is_light_source() {
            return false;
        }

        // Check survival obtainable
        if self.survival_obtainable_only && !Self::is_survival_obtainable(&id) {
            return false;
        }

        // Check required/excluded vanilla tags
        if !self.required_tags.is_empty() && !self.required_tags.iter().all(|t| block.has_tag(t)) {
            return false;
        }
        if self.excluded_tags.iter().any(|t| block.has_tag(t)) {
            return false;
        }

        // Check official block kinds (empty = all allowed)
        if !self.kinds.is_empty()
            && !self
                .kinds
                .iter()
                .any(|k| block.kind() == normalize_mc_name(k))
        {
            return false;
        }

        true
    }

    fn is_falling_block(id: &str) -> bool {
        matches!(id,
            id if id.contains("sand") ||
                  id.contains("gravel") ||
                  id.contains("anvil") ||
                  id.contains("concrete_powder")
        )
    }

    fn needs_support(id: &str) -> bool {
        matches!(id,
            id if id.contains("torch") ||
                  id.contains("flower") ||
                  id.contains("grass") && !id.contains("grass_block") ||
                  id.contains("fern") ||
                  id.contains("sapling") ||
                  id.contains("mushroom") && !id.contains("mushroom_block") ||
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
                  id.contains("coral") && !id.contains("coral_block") ||
                  id.contains("button") ||
                  id.contains("lever") ||
                  id.contains("sign") ||
                  id.contains("banner") ||
                  id.contains("painting")
        )
    }

    fn is_survival_obtainable(id: &str) -> bool {
        // Exclude creative-only blocks
        !matches!(id,
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
}

#[allow(dead_code, clippy::needless_borrow, clippy::explicit_auto_deref)] // API for future use
impl BlockPaletteGenerator {
    /// Generate a gradient palette of blocks between two blocks with filtering
    pub fn generate_block_gradient(
        start_block: &'static BlockFacts,
        end_block: &'static BlockFacts,
        steps: usize,
    ) -> Option<BlockPalette> {
        Self::generate_block_gradient_filtered(
            start_block,
            end_block,
            steps,
            &BlockFilter::default(),
        )
    }

    /// Generate a gradient palette with custom filtering
    pub fn generate_block_gradient_filtered(
        start_block: &'static BlockFacts,
        end_block: &'static BlockFacts,
        steps: usize,
        filter: &BlockFilter,
    ) -> Option<BlockPalette> {
        let start_color = start_block.extras.color?.to_extended();
        let end_color = end_block.extras.color?.to_extended();

        // Generate color gradient
        let color_gradient = super::palettes::PaletteGenerator::generate_gradient_palette(
            start_color,
            end_color,
            steps,
            GradientMethod::LinearOklab,
        );

        // Find blocks that match each color in the gradient, honoring the filter
        let mut blocks = Vec::new();
        for (i, target_color) in color_gradient.iter().enumerate() {
            if let Some(block) = Self::find_closest_block_to_color_filtered(*target_color, filter) {
                let role = match i {
                    0 => BlockRole::Primary,
                    i if i == steps - 1 => BlockRole::Accent,
                    i if i == steps / 2 => BlockRole::Secondary,
                    _ => BlockRole::Transition,
                };

                let usage_notes = Self::generate_usage_notes(&block, &role);

                blocks.push(BlockRecommendation {
                    block,
                    color: block.extras.color?.to_extended(),
                    role,
                    usage_notes,
                });
            }
        }

        Some(BlockPalette {
            name: format!(
                "{} to {} Gradient",
                Self::block_display_name(start_block),
                Self::block_display_name(end_block)
            ),
            description: format!(
                "A smooth gradient from {} to {} using {} blocks for natural color flow",
                Self::block_display_name(start_block),
                Self::block_display_name(end_block),
                blocks.len()
            ),
            blocks,
            theme: PaletteTheme::Gradient,
        })
    }

    /// Generate a monochrome palette around a base block
    pub fn generate_monochrome_palette(
        base_block: &'static BlockFacts,
        range: usize,
    ) -> Option<BlockPalette> {
        let base_color = base_block.extras.color?.to_extended();

        // Generate monochrome color variations
        let mono_colors =
            super::palettes::PaletteGenerator::generate_monochrome_palette(base_color, range);

        let mut blocks = Vec::new();
        for (i, target_color) in mono_colors.iter().enumerate() {
            if let Some(block) = Self::find_closest_block_to_color(*target_color) {
                let role = match i {
                    0 => BlockRole::Accent,                      // Darkest
                    i if i == range / 2 => BlockRole::Primary,   // Base
                    i if i == range - 1 => BlockRole::Highlight, // Lightest
                    _ => BlockRole::Secondary,
                };

                let usage_notes = Self::generate_usage_notes(&block, &role);

                blocks.push(BlockRecommendation {
                    block,
                    color: block.extras.color?.to_extended(),
                    role,
                    usage_notes,
                });
            }
        }

        Some(BlockPalette {
            name: format!("{} Monochrome", Self::block_display_name(base_block)),
            description: format!(
                "A monochrome palette based on {} with {} tonal variations from dark to light",
                Self::block_display_name(base_block),
                blocks.len()
            ),
            blocks,
            theme: PaletteTheme::Monochrome,
        })
    }

    /// Generate a complementary palette
    pub fn generate_complementary_palette(base_block: &'static BlockFacts) -> Option<BlockPalette> {
        let base_color = base_block.extras.color?.to_extended();
        let comp_colors =
            super::palettes::PaletteGenerator::generate_complementary_palette(&base_color);

        let mut blocks = Vec::new();
        for (i, target_color) in comp_colors.iter().enumerate() {
            if let Some(block) = Self::find_closest_block_to_color(*target_color) {
                let role = match i {
                    0 => BlockRole::Primary,
                    1 => BlockRole::Accent,
                    _ => BlockRole::Secondary,
                };

                let usage_notes = Self::generate_usage_notes(&block, &role);

                blocks.push(BlockRecommendation {
                    block,
                    color: block.extras.color?.to_extended(),
                    role,
                    usage_notes,
                });
            }
        }

        Some(BlockPalette {
            name: format!("{} Complementary", Self::block_display_name(base_block)),
            description: format!(
                "A complementary color scheme based on {} with high contrast blocks",
                Self::block_display_name(base_block)
            ),
            blocks,
            theme: PaletteTheme::Complementary,
        })
    }

    /// Generate natural palettes based on Minecraft biomes/themes
    pub fn generate_natural_palette(theme: &str) -> Option<BlockPalette> {
        Self::generate_natural_palette_filtered(theme, &BlockFilter::default())
    }

    /// Generate natural palettes with custom filtering
    pub fn generate_natural_palette_filtered(
        theme: &str,
        filter: &BlockFilter,
    ) -> Option<BlockPalette> {
        match theme.to_lowercase().as_str() {
            "forest" | "woods" => Self::generate_forest_palette_filtered(filter),
            "desert" | "sand" => Self::generate_desert_palette_filtered(filter),
            "ocean" | "water" => Self::generate_ocean_palette_filtered(filter),
            "mountain" | "stone" => Self::generate_mountain_palette_filtered(filter),
            "nether" => Self::generate_nether_palette_filtered(filter),
            "end" => Self::generate_end_palette_filtered(filter),
            _ => None,
        }
    }

    /// Generate an architectural palette for building styles
    pub fn generate_architectural_palette(style: &str) -> Option<BlockPalette> {
        Self::generate_architectural_palette_filtered(style, &BlockFilter::default())
    }

    /// Generate architectural palettes with custom filtering
    pub fn generate_architectural_palette_filtered(
        style: &str,
        filter: &BlockFilter,
    ) -> Option<BlockPalette> {
        match style.to_lowercase().as_str() {
            "medieval" => Self::generate_medieval_palette_filtered(filter),
            "modern" => Self::generate_modern_palette_filtered(filter),
            "rustic" => Self::generate_rustic_palette_filtered(filter),
            "industrial" => Self::generate_industrial_palette_filtered(filter),
            _ => None,
        }
    }

    /// Find the closest block to a target color
    fn find_closest_block_to_color(target_color: ExtendedColorData) -> Option<&'static BlockFacts> {
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

        best_block
    }

    /// Find the closest block to a target color among blocks passing `filter`
    fn find_closest_block_to_color_filtered(
        target_color: ExtendedColorData,
        filter: &BlockFilter,
    ) -> Option<&'static BlockFacts> {
        let mut best_block = None;
        let mut best_distance = f32::INFINITY;

        for block in BLOCKS.values() {
            if !filter.allows_block(block) {
                continue;
            }
            if let Some(block_color) = block.extras.color {
                let distance = block_color.to_extended().distance_oklab(&target_color);
                if distance < best_distance {
                    best_distance = distance;
                    best_block = Some(*block);
                }
            }
        }

        best_block
    }

    /// Generate usage notes for a block in a specific role
    fn generate_usage_notes(block: &BlockFacts, role: &BlockRole) -> String {
        let block_type = Self::categorize_block(block);

        match (role, block_type.as_str()) {
            (BlockRole::Primary, "stone") => {
                "Excellent for foundations, walls, and main structures".to_string()
            }
            (BlockRole::Primary, "wood") => {
                "Great for frames, floors, and warm architectural elements".to_string()
            }
            (BlockRole::Primary, "concrete") => {
                "Perfect for modern builds and large surfaces".to_string()
            }

            (BlockRole::Secondary, "stone") => {
                "Use for detailing, trim, and structural accents".to_string()
            }
            (BlockRole::Secondary, "wood") => {
                "Ideal for stairs, slabs, and secondary features".to_string()
            }
            (BlockRole::Secondary, _) => {
                "Good for supporting elements and medium-scale features".to_string()
            }

            (BlockRole::Accent, _) => {
                "Use sparingly for highlights, borders, and eye-catching details".to_string()
            }
            (BlockRole::Transition, _) => {
                "Perfect for gradual color changes and smooth blending".to_string()
            }
            (BlockRole::Highlight, _) => {
                "Excellent for focal points, lighting accents, and key features".to_string()
            }

            _ => "Versatile block suitable for various building applications".to_string(),
        }
    }

    /// Categorize a block by its material type
    fn categorize_block(block: &BlockFacts) -> String {
        let id = block.id().to_lowercase();

        if id.contains("stone") || id.contains("cobblestone") || id.contains("brick") {
            "stone".to_string()
        } else if id.contains("wood") || id.contains("plank") || id.contains("log") {
            "wood".to_string()
        } else if id.contains("concrete") || id.contains("terracotta") {
            "concrete".to_string()
        } else if id.contains("wool") || id.contains("carpet") {
            "fabric".to_string()
        } else if id.contains("glass") {
            "glass".to_string()
        } else if id.contains("metal") || id.contains("iron") || id.contains("gold") {
            "metal".to_string()
        } else {
            "other".to_string()
        }
    }

    /// Get a friendly display name for a block
    fn block_display_name(block: &BlockFacts) -> String {
        block
            .id()
            .strip_prefix("minecraft:")
            .unwrap_or(block.id())
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

    // Natural palette generators
    fn generate_forest_palette() -> Option<BlockPalette> {
        Self::generate_forest_palette_filtered(&BlockFilter::default())
    }

    fn generate_forest_palette_filtered(filter: &BlockFilter) -> Option<BlockPalette> {
        let forest_blocks = [
            "minecraft:oak_log",
            "minecraft:oak_leaves",
            "minecraft:grass_block",
            "minecraft:coarse_dirt",
            "minecraft:moss_block",
            "minecraft:fern",
        ];

        Self::create_themed_palette_filtered(
            "Forest Biome",
            "Natural forest colors with browns, greens, and earth tones",
            &forest_blocks,
            PaletteTheme::Natural,
            filter,
        )
    }

    fn generate_desert_palette() -> Option<BlockPalette> {
        Self::generate_desert_palette_filtered(&BlockFilter::default())
    }

    fn generate_desert_palette_filtered(filter: &BlockFilter) -> Option<BlockPalette> {
        let desert_blocks = [
            "minecraft:sand",
            "minecraft:sandstone",
            "minecraft:smooth_sandstone",
            "minecraft:cut_sandstone",
            "minecraft:red_sand",
            "minecraft:terracotta",
        ];

        Self::create_themed_palette_filtered(
            "Desert Biome",
            "Warm sandy colors and sun-baked earth tones",
            &desert_blocks,
            PaletteTheme::Natural,
            filter,
        )
    }

    fn generate_ocean_palette() -> Option<BlockPalette> {
        Self::generate_ocean_palette_filtered(&BlockFilter::default())
    }

    fn generate_ocean_palette_filtered(filter: &BlockFilter) -> Option<BlockPalette> {
        let ocean_blocks = [
            "minecraft:water",
            "minecraft:prismarine",
            "minecraft:dark_prismarine",
            "minecraft:sea_lantern",
            "minecraft:kelp",
            "minecraft:sand",
        ];

        Self::create_themed_palette_filtered(
            "Ocean Biome",
            "Cool blues and aquatic colors for underwater builds",
            &ocean_blocks,
            PaletteTheme::Natural,
            filter,
        )
    }

    fn generate_mountain_palette() -> Option<BlockPalette> {
        Self::generate_mountain_palette_filtered(&BlockFilter::default())
    }

    fn generate_mountain_palette_filtered(filter: &BlockFilter) -> Option<BlockPalette> {
        let mountain_blocks = [
            "minecraft:stone",
            "minecraft:cobblestone",
            "minecraft:andesite",
            "minecraft:granite",
            "minecraft:diorite",
            "minecraft:gravel",
        ];

        Self::create_themed_palette_filtered(
            "Mountain Biome",
            "Rocky grays and mineral tones for mountainous terrain",
            &mountain_blocks,
            PaletteTheme::Natural,
            filter,
        )
    }

    fn generate_nether_palette() -> Option<BlockPalette> {
        Self::generate_nether_palette_filtered(&BlockFilter::default())
    }

    fn generate_nether_palette_filtered(filter: &BlockFilter) -> Option<BlockPalette> {
        let nether_blocks = [
            "minecraft:netherrack",
            "minecraft:nether_bricks",
            "minecraft:blackstone",
            "minecraft:crimson_planks",
            "minecraft:warped_planks",
            "minecraft:soul_sand",
        ];

        Self::create_themed_palette_filtered(
            "Nether Dimension",
            "Dark reds, blacks, and otherworldly colors",
            &nether_blocks,
            PaletteTheme::Natural,
            filter,
        )
    }

    fn generate_end_palette() -> Option<BlockPalette> {
        Self::generate_end_palette_filtered(&BlockFilter::default())
    }

    fn generate_end_palette_filtered(filter: &BlockFilter) -> Option<BlockPalette> {
        let end_blocks = [
            "minecraft:end_stone",
            "minecraft:purpur_block",
            "minecraft:end_stone_bricks",
            "minecraft:obsidian",
            "minecraft:chorus_flower",
            "minecraft:chorus_plant",
        ];

        Self::create_themed_palette_filtered(
            "End Dimension",
            "Pale yellows, purples, and ethereal tones",
            &end_blocks,
            PaletteTheme::Natural,
            filter,
        )
    }

    // Architectural palette generators
    fn generate_medieval_palette() -> Option<BlockPalette> {
        Self::generate_medieval_palette_filtered(&BlockFilter::default())
    }

    fn generate_medieval_palette_filtered(filter: &BlockFilter) -> Option<BlockPalette> {
        let medieval_blocks = [
            "minecraft:cobblestone",
            "minecraft:oak_planks",
            "minecraft:stone_bricks",
            "minecraft:dark_oak_planks",
            "minecraft:mossy_cobblestone",
            "minecraft:oak_log",
        ];

        Self::create_themed_palette_filtered(
            "Medieval Architecture",
            "Traditional building materials for castles and medieval structures",
            &medieval_blocks,
            PaletteTheme::Architectural,
            filter,
        )
    }

    fn generate_modern_palette() -> Option<BlockPalette> {
        Self::generate_modern_palette_filtered(&BlockFilter::default())
    }

    fn generate_modern_palette_filtered(filter: &BlockFilter) -> Option<BlockPalette> {
        let modern_blocks = [
            "minecraft:white_concrete",
            "minecraft:light_gray_concrete",
            "minecraft:glass",
            "minecraft:iron_block",
            "minecraft:quartz_block",
            "minecraft:black_concrete",
        ];

        Self::create_themed_palette_filtered(
            "Modern Architecture",
            "Clean lines and contemporary materials for modern builds",
            &modern_blocks,
            PaletteTheme::Architectural,
            filter,
        )
    }

    fn generate_rustic_palette() -> Option<BlockPalette> {
        Self::generate_rustic_palette_filtered(&BlockFilter::default())
    }

    fn generate_rustic_palette_filtered(filter: &BlockFilter) -> Option<BlockPalette> {
        let rustic_blocks = [
            "minecraft:stripped_oak_log",
            "minecraft:cobblestone",
            "minecraft:coarse_dirt",
            "minecraft:hay_bale",
            "minecraft:oak_fence",
            "minecraft:stone",
        ];

        Self::create_themed_palette_filtered(
            "Rustic Style",
            "Natural materials for farmhouses and country builds",
            &rustic_blocks,
            PaletteTheme::Architectural,
            filter,
        )
    }

    fn generate_industrial_palette() -> Option<BlockPalette> {
        Self::generate_industrial_palette_filtered(&BlockFilter::default())
    }

    fn generate_industrial_palette_filtered(filter: &BlockFilter) -> Option<BlockPalette> {
        let industrial_blocks = [
            "minecraft:iron_block",
            "minecraft:gray_concrete",
            "minecraft:observer",
            "minecraft:anvil",
            "minecraft:cauldron",
            "minecraft:redstone_block",
        ];

        Self::create_themed_palette_filtered(
            "Industrial Style",
            "Metallic and mechanical blocks for factories and tech builds",
            &industrial_blocks,
            PaletteTheme::Architectural,
            filter,
        )
    }

    /// Helper to create themed palettes
    fn create_themed_palette(
        name: &str,
        description: &str,
        block_ids: &[&str],
        theme: PaletteTheme,
    ) -> Option<BlockPalette> {
        Self::create_themed_palette_filtered(
            name,
            description,
            block_ids,
            theme,
            &BlockFilter::default(),
        )
    }

    /// Helper to create themed palettes with filtering
    fn create_themed_palette_filtered(
        name: &str,
        description: &str,
        block_ids: &[&str],
        theme: PaletteTheme,
        filter: &BlockFilter,
    ) -> Option<BlockPalette> {
        let mut blocks = Vec::new();

        for (i, block_id) in block_ids.iter().enumerate() {
            if let Some(block) = BLOCKS.get(*block_id) {
                // Apply filter
                if !filter.allows_block(block) {
                    continue;
                }

                let role = match i {
                    0 => BlockRole::Primary,
                    1 => BlockRole::Secondary,
                    _ if i == block_ids.len() - 1 => BlockRole::Accent,
                    _ => BlockRole::Transition,
                };

                let color = block
                    .extras
                    .color
                    .map(|c| c.to_extended())
                    .unwrap_or_else(|| ExtendedColorData::from_rgb(128, 128, 128));

                let usage_notes = Self::generate_usage_notes(block, &role);

                blocks.push(BlockRecommendation {
                    block: *block,
                    color,
                    role,
                    usage_notes,
                });
            }
        }

        if blocks.is_empty() {
            return None;
        }

        Some(BlockPalette {
            name: name.to_string(),
            description: description.to_string(),
            blocks,
            theme,
        })
    }

    /// Find blocks by color similarity for custom palettes
    pub fn find_blocks_by_color_range(
        target_color: ExtendedColorData,
        tolerance: f32,
        max_blocks: usize,
    ) -> Vec<&'static BlockFacts> {
        let mut candidates: Vec<_> = BLOCKS
            .values()
            .filter_map(|block| {
                block.extras.color.map(|color| {
                    let distance = color.to_extended().distance_oklab(&target_color);
                    (*block, distance)
                })
            })
            .filter(|(_, distance)| *distance <= tolerance)
            .collect();

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates
            .into_iter()
            .take(max_blocks)
            .map(|(block, _)| block)
            .collect()
    }

    /// Get all available natural themes
    pub fn get_natural_themes() -> Vec<&'static str> {
        vec!["forest", "desert", "ocean", "mountain", "nether", "end"]
    }

    /// Get all available architectural styles
    pub fn get_architectural_styles() -> Vec<&'static str> {
        vec!["medieval", "modern", "rustic", "industrial"]
    }
}

impl BlockPalette {
    /// Export palette as a text list for easy copying
    pub fn to_text_list(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("# {}\n", self.name));
        output.push_str(&format!("{}\n\n", self.description));

        for recommendation in &self.blocks {
            output.push_str(&format!(
                "- {} ({}): {}\n",
                Self::format_block_name(recommendation.block.id()),
                recommendation.color.hex_string(),
                recommendation.usage_notes
            ));
        }

        output
    }

    /// Export palette as JSON for programmatic use
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "theme": format!("{:?}", self.theme),
            "blocks": self.blocks.iter().map(|rec| {
                serde_json::json!({
                    "id": rec.block.id(),
                    "name": Self::format_block_name(rec.block.id()),
                    "color": rec.color.hex_string(),
                    "role": format!("{:?}", rec.role),
                    "usage": rec.usage_notes
                })
            }).collect::<Vec<_>>()
        })
        .to_string()
    }

    /// Format block ID into a readable name
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
}
