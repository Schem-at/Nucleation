//! V703 (1.11 snapshot, MCVersions.V1_10_2 + 191 = 703) — schematic-relevant
//! subset of V703.java.
//!
//! Splits the legacy `EntityHorse` entity into `Horse` / `Donkey` / `Mule` /
//! `ZombieHorse` / `SkeletonHorse` based on the int `Type` (which is then
//! removed), and registers item walkers for each resulting id (armor/saddle
//! slots and chest `Items`). All schematic-relevant (ENTITY id + ITEM_STACK
//! descent).

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{item_lists, items};

const VERSION: i32 = 703;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "EntityHorse",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Java: data.getInt("Type") defaults to 0 when absent.
            let horse_type = data.get_i32("Type").unwrap_or(0);
            data.take("Type");

            match horse_type {
                1 => data.set_string("id", "Donkey"),
                2 => data.set_string("id", "Mule"),
                3 => data.set_string("id", "ZombieHorse"),
                4 => data.set_string("id", "SkeletonHorse"),
                // case 0 and default
                _ => data.set_string("id", "Horse"),
            }
        }),
    );

    // Reverse: merge the typed horse ids back into legacy "EntityHorse" + int
    // `Type`. Each new id uniquely encodes the old discriminator, so this is the
    // exact inverse of the forward split (lossless — no report_loss). Mirrors the
    // Java switch in V703.convert (cases 0..4).
    for (mid, ty) in [
        ("Horse", 0),
        ("Donkey", 1),
        ("Mule", 2),
        ("ZombieHorse", 3),
        ("SkeletonHorse", 4),
    ] {
        reg.entity.add_reverse_converter_for_id(
            mid,
            VERSION,
            0,
            Box::new(move |data: &mut NbtMap, _from, _to| {
                data.set_string("id", "EntityHorse");
                data.set_i32("Type", ty);
            }),
        );
    }

    reg.entity
        .add_walker(VERSION, 0, "Horse", items(&["ArmorItem", "SaddleItem"]));

    reg.entity
        .add_walker(VERSION, 0, "Donkey", items(&["SaddleItem"]));
    reg.entity
        .add_walker(VERSION, 0, "Donkey", item_lists(&["Items"]));

    reg.entity
        .add_walker(VERSION, 0, "Mule", items(&["SaddleItem"]));
    reg.entity
        .add_walker(VERSION, 0, "Mule", item_lists(&["Items"]));

    reg.entity
        .add_walker(VERSION, 0, "ZombieHorse", items(&["SaddleItem"]));

    reg.entity
        .add_walker(VERSION, 0, "SkeletonHorse", items(&["SaddleItem"]));
}
