//! V5014 (26.3 snapshot) — the inline `minecraft:block_transformer` component
//! is removed.
//!
//! Upstream DataConverter has no step past V4892 yet; this ports Mojang's 26.3
//! `net.minecraft.util.datafix.fixes.RemoveBlockTransformerComponentFix`
//! (registered at schema 5014 in `net.minecraft.util.datafix.DataFixers`):
//! `minecraft:block_transformer` is dropped from every DATA_COMPONENTS map.
//! The component only existed in 26.3 snapshots (it is not in 26.2's
//! `DataComponents`), but item stacks carry it, so this is ported.
//!
//! Reverse: none needed — no released version before this one has the
//! component, and the forward leaves nothing behind (cheatsheet rule 10).
//!
//! VERSION = 5014 (Mojang schema id).

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 5014;

pub fn register(reg: &mut RegistryBuilder) {
    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.take("minecraft:block_transformer");
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::convert_item_stack;

    #[test]
    fn block_transformer_component_removed() {
        let mut components = NbtMap::new();
        components.set_map("minecraft:block_transformer", NbtMap::new());
        components.set_i32("minecraft:max_stack_size", 16);
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:stick");
        item.set_i32("count", 1);
        item.set_map("components", components);

        convert_item_stack(&mut item, VERSION - 1, VERSION);
        let c = item.get_map("components").unwrap();
        assert!(!c.has_key("minecraft:block_transformer"));
        assert_eq!(c.get_i32("minecraft:max_stack_size"), Some(16));
    }
}
