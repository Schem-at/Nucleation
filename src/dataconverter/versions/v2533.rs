//! V2533 (20w18a + 1) — schematic-relevant subset of `V2533.java`.
//!
//! ENTITY `minecraft:villager`: bump the `generic.follow_range` attribute base
//! from 16.0 to 48.0 (V2533.java:15-38). This runs after V2523's attribute
//! rename, so the attribute name is the namespaced `minecraft:generic.follow_range`.
//!
//! VERSION = MCVersions.V20W18A (2532) + 1 = 2533.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 2533;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let attributes = match data.get_list_mut("Attributes") {
                Some(a) => a,
                None => return,
            };
            for el in attributes.iter_mut() {
                let attr = match el.as_compound_mut() {
                    Some(a) => a,
                    None => continue,
                };
                if attr.get_string("Name") != Some("minecraft:generic.follow_range") {
                    continue;
                }
                // getDouble("Base") defaults to 0.0; absent never equals 16.0.
                if attr.get_f64("Base") == Some(16.0) {
                    attr.set_f64("Base", 48.0);
                }
            }
        }),
    );

    // Reverse of the villager follow_range bump (V2533.java:15-38).
    // Forward set 16.0 -> 48.0 only; values other than 16.0 were left alone.
    // So a modern Base of 48.0 was either originally 16.0 (bumped) or already
    // 48.0 (left untouched, since 48.0 != 16.0). Modern data cannot distinguish
    // these, so reversing 48.0 -> 16.0 is the canonical (typical) preimage:
    // vanilla villagers carried 16.0 and the forward's whole purpose was to lift
    // that to 48.0. Lossy per rule 11, but the substitution is usually correct.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let attributes = match data.get_list_mut("Attributes") {
                Some(a) => a,
                None => return,
            };
            for el in attributes.iter_mut() {
                let attr = match el.as_compound_mut() {
                    Some(a) => a,
                    None => continue,
                };
                if attr.get_string("Name") != Some("minecraft:generic.follow_range") {
                    continue;
                }
                if attr.get_f64("Base") == Some(48.0) {
                    attr.set_f64("Base", 16.0);
                    report_loss(
                        VERSION,
                        LossKind::RenameAmbiguous,
                        Severity::Approximated,
                        "villager generic.follow_range Base=48.0 reverted to 16.0; \
                         a genuinely-48.0 pre-2533 value cannot be distinguished",
                    );
                }
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use crate::dataconverter::types::{MapExt, ValueExt};
    use crate::dataconverter::{convert_entity, convert_entity_reverse};
    use crate::nbt::{NbtMap, NbtValue};

    #[test]
    fn villager_follow_range_bump_uses_namespaced_attribute_name() {
        let mut attr = NbtMap::new();
        attr.set_string("Name", "generic.followRange");
        attr.set_f64("Base", 16.0);
        let mut villager = NbtMap::new();
        villager.set_string("id", "minecraft:villager");
        villager.set_list("Attributes", vec![NbtValue::Compound(attr)]);

        convert_entity(&mut villager, 2522, 2533);

        let attr = villager.get_list("Attributes").unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(
            attr.get_string("Name"),
            Some("minecraft:generic.follow_range")
        );
        assert_eq!(attr.get_f64("Base"), Some(48.0));

        let report = convert_entity_reverse(&mut villager, 2533, 2522);

        let attr = villager.get_list("Attributes").unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(attr.get_string("Name"), Some("generic.followRange"));
        assert_eq!(attr.get_f64("Base"), Some(16.0));
        assert!(!report.is_empty());
    }
}
