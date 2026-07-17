use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Default biome tint constants (vanilla plains biome)
// ---------------------------------------------------------------------------
//
// Some vanilla textures are grayscale and colorized in-game by a biome color
// provider. When computing representative block colors we apply the standard
// plains-biome constants (the values vanilla uses for the default/fallback
// biome):
//
// - Grass (plains):    #91BD59 - grass_block top, short/tall grass, ferns,
//   sugar cane, and the pink_petals stem/leaf parts
// - Foliage (plains):  #77AB2F - oak/jungle/acacia/dark_oak/azalea(vines)
//   leaves and vines (net.minecraft FoliageColor plains value)
// - Birch foliage:     #80A755 (fixed, biome-independent)
// - Spruce foliage:    #619961 (fixed, biome-independent)
// - Mangrove foliage:  #92C648 (fixed, biome-independent)
// - Water (default):   #3F76E4
// - Lily pad:          #208030
// - Ripe stems:        #E0C71C (melon/pumpkin stem at full growth, age=7:
//   r=age*32, g=255-age*8, b=age*4)
// - Redstone wire:     #4C0000 (unpowered, power=0 -> r=0.3*255)
//
// Cherry and pale oak leaves are NOT tinted; azalea leaves are not tinted
// either (only their vines are). Applied as a per-channel multiply.

/// Plains-biome grass tint (#91BD59)
pub const PLAINS_GRASS_TINT: [u8; 3] = [0x91, 0xBD, 0x59];
/// Plains-biome foliage tint (#77AB2F)
pub const PLAINS_FOLIAGE_TINT: [u8; 3] = [0x77, 0xAB, 0x2F];
/// Birch foliage tint (#80A755, biome-independent)
pub const BIRCH_FOLIAGE_TINT: [u8; 3] = [0x80, 0xA7, 0x55];
/// Spruce foliage tint (#619961, biome-independent)
pub const SPRUCE_FOLIAGE_TINT: [u8; 3] = [0x61, 0x99, 0x61];
/// Mangrove foliage tint (#92C648, biome-independent)
pub const MANGROVE_FOLIAGE_TINT: [u8; 3] = [0x92, 0xC6, 0x48];
/// Default water tint (#3F76E4)
pub const WATER_TINT: [u8; 3] = [0x3F, 0x76, 0xE4];
/// Lily pad tint (#208030)
pub const LILY_PAD_TINT: [u8; 3] = [0x20, 0x80, 0x30];
/// Fully-grown melon/pumpkin stem tint (#E0C71C)
pub const STEM_TINT: [u8; 3] = [0xE0, 0xC7, 0x1C];
/// Unpowered redstone wire tint (#4C0000)
pub const REDSTONE_WIRE_TINT: [u8; 3] = [0x4C, 0x00, 0x00];

/// Returns the default (plains-biome) tint that vanilla applies to a block's
/// grayscale texture, if any. `block_name` is the bare name without the
/// `minecraft:` prefix.
pub fn default_biome_tint(block_name: &str) -> Option<[u8; 3]> {
    let name = block_name.strip_prefix("potted_").unwrap_or(block_name);
    match name {
        "grass_block" | "short_grass" | "grass" | "tall_grass" | "fern" | "large_fern"
        | "sugar_cane" | "pink_petals" | "wildflowers" | "bush" | "firefly_bush"
        | "leaf_litter" => Some(PLAINS_GRASS_TINT),
        "oak_leaves" | "jungle_leaves" | "acacia_leaves" | "dark_oak_leaves" | "vine" => {
            Some(PLAINS_FOLIAGE_TINT)
        }
        "birch_leaves" => Some(BIRCH_FOLIAGE_TINT),
        "spruce_leaves" => Some(SPRUCE_FOLIAGE_TINT),
        "mangrove_leaves" => Some(MANGROVE_FOLIAGE_TINT),
        "water" | "bubble_column" | "water_cauldron" => Some(WATER_TINT),
        "lily_pad" => Some(LILY_PAD_TINT),
        "melon_stem" | "pumpkin_stem" | "attached_melon_stem" | "attached_pumpkin_stem" => {
            Some(STEM_TINT)
        }
        "redstone_wire" => Some(REDSTONE_WIRE_TINT),
        _ => None,
    }
}

