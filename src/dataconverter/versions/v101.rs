//! V101 (MCVersions.V15W32A + 1 = 101) — schematic-relevant subset of V101.java.
//!
//! - TILE_ENTITY "Sign": rewrites Text1/Text2/Text3/Text4 from lenient legacy
//!   strings into proper JSON text components via ComponentUtils.convertFromLenient
//!   (only when the line is present, matching Java's null-guard).
//! - ENTITY "Villager": sets CanPickUpLoot = true.

use crate::nbt::NbtMap;

use super::super::helpers::convert_from_lenient;
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 101;

fn update_line(data: &mut NbtMap, path: &str) {
    // Java: getString returns null when absent; only rewrite present lines.
    let text = data.get_string(path).map(str::to_string);
    if let Some(text) = text {
        data.set_string(path, convert_from_lenient(&text));
    }
}

/// Inverse of `update_line` / `convert_from_lenient` for one sign line.
///
/// Forward wraps a legacy lenient string into a JSON text component; for ordinary
/// sign text that is `{"text":"<line>"}`. The legacy format stored a raw lenient
/// string. So:
/// - a plain-text-only component (`{"text": s}` with no other keys) → restore the
///   raw string `s` (exact inverse of the plain-text wrap path);
/// - any other valid JSON component (translate/formatted/empty `{"text":""}` which
///   the forward also used for empty/"null") is left as its JSON string: the legacy
///   lenient parser accepted JSON delimiters verbatim, so this round-trips and is
///   not lossy for real downgrades. The only genuine loss is that the original
///   distinction between empty-string and the literal "null" sentinel is gone —
///   report it once, best-effort emit "" (the dominant preimage).
fn revert_line(data: &mut NbtMap, path: &str) {
    let text = match data.get_string(path).map(str::to_string) {
        Some(t) => t,
        None => return,
    };
    let parsed: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        // Not JSON: forward never emits non-JSON, so this is foreign data — leave it.
        Err(_) => return,
    };
    if let serde_json::Value::Object(obj) = &parsed {
        if let Some(serde_json::Value::String(s)) = obj.get("text") {
            if obj.len() == 1 {
                if s.is_empty() {
                    // Forward collapsed both ""/"null" inputs into {"text":""}; the
                    // legacy discriminator is unrecoverable. Restore "".
                    report_loss(
                        VERSION,
                        LossKind::FingerprintCollapse,
                        Severity::Approximated,
                        "sign line: empty/\"null\" legacy strings both became {\"text\":\"\"}; restoring \"\"",
                    );
                }
                data.set_string(path, s);
            }
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_converter_for_id(
        "Sign",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            update_line(data, "Text1");
            update_line(data, "Text2");
            update_line(data, "Text3");
            update_line(data, "Text4");
        }),
    );

    // Reverse: turn the JSON text components back into legacy lenient strings.
    reg.tile_entity.add_reverse_converter_for_id(
        "Sign",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            revert_line(data, "Text1");
            revert_line(data, "Text2");
            revert_line(data, "Text3");
            revert_line(data, "Text4");
        }),
    );

    reg.entity.add_converter_for_id(
        "Villager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.set_bool("CanPickUpLoot", true);
        }),
    );

    // Reverse: CanPickUpLoot is an additive default (forward always set it true).
    // Drop it only when it is exactly the value the forward wrote, so we never
    // clobber a meaningful pre-existing flag carried by genuinely old data.
    reg.entity.add_reverse_converter_for_id(
        "Villager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_bool("CanPickUpLoot") == Some(true) {
                data.remove("CanPickUpLoot");
            }
        }),
    );
}
