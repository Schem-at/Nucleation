//! V135 (15w40b+1) — schematic-relevant subset of `V135.java`.
//!
//! Two changes are ported:
//!  * The ENTITY structure converter that migrates the legacy single-rider
//!    `Riding` compound into the modern `Passengers` list, flipping the data
//!    layout so the topmost vehicle becomes the root entity and each vehicle
//!    carries the entity below it in `Passengers`.
//!  * The ENTITY structure walker, which now descends the `Passengers` list and
//!    the entity equipment (replacing the V99 `Riding` walker).
//!
//! Skipped (non-schematic): the V135 PLAYER structure walkers (Inventory /
//! EnderItems / RootVehicle / ender_pearls) — PLAYER never appears in a
//! schematic file.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::convert_list;

const VERSION: i32 = 135;

pub fn register(reg: &mut RegistryBuilder) {
    // Riding -> Passengers (V135.java:21-38).
    //
    // Java walks UP the `Riding` chain, removing each `Riding` and re-parenting
    // the lower entity as a `Passengers` entry of the vehicle above it, then
    // returns the topmost vehicle as the new root. The Rust `Converter` mutates
    // the map in place (it cannot return a replacement), so we rebuild the new
    // root from a clone of `data` and overwrite `data` with it.
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // No vehicle chain -> nothing to do (Java returns null).
            if data.get_map("Riding").is_none() {
                return;
            }

            // `current` is the rider at the bottom of the chain; on each step we
            // detach its vehicle, make `current` a passenger of the vehicle, and
            // climb up to the vehicle. Mirrors `ret = data = riding`.
            let mut current = std::mem::take(data);
            while let Some(NbtValue::Compound(mut vehicle)) = current.take("Riding") {
                vehicle.set_list("Passengers", vec![NbtValue::Compound(current)]);
                current = vehicle;
            }

            *data = current;
        }),
    );

    // Reverse: Passengers nesting -> Riding chain (inverse of V135.java:21-38).
    //
    // The forward converter turned a single-rider `Riding` chain into a nested
    // `Passengers` layout, flipping it so the TOP vehicle is the root and each
    // vehicle carries the entity below it as its sole `Passengers` entry. The
    // inverse re-nests that as a `Riding` chain with the BOTTOM rider as root.
    //
    // IMPORTANT — interaction with the walker (see engine `convert_reverse`):
    // reverse conversion descends the walker FIRST, so by the time *this* node's
    // reverse converter runs, the ENTITY walker has already recursed into our
    // single `Passengers` entry and reverse-converted that whole sub-tree into a
    // finished `Riding` chain. So we must NOT rebuild the entire chain here (that
    // would re-process, and overwriting the rider's existing `Riding` would drop
    // the middle of the chain). We only need to attach OUR vehicle (this node,
    // stripped of `Passengers`) at the BOTTOM of the rider's already-built
    // `Riding` chain, and make the rider the new root.
    //
    // Lossless for chains the forward produced. If a `Passengers` list holds
    // multiple riders (a genuinely multi-passenger entity with no legacy `Riding`
    // representation) the older schema cannot encode the siblings; we keep the
    // first and report the loss.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // No nested passenger -> nothing to do (forward returned null).
            if data.get_list("Passengers").map(|p| p.is_empty()).unwrap_or(true) {
                return;
            }

            // This node is the vehicle; detach its Passengers (the key did not
            // exist pre-V135).
            let mut vehicle = std::mem::take(data);
            let Some(NbtValue::List(mut passengers)) = vehicle.take("Passengers") else {
                *data = vehicle;
                return;
            };
            if passengers.is_empty() {
                *data = vehicle;
                return;
            }

            // Legacy `Riding` supports only one rider per vehicle.
            if passengers.len() > 1 {
                report_loss(
                    VERSION,
                    LossKind::UnsupportedInTarget,
                    Severity::Loss,
                    "multiple Passengers cannot be represented by the legacy single-rider Riding chain; extra riders dropped",
                );
            }

            // The (single) passenger is ALREADY a finished `Riding` chain — the
            // walker reverse-converted it before us.
            let mut rider = match passengers.remove(0) {
                NbtValue::Compound(m) => m,
                // Non-compound passenger: cannot form a Riding chain; restore and
                // stop (defensive — forward only ever wrote compounds).
                other => {
                    passengers.insert(0, other);
                    vehicle.set_list("Passengers", passengers);
                    *data = vehicle;
                    return;
                }
            };

            // Walk to the bottom of the rider's existing Riding chain and hang our
            // (Passengers-stripped) vehicle off the end.
            let mut node = &mut rider;
            while node.get_map("Riding").is_some() {
                node = node.get_map_mut("Riding").expect("just checked Riding is a map");
            }
            node.set_map("Riding", vehicle);

            *data = rider;
        }),
    );

    // ENTITY structure walker: descend Passengers list + equipment
    // (V135.java:53-57). This supersedes the V99 `Riding` walker.
    reg.entity.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, data, from, to| {
            convert_list(reg, &reg.entity, data, "Passengers", from, to);
            reg.entity_equipment.convert(reg, data, from, to);
        }),
    );
}
