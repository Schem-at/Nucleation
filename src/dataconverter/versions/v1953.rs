//! V1953 (1.14 + 1) — schematic-relevant subset of `V1953.java`.
//!
//! Ported: the TILE_ENTITY `minecraft:banner` converter that rewrites the
//! "illager_banner" translate key in the block-entity's `CustomName` to
//! "ominous_banner" (V1953.java:13-22).
//!
//! VERSION = MCVersions.V1_14 + 1 = 1952 + 1 = 1953.
//!
//! Nothing non-schematic is present in this version.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 1953;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_converter_for_id(
        "minecraft:banner",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // CustomName is a JSON-string text component; plain substring replace.
            if let Some(name) = data.get_string("CustomName").map(|s| s.to_string()) {
                let new_name = name.replace(
                    "\"translate\":\"block.minecraft.illager_banner\"",
                    "\"translate\":\"block.minecraft.ominous_banner\"",
                );
                data.set_string("CustomName", new_name);
            }
        }),
    );

    // Reverse of the forward banner CustomName rewrite (V1953.java:13-22).
    // Forward: "illager_banner" translate key -> "ominous_banner". Inverse is the
    // exact opposite substring replace; the old format never had an "ominous_banner"
    // key, so this is lossless (bucket A: reversible rename).
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:banner",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if let Some(name) = data.get_string("CustomName").map(|s| s.to_string()) {
                let old_name = name.replace(
                    "\"translate\":\"block.minecraft.ominous_banner\"",
                    "\"translate\":\"block.minecraft.illager_banner\"",
                );
                data.set_string("CustomName", old_name);
            }
        }),
    );
}