/// Explicit block -> texture overrides for blocks whose representative texture
/// cannot be derived from the block name. Texture names are the file stem
/// (no `.png`) inside `assets/minecraft/textures/block/`.
fn texture_override(block_name: &str) -> Option<&'static str> {
    Some(match block_name {
        // Surface blocks: use the (tinted) top face - it is the color you see
        "grass_block" => "grass_block_top",
        "dirt_path" => "dirt_path_top",
        "farmland" => "farmland_moist",
        "podzol" => "podzol_top",
        "mycelium" => "mycelium_top",
        "crimson_nylium" => "crimson_nylium",
        "warped_nylium" => "warped_nylium",
        "snow" => "snow",
        // Fluids: still-frame animation strips
        "water" => "water_still",
        "bubble_column" => "water_still",
        "lava" => "lava_still",
        "water_cauldron" => "water_still",
        "lava_cauldron" => "lava_still",
        "powder_snow_cauldron" => "powder_snow",
        "fire" => "fire_0",
        "soul_fire" => "soul_fire_0",
        "nether_portal" => "nether_portal",
        // Plants / crops: use the mature stage
        "wheat" => "wheat_stage7",
        "carrots" => "carrots_stage3",
        "potatoes" => "potatoes_stage3",
        "beetroots" => "beetroots_stage3",
        "nether_wart" => "nether_wart_stage2",
        "torchflower_crop" => "torchflower_crop_stage1",
        "pitcher_crop" => "pitcher_crop_top_stage_4",
        "sweet_berry_bush" => "sweet_berry_bush_stage3",
        "cocoa" => "cocoa_stage2",
        "melon_stem" => "melon_stem_stage7",
        "pumpkin_stem" => "pumpkin_stem_stage7",
        "bamboo" => "bamboo_stalk",
        "bamboo_sapling" => "bamboo_stage0",
        "kelp" => "kelp_plant",
        "tall_seagrass" => "tall_seagrass_top",
        "cave_vines" => "cave_vines_lit",
        "cave_vines_plant" => "cave_vines_plant_lit",
        "short_grass" | "grass" => "short_grass",
        "tall_grass" => "tall_grass_top",
        "large_fern" => "large_fern_top",
        "sunflower" => "sunflower_front",
        "lilac" => "lilac_top",
        "rose_bush" => "rose_bush_top",
        "peony" => "peony_top",
        "pitcher_plant" => "pitcher_crop_top_stage_4",
        "chorus_flower" => "chorus_flower",
        "big_dripleaf" => "big_dripleaf_top",
        "small_dripleaf" => "small_dripleaf_top",
        "mangrove_propagule" => "mangrove_propagule",
        // Redstone / mechanisms
        "redstone_wire" => "redstone_dust_dot",
        "sticky_piston" => "piston_side",
        "piston_head" => "piston_top",
        "moving_piston" => "piston_side",
        "repeater" => "repeater",
        "comparator" => "comparator",
        "daylight_detector" => "daylight_detector_top",
        "lightning_rod" => "lightning_rod_on",
        "tripwire" => "tripwire",
        "tripwire_hook" => "tripwire_hook",
        // Tile entities whose visual is an entity texture: use a
        // color-representative block texture instead
        "chest" => "oak_planks",
        "trapped_chest" => "oak_planks",
        "ender_chest" => "obsidian",
        "copper_chest" => "copper_block",
        "exposed_copper_chest" => "exposed_copper",
        "weathered_copper_chest" => "weathered_copper",
        "oxidized_copper_chest" => "oxidized_copper",
        "waxed_copper_chest" => "copper_block",
        "waxed_exposed_copper_chest" => "exposed_copper",
        "waxed_weathered_copper_chest" => "weathered_copper",
        "waxed_oxidized_copper_chest" => "oxidized_copper",
        "shulker_box" => "purple_wool",
        "conduit" => "prismarine",
        "beacon" => "beacon",
        "end_portal" => "obsidian",
        "end_gateway" => "obsidian",
        "bell" => "gold_block",
        "decorated_pot" => "terracotta",
        // Smooth variants keep the flat face of their base material
        "snow_block" => "snow",
        "smooth_sandstone" => "sandstone_top",
        "smooth_red_sandstone" => "red_sandstone_top",
        "smooth_quartz" => "quartz_block_bottom",
        // Weighted pressure plates are made of their metal
        "heavy_weighted_pressure_plate" => "iron_block",
        "light_weighted_pressure_plate" => "gold_block",
        "dried_kelp_block" => "dried_kelp_side",
        // Misc
        "cake" => "cake_side",
        "frosted_ice" => "frosted_ice_0",
        "magma_block" => "magma",
        "jigsaw" => "jigsaw_side",
        "structure_block" => "structure_block",
        "lodestone" => "lodestone_side",
        "stonecutter" => "stonecutter_side",
        "grindstone" => "grindstone_side",
        "hopper" => "hopper_outside",
        "cauldron" => "cauldron_side",
        "composter" => "composter_side",
        "respawn_anchor" => "respawn_anchor_side1",
        "scaffolding" => "scaffolding_side",
        "creaking_heart" => "creaking_heart",
        "trial_spawner" => "trial_spawner_side_inactive",
        "vault" => "vault_side_off",
        "suspicious_sand" => "suspicious_sand_0",
        "suspicious_gravel" => "suspicious_gravel_0",
        "sculk_sensor" => "sculk_sensor_top",
        "calibrated_sculk_sensor" => "calibrated_sculk_sensor_top",
        "sniffer_egg" => "sniffer_egg_not_cracked_top",
        "dried_ghast" => "dried_ghast_hydration_0_top",
        "chiseled_bookshelf" => "chiseled_bookshelf_occupied",
        "copper_golem_statue" => "copper_block",
        "weathered_copper_golem_statue" => "weathered_copper",
        "exposed_copper_golem_statue" => "exposed_copper",
        "oxidized_copper_golem_statue" => "oxidized_copper",
        "waxed_copper_golem_statue" => "copper_block",
        "waxed_weathered_copper_golem_statue" => "weathered_copper",
        "waxed_exposed_copper_golem_statue" => "exposed_copper",
        "waxed_oxidized_copper_golem_statue" => "oxidized_copper",
        _ => return None,
    })
}

