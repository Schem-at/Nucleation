//! Tests for the official block semantics facet (kind / base block / tags /
//! full-cube geometry) extracted from the 26.2 server + client jars into
//! `data/blockpedia/block_semantics.json.gz`.

use nucleation::blockpedia::color::block_palettes::BlockFilter;
use nucleation::blockpedia::{all_blocks, blocks_by_tag, get_block, variants_of};

#[test]
fn kind_and_base_come_from_the_vanilla_report() {
    let stairs = get_block("minecraft:oak_stairs").unwrap();
    assert_eq!(stairs.kind(), "minecraft:stair");
    assert_eq!(stairs.base_block(), Some("minecraft:oak_planks"));

    let slab = get_block("minecraft:oak_slab").unwrap();
    assert_eq!(slab.kind(), "minecraft:slab");
    // Slabs carry no base_state in the report; the base comes from the
    // model-texture linkage (oak_slab renders with block/oak_planks).
    assert_eq!(slab.base_block(), Some("minecraft:oak_planks"));

    let door = get_block("minecraft:oak_door").unwrap();
    assert_eq!(door.kind(), "minecraft:door");

    let stone = get_block("minecraft:stone").unwrap();
    assert_eq!(stone.kind(), "minecraft:block");
    assert_eq!(stone.base_block(), None);
}

#[test]
fn blocks_carry_vanilla_tags() {
    let wool = get_block("minecraft:red_wool").unwrap();
    assert!(wool.has_tag("minecraft:wool"));
    assert!(wool.has_tag("wool"), "short tag form should work");
    assert!(!wool.has_tag("minecraft:planks"));

    let stone = get_block("minecraft:stone").unwrap();
    assert!(stone.has_tag("minecraft:mineable/pickaxe"));
    assert!(stone.has_tag("mineable/pickaxe"));

    let planks = get_block("minecraft:oak_planks").unwrap();
    assert!(planks.has_tag("minecraft:planks"));
    assert!(planks.has_tag("minecraft:mineable/axe"));

    let stairs = get_block("minecraft:oak_stairs").unwrap();
    assert!(stairs.has_tag("minecraft:stairs"));
    assert!(stairs.has_tag("minecraft:wooden_stairs"));
}

#[test]
fn blocks_by_tag_resolves_the_full_tag() {
    let wool: Vec<_> = blocks_by_tag("minecraft:wool").map(|b| b.id()).collect();
    assert_eq!(wool.len(), 16, "16 wool colors: {wool:?}");
    assert!(wool.contains(&"minecraft:red_wool"));

    // Short form and nested tag paths work too
    assert_eq!(blocks_by_tag("wool").count(), 16);
    assert!(blocks_by_tag("mineable/pickaxe").count() > 300);
    assert_eq!(blocks_by_tag("no_such_tag").count(), 0);
}

#[test]
fn variants_of_links_shape_blocks_to_their_base() {
    let ids: Vec<&str> = variants_of("minecraft:oak_planks")
        .iter()
        .map(|b| b.id())
        .collect();
    assert!(ids.contains(&"minecraft:oak_stairs"), "got {ids:?}");
    assert!(ids.contains(&"minecraft:oak_slab"), "got {ids:?}");
    assert!(ids.contains(&"minecraft:oak_fence"), "got {ids:?}");

    let stone_variants: Vec<&str> = variants_of("minecraft:stone_bricks")
        .iter()
        .map(|b| b.id())
        .collect();
    assert!(stone_variants.contains(&"minecraft:stone_brick_stairs"));
    assert!(stone_variants.contains(&"minecraft:stone_brick_wall"));
}

