//! V701 (1.11 snapshot, MCVersions.V1_10_2 + 189 = 701) — schematic-relevant
//! subset of V701.java.
//!
//! Splits the legacy `Skeleton` entity into `Skeleton` / `WitherSkeleton` /
//! `Stray` based on the int `SkeletonType` (which is then removed).
//! Schematic-relevant: mutates an ENTITY's `id`. The commented-out
//! `registerMob(...)` lines in the Java are no-ops (simple mobs).

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 701;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "Skeleton",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Java: data.getInt("SkeletonType") defaults to 0 when absent.
            let skeleton_type = data.get_i32("SkeletonType").unwrap_or(0);
            data.take("SkeletonType");

            match skeleton_type {
                1 => data.set_string("id", "WitherSkeleton"),
                2 => data.set_string("id", "Stray"),
                _ => {}
            }
        }),
    );

    // Reverse of V701.java:13-30 — merge the split ids back to "Skeleton" and
    // restore the int `SkeletonType` discriminator the forward removed. The new
    // id encodes the type, so this is lossless.
    //
    // Reverse converters match the NEW (forward-output) id (cheatsheet rule 4).
    // WitherSkeleton -> Skeleton + SkeletonType=1 (Java:21-22).
    reg.entity.add_reverse_converter_for_id(
        "WitherSkeleton",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.set_string("id", "Skeleton");
            data.set_i32("SkeletonType", 1);
        }),
    );
    // Stray -> Skeleton + SkeletonType=2 (Java:23-24).
    reg.entity.add_reverse_converter_for_id(
        "Stray",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.set_string("id", "Skeleton");
            data.set_i32("SkeletonType", 2);
        }),
    );
    // Skeleton stays Skeleton; forward removed `SkeletonType` for type 0/absent
    // (Java:16-17). Restore the canonical SkeletonType=0 so the pre-forward shape
    // is reproduced exactly.
    reg.entity.add_reverse_converter_for_id(
        "Skeleton",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.set_i32("SkeletonType", 0);
        }),
    );
}
