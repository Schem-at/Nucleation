//! V147 (15w46a+1) — schematic-relevant subset of `V147.java`.
//!
//! For `ArmorStand` entities, drops the redundant `Silent` flag when the stand
//! is silent but not a marker (`Silent && !Marker`). Java's `getBoolean`
//! defaults to `false` for absent keys, mirrored here with `unwrap_or(false)`.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 147;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "ArmorStand",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let silent = data.get_bool("Silent").unwrap_or(false);
            let marker = data.get_bool("Marker").unwrap_or(false);
            if silent && !marker {
                data.take("Silent");
            }
        }),
    );

    // Inverse of `V147.java` lines 16-18: the forward drops `Silent` from a
    // non-marker armor stand when it was `true`, because that case was treated
    // as redundant. A non-marker armor stand with NO `Silent` tag in the
    // forward-output schema is therefore indistinguishable from one that was
    // genuinely not silent — modern data cannot recover the original value.
    // Best-effort: restore `Silent=true` (the value the forward removed) for a
    // non-marker stand that lacks the tag, since that is the only case the
    // forward ever erased. Lossy (rule 11): a non-marker stand that legitimately
    // had no `Silent`/`Silent=false` will be wrongly marked silent.
    reg.entity.add_reverse_converter_for_id(
        "ArmorStand",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let marker = data.get_bool("Marker").unwrap_or(false);
            if !marker && data.get_bool("Silent").is_none() {
                data.set_bool("Silent", true);
                report_loss(
                    VERSION,
                    LossKind::Other,
                    Severity::Approximated,
                    "ArmorStand: restored Silent=true for non-marker stand; the forward dropped Silent when silent && !Marker, so absence is ambiguous",
                );
            }
        }),
    );
}
