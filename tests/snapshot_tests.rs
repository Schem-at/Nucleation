use nucleation::block_entity::BlockEntity;
use nucleation::formats::manager::get_manager;
use nucleation::formats::snapshot::{from_snapshot, to_snapshot};
use nucleation::utils::NbtValue;
use nucleation::{BlockState, Region, UniversalSchematic};

/// Basic round-trip: serialize then deserialize, verify metadata and blocks match.
#[test]
fn snapshot_roundtrip_basic() {
    let mut schematic = UniversalSchematic::new("RoundTripTest".to_string());
    schematic.metadata.author = Some("tester".to_string());
    schematic.metadata.description = Some("A test schematic".to_string());

    let stone = BlockState::new("minecraft:stone".to_string());
    let dirt = BlockState::new("minecraft:dirt".to_string());
    schematic.set_block(0, 0, 0, &stone);
    schematic.set_block(1, 0, 0, &dirt);
    schematic.set_block(0, 1, 0, &stone);

    let bytes = to_snapshot(&schematic).unwrap();
    let restored = from_snapshot(&bytes).unwrap();

    assert_eq!(
        schematic.metadata.name, restored.metadata.name,
        "metadata name mismatch"
    );
    assert_eq!(
        schematic.metadata.author, restored.metadata.author,
        "metadata author mismatch"
    );
    assert_eq!(
        schematic.metadata.description, restored.metadata.description,
        "metadata description mismatch"
    );

    // Verify blocks
    assert_eq!(
        restored.get_block(0, 0, 0).map(|b| b.name.to_string()),
        Some("minecraft:stone".to_string())
    );
    assert_eq!(
        restored.get_block(1, 0, 0).map(|b| b.name.to_string()),
        Some("minecraft:dirt".to_string())
    );
    assert_eq!(
        restored.get_block(0, 1, 0).map(|b| b.name.to_string()),
        Some("minecraft:stone".to_string())
    );
}

/// Verify block properties (e.g. facing, powered) survive the round-trip.
#[test]
fn snapshot_roundtrip_block_properties() {
    let mut schematic = UniversalSchematic::new("PropertiesTest".to_string());
    let lever = BlockState::new("minecraft:lever".to_string())
        .with_property("face".to_string(), "wall".to_string())
        .with_property("facing".to_string(), "north".to_string())
        .with_property("powered".to_string(), "true".to_string());
    schematic.set_block(5, 10, 15, &lever);

    let bytes = to_snapshot(&schematic).unwrap();
    let restored = from_snapshot(&bytes).unwrap();

    let block = restored.get_block(5, 10, 15).expect("block not found");
    assert_eq!(block.name.as_str(), "minecraft:lever");
    assert_eq!(block.get_property("face").map(|s| s.as_str()), Some("wall"));
    assert_eq!(
        block.get_property("facing").map(|s| s.as_str()),
        Some("north")
    );
    assert_eq!(
        block.get_property("powered").map(|s| s.as_str()),
        Some("true")
    );
}

/// Verify block entities (tile entities) survive the round-trip.
#[test]
fn snapshot_roundtrip_block_entities() {
    let mut schematic = UniversalSchematic::new("BlockEntityTest".to_string());
    let chest = BlockState::new("minecraft:chest".to_string());
    schematic.set_block(0, 0, 0, &chest);

    let mut block_entity = BlockEntity::new("minecraft:chest".to_string(), (0, 0, 0));
    block_entity.nbt.insert(
        "CustomName".to_string(),
        NbtValue::String("Test Chest".to_string()),
    );
    schematic.add_block_entity(block_entity);

    let bytes = to_snapshot(&schematic).unwrap();
    let restored = from_snapshot(&bytes).unwrap();

    let entities = restored.get_block_entities_as_list();
    assert_eq!(entities.len(), 1, "expected 1 block entity");
    assert_eq!(entities[0].id, "minecraft:chest");
    assert_eq!(entities[0].position, (0, 0, 0));
    assert_eq!(
        entities[0].nbt.get("CustomName"),
        Some(&NbtValue::String("Test Chest".to_string()))
    );
}

