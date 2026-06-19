//! V1948 (1.14-pre2) — schematic-relevant subset of `V1948.java`.
//!
//! Ported: the ITEM_STACK `minecraft:white_banner` converter that rewrites the
//! "illager_banner" translate key in the item's display name to
//! "ominous_banner" (V1948.java:13-37).
//!
//! VERSION = MCVersions.V1_14_PRE2 = 1948.
//!
//! Nothing non-schematic is present in this version.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 1948;

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_converter_for_id(
        "minecraft:white_banner",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // data["tag"]["display"]["Name"] is a JSON-string text component; the
            // converter does a plain substring replace on it.
            let new_name = match data
                .get_map("tag")
                .and_then(|tag| tag.get_map("display"))
                .and_then(|display| display.get_string("Name"))
            {
                Some(name) => name.replace(
                    "\"translate\":\"block.minecraft.illager_banner\"",
                    "\"translate\":\"block.minecraft.ominous_banner\"",
                ),
                None => return,
            };

            if let Some(display) = data
                .get_map_mut("tag")
                .and_then(|tag| tag.get_map_mut("display"))
            {
                display.set_string("Name", new_name);
            }
        }),
    );

    // Reverse of V1948.java:13-37. The forward did a plain substring replace
    // "illager_banner" -> "ominous_banner" on the white_banner display Name.
    // The inverse restores it: "ominous_banner" -> "illager_banner". This is
    // exact/lossless — the forward produced this exact substring, so reversing
    // it reconstructs the original key (bucket A: rename inverse).
    reg.item_stack.add_reverse_converter_for_id(
        "minecraft:white_banner",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let new_name = match data
                .get_map("tag")
                .and_then(|tag| tag.get_map("display"))
                .and_then(|display| display.get_string("Name"))
            {
                Some(name) => name.replace(
                    "\"translate\":\"block.minecraft.ominous_banner\"",
                    "\"translate\":\"block.minecraft.illager_banner\"",
                ),
                None => return,
            };

            if let Some(display) = data
                .get_map_mut("tag")
                .and_then(|tag| tag.get_map_mut("display"))
            {
                display.set_string("Name", new_name);
            }
        }),
    );
}
