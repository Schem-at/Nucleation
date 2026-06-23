//! V2553 (20w20b + 16) — schematic-relevant subset of `V2553.java`.
//!
//! The big legacy -> 1.16 BIOME value rename table (V2553.java:14-73). Applied to
//! the BIOME value type only. Nothing non-schematic is present.
//!
//! VERSION = MCVersions.V20W20B (2537) + 16 = 2553.

use super::super::helpers::{map_renamer, register_value_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2553;

const BIOME_RENAMES: &[(&str, &str)] = &[
    ("minecraft:extreme_hills", "minecraft:mountains"),
    ("minecraft:swampland", "minecraft:swamp"),
    ("minecraft:hell", "minecraft:nether_wastes"),
    ("minecraft:sky", "minecraft:the_end"),
    ("minecraft:ice_flats", "minecraft:snowy_tundra"),
    ("minecraft:ice_mountains", "minecraft:snowy_mountains"),
    ("minecraft:mushroom_island", "minecraft:mushroom_fields"),
    (
        "minecraft:mushroom_island_shore",
        "minecraft:mushroom_field_shore",
    ),
    ("minecraft:beaches", "minecraft:beach"),
    ("minecraft:forest_hills", "minecraft:wooded_hills"),
    ("minecraft:smaller_extreme_hills", "minecraft:mountain_edge"),
    ("minecraft:stone_beach", "minecraft:stone_shore"),
    ("minecraft:cold_beach", "minecraft:snowy_beach"),
    ("minecraft:roofed_forest", "minecraft:dark_forest"),
    ("minecraft:taiga_cold", "minecraft:snowy_taiga"),
    ("minecraft:taiga_cold_hills", "minecraft:snowy_taiga_hills"),
    ("minecraft:redwood_taiga", "minecraft:giant_tree_taiga"),
    (
        "minecraft:redwood_taiga_hills",
        "minecraft:giant_tree_taiga_hills",
    ),
    (
        "minecraft:extreme_hills_with_trees",
        "minecraft:wooded_mountains",
    ),
    ("minecraft:savanna_rock", "minecraft:savanna_plateau"),
    ("minecraft:mesa", "minecraft:badlands"),
    ("minecraft:mesa_rock", "minecraft:wooded_badlands_plateau"),
    ("minecraft:mesa_clear_rock", "minecraft:badlands_plateau"),
    ("minecraft:sky_island_low", "minecraft:small_end_islands"),
    ("minecraft:sky_island_medium", "minecraft:end_midlands"),
    ("minecraft:sky_island_high", "minecraft:end_highlands"),
    ("minecraft:sky_island_barren", "minecraft:end_barrens"),
    ("minecraft:void", "minecraft:the_void"),
    ("minecraft:mutated_plains", "minecraft:sunflower_plains"),
    ("minecraft:mutated_desert", "minecraft:desert_lakes"),
    (
        "minecraft:mutated_extreme_hills",
        "minecraft:gravelly_mountains",
    ),
    ("minecraft:mutated_forest", "minecraft:flower_forest"),
    ("minecraft:mutated_taiga", "minecraft:taiga_mountains"),
    ("minecraft:mutated_swampland", "minecraft:swamp_hills"),
    ("minecraft:mutated_ice_flats", "minecraft:ice_spikes"),
    ("minecraft:mutated_jungle", "minecraft:modified_jungle"),
    (
        "minecraft:mutated_jungle_edge",
        "minecraft:modified_jungle_edge",
    ),
    (
        "minecraft:mutated_birch_forest",
        "minecraft:tall_birch_forest",
    ),
    (
        "minecraft:mutated_birch_forest_hills",
        "minecraft:tall_birch_hills",
    ),
    (
        "minecraft:mutated_roofed_forest",
        "minecraft:dark_forest_hills",
    ),
    (
        "minecraft:mutated_taiga_cold",
        "minecraft:snowy_taiga_mountains",
    ),
    (
        "minecraft:mutated_redwood_taiga",
        "minecraft:giant_spruce_taiga",
    ),
    (
        "minecraft:mutated_redwood_taiga_hills",
        "minecraft:giant_spruce_taiga_hills",
    ),
    (
        "minecraft:mutated_extreme_hills_with_trees",
        "minecraft:modified_gravelly_mountains",
    ),
    ("minecraft:mutated_savanna", "minecraft:shattered_savanna"),
    (
        "minecraft:mutated_savanna_rock",
        "minecraft:shattered_savanna_plateau",
    ),
    ("minecraft:mutated_mesa", "minecraft:eroded_badlands"),
    (
        "minecraft:mutated_mesa_rock",
        "minecraft:modified_wooded_badlands_plateau",
    ),
    (
        "minecraft:mutated_mesa_clear_rock",
        "minecraft:modified_badlands_plateau",
    ),
    ("minecraft:warm_deep_ocean", "minecraft:deep_warm_ocean"),
    (
        "minecraft:lukewarm_deep_ocean",
        "minecraft:deep_lukewarm_ocean",
    ),
    ("minecraft:cold_deep_ocean", "minecraft:deep_cold_ocean"),
    ("minecraft:frozen_deep_ocean", "minecraft:deep_frozen_ocean"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_value_rename(&mut reg.biome, VERSION, 0, map_renamer(BIOME_RENAMES));
}
