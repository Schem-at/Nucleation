//! V3439 (1.19.4 + 102) — schematic-relevant subset of `V3439.java`.
//!
//! VERSION = MCVersions.V1_19_4 (3337) + 102 = 3439.
//!
//! The 1.20 "hanging sign / dyed text" rework. Two things are ported, both fully
//! schematic-relevant (TILE_ENTITY):
//!   * `signTileUpdater` (V3439.java:34-111): an ITEM-/block-entity converter,
//!     registered on both `minecraft:sign` and `minecraft:hanging_sign`, that
//!     migrates the legacy flat `Text1..Text4` / `FilteredText0..3` / `Color` /
//!     `GlowingText` fields into the modern `front_text` / `back_text`
//!     sub-compounds (with `messages` / `filtered_messages` lists), and adds the
//!     `is_waxed` flag.
//!   * `registerSign` (V3439.java:24-31, 113-115): a TILE_ENTITY walker on both
//!     ids that descends `front_text` / `back_text` -> `messages` /
//!     `filtered_messages` as TEXT_COMPONENT lists.
//!
//! Nothing in V3439 is non-schematic.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::version::EncodedVersion;
use super::super::walker::convert_list;

const VERSION: i32 = 3439;

const DEFAULT_COLOR: &str = "black";

/// `ComponentUtils.EMPTY` — the stable-JSON encoding of a plain empty text
/// component, `{"text":""}` (ComponentUtils.java:12-20).
const EMPTY_COMPONENT: &str = r#"{"text":""}"#;

/// `migrateToList(root, prefix)` (V3439.java:37-50): build a 4-element string
/// list from `<prefix>1..<prefix>4`, defaulting any absent entry to EMPTY.
fn migrate_to_list(root: &NbtMap, prefix: &str) -> Vec<NbtValue> {
    let mut ret = Vec::with_capacity(4);
    for i in 1..=4 {
        let key = format!("{prefix}{i}");
        let s = root.get_string(&key).unwrap_or(EMPTY_COMPONENT).to_string();
        ret.push(NbtValue::String(s));
    }
    ret
}

/// Read the i-th string out of an already-built `messages` list (used to seed
/// `filtered_messages` from the front messages). Defaults to EMPTY.
fn list_string_at(list: &[NbtValue], i: usize) -> String {
    list.get(i)
        .and_then(|v| v.as_str())
        .unwrap_or(EMPTY_COMPONENT)
        .to_string()
}

/// `signTileUpdater.convert` (V3439.java:52-107).
fn sign_tile_updater(data: &mut NbtMap, _from: EncodedVersion, _to: EncodedVersion) {
    // --- front text --------------------------------------------------------
    let front_messages = migrate_to_list(data, "Text");

    // Build filtered_messages only if any FilteredText<i> is present; otherwise
    // leave it absent. Mirrors the Java loop over i = 0..3.
    let mut front_filtered: Option<Vec<NbtValue>> = None;
    for i in 0..4 {
        let key = format!("FilteredText{i}");
        match data.get_string(&key).map(|s| s.to_string()) {
            None => {
                if let Some(list) = front_filtered.as_mut() {
                    list.push(NbtValue::String(list_string_at(&front_messages, i)));
                }
            }
            Some(filtered) => {
                if front_filtered.is_none() {
                    // Seed with the front messages preceding index i.
                    let mut seed = Vec::with_capacity(4);
                    for k in 0..i {
                        seed.push(NbtValue::String(list_string_at(&front_messages, k)));
                    }
                    front_filtered = Some(seed);
                }
                front_filtered
                    .as_mut()
                    .unwrap()
                    .push(NbtValue::String(filtered));
            }
        }
    }

    let color = data
        .get_string("Color")
        .unwrap_or(DEFAULT_COLOR)
        .to_string();
    let glowing = data.get_bool("GlowingText").unwrap_or(false);

    let mut front_text = NbtMap::new();
    front_text.set_list("messages", front_messages);
    if let Some(filtered) = front_filtered {
        front_text.set_list("filtered_messages", filtered);
    }
    front_text.set_string("color", color);
    front_text.set_bool("has_glowing_text", glowing);
    front_text.set_bool("_filtered_correct", true);
    data.set_map("front_text", front_text);

    // --- back text (blank) -------------------------------------------------
    let mut back_text = NbtMap::new();
    let blank: Vec<NbtValue> = (0..4)
        .map(|_| NbtValue::String(EMPTY_COMPONENT.to_string()))
        .collect();
    back_text.set_list("messages", blank);
    back_text.set_string("color", DEFAULT_COLOR);
    back_text.set_bool("has_glowing_text", false);
    data.set_map("back_text", back_text);

    // --- misc --------------------------------------------------------------
    data.set_bool("is_waxed", false);
}

