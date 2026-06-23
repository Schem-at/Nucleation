//! V110 (15w32c + 6) — schematic-relevant subset of `V110.java`.
//!
//! VERSION = MCVersions.V15W32C (104) + 6 = 110.
//!
//! Ported: the ENTITY `addConverterForId("EntityHorse")` converter
//! (V110.java:17-34) that moves the legacy boolean `Saddle` flag onto an actual
//! `SaddleItem` compound. If `Saddle` is false/absent, OR a `SaddleItem` map is
//! already present, the converter is a no-op (Java returns null). Otherwise it
//! removes `Saddle` and writes `SaddleItem = {id:"minecraft:saddle", Count:1b,
//! Damage:0s}`. No walker is added here — the `SaddleItem` item walker already
//! exists in V99 (per the Java comment).

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use crate::nbt::{NbtMap, NbtValue};

const VERSION: i32 = 110;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "EntityHorse",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // Java: if (!data.getBoolean("Saddle") || data.hasKey("SaddleItem", MAP)) return null;
            if !data.get_bool("Saddle").unwrap_or(false) || data.get_map("SaddleItem").is_some() {
                return;
            }

            data.take("Saddle");

            let mut saddle_item = NbtMap::new();
            saddle_item.set_string("id", "minecraft:saddle");
            saddle_item.set_byte("Count", 1);
            saddle_item.set_short("Damage", 0);
            data.set_map("SaddleItem", saddle_item);
        }),
    );

    // Reverse of V110.java:17-34. Forward turned the legacy boolean `Saddle`
    // into a `SaddleItem` compound; the old format only carried a boolean, so
    // a horse wearing a saddle is represented by `Saddle: true` with no item.
    // Reverse: if a `SaddleItem` map is present, restore `Saddle: true` and
    // drop the item. Lossless for the canonical saddle the forward emitted
    // ({id:"minecraft:saddle", Count:1b, Damage:0s}); any extra NBT a modern
    // SaddleItem carried (custom name, enchants, components, etc.) cannot be
    // represented by the legacy boolean, so we report loss in that case.
    reg.entity.add_reverse_converter_for_id(
        "EntityHorse",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // Only act when a SaddleItem map exists; mirrors the forward's
            // guard (forward never produced a SaddleItem unless Saddle was set).
            let saddle = match data.get_map("SaddleItem") {
                Some(m) => m,
                None => return,
            };

            // Detect whether this is exactly the canonical saddle the forward
            // wrote. Anything beyond {id, Count(==1b), Damage(==0s)} is extra
            // data the legacy boolean can't hold.
            let is_saddle_id = saddle.get_string("id") == Some("minecraft:saddle");
            let count_ok = matches!(saddle.get("Count"), Some(NbtValue::Byte(1)) | None);
            let damage_ok = matches!(saddle.get("Damage"), Some(NbtValue::Short(0)) | None);
            let extra_keys = saddle
                .keys()
                .iter()
                .any(|k| k != "id" && k != "Count" && k != "Damage");
            if !is_saddle_id || !count_ok || !damage_ok || extra_keys {
                report_loss(
                    VERSION,
                    LossKind::ComponentDropped,
                    Severity::Loss,
                    "EntityHorse SaddleItem carried data the legacy `Saddle` boolean cannot hold",
                );
            }

            data.take("SaddleItem");
            data.set_bool("Saddle", true);
        }),
    );
}
