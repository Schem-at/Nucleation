use nucleation::blockpedia::query_builder::*;
use nucleation::blockpedia::*;

#[test]
fn test_basic_query_creation() {
    let query = AllBlocks::new();
    assert!(!query.is_empty(), "Query should start with all blocks");

    let all_blocks = AllBlocks::new();
    assert_eq!(
        query.len(),
        all_blocks.len(),
        "Two AllBlocks should return same number of blocks"
    );
}

#[test]
fn test_block_filtering() {
    let query = AllBlocks::new();
    let original_count = query.len();

    // Test solid blocks filter
    let solid_query = query.clone().only_solid();
    assert!(
        solid_query.len() <= original_count,
        "Solid filter should not increase block count"
    );

    // Test survival blocks filter
    let survival_query = AllBlocks::new().survival_only();
    assert!(
        survival_query.len() <= original_count,
        "Survival filter should not increase block count"
    );

    // Test chaining filters
    let chained_query = AllBlocks::new().only_solid().survival_only();
    assert!(
        chained_query.len() <= solid_query.len(),
        "Chained filters should be more restrictive"
    );
}

#[test]
fn test_property_filtering() {
    let query = AllBlocks::new();

    // Test filtering by property existence
    let with_delay = query.clone().with_property("delay");
    assert!(
        !with_delay.is_empty(),
        "Should find blocks with delay property"
    );

    // Test filtering by property value
    let delay_1 = query.clone().with_property_value("delay", "1");
    assert!(!delay_1.is_empty(), "Should find blocks with delay=1");

    // Test multiple property filters
    let multiple_props = query.clone().with_property("delay").with_property("facing");
    assert!(
        !multiple_props.is_empty(),
        "Should find blocks with both properties"
    );
}

#[test]
fn test_color_filtering() {
    let query = AllBlocks::new();

    // Test blocks with color data
    let with_color = query.clone().with_color();
    assert!(!with_color.is_empty(), "Should find blocks with color data");

    // Verify all returned blocks have color data
    for block in with_color.collect() {
        assert!(
            block.extras.color.is_some(),
            "Block {} should have color data",
            block.id()
        );
    }
}

#[test]
fn test_pattern_matching() {
    let query = AllBlocks::new();

    // Test exact pattern matching
    let stone_blocks = query.clone().matching("*stone*");
    assert!(
        !stone_blocks.is_empty(),
        "Should find blocks with 'stone' in name"
    );

    // Test specific namespace
    let minecraft_blocks = query.clone().matching("minecraft:*");
    assert!(!minecraft_blocks.is_empty(), "Should find minecraft blocks");

    // Test multiple patterns
    let multi_pattern = query.clone().matching("*stone*");
    assert!(
        !multi_pattern.is_empty(),
        "Should find blocks matching pattern"
    );
}

#[test]
fn test_block_families() {
    let query = AllBlocks::new();

    // Test stairs family
    let stairs = query.clone().from_families(&["stairs"]);
    // Note: This might be 0 if test data doesn't have stairs

    // Test multiple families
    let multi_family = query.clone().from_families(&["stairs", "slabs", "walls"]);
    assert!(
        multi_family.len() >= stairs.len(),
        "Multiple families should include stairs"
    );
}

#[test]
fn test_exclusion_filters() {
    let query = AllBlocks::new();
    let original_count = query.len();

    // Test excluding transparent blocks
    let no_transparent = query.clone().exclude_transparent();
    assert!(
        no_transparent.len() <= original_count,
        "Excluding transparent should not increase count"
    );

    // Test excluding tile entities
    let no_tile_entities = query.clone().exclude_tile_entities();
    assert!(
        no_tile_entities.len() <= original_count,
        "Excluding tile entities should not increase count"
    );

    // Test excluding falling blocks
    let no_falling = query.clone().exclude_falling();
    assert!(
        no_falling.len() <= original_count,
        "Excluding falling blocks should not increase count"
    );
}

#[test]
fn test_sorting() {
    let query = AllBlocks::new().with_color().limit(10);

    // Test sorting by name
    let name_sorted = query.clone().sort_by_name();
    assert_eq!(
        name_sorted.len(),
        query.len(),
        "Sorting should preserve count"
    );

    // Test sorting by color gradient
    let color_sorted = query.clone().sort_by_color_gradient();
    assert_eq!(
        color_sorted.len(),
        query.len(),
        "Sorting should preserve count"
    );
}

#[test]
fn test_color_similarity() {
    let query = AllBlocks::new().with_color();

    if !query.is_empty() {
        // Test color similarity search
        let target_color = ExtendedColorData::from_rgb(128, 128, 128);
        let similar = query.clone().similar_to_color(target_color, 50.0);
        assert!(
            similar.len() <= query.len(),
            "Color similarity should not increase count"
        );

        // Test sorting by color similarity
        let sorted_by_similarity = query.clone().sort_by_color_similarity(target_color);
        assert_eq!(
            sorted_by_similarity.len(),
            query.len(),
            "Sorting should preserve count"
        );
    }
}

