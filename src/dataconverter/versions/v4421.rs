//! V4421 (1.21.5 + 96) — no schematic-relevant registrations.
//!
//! V4421.java only carries a comment ("happy_ghast is simple entity") and
//! registers nothing. Kept as an empty registrar so the version is wired in
//! sequence; there is nothing to convert.

use super::super::registry::RegistryBuilder;

#[allow(dead_code)]
const VERSION: i32 = 4421;

pub fn register(_reg: &mut RegistryBuilder) {}
