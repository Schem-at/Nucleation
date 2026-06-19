//! V3084 (22w12a + 2) — schematic-relevant subset of `V3084.java`.
//!
//! Kept: the GAME_EVENT_NAME value-type renames (game-event id renames). Java
//! normalizes the input through `NamespaceUtil.correctNamespace(name)` before the
//! map lookup; every rename-table key is already `minecraft:`-prefixed, so we
//! replicate that by namespace-correcting the value before lookup. This is
//! implemented inline (rather than `register_value_rename` with a plain
//! `map_renamer`) so that an unnamespaced input id still matches.
//!
//! VERSION = MCVersions.V22W12A (3082) + 2 = 3084.

use crate::nbt::NbtValue;

use super::super::helpers::correct_namespace_or_null;
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 3084;

/// `(old, new)` game-event id renames (V3084.java:19-35). Keys are namespaced.
const GAME_EVENT_RENAMES: &[(&str, &str)] = &[
    ("minecraft:block_press", "minecraft:block_activate"),
    ("minecraft:block_switch", "minecraft:block_activate"),
    ("minecraft:block_unpress", "minecraft:block_deactivate"),
    ("minecraft:block_unswitch", "minecraft:block_deactivate"),
    ("minecraft:drinking_finish", "minecraft:drink"),
    ("minecraft:elytra_free_fall", "minecraft:elytra_glide"),
    ("minecraft:entity_damaged", "minecraft:entity_damage"),
    ("minecraft:entity_dying", "minecraft:entity_die"),
    ("minecraft:entity_killed", "minecraft:entity_die"),
    ("minecraft:mob_interact", "minecraft:entity_interact"),
    ("minecraft:ravager_roar", "minecraft:entity_roar"),
    ("minecraft:ring_bell", "minecraft:block_change"),
    ("minecraft:shulker_close", "minecraft:container_close"),
    ("minecraft:shulker_open", "minecraft:container_open"),
    ("minecraft:wolf_shaking", "minecraft:entity_shake"),
];

/// REVERSE of GAME_EVENT_RENAMES: `(new, old, ambiguous)`. Listed in the *same*
/// order as the forward table so that, for new ids the forward MERGED several old
/// ids into, the canonical (first-listed) preimage is the one we restore.
///
/// `ambiguous = true` marks a new id that the forward produced from >1 distinct
/// old id (`block_activate`, `block_deactivate`, `entity_die`): modern data keeps
/// no discriminator, so the exact original is unrecoverable (rule 11) and we
/// pick the canonical preimage + report loss. The remaining new ids each have a
/// single preimage in the table, so their inverse is exact (bucket B, lossless).
/// `block_change` is special: it is *also* a genuine pre-existing game event, so
/// a modern `block_change` may be either the renamed `ring_bell` or an original
/// `block_change`; reversing it to `ring_bell` is best-effort, hence flagged.
const GAME_EVENT_REVERSES: &[(&str, &str, bool)] = &[
    ("minecraft:block_activate", "minecraft:block_press", true), // also <- block_switch
    ("minecraft:block_deactivate", "minecraft:block_unpress", true), // also <- block_unswitch
    ("minecraft:drink", "minecraft:drinking_finish", false),
    ("minecraft:elytra_glide", "minecraft:elytra_free_fall", false),
    ("minecraft:entity_damage", "minecraft:entity_damaged", false),
    ("minecraft:entity_die", "minecraft:entity_dying", true), // also <- entity_killed
    ("minecraft:entity_interact", "minecraft:mob_interact", false),
    ("minecraft:entity_roar", "minecraft:ravager_roar", false),
    ("minecraft:block_change", "minecraft:ring_bell", true), // block_change also a real pre-rename event
    ("minecraft:container_close", "minecraft:shulker_close", false),
    ("minecraft:container_open", "minecraft:shulker_open", false),
    ("minecraft:entity_shake", "minecraft:wolf_shaking", false),
];

pub fn register(reg: &mut RegistryBuilder) {
    reg.game_event_name.add_converter(
        VERSION,
        0,
        Box::new(|val: &mut NbtValue, _from, _to| {
            if let NbtValue::String(s) = val {
                // Match Java: look up the namespace-corrected name.
                let corrected = correct_namespace_or_null(s);
                let key: &str = corrected.as_deref().unwrap_or(s.as_str());
                if let Some((_, new)) = GAME_EVENT_RENAMES.iter().find(|(old, _)| *old == key) {
                    *s = (*new).to_string();
                }
            }
        }),
    );

    // REVERSE: new game-event id -> old id. The forward `.add_converter` is a
    // hand-written value rename (not `map_renamer`), so it is NOT auto-inverted —
    // we supply the inverse explicitly. Namespace-correct the input first, mirroring
    // the forward lookup, so an unnamespaced modern id still matches.
    reg.game_event_name.add_reverse_converter(
        VERSION,
        0,
        Box::new(|val: &mut NbtValue, _from, _to| {
            if let NbtValue::String(s) = val {
                let corrected = correct_namespace_or_null(s);
                let key: &str = corrected.as_deref().unwrap_or(s.as_str());
                if let Some((_, old, ambiguous)) =
                    GAME_EVENT_REVERSES.iter().find(|(new, _, _)| *new == key)
                {
                    if *ambiguous {
                        report_loss(
                            VERSION,
                            LossKind::RenameAmbiguous,
                            Severity::Approximated,
                            "game-event id merged in V3084; restored canonical preimage",
                        );
                    }
                    *s = (*old).to_string();
                }
            }
        }),
    );
}
