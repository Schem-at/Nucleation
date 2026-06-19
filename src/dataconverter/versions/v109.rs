//! V109 (15w32c + 5) — schematic-relevant subset of `V109.java`.
//!
//! VERSION = MCVersions.V15W32C (104) + 5 = 109.
//!
//! Ported: the ENTITY structure converter (V109.java:14-37) that normalizes mob
//! health and cleans up the legacy `HealF`/`Health` split. If a numeric `HealF`
//! field exists it is removed and its value becomes the new health; otherwise the
//! existing numeric `Health` value is used. If neither is present the converter is
//! a no-op (Java returns null without mutating). The resulting health is written
//! back to `Health` as a float.

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 109;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // Number healF = data.getNumber("HealF");
            let heal_f = data
                .get("HealF")
                .and_then(|v| v.as_number_f64());

            let new_health: f32 = if let Some(heal_f) = heal_f {
                // healF != null: remove HealF, use its float value.
                data.take("HealF");
                heal_f as f32
            } else {
                // Number heal = data.getNumber("Health");
                match data.get("Health").and_then(|v| v.as_number_f64()) {
                    Some(heal) => heal as f32,
                    // heal == null -> return null (no-op).
                    None => return,
                }
            };

            // data.setFloat("Health", newHealth);
            data.set_f32("Health", new_health);
        }),
    );

    // Reverse (new -> old): restore the pre-V109 two-field health schema.
    //
    // Before V109, vanilla mobs stored health in BOTH `HealF` (float, the precise
    // value) and `Health` (short, the integer health used by older code). The
    // forward converter collapsed these into a single float `Health`, dropping the
    // legacy `HealF`. To downgrade we reconstruct both fields from the surviving
    // float `Health`:
    //   - `HealF`  = the float value (carries the exact health, no loss),
    //   - `Health` = that value cast to a short (the canonical old integer form).
    //
    // This is the canonical preimage (rule 11): the old `Health` short was always
    // the integer representation of the same health, so the float in `HealF`
    // preserves the value precisely and `Health` is its conventional short cast.
    // No `report_loss` — the value is preserved exactly and the integer cast is the
    // documented old encoding, not unrecoverable user data.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // Use whatever numeric health survives forward output (float `Health`).
            let health = match data.get("Health").and_then(|v| v.as_number_f64()) {
                Some(h) => h as f32,
                // No health at all -> nothing to split (mirrors the forward no-op).
                None => return,
            };

            // HealF = the precise float value.
            data.set_f32("HealF", health);
            // Health = the integer (short) cast, the old conventional form.
            data.set_short("Health", health as i16);
        }),
    );
}
