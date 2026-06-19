//! V165 (MCVersions.V1_9_PRE2 = 165) — schematic-relevant subset of V165.java.
//!
//! - ITEM_STACK structure converter: for items with id minecraft:written_book,
//!   rewrites each STRING entry of tag.pages from lenient legacy JSON into a
//!   strict JSON text component via ComponentUtils.convertFromLenient.
//!   Mirrors Java's null-guards: bail if not a written_book, if tag is absent,
//!   or if pages is absent. Only String list entries are rewritten (Java reads
//!   pages as a list of ObjectType.STRING).

use crate::nbt::NbtValue;

use super::super::helpers::convert_from_lenient;
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 165;

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // Java: bail unless id is exactly minecraft:written_book.
            if data.get_string("id") != Some("minecraft:written_book") {
                return;
            }

            // Java: getMap("tag") returns null when absent -> no change.
            let tag = match data.get_map_mut("tag") {
                Some(tag) => tag,
                None => return,
            };

            // Java: getList("pages", STRING) returns null when absent -> no change.
            let pages = match tag.get_list_mut("pages") {
                Some(pages) => pages,
                None => return,
            };

            for page in pages.iter_mut() {
                if let NbtValue::String(s) = page {
                    *s = convert_from_lenient(s);
                }
            }
        }),
    );

    // Reverse: turn each JSON text-component page back into a legacy lenient string.
    //
    // This is the page-list analogue of V101's sign-line inverse. Forward wraps a
    // legacy lenient page string into a JSON text component; for ordinary book text
    // that is `{"text":"<page>"}`. The legacy written_book format stored raw lenient
    // strings. So, per page:
    // - a plain-text-only component (`{"text": s}` with no other keys) → restore the
    //   raw string `s` (exact inverse of the plain-text wrap path);
    // - any other valid JSON component (translate/formatted/etc.) is left as its JSON
    //   string: the legacy lenient parser accepted JSON delimiters verbatim, so it
    //   round-trips and is not lossy for real downgrades;
    // - a non-JSON entry is foreign data (forward never emits non-JSON) — leave it.
    // The only genuine loss: forward collapsed both ""/"null" inputs into
    // {"text":""}, so when s is empty the original discriminator is unrecoverable —
    // report it once, best-effort emit "" (the dominant preimage).
    reg.item_stack.add_reverse_converter_for_id(
        "minecraft:written_book",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let tag = match data.get_map_mut("tag") {
                Some(tag) => tag,
                None => return,
            };
            let pages = match tag.get_list_mut("pages") {
                Some(pages) => pages,
                None => return,
            };

            for page in pages.iter_mut() {
                let text = match page {
                    NbtValue::String(s) => s,
                    _ => continue,
                };
                let parsed: serde_json::Value = match serde_json::from_str(text) {
                    Ok(v) => v,
                    // Not JSON: forward never emits non-JSON, so this is foreign data.
                    Err(_) => continue,
                };
                if let serde_json::Value::Object(obj) = &parsed {
                    if obj.len() == 1 {
                        if let Some(serde_json::Value::String(s)) = obj.get("text") {
                            // A single-key {"text": "..."} page could have come
                            // from a raw legacy string or from a legacy string
                            // that was already a JSON component. Choosing raw
                            // text is the useful preimage, but the fingerprint
                            // is collapsed for every such page.
                            report_loss(
                                VERSION,
                                LossKind::FingerprintCollapse,
                                Severity::Approximated,
                                "written_book page: single text component could be raw text or an original JSON component; restoring raw text",
                            );
                            *text = s.clone();
                        }
                    }
                }
            }
        }),
    );
}
