//! V3086 (22w13a + 1) — schematic-relevant subset of `V3086.java`.
//!
//! Kept: the ENTITY `minecraft:cat` converter, a `ConverterEntityToVariant` that
//! reads the integer `CatType` and writes the string `variant` via the cat-id
//! table (default `minecraft:tabby`). Mirrors Java: if the source field is absent
//! nothing happens; the old `CatType` field is left in place (DFU does the same).
//!
//! Skipped (non-schematic): the ADVANCEMENTS criteria rename
//! (`husbandry/complete_catalogue`) — ADVANCEMENTS never appears in a schematic.
//!
//! VERSION = MCVersions.V22W13A (3085) + 1 = 3086.

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 3086;

/// `CatType` int -> variant id (V3086.java:18-31); default `minecraft:tabby`.
fn cat_variant(id: i32) -> &'static str {
    match id {
        1 => "minecraft:black",
        2 => "minecraft:red",
        3 => "minecraft:siamese",
        4 => "minecraft:british",
        5 => "minecraft:calico",
        6 => "minecraft:persian",
        7 => "minecraft:ragdoll",
        8 => "minecraft:white",
        9 => "minecraft:jellie",
        10 => "minecraft:all_black",
        _ => "minecraft:tabby", // 0 and default
    }
}

fn cat_type(variant: &str) -> Option<i32> {
    match variant {
        "minecraft:tabby" => Some(0),
        "minecraft:black" => Some(1),
        "minecraft:red" => Some(2),
        "minecraft:siamese" => Some(3),
        "minecraft:british" => Some(4),
        "minecraft:calico" => Some(5),
        "minecraft:persian" => Some(6),
        "minecraft:ragdoll" => Some(7),
        "minecraft:white" => Some(8),
        "minecraft:jellie" => Some(9),
        "minecraft:all_black" => Some(10),
        _ => None,
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:cat",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // getNumber: nothing to do when the field is absent.
            if let Some(value) = data.get_i32("CatType") {
                data.set_string("variant", cat_variant(value));
            }
        }),
    );

    // Reverse: the forward kept the original int `CatType` in place (Java
    // explicitly does NOT remove it — see ConverterEntityToVariant) and only
    // *added* the string `variant`. So the old discriminator survives intact and
    // the inverse is the additive-default removal of `variant` (bucket D),
    // lossless. We drop it only when it equals what the forward would have
    // produced from the present `CatType`, so any unrelated/user `variant` is
    // preserved.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:cat",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let variant = data.get_string("variant").map(|s| s.to_string());
            match (data.get_i32("CatType"), variant.as_deref()) {
                (Some(value), Some(v)) if v == cat_variant(value) => {
                    data.take("variant");
                }
                (Some(value), Some(v)) => {
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        format!(
                            "cat variant `{v}` conflicts with legacy CatType={value}; dropping modern variant"
                        ),
                    );
                    data.take("variant");
                }
                (None, Some(v)) => match cat_type(v) {
                    Some(value) => {
                        data.set_i32("CatType", value);
                        data.take("variant");
                    }
                    None => {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            format!("cat variant `{v}` has no legacy CatType preimage; dropping it"),
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
    fn reverse_recovers_cat_type_from_variant_when_missing() {
        let mut cat = NbtMap::new();
        cat.set_string("id", "minecraft:cat");
        cat.set_string("variant", "minecraft:jellie");

        let report = convert_entity_reverse(&mut cat, 3086, 3085);

        assert_eq!(cat.get_i32("CatType"), Some(9));
        assert!(!cat.has_key("variant"));
        assert!(report.is_empty());
    }
}
