//! Block states, interned to small integers.
//!
//! # Why interning
//!
//! A schematic's natural block representation is a name plus a property map —
//! readable, and hopeless in a tick loop, where every neighbour read would chase
//! a pointer and every comparison would walk a string. Here a block is a
//! [`StateId`]: two bytes, `Copy`, comparable in one instruction. Conversion
//! happens once at load.
//!
//! # Why the registry is populated from outside
//!
//! This crate deliberately knows nothing about Minecraft's block list. The
//! caller supplies the states it uses, so `mc-tick` stays free of both a data
//! dependency and a version dependency — the same engine can run 26.2 data
//! today and something else later without a rebuild.

use std::collections::HashMap;

/// An interned block state.
///
/// Two bytes, so a bounded region is a dense `Vec<StateId>`. [`StateId::AIR`] is
/// guaranteed to be zero, which makes a zeroed buffer a valid empty world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateId(pub u16);

impl StateId {
    /// Air, and the value a freshly allocated world is filled with.
    ///
    /// Fixed at zero so `vec![StateId::AIR; n]` is a memset and every registry
    /// agrees on what an empty block is without consulting a table.
    pub const AIR: StateId = StateId(0);

    /// The raw index.
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// Errors from interning.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StateError {
    /// More distinct states than a `u16` can address.
    ///
    /// Vanilla has on the order of 30k block states, so a real world will not
    /// reach this. Hitting it means something is generating states in a loop.
    #[error("more than {} distinct block states", u16::MAX as u32 + 1)]
    TooManyStates,
}

/// Maps block state descriptors to [`StateId`]s and back.
///
/// A descriptor is whatever string the caller uses to identify a state, e.g.
/// `minecraft:repeater[delay=2,facing=north,powered=false]`. This crate treats
/// it as opaque: it never parses it, and equal strings mean the same state.
#[derive(Debug, Clone)]
pub struct StateRegistry {
    descriptors: Vec<String>,
    lookup: HashMap<String, StateId>,
}

impl StateRegistry {
    /// A registry containing only air.
    pub fn new() -> Self {
        let air = "minecraft:air".to_string();
        let mut lookup = HashMap::new();
        lookup.insert(air.clone(), StateId::AIR);
        Self {
            descriptors: vec![air],
            lookup,
        }
    }

    /// The id for `descriptor`, interning it if new.
    pub fn intern(&mut self, descriptor: &str) -> Result<StateId, StateError> {
        if let Some(&id) = self.lookup.get(descriptor) {
            return Ok(id);
        }
        let next = u16::try_from(self.descriptors.len()).map_err(|_| StateError::TooManyStates)?;
        let id = StateId(next);
        self.descriptors.push(descriptor.to_string());
        self.lookup.insert(descriptor.to_string(), id);
        Ok(id)
    }

    /// The id for `descriptor` if already interned.
    pub fn get(&self, descriptor: &str) -> Option<StateId> {
        self.lookup.get(descriptor).copied()
    }

    /// The descriptor an id refers to.
    pub fn descriptor(&self, id: StateId) -> Option<&str> {
        self.descriptors.get(id.0 as usize).map(String::as_str)
    }

    /// How many distinct states are interned, air included.
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    /// Whether the registry holds nothing but air.
    pub fn is_empty(&self) -> bool {
        self.descriptors.len() <= 1
    }
}

impl Default for StateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn air_is_zero_in_a_fresh_registry() {
        // A zeroed buffer has to be a valid empty world; that only holds if air
        // is genuinely id 0.
        let registry = StateRegistry::new();
        assert_eq!(StateId::AIR.raw(), 0);
        assert_eq!(registry.descriptor(StateId::AIR), Some("minecraft:air"));
    }

    #[test]
    fn interning_is_stable_and_reversible() {
        let mut registry = StateRegistry::new();
        let wire = registry.intern("minecraft:redstone_wire[power=0]").unwrap();
        let again = registry.intern("minecraft:redstone_wire[power=0]").unwrap();
        assert_eq!(wire, again, "same descriptor must intern to the same id");
        assert_eq!(
            registry.descriptor(wire),
            Some("minecraft:redstone_wire[power=0]")
        );
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn states_differing_only_in_properties_are_distinct() {
        // Powered and unpowered are different blocks to the tick loop; collapsing
        // them would silently break every gate.
        let mut registry = StateRegistry::new();
        let off = registry.intern("minecraft:redstone_wire[power=0]").unwrap();
        let on = registry.intern("minecraft:redstone_wire[power=15]").unwrap();
        assert_ne!(off, on);
    }

    #[test]
    fn unknown_descriptors_and_ids_return_none() {
        let registry = StateRegistry::new();
        assert_eq!(registry.get("minecraft:stone"), None);
        assert_eq!(registry.descriptor(StateId(999)), None);
    }

    #[test]
    fn a_fresh_registry_reports_empty() {
        let mut registry = StateRegistry::new();
        assert!(registry.is_empty());
        registry.intern("minecraft:stone").unwrap();
        assert!(!registry.is_empty());
    }
}
