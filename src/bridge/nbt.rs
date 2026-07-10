//! NBT helpers (chest / sign / text SNBT builders). Port of the `nbt_*` fns from
//! `ffi/mod.rs`. Output targets the modern (1.20+) Minecraft NBT schemas.

use crate::bridge::shared::ffi::NucleationError;

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn snbt_string(s: &str) -> String {
    format!("\"{}\"", json_escape(s))
}

fn build_text_json(s: &str, color: Option<&str>, bold: i32, italic: i32) -> String {
    let mut parts = vec![format!("\"text\":\"{}\"", json_escape(s))];
    if let Some(c) = color {
        parts.push(format!("\"color\":\"{}\"", json_escape(c)));
    }
    if bold >= 0 {
        parts.push(format!("\"bold\":{}", bold != 0));
    }
    if italic >= 0 {
        parts.push(format!("\"italic\":{}", italic != 0));
    }
    format!("{{{}}}", parts.join(","))
}

/// A single inventory item for `Nbt::chest_build` (the old `CItem`).
#[derive(serde::Deserialize)]
struct ChestItem {
    id: String,
    #[serde(default)]
    count: Option<i32>,
    #[serde(default)]
    slot: Option<i32>,
}

/// Normalize up to 4 sign lines: plain strings are wrapped as JSON text
/// components, strings starting with `{` pass through as-is, empty strings
/// stay empty; the list is padded to 4 lines.
fn collect_sign_lines(lines: Vec<String>) -> Result<Vec<String>, NucleationError> {
    if lines.len() > 4 {
        return Err(NucleationError::InvalidArgument);
    }
    let mut out = Vec::with_capacity(4);
    for s in lines {
        if s.is_empty() {
            out.push(String::from("\"\""));
        } else if s.starts_with('{') {
            out.push(s);
        } else {
            out.push(build_text_json(&s, None, -1, -1));
        }
    }
    while out.len() < 4 {
        out.push(String::from("\"\""));
    }
    Ok(out)
}

#[diplomat::bridge]
pub mod ffi {
    use super::super::shared::ffi::NucleationError;
    use super::{build_text_json, collect_sign_lines, json_escape, snbt_string, ChestItem};
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Namespace type for the free-standing NBT builder helpers (the old
    /// `nbt_text_build` / `nbt_chest_build` / `nbt_sign_build`), following the
    /// static-methods-on-a-dummy-opaque pattern.
    #[diplomat::opaque]
    pub struct Nbt;

    impl Nbt {
        /// Build a Minecraft JSON text-component string.
        ///
        /// `color` may be empty (no color). `bold` and `italic` use `-1` for
        /// unset, `0` for false, nonzero for true.
        pub fn text_build(
            s: &DiplomatStr,
            color: &DiplomatStr,
            bold: i32,
            italic: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let text =
                std::str::from_utf8(s).map_err(|_| NucleationError::InvalidArgument)?;
            let color =
                std::str::from_utf8(color).map_err(|_| NucleationError::InvalidArgument)?;
            let color = if color.is_empty() { None } else { Some(color) };
            let _ = write!(out, "{}", build_text_json(text, color, bold, italic));
            Ok(())
        }

        /// Build a chest-NBT SNBT string for use as the `{...}` portion of a block
        /// string.
        ///
        /// `items_json` is a JSON array of `{"id": string, "count"?: int,
        /// "slot"?: int}` entries (may be empty or `[]`); a missing/non-positive
        /// `count` defaults to 1, a missing/negative `slot` auto-assigns
        /// positionally. `name` is an optional plain-text custom name (empty = no
        /// `CustomName`); it is wrapped in a JSON text component automatically
        /// unless it already starts with `{`.
        pub fn chest_build(
            items_json: &DiplomatStr,
            name: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let items_str =
                std::str::from_utf8(items_json).map_err(|_| NucleationError::InvalidArgument)?;
            let name =
                std::str::from_utf8(name).map_err(|_| NucleationError::InvalidArgument)?;
            let items: Vec<ChestItem> = if items_str.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(items_str).map_err(|_| NucleationError::Parse)?
            };

            let mut entries = Vec::with_capacity(items.len());
            for (i, it) in items.iter().enumerate() {
                let count = match it.count {
                    Some(c) if c > 0 => c,
                    _ => 1,
                };
                let slot: i32 = match it.slot {
                    Some(s) if s >= 0 => s,
                    _ => i as i32,
                };
                entries.push(format!(
                    "{{Slot:{}b,id:\"{}\",Count:{}b}}",
                    slot,
                    json_escape(&it.id),
                    count
                ));
            }

            let mut parts = vec![format!("Items:[{}]", entries.join(","))];
            if !name.is_empty() {
                let inner = if name.starts_with('{') {
                    name.to_string()
                } else {
                    build_text_json(name, None, -1, -1)
                };
                // CustomName is stored as a string holding JSON.
                parts.push(format!("CustomName:{}", snbt_string(&inner)));
            }
            let _ = write!(out, "{{{}}}", parts.join(","));
            Ok(())
        }

        /// Build a modern (1.20+) sign-NBT SNBT string.
        ///
        /// `front_json` and `back_json` are JSON arrays of up to 4 line strings
        /// (either may be empty or `[]`). Each line may be a plain string
        /// (auto-wrapped via `text_build`) or an already-built JSON component
        /// (starts with `{`). `color` is the dye color string (empty defaults to
        /// `"black"`).
        pub fn sign_build(
            front_json: &DiplomatStr,
            back_json: &DiplomatStr,
            color: &DiplomatStr,
            glowing: bool,
            waxed: bool,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            fn parse_lines(json: &[u8]) -> Result<Vec<String>, NucleationError> {
                let s =
                    std::str::from_utf8(json).map_err(|_| NucleationError::InvalidArgument)?;
                if s.is_empty() {
                    return Ok(Vec::new());
                }
                serde_json::from_str(s).map_err(|_| NucleationError::Parse)
            }
            let front_msgs = collect_sign_lines(parse_lines(front_json)?)?;
            let back_msgs = collect_sign_lines(parse_lines(back_json)?)?;

            let color =
                std::str::from_utf8(color).map_err(|_| NucleationError::InvalidArgument)?;
            let color = if color.is_empty() { "black" } else { color };
            let g = if glowing { "1b" } else { "0b" };
            let w = if waxed { "1b" } else { "0b" };

            let messages = |msgs: &[String]| -> String {
                // Each message must be stored as a *string* containing JSON.
                let quoted: Vec<String> = msgs.iter().map(|m| snbt_string(m)).collect();
                format!("[{}]", quoted.join(","))
            };

            let _ = write!(
                out,
                "{{front_text:{{messages:{},color:\"{}\",has_glowing_text:{}}},back_text:{{messages:{},color:\"{}\",has_glowing_text:{}}},is_waxed:{}}}",
                messages(&front_msgs),
                json_escape(color),
                g,
                messages(&back_msgs),
                json_escape(color),
                g,
                w,
            );
            Ok(())
        }
    }
}
