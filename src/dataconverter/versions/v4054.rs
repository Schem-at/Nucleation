//! V4054 (1.21.1 + 99) — schematic-relevant subset of `V4054.java`.
//!
//! The ominous-banner identity is upgraded: a banner whose `minecraft:item_name`
//! component is the translatable component `block.minecraft.ominous_banner`
//! gains `minecraft:rarity = "uncommon"` and has its item name normalised to the
//! canonical translatable component.
//!
//!   * TILE_ENTITY `minecraft:banner` converter.
//!   * ITEM_STACK `minecraft:white_banner` converter.
//!
//! Both share `convertComponents`. Component values are stored as JSON strings;
//! `retrieveTranslationString` parses the JSON and pulls the `translate` key,
//! and `createTranslatableComponent` re-emits the canonical stable form
//! (`{"translate":"block.minecraft.ominous_banner"}`).
//!
//! Nothing non-schematic is present in this version.
//!
//! VERSION = MCVersions.V1_21_1 (3955) + 99 = 4054.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4054;

/// `ComponentUtils.retrieveTranslationString`: parse `possible_json` and, if it
/// is an object with a string `translate` field, return that string. Any parse
/// failure or shape mismatch yields `None` (Java returns null).
fn retrieve_translation_string(possible_json: &str) -> Option<String> {
    let element: serde_json::Value = serde_json::from_str(possible_json).ok()?;
    element.get("translate")?.as_str().map(|s| s.to_string())
}

/// `ComponentUtils.createTranslatableComponent`: the stable JSON form
/// `{"translate":"<key>"}` (single key, so no key ordering concerns).
fn create_translatable_component(key: &str) -> String {
    serde_json::json!({ "translate": key }).to_string()
}

/// `convertComponents`: promote an ominous banner's rarity + item name.
fn convert_components(components: Option<&mut NbtMap>) {
    let components = match components {
        Some(c) => c,
        None => return,
    };

    let item_name_key = components
        .get_string("minecraft:item_name")
        .and_then(retrieve_translation_string);

    if item_name_key.as_deref() != Some("block.minecraft.ominous_banner") {
        return;
    }

    components.set_string("minecraft:rarity", "uncommon");
    components.set_string(
        "minecraft:item_name",
        create_translatable_component("block.minecraft.ominous_banner"),
    );
}

/// Reverse of `convertComponents` (V4054.java:30-43).
///
/// The forward step does two additive/normalizing things to an ominous banner
/// (item whose `minecraft:item_name` translates to `block.minecraft.ominous_banner`):
///   * adds `minecraft:rarity = "uncommon"` (bucket D — drop on the way back);
///   * canonicalizes `minecraft:item_name` to `{"translate":"…"}` (a pure JSON
///     normalization whose inverse is identity — the pre-canonical text cannot
///     be recovered, but the logical value is unchanged, so this is not a loss).
///
/// We identify the ominous banner via the (canonical) item name, then drop the
/// rarity the forward inserted, but only when it still equals the value the
/// forward set ("uncommon") so we never discard a user-authored rarity.
fn reverse_convert_components(components: Option<&mut NbtMap>) {
    let components = match components {
        Some(c) => c,
        None => return,
    };

    let item_name_key = components
        .get_string("minecraft:item_name")
        .and_then(retrieve_translation_string);

    if item_name_key.as_deref() != Some("block.minecraft.ominous_banner") {
        return;
    }

    if components.get_string("minecraft:rarity") == Some("uncommon") {
        components.take("minecraft:rarity");
        report_loss(
            VERSION,
            LossKind::FingerprintCollapse,
            Severity::Approximated,
            "ominous banner uncommon rarity may be forward-added or user-authored; removed it for the legacy preimage",
        );
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_converter_for_id(
        "minecraft:banner",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            convert_components(data.get_map_mut("components"));
        }),
    );
    // Reverse of TILE_ENTITY minecraft:banner (V4054.java:14-20).
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:banner",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            reverse_convert_components(data.get_map_mut("components"));
        }),
    );

    reg.item_stack.add_converter_for_id(
        "minecraft:white_banner",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            convert_components(data.get_map_mut("components"));
        }),
    );
    // Reverse of ITEM_STACK minecraft:white_banner (V4054.java:21-27).
    reg.item_stack.add_reverse_converter_for_id(
        "minecraft:white_banner",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            reverse_convert_components(data.get_map_mut("components"));
        }),
    );
}
