//! Tests covering the 1.21.11 data refresh and the texture color pipeline.

use nucleation::blockpedia::color::block_palettes::{BlockFilter, BlockPaletteGenerator};
use nucleation::blockpedia::{all_blocks, get_block};

#[test]
fn new_1_21_blocks_are_resolvable() {
    // 1.21.0/1.21.2
    for id in [
        "minecraft:crafter",
        "minecraft:trial_spawner",
        "minecraft:vault",
        "minecraft:pale_oak_planks",
        "minecraft:pale_oak_log",
        "minecraft:creaking_heart",
        "minecraft:resin_block",
        // 1.21.5/1.21.6
        "minecraft:bush",
        "minecraft:firefly_bush",
        "minecraft:cactus_flower",
        "minecraft:dried_ghast",
        // 1.21.9 (Copper Age)
        "minecraft:copper_chest",
        "minecraft:copper_chain",
        "minecraft:copper_golem_statue",
        "minecraft:waxed_oxidized_copper_bars",
    ] {
        let block = get_block(id);
        assert!(block.is_some(), "expected {id} to exist in block data");
        assert_eq!(block.unwrap().id(), id);
    }
}

#[test]
fn new_blocks_have_texture_derived_colors() {
    for id in [
        "minecraft:crafter",
        "minecraft:pale_oak_planks",
        "minecraft:creaking_heart",
        "minecraft:resin_block",
        "minecraft:firefly_bush",
        "minecraft:dried_ghast",
        "minecraft:copper_chest",
        "minecraft:copper_bulb",
        "minecraft:polished_tuff",
        "minecraft:chiseled_copper",
    ] {
        let block = get_block(id).unwrap_or_else(|| panic!("{id} missing"));
        assert!(
            block.extras.color.is_some(),
            "expected {id} to have a color from the texture pipeline"
        );
    }

    // Pale oak planks are distinctly pale
    let pale = get_block("minecraft:pale_oak_planks").unwrap();
    let rgb = pale.extras.color.unwrap().rgb;
    assert!(rgb.iter().all(|&c| c > 180), "pale_oak_planks rgb: {rgb:?}");
}

#[test]
fn color_coverage_is_nearly_complete() {
    let total = all_blocks().count();
    let with_color = all_blocks().filter(|b| b.extras.color.is_some()).count();

    assert!(total >= 1166, "expected 1.21.11 block count, got {total}");
    // Every block with a block texture has a color; only air variants,
    // barrier, structure_void and mob heads/skulls are excluded.
    assert!(
        with_color * 100 >= total * 95,
        "color coverage regressed: {with_color}/{total}"
    );

    // Tinted blocks pick up the plains-biome constants (i.e. they are green,
    // not the grayscale texture average)
    let grass = get_block("minecraft:grass_block").unwrap();
    let [r, g, b] = grass.extras.color.unwrap().rgb;
    assert!(g > r && g > b, "grass_block should be green: {:?}", [r, g, b]);

    let leaves = get_block("minecraft:oak_leaves").unwrap();
    let [r, g, b] = leaves.extras.color.unwrap().rgb;
    assert!(g > r && g > b, "oak_leaves should be green: {:?}", [r, g, b]);

    let water = get_block("minecraft:water").unwrap();
    let [r, g, b] = water.extras.color.unwrap().rgb;
    assert!(b > r && b > g, "water should be blue: {:?}", [r, g, b]);
}

#[test]
fn gradient_filter_is_applied() {
    let start = get_block("minecraft:white_wool").unwrap();
    let end = get_block("minecraft:black_wool").unwrap();

    // Unfiltered: the first step resolves to an exact color match for the
    // start block (possibly a same-colored block like white_bed)
    let unfiltered = BlockPaletteGenerator::generate_block_gradient(start, end, 7)
        .expect("gradient should generate");
    assert_eq!(
        unfiltered.blocks.first().map(|rec| rec.color.rgb),
        Some(start.extras.color.unwrap().rgb),
        "unfiltered gradient should start at the start block's color"
    );

    // Filtered: excluding wool must remove it from every step
    let no_wool = BlockFilter {
        exclude_patterns: vec!["wool".to_string()],
        ..Default::default()
    };
    let filtered = BlockPaletteGenerator::generate_block_gradient_filtered(start, end, 7, &no_wool)
        .expect("filtered gradient should generate");
    assert!(!filtered.blocks.is_empty());
    for rec in &filtered.blocks {
        assert!(
            !rec.block.id().contains("wool"),
            "filter was ignored: {} in gradient",
            rec.block.id()
        );
    }

    // A restrictive include filter only yields matching blocks
    let only_concrete = BlockFilter {
        include_patterns: vec!["_concrete".to_string()],
        ..Default::default()
    };
    let concrete_only =
        BlockPaletteGenerator::generate_block_gradient_filtered(start, end, 5, &only_concrete)
            .expect("concrete gradient should generate");
    assert!(!concrete_only.blocks.is_empty());
    for rec in &concrete_only.blocks {
        assert!(
            rec.block.id().contains("_concrete"),
            "include filter was ignored: {}",
            rec.block.id()
        );
    }
}
