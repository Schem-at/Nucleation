//! V3094 (22w17a + 1) — schematic-relevant subset of `V3094.java`.
//!
//! Kept: the ITEM_STACK `minecraft:goat_horn` converter, which reads the integer
//! `tag.SoundVariant`, removes it, and writes `tag.instrument` from the sound-
//! variant -> instrument table (out-of-range index clamps to 0, matching Java).
//! Java's `getInt` defaults to 0 when the field is absent, so an unset
//! `SoundVariant` resolves to the first instrument.
//!
//! VERSION = MCVersions.V22W17A (3093) + 1 = 3094.

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 3094;

/// `SoundVariant` int -> instrument id (V3094.java:14-23).
const SOUND_VARIANT_TO_INSTRUMENT: &[&str] = &[
    "minecraft:ponder_goat_horn",
    "minecraft:sing_goat_horn",
    "minecraft:seek_goat_horn",
    "minecraft:feel_goat_horn",
    "minecraft:admire_goat_horn",
    "minecraft:call_goat_horn",
    "minecraft:yearn_goat_horn",
    "minecraft:dream_goat_horn",
];

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_converter_for_id(
        "minecraft:goat_horn",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let tag = match data.get_map_mut("tag") {
                Some(t) => t,
                None => return,
            };

            // getInt defaults to 0 when absent.
            let sound_variant = tag.get_i32("SoundVariant").unwrap_or(0);
            tag.take("SoundVariant");

            let idx = if sound_variant < 0 || sound_variant as usize >= SOUND_VARIANT_TO_INSTRUMENT.len() {
                0
            } else {
                sound_variant as usize
            };
            tag.set_string("instrument", SOUND_VARIANT_TO_INSTRUMENT[idx]);
        }),
    );

    // Reverse: tag.instrument (string) -> tag.SoundVariant (int), removing
    // `instrument`. The forward table is a bijection on its 8 entries, so a
    // known instrument id uniquely encodes the original SoundVariant index
    // (lossless for real downgrades, per rule 11). An unknown/modded instrument
    // has no preimage in the table: fall back to SoundVariant=0 (the value the
    // forward clamp used for out-of-range indices) and report the loss.
    reg.item_stack.add_reverse_converter_for_id(
        "minecraft:goat_horn",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let tag = match data.get_map_mut("tag") {
                Some(t) => t,
                None => return,
            };

            let instrument = match tag.get_string("instrument") {
                Some(s) => s.to_string(),
                None => return,
            };
            tag.take("instrument");

            match SOUND_VARIANT_TO_INSTRUMENT
                .iter()
                .position(|&i| i == instrument)
            {
                Some(idx) => tag.set_i32("SoundVariant", idx as i32),
                None => {
                    tag.set_i32("SoundVariant", 0);
                    report_loss(
                        VERSION,
                        LossKind::RenameAmbiguous,
                        Severity::Approximated,
                        "goat_horn instrument has no SoundVariant preimage; defaulted to 0",
                    );
                }
            }
        }),
    );
}
