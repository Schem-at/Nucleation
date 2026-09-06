use nucleation::{BlockState, Region};

#[test]
fn merging_disjoint_regions_preserves_both_blocks_when_compacted() {
    let stone = BlockState::new("minecraft:stone".to_string());
    let log = BlockState::new("minecraft:oak_log".to_string())
        .with_property("axis".to_string(), "x".to_string());
    let mut first = Region::new("first".to_string(), (-2, 0, -1), (1, 1, 1));
    first.set_block(-2, 0, -1, &stone);
    let mut second = Region::new("second".to_string(), (1, 1, 2), (1, 1, 1));
    second.set_block(1, 1, 2, &log);

    first.merge(&second);
    assert_eq!(first.count_blocks(), 2);
    let compact = first.to_compact();

    assert_eq!(compact.get_block(-2, 0, -1), Some(&stone));
    assert_eq!(compact.get_block(1, 1, 2), Some(&log));
    assert_eq!(compact.count_blocks(), 2);
}

#[test]
fn merging_into_an_empty_region_preserves_blocks_when_compacted() {
    let stone = BlockState::new("minecraft:stone".to_string());
    let mut empty = Region::new("empty".to_string(), (0, 0, 0), (1, 1, 1));
    let mut source = Region::new("source".to_string(), (-3, 2, 4), (1, 1, 1));
    source.set_block(-3, 2, 4, &stone);

    empty.merge(&source);
    assert_eq!(empty.count_blocks(), 1);
    let compact = empty.to_compact();

    assert_eq!(compact.get_block(-3, 2, 4), Some(&stone));
    assert_eq!(compact.count_blocks(), 1);
    assert_eq!(compact.get_dimensions(), (1, 1, 1));
}

#[test]
fn merging_negative_sized_regions_preserves_blocks_when_compacted() {
    let stone = BlockState::new("minecraft:stone".to_string());
    let gold = BlockState::new("minecraft:gold_block".to_string());
    let mut first = Region::new("first".to_string(), (0, 0, 0), (1, 1, 1));
    first.set_block(0, 0, 0, &stone);
    let mut second = Region::new("second".to_string(), (-2, -2, -2), (-2, -2, -2));
    second.set_block(-3, -3, -3, &gold);

    first.merge(&second);
    let compact = first.to_compact();

    assert_eq!(compact.get_block(0, 0, 0), Some(&stone));
    assert_eq!(compact.get_block(-3, -3, -3), Some(&gold));
    assert_eq!(compact.count_blocks(), 2);
}

#[test]
fn merging_air_does_not_erase_blocks_or_expand_occupied_bounds() {
    let stone = BlockState::new("minecraft:stone".to_string());
    let mut first = Region::new("first".to_string(), (0, 0, 0), (1, 1, 1));
    first.set_block(0, 0, 0, &stone);
    let air = Region::new("air".to_string(), (-2, -2, -2), (5, 5, 5));

    first.merge(&air);
    let compact = first.to_compact();

    assert_eq!(compact.get_block(0, 0, 0), Some(&stone));
    assert_eq!(compact.count_blocks(), 1);
    assert_eq!(compact.get_dimensions(), (1, 1, 1));
}

#[test]
fn litematica_v7_to_sponge_preserves_separate_regions() {
    let source =
        nucleation::litematic::from_litematic(include_bytes!("fixtures/multi-region-v7.litematic"))
            .unwrap();
    let bytes = nucleation::schematic::to_schematic(&source).unwrap();
    let restored = nucleation::schematic::from_schematic(&bytes).unwrap();

    // Sponge import normalizes the exported offset to zero.
    assert_eq!(restored.get_block(0, 0, 0).unwrap().name, "minecraft:stone");
    assert_eq!(
        restored.get_block(1, 0, 0).unwrap(),
        &BlockState::new("minecraft:oak_log".to_string())
            .with_property("axis".to_string(), "x".to_string())
    );
    assert_eq!(restored.get_block(3, 1, 3).unwrap().name, "minecraft:chest");
    assert_eq!(restored.total_blocks(), 3);
}
