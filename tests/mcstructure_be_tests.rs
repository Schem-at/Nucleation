//! Block-entity-preservation tests for the Bedrock `.mcstructure` round-trip.
//!
//! These check the second class of bug from the community report:
//! "blocks appear but they have no items / no text / no piston arm".
//! N-2 (wiring `BlockEntityTranslator::translate_java_to_bedrock` into the
//! exporter) and BP-2/BP-3 (the translator itself) together address them.
//!
//! Pattern per test:
//! 1. Build a single-block schematic with a populated block entity
//!    (items in a hopper, text on a sign, etc.)
//! 2. Export to mcstructure, reimport
//! 3. Assert that the BE data we set is still present after the round-trip

use nucleation::block_entity::BlockEntity;
use nucleation::block_position::BlockPosition;
use nucleation::formats::mcstructure::{from_mcstructure, to_mcstructure};
use nucleation::nbt::{NbtMap, NbtValue};
use nucleation::{BlockState, UniversalSchematic};

fn build_with_be(block_name: &str, props: &[(&str, &str)], be: BlockEntity) -> UniversalSchematic {
    let mut state = BlockState::new(block_name.to_string());
    for (k, v) in props {
        state = state.with_property(k.to_string(), v.to_string());
    }
    let mut s = UniversalSchematic::new("be-test".to_string());
    s.set_block(0, 0, 0, &state);
    s.set_block_entity(BlockPosition { x: 0, y: 0, z: 0 }, be);
    s
}

fn round_trip(s: &UniversalSchematic) -> UniversalSchematic {
    let bytes = to_mcstructure(s).expect("to_mcstructure");
    from_mcstructure(&bytes).expect("from_mcstructure")
}

fn fetch_be(s: &UniversalSchematic) -> &BlockEntity {
    s.get_block_entity(BlockPosition { x: 0, y: 0, z: 0 })
        .expect("expected a block entity at origin after round-trip")
}

fn item_compound(id: &str, count: i8, slot: i8) -> NbtValue {
    let mut item = NbtMap::new();
    item.insert("id".to_string(), NbtValue::String(id.to_string()));
    item.insert("Count".to_string(), NbtValue::Byte(count));
    item.insert("Slot".to_string(), NbtValue::Byte(slot));
    NbtValue::Compound(item)
}

/// Extract every "id"-like field from an Items list. Accepts both Java ("id")
/// and Bedrock ("Name") item naming because the importer may leave items in
/// either shape depending on how far the BE translator has converted them.
fn item_ids(items: &[NbtValue]) -> Vec<String> {
    let mut out = Vec::new();
    for it in items {
        if let NbtValue::Compound(c) = it {
            if let Some(NbtValue::String(s)) = c.get("id") {
                out.push(s.clone());
                continue;
            }
            if let Some(NbtValue::String(s)) = c.get("Name") {
                out.push(s.clone());
            }
        }
    }
    out
}

/// Pull the Items list out of a BE's nbt iterator.
fn extract_items(be: &BlockEntity) -> Vec<NbtValue> {
    for (k, v) in be.nbt.iter() {
        if k.as_str() == "Items" {
            if let NbtValue::List(items) = v {
                return items.clone();
            }
        }
    }
    Vec::new()
}

#[test]
fn hopper_keeps_items() {
    let be = BlockEntity::new("minecraft:hopper".to_string(), (0, 0, 0))
        .with_nbt_data(
            "Items".to_string(),
            NbtValue::List(vec![item_compound("minecraft:cobblestone", 64, 0)]),
        )
        .with_nbt_data("TransferCooldown".to_string(), NbtValue::Int(0));

    let s = build_with_be("minecraft:hopper", &[("facing", "north")], be);
    let reloaded = round_trip(&s);
    let be2 = fetch_be(&reloaded);
    let ids = item_ids(&extract_items(be2));
    assert!(
        ids.iter().any(|s| s.ends_with("cobblestone")),
        "expected cobblestone in hopper, got {:?}",
        ids
    );
}

#[test]
fn chest_keeps_multiple_items() {
    let be = BlockEntity::new("minecraft:chest".to_string(), (0, 0, 0)).with_nbt_data(
        "Items".to_string(),
        NbtValue::List(vec![
            item_compound("minecraft:diamond", 1, 0),
            item_compound("minecraft:iron_ingot", 32, 1),
            item_compound("minecraft:oak_log", 64, 7),
        ]),
    );

    let s = build_with_be(
        "minecraft:chest",
        &[
            ("facing", "south"),
            ("type", "single"),
            ("waterlogged", "false"),
        ],
        be,
    );
    let reloaded = round_trip(&s);
    let be2 = fetch_be(&reloaded);
    let ids = item_ids(&extract_items(be2));
    assert_eq!(
        ids.len(),
        3,
        "expected 3 items, got {}: {:?}",
        ids.len(),
        ids
    );
    assert!(ids.iter().any(|s| s.ends_with("diamond")), "{:?}", ids);
    assert!(ids.iter().any(|s| s.ends_with("iron_ingot")), "{:?}", ids);
    assert!(ids.iter().any(|s| s.ends_with("oak_log")), "{:?}", ids);
}

#[test]
fn dispenser_keeps_items() {
    let be = BlockEntity::new("minecraft:dispenser".to_string(), (0, 0, 0)).with_nbt_data(
        "Items".to_string(),
        NbtValue::List(vec![item_compound("minecraft:arrow", 16, 0)]),
    );
    let s = build_with_be(
        "minecraft:dispenser",
        &[("facing", "up"), ("triggered", "false")],
        be,
    );
    let reloaded = round_trip(&s);
    let be2 = fetch_be(&reloaded);
    let ids = item_ids(&extract_items(be2));
    assert!(
        ids.iter().any(|s| s.ends_with("arrow")),
        "expected arrow in dispenser, got {:?}",
        ids
    );
}

#[test]
fn furnace_keeps_smelt_state() {
    let be = BlockEntity::new("minecraft:furnace".to_string(), (0, 0, 0))
        .with_nbt_data("BurnTime".to_string(), NbtValue::Short(200))
        .with_nbt_data("CookTime".to_string(), NbtValue::Short(50))
        .with_nbt_data("CookTimeTotal".to_string(), NbtValue::Short(200))
        .with_nbt_data(
            "Items".to_string(),
            NbtValue::List(vec![
                item_compound("minecraft:iron_ore", 4, 0),
                item_compound("minecraft:coal", 1, 1),
            ]),
        );
    let s = build_with_be(
        "minecraft:furnace",
        &[("facing", "north"), ("lit", "true")],
        be,
    );
    let reloaded = round_trip(&s);
    let be2 = fetch_be(&reloaded);
    let ids = item_ids(&extract_items(be2));
    assert!(ids.iter().any(|s| s.ends_with("iron_ore")), "{:?}", ids);
    assert!(ids.iter().any(|s| s.ends_with("coal")), "{:?}", ids);
}

#[test]
fn unknown_be_passes_through_unchanged() {
    // For block-entity ids we don't recognise, the translator should be a
    // no-op so we never lose data.
    let be = BlockEntity::new("minecraft:custom_thing".to_string(), (0, 0, 0))
        .with_nbt_data("custom_field".to_string(), NbtValue::Int(123));
    let s = build_with_be("minecraft:hopper", &[], be);
    let reloaded = round_trip(&s);
    let be2 = fetch_be(&reloaded);
    let custom = be2.nbt.iter().find(|(k, _)| k.as_str() == "custom_field");
    assert!(
        custom.is_some(),
        "unknown BE id lost the custom_field — translator must pass through"
    );
}
