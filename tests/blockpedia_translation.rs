use nucleation::blockpedia::{BlockState, block_entity::{BlockEntityTranslator, NbtValue}};
use std::collections::HashMap;

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