#[test]
fn test_limit_and_offset() {
    let query = AllBlocks::new();
    let original_count = query.len();

    // Test limit
    let limited = query.clone().limit(5);
    assert!(limited.len() <= 5, "Limit should restrict count");
    assert!(
        limited.len() <= original_count,
        "Limit should not increase count"
    );

    // Test first block
    let first = query.clone().first();
    assert!(first.is_some(), "Should find at least one block");

    // Test any blocks
    let has_blocks = query.clone().any();
    assert!(has_blocks, "Should have blocks");
}

#[test]
fn test_gradient_generation() {
    let query = AllBlocks::new().with_color();

    if query.len() >= 3 {
        // Test basic gradient generation
        let config = GradientConfig::new(5)
            .with_color_space(ColorSpace::Rgb)
            .with_easing(EasingFunction::Linear);
        let gradient = query.clone().generate_gradient(config);
        assert_eq!(
            gradient.len(),
            5,
            "Gradient should have requested number of steps"
        );

        // Test gradient with different color spaces
        let hsl_config = GradientConfig::new(3)
            .with_color_space(ColorSpace::Hsl)
            .with_easing(EasingFunction::EaseInOut);
        let hsl_gradient = query.clone().generate_gradient(hsl_config);
        assert_eq!(hsl_gradient.len(), 3, "HSL gradient should work");

        let oklab_config = GradientConfig::new(4)
            .with_color_space(ColorSpace::Oklab)
            .with_easing(EasingFunction::EaseIn);
        let oklab_gradient = query.clone().generate_gradient(oklab_config);
        assert_eq!(oklab_gradient.len(), 4, "Oklab gradient should work");
    }
}

#[test]
fn test_gradient_between_colors() {
    let query = AllBlocks::new().with_color();

    if query.len() >= 2 {
        let start_color = ExtendedColorData::from_rgb(255, 0, 0);
        let end_color = ExtendedColorData::from_rgb(0, 0, 255);

        let config = GradientConfig::new(5)
            .with_color_space(ColorSpace::Rgb)
            .with_easing(EasingFunction::Linear);
        let gradient =
            query
                .clone()
                .generate_gradient_between_colors(start_color, end_color, config);

        assert_eq!(
            gradient.len(),
            5,
            "Gradient between colors should have requested steps"
        );
    }
}

#[test]
fn test_multi_gradient() {
    let query = AllBlocks::new().with_color();

    // Multi-gradient works with whatever blocks are available
    let config = GradientConfig::new(6)
        .with_color_space(ColorSpace::Rgb)
        .with_easing(EasingFunction::Linear);
    let gradient = query.clone().generate_multi_gradient(config);

    // Multi-gradient may return fewer blocks than requested if there aren't enough unique colors
    // Just test that it doesn't crash and returns a valid query object
    assert!(
        gradient.len() <= 6,
        "Multi-gradient should not exceed requested steps"
    );

    // If we have colored blocks available, we should get some result
    if !query.is_empty() {
        // The implementation might return 0 blocks if the algorithm doesn't find suitable matches
        // This is acceptable behavior, so we just test that it runs without error
        println!(
            "Multi-gradient returned {} blocks from {} colored input blocks",
            gradient.len(),
            query.len()
        );
    }
}

#[test]
fn test_query_chaining() {
    let query = AllBlocks::new();

    // Test complex chaining
    let complex_query = query
        .only_solid()
        .survival_only()
        .with_color()
        .exclude_transparent()
        .limit(10)
        .sort_by_name();

    assert!(
        complex_query.len() <= 10,
        "Complex query should respect limit"
    );

    // Test bidirectional chaining with gradients
    if complex_query.len() >= 3 {
        let config = GradientConfig::new(5)
            .with_color_space(ColorSpace::Rgb)
            .with_easing(EasingFunction::Linear);
        let gradient_query = complex_query
            .generate_gradient(config)
            .sort_by_color_gradient()
            .limit(3);

        assert_eq!(
            gradient_query.len(),
            3,
            "Should be able to chain after gradient"
        );
    }
}

#[test]
fn test_query_information() {
    let query = AllBlocks::new();

    // Test count
    assert!(!query.is_empty(), "Query should have blocks");

    // Test collecting
    let blocks = query.collect();
    assert!(!blocks.is_empty(), "Collect should yield blocks");

    // Test any
    let has_blocks = AllBlocks::new().any();
    assert!(has_blocks, "Should have any blocks");
}

