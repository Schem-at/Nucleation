//! V3564 (1.20.2, `V1_20_1 + 99` = 3564) — schematic-relevant subset of
//! `V3564.java`.
//!
//! Ported: the TILE_ENTITY converter registered for both `minecraft:sign` and
//! `minecraft:hanging_sign` that cleans up the modern sign text format. For each
//! of `front_text` / `back_text`:
//!   * if `_filtered_correct` is true, just drop that marker and stop;
//!   * otherwise, reconcile `filtered_messages` against `messages`, replacing the
//!     "empty component" sentinel with the corresponding non-filtered message,
//!     and dropping `filtered_messages` entirely if the result is all-empty.
//! Then the legacy fields (`Text1..4`, `FilteredText1..4`, `Color`,
//! `GlowingText`) are removed from the root.
//!
//! Nothing here targets a non-schematic type (sign/hanging_sign are block
//! entities), so the whole file is ported.
//!
//! Implemented inline with NbtMap/NbtValue because the logic (sentinel-aware
//! list reconciliation) is not expressible with the rename/walker helpers.
//! `ComponentUtils.EMPTY` is the stable JSON text component for the empty
//! string, i.e. `{"text":""}`.

use crate::nbt::{NbtMap, NbtValue};

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::version::EncodedVersion;

const VERSION: i32 = 3564;

/// `ComponentUtils.EMPTY` — the stable-serialized empty text component.
const EMPTY_COMPONENT: &str = "{\"text\":\"\"}";

const LEGACY_FIELDS: &[&str] = &[
    "Text1",
    "Text2",
    "Text3",
    "Text4",
    "FilteredText0",
    "FilteredText1",
    "FilteredText2",
    "FilteredText3",
    "Color",
    "GlowingText",
];

/// `updateText` (V3564.java:35-73).
fn update_text(text: &mut NbtMap) {
    // _filtered_correct => drop the marker and return.
    if text.get_bool("_filtered_correct").unwrap_or(false) {
        text.take("_filtered_correct");
        return;
    }

    // Read filtered_messages (list of strings); null/empty -> nothing to do.
    let filtered: Vec<String> = match text.get_list("filtered_messages") {
        Some(list) if !list.is_empty() => list
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect(),
        _ => return,
    };

    // messages list (treat null as empty).
    let messages: Vec<String> = text
        .get_list("messages")
        .map(|list| {
            list.iter()
                .map(|v| v.as_str().unwrap_or("").to_string())
                .collect()
        })
        .unwrap_or_default();

    let mut new_filtered: Vec<NbtValue> = Vec::with_capacity(filtered.len());
    let mut new_filtered_is_empty = true;

    for (i, f) in filtered.iter().enumerate() {
        let message = messages
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or(EMPTY_COMPONENT);
        let new_filtered_str = if f == EMPTY_COMPONENT {
            message
        } else {
            f.as_str()
        };
        new_filtered.push(NbtValue::String(new_filtered_str.to_string()));
        new_filtered_is_empty = new_filtered_is_empty && new_filtered_str == EMPTY_COMPONENT;
    }

    if new_filtered_is_empty {
        text.take("filtered_messages");
    } else {
        text.set_list("filtered_messages", new_filtered);
    }
}

/// The shared `minecraft:sign` / `minecraft:hanging_sign` converter
/// (V3564.java:75-85): clean both text faces, then strip the legacy fields.
fn convert_sign(data: &mut NbtMap, _from: EncodedVersion, _to: EncodedVersion) {
    if let Some(front) = data.get_map_mut("front_text") {
        update_text(front);
    }
    if let Some(back) = data.get_map_mut("back_text") {
        update_text(back);
    }
    for to_remove in LEGACY_FIELDS {
        data.take(to_remove);
    }
}

/// Inverse of `convert_sign`, restoring the version-3563 sign schema.
///
/// The forward does two things; both invert losslessly here because of how the
/// 3563 schema is actually produced (by V3439, see v3439.rs):
///
///   * **`_filtered_correct` drop (`update_text`).** V3439's forward always sets
///     `front_text._filtered_correct = true` and never sets it on `back_text`.
///     So at version 3563 the marker is *always* present on `front_text` and
///     *always* absent on `back_text`; V3564's `update_text` then strips it from
///     `front_text` (taking the `_filtered_correct` branch). Re-adding
///     `_filtered_correct = true` to `front_text` is therefore the exact
///     restoration of an invariant the old schema always carried (cheatsheet
///     rule 11), not a guess — no loss. The `filtered_messages` reconciliation
///     branch is never reached for real 3563 data (front takes the
///     `_filtered_correct` branch; back has no `filtered_messages`), and the
///     reconciled list is itself valid 3563 data, so its inverse is identity.
///
///   * **Legacy-field removal.** The legacy keys (`Text1..4`, `FilteredText1..4`,
///     `Color`, `GlowingText`) were left *present* at 3563 by V3439 and only
///     stripped by V3564. They are not reconstructed here: V3564 has no mapping
///     from `front_text` back to them. That reconstruction is owned by V3439's
///     reverse (`sign_tile_reverter`), which rebuilds each legacy key from
///     `front_text` *only when it is absent*. Re-adding them here would both be
///     value-less (no source data) and actively block V3439's reconstruction, so
///     this part of the inverse is intentionally identity.
fn reverse_sign(data: &mut NbtMap, _from: EncodedVersion, _to: EncodedVersion) {
    if let Some(front) = data.get_map_mut("front_text") {
        front.set_bool("_filtered_correct", true);
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity
        .add_converter_for_id("minecraft:sign", VERSION, 0, Box::new(convert_sign));
    reg.tile_entity.add_converter_for_id(
        "minecraft:hanging_sign",
        VERSION,
        0,
        Box::new(convert_sign),
    );

    // Reverse: restore `front_text._filtered_correct = true` (V3439's invariant
    // that V3564 stripped). Legacy fields are left to V3439's reverse to rebuild.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:sign",
        VERSION,
        0,
        Box::new(reverse_sign),
    );
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:hanging_sign",
        VERSION,
        0,
        Box::new(reverse_sign),
    );
}
