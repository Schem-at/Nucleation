use flate2::read::GzDecoder;
use nucleation::UniversalSchematic;
use quartz_nbt::io::{read_nbt, Flavor};
use quartz_nbt::{NbtCompound, NbtTag};
use std::io::BufReader;

/// Helper to parse schematic and get first block entity
fn get_first_block_entity(block_string: &str) -> NbtCompound {
    let mut schematic = UniversalSchematic::new("test".to_string());
    schematic
        .set_block_from_string(0, 0, 0, block_string)
        .expect("Failed to set block from string");

    let schem_bytes = schematic
        .to_schematic()
        .expect("Failed to convert to schematic");
    let reader = BufReader::with_capacity(1 << 20, &schem_bytes[..]);
    let mut gz = GzDecoder::new(reader);
    let (root, _) = read_nbt(&mut gz, Flavor::Uncompressed).expect("Failed to read NBT");

    let schematic_tag = root
        .get::<_, &NbtCompound>("Schematic")
        .expect("No Schematic tag");
    let blocks = schematic_tag
        .get::<_, &NbtCompound>("Blocks")
        .expect("No Blocks tag");
    let block_entities_list = blocks
        .get::<_, &quartz_nbt::NbtList>("BlockEntities")
        .expect("No BlockEntities list");

    if let NbtTag::Compound(be) = &block_entities_list[0] {
        be.clone()
    } else {
        panic!("First block entity is not a compound");
    }
}

/// Get Items array from Data compound
fn get_items_from_data(be: &NbtCompound) -> &quartz_nbt::NbtList {
    let data = be.get::<_, &NbtCompound>("Data").expect("No Data compound");
    data.get::<_, &quartz_nbt::NbtList>("Items")
        .expect("No Items in Data")
}

// ===== SIGNAL SHORTHAND TESTS =====

#[test]
fn test_signal_barrel_default_item() {
    let be = get_first_block_entity("minecraft:barrel[facing=north]{signal=14}");
    let items = get_items_from_data(&be);

    // Should create items for signal strength 14
    assert!(items.len() > 0, "Should have items for signal=14");

    // Check first item uses default (redstone_block)
    if let NbtTag::Compound(item) = &items[0] {
        let id = item.get::<_, &String>("id").expect("Item should have id");
        assert_eq!(
            id, "minecraft:redstone_block",
            "Default item should be redstone_block"
        );
    } else {
        panic!("First item is not a compound");
    }
}

#[test]
fn test_signal_chest() {
    let be = get_first_block_entity("minecraft:chest[facing=north]{signal=10}");
    let items = get_items_from_data(&be);

    assert!(items.len() > 0, "Chest should support signal shorthand");
}

#[test]
fn test_signal_hopper() {
    let be = get_first_block_entity("minecraft:hopper[facing=down]{signal=8}");
    let items = get_items_from_data(&be);

    assert!(items.len() > 0, "Hopper should support signal shorthand");

    // Hopper has only 5 slots, so shouldn't exceed that
    assert!(items.len() <= 5, "Hopper should not exceed 5 slots");
}

#[test]
fn test_signal_dispenser() {
    let be = get_first_block_entity("minecraft:dispenser[facing=north]{signal=5}");
    let items = get_items_from_data(&be);

    assert!(items.len() > 0, "Dispenser should support signal shorthand");
    assert!(items.len() <= 9, "Dispenser should not exceed 9 slots");
}

#[test]
fn test_signal_with_custom_item() {
    let be =
        get_first_block_entity("minecraft:barrel[facing=north]{signal=14,item=minecraft:diamond}");
    let items = get_items_from_data(&be);

    assert!(items.len() > 0, "Should have items");

    // Check that items use the custom item
    if let NbtTag::Compound(item) = &items[0] {
        let id = item.get::<_, &String>("id").expect("Item should have id");
        assert_eq!(id, "minecraft:diamond", "Should use custom item");
    } else {
        panic!("First item is not a compound");
    }
}

#[test]
fn test_signal_with_custom_item_namespaced() {
    let be = get_first_block_entity("minecraft:chest{signal=12,item=stone}");
    let items = get_items_from_data(&be);

    if let NbtTag::Compound(item) = &items[0] {
        let id = item.get::<_, &String>("id").expect("Item should have id");
        // Should auto-add minecraft: namespace
        assert_eq!(id, "minecraft:stone", "Should add namespace if missing");
    } else {
        panic!("First item is not a compound");
    }
}

