//! V813 (16w40a) — schematic-relevant subset.
//!
//! Port of DataConverterJava .../versions/V813.java:
//!   * ITEM_STACK `minecraft:shulker_box`: read `tag.BlockEntityTag.Color`,
//!     remove it, and rewrite the item `id` to the matching coloured
//!     `*_shulker_box` (indexed by `Color % 16`).
//!   * TILE_ENTITY `minecraft:shulker_box`: drop the now-unused `Color` field.
//!
//! Nothing is skipped — both registrations target schematic types.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 813;

/// Coloured shulker-box item ids indexed by the legacy `Color` value
/// (V813.java:12-29).
const SHULKER_ID_BY_COLOUR: &[&str] = &[
    "minecraft:white_shulker_box",
    "minecraft:orange_shulker_box",
    "minecraft:magenta_shulker_box",
    "minecraft:light_blue_shulker_box",
    "minecraft:yellow_shulker_box",
    "minecraft:lime_shulker_box",
    "minecraft:pink_shulker_box",
    "minecraft:gray_shulker_box",
    "minecraft:silver_shulker_box",
    "minecraft:cyan_shulker_box",
    "minecraft:purple_shulker_box",
    "minecraft:blue_shulker_box",
    "minecraft:brown_shulker_box",
    "minecraft:green_shulker_box",
    "minecraft:red_shulker_box",
    "minecraft:black_shulker_box",
];

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_converter_for_id(
        "minecraft:shulker_box",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // Need tag.BlockEntityTag; bail if either is missing (Java returns null).
            let color = match data.get_map_mut("tag").and_then(|t| t.get_map_mut("BlockEntityTag"))
            {
                Some(block_entity) => {
                    // getInt returns 0 when absent/non-numeric, like vanilla.
                    let color = block_entity.get_i32("Color").unwrap_or(0);
                    block_entity.take("Color");
                    color
                }
                None => return,
            };

            // color % len, guarding against negative values the same way Java's
            // `%` would for the in-range colours actually produced here.
            let len = SHULKER_ID_BY_COLOUR.len() as i32;
            let idx = color.rem_euclid(len) as usize;
            data.set_string("id", SHULKER_ID_BY_COLOUR[idx]);
        }),
    );

    // Reverse of the ITEM_STACK converter (new -> old). The forward replaced the
    // colourless `minecraft:shulker_box` id with one of the 16 coloured
    // `*_shulker_box` ids (indexed by `Color % 16`) and removed
    // `tag.BlockEntityTag.Color`. Each coloured id appears exactly once in
    // SHULKER_ID_BY_COLOUR, so the id uniquely encodes the colour index =>
    // this is a lossless structural inverse (rule 11): restore the generic
    // `minecraft:shulker_box` id and write the colour back into
    // `tag.BlockEntityTag.Color`. No loss report.
    //
    // We match on the NEW (coloured) id per rule 4, capturing the colour index
    // by `move` for each of the 16 ids.
    for (color, &coloured_id) in SHULKER_ID_BY_COLOUR.iter().enumerate() {
        let color = color as i32;
        reg.item_stack.add_reverse_converter_for_id(
            coloured_id,
            VERSION,
            0,
            Box::new(move |data, _from, _to| {
                data.set_string("id", "minecraft:shulker_box");

                // tag.getOrCreate -> BlockEntityTag.getOrCreate -> set Color.
                if data.get_map("tag").is_none() {
                    data.set_map("tag", NbtMap::new());
                }
                let tag = data.get_map_mut("tag").unwrap();
                if tag.get_map("BlockEntityTag").is_none() {
                    tag.set_map("BlockEntityTag", NbtMap::new());
                }
                let block_entity = tag.get_map_mut("BlockEntityTag").unwrap();
                block_entity.set_i32("Color", color);
            }),
        );
    }

    reg.tile_entity.add_converter_for_id(
        "minecraft:shulker_box",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.take("Color");
        }),
    );

    // Reverse of the TILE_ENTITY converter (new -> old). The forward simply
    // dropped the `Color` field from the shulker-box tile entity. At V813
    // (16w40a, pre-flattening) the block/tile id is the plain
    // `minecraft:shulker_box` regardless of colour, so nothing in the modern
    // data preserves what `Color` used to be — the discriminator was genuinely
    // erased (rule 11 lossy, NOT a recoverable split). Best-effort: restore the
    // canonical default `Color=0` (white) so the legacy field exists, and
    // report the unavoidable loss.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:shulker_box",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            report_loss(
                VERSION,
                LossKind::FingerprintCollapse,
                Severity::Approximated,
                "shulker_box tile entity Color was dropped forward with no surviving \
                 discriminator; restoring default Color=0 (white)",
            );
            data.set_i32("Color", 0);
        }),
    );
}
