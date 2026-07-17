//! Tests for the block-query JSON API (`blockpedia::facts_json`, the core
//! behind the bridge `Blocks` opaque) and the metadata-driven
//! `PaletteBuilder` filters (tags / kinds).

use nucleation::blockpedia::facts_json::{
    all_block_ids_json, all_tags_json, block_count, block_facts_json, block_ids_by_kind_json,
    block_ids_by_tag_json, block_states_json, block_states_json_with_limit, variants_of_ids_json,
};
use nucleation::building::BlockPalette;
use serde_json::Value;

fn parse(json: &str) -> Value {
    serde_json::from_str(json).expect("valid JSON")
}

#[test]
fn get_json_reports_full_facts_for_oak_stairs() {
    let v = parse(&block_facts_json("minecraft:oak_stairs").unwrap());
    assert_eq!(v["id"], "minecraft:oak_stairs");
    assert_eq!(v["kind"], "minecraft:stair");
    assert_eq!(v["base_block"], "minecraft:oak_planks");
    assert_eq!(v["full_cube"], false);
    let tags: Vec<&str> = v["tags"].as_array().unwrap().iter().map(|t| t.as_str().unwrap()).collect();
    assert!(tags.contains(&"minecraft:stairs"), "{tags:?}");
    assert!(tags.contains(&"minecraft:wooden_stairs"), "{tags:?}");
    let facing = v["properties"]["facing"].as_array().unwrap();
    assert!(facing.iter().any(|f| f == "north"));
    assert!(v["color"].is_array() && v["color"].as_array().unwrap().len() == 3);
    assert!(v["default_state"].is_object());

    // Short form resolves too; base_block is null for base blocks
    let planks = parse(&block_facts_json("oak_planks").unwrap());
    assert_eq!(planks["kind"], "minecraft:block");
    assert!(planks["base_block"].is_null());

    assert!(block_facts_json("minecraft:no_such_block").is_none());
}

#[test]
fn ids_json_lists_every_block_sorted() {
    let ids = parse(&all_block_ids_json());
    let ids: Vec<&str> = ids.as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(ids.len(), block_count());
    assert!(ids.len() >= 1196);
    assert!(ids.contains(&"minecraft:stone"));
    assert!(ids.windows(2).all(|w| w[0] < w[1]), "sorted, no dups");
}

#[test]
fn by_tag_json_finds_the_16_wools() {
    let wool = parse(&block_ids_by_tag_json("wool"));
    assert_eq!(wool.as_array().unwrap().len(), 16, "{wool}");
    assert!(wool.as_array().unwrap().iter().any(|v| v == "minecraft:red_wool"));
    // Prefixed form matches the short form
    assert_eq!(block_ids_by_tag_json("minecraft:wool"), block_ids_by_tag_json("wool"));
    assert_eq!(block_ids_by_tag_json("no_such_tag"), "[]");
}

#[test]
fn by_kind_json_groups_by_definition_kind() {
    let stairs = parse(&block_ids_by_kind_json("stair"));
    let stairs = stairs.as_array().unwrap();
    assert!(stairs.iter().any(|v| v == "minecraft:oak_stairs"));
    assert!(stairs.len() >= 30, "expected many stair kinds, got {}", stairs.len());
    assert_eq!(
        block_ids_by_kind_json("minecraft:stair"),
        block_ids_by_kind_json("stair")
    );
    assert_eq!(block_ids_by_kind_json("no_such_kind"), "[]");
}

#[test]
fn variants_of_json_puts_the_base_first() {
    let v = parse(&variants_of_ids_json("minecraft:oak_planks").unwrap());
    let ids: Vec<&str> = v.as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(ids[0], "minecraft:oak_planks", "base first: {ids:?}");
    assert!(ids.contains(&"minecraft:oak_stairs"), "{ids:?}");
    assert!(ids.contains(&"minecraft:oak_slab"), "{ids:?}");
    assert!(variants_of_ids_json("minecraft:no_such_block").is_none());
}

#[test]
fn tags_json_lists_known_tag_names() {
    let tags = parse(&all_tags_json());
    let tags: Vec<&str> = tags.as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert!(tags.contains(&"minecraft:wool"));
    assert!(tags.contains(&"minecraft:mineable/pickaxe"));
    assert!(tags.windows(2).all(|w| w[0] < w[1]), "sorted, no dups");
}

#[test]
fn states_json_enumerates_every_combination() {
    // lever: face (3) x facing (4) x powered (2) = 24 states
    let states = parse(&block_states_json("minecraft:lever").unwrap());
    let states = states.as_array().unwrap();
    assert_eq!(states.len(), 24);
    assert!(states.iter().all(|s| {
        let o = s.as_object().unwrap();
        o.contains_key("face") && o.contains_key("facing") && o.contains_key("powered")
    }));
    // All combinations are distinct
    let unique: std::collections::HashSet<String> =
        states.iter().map(|s| s.to_string()).collect();
    assert_eq!(unique.len(), 24);

    // Property-less blocks have exactly one (empty) state
    let stone = parse(&block_states_json("minecraft:stone").unwrap());
    assert_eq!(stone.as_array().unwrap().len(), 1);
    assert_eq!(stone[0], serde_json::json!({}));

    assert!(block_states_json("minecraft:no_such_block").is_err());
}

#[test]
fn states_json_guards_against_combinatorial_blowup() {
    // No current vanilla block exceeds the 4096 default (max is
    // note_block at 1350) — exercise the guard through the limit knob.
    assert!(block_states_json_with_limit("minecraft:lever", 10).is_err());
    assert!(block_states_json_with_limit("minecraft:lever", 24).is_ok());
    assert!(block_states_json("minecraft:note_block").is_ok());
}

#[test]
fn palette_builder_tag_filter_matches_the_wool_preset() {
    let tagged = BlockPalette::builder().tag("wool").build();
    let preset = BlockPalette::new_wool();
    let mut tagged_ids: Vec<&str> = tagged.block_ids().collect();
    let mut preset_ids: Vec<&str> = preset.block_ids().collect();
    tagged_ids.sort_unstable();
    preset_ids.sort_unstable();
    assert_eq!(tagged_ids, preset_ids);
    assert_eq!(tagged_ids.len(), 16);
}

#[test]
fn palette_builder_kind_and_exclude_tag_filters() {
    let stairs = BlockPalette::builder().kind("stair").build();
    assert!(stairs.block_ids().any(|id| id == "minecraft:oak_stairs"));
    assert!(stairs.block_ids().all(|id| id.ends_with("_stairs")));

    let no_wool = BlockPalette::builder().exclude_tag("wool").build();
    assert!(no_wool.block_ids().all(|id| !id.ends_with("_wool")));
    assert!(no_wool.block_ids().any(|id| id == "minecraft:stone"));
}

#[test]
fn palettes_still_exclude_technical_blocks_by_kind() {
    // NON_BUILDABLE is now derived from definition kinds; the all-palette
    // must still reject the technical colored blocks the id list covered.
    let all = BlockPalette::new_all();
    for id in [
        "minecraft:water",
        "minecraft:lava",
        "minecraft:nether_portal",
        "minecraft:fire",
        "minecraft:redstone_wire",
    ] {
        assert!(all.block_ids().all(|b| b != id), "{id} must stay excluded");
    }
    assert!(all.block_ids().any(|b| b == "minecraft:stone"));
}
