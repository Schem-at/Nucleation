//! V1803 (1.13.2 + 172) — schematic-relevant subset of `V1803.java`.
//!
//! VERSION = MCVersions.V1_13_2 (1631) + 172 = 1803.
//!
//! Ported (the entire version is schematic-relevant): the ITEM_STACK structure
//! converter that wraps each plain string in `tag.display.Lore` into a
//! plain-text JSON text component (V1803.java:16-42).
//!
//! No helper exists for `ComponentUtils.createPlainTextComponent`, so it is
//! implemented inline. Java builds `{"text": <value>}` with Gson and serialises
//! via `GsonHelper.toStableString`, whose `JsonWriter` is HTML-safe by default.
//! `create_plain_text_component` reproduces Gson's exact HTML-safe string
//! escaping (escaping `<`, `>`, `&`, `=`, `'`, the control set, and the line/para
//! separators U+2028/U+2029) so the produced string is byte-for-byte identical to
//! vanilla's. Since the object has a single key, the stable key sort is a no-op.

use crate::nbt::NbtValue;

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 1803;

/// `ComponentUtils.createPlainTextComponent(text)` -> `GsonHelper.toStableString`
/// of `{"text": text}`. Reproduces Gson's default (HTML-safe) `JsonWriter` string
/// escaping exactly.
fn create_plain_text_component(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 12);
    out.push_str("{\"text\":");
    write_gson_string(&mut out, text);
    out.push('}');
    out
}

/// Port of Gson `JsonWriter.string(value)` with `htmlSafe = true` (the default).
fn write_gson_string(out: &mut String, value: &str) {
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{000C}' => out.push_str("\\f"),
            // HTML-safe escapes (Gson's default JsonWriter).
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '=' => out.push_str("\\u003d"),
            '\'' => out.push_str("\\u0027"),
            // Line/paragraph separators, which Gson always escapes.
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            // Remaining control characters below 0x20.
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let display = match data
                .get_map_mut("tag")
                .and_then(|tag| tag.get_map_mut("display"))
            {
                Some(d) => d,
                None => return,
            };

            // Java fetches the Lore list filtered to STRING entries; non-string
            // entries are skipped (left untouched).
            let lore = match display.get_list_mut("Lore") {
                Some(l) => l,
                None => return,
            };

            for el in lore.iter_mut() {
                if let Some(s) = el.as_str() {
                    let wrapped = create_plain_text_component(s);
                    *el = NbtValue::String(wrapped);
                }
            }
        }),
    );

    // Reverse of the ITEM_STACK converter: unwrap each `{"text": s}` plain-text
    // component in `tag.display.Lore` back to its raw legacy string `s`.
    //
    // The forward (`create_plain_text_component`) only ever produced a single-key
    // `{"text": <raw string>}` from each legacy Lore string. That mapping is
    // injective — the `"text"` value is exactly the original raw string (even if
    // that string itself looked like JSON, it was stored verbatim as the string
    // value) — so this unwrap is an exact, lossless inverse for any real downgrade
    // (rule 11). Non-string entries (the forward left them untouched) and any
    // value that is not a single-key `{"text":…}` string component are foreign to
    // the forward output and are left as-is.
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let lore = match data
                .get_map_mut("tag")
                .and_then(|tag| tag.get_map_mut("display"))
                .and_then(|display| display.get_list_mut("Lore"))
            {
                Some(l) => l,
                None => return,
            };

            for el in lore.iter_mut() {
                let Some(text) = el.as_str() else { continue };
                let parsed: serde_json::Value = match serde_json::from_str(text) {
                    Ok(v) => v,
                    // Forward only ever emits JSON here; non-JSON is foreign — leave it.
                    Err(_) => continue,
                };
                if let serde_json::Value::Object(obj) = &parsed {
                    if obj.len() == 1 {
                        if let Some(serde_json::Value::String(s)) = obj.get("text") {
                            *el = NbtValue::String(s.clone());
                        }
                    }
                }
            }
        }),
    );
}
