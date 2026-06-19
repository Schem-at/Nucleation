//! V4176 (24w44a + 2) — schematic-relevant subset of `V4176.java`.
//!
//! VERSION = MCVersions.V24W44A (4174) + 2 = 4176.
//!
//! Ported: the `fixInvalidLock` helper applied to TILE_ENTITY (`lock`) and
//! DATA_COMPONENTS (`minecraft:lock`) (V4176.java:81-110). A lock that consists
//! solely of `{components:{minecraft:custom_name:"\"\""}}` (an empty-string
//! custom name, the only key) is an invalid no-op lock and is removed.
//!
//! Nothing non-schematic is present in this version.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4176;

/// `V4176.fixInvalidLock` (V4176.java:81-93): remove `root[path]` when it is a
/// lock map of size 1 whose only entry is `components` (also size 1) holding
/// exactly `minecraft:custom_name == "\"\""`.
fn fix_invalid_lock(root: &mut NbtMap, path: &str) {
    let should_remove = match root.get_map(path) {
        Some(lock) if lock.inner().len() == 1 => match lock.get_map("components") {
            Some(components) => {
                components.inner().len() == 1
                    && components.get_string("minecraft:custom_name") == Some("\"\"")
            }
            None => false,
        },
        _ => false,
    };

    if should_remove {
        root.take(path);
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            fix_invalid_lock(data, "lock");
        }),
    );
    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            fix_invalid_lock(data, "minecraft:lock");
        }),
    );
}
