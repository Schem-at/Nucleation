//! V1125 (17w15a) — schematic-relevant subset of
//! `DataConverterJava/.../versions/V1125.java`.
//!
//! Kept:
//!   * ITEM_STACK `minecraft:bed` converter: legacy beds had no color, so a bed
//!     item with `Damage == 0` is reinterpreted as red (`Damage = 14`)
//!     (V1125.java:75-84).
//!   * BIOME namespaced-id enforcement hook (V1125.java:97).
//!
//! Skipped (non-schematic): the CHUNK structure converter that scans block
//! sections to synthesize bed block-entities (V1125.java:19-73) and the
//! ADVANCEMENTS structure walker (V1125.java:87-94).

use super::super::helpers::enforce_namespaced_value_hook;
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 1125;

pub fn register(reg: &mut RegistryBuilder) {
    // Legacy beds were colorless; a bed item with Damage==0 becomes red (14).
    // Java reads getShort("Damage") which defaults to 0 when absent, so an
    // unset Damage is treated as 0 and rewritten to 14.
    reg.item_stack.add_converter_for_id(
        "minecraft:bed",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_i64("Damage").unwrap_or(0) == 0 {
                data.set_short("Damage", 14); // Red
            }
        }),
    );

    // Inverse of the bed colorization. The forward rewrote the colorless
    // legacy default (Damage 0, absent treated as 0) to red (Damage 14). The
    // bed item id is unchanged (still "minecraft:bed"; the per-color ids only
    // appear at the Flattening), so the reverse keys on "minecraft:bed".
    //
    // Lossy (rule 11): a modern bed with Damage==14 is ambiguous — it may be an
    // original colorless bed the forward rewrote to 14, or a genuine red bed
    // that already carried Damage 14. Pre-1125 beds had no color concept, so we
    // undo the forward by mapping 14 back to the colorless 0 and report it as
    // Approximated (the forward's only effect was 0->14, so this is the usual
    // correct preimage). All other Damage values were untouched by the forward
    // and are left alone.
    reg.item_stack.add_reverse_converter_for_id(
        "minecraft:bed",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if data.get_i64("Damage").unwrap_or(0) == 14 {
                data.set_short("Damage", 0); // colorless legacy default
                report_loss(
                    VERSION,
                    LossKind::FingerprintCollapse,
                    Severity::Approximated,
                    "minecraft:bed Damage==14 is ambiguous (original 0 vs 14); reverted to colorless 0",
                );
            }
        }),
    );

    // Enforce namespacing for biome ids.
    reg.biome
        .add_structure_hook(VERSION, 0, enforce_namespaced_value_hook());
}
