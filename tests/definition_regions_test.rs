use nucleation::block_entity::BlockEntity;
use nucleation::definition_region::DefinitionRegion;
use nucleation::utils::NbtValue;
use nucleation::{BlockState, UniversalSchematic};
use std::collections::HashMap;

#[test]
fn test_definition_regions_roundtrip_chain() {
    // 1. Forge a Universal Schematic
    let mut original = UniversalSchematic::new("Roundtrip Test".to_string());

    // Add some blocks
    original.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));
    original.set_block(1, 1, 1, BlockState::new("minecraft:dirt".to_string()));

    // Add Definition Regions
    let mut def_region_a = DefinitionRegion::new();
    def_region_a.add_bounds((0, 0, 0), (5, 5, 5));
    def_region_a.set_metadata("type", "input");
    def_region_a.set_metadata("signal", "redstone");

    let mut def_region_b = DefinitionRegion::new();
    def_region_b.add_bounds((10, 10, 10), (15, 15, 15));
    def_region_b.set_metadata("type", "output");

    original
        .definition_regions
        .insert("InputA".to_string(), def_region_a);
    original
        .definition_regions
        .insert("OutputB".to_string(), def_region_b);

    // 2. Roundtrip to .schem (V3)
    let schem_data = original
        .to_schematic()
        .expect("Failed to serialize to .schem");
    let from_schem =
        UniversalSchematic::from_schematic(&schem_data).expect("Failed to deserialize from .schem");

    // Verify consistency
    verify_consistency(&original, &from_schem, "Original -> Schem");

    // 3. Roundtrip from the loaded schematic to .litematic
    let litematic_data = nucleation::litematic::to_litematic(&from_schem)
        .expect("Failed to serialize to .litematic");
    let from_litematic = nucleation::litematic::from_litematic(&litematic_data)
        .expect("Failed to deserialize from .litematic");

    // Verify consistency
    verify_consistency(&from_schem, &from_litematic, "Schem -> Litematic");

    // 4. Verify original vs final
    verify_consistency(&original, &from_litematic, "Original -> Final");
}

#[test]
fn test_schematic_v2_definition_regions() {
    let mut original = UniversalSchematic::new("V2 Test".to_string());
    original.set_block(0, 0, 0, BlockState::new("minecraft:stone".to_string()));

    let mut def_region = DefinitionRegion::new();
    def_region.add_bounds((0, 0, 0), (1, 1, 1));
    def_region.set_metadata("test", "v2");
    original
        .definition_regions
        .insert("TestV2".to_string(), def_region);

    // Explicitly use V2
    let schem_data = nucleation::schematic::to_schematic_version(
        &original,
        nucleation::schematic::SchematicVersion::V2,
    )
    .expect("Failed to serialize to .schem v2");

    let from_schem = UniversalSchematic::from_schematic(&schem_data)
        .expect("Failed to deserialize from .schem v2");

    verify_consistency(&original, &from_schem, "Original -> Schem V2");
}

#[test]
fn test_insign_integration() {
    let mut schematic = UniversalSchematic::new("Insign Test".to_string());

    // Create a sign that defines a region via Insign syntax
    // @region_name=rc([min],[max])
    // #region_name:key=value
    let mut sign = BlockEntity::new("minecraft:sign".to_string(), (0, 0, 0));
    sign.nbt.insert(
        "Text1".to_string(),
        NbtValue::String(r##"{"text":"@my_region"}"##.to_string()),
    );
    sign.nbt.insert(
        "Text2".to_string(),
        NbtValue::String(r##"{"text":"=rc([0,0,0],"}"##.to_string()),
    );
    sign.nbt.insert(
        "Text3".to_string(),
        NbtValue::String(r##"{"text":"[5,5,5])"}"##.to_string()),
    );
    sign.nbt.insert(
        "Text4".to_string(),
        NbtValue::String(r##"{"text":"#my_region:k=\"v\""}"##.to_string()),
    );

    schematic.add_block_entity(sign);

    // Run import
    schematic
        .import_insign_regions()
        .expect("Failed to import insign regions");

    // Verify
    assert!(schematic.definition_regions.contains_key("my_region"));
    let region = schematic.definition_regions.get("my_region").unwrap();

    // Check bounds (should be 0,0,0 to 5,5,5)
    let bounds = region.get_bounds().unwrap();
    assert_eq!(bounds.min, (0, 0, 0));
    assert_eq!(bounds.max, (5, 5, 5));

    // Check metadata
    assert_eq!(region.get_metadata("k"), Some(&"v".to_string()));
}

fn verify_consistency(expected: &UniversalSchematic, actual: &UniversalSchematic, stage: &str) {
    assert_eq!(
        expected.definition_regions.len(),
        actual.definition_regions.len(),
        "{}: Region count mismatch",
        stage
    );

    for (name, exp_region) in &expected.definition_regions {
        assert!(
            actual.definition_regions.contains_key(name),
            "{}: Missing region '{}'",
            stage,
            name
        );
        let act_region = actual.definition_regions.get(name).unwrap();

        // Compare metadata
        assert_eq!(
            exp_region.metadata, act_region.metadata,
            "{}: Metadata mismatch for region '{}'",
            stage, name
        );

        // Compare boxes count
        assert_eq!(
            exp_region.boxes.len(),
            act_region.boxes.len(),
            "{}: Box count mismatch for region '{}'",
            stage,
            name
        );

        // Compare total volume as a proxy for box geometry (exact box order might vary depending on impl details, though here it should be exact)
        assert_eq!(
            exp_region.volume(),
            act_region.volume(),
            "{}: Volume mismatch for region '{}'",
            stage,
            name
        );

        // Compare exact boxes (assuming order is preserved by serialization which it should be for Vec)
        for (i, (exp_box, act_box)) in exp_region
            .boxes
            .iter()
            .zip(act_region.boxes.iter())
            .enumerate()
        {
            assert_eq!(
                exp_box.min, act_box.min,
                "{}: Box {} min mismatch for region '{}'",
                stage, i, name
            );
            assert_eq!(
                exp_box.max, act_box.max,
                "{}: Box {} max mismatch for region '{}'",
                stage, i, name
            );
        }
    }
}
