//! V3097 (22w19a + 1) — schematic-relevant subset of `V3097.java`.
//!
//! Kept:
//!   * ITEM_STACK `minecraft:writable_book` / `minecraft:written_book`: drop the
//!     now-unused `tag.filtered_title` and `tag.filtered_pages` fields.
//!   * TILE_ENTITY `minecraft:sign`: drop `FilteredText1..4`.
//!   * ENTITY `minecraft:cat`: `ConverterEntityVariantRename` of the string
//!     `variant` field — `minecraft:british` -> `minecraft:british_shorthair`.
//!
//! Skipped (non-schematic): the ADVANCEMENTS criteria rename
//! (`husbandry/complete_catalogue`) and the POI_CHUNK deletion of
//! `unemployed` / `nitwit` — ADVANCEMENTS and POI never appear in a schematic.
//!
//! VERSION = MCVersions.V22W19A (3096) + 1 = 3097.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 3097;

pub fn register(reg: &mut RegistryBuilder) {
    // Remove the filtered book text from writable/written books.
    let remove_filtered_book_text = |data: &mut crate::nbt::NbtMap, _from, _to| {
        if let Some(tag) = data.get_map_mut("tag") {
            tag.take("filtered_title");
            tag.take("filtered_pages");
        }
    };
    reg.item_stack.add_converter_for_id(
        "minecraft:writable_book",
        VERSION,
        0,
        Box::new(remove_filtered_book_text),
    );
    reg.item_stack.add_converter_for_id(
        "minecraft:written_book",
        VERSION,
        0,
        Box::new(remove_filtered_book_text),
    );

    // Signs: drop the per-line filtered text.
    reg.tile_entity.add_converter_for_id(
        "minecraft:sign",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.take("FilteredText1");
            data.take("FilteredText2");
            data.take("FilteredText3");
            data.take("FilteredText4");
        }),
    );

    // Cat variant rename: british -> british_shorthair (ConverterEntityVariantRename
    // reads the string `variant` and renames in place; no-op if absent/unmapped).
    reg.entity.add_converter_for_id(
        "minecraft:cat",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_string("variant") == Some("minecraft:british") {
                data.set_string("variant", "minecraft:british_shorthair");
            }
        }),
    );
    // Reverse: british_shorthair -> british. The forward maps the single value
    // `minecraft:british` to `minecraft:british_shorthair`; the new value uniquely
    // encodes the old one, so this is an exact, lossless inverse (bucket A).
    reg.entity.add_reverse_converter_for_id(
        "minecraft:cat",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_string("variant") == Some("minecraft:british_shorthair") {
                data.set_string("variant", "minecraft:british");
            }
        }),
    );

    // No reverse for the book/sign filtered-text deletions: the forward removes a
    // deprecated feature's fields (`tag.filtered_title`/`filtered_pages` on books,
    // `FilteredText1..4` on signs). Modern (post-3097) data never carries these
    // fields, so there is nothing to restore -- the inverse is identity (rule 10).
}
