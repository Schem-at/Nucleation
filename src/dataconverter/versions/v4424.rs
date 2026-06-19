//! V4424 (25w15a + 2) — schematic-relevant subset of
//! `DataConverterJava/.../versions/V4424.java`.
//!
//! When an entity carries a `locator_bar_icon` compound, a default `style` of
//! `minecraft:default` is added (V4424.java:21-34). The same converter is also
//! registered on PLAYER (non-schematic — skipped) and a `ConverterRemoveFeatureFlag`
//! is registered on LIGHTWEIGHT_LEVEL (non-schematic — skipped). Only the ENTITY
//! registration is ported.

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4424;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if let Some(icon) = data.get_map_mut("locator_bar_icon") {
                icon.set_string("style", "minecraft:default");
            }
        }),
    );

    // Reverse: the forward unconditionally adds `style = "minecraft:default"` to any
    // `locator_bar_icon` compound (V4424.java:29). The pre-V4424 format had no `style`
    // field on that compound, so drop it — only when it equals the default the forward
    // wrote, to avoid discarding any genuine user value (bucket D, lossless).
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if let Some(icon) = data.get_map_mut("locator_bar_icon") {
                if icon.get_string("style") == Some("minecraft:default") {
                    icon.take("style");
                } else if icon.has_key("style") {
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        "locator_bar_icon.style is not supported before V4424; dropping non-default style",
                    );
                    icon.take("style");
                }
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::convert_entity_reverse;

    #[test]
    fn reverse_non_default_locator_bar_icon_style_reports_and_drops() {
        let mut data = crate::nbt::NbtMap::new();
        data.set_string("id", "minecraft:pig");
        let mut icon = crate::nbt::NbtMap::new();
        icon.set_string("style", "minecraft:fancy");
        data.set_map("locator_bar_icon", icon);

        let report = convert_entity_reverse(&mut data, VERSION, VERSION - 1);

        assert_eq!(report.loss_count(), 1);
        assert!(!data.get_map("locator_bar_icon").unwrap().has_key("style"));
    }
}
