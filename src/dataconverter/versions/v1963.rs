//! V1963 (1.14.2) — schematic-relevant subset of `V1963.java`.
//!
//! Ported: the ENTITY `minecraft:villager` converter that strips every gossip
//! of `Type == "golem"` from the `Gossips` list (V1963.java:15-32).
//!
//! VERSION = MCVersions.V1_14_2 = 1963.
//!
//! Implementation note: no rename helper fits this — it is a list filter, so it
//! is implemented inline with NbtMap/MapExt over the `Gossips` list (a list of
//! maps). `retain` is equivalent to Java's index walk that removes matching
//! entries.
//!
//! Nothing non-schematic is present in this version.

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 1963;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if let Some(gossips) = data.get_list_mut("Gossips") {
                gossips.retain(|gossip| {
                    gossip
                        .as_compound_ref()
                        .and_then(|m| m.get_string("Type"))
                        != Some("golem")
                });
            }
        }),
    );

    // Reverse (new -> old): the forward converter (V1963.java:15-32) *deletes*
    // every gossip whose `Type == "golem"` from the villager's `Gossips` list.
    // Deletion is irreversible: modern data carries no record of which golem
    // gossips (or their values) once existed, so there is no information to
    // reconstruct them (rule 11 — modern data genuinely cannot determine the
    // old value). Best-effort inverse is a no-op on the data; we only flag the
    // loss for villagers that actually have a `Gossips` list (mirroring the
    // forward guard, which returned early when `Gossips` was absent — those
    // could never have lost a golem entry). LossKind::Other: this is a deleted
    // gameplay list entry, not a flattening/component/rename/merge case.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_list("Gossips").is_some() {
                report_loss(
                    VERSION,
                    LossKind::Other,
                    Severity::Loss,
                    "villager golem-type gossips were stripped on upgrade and cannot be restored",
                );
            }
        }),
    );
}
