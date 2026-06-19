//! V1510 (1.13-pre4+6) — schematic-relevant subset of `V1510.java`.
//!
//! Kept:
//!   * BLOCK rename (`RENAMED_BLOCKS`): portal -> nether_portal, the `*_bark`
//!     log ids -> `*_wood`, and `mob_spawner` -> `spawner` (V1510.java:33-50).
//!   * ITEM rename (`RENAMED_ITEMS` = `RENAMED_BLOCKS` + clownfish /
//!     chorus_fruit_popped / two illager spawn eggs) (V1510.java:52-60).
//!   * ENTITY rename (id + ENTITY_NAME): strips a `minecraft:bred_` prefix, then
//!     looks the result up in `RENAMED_ENTITY_IDS` (V1510.java:80-86). Implemented
//!     as a function-style `Renamer`.
//!   * ENTITY `copyWalkers` for each renamed entity id (V1510.java:96-107).
//!
//! VERSION = MCVersions.V1_13_PRE4 + 6 = 1504 + 6 = 1510.
//!
//! Skipped (non-schematic): the RECIPE rename (`RECIPES_UPDATES`,
//! V1510.java:62-71/78) and the STATS rename (V1510.java:88-93).

use std::sync::Arc;

use super::super::helpers::{
    map_renamer, register_block_rename, register_entity_rename, register_item_rename, RenameSpec,
    Renamer,
};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 1510;

/// `RENAMED_BLOCKS` (V1510.java:33-50).
const RENAMED_BLOCKS: &[(&str, &str)] = &[
    ("minecraft:portal", "minecraft:nether_portal"),
    ("minecraft:oak_bark", "minecraft:oak_wood"),
    ("minecraft:spruce_bark", "minecraft:spruce_wood"),
    ("minecraft:birch_bark", "minecraft:birch_wood"),
    ("minecraft:jungle_bark", "minecraft:jungle_wood"),
    ("minecraft:acacia_bark", "minecraft:acacia_wood"),
    ("minecraft:dark_oak_bark", "minecraft:dark_oak_wood"),
    ("minecraft:stripped_oak_bark", "minecraft:stripped_oak_wood"),
    ("minecraft:stripped_spruce_bark", "minecraft:stripped_spruce_wood"),
    ("minecraft:stripped_birch_bark", "minecraft:stripped_birch_wood"),
    ("minecraft:stripped_jungle_bark", "minecraft:stripped_jungle_wood"),
    ("minecraft:stripped_acacia_bark", "minecraft:stripped_acacia_wood"),
    ("minecraft:stripped_dark_oak_bark", "minecraft:stripped_dark_oak_wood"),
    ("minecraft:mob_spawner", "minecraft:spawner"),
];

/// `RENAMED_ITEMS` = `RENAMED_BLOCKS` + four extra item-only ids
/// (V1510.java:52-60).
const RENAMED_ITEMS: &[(&str, &str)] = &[
    // RENAMED_BLOCKS (duplicated; Java does `putAll(RENAMED_BLOCKS)`).
    ("minecraft:portal", "minecraft:nether_portal"),
    ("minecraft:oak_bark", "minecraft:oak_wood"),
    ("minecraft:spruce_bark", "minecraft:spruce_wood"),
    ("minecraft:birch_bark", "minecraft:birch_wood"),
    ("minecraft:jungle_bark", "minecraft:jungle_wood"),
    ("minecraft:acacia_bark", "minecraft:acacia_wood"),
    ("minecraft:dark_oak_bark", "minecraft:dark_oak_wood"),
    ("minecraft:stripped_oak_bark", "minecraft:stripped_oak_wood"),
    ("minecraft:stripped_spruce_bark", "minecraft:stripped_spruce_wood"),
    ("minecraft:stripped_birch_bark", "minecraft:stripped_birch_wood"),
    ("minecraft:stripped_jungle_bark", "minecraft:stripped_jungle_wood"),
    ("minecraft:stripped_acacia_bark", "minecraft:stripped_acacia_wood"),
    ("minecraft:stripped_dark_oak_bark", "minecraft:stripped_dark_oak_wood"),
    ("minecraft:mob_spawner", "minecraft:spawner"),
    // item-only extras
    ("minecraft:clownfish", "minecraft:tropical_fish"),
    ("minecraft:chorus_fruit_popped", "minecraft:popped_chorus_fruit"),
    ("minecraft:evocation_illager_spawn_egg", "minecraft:evoker_spawn_egg"),
    ("minecraft:vindication_illager_spawn_egg", "minecraft:vindicator_spawn_egg"),
];

/// `RENAMED_ENTITY_IDS` (V1510.java:16-31), used both by the entity renamer and
/// to drive `copyWalkers`.
const RENAMED_ENTITY_IDS: &[(&str, &str)] = &[
    ("minecraft:commandblock_minecart", "minecraft:command_block_minecart"),
    ("minecraft:ender_crystal", "minecraft:end_crystal"),
    ("minecraft:snowman", "minecraft:snow_golem"),
    ("minecraft:evocation_illager", "minecraft:evoker"),
    ("minecraft:evocation_fangs", "minecraft:evoker_fangs"),
    ("minecraft:illusion_illager", "minecraft:illusioner"),
    ("minecraft:vindication_illager", "minecraft:vindicator"),
    ("minecraft:villager_golem", "minecraft:iron_golem"),
    ("minecraft:xp_orb", "minecraft:experience_orb"),
    ("minecraft:xp_bottle", "minecraft:experience_bottle"),
    ("minecraft:eye_of_ender_signal", "minecraft:eye_of_ender"),
    ("minecraft:fireworks_rocket", "minecraft:firework_rocket"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(RENAMED_BLOCKS));
    register_item_rename(reg, VERSION, map_renamer(RENAMED_ITEMS));

    // Entity rename: strip a leading `minecraft:bred_` (re-prefixing with
    // `minecraft:`), then look up in RENAMED_ENTITY_IDS (V1510.java:80-86).
    let entity_renamer: Renamer = Arc::new(|input: &str| {
        let key = if let Some(rest) = input.strip_prefix("minecraft:bred_") {
            format!("minecraft:{rest}")
        } else {
            input.to_string()
        };
        RENAMED_ENTITY_IDS
            .iter()
            .find(|(old, _)| *old == key)
            .map(|(_, new)| (*new).to_string())
    });
    // Reverse: invert RENAMED_ENTITY_IDS (new -> old). The forward `bred_` prefix
    // strip is a one-way normalization with no inverse, so we restore the plain
    // (non-`bred_`) legacy id — the canonical pre-V1510 form.
    let reverse_renamer: Renamer = Arc::new(|input: &str| {
        RENAMED_ENTITY_IDS
            .iter()
            .find(|(_, new)| *new == input)
            .map(|(old, _)| (*old).to_string())
    });
    register_entity_rename(reg, VERSION, RenameSpec::custom(entity_renamer, reverse_renamer));

    // copyWalkers for each renamed entity id (V1510.java:96-107).
    for (old_id, new_id) in RENAMED_ENTITY_IDS {
        reg.entity.copy_walkers(VERSION, 0, old_id, new_id);
    }
}
