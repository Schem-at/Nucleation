//! V2679 (1.16.5 + 93) — schematic-relevant subset of `V2679.java`.
//!
//! BLOCK_STATE structure converter for `minecraft:cauldron`: with the 1.17
//! cauldron split, a cauldron block-state with `level == "0"` (the default when
//! absent) has its now-meaningless `Properties` removed, while any non-zero
//! water level becomes a `minecraft:water_cauldron` (V2679.java:13-34).
//!
//! VERSION = MCVersions.V1_16_5 (2586) + 93 = 2679.
//!
//! Nothing non-schematic is present in this version.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 2679;

pub fn register(reg: &mut RegistryBuilder) {
    reg.block_state.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("Name") != Some("minecraft:cauldron") {
                return;
            }

            // No Properties -> nothing to do (Java returns null).
            if data.get_map("Properties").is_none() {
                return;
            }

            // getString("level", "0") -> defaults to "0" when absent.
            let level = data
                .get_map("Properties")
                .and_then(|p| p.get_string("level"))
                .unwrap_or("0")
                .to_string();

            if level == "0" {
                data.take("Properties");
            } else {
                data.set_string("Name", "minecraft:water_cauldron");
            }
        }),
    );

    // Reverse (1.17 -> 1.16.5): undo the cauldron split.
    //
    // The forward's only id-changing branch renamed a `minecraft:cauldron`
    // with a non-zero `level` to `minecraft:water_cauldron`, preserving its
    // `Properties` (including `level`). The new id therefore uniquely encodes
    // the old discriminator, so the inverse is exact (rule 11): rename back to
    // `minecraft:cauldron` with `Properties`/`level` untouched. No loss.
    //
    // The forward's other branch (level == "0" -> drop `Properties`, id stays
    // `minecraft:cauldron`) needs no reverse: old `minecraft:cauldron` with an
    // absent `level` is exactly level 0 (the old `getString("level", "0")`
    // default), so the empty-cauldron state already round-trips.
    // Block states carry their id in `Name`, so this cannot use
    // `add_reverse_converter_for_id` (which matches `id`).
    reg.block_state.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("Name") == Some("minecraft:water_cauldron") {
                data.set_string("Name", "minecraft:cauldron");
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use crate::dataconverter::loss;
    use crate::dataconverter::registry::registry;
    use crate::dataconverter::version::{encode_versions, MAX_STEP};
    use crate::nbt::{NbtMap, NbtValue};

    #[test]
    fn reverse_turns_water_cauldron_back_into_cauldron() {
        let mut state = NbtMap::new();
        state.insert("Name".into(), NbtValue::String("minecraft:water_cauldron".into()));
        let reg = registry();
        loss::run_reverse(|| {
            reg.block_state.convert(
                reg,
                &mut state,
                encode_versions(2679, MAX_STEP),
                encode_versions(2678, MAX_STEP),
            );
        });
        assert_eq!(
            state.get("Name"),
            Some(&NbtValue::String("minecraft:cauldron".into()))
        );
    }
}