#[test]
fn full_cube_is_model_geometry_not_name_guessing() {
    assert!(get_block("minecraft:stone").unwrap().is_full_cube());
    assert!(!get_block("minecraft:oak_stairs").unwrap().is_full_cube());
    assert!(!get_block("minecraft:oak_slab").unwrap().is_full_cube());

    // Glass is a full cube geometrically while still transparent
    let glass = get_block("minecraft:glass").unwrap();
    assert!(glass.is_full_cube());
    assert!(glass.transparent);

    // Non-template cubes caught by the full-16^3-element fallback
    assert!(get_block("minecraft:grass_block").unwrap().is_full_cube());
    assert!(get_block("minecraft:command_block").unwrap().is_full_cube());

    // Mushroom blocks render as a multipart shell of six face planes —
    // model geometry indistinguishable from vines — but are full opaque
    // cubes in game; the extractor's `full_cube_override` settles them.
    assert!(get_block("minecraft:brown_mushroom_block")
        .unwrap()
        .is_full_cube());
    assert!(get_block("minecraft:red_mushroom_block")
        .unwrap()
        .is_full_cube());
    assert!(get_block("minecraft:mushroom_stem").unwrap().is_full_cube());

    // Shapes without telltale name substrings are now correctly non-full
    assert!(!get_block("minecraft:rose_bush").unwrap().is_full_cube());
    assert!(!get_block("minecraft:rail").unwrap().is_full_cube());
    assert!(!get_block("minecraft:snow").unwrap().is_full_cube());
}

#[test]
fn semantics_coverage_is_sane() {
    let total = all_blocks().count();
    let full_cubes = all_blocks().filter(|b| b.is_full_cube()).count();
    let tagged = all_blocks().filter(|b| !b.tags.is_empty()).count();
    let non_generic_kind = all_blocks()
        .filter(|b| b.kind() != "minecraft:block")
        .count();
    let with_base = all_blocks().filter(|b| b.base_block().is_some()).count();

    assert!(total >= 1196, "expected 26.2 block count, got {total}");
    // 26.2 extraction: 427 full cubes (424 from models + 3 mushroom
    // overrides), 1132 tagged, 1015 non-generic kinds, 234 base links
    // (64 report base_state + 170 model-texture).
    assert!(full_cubes >= 400, "full cube count regressed: {full_cubes}");
    assert!(
        tagged * 100 >= total * 90,
        "tag coverage regressed: {tagged}/{total}"
    );
    assert!(
        non_generic_kind >= 1000,
        "kind coverage regressed: {non_generic_kind}"
    );
    assert!(
        with_base >= 220,
        "base-link coverage regressed: {with_base}"
    );
}

#[test]
fn filter_supports_tags_and_kinds() {
    let wool_only = BlockFilter {
        required_tags: vec!["wool".to_string()],
        ..Default::default()
    };
    let matches: Vec<&str> = all_blocks()
        .filter(|b| wool_only.allows_block(b))
        .map(|b| b.id())
        .collect();
    assert_eq!(matches.len(), 16, "{matches:?}");
    assert!(matches.contains(&"minecraft:red_wool"));

    let no_wool = BlockFilter {
        excluded_tags: vec!["minecraft:wool".to_string()],
        ..Default::default()
    };
    assert!(!no_wool.allows_block(get_block("minecraft:red_wool").unwrap()));
    assert!(no_wool.allows_block(get_block("minecraft:stone").unwrap()));

    let stairs_and_slabs = BlockFilter {
        kinds: vec!["stair".to_string(), "minecraft:slab".to_string()],
        ..Default::default()
    };
    assert!(stairs_and_slabs.allows_block(get_block("minecraft:oak_stairs").unwrap()));
    assert!(stairs_and_slabs.allows_block(get_block("minecraft:oak_slab").unwrap()));
    assert!(!stairs_and_slabs.allows_block(get_block("minecraft:stone").unwrap()));
}

#[test]
fn full_blocks_only_uses_real_geometry() {
    let filter = BlockFilter {
        full_blocks_only: true,
        ..Default::default()
    };
    assert!(filter.allows_block(get_block("minecraft:stone").unwrap()));
    assert!(filter.allows_block(get_block("minecraft:grass_block").unwrap()));
    assert!(!filter.allows_block(get_block("minecraft:oak_stairs").unwrap()));
    // The old substring heuristic wrongly passed these non-cubes
    // (no "slab"/"stairs"/... in the name); the model data rejects them.
    assert!(!filter.allows_block(get_block("minecraft:rose_bush").unwrap()));
    assert!(!filter.allows_block(get_block("minecraft:cake").unwrap()));
    assert!(!filter.allows_block(get_block("minecraft:hopper").unwrap()));
}
