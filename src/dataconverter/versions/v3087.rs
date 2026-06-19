//! V3087 (22w13a + 2) — schematic-relevant subset of `V3087.java`.
//!
//! Kept: the ENTITY `minecraft:frog` converter, a `ConverterEntityToVariant` that
//! reads the integer `Variant` and writes the string `variant` (0 temperate,
//! 1 warm, 2 cold). Mirrors Java: absent source field -> no-op; the old field is
//! left in place. Java's frog table has no default (an out-of-range id would NPE);
//! the engine only ever sees the in-range ids it just wrote, so we leave `variant`
//! unset for any unexpected value rather than panic — noted below.
//!
//! VERSION = MCVersions.V22W13A (3085) + 2 = 3087.

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 3087;

/// `Variant` int -> variant id (V3087.java:14-18).
fn frog_variant(id: i32) -> Option<&'static str> {
    match id {
        0 => Some("minecraft:temperate"),
        1 => Some("minecraft:warm"),
        2 => Some("minecraft:cold"),
        _ => None,
    }
}

fn frog_variant_id(variant: &str) -> Option<i32> {
    match variant {
        "minecraft:temperate" => Some(0),
        "minecraft:warm" => Some(1),
        "minecraft:cold" => Some(2),
        _ => None,
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:frog",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if let Some(value) = data.get_i32("Variant") {
                if let Some(variant) = frog_variant(value) {
                    data.set_string("variant", variant);
                }
            }
        }),
    );

    // Reverse (new -> old): the forward only ADDS the string `variant` derived
    // from the int `Variant`, leaving `Variant` itself in place (see the Java
    // ConverterEntityToVariant note "DFU doesn't appear to remove the old
    // field"). The pre-V3087 frog had no `variant` string and carried the int
    // `Variant` as its sole discriminator, so the inverse is the lossless,
    // additive (bucket D) removal of the string `variant`. The int `Variant`
    // survives the forward, so no information is reconstructed and no loss is
    // possible. We only drop `variant` when it equals the value this converter
    // would have produced from `Variant`, so unrelated user data is preserved.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:frog",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let variant = data.get_string("variant").map(|s| s.to_string());
            match (data.get_i32("Variant"), variant.as_deref()) {
                (Some(value), Some(v)) if frog_variant(value) == Some(v) => {
                    data.take("variant");
                }
                (Some(value), Some(v)) => {
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        format!(
                            "frog variant `{v}` conflicts with legacy Variant={value}; dropping modern variant"
                        ),
                    );
                    data.take("variant");
                }
                (None, Some(v)) => match frog_variant_id(v) {
                    Some(value) => {
                        data.set_i32("Variant", value);
                        data.take("variant");
                    }
                    None => {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            format!("frog variant `{v}` has no legacy Variant preimage; dropping it"),
                        );
                        data.take("variant");
                    }
                },
                _ => {}
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use crate::dataconverter::convert_entity_reverse;
    use crate::dataconverter::types::MapExt;
    use crate::nbt::NbtMap;

    #[test]
    fn reverse_recovers_frog_variant_int_from_variant_when_missing() {
        let mut frog = NbtMap::new();
        frog.set_string("id", "minecraft:frog");
        frog.set_string("variant", "minecraft:cold");

        let report = convert_entity_reverse(&mut frog, 3087, 3086);

        assert_eq!(frog.get_i32("Variant"), Some(2));
        assert!(!frog.has_key("variant"));
        assert!(report.is_empty());
    }
}
