//! V3683 (23w41a+2, `V23W41A + 2` = 3683) — schematic-relevant subset of
//! `V3683.java`.
//!
//! The TNT entity gained a `block_state`. Ported:
//!   * ENTITY `minecraft:tnt` converter: rename `Fuse` -> `fuse` and add a
//!     default `block_state = {Name: "minecraft:tnt"}`.
//!   * ENTITY `minecraft:tnt` walker: `DataWalkerTypePaths<BLOCK_STATE,
//!     "block_state">` — recurse the `block_state` compound through BLOCK_STATE.
//!
//! Nothing non-schematic here (the TNT entity is schematic content).

use std::sync::Arc;

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::convert;

const VERSION: i32 = 3683;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:tnt",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("Fuse", "fuse");

            let mut default_state = NbtMap::new();
            default_state.set_string("Name", "minecraft:tnt");
            data.set_map("block_state", default_state);
        }),
    );

    // Reverse of the forward `minecraft:tnt` converter above.
    //   * undo the `fuse` -> `Fuse` rename (lossless inverse of the forward rename;
    //     the forward used `RenameHelper.renameSingle(data, "Fuse", "fuse")`, which
    //     is a direct field rename, NOT a `map_renamer`, so it is not auto-inverted).
    //   * drop the `block_state` the forward added. The pre-3683 TNT entity had no
    //     `block_state` field at all (it was always implicitly `minecraft:tnt`), so
    //     removal is the exact inverse for the default case (bucket D). Newer data may
    //     carry a non-default `block_state` (e.g. a different Name/Properties) that the
    //     old format cannot represent — that information is genuinely lost on downgrade.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:tnt",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("fuse", "Fuse");

            if let Some(block_state) = data.take("block_state") {
                if let crate::nbt::NbtValue::Compound(bs) = block_state {
                    let name = bs.get_string("Name");
                    let is_default_name = name == Some("minecraft:tnt") || name == Some("TNT");
                    let is_default = is_default_name && bs.get("Properties").is_none();
                    if !is_default {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            "minecraft:tnt block_state cannot be represented before 3683; dropped",
                        );
                    }
                }
            }
        }),
    );

    // DataWalkerTypePaths<BLOCK_STATE, "block_state">.
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:tnt",
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.block_state, data, "block_state", from, to)
        }),
    );
}
