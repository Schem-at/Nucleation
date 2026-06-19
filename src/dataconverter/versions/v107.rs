//! V107 (15w32c + 3 = 107) — schematic-relevant subset of V107.java.
//!
//! Splits the generic `Minecart` entity into typed ids based on the int `Type`
//! (0..=6), which is then removed. The id is set from MINECART_IDS, defaulting
//! to `MinecartRideable` for any out-of-range value. Cites V107.java.
//!
//! Nothing skipped — the Java file contains only this single ENTITY converter.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 107;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "Minecart",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Java MINECART_IDS, indexed by Type (0..6).
            const MINECART_IDS: [&str; 7] = [
                "MinecartRideable",    // 0
                "MinecartChest",       // 1
                "MinecartFurnace",     // 2
                "MinecartTNT",         // 3
                "MinecartSpawner",     // 4
                "MinecartHopper",      // 5
                "MinecartCommandBlock", // 6
            ];

            // Java: data.getInt("Type") defaults to 0 when absent.
            let type_idx = data.get_i32("Type").unwrap_or(0);
            data.take("Type");

            let mut new_id = "MinecartRideable"; // dfl
            if type_idx >= 0 && (type_idx as usize) < MINECART_IDS.len() {
                new_id = MINECART_IDS[type_idx as usize];
            }
            data.set_string("id", new_id);
        }),
    );

    // Reverse of V107.java:13-39 — collapse each typed Minecart id back into the
    // generic "Minecart" id plus the int `Type` discriminator the forward removed.
    // The new id fully encodes Type (1:1), so this is lossless (bucket B).
    // Note: "MinecartRideable" (Type 0) is also the forward default for out-of-range
    // Type values; restoring Type=0 here is the canonical preimage.
    const MINECART_IDS: [&str; 7] = [
        "MinecartRideable",     // 0 (V107.java:15)
        "MinecartChest",        // 1 (V107.java:16)
        "MinecartFurnace",      // 2 (V107.java:17)
        "MinecartTNT",          // 3 (V107.java:18)
        "MinecartSpawner",      // 4 (V107.java:19)
        "MinecartHopper",       // 5 (V107.java:20)
        "MinecartCommandBlock", // 6 (V107.java:21)
    ];
    for (idx, mid) in MINECART_IDS.iter().enumerate() {
        let ty = idx as i32;
        reg.entity.add_reverse_converter_for_id(
            mid,
            VERSION,
            0,
            Box::new(move |data: &mut NbtMap, _from, _to| {
                data.set_string("id", "Minecart");
                data.set_i32("Type", ty);
            }),
        );
    }
}
