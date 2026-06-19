//! V111 (15w33b = 111) — EntityRotationFix for legacy `Painting` and
//! `ItemFrame` entities; cites V111.java.
//!
//! Migrates pre-1.9 hanging entities that lack a `Facing` key: the old
//! `Direction`/`Dir` byte is consumed and re-emitted as a new `Facing` byte.
//! When `Direction` is present, the entity's `TileX/TileY/TileZ` are offset by
//! the direction unit vector and (for `ItemFrame`) `ItemRotation` is doubled.
//!
//! Registered on the legacy ids `Painting` and `ItemFrame` (entity namespacing
//! happens later, at V704), matching the exact id strings the Java converter
//! checks against. Nothing skipped — the file contains only these two ENTITY
//! registrations sharing one `EntityRotationFix` instance.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 111;

/// Java `EntityRotationFix.DIRECTIONS` — unit offsets indexed by `facing`.
const DIRECTIONS: [[i32; 3]; 4] = [[0, 0, 1], [-1, 0, 0], [0, 0, -1], [1, 0, 0]];

fn entity_rotation_fix(data: &mut NbtMap) {
    // Java: if (data.getNumber("Facing") != null) return null; — already migrated.
    if data.get_i64("Facing").is_some() {
        return;
    }

    let facing: i32;
    // Java: final Number direction = data.getNumber("Direction");
    if let Some(direction) = data.get_i64("Direction") {
        data.take("Direction");
        // facing = direction.intValue() % DIRECTIONS.length;
        facing = (direction as i32) % (DIRECTIONS.len() as i32);
        let offsets = DIRECTIONS[facing as usize];
        // data.setInt("TileX", data.getInt("TileX") + offsets[0]); (getInt default 0)
        data.set_i32("TileX", data.get_i32("TileX").unwrap_or(0) + offsets[0]);
        data.set_i32("TileY", data.get_i32("TileY").unwrap_or(0) + offsets[1]);
        data.set_i32("TileZ", data.get_i32("TileZ").unwrap_or(0) + offsets[2]);

        // if ("ItemFrame".equals(data.getString("id"))) { ... }
        if data.get_string("id") == Some("ItemFrame") {
            // final Number rotation = data.getNumber("ItemRotation");
            if let Some(rotation) = data.get_i32("ItemRotation") {
                // data.setByte("ItemRotation", (byte)(rotation.byteValue() * 2));
                let r = rotation as i8; // Number.byteValue() truncates to a byte first
                data.set_byte("ItemRotation", r.wrapping_mul(2));
            }
        }
    } else {
        // facing = data.getByte("Dir") % DIRECTIONS.length; (getByte default 0)
        // getByte truncates the numeric tag to a signed byte first (byteValue()).
        facing = (data.get_i32("Dir").unwrap_or(0) as i8 as i32) % (DIRECTIONS.len() as i32);
        data.take("Dir");
    }

    // data.setByte("Facing", (byte)facing);
    data.set_byte("Facing", facing as i8);
}

/// Inverse of `entity_rotation_fix`: restore the legacy hanging-entity shape
/// that the older (<1.9) game reads. The forward collapsed two distinct legacy
/// inputs — the `Direction` path (which also offset `TileX/Y/Z` and doubled an
/// `ItemFrame`'s `ItemRotation`) and the older `Dir` path (coords/rotation
/// untouched) — into a single `Facing` byte, so modern data can no longer tell
/// which one produced it. We reconstruct the canonical `Direction` preimage,
/// because that is the format 1.8-era hanging entities actually wrote (`Dir` is
/// the pre-1.8 byte form); reversing its coordinate offset and rotation doubling
/// is exact for that preimage. The `Direction`-vs-`Dir` collapse and the
/// `%4` folding of out-of-range/negative values are genuinely many-to-one, so
/// we report an approximation (the substitution is correct for the common case).
fn entity_rotation_fix_reverse(data: &mut NbtMap) {
    // If there's no Facing, the forward never ran on this entity (it bailed
    // because Facing was already present, or this isn't a migrated entity).
    let Some(facing_raw) = data.get_i64("Facing") else {
        return;
    };
    // Mirror the forward's `% DIRECTIONS.length` indexing so the offset we
    // subtract matches the one the forward added.
    let facing = (facing_raw as i32) % (DIRECTIONS.len() as i32);
    // Guard the index: forward only produced facing in 0..4, but be defensive
    // against negative/garbage Facing values rather than panicking.
    if !(0..DIRECTIONS.len() as i32).contains(&facing) {
        return;
    }
    let offsets = DIRECTIONS[facing as usize];

    // Reverse of the `Direction` branch: re-emit Direction, undo the TileX/Y/Z
    // offset, and (for ItemFrame) halve ItemRotation. This is the exact inverse
    // of that branch; choosing it over the `Dir` branch is the lossy part.
    data.take("Facing");
    data.set_i32("Direction", facing);
    data.set_i32("TileX", data.get_i32("TileX").unwrap_or(0) - offsets[0]);
    data.set_i32("TileY", data.get_i32("TileY").unwrap_or(0) - offsets[1]);
    data.set_i32("TileZ", data.get_i32("TileZ").unwrap_or(0) - offsets[2]);

    if data.get_string("id") == Some("ItemFrame") {
        if let Some(rotation) = data.get_i32("ItemRotation") {
            // Forward did `(byte)(rotation.byteValue() * 2)`; invert by halving.
            // Integer division is exact for the values the forward produces
            // (even bytes); odd inputs (not forward-produced) round toward zero.
            let r = rotation as i8;
            data.set_byte("ItemRotation", r / 2);
        }
    }

    report_loss(
        VERSION,
        LossKind::EntityMergeAmbiguous,
        Severity::Approximated,
        "hanging entity Facing reconstructed via the Direction preimage; the original Direction-vs-Dir form and any out-of-range/negative discriminator are not recoverable",
    );
}

pub fn register(reg: &mut RegistryBuilder) {
    // Java: ENTITY.addConverterForId("Painting", rotationFix);
    reg.entity.add_converter_for_id(
        "Painting",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| entity_rotation_fix(data)),
    );
    // Reverse: id is unchanged at this version (namespacing happens at V704),
    // so match the same legacy "Painting" id the forward registered on.
    reg.entity.add_reverse_converter_for_id(
        "Painting",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| entity_rotation_fix_reverse(data)),
    );
    // Java: ENTITY.addConverterForId("ItemFrame", rotationFix);
    reg.entity.add_converter_for_id(
        "ItemFrame",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| entity_rotation_fix(data)),
    );
    // Reverse of the ItemFrame branch (also halves ItemRotation).
    reg.entity.add_reverse_converter_for_id(
        "ItemFrame",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| entity_rotation_fix_reverse(data)),
    );
}
