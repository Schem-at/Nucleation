//! V1450 (V17W46A + 1 = 1450) — the 1.13 block-state Flattening (`V1450.java`).
//!
//! A single BLOCK_STATE structure converter that replaces a legacy
//! `{Name, Properties}` block state with its flattened form
//! (`HelperBlockFlatteningV1450.flattenNBT`). Java returns a *new* map; our
//! engine performs the replacement with `*data = …`.
//!
//! DataConverter emits the `%%FILTER_ME%%` placeholder for skull blocks — their
//! modern id is only resolvable from the per-position block entity, which the
//! chunk pipeline handles and the standalone block-state path cannot. In a
//! schematic block palette we mirror `ConverterFlattenChunk`'s terminal behavior
//! and map the placeholder to air.

use crate::nbt::NbtMap;

use super::super::flattening::{flatten_nbt, unflatten_nbt, Unflatten, FILTER_ME};
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 1450;

pub fn register(reg: &mut RegistryBuilder) {
    reg.block_state.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if let Some(flattened) = flatten_nbt(data) {
                if flattened.get_string("Name") == Some(FILTER_ME) {
                    let mut air = NbtMap::new();
                    air.set_string("Name", "minecraft:air");
                    *data = air;
                } else {
                    *data = flattened;
                }
            }
        }),
    );

    // Reverse: un-flatten a modern `{Name, Properties}` block state to its
    // pre-1.13 form (the inverse of `HelperBlockFlatteningV1450.flattenNBT`).
    //
    // The Flattening is the chain's biggest many-to-one collapse, so the inverse
    // is the chain's biggest lossy point:
    //   * Exact match  -> losslessly restored.
    //   * Subset match -> the pre-1.13 block kept its legacy properties but any
    //     modern-only property (e.g. `waterlogged`) is dropped (the old block
    //     could not hold it); reported as `FlatteningAmbiguous`/Approximated when
    //     a non-trivial property is actually lost.
    //   * Unknown      -> a block introduced in 1.13+ that has no pre-Flattening
    //     representation; the modern name is kept and the loss is reported.
    reg.block_state.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| match unflatten_nbt(data) {
            Unflatten::Exact(old) => {
                *data = old;
            }
            Unflatten::Approximated(old) => {
                let dropped = dropped_property_keys(data, &old);
                if !dropped.is_empty() {
                    let name = data.get_string("Name").unwrap_or("?");
                    report_loss(
                        VERSION,
                        LossKind::FlatteningAmbiguous,
                        Severity::Approximated,
                        format!(
                            "{name}: dropped modern-only block propertie(s) [{}] with no pre-1.13 representation",
                            dropped.join(", ")
                        ),
                    );
                }
                *data = old;
            }
            Unflatten::Unknown => {
                let name = data.get_string("Name").unwrap_or("?").to_string();
                report_loss(
                    VERSION,
                    LossKind::FlatteningUnknownBlock,
                    Severity::Loss,
                    format!("{name} has no pre-1.13 (Flattening) block form; kept the modern name"),
                );
            }
        }),
    );
}

/// Keys present in the modern state's `Properties` that the chosen pre-1.13 form
/// does not carry (i.e. dropped on downgrade).
fn dropped_property_keys(modern: &NbtMap, old: &NbtMap) -> Vec<String> {
    let old_keys: Vec<String> = old.get_map("Properties").map(|p| p.keys()).unwrap_or_default();
    match modern.get_map("Properties") {
        Some(mp) => mp
            .keys()
            .into_iter()
            .filter(|k| !old_keys.iter().any(|ok| ok == k))
            .collect(),
        None => Vec::new(),
    }
}
