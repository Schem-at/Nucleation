//! V1460 (18w01a+1) — schematic-relevant subset of
//! `DataConverterJava/.../versions/V1460.java`.
//!
//! `minecraft:painting` entities store a `Motive` string. It is lowercased
//! (Locale.ROOT), remapped via MOTIVE_REMAP (donkeykong->donkey_kong,
//! burningskull->burning_skull, skullandroses->skull_and_roses), then run
//! through `NamespaceUtil.correctNamespace`, and written back (V1460.java:25-35).
//! The Java comment notes the many redundant type redefinitions in this version
//! are no-ops (no data-structure change), so nothing else is ported.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::helpers::correct_namespace;
use super::super::loss::{report_loss, LossKind, Severity};

const VERSION: i32 = 1460;

/// `MOTIVE_REMAP.getOrDefault(motive, motive)` (V1460.java:14-20).
fn motive_remap(motive: &str) -> &str {
    match motive {
        "donkeykong" => "donkey_kong",
        "burningskull" => "burning_skull",
        "skullandroses" => "skull_and_roses",
        other => other,
    }
}

/// Inverse of `MOTIVE_REMAP` — restore the pre-1460 un-namespaced motive name for
/// the three remapped paintings (V1460.java:17-19). All other names pass through.
fn motive_remap_inverse(motive: &str) -> &str {
    match motive {
        "donkey_kong" => "donkeykong",
        "burning_skull" => "burningskull",
        "skull_and_roses" => "skullandroses",
        other => other,
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:painting",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if let Some(motive) = data.get_string("Motive") {
                // toLowerCase(Locale.ROOT) -> remap -> correctNamespace.
                let lowered = motive.to_lowercase();
                let remapped = motive_remap(&lowered);
                let corrected = correct_namespace(remapped);
                data.set_string("Motive", &corrected);
            }
        }),
    );

    // Reverse: forward = toLowerCase -> MOTIVE_REMAP -> correctNamespace. We undo
    // the two reversible steps — strip the `minecraft:` namespace `correctNamespace`
    // added, and invert the three MOTIVE_REMAP renames — yielding the pre-1460
    // lowercased un-namespaced motive (e.g. `minecraft:donkey_kong` -> `donkeykong`,
    // `minecraft:kebab` -> `kebab`). The `toLowerCase` step is NOT invertible: the
    // legacy motives were CamelCase (`Kebab`, `SkullAndRoses`, ...) and the original
    // casing is gone from the modern data, so this is a best-effort approximation
    // (the lowercased name is still a valid/recognized id in the old format).
    reg.entity.add_reverse_converter_for_id(
        "minecraft:painting",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if let Some(motive) = data.get_string("Motive") {
                // Strip a leading `minecraft:` namespace (added by correctNamespace);
                // a non-minecraft namespace is preserved as-is.
                let stripped = motive.strip_prefix("minecraft:").unwrap_or(motive);
                let unmapped = motive_remap_inverse(stripped).to_string();
                let changed = unmapped != motive;
                if changed {
                    data.set_string("Motive", unmapped);
                }
                report_loss(
                    VERSION,
                    LossKind::RenameAmbiguous,
                    Severity::Approximated,
                    "painting Motive was lowercased by V1460; original CamelCase casing cannot be recovered",
                );
            }
        }),
    );
}