/// Resolve the representative texture (file stem, no `.png`) for a block.
///
/// `block_name` is the bare block name (no `minecraft:` prefix) and
/// `available` is the set of texture file stems extracted from the client
/// jar's `assets/minecraft/textures/block/` directory.
///
/// Resolution order:
/// 1. explicit override table
/// 2. exact name, then common face suffixes (`_side`, `_top`, ...)
/// 3. structural derivations (stairs/slabs/walls -> base material, wood ->
///    log, carpet/bed/banner -> wool, sign -> planks, `waxed_`/`infested_`/
///    `potted_`/`wall_` stripping, ...)
/// 4. prefix scan over the available set (picks `*_side` > `*_top` >
///    `*_front` > lexicographically last, which favors mature crop stages)
pub fn resolve_texture(block_name: &str, available: &HashSet<String>) -> Option<String> {
    let name = block_name.strip_prefix("minecraft:").unwrap_or(block_name);

    // 1. Explicit override
    if let Some(t) = texture_override(name) {
        if available.contains(t) {
            return Some(t.to_string());
        }
    }

    // 2. Direct candidates
    for cand in [
        name.to_string(),
        format!("{name}_side"),
        format!("{name}_top"),
        format!("{name}_front"),
        format!("{name}_still"),
        format!("{name}_bottom"),
    ] {
        if available.contains(&cand) {
            return Some(cand);
        }
    }

    // 3. Structural derivations -> recurse on the base material name
    let recurse = |base: &str| resolve_texture(base, available);

    // Prefix strips
    for prefix in ["waxed_", "infested_", "potted_"] {
        if let Some(base) = name.strip_prefix(prefix) {
            if let Some(t) = recurse(base) {
                return Some(t);
            }
        }
    }
    // candle_cake and its 16 dyed variants all look like cake
    if name.ends_with("candle_cake") && available.contains("cake_side") {
        return Some("cake_side".to_string());
    }
    // wall_torch, oak_wall_sign, dead_brain_coral_wall_fan, ...
    if name.contains("wall_") {
        let base = name.replacen("wall_", "", 1);
        if let Some(t) = recurse(&base) {
            return Some(t);
        }
    }

    // Suffix-based derivations
    let suffix_rules: [(&str, fn(&str) -> Vec<String>); 14] = [
        // Wooden variants must resolve to planks before the generic prefix
        // scan can pick e.g. oak_log_top for oak_stairs.
        ("_stairs", |b| vec![b.to_string(), format!("{b}s"), format!("{b}_planks")]),
        ("_slab", |b| vec![b.to_string(), format!("{b}s"), format!("{b}_planks")]),
        ("_wall", |b| vec![b.to_string(), format!("{b}s")]),
        ("_button", |b| vec![b.to_string(), format!("{b}s"), format!("{b}_planks")]),
        ("_pressure_plate", |b| vec![b.to_string(), format!("{b}s"), format!("{b}_planks")]),
        ("_fence_gate", |b| vec![format!("{b}_planks"), b.to_string(), format!("{b}s")]),
        ("_fence", |b| vec![format!("{b}_planks"), b.to_string(), format!("{b}s")]),
        ("_wood", |b| vec![format!("{b}_log")]),
        ("_hyphae", |b| vec![format!("{b}_stem")]),
        ("_carpet", |b| vec![b.to_string(), format!("{b}_wool")]),
        ("_bed", |b| vec![format!("{b}_wool")]),
        ("_banner", |b| vec![format!("{b}_wool")]),
        ("_shulker_box", |b| vec![format!("{b}_wool")]),
        ("_sign", |b| {
            vec![
                format!("{b}_planks"),
                format!("stripped_{b}_log"),
                format!("stripped_{b}_stem"),
                format!("{b}_block"),
                b.to_string(),
            ]
        }),
    ];
    for (suffix, candidates) in suffix_rules {
        if let Some(base) = name.strip_suffix(suffix) {
            // `_sign` also covers `_hanging_sign`
            let base = base.strip_suffix("_hanging").unwrap_or(base);
            for cand in candidates(base) {
                if available.contains(&cand) {
                    return Some(cand);
                }
            }
            // Fall back to recursion for compound bases
            // (e.g. waxed_oxidized_cut_copper_stairs)
            if let Some(t) = recurse(base) {
                return Some(t);
            }
        }
    }

    // Special slab/base spellings that recursion can't guess
    if let Some(base) = name.strip_suffix("_slab").or(name.strip_suffix("_stairs")) {
        if base == "petrified_oak" {
            if available.contains("oak_planks") {
                return Some("oak_planks".to_string());
            }
        }
    }

    // 4. Prefix scan over available textures
    let prefix = format!("{name}_");
    let mut matches: Vec<&String> = available.iter().filter(|t| t.starts_with(&prefix)).collect();
    if !matches.is_empty() {
        matches.sort();
        for face in ["_side", "_top", "_front"] {
            if let Some(t) = matches.iter().find(|t| t.ends_with(face)) {
                return Some((*t).clone());
            }
        }
        // Lexicographically last favors the highest crop/growth stage
        return Some((*matches.last().unwrap()).clone());
    }

    None
}

