//! V4292 (1.21.4 + 103) — schematic-relevant subset of `V4292.java`.
//!
//! VERSION = MCVersions.V1_21_4 (4189) + 103 = 4292.
//!
//! Ported (TEXT_COMPONENT structure converter + walker, V4292.java:647-804). The
//! converter restructures `hoverEvent`/`clickEvent` into the flattened
//! `hover_event`/`click_event` schema used from 1.21.5:
//!   * `hoverEvent` -> `hover_event`, then by action:
//!       - `show_text`   : `contents` -> `value`
//!       - `show_item`   : a bare-string `contents` -> `id`; otherwise the
//!                         `contents` map is dissolved, hoisting `id`/`count`/
//!                         `components` up to the event
//!       - `show_entity` : the `contents` map is dissolved, mapping
//!                         `id`->`uuid`, `type`->`id`, `name`->`name`
//!   * `clickEvent` -> `click_event`, then by action:
//!       - `open_url`     : drop the event unless the value is an http(s) URL,
//!                          else `value` -> `url`
//!       - `open_file`    : `value` -> `path`
//!       - `run_command` / `suggest_command` : drop the event unless the value is
//!                          a valid command/chat string, else `value` -> `command`
//!       - `change_page`  : parse the value as an int (drop the event if it is
//!                          not), then store `page = max(1, value)`
//!
//! The walker descends `extra` / `separator` and routes the (now flattened)
//! `hover_event` sub-reads through TEXT_COMPONENT / ITEM_STACK / ENTITY_NAME.
//!
//! Skipped: the `clickEvent` `command` descent into
//! DATACONVERTER_CUSTOM_TYPE_COMMAND (a non-schematic custom command type absent
//! from the restricted registry; consistent with the v1488/v4290 skip).

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::walker::{convert, convert_list, convert_value};

const VERSION: i32 = 4292;

/// `CopyHelper.move(src, src_key, dst, dst_key)`: move `src[src_key]` to
/// `dst[dst_key]`; if `src[src_key]` is absent, remove `dst[dst_key]`.
fn move_key(src: &mut NbtMap, src_key: &str, dst: &mut NbtMap, dst_key: &str) {
    match src.take(src_key) {
        Some(v) => dst.set_generic(dst_key, v),
        None => {
            dst.take(dst_key);
        }
    }
}

/// `URI(value).getScheme()` is `http`/`https` (case-insensitive). A value with no
/// scheme (or that fails to parse) is not a web URL.
fn is_web_url(value: &str) -> bool {
    // A scheme is the prefix before the first ':' and must be a valid RFC 3986
    // scheme (ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )). We only care whether it
    // is exactly http/https (case-insensitive).
    let scheme = match value.find(':') {
        Some(i) => &value[..i],
        None => return false,
    };
    if scheme.is_empty() {
        return false;
    }
    let mut chars = scheme.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() {
        return false;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')) {
        return false;
    }
    scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
}

/// `isValidCommandOrChat`: rejects control characters (< 0x20), DEL (127), and
/// the legacy section sign (167).
fn is_valid_command_or_chat(value: &str) -> bool {
    !value.chars().any(|c| {
        let n = c as u32;
        n < 0x20 || n == 127 || n == 167
    })
}

/// `parseInteger(root, path)`: a string value is parsed as an int (None on
/// failure); a numeric value is narrowed to i32; anything else is None.
fn parse_integer(root: &NbtMap, path: &str) -> Option<i32> {
    match root.get(path) {
        Some(NbtValue::String(s)) => s.parse::<i32>().ok(),
        Some(v) => v.as_number_i64().map(|n| n as i32),
        None => None,
    }
}

