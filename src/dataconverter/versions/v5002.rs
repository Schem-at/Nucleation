//! V5002 (26.3 snapshot) — signs gain `allow_op_features`.
//!
//! Upstream DataConverter has no step past V4892 yet; this ports Mojang's 26.3
//! `net.minecraft.util.datafix.fixes.SignsOpFlagFix` (registered at schema
//! 5002 in `DataFixers`): every existing `minecraft:sign` and
//! `minecraft:hanging_sign` block entity gets `allow_op_features: true`
//! (`remainder.createBoolean(true)` -> Byte(1)), preserving the pre-26.3
//! behaviour where a sign's click commands could use operator features.
//! TILE_ENTITY is schematic-relevant (also reached through an item's
//! `minecraft:block_entity_data`), so this is ported in full.
//!
//! Reverse: drop the flag (bucket D). `true` is exactly the pre-26.3 behaviour,
//! so removing it is lossless. `false` restricts the sign in a way older
//! versions cannot express; it is dropped and reported as loss.
//!
//! VERSION = 5002 (Mojang schema id).

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 5002;

const SIGNS: [&str; 2] = ["minecraft:sign", "minecraft:hanging_sign"];

pub fn register(reg: &mut RegistryBuilder) {
    for id in SIGNS {
        reg.tile_entity.add_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data, _from, _to| {
                data.set_bool("allow_op_features", true);
            }),
        );
        reg.tile_entity.add_reverse_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data, _from, _to| match data.get_bool("allow_op_features") {
                Some(true) => {
                    data.take("allow_op_features");
                }
                Some(false) => {
                    data.take("allow_op_features");
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        "sign allow_op_features=false has no pre-26.3 equivalent; its click commands will run with operator features",
                    );
                }
                None => {}
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::{convert_block_entity, convert_block_entity_reverse};
    use crate::nbt::NbtMap;

    #[test]
    fn signs_gain_flag_and_lose_it_on_downgrade() {
        for id in SIGNS {
            let mut sign = NbtMap::new();
            sign.set_string("id", id);
            convert_block_entity(&mut sign, VERSION - 1, VERSION);
            assert_eq!(sign.get_bool("allow_op_features"), Some(true));

            let report = convert_block_entity_reverse(&mut sign, VERSION, VERSION - 1);
            assert!(report.is_empty());
            assert!(!sign.has_key("allow_op_features"));
        }
    }

    #[test]
    fn other_block_entities_untouched() {
        let mut chest = NbtMap::new();
        chest.set_string("id", "minecraft:chest");
        convert_block_entity(&mut chest, VERSION - 1, VERSION);
        assert!(!chest.has_key("allow_op_features"));
    }

    #[test]
    fn disabled_flag_reports_loss() {
        let mut sign = NbtMap::new();
        sign.set_string("id", "minecraft:sign");
        sign.set_bool("allow_op_features", false);
        let report = convert_block_entity_reverse(&mut sign, VERSION, VERSION - 1);
        assert_eq!(report.len(), 1);
        assert_eq!(report.entries[0].kind, LossKind::UnsupportedInTarget);
        assert!(!sign.has_key("allow_op_features"));
    }
}
