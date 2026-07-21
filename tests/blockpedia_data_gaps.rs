//! Tests for the three blockpedia data gaps closed against the 26.2
//! official reports:
//!
//! 1. `default_state` is populated from the vanilla report state flagged
//!    `"default": true` (was empty for every block).
//! 2. `has_block_entity` comes from the `block_entity_type` registry joined
//!    to the report's definition kinds (was a name-substring guess).
//! 3. Mushroom blocks are full cubes via the extractor's geometry override
//!    (their six-face multipart shell is not classifiable from models).
//!
//! Plus: `is_light_source` now reads the prismarine `emitLight` data field.

use nucleation::blockpedia::color::block_palettes::BlockFilter;
use nucleation::blockpedia::{all_blocks, get_block, AllBlocks, BlockState};

// --- gap 1: default_state ---------------------------------------------------

#[test]
fn default_state_matches_the_vanilla_report() {
    let stairs = get_block("minecraft:oak_stairs").unwrap();
    let mut defaults: Vec<(&str, &str)> = stairs.default_state.to_vec();
    defaults.sort_unstable();
    assert_eq!(
        defaults,
        vec![
            ("facing", "north"),
            ("half", "bottom"),
            ("shape", "straight"),
            ("waterlogged", "false"),
        ]
    );
    assert_eq!(stairs.get_property("facing"), Some("north"));

    // A sampling of other well-known vanilla defaults
    let lever = get_block("minecraft:lever").unwrap();
    assert_eq!(lever.get_property("powered"), Some("false"));
    let snow = get_block("minecraft:snow").unwrap();
    assert_eq!(snow.get_property("layers"), Some("1"));
    let campfire = get_block("minecraft:campfire").unwrap();
    assert_eq!(campfire.get_property("lit"), Some("true"));

    // Property-less blocks have an empty default state
    assert!(get_block("minecraft:stone")
        .unwrap()
        .default_state
        .is_empty());
}

#[test]
fn every_block_has_a_complete_default_state() {
    // For all 1196 blocks the default state must assign exactly the
    // block's property set, and every value must be a legal one.
    for block in all_blocks() {
        let props: std::collections::BTreeSet<&str> =
            block.properties.iter().map(|(name, _)| *name).collect();
        let defaults: std::collections::BTreeSet<&str> =
            block.default_state.iter().map(|(name, _)| *name).collect();
        assert_eq!(
            props,
            defaults,
            "{}: default_state keys must match the property set",
            block.id()
        );
        for (name, value) in block.default_state {
            let allowed = block
                .get_property_values(name)
                .unwrap_or_else(|| panic!("{}: unknown default property {name}", block.id()));
            assert!(
                allowed.contains(&value.to_string()),
                "{}: default {name}={value} not in {allowed:?}",
                block.id()
            );
        }
    }
}

#[test]
fn block_state_from_default_carries_the_report_defaults() {
    let stairs = get_block("minecraft:oak_stairs").unwrap();
    let state = BlockState::from_default(stairs).unwrap();
    assert_eq!(state.get_property("facing"), Some("north"));
    assert_eq!(state.get_property("half"), Some("bottom"));
    assert_eq!(state.get_property("shape"), Some("straight"));
    assert_eq!(state.get_property("waterlogged"), Some("false"));
}

// --- gap 2: has_block_entity ------------------------------------------------

#[test]
fn block_entities_come_from_the_registry_not_substrings() {
    // Classics the old substring guess also caught
    for id in [
        "minecraft:chest",
        "minecraft:trapped_chest",
        "minecraft:ender_chest",
        "minecraft:furnace",
        "minecraft:blast_furnace",
        "minecraft:hopper",
        "minecraft:barrel",
        "minecraft:campfire",
        "minecraft:jukebox",
    ] {
        assert!(get_block(id).unwrap().has_block_entity(), "{id}");
    }

    // Block entities the substrings missed entirely
    for id in [
        "minecraft:oak_sign",
        "minecraft:oak_wall_sign",
        "minecraft:oak_hanging_sign",
        "minecraft:red_banner",
        "minecraft:skeleton_skull",
        "minecraft:player_head",
        "minecraft:spawner",
        "minecraft:trial_spawner",
        "minecraft:command_block",
        "minecraft:conduit",
        "minecraft:bell",
        "minecraft:beehive",
        "minecraft:sculk_sensor",
        "minecraft:decorated_pot",
        "minecraft:suspicious_sand",
        "minecraft:moving_piston",
        "minecraft:copper_chest",
        "minecraft:oak_shelf",
    ] {
        assert!(get_block(id).unwrap().has_block_entity(), "{id}");
    }

    // Not block entities (the piston *bases* carry none — only the
    // in-motion technical block does; beds lost theirs in the 26.x era)
    for id in [
        "minecraft:stone",
        "minecraft:oak_stairs",
        "minecraft:piston",
        "minecraft:sticky_piston",
        "minecraft:crafting_table",
        "minecraft:red_bed",
        "minecraft:torch",
    ] {
        assert!(!get_block(id).unwrap().has_block_entity(), "{id}");
    }

    // 26.2: 186 blocks across the 49 block_entity_type registry entries
    let count = all_blocks().filter(|b| b.has_block_entity()).count();
    assert!(
        (150..=250).contains(&count),
        "block entity count out of range: {count}"
    );
}

