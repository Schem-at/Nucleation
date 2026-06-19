//! V143 (15w44b) — schematic-relevant subset of `V143.java`.
//!
//! Renames the `TippedArrow` entity to `Arrow`. The Java mapping is a function
//! (only `TippedArrow` matches) rather than a table, so we drive
//! `register_entity_rename` with a closure-based `Renamer`. This renames the
//! entity `id` field and the ENTITY_NAME value type.

use std::sync::Arc;

use super::super::helpers::{register_entity_rename, Renamer};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 143;

pub fn register(reg: &mut RegistryBuilder) {
    let renamer: Renamer = Arc::new(|id: &str| {
        if id == "TippedArrow" {
            Some("Arrow".to_string())
        } else {
            None
        }
    });
    register_entity_rename(reg, VERSION, renamer);
}