fn convert_component(root: &mut NbtMap) {
    // --- hoverEvent -> hover_event ----------------------------------------
    root.rename_key("hoverEvent", "hover_event");
    if let Some(hover_event) = root.get_map_mut("hover_event") {
        match hover_event
            .get_string("action")
            .unwrap_or("")
            .to_string()
            .as_str()
        {
            "show_text" => {
                hover_event.rename_key("contents", "value");
            }
            "show_item" => {
                let contents_is_string =
                    matches!(hover_event.get("contents"), Some(NbtValue::String(_)));
                if contents_is_string {
                    hover_event.rename_key("contents", "id");
                } else {
                    // Dissolve the contents map, hoisting id/count/components.
                    let mut contents = match hover_event.take("contents") {
                        Some(NbtValue::Compound(m)) => m,
                        _ => NbtMap::new(),
                    };
                    move_key(&mut contents, "id", hover_event, "id");
                    move_key(&mut contents, "count", hover_event, "count");
                    move_key(&mut contents, "components", hover_event, "components");
                }
            }
            "show_entity" => {
                let mut contents = match hover_event.take("contents") {
                    Some(NbtValue::Compound(m)) => m,
                    _ => NbtMap::new(),
                };
                move_key(&mut contents, "id", hover_event, "uuid");
                move_key(&mut contents, "type", hover_event, "id");
                move_key(&mut contents, "name", hover_event, "name");
            }
            _ => {}
        }
    }

    // --- clickEvent -> click_event ----------------------------------------
    root.rename_key("clickEvent", "click_event");
    if let Some(click_event) = root.get_map_mut("click_event") {
        let action = click_event.get_string("action").unwrap_or("").to_string();
        let value = click_event.get_string("value").unwrap_or("").to_string();
        let mut drop_event = false;
        match action.as_str() {
            "open_url" => {
                if !is_web_url(&value) {
                    drop_event = true;
                } else {
                    click_event.rename_key("value", "url");
                }
            }
            "open_file" => {
                click_event.rename_key("value", "path");
            }
            "run_command" | "suggest_command" => {
                if !is_valid_command_or_chat(&value) {
                    drop_event = true;
                } else {
                    click_event.rename_key("value", "command");
                }
            }
            "change_page" => match parse_integer(click_event, "value") {
                None => drop_event = true,
                Some(page) => {
                    click_event.take("value");
                    click_event.set_i32("page", page.max(1));
                }
            },
            _ => {}
        }
        if drop_event {
            root.take("click_event");
        }
    }
}

/// Reverse of `move_key`: move `src[src_key]` to `dst[dst_key]` only when it is
/// present (do not synthesize a missing entry, so we don't fabricate keys the
/// old schema never carried).
fn move_key_if_present(src: &mut NbtMap, src_key: &str, dst: &mut NbtMap, dst_key: &str) {
    if let Some(v) = src.take(src_key) {
        dst.set_generic(dst_key, v);
    }
}

