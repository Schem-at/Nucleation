//! V1488 (18w19b+3) — schematic-relevant subset of `V1488.java`.
//!
//! VERSION = MCVersions.V18W19B + 3 = 1485 + 3 = 1488.
//!
//! Ported:
//!   * BLOCK_NAME / BLOCK_STATE / FLAT_BLOCK_STATE rename `kelp_top` -> `kelp`,
//!     `kelp` -> `kelp_plant` (via `register_block_rename`).
//!   * ITEM_NAME rename `kelp_top` -> `kelp`.
//!   * TILE_ENTITY `minecraft:command_block` converter: `updateCustomName`
//!     (the V1458 plain-string -> text-component migration, run here only for
//!     command blocks since V1458 deliberately skipped them).
//!   * ENTITY `minecraft:commandblock_minecart` converter: `updateCustomName`.
//!   * TILE_ENTITY `minecraft:command_block` walker for TEXT_COMPONENT paths
//!     `CustomName` + `LastOutput`.
//!
//! Skipped:
//!   * The STRUCTURE_FEATURE structure converter (igloo `Children` collapse) —
//!     STRUCTURE_FEATURE is non-schematic and is not in the registry.
//!   * The TILE_ENTITY `command_block` walker targeting
//!     `DATACONVERTER_CUSTOM_TYPE_COMMAND` ("Command") — that custom command type
//!     is not a schematic type and is not present in the restricted registry; it
//!     only rewrites the command string and has no nested schematic data.

use std::sync::Arc;

use crate::nbt::NbtMap;

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::convert;

const VERSION: i32 = 1488;

/// Block renames (V1488.java:25-30). Note: `kelp` is both a target (from
/// `kelp_top`) and a source (to `kelp_plant`); `map_renamer` does a single-pass
/// HashMap lookup, matching Java's `HashMap::get` semantics (no chaining).
const BLOCK_RENAMES: &[(&str, &str)] = &[
    ("minecraft:kelp_top", "minecraft:kelp"),
    ("minecraft:kelp", "minecraft:kelp_plant"),
];

/// Item rename (V1488.java:31-35).
const ITEM_RENAMES: &[(&str, &str)] = &[("minecraft:kelp_top", "minecraft:kelp")];

/// `V1458.updateCustomName` (V1458.java:16-26): a non-empty plain-string
/// `CustomName` becomes a JSON text component `{"text":"<name>"}`; an
/// empty/absent one is removed.
pub(super) fn update_custom_name(data: &mut NbtMap) {
    let name = data.get_string("CustomName").unwrap_or("").to_string();
    if name.is_empty() {
        data.take("CustomName");
    } else {
        data.set_string("CustomName", create_plain_text_component(&name));
    }
}

/// `ComponentUtils.createPlainTextComponent`: a stable-ordered JSON object with a
/// single `text` property, i.e. `{"text":"<escaped>"}`.
pub(super) fn create_plain_text_component(text: &str) -> String {
    format!("{{\"text\":{}}}", json_string(text))
}

/// Minimal JSON string escaping (mirrors Gson's default string serialization for
/// the characters that appear in custom names).
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Inverse of `update_custom_name`.
///
/// Forward turned a non-empty plain-string `CustomName` into the JSON text
/// component `{"text":"<name>"}` and dropped empty/absent ones. The reverse
/// restores the legacy plain string:
/// - a plain-text-only component (`{"text": s}` with no other keys, the exact
///   shape the forward emits) → restore the raw string `s`. This is the exact
///   inverse of the wrap path, so no loss.
/// - the forward's "empty/absent → removed" branch leaves nothing to restore: an
///   absent `CustomName` was also absent in the legacy schema, so there is
///   nothing to invert (lossless for real downgrades).
/// - a richer component the user authored at a newer version (translate/format
///   keys, or `{"text":...}` with siblings) has no plain-string preimage; the
///   legacy field was a raw string only. Best-effort: collapse to the `text`
///   value if present (else leave as-is) and report the dropped formatting.
fn revert_custom_name(data: &mut NbtMap) {
    let text = match data.get_string("CustomName").map(str::to_string) {
        Some(t) => t,
        None => return,
    };
    // Forward only ever emits valid JSON; non-JSON is foreign — leave it.
    let parsed: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return,
    };
    if let serde_json::Value::Object(obj) = &parsed {
        if let Some(serde_json::Value::String(s)) = obj.get("text") {
            if obj.len() == 1 {
                // Exact inverse of the plain-text wrap path.
                data.set_string("CustomName", s.clone());
                return;
            }
            // {"text":...} with sibling keys (color, bold, …): the legacy field
            // held only a raw string, so the extra formatting can't be encoded.
            report_loss(
                VERSION,
                LossKind::ComponentDropped,
                Severity::Approximated,
                "command-block CustomName: dropped text-component formatting, kept plain text",
            );
            data.set_string("CustomName", s.clone());
            return;
        }
        // A component with no `text` (e.g. translate) has no plain-string preimage.
        report_loss(
            VERSION,
            LossKind::ComponentDropped,
            Severity::Loss,
            "command-block CustomName: non-text component has no legacy plain-string form",
        );
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    register_block_rename(reg, VERSION, map_renamer(BLOCK_RENAMES));
    register_item_rename(reg, VERSION, map_renamer(ITEM_RENAMES));

    // V1458 wrote its CustomName converter to skip command blocks; this version
    // applies it ONLY to command blocks (TILE_ENTITY) and command-block
    // minecarts (ENTITY).
    reg.tile_entity.add_converter_for_id(
        "minecraft:command_block",
        VERSION,
        0,
        Box::new(|data, _from, _to| update_custom_name(data)),
    );
    // Reverse: unwrap the {"text":…} component back to a legacy plain string. No
    // id rename here, so the reverse matches the same id as the forward.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:command_block",
        VERSION,
        0,
        Box::new(|data, _from, _to| revert_custom_name(data)),
    );

    reg.entity.add_converter_for_id(
        "minecraft:commandblock_minecart",
        VERSION,
        0,
        Box::new(|data, _from, _to| update_custom_name(data)),
    );
    reg.entity.add_reverse_converter_for_id(
        "minecraft:commandblock_minecart",
        VERSION,
        0,
        Box::new(|data, _from, _to| revert_custom_name(data)),
    );

    // command_block walker: CustomName + LastOutput are TEXT_COMPONENTs.
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:command_block",
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.text_component, data, "CustomName", from, to);
            convert(reg, &reg.text_component, data, "LastOutput", from, to);
        }),
    );
}
