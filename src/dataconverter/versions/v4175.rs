//! V4175 (24w44a + 1) — schematic-relevant subset of `V4175.java`.
//!
//! VERSION = MCVersions.V24W44A (4174) + 1 = 4175.
//!
//! Ported (DATA_COMPONENTS structure converter, V4175.java:16-63):
//!   * Rename the `model` field of the `minecraft:equippable` component to
//!     `asset_id`.
//!   * Migrate the legacy scalar `minecraft:custom_model_data` (a single number)
//!     into the new object form `{floats:[<float>]}`. The value is taken via
//!     `floatValue()` in Java, so any numeric tag is widened to f32.
//!
//! Nothing non-schematic is present in this version.

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4175;

pub fn register(reg: &mut RegistryBuilder) {
    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // RenameHelper.renameSingle(data.getMap("minecraft:equippable"), "model", "asset_id")
            if let Some(equippable) = data.get_map_mut("minecraft:equippable") {
                equippable.rename_key("model", "asset_id");
            }

            // The legacy custom_model_data was a single number; wrap it into the
            // new {floats:[<float>]} object form (taking floatValue()).
            let model_data = data.get_f64("minecraft:custom_model_data");
            if let Some(model_data) = model_data {
                let mut new_model_data = NbtMap::new();
                new_model_data.set_list("floats", vec![NbtValue::Float(model_data as f32)]);
                data.set_map("minecraft:custom_model_data", new_model_data);
            }
        }),
    );

    // Reverse of the DATA_COMPONENTS structure converter (V4175.java:16-36).
    //
    // Two operations to undo, both lossless for a real downgrade (rule 3: by the
    // time this runs every newer version is already undone, so the data is in
    // this version's forward-output schema):
    //
    //   * Rename `asset_id` back to `model` inside `minecraft:equippable`. The
    //     forward did this with a manual RenameHelper.renameSingle (not a
    //     map_renamer registration), so the engine does not auto-invert it; the
    //     exact inverse is the swapped rename (bucket A).
    //   * Collapse the new `minecraft:custom_model_data` object form
    //     `{floats:[<float>]}` back to the legacy scalar number. In this
    //     version's forward output the object only ever holds a single float in
    //     `floats` (the value `floatValue()` produced), so taking that float
    //     reproduces the legacy scalar exactly (bucket B — structural collapse).
    reg.data_components.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(equippable) = data.get_map_mut("minecraft:equippable") {
                equippable.rename_key("asset_id", "model");
            }

            // Undo {floats:[<float>]} -> scalar number. Pull the first float out
            // of the object's `floats` list and restore it as the bare scalar.
            let model_data = data.get_map("minecraft:custom_model_data");
            let scalar = model_data
                .and_then(|m| m.get_list("floats"))
                .and_then(|floats| floats.first())
                .and_then(|v| v.as_f64());
            if let Some(model_data) = model_data {
                let float_count = model_data.get_list("floats").map(|f| f.len()).unwrap_or(0);
                if float_count != 1 || model_data.inner().len() != 1 {
                    report_loss(
                        VERSION,
                        LossKind::ComponentDropped,
                        Severity::Loss,
                        "custom_model_data object had extra/missing floats or fields; legacy format stores only one scalar custom_model_data value",
                    );
                }
            }
            if let Some(scalar) = scalar {
                data.set_generic(
                    "minecraft:custom_model_data",
                    NbtValue::Float(scalar as f32),
                );
            }
        }),
    );
}