/// Inverse of [`convert_component`]: restore the pre-1.21.5 `hoverEvent` /
/// `clickEvent` schema from the flattened `hover_event` / `click_event` shape.
///
/// We undo the inner per-action restructure while the key still has its forward
/// (`hover_event` / `click_event`) name, then rename the key back, mirroring the
/// forward order in reverse.
fn convert_component_reverse(root: &mut NbtMap) {
    // --- hover_event -> hoverEvent ----------------------------------------
    if let Some(hover_event) = root.get_map_mut("hover_event") {
        match hover_event
            .get_string("action")
            .unwrap_or("")
            .to_string()
            .as_str()
        {
            "show_text" => {
                // forward: contents -> value. Lossless 1:1.
                hover_event.rename_key("value", "contents");
            }
            "show_item" => {
                // forward dissolved a `contents` *map* (hoisting id/count/
                // components) and renamed a bare-string `contents` to `id`.
                // We reconstruct the map form `contents: { id[, count][,
                // components] }`, which is a valid preimage: re-running the
                // forward on it yields the same flattened shape. The bare-string
                // variant collapses into the equivalent map form, so this is
                // lossless in meaning (rule 11 — no report).
                let mut contents = NbtMap::new();
                move_key_if_present(hover_event, "id", &mut contents, "id");
                move_key_if_present(hover_event, "count", &mut contents, "count");
                move_key_if_present(hover_event, "components", &mut contents, "components");
                if contents.len() == 1 && contents.has_key("id") {
                    report_loss(
                        VERSION,
                        LossKind::FingerprintCollapse,
                        Severity::Approximated,
                        "show_item hover_event with only id could have come from legacy bare-string contents or contents map; restored map form",
                    );
                }
                hover_event.set_map("contents", contents);
            }
            "show_entity" => {
                // forward dissolved `contents`, mapping id->uuid, type->id,
                // name->name. Rebuild the `contents` map. Lossless 1:1.
                let mut contents = NbtMap::new();
                move_key_if_present(hover_event, "uuid", &mut contents, "id");
                move_key_if_present(hover_event, "id", &mut contents, "type");
                move_key_if_present(hover_event, "name", &mut contents, "name");
                hover_event.set_map("contents", contents);
            }
            _ => {}
        }
    }
    root.rename_key("hover_event", "hoverEvent");

    // --- click_event -> clickEvent ----------------------------------------
    if let Some(click_event) = root.get_map_mut("click_event") {
        match click_event
            .get_string("action")
            .unwrap_or("")
            .to_string()
            .as_str()
        {
            "open_url" => {
                // forward: value -> url (events with non-web URLs were dropped
                // and cannot be recovered — nothing to do for those). Lossless
                // 1:1 for surviving events.
                click_event.rename_key("url", "value");
            }
            "open_file" => {
                // forward: value -> path. Lossless 1:1.
                click_event.rename_key("path", "value");
            }
            "run_command" | "suggest_command" => {
                // forward: value -> command (invalid command/chat strings were
                // dropped — unrecoverable). Lossless 1:1 for survivors.
                click_event.rename_key("command", "value");
            }
            "change_page" => {
                // forward: parse `value` (string or number) -> int
                // `page = max(1, value)`. The legacy `value` was a string, so we
                // store the page back as a string. The original type
                // (string vs number) and any pre-clamp value (< 1) cannot be
                // recovered from the modern int, so this is lossy.
                if let Some(page) = click_event.get_i32("page") {
                    click_event.take("page");
                    click_event.set_string("value", page.to_string());
                    report_loss(
                        VERSION,
                        LossKind::Other,
                        Severity::Approximated,
                        "click_event change_page restored as string value from clamped int page; original value type/sub-1 magnitude not recoverable",
                    );
                }
            }
            _ => {}
        }
    }
    root.rename_key("click_event", "clickEvent");
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.text_component.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| convert_component(data)),
    );
    // Reverse: rebuild the pre-1.21.5 `hoverEvent` / `clickEvent` schema
    // (V4292.java:647-804). Most branches are lossless 1:1 field renames /
    // map re-nesting; only `change_page` is lossy (clamped int -> string).
    reg.text_component.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| convert_component_reverse(data)),
    );

    reg.text_component.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, root, from, to| {
            convert_list(reg, &reg.text_component, root, "extra", from, to);
            convert(reg, &reg.text_component, root, "separator", from, to);

            // clickEvent `command` -> custom command type (non-schematic, skipped).

            // Read the action first so the `hover_event` borrow is released before
            // the `show_item` branch, which needs `root` mutably.
            let action = root
                .get_map("hover_event")
                .and_then(|h| h.get_string("action"))
                .map(|s| s.to_string());
            match action.as_deref() {
                Some("show_text") => {
                    if let Some(hover_event) = root.get_map_mut("hover_event") {
                        convert(reg, &reg.text_component, hover_event, "value", from, to);
                    }
                }
                Some("show_item") => {
                    // The whole hover_event map is the item stack.
                    convert(reg, &reg.item_stack, root, "hover_event", from, to);
                }
                Some("show_entity") => {
                    if let Some(hover_event) = root.get_map_mut("hover_event") {
                        convert_value(&reg.entity_name, hover_event, "id", from, to);
                        convert(reg, &reg.text_component, hover_event, "name", from, to);
                    }
                }
                _ => {}
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::super::super::loss;
    use super::*;

    #[test]
    fn reverse_show_item_id_only_reports_string_map_collapse() {
        let mut component = NbtMap::new();
        let mut hover = NbtMap::new();
        hover.set_string("action", "show_item");
        hover.set_string("id", "minecraft:stone");
        component.set_map("hover_event", hover);

        let (_, report) = loss::run_reverse(|| convert_component_reverse(&mut component));

        assert_eq!(report.len(), 1);
        assert_eq!(report.entries[0].kind, LossKind::FingerprintCollapse);
        let hover = component.get_map("hoverEvent").unwrap();
        let contents = hover.get_map("contents").unwrap();
        assert_eq!(contents.get_string("id"), Some("minecraft:stone"));
    }
}
