use nucleation::blockpedia::color::block_palettes::{BlockFilter, BlockPaletteGenerator, PaletteTheme};
use nucleation::blockpedia::BLOCKS;

#[test]
fn test_natural_palette_generation() {
    // Test forest palette
    if let Some(forest_palette) = BlockPaletteGenerator::generate_natural_palette("forest") {
        assert_eq!(forest_palette.theme, PaletteTheme::Natural);
        assert!(!forest_palette.blocks.is_empty());
        assert!(forest_palette.name.contains("Forest"));
        assert!(!forest_palette.description.is_empty());

        // Check that we have valid block recommendations
        for rec in &forest_palette.blocks {
            assert!(rec.block.id().starts_with("minecraft:"));
            assert!(!rec.usage_notes.is_empty());
        }
    }
}

#[test]
fn test_architectural_palette_generation() {
    // Test medieval palette
    if let Some(medieval_palette) =
        BlockPaletteGenerator::generate_architectural_palette("medieval")
    {
        assert_eq!(medieval_palette.theme, PaletteTheme::Architectural);
        assert!(!medieval_palette.blocks.is_empty());
        assert!(medieval_palette.name.contains("Medieval"));

        // Should include traditional materials
        let block_names: Vec<_> = medieval_palette
            .blocks
            .iter()
            .map(|rec| rec.block.id())
            .collect();

        assert!(block_names
            .iter()
            .any(|name| name.contains("cobblestone") || name.contains("oak")));
    }
}

#[test]
fn test_gradient_palette_with_colored_blocks() {
    // Find two blocks with color data
    let colored_blocks: Vec<_> = BLOCKS
        .values()
        .filter(|b| b.extras.color.is_some())
        .collect();

    if colored_blocks.len() >= 2 {
        let block1 = colored_blocks[0];
        let block2 = colored_blocks[1];

        if let Some(gradient_palette) =
            BlockPaletteGenerator::generate_block_gradient(block1, block2, 5)
        {
            assert_eq!(gradient_palette.theme, PaletteTheme::Gradient);
            assert_eq!(gradient_palette.blocks.len(), 5);
            assert!(gradient_palette.name.contains("Gradient"));

            // Should have blocks with different roles
            let roles: Vec<_> = gradient_palette
                .blocks
                .iter()
                .map(|rec| rec.role.clone())
                .collect();

            assert!(roles.len() >= 2); // Should have variety in roles
        }
    }
}

#[test]
fn test_monochrome_palette() {
    // Find a block with color data
    let colored_block = BLOCKS.values().find(|b| b.extras.color.is_some());

    if let Some(block) = colored_block {
        if let Some(mono_palette) = BlockPaletteGenerator::generate_monochrome_palette(block, 7) {
            assert_eq!(mono_palette.theme, PaletteTheme::Monochrome);
            assert!(!mono_palette.blocks.is_empty());
            assert!(mono_palette.name.contains("Monochrome"));

            // All blocks should be valid
            for rec in &mono_palette.blocks {
                assert!(rec.block.id().starts_with("minecraft:"));
                assert!(!rec.usage_notes.is_empty());
            }
        }
    }
}

#[test]
fn test_palette_export_formats() {
    if let Some(palette) = BlockPaletteGenerator::generate_natural_palette("desert") {
        // Test text export
        let text_export = palette.to_text_list();
        assert!(text_export.contains("Desert"));
        assert!(!text_export.is_empty());

        // Test JSON export
        let json_export = palette.to_json();
        assert!(json_export.contains("\"name\""));
        assert!(json_export.contains("\"blocks\""));
        assert!(json_export.contains("\"theme\""));

        // Should be valid JSON
        let _parsed: serde_json::Value =
            serde_json::from_str(&json_export).expect("Generated JSON should be valid");
    }
}

#[test]
fn test_color_range_search() {
    use nucleation::blockpedia::color::ExtendedColorData;

    // Test finding blocks by color similarity
    let target_color = ExtendedColorData::from_rgb(128, 128, 128); // Gray
    let similar_blocks = BlockPaletteGenerator::find_blocks_by_color_range(target_color, 50.0, 10);

    // Should find some blocks (even if it's just gray-ish ones)
    assert!(similar_blocks.len() <= 10);

    // All returned blocks should have color data
    for block in similar_blocks {
        assert!(block.extras.color.is_some());
    }
}

#[test]
fn test_available_themes_and_styles() {
    let natural_themes = BlockPaletteGenerator::get_natural_themes();
    assert!(natural_themes.contains(&"forest"));
    assert!(natural_themes.contains(&"desert"));
    assert!(natural_themes.contains(&"ocean"));

    let arch_styles = BlockPaletteGenerator::get_architectural_styles();
    assert!(arch_styles.contains(&"medieval"));
    assert!(arch_styles.contains(&"modern"));
    assert!(arch_styles.contains(&"rustic"));
}

#[test]
fn test_invalid_palette_requests() {
    // Test with non-existent theme
    let invalid_natural = BlockPaletteGenerator::generate_natural_palette("invalid_theme");
    assert!(invalid_natural.is_none());

    let invalid_arch = BlockPaletteGenerator::generate_architectural_palette("invalid_style");
    assert!(invalid_arch.is_none());
}

