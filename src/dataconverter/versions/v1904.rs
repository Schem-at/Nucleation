//! V1904 (18w43c + 1) — schematic-relevant subset of `V1904.java`.
//!
//! VERSION = MCVersions.V18W43C (1903) + 1 = 1904.
//!
//! Ported (schematic-relevant): the per-id ENTITY converter for
//! `minecraft:ocelot` (V1904.java:14-32). The wild ocelot `CatType` field is
//! reinterpreted: type 0 (wild) with an owner becomes a trusting ocelot, while
//! types 1..3 are split off into the new `minecraft:cat` entity.
//!
//! Skipped (non-schematic / no-op): the commented-out `registerMob("minecraft:cat")`.

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 1904;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:ocelot",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // getInt defaults to 0 when CatType is absent.
            let cat_type = data.get_i32("CatType").unwrap_or(0);

            if cat_type == 0 {
                // A wild ocelot that already has an owner becomes "trusting".
                let owner = data.get_string("Owner").map(|s| s.to_string());
                let owner_uuid = data.get_string("OwnerUUID").map(|s| s.to_string());
                let has_owner = owner.as_deref().map(|s| !s.is_empty()).unwrap_or(false)
                    || owner_uuid
                        .as_deref()
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);
                if has_owner {
                    data.set_bool("Trusting", true);
                }
            } else if cat_type > 0 && cat_type < 4 {
                // Domestic cat types are split into the new cat entity. Java
                // re-sets OwnerUUID to its current value (or "" when absent),
                // which normalises a missing OwnerUUID to an empty string.
                data.set_string("id", "minecraft:cat");
                let owner_uuid = data
                    .get_string("OwnerUUID")
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                data.set_string("OwnerUUID", owner_uuid);
            }
        }),
    );

    // Reverse of branch B (V1904.java:25-28): a `minecraft:cat` produced here was
    // an ocelot with CatType 1, 2, or 3. Restore the ocelot id, but the specific
    // CatType discriminator (siamese / red / persian) was discarded by the
    // forward split — the new `minecraft:cat` entity carries no field encoding it.
    // This is a genuine merge: every domestic CatType collapses to one modern id
    // with no surviving discriminator, so reconstruction is ambiguous. Pick the
    // canonical preimage CatType=1 and report the loss.
    //
    // Keyed on the NEW id `minecraft:cat` (rule 4); any id rename runs later.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:cat",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.set_string("id", "minecraft:ocelot");
            match data.get_i32("CatType") {
                Some(1..=3) => {}
                Some(other) => {
                    data.set_i32("CatType", 1);
                    report_loss(
                        VERSION,
                        LossKind::EntityMergeAmbiguous,
                        Severity::Approximated,
                        format!(
                            "minecraft:cat CatType={other} is not a V1904 ocelot split value; defaulting to 1"
                        ),
                    );
                }
                None => {
                    data.set_i32("CatType", 1);
                    report_loss(
                        VERSION,
                        LossKind::EntityMergeAmbiguous,
                        Severity::Approximated,
                        "minecraft:cat missing CatType; original ocelot CatType 1..3 is unrecoverable, defaulting to 1",
                    );
                }
            }
        }),
    );

    // Reverse of branch A (V1904.java:19-24): a wild ocelot (CatType 0) that had
    // an owner gained `Trusting=true`. Reverse only touches ocelots that kept the
    // ocelot id (branch B re-ids to cat and is handled above). `CatType` is absent
    // in the new format for wild ocelots — restoring the implicit 0 the old format
    // always carried is exact (rule 11). The `Trusting` flag the forward added has
    // no place in the old schema, so drop it (bucket D). This is lossless: only
    // CatType==0 ocelots reach branch A, so reconstructing CatType=0 is unambiguous.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:ocelot",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.take("Trusting");
            data.set_i32("CatType", 0);
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::convert_entity_reverse;
    use crate::nbt::NbtMap;

    #[test]
    fn reverse_cat_preserves_existing_legacy_cat_type() {
        let mut cat = NbtMap::new();
        cat.set_string("id", "minecraft:cat");
        cat.set_i32("CatType", 3);

        let report = convert_entity_reverse(&mut cat, 1904, 1903);

        assert!(report.is_empty());
        assert_eq!(cat.get_string("id"), Some("minecraft:ocelot"));
        assert_eq!(cat.get_i32("CatType"), Some(3));
    }
}
