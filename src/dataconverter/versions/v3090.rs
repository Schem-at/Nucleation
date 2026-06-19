//! V3090 (22w15a + 1) — schematic-relevant subset of `V3090.java`.
//!
//! Kept: the ENTITY `minecraft:painting` converter, which renames `Motive` ->
//! `variant` and `Facing` -> `facing` (RenameHelper.renameSingle, i.e. move the
//! value under the new key only when the old key exists).
//!
//! VERSION = MCVersions.V22W15A (3089) + 1 = 3090.

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 3090;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:painting",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // renameSingle: no-op if the source key is absent.
            data.rename_key("Motive", "variant");
            data.rename_key("Facing", "facing");
        }),
    );

    // Reverse: undo the painting key renames. The id is unchanged ("minecraft:painting"),
    // so match the same id. Each rename_key is a no-op when the source key is absent,
    // exactly mirroring RenameHelper.renameSingle. Lossless inverse (bucket A) — no loss.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:painting",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("variant", "Motive");
            data.rename_key("facing", "Facing");
        }),
    );
}
