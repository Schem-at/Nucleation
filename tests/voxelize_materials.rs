#![cfg(feature = "voxelize")]
use nucleation::{
    blockpedia::get_block,
    building::BlockPalette,
    voxelize::{MeshModel, VoxelLight, VoxelizeOptions},
};
#[test]
fn glb_default_scene_skin_alpha_transmission_and_emission_survive_export() {
    let model =
        MeshModel::from_glb_bytes(include_bytes!("fixtures/voxelize-materials.glb")).unwrap();
    assert_eq!(
        model.triangles.len(),
        18,
        "alternative scenes are not imported"
    );
    assert_eq!(
        model.aabb().unwrap(),
        ([1.0, 1.0, 1.0], [47.0, 5.0, 1.25]),
        "bind matrices and default joint transforms position the mesh"
    );
    let options = VoxelizeOptions {
        target_size: 46.0,
        lighting: Some(VoxelLight {
            direction: [0.0, 0.0, -1.0],
            strength: 1.0,
        }),
        ..Default::default()
    };
    let out = model
        .voxelize_with_options(&options, &BlockPalette::new_materials(), "materials")
        .unwrap();
    let id = |x| {
        out.get_block(x, 2, 0)
            .map(|b| b.name.as_str())
            .unwrap_or("minecraft:air")
    };
    assert!(
        !get_block(id(2)).unwrap().transparent,
        "OPAQUE texture with zero alpha stays solid"
    );
    assert_eq!(id(8), "minecraft:air", "MASK hole");
    assert_eq!(id(14), "minecraft:air", "invisible BLEND");
    assert_eq!(
        id(20),
        "minecraft:glass",
        "BLEND white lens remains neutral glass under shading"
    );
    assert_eq!(
        id(26),
        "minecraft:glass",
        "transmission works with OPAQUE alphaMode"
    );
    let emission = get_block(id(32)).unwrap().extras.color.unwrap().rgb;
    assert!(
        emission[2] > emission[0] + 40,
        "emissive blue eye survives total shade: {emission:?}"
    );
    assert_ne!(
        id(44),
        "minecraft:air",
        "masked foreground cannot hide the backing plane"
    );
    let error = model
        .voxelize_with_options(&options, &BlockPalette::new_wool(), "restricted")
        .err()
        .unwrap();
    assert!(error.contains("no glass blocks"), "{error}");
}
