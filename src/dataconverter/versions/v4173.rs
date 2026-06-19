//! V4173 (1.21.3 + 91) — schematic-relevant subset of `V4173.java`.
//!
//! VERSION = MCVersions.V1_21_3 (4082) + 91 = 4173.
//!
//! Ported: the ENTITY structure converter that renames the `TNTFuse` field to
//! `fuse` (V4173.java:14-21). This applies to every entity compound; the rename
//! is a no-op when the key is absent (mirroring `RenameHelper.renameSingle`).
//! Nothing non-schematic is present in this version.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4173;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.rename_key("TNTFuse", "fuse");
        }),
    );
    // Reverse: undo the `TNTFuse` -> `fuse` rename (V4173.java:17). Lossless
    // 1:1 field rename; no loss. (The forward is a structure-converter rename,
    // not a `map_renamer` value rename, so it is not auto-inverted.)
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.rename_key("fuse", "TNTFuse");
        }),
    );
}