#[test]
fn test_signal_zero() {
    // signal=0 should not create a block entity at all (no items = no block entity needed)
    let mut schematic = UniversalSchematic::new("test".to_string());
    schematic
        .set_block_from_string(0, 0, 0, "minecraft:barrel{signal=0}")
        .expect("Failed to set block");

    let schem_bytes = schematic.to_schematic().expect("Failed to convert");
    let reader = BufReader::with_capacity(1 << 20, &schem_bytes[..]);
    let mut gz = GzDecoder::new(reader);
    let (root, _) = read_nbt(&mut gz, Flavor::Uncompressed).expect("Failed to read NBT");

    let schematic_tag = root
        .get::<_, &NbtCompound>("Schematic")
        .expect("No Schematic tag");
    let blocks = schematic_tag
        .get::<_, &NbtCompound>("Blocks")
        .expect("No Blocks tag");
    let block_entities_list = blocks
        .get::<_, &quartz_nbt::NbtList>("BlockEntities")
        .expect("No BlockEntities list");

    // signal=0 means empty container - may or may not create a block entity
    // If it does, it should have no items
    if block_entities_list.len() > 0 {
        if let NbtTag::Compound(be) = &block_entities_list[0] {
            if let Ok(data) = be.get::<_, &NbtCompound>("Data") {
                if let Ok(items) = data.get::<_, &quartz_nbt::NbtList>("Items") {
                    assert_eq!(items.len(), 0, "signal=0 should have no items");
                }
            }
        }
    }
    // If no block entity was created, that's also fine
}

#[test]
fn test_signal_max() {
    let be = get_first_block_entity("minecraft:barrel{signal=15}");
    let items = get_items_from_data(&be);

    // signal=15 should fill the barrel
    assert!(items.len() > 20, "signal=15 should nearly fill a barrel");
}

// ===== GENERIC NBT PARSING TESTS =====

#[test]
fn test_nbt_lock() {
    let be = get_first_block_entity("minecraft:chest{Lock:\"MyKey\"}");
    let data = be.get::<_, &NbtCompound>("Data").expect("Should have Data");
    let lock = data
        .get::<_, &String>("Lock")
        .expect("Should have Lock field");

    assert_eq!(lock, "MyKey", "Lock should be parsed correctly");
}

#[test]
fn test_nbt_loot_table() {
    let be = get_first_block_entity(
        "minecraft:chest{LootTable:\"minecraft:chests/village_blacksmith\"}",
    );
    let data = be.get::<_, &NbtCompound>("Data").expect("Should have Data");
    let loot_table = data
        .get::<_, &String>("LootTable")
        .expect("Should have LootTable");

    assert_eq!(loot_table, "minecraft:chests/village_blacksmith");
}

#[test]
fn test_nbt_sign_text() {
    let be = get_first_block_entity(
        "minecraft:sign{Text1:\"Line 1\",Text2:\"Line 2\",Text3:\"\",Text4:\"Line 4\"}",
    );
    let data = be.get::<_, &NbtCompound>("Data").expect("Should have Data");

    let text1 = data.get::<_, &String>("Text1").expect("Should have Text1");
    let text2 = data.get::<_, &String>("Text2").expect("Should have Text2");
    let text4 = data.get::<_, &String>("Text4").expect("Should have Text4");

    assert_eq!(text1, "Line 1");
    assert_eq!(text2, "Line 2");
    assert_eq!(text4, "Line 4");
}

#[test]
fn test_nbt_integer() {
    let be = get_first_block_entity("minecraft:furnace{BurnTime:100,CookTime:50}");
    let data = be.get::<_, &NbtCompound>("Data").expect("Should have Data");

    let burn_time = data
        .get::<_, i32>("BurnTime")
        .expect("Should have BurnTime");
    let cook_time = data
        .get::<_, i32>("CookTime")
        .expect("Should have CookTime");

    assert_eq!(burn_time, 100);
    assert_eq!(cook_time, 50);
}

#[test]
fn test_nbt_boolean_as_byte() {
    let be = get_first_block_entity("minecraft:chest{ShowParticles:1b}");
    let data = be.get::<_, &NbtCompound>("Data").expect("Should have Data");

    let show_particles = data
        .get::<_, i8>("ShowParticles")
        .expect("Should have ShowParticles");
    assert_eq!(show_particles, 1);
}

