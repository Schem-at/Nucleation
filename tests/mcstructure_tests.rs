use nucleation::formats::mcstructure::{from_mcstructure, to_mcstructure};
use nucleation::{BlockState, Region, UniversalSchematic};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_mcstructure_round_trip() {
    let mut schematic = UniversalSchematic::new("Test Structure".to_string());
    let mut region = Region::new("Main".to_string(), (0, 0, 0), (3, 3, 3));

    let stone = BlockState::new("minecraft:stone".to_string());
    let dirt = BlockState::new("minecraft:dirt".to_string());

    // Set blocks
    region.set_block(0, 0, 0, &stone.clone());
    region.set_block(1, 1, 1, &dirt.clone());
    region.set_block(2, 2, 2, &stone.clone());

    schematic.add_region(region);

    // Export
    let data = to_mcstructure(&schematic).expect("Failed to export mcstructure");

    // Import
    let loaded = from_mcstructure(&data).expect("Failed to import mcstructure");

    assert_eq!(loaded.total_blocks(), 3);

    let loaded_region = loaded.get_merged_region(); // McStructure import flattens to one region currently

    assert_eq!(
        loaded_region.get_block(0, 0, 0).unwrap().name,
        "minecraft:stone"
    );
    assert_eq!(
        loaded_region.get_block(1, 1, 1).unwrap().name,
        "minecraft:dirt"
    );
    assert_eq!(
        loaded_region.get_block(2, 2, 2).unwrap().name,
        "minecraft:stone"
    );

    // Check dimensions
    // Note: mcstructure export uses tight bounds via to_compact()
    // Original region was 3x3x3. Blocks at (0,0,0) and (2,2,2) -> tight bounds min (0,0,0) max (2,2,2) -> size (3,3,3).
    let (w, h, l) = loaded_region.get_dimensions();
    assert_eq!(w, 3);
    assert_eq!(h, 3);
    assert_eq!(l, 3);
}

#[test]
fn test_mcstructure_properties() {
    let mut schematic = UniversalSchematic::new("Prop Test".to_string());
    let mut region = Region::new("Main".to_string(), (0, 0, 0), (1, 1, 1));

    let mut props = HashMap::new();
    props.insert("facing".to_string(), "north".to_string());
    props.insert("powered".to_string(), "true".to_string()); // Should become Byte(1)
    props.insert("layers".to_string(), "8".to_string()); // Should become Int(8)

    let block = BlockState {
        name: "minecraft:snow".to_string(),
        properties: props,
    };

    region.set_block(0, 0, 0, &block);
    schematic.add_region(region);

    let data = to_mcstructure(&schematic).expect("Failed to export");
    let loaded = from_mcstructure(&data).expect("Failed to import");

    let loaded_block = loaded.get_block(0, 0, 0).unwrap();
    assert_eq!(loaded_block.name, "minecraft:snow");
    assert_eq!(loaded_block.properties.get("facing").unwrap(), "north");
    assert_eq!(loaded_block.properties.get("powered").unwrap(), "true");
    assert_eq!(loaded_block.properties.get("layers").unwrap(), "8");
}

#[test]
fn test_sample_files() {
    let samples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/samples");
    let files = vec![
        "8-bit_alu.mcstructure",
        "16-bit_divider.mcstructure", // Note: Filename capitalization might vary, check LS output
    ];

    for filename in files {
        // Handle potential case sensitivity or exact name from LS
        // "16-Bit Divider.mcstructure" vs "16-bit_divider.mcstructure"
        // The user said "16-Bit Divider.mcstructure" but LS showed "16-bit_divider.mcstructure" ?
        // LS result:
        // - 16-bit_divider.mcstructure
        // - 8-bit_alu.mcstructure
        // So I use lower case ones from LS.

        let path = samples_dir.join(filename);
        if !path.exists() {
            println!("Skipping missing sample file: {:?}", path);
            continue;
        }

        println!("Testing sample: {:?}", filename);
        let data = std::fs::read(&path).expect("Failed to read sample file");

        // Test Import
        let schematic = from_mcstructure(&data).expect("Failed to parse mcstructure");

        println!("  Total blocks: {}", schematic.total_blocks());
        println!("  Total volume: {}", schematic.total_volume());

        let region = schematic.get_merged_region();
        let (w, h, l) = region.get_dimensions();
        println!("  Dimensions: {}x{}x{}", w, h, l);

        assert!(
            schematic.total_blocks() > 0,
            "Schematic should not be empty"
        );

        // Test Round Trip
        let exported = to_mcstructure(&schematic).expect("Failed to export mcstructure");
        let reimported =
            from_mcstructure(&exported).expect("Failed to re-import exported mcstructure");

        assert_eq!(
            schematic.total_blocks(),
            reimported.total_blocks(),
            "Block count mismatch after round trip"
        );

        // Check tile entities
        let tile_count = region.block_entities.len();
        println!("  Block Entities: {}", tile_count);
        let re_tile_count = reimported.get_merged_region().block_entities.len();
        assert_eq!(tile_count, re_tile_count, "Block entity count mismatch");
    }
}