/// Empty schematic should round-trip cleanly.
#[test]
fn snapshot_roundtrip_empty() {
    let schematic = UniversalSchematic::new("Empty".to_string());

    let bytes = to_snapshot(&schematic).unwrap();
    let restored = from_snapshot(&bytes).unwrap();

    assert_eq!(schematic.metadata.name, restored.metadata.name);
    assert_eq!(restored.total_blocks(), 0);
}

/// Diverse palette: many different block types should all survive.
#[test]
fn snapshot_roundtrip_diverse_palette() {
    let block_names = [
        "minecraft:stone",
        "minecraft:dirt",
        "minecraft:grass_block",
        "minecraft:cobblestone",
        "minecraft:oak_planks",
        "minecraft:spruce_planks",
        "minecraft:birch_planks",
        "minecraft:sand",
        "minecraft:gravel",
        "minecraft:gold_ore",
        "minecraft:iron_ore",
        "minecraft:coal_ore",
        "minecraft:oak_log",
        "minecraft:spruce_log",
        "minecraft:glass",
        "minecraft:lapis_ore",
        "minecraft:sandstone",
        "minecraft:white_wool",
        "minecraft:bricks",
        "minecraft:bookshelf",
    ];

    let mut schematic = UniversalSchematic::new("DiverseTest".to_string());
    for (i, name) in block_names.iter().enumerate() {
        let block = BlockState::new(name.to_string());
        schematic.set_block(i as i32, 0, 0, &block);
    }

    let bytes = to_snapshot(&schematic).unwrap();
    let restored = from_snapshot(&bytes).unwrap();

    for (i, name) in block_names.iter().enumerate() {
        let block = restored
            .get_block(i as i32, 0, 0)
            .expect(&format!("block at x={} not found", i));
        assert_eq!(block.name.as_str(), *name, "block name mismatch at x={}", i);
    }
}

/// Solid cube: every position should be set and match after round-trip.
#[test]
fn snapshot_roundtrip_solid_cube() {
    let size = 16;
    let mut schematic = UniversalSchematic::new("SolidCube".to_string());
    let region = Region::new("Main".to_string(), (0, 0, 0), (size, size, size));
    schematic.add_region(region);

    let stone = BlockState::new("minecraft:stone".to_string());
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                schematic.set_block(x, y, z, &stone);
            }
        }
    }

    let bytes = to_snapshot(&schematic).unwrap();
    let restored = from_snapshot(&bytes).unwrap();

    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                let block = restored
                    .get_block(x, y, z)
                    .expect(&format!("block at ({},{},{}) not found", x, y, z));
                assert_eq!(
                    block.name.as_str(),
                    "minecraft:stone",
                    "mismatch at ({},{},{})",
                    x,
                    y,
                    z
                );
            }
        }
    }
}

/// Sparse data: only some positions are filled; unfilled should be air/absent.
#[test]
fn snapshot_roundtrip_sparse() {
    let mut schematic = UniversalSchematic::new("Sparse".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());

    // Set a few scattered blocks
    schematic.set_block(0, 0, 0, &stone);
    schematic.set_block(10, 20, 30, &stone);
    schematic.set_block(50, 50, 50, &stone);

    let bytes = to_snapshot(&schematic).unwrap();
    let restored = from_snapshot(&bytes).unwrap();

    assert_eq!(
        restored.get_block(0, 0, 0).map(|b| b.name.to_string()),
        Some("minecraft:stone".to_string())
    );
    assert_eq!(
        restored.get_block(10, 20, 30).map(|b| b.name.to_string()),
        Some("minecraft:stone".to_string())
    );
    assert_eq!(
        restored.get_block(50, 50, 50).map(|b| b.name.to_string()),
        Some("minecraft:stone".to_string())
    );

    // An unset position should be air (palette index 0) or absent
    let air_block = restored.get_block(5, 5, 5);
    if let Some(block) = air_block {
        assert_eq!(block.name.as_str(), "minecraft:air");
    }
}

/// Format manager auto-detection: snapshot bytes should be detected as "snapshot".
#[test]
fn snapshot_format_detection() {
    let mut schematic = UniversalSchematic::new("DetectTest".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());
    schematic.set_block(0, 0, 0, &stone);

    let bytes = to_snapshot(&schematic).unwrap();

    let manager = get_manager();
    let manager = manager.lock().unwrap();
    let detected = manager.detect_format(&bytes);
    assert_eq!(detected, Some("snapshot".to_string()));
}