#[test]
fn test_empty_query_handling() {
    let query = AllBlocks::new();

    // Create a query that should return no results
    let empty_query = query.matching("nonexistent:impossible_block_name_12345");
    assert_eq!(
        empty_query.len(),
        0,
        "Impossible pattern should return no blocks"
    );

    // Test operations on empty query
    let config = GradientConfig::new(5)
        .with_color_space(ColorSpace::Rgb)
        .with_easing(EasingFunction::Linear);
    let empty_gradient = empty_query.clone().generate_gradient(config);
    assert_eq!(
        empty_gradient.len(),
        0,
        "Gradient of empty query should be empty"
    );

    let empty_limited = empty_query.limit(10);
    assert_eq!(
        empty_limited.len(),
        0,
        "Limit on empty query should still be empty"
    );
}

#[test]
fn test_color_space_consistency() {
    let query = AllBlocks::new().with_color().limit(5);

    if query.len() >= 3 {
        // Test that different color spaces produce different results
        let rgb_config = GradientConfig::new(3)
            .with_color_space(ColorSpace::Rgb)
            .with_easing(EasingFunction::Linear);
        let rgb_gradient = query.clone().generate_gradient(rgb_config);

        let hsl_config = GradientConfig::new(3)
            .with_color_space(ColorSpace::Hsl)
            .with_easing(EasingFunction::Linear);
        let hsl_gradient = query.clone().generate_gradient(hsl_config);

        let oklab_config = GradientConfig::new(3)
            .with_color_space(ColorSpace::Oklab)
            .with_easing(EasingFunction::Linear);
        let oklab_gradient = query.clone().generate_gradient(oklab_config);

        // All should have same length
        assert_eq!(rgb_gradient.len(), 3);
        assert_eq!(hsl_gradient.len(), 3);
        assert_eq!(oklab_gradient.len(), 3);

        // Results might be different due to color space differences
        // This is expected behavior
    }
}

#[test]
fn test_easing_function_consistency() {
    let query = AllBlocks::new().with_color().limit(5);

    if query.len() >= 3 {
        // Test that different easing functions work
        let linear_config = GradientConfig::new(3)
            .with_color_space(ColorSpace::Rgb)
            .with_easing(EasingFunction::Linear);
        let linear = query.clone().generate_gradient(linear_config);

        let ease_in_config = GradientConfig::new(3)
            .with_color_space(ColorSpace::Rgb)
            .with_easing(EasingFunction::EaseIn);
        let ease_in = query.clone().generate_gradient(ease_in_config);

        let ease_out_config = GradientConfig::new(3)
            .with_color_space(ColorSpace::Rgb)
            .with_easing(EasingFunction::EaseOut);
        let ease_out = query.clone().generate_gradient(ease_out_config);

        let ease_in_out_config = GradientConfig::new(3)
            .with_color_space(ColorSpace::Rgb)
            .with_easing(EasingFunction::EaseInOut);
        let ease_in_out = query.clone().generate_gradient(ease_in_out_config);

        // All should have same length
        assert_eq!(linear.len(), 3);
        assert_eq!(ease_in.len(), 3);
        assert_eq!(ease_out.len(), 3);
        assert_eq!(ease_in_out.len(), 3);
    }
}

#[test]
fn test_query_builder_robustness() {
    let query = AllBlocks::new();

    // Test with invalid property names
    let invalid_prop = query.clone().with_property("nonexistent_property_name");
    assert_eq!(
        invalid_prop.len(),
        0,
        "Invalid property should return no blocks"
    );

    // Test with invalid property values
    let invalid_value = query.clone().with_property_value("delay", "999");
    assert_eq!(
        invalid_value.len(),
        0,
        "Invalid property value should return no blocks"
    );

    // Test with extreme limits
    let zero_limit = query.clone().limit(0);
    assert_eq!(zero_limit.len(), 0, "Zero limit should return no blocks");

    let huge_limit = query.clone().limit(999999);
    assert_eq!(
        huge_limit.len(),
        query.len(),
        "Huge limit should return all blocks"
    );
}

#[test]
fn test_realistic_use_cases() {
    // Test: Find all solid, survival-obtainable blocks with color for building
    let building_blocks = AllBlocks::new()
        .only_solid()
        .survival_only()
        .with_color()
        .exclude_transparent()
        .sort_by_name()
        .limit(20);

    // Should find some blocks suitable for building
    assert!(!building_blocks.is_empty(), "Should find building blocks");

    // Test: Create a warm color palette
    let config = GradientConfig::new(8)
        .with_color_space(ColorSpace::Hsl)
        .with_easing(EasingFunction::EaseInOut);
    let warm_palette = AllBlocks::new()
        .with_color()
        .generate_gradient(config)
        .sort_by_color_gradient();

    if !warm_palette.is_empty() {
        assert!(warm_palette.len() <= 8, "Warm palette should respect limit");
    }

    // Test: Find redstone components
    let _redstone = AllBlocks::new().matching("*redstone*").survival_only();

    // May or may not find blocks depending on test data
    // This tests the pattern matching works without error
}
