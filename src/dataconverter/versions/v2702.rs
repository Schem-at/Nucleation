//! V2702 (21w10a + 3) — schematic-relevant subset of `V2702.java`.
//!
//! Arrow-family `player` flag -> `pickup` byte (V2702.java:13-31). When an arrow,
//! spectral_arrow, or trident has no `pickup` key, the legacy boolean `player`
//! (default `true`) is removed and rewritten as a `pickup` byte (1 if player,
//! else 0). Registered per-id on ENTITY. Nothing non-schematic is present.
//!
//! VERSION = MCVersions.V21W10A (2699) + 3 = 2702.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 2702;

fn convert_arrow(
    data: &mut NbtMap,
    _from: super::super::version::EncodedVersion,
    _to: super::super::version::EncodedVersion,
) {
    if data.has_key("pickup") {
        return;
    }
    // getBoolean("player", true): default true when absent.
    let player = data.get_bool("player").unwrap_or(true);
    data.take("player");
    data.set_byte("pickup", if player { 1 } else { 0 });
}

/// Inverse of `convert_arrow` (V2702.java:13-31): restore the legacy boolean
/// `player` from the `pickup` byte and drop `pickup`.
///
/// The forward only ever produced `pickup` 0 (player=false) or 1 (player=true),
/// so those map back exactly. `pickup == 2` (creative-only pickup) has no
/// representation in the pre-2702 boolean `player`; we best-effort it as
/// `player = (pickup != 0)` -> `true` and report the dropped distinction
/// (rule 11: the modern value genuinely can't be encoded in the old boolean).
fn revert_arrow(
    data: &mut NbtMap,
    _from: super::super::version::EncodedVersion,
    _to: super::super::version::EncodedVersion,
) {
    // Forward removed `player` and always wrote `pickup`; if `pickup` is somehow
    // absent, mirror the forward default (player=true).
    let pickup = data.get_i32("pickup").unwrap_or(1);
    if pickup == 2 {
        report_loss(
            VERSION,
            LossKind::UnsupportedInTarget,
            Severity::Loss,
            "arrow pickup=2 (creative pickup) collapses to player=true; the creative-only distinction is lost",
        );
    }
    data.take("pickup");
    // vanilla treats `player` as a boolean -> non-zero pickup means player pickup.
    data.set_bool("player", pickup != 0);
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity
        .add_converter_for_id("minecraft:arrow", VERSION, 0, Box::new(convert_arrow));
    reg.entity
        .add_reverse_converter_for_id("minecraft:arrow", VERSION, 0, Box::new(revert_arrow));
    reg.entity.add_converter_for_id(
        "minecraft:spectral_arrow",
        VERSION,
        0,
        Box::new(convert_arrow),
    );
    reg.entity.add_reverse_converter_for_id(
        "minecraft:spectral_arrow",
        VERSION,
        0,
        Box::new(revert_arrow),
    );
    reg.entity
        .add_converter_for_id("minecraft:trident", VERSION, 0, Box::new(convert_arrow));
    reg.entity.add_reverse_converter_for_id(
        "minecraft:trident",
        VERSION,
        0,
        Box::new(revert_arrow),
    );
}