/// Format manager read: auto-detect and read snapshot bytes.
#[test]
fn snapshot_format_manager_read() {
    let mut schematic = UniversalSchematic::new("ManagerRead".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());
    schematic.set_block(3, 4, 5, &stone);

    let bytes = to_snapshot(&schematic).unwrap();

    let manager = get_manager();
    let manager = manager.lock().unwrap();
    let restored = manager.read(&bytes).unwrap();

    assert_eq!(
        restored.get_block(3, 4, 5).map(|b| b.name.to_string()),
        Some("minecraft:stone".to_string())
    );
}

/// Format manager write: export via save_as("snapshot").
#[test]
fn snapshot_format_manager_write() {
    let mut schematic = UniversalSchematic::new("ManagerWrite".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());
    schematic.set_block(7, 8, 9, &stone);

    let manager = get_manager();
    let manager = manager.lock().unwrap();
    let bytes = manager.write("snapshot", &schematic, None).unwrap();

    // Should start with NUSN magic
    assert_eq!(&bytes[0..4], b"NUSN");

    // Should round-trip back
    let restored = from_snapshot(&bytes).unwrap();
    assert_eq!(
        restored.get_block(7, 8, 9).map(|b| b.name.to_string()),
        Some("minecraft:stone".to_string())
    );
}

/// Invalid data: should return errors, not panic.
#[test]
fn snapshot_invalid_data() {
    // Too short
    assert!(from_snapshot(&[]).is_err());
    assert!(from_snapshot(&[0, 1, 2]).is_err());

    // Wrong magic
    assert!(from_snapshot(&[0, 0, 0, 0, 1, 0, 0, 0]).is_err());

    // Wrong version
    let mut bad_version = Vec::new();
    bad_version.extend_from_slice(b"NUSN");
    bad_version.extend_from_slice(&99u32.to_le_bytes());
    assert!(from_snapshot(&bad_version).is_err());

    // Valid header but truncated payload
    let mut truncated = Vec::new();
    truncated.extend_from_slice(b"NUSN");
    truncated.extend_from_slice(&1u32.to_le_bytes());
    truncated.extend_from_slice(&[0, 1, 2]); // garbage payload
    assert!(from_snapshot(&truncated).is_err());
}

/// Magic bytes header check.
#[test]
fn snapshot_header_format() {
    let schematic = UniversalSchematic::new("Header".to_string());
    let bytes = to_snapshot(&schematic).unwrap();

    assert!(bytes.len() >= 8);
    assert_eq!(&bytes[0..4], b"NUSN", "magic mismatch");
    assert_eq!(
        u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        1,
        "version mismatch"
    );
}

/// Metadata fields should all survive the round-trip.
#[test]
fn snapshot_roundtrip_all_metadata() {
    let mut schematic = UniversalSchematic::new("MetaTest".to_string());
    schematic.metadata.author = Some("Alice".to_string());
    schematic.metadata.description = Some("Desc".to_string());
    schematic.metadata.created = Some(1000);
    schematic.metadata.modified = Some(2000);
    schematic.metadata.mc_version = Some(3578);
    schematic.metadata.we_version = Some(7);
    schematic.metadata.lm_version = Some(6);

    let bytes = to_snapshot(&schematic).unwrap();
    let restored = from_snapshot(&bytes).unwrap();

    assert_eq!(restored.metadata.name, schematic.metadata.name);
    assert_eq!(restored.metadata.author, schematic.metadata.author);
    assert_eq!(
        restored.metadata.description,
        schematic.metadata.description
    );
    assert_eq!(restored.metadata.created, schematic.metadata.created);
    assert_eq!(restored.metadata.modified, schematic.metadata.modified);
    assert_eq!(restored.metadata.mc_version, schematic.metadata.mc_version);
    assert_eq!(restored.metadata.we_version, schematic.metadata.we_version);
    assert_eq!(restored.metadata.lm_version, schematic.metadata.lm_version);
}