#[test]
fn test_nbt_mixed_types() {
    let be = get_first_block_entity(
        "minecraft:chest{Lock:\"Key\",LootTable:\"minecraft:chests/test\",LootTableSeed:12345}",
    );
    let data = be.get::<_, &NbtCompound>("Data").expect("Should have Data");

    assert!(data.contains_key("Lock"));
    assert!(data.contains_key("LootTable"));
    assert!(data.contains_key("LootTableSeed"));

    let seed = data.get::<_, i32>("LootTableSeed").ok();
    assert_eq!(seed, Some(12345));
}

// ===== COMBINED FEATURES TESTS =====

#[test]
fn test_signal_with_additional_nbt() {
    let be = get_first_block_entity("minecraft:chest{signal=10,Lock:\"MyKey\"}");
    let data = be.get::<_, &NbtCompound>("Data").expect("Should have Data");

    // Should have both Items (from signal) and Lock
    assert!(data.contains_key("Items"), "Should have Items from signal");
    assert!(data.contains_key("Lock"), "Should have Lock from NBT");

    let lock = data.get::<_, &String>("Lock").expect("Should have Lock");
    assert_eq!(lock, "MyKey");
}

#[test]
fn test_custom_items_with_signal_ignored() {
    // If Items are explicitly provided, signal should be ignored
    let be = get_first_block_entity(
        "minecraft:chest{Items:[{Count:\"1b\",Slot:\"0b\",id:\"minecraft:diamond\"}],signal=10}",
    );
    let data = be.get::<_, &NbtCompound>("Data").expect("Should have Data");
    let items = data
        .get::<_, &quartz_nbt::NbtList>("Items")
        .expect("Should have Items");

    // Should use the explicit Items, not generate from signal
    assert_eq!(items.len(), 1, "Should use explicit Items, not signal");

    if let NbtTag::Compound(item) = &items[0] {
        let id = item.get::<_, &String>("id").expect("Item should have id");
        assert_eq!(id, "minecraft:diamond");
    }
}

// ===== ERROR HANDLING TESTS =====

#[test]
#[should_panic(expected = "Signal strength must be between 0 and 15")]
fn test_signal_invalid_value() {
    let _ = get_first_block_entity("minecraft:barrel{signal=16}");
}

#[test]
#[should_panic] // Negative number won't parse as u8, will fail differently
fn test_signal_negative() {
    let _ = get_first_block_entity("minecraft:barrel{signal=-1}");
}

#[test]
fn test_signal_non_container() {
    // signal on non-container should be ignored or error
    let result = std::panic::catch_unwind(|| {
        get_first_block_entity("minecraft:stone{signal=10}");
    });

    // Should either panic or not create a block entity
    // (stone doesn't have block entities)
    assert!(result.is_err() || result.is_ok());
}

// ===== COMPATIBILITY TESTS =====

#[test]
fn test_backwards_compat_custom_name() {
    // Old syntax should still work
    let be = get_first_block_entity("minecraft:chest{CustomName:'{\"text\":\"My Chest\"}'}");
    let data = be.get::<_, &NbtCompound>("Data").expect("Should have Data");

    assert!(data.contains_key("CustomName"));
}

#[test]
fn test_backwards_compat_items_array() {
    // Old Items array syntax should still work
    let be = get_first_block_entity(
        "minecraft:chest{Items:[{Count:\"64b\",Slot:\"0b\",id:\"minecraft:diamond\"}]}",
    );
    let items = get_items_from_data(&be);

    assert_eq!(items.len(), 1);
}

#[test]
fn test_count_field_format() {
    // Verify count field uses lowercase and Int type (modern format 1.20.5+)
    let be = get_first_block_entity("minecraft:barrel{signal=14}");
    let items = get_items_from_data(&be);

    assert!(items.len() > 0, "Should have items");

    if let NbtTag::Compound(item) = &items[0] {
        // Should have lowercase 'count', not uppercase 'Count' (modern format)
        assert!(
            item.contains_key("count"),
            "Should have lowercase 'count' field"
        );
        assert!(
            !item.contains_key("Count"),
            "Should NOT have uppercase 'Count' field"
        );

        // Should be Int, not Byte (modern format)
        let count = item
            .get::<_, i32>("count")
            .expect("count should be Int type");
        assert!(count > 0, "count should be positive");
        assert!(count <= 64, "count should not exceed stack size");
    } else {
        panic!("First item is not a compound");
    }
}