/// Inverse of `sign_tile_updater`.
///
/// The forward converter is purely *additive* to the legacy fields: it reads
/// `Text1..4` / `FilteredText0..3` / `Color` / `GlowingText` but never removes
/// them (V3439.java:52-107 only ever calls `set*`/`setMap`), then adds the
/// `front_text` / `back_text` sub-compounds and the `is_waxed` flag. So for a
/// real downgrade chain the legacy keys are still present and the inverse is
/// essentially a bucket-D removal of the three added fields.
///
/// To also handle data that originated in the modern (1.20+) flat schema — a
/// sign that carries only `front_text` / `back_text` and no legacy keys — we
/// reconstruct the legacy keys from `front_text` when they are absent. By the
/// time this runs the TEXT_COMPONENT walker has already downgraded the strings
/// inside `front_text.messages` / `filtered_messages`, so they are exactly the
/// old-version component JSON that `Text1..4` / `FilteredText*` expect.
///
/// Genuinely unrepresentable modern data is reported: the legacy single-sided
/// sign has no back text and no waxed flag, so non-blank `back_text` messages
/// and `is_waxed = true` are losses.
fn sign_tile_reverter(data: &mut NbtMap, version: i32) {
    // Pull the modern sub-compounds back out (they are always removed below).
    let front_text = data.get_map("front_text").cloned();
    let back_text = data.get_map("back_text").cloned();

    if let Some(front) = front_text.as_ref() {
        // messages -> Text1..4 (only when the legacy keys are not already there,
        // so a real old->new->old round trip keeps its untouched originals).
        if let Some(messages) = front.get_list("messages") {
            for i in 0..4 {
                let key = format!("Text{}", i + 1);
                if !data.has_key(&key) {
                    if let Some(s) = messages.get(i).and_then(ValueExt::as_str) {
                        data.set_string(&key, s);
                    }
                }
            }
        }

        // filtered_messages -> FilteredText0..3. The forward back-fills absent
        // filtered entries with the front message, so re-emitting one key per
        // filtered entry yields valid (old-game-readable) data; "which keys were
        // originally present" is not round-trip-meaningful, so no loss is reported.
        if let Some(filtered) = front.get_list("filtered_messages") {
            for i in 0..4 {
                let key = format!("FilteredText{i}");
                if !data.has_key(&key) {
                    if let Some(s) = filtered.get(i).and_then(ValueExt::as_str) {
                        data.set_string(&key, s);
                    }
                }
            }
        }

        // color / has_glowing_text -> Color / GlowingText (defaults match the
        // forward: "black" and false were the legacy defaults).
        if !data.has_key("Color") {
            if let Some(color) = front.get_string("color") {
                if color != DEFAULT_COLOR {
                    data.set_string("Color", color);
                }
            }
        }
        if !data.has_key("GlowingText") && front.get_bool("has_glowing_text").unwrap_or(false) {
            data.set_bool("GlowingText", true);
        }
    }

    // back_text has no legacy representation: a 1.19.4 sign is single-sided.
    // Report loss only when it actually carries text (the forward writes a blank
    // back_text whose messages are all EMPTY_COMPONENT, which is no real loss).
    if let Some(back) = back_text.as_ref() {
        let has_back_text = back
            .get_list("messages")
            .map(|msgs| {
                msgs.iter().any(|m| {
                    m.as_str()
                        .is_some_and(|s| !s.is_empty() && s != EMPTY_COMPONENT)
                })
            })
            .unwrap_or(false);
        if has_back_text {
            report_loss(
                version,
                LossKind::UnsupportedInTarget,
                Severity::Loss,
                "sign back_text has no representation before 1.20; back-side text dropped",
            );
        }
        let has_back_filtered_text = back
            .get_list("filtered_messages")
            .map(|msgs| {
                msgs.iter().any(|m| {
                    m.as_str()
                        .is_some_and(|s| !s.is_empty() && s != EMPTY_COMPONENT)
                })
            })
            .unwrap_or(false);
        if has_back_filtered_text {
            report_loss(
                version,
                LossKind::UnsupportedInTarget,
                Severity::Loss,
                "sign back_text filtered_messages has no representation before 1.20; dropped",
            );
        }
        if back
            .get_string("color")
            .is_some_and(|color| color != DEFAULT_COLOR)
        {
            report_loss(
                version,
                LossKind::UnsupportedInTarget,
                Severity::Loss,
                "sign back_text color has no representation before 1.20; dropped",
            );
        }
        if back.get_bool("has_glowing_text").unwrap_or(false) {
            report_loss(
                version,
                LossKind::UnsupportedInTarget,
                Severity::Loss,
                "sign back_text glowing flag has no representation before 1.20; dropped",
            );
        }
    }

    // is_waxed has no legacy representation; only a waxed (true) sign loses data.
    if data.get_bool("is_waxed").unwrap_or(false) {
        report_loss(
            version,
            LossKind::UnsupportedInTarget,
            Severity::Loss,
            "sign is_waxed has no representation before 1.20; waxed flag dropped",
        );
    }

    // Drop the three fields the forward added.
    data.take("front_text");
    data.take("back_text");
    data.take("is_waxed");
}