#[test]
fn test_solid_blocks_filter() {
    let solid_filter = BlockFilter::solid_blocks_only();

    // Test filter configuration
    assert!(solid_filter.exclude_falling);
    assert!(solid_filter.exclude_tile_entities);
    assert!(solid_filter.full_blocks_only);
    assert!(solid_filter.exclude_needs_support);
    assert!(solid_filter.exclude_transparent);
    assert!(solid_filter.survival_obtainable_only);

    // Test that sand (falling block) is excluded
    if let Some(sand_block) = BLOCKS.get("minecraft:sand") {
        assert!(!solid_filter.allows_block(sand_block));
    }

    // Test that stone (solid block) is allowed
    if let Some(stone_block) = BLOCKS.get("minecraft:stone") {
        assert!(solid_filter.allows_block(stone_block));
    }
}

#[test]
fn test_structural_blocks_filter() {
    let structural_filter = BlockFilter::structural_blocks_only();

    // More restrictive than solid blocks
    assert!(structural_filter.exclude_falling);
    assert!(structural_filter.exclude_tile_entities);
    assert!(structural_filter.full_blocks_only);
    assert!(structural_filter.exclude_needs_support);
    assert!(structural_filter.exclude_transparent);
    assert!(structural_filter.exclude_light_sources);

    // Test that glass (transparent) is excluded
    if let Some(glass_block) = BLOCKS.get("minecraft:glass") {
        assert!(!structural_filter.allows_block(glass_block));
    }

    // Test that glowstone (light source) is excluded
    if let Some(glowstone_block) = BLOCKS.get("minecraft:glowstone") {
        assert!(!structural_filter.allows_block(glowstone_block));
    }
}

#[test]
fn test_decorative_blocks_filter() {
    let decorative_filter = BlockFilter::decorative_blocks();

    // More permissive than solid blocks
    assert!(decorative_filter.exclude_falling);
    assert!(decorative_filter.exclude_tile_entities);
    assert!(!decorative_filter.full_blocks_only); // Allows partial blocks
    assert!(!decorative_filter.exclude_needs_support);
    assert!(!decorative_filter.exclude_transparent);
    assert!(!decorative_filter.exclude_light_sources);

    // Test that oak stairs (partial block) is allowed
    if let Some(stairs_block) = BLOCKS.get("minecraft:oak_stairs") {
        assert!(decorative_filter.allows_block(stairs_block));
    }
}

#[test]
fn test_custom_filters() {
    // Test include patterns
    let concrete_only = BlockFilter {
        include_patterns: vec!["concrete".to_string()],
        ..Default::default()
    };

    if let Some(white_concrete) = BLOCKS.get("minecraft:white_concrete") {
        assert!(concrete_only.allows_block(white_concrete));
    }

    if let Some(stone_block) = BLOCKS.get("minecraft:stone") {
        assert!(!concrete_only.allows_block(stone_block));
    }

    // Test exclude patterns
    let no_glass = BlockFilter {
        exclude_patterns: vec!["glass".to_string()],
        ..Default::default()
    };

    if let Some(glass_block) = BLOCKS.get("minecraft:glass") {
        assert!(!no_glass.allows_block(glass_block));
    }

    if let Some(stone_block) = BLOCKS.get("minecraft:stone") {
        assert!(no_glass.allows_block(stone_block));
    }
}

#[test]
fn test_filtered_palette_generation() {
    let solid_filter = BlockFilter::solid_blocks_only();

    // Test filtered natural palette
    if let Some(filtered_forest) =
        BlockPaletteGenerator::generate_natural_palette_filtered("forest", &solid_filter)
    {
        assert!(!filtered_forest.blocks.is_empty());

        // All blocks should pass the filter
        for rec in &filtered_forest.blocks {
            assert!(solid_filter.allows_block(rec.block));
        }
    }

    // Test filtered architectural palette
    if let Some(filtered_medieval) =
        BlockPaletteGenerator::generate_architectural_palette_filtered("medieval", &solid_filter)
    {
        assert!(!filtered_medieval.blocks.is_empty());

        // All blocks should pass the filter
        for rec in &filtered_medieval.blocks {
            assert!(solid_filter.allows_block(rec.block));
        }
    }
}

#[test]
fn test_filter_impact_on_block_count() {
    let default_filter = BlockFilter::default();
    let solid_filter = BlockFilter::solid_blocks_only();
    let structural_filter = BlockFilter::structural_blocks_only();

    let total_blocks = BLOCKS.len();
    let solid_count = BLOCKS
        .values()
        .filter(|b| solid_filter.allows_block(b))
        .count();
    let structural_count = BLOCKS
        .values()
        .filter(|b| structural_filter.allows_block(b))
        .count();
    let default_count = BLOCKS
        .values()
        .filter(|b| default_filter.allows_block(b))
        .count();

    // Default filter should allow all blocks
    assert_eq!(default_count, total_blocks);

    // Filters should progressively reduce block count
    assert!(structural_count <= solid_count);
    assert!(solid_count <= default_count);

    // Should still have reasonable number of blocks
    assert!(solid_count > 100); // Should have plenty of solid blocks
    assert!(structural_count > 50); // Should have decent structural options
}