/// Maps Minecraft block IDs to texture file names
#[derive(Debug, Clone)]
pub struct TextureMapping {
    mappings: HashMap<String, String>,
}

impl TextureMapping {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Get texture file for a block ID
    pub fn get_texture(&self, block_id: &str) -> Option<&String> {
        self.mappings.get(block_id)
    }

    /// Get all mappings
    pub fn all_mappings(&self) -> &HashMap<String, String> {
        &self.mappings
    }

    /// Get statistics about texture coverage
    pub fn get_coverage_stats(&self, total_blocks: usize) -> TextureCoverageStats {
        TextureCoverageStats {
            total_blocks,
            blocks_with_textures: self.mappings.len(),
            unique_textures: self
                .mappings
                .values()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            coverage_percentage: (self.mappings.len() as f32 / total_blocks as f32) * 100.0,
        }
    }
}

impl Default for TextureMapping {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TextureCoverageStats {
    pub total_blocks: usize,
    pub blocks_with_textures: usize,
    pub unique_textures: usize,
    pub coverage_percentage: f32,
}

impl std::fmt::Display for TextureCoverageStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Texture Coverage: {}/{} blocks ({:.1}%) using {} unique textures",
            self.blocks_with_textures,
            self.total_blocks,
            self.coverage_percentage,
            self.unique_textures
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn avail(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn resolves_direct_and_derived_textures() {
        let a = avail(&[
            "stone",
            "oak_planks",
            "oak_log",
            "cut_copper",
            "purple_wool",
            "grass_block_top",
            "cake_side",
        ]);

        assert_eq!(resolve_texture("stone", &a).as_deref(), Some("stone"));
        // Wooden derivatives resolve to planks
        assert_eq!(resolve_texture("oak_stairs", &a).as_deref(), Some("oak_planks"));
        assert_eq!(resolve_texture("oak_fence", &a).as_deref(), Some("oak_planks"));
        assert_eq!(resolve_texture("oak_sign", &a).as_deref(), Some("oak_planks"));
        assert_eq!(resolve_texture("oak_wall_sign", &a).as_deref(), Some("oak_planks"));
        // Wood -> log
        assert_eq!(resolve_texture("oak_wood", &a).as_deref(), Some("oak_log"));
        // waxed_ stripping recurses
        assert_eq!(
            resolve_texture("waxed_cut_copper_slab", &a).as_deref(),
            Some("cut_copper")
        );
        // Wool stand-ins
        assert_eq!(resolve_texture("purple_bed", &a).as_deref(), Some("purple_wool"));
        // Overrides
        assert_eq!(resolve_texture("grass_block", &a).as_deref(), Some("grass_block_top"));
        assert_eq!(resolve_texture("magenta_candle_cake", &a).as_deref(), Some("cake_side"));
        // minecraft: prefix is accepted
        assert_eq!(resolve_texture("minecraft:stone", &a).as_deref(), Some("stone"));
        // Unknown blocks resolve to nothing
        assert_eq!(resolve_texture("air", &a), None);
    }

    #[test]
    fn prefix_scan_prefers_faces_then_last_stage() {
        let a = avail(&["furnace_front", "furnace_side", "furnace_top"]);
        assert_eq!(resolve_texture("furnace", &a).as_deref(), Some("furnace_side"));

        let a = avail(&["wheat_stage0", "wheat_stage3", "wheat_stage7"]);
        assert_eq!(resolve_texture("wheat", &a).as_deref(), Some("wheat_stage7"));
    }

    #[test]
    fn default_tints_cover_the_documented_blocks() {
        assert_eq!(default_biome_tint("grass_block"), Some(PLAINS_GRASS_TINT));
        assert_eq!(default_biome_tint("oak_leaves"), Some(PLAINS_FOLIAGE_TINT));
        assert_eq!(default_biome_tint("birch_leaves"), Some(BIRCH_FOLIAGE_TINT));
        assert_eq!(default_biome_tint("spruce_leaves"), Some(SPRUCE_FOLIAGE_TINT));
        assert_eq!(default_biome_tint("mangrove_leaves"), Some(MANGROVE_FOLIAGE_TINT));
        assert_eq!(default_biome_tint("water"), Some(WATER_TINT));
        assert_eq!(default_biome_tint("lily_pad"), Some(LILY_PAD_TINT));
        assert_eq!(default_biome_tint("melon_stem"), Some(STEM_TINT));
        assert_eq!(default_biome_tint("redstone_wire"), Some(REDSTONE_WIRE_TINT));
        // Cherry / pale oak leaves are NOT tinted
        assert_eq!(default_biome_tint("cherry_leaves"), None);
        assert_eq!(default_biome_tint("pale_oak_leaves"), None);
        assert_eq!(default_biome_tint("stone"), None);
    }
}
