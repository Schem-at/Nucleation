use nucleation::{
    blockpedia::{get_block, ExtendedColorData},
    building::BlockPalette,
};
#[test]
fn opaque_and_glass_matching_and_dithering_never_cross_materials() {
    let palette = BlockPalette::from_block_ids([
        "minecraft:white_concrete",
        "minecraft:black_concrete",
        "minecraft:white_stained_glass",
        "minecraft:glass",
        "minecraft:oak_leaves",
        "minecraft:copper_grate",
    ]);
    let opaque = palette.for_material(false);
    let glass = palette.for_material(true);
    assert_eq!(opaque.len(), 2);
    assert_eq!(glass.len(), 2);
    for (p, translucent) in [(&opaque, false), (&glass, true)] {
        for v in 0..=255 {
            let color = ExtendedColorData::from_rgb(v, v, v);
            let id = p.find_closest(&color).unwrap();
            assert_eq!(get_block(&id).unwrap().is_glass(), translucent);
            for x in 0..4 {
                let id = p.find_closest_dithered(&color, x, 0, 0).unwrap();
                assert_eq!(get_block(&id).unwrap().is_glass(), translucent);
            }
        }
    }
    assert_eq!(
        glass
            .find_closest(&ExtendedColorData::from_rgb(255, 255, 255))
            .unwrap(),
        "minecraft:glass"
    );
    assert!(BlockPalette::new_wool().for_material(true).is_empty());
    assert!(BlockPalette::from_block_ids(["minecraft:glass"])
        .for_material(false)
        .is_empty());
    assert!(BlockPalette::new_materials().for_material(true).len() >= 17);
}