/// `handleSignText` (V3439.java:15-22): convert `messages` / `filtered_messages`
/// of a text sub-compound as TEXT_COMPONENT lists.
fn handle_sign_text(
    reg: &super::super::registry::Registry,
    data: &mut NbtMap,
    key: &str,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    if let Some(text) = data.get_map_mut(key) {
        convert_list(reg, &reg.text_component, text, "messages", from, to);
        convert_list(
            reg,
            &reg.text_component,
            text,
            "filtered_messages",
            from,
            to,
        );
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // signTileUpdater on both sign ids.
    reg.tile_entity
        .add_converter_for_id("minecraft:sign", VERSION, 0, Box::new(sign_tile_updater));
    reg.tile_entity.add_converter_for_id(
        "minecraft:hanging_sign",
        VERSION,
        0,
        Box::new(sign_tile_updater),
    );

    // Reverse: undo signTileUpdater. The forward keeps the same id and is
    // additive over the legacy keys, so the inverse rebuilds any missing legacy
    // keys from front_text and removes the front_text/back_text/is_waxed fields.
    // back_text and is_waxed predate the legacy schema -> lossy when populated.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:sign",
        VERSION,
        0,
        Box::new(|data, _from, _to| sign_tile_reverter(data, VERSION)),
    );
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:hanging_sign",
        VERSION,
        0,
        Box::new(|data, _from, _to| sign_tile_reverter(data, VERSION)),
    );

    // registerSign walker on both sign ids.
    let sign_walker: super::super::engine::Walker = Arc::new(|reg, data, from, to| {
        handle_sign_text(reg, data, "front_text", from, to);
        handle_sign_text(reg, data, "back_text", from, to);
    });
    reg.tile_entity
        .add_walker(VERSION, 0, "minecraft:sign", sign_walker.clone());
    reg.tile_entity
        .add_walker(VERSION, 0, "minecraft:hanging_sign", sign_walker);
}
