use nucleation::blockpedia::{BlockState, block_entity::{BlockEntityTranslator, NbtValue}};
use nucleation::blockpedia::bedrock_mapping::{BedrockBlockStateMapper, BEDROCK_J2B_MAP};
use std::collections::HashMap;

/// Guards the GeyserMC mapping snapshot (data/blockpedia/geyser_mappings.json.gz).
///
/// The count floor is the full Java 26.2 blockstate space (32,366 states,
/// GeyserMC/mappings @ efe0f2c): a regeneration that loses coverage or a
/// converter regression that misaligns state ids should trip one of these.
#[test]
fn test_bedrock_mapping_count_and_known_blocks() {
    assert!(
        BEDROCK_J2B_MAP.len() >= 32_366,
        "Java->Bedrock mapping count regressed: {} < 32366",
        BEDROCK_J2B_MAP.len()
    );

    // Well-known identity mapping
    assert_eq!(
        BedrockBlockStateMapper::java_to_bedrock("minecraft:stone[]"),
        Some("minecraft:stone[]")
    );

    // Property translation: Java stairs facing/half -> Bedrock weirdo_direction/upside_down_bit
    assert_eq!(
        BedrockBlockStateMapper::java_to_bedrock(
            "minecraft:oak_stairs[facing=north,half=top,shape=straight,waterlogged=true]"
        ),
        Some("minecraft:oak_stairs[upside_down_bit=true,weirdo_direction=3]")
    );

    // 26.2-only block, gained with the GeyserMC/mappings 26.2 refresh
    assert_eq!(
        BedrockBlockStateMapper::java_to_bedrock("minecraft:cinnabar[]"),
        Some("minecraft:cinnabar[]")
    );

    // Bedrock -> Java direction survives too
    assert_eq!(
        BedrockBlockStateMapper::bedrock_to_java("minecraft:stone[]"),
        Some("minecraft:stone[]")
    );
}

#[test]
fn test_block_state_translation_round_trip() {
    // 1. Bedrock -> Java
    // Bedrock: minecraft:stone[]
    let props = HashMap::new();
    let java_state = BlockState::from_bedrock("minecraft:stone", props.clone()).unwrap();
    assert_eq!(java_state.id(), "minecraft:stone");
    
    // Java -> Bedrock
    let bedrock_state = java_state.to_bedrock().unwrap();
    assert_eq!(bedrock_state.id(), "minecraft:stone");
}

#[test]
fn test_procedural_mapping_fallback() {
    // Bedrock: minecraft:chest [minecraft:cardinal_direction=north]
    // Should map to Java: minecraft:chest [facing=north]
    
    let mut props = HashMap::new();
    props.insert("minecraft:cardinal_direction".to_string(), "2".to_string()); // 2 = north
    
    // Note: The mapping logic for "2" -> "north" needs to be implemented/verified in from_bedrock
    // Currently implemented: "2" -> "north"
    
    let java_state = BlockState::from_bedrock("minecraft:chest", props).unwrap();
    assert_eq!(java_state.id(), "minecraft:chest");
    assert_eq!(java_state.get_property("facing"), Some("north"));
}

#[test]
fn test_block_entity_translation() {
    let mut item = HashMap::new();
    item.insert("Name".to_string(), NbtValue::String("minecraft:diamond".to_string()));
    item.insert("Count".to_string(), NbtValue::Byte(64));
    
    let item_tag = NbtValue::Compound(item);
    
    let mut items_list = Vec::new();
    items_list.push(item_tag);
    
    let mut bedrock_nbt = HashMap::new();
    bedrock_nbt.insert("id".to_string(), NbtValue::String("Chest".to_string()));
    bedrock_nbt.insert("Items".to_string(), NbtValue::List(items_list));
    
    let java_nbt = BlockEntityTranslator::translate_bedrock_to_java(&bedrock_nbt);
    
    // Check item translation
    if let Some(NbtValue::List(items)) = java_nbt.get("Items") {
        let first_item = &items[0];
        if let NbtValue::Compound(tag) = first_item {
            assert!(tag.contains_key("id"));
            assert_eq!(tag.get("id"), Some(&NbtValue::String("minecraft:diamond".to_string())));
            assert!(!tag.contains_key("Name"));
        } else {
            panic!("Item is not a compound");
        }
    } else {
        panic!("Items list missing");
    }
}