#[test]
fn tile_entity_filters_use_the_registry_data() {
    // BlockQuery::exclude_tile_entities
    let ids: Vec<&str> = AllBlocks::new()
        .exclude_tile_entities()
        .collect()
        .iter()
        .map(|b| b.id())
        .collect();
    assert!(ids.contains(&"minecraft:stone"));
    assert!(!ids.contains(&"minecraft:chest"));
    assert!(
        !ids.contains(&"minecraft:oak_sign"),
        "substring guess missed signs"
    );

    // BlockFilter::exclude_tile_entities
    let filter = BlockFilter {
        exclude_tile_entities: true,
        ..Default::default()
    };
    assert!(filter.allows_block(get_block("minecraft:stone").unwrap()));
    assert!(!filter.allows_block(get_block("minecraft:chest").unwrap()));
    assert!(!filter.allows_block(get_block("minecraft:red_banner").unwrap()));
    assert!(!filter.allows_block(get_block("minecraft:spawner").unwrap()));
}

// --- gap 3: mushroom full cubes ----------------------------------------------

#[test]
fn mushroom_blocks_are_full_cubes_via_the_geometry_override() {
    for id in [
        "minecraft:brown_mushroom_block",
        "minecraft:red_mushroom_block",
        "minecraft:mushroom_stem",
    ] {
        assert!(get_block(id).unwrap().is_full_cube(), "{id}");
    }
    // The small mushroom plants stay non-full
    assert!(!get_block("minecraft:brown_mushroom")
        .unwrap()
        .is_full_cube());
    // ...and vines, whose geometry the mushroom shell mimics, stay non-full
    assert!(!get_block("minecraft:vine").unwrap().is_full_cube());
}

// --- light sources from emitLight --------------------------------------------

#[test]
fn light_sources_use_the_emit_light_data_field() {
    assert_eq!(get_block("minecraft:glowstone").unwrap().emit_light, 15);
    assert_eq!(get_block("minecraft:torch").unwrap().emit_light, 14);
    assert_eq!(get_block("minecraft:magma_block").unwrap().emit_light, 3);
    assert_eq!(get_block("minecraft:stone").unwrap().emit_light, 0);
    assert!(get_block("minecraft:glowstone").unwrap().is_light_source());
    assert!(!get_block("minecraft:stone").unwrap().is_light_source());

    // State-dependent emitters count their default state: campfires are lit
    // by default, redstone lamps and candles are not.
    assert!(get_block("minecraft:campfire").unwrap().is_light_source());
    assert!(!get_block("minecraft:redstone_lamp")
        .unwrap()
        .is_light_source());
    assert!(!get_block("minecraft:candle").unwrap().is_light_source());

    let ids: Vec<&str> = AllBlocks::new()
        .exclude_light_sources()
        .collect()
        .iter()
        .map(|b| b.id())
        .collect();
    assert!(!ids.contains(&"minecraft:glowstone"));
    assert!(!ids.contains(&"minecraft:lava"));
    assert!(ids.contains(&"minecraft:stone"));
    // Unlit by default -> no longer excluded as a light source
    assert!(ids.contains(&"minecraft:redstone_lamp"));

    let filter = BlockFilter {
        exclude_light_sources: true,
        ..Default::default()
    };
    assert!(!filter.allows_block(get_block("minecraft:glowstone").unwrap()));
    assert!(filter.allows_block(get_block("minecraft:redstone_lamp").unwrap()));
}
