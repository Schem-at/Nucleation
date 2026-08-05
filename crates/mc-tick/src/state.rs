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
    /// Which *block* each state belongs to — the descriptor with its
    /// properties stripped, interned separately.
    ///
    /// Vanilla distinguishes `BlockState` from `Block` constantly, and one
    /// place it matters is `doBlockEvent`, which refuses an event whose
    /// position no longer holds the **Block** it was queued for (the state may
    /// differ freely: a piston that became `extended=true` still passes).
    block_of: Vec<u16>,
    block_lookup: HashMap<String, u16>,
}

impl StateRegistry {
    /// A registry containing only air.
    pub fn new() -> Self {
        let air = "minecraft:air".to_string();
        let mut lookup = HashMap::new();
        lookup.insert(air.clone(), StateId::AIR);
        let mut block_lookup = HashMap::new();
        block_lookup.insert("minecraft:air".to_string(), 0u16);
        Self {
            descriptors: vec![air],
            lookup,
            block_of: vec![0],
            block_lookup,
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
        let name = descriptor.split('[').next().unwrap_or(descriptor);
        let block = match self.block_lookup.get(name) {
            Some(block) => *block,
            None => {
                let block = u16::try_from(self.block_lookup.len())
                    .map_err(|_| StateError::TooManyStates)?;
                self.block_lookup.insert(name.to_string(), block);
                block
            }
        };
        self.block_of.push(block);
        Ok(id)
    }

    /// The block `state` belongs to, ignoring its properties.
    pub fn block_of(&self, state: StateId) -> u16 {
        self.block_of.get(state.raw() as usize).copied().unwrap_or(0)
    }

    /// Whether two states are the same **block** — `BlockState.is(Block)`.
    pub fn same_block(&self, a: StateId, b: StateId) -> bool {
        self.block_of(a) == self.block_of(b)
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

/// A set of block states, indexed rather than searched.
///
/// The rules table classifies states a dozen ways — conductor, full cube,
/// comparator, slime, immovable — and the redstone hot path asks those
/// questions per neighbour, per block, per tick. Held as `Vec<StateId>` and
/// answered with `contains`, each question was a linear scan of every state
/// wearing that label; a wire-and-comparator build made construction alone
/// take 88 ms where a larger piston build took 2.7 ms.
///
/// `StateId` is a dense `u16`, so a bitset answers the same question by
/// indexing. `push` and `contains` keep the shapes `Vec` had, which is what
/// makes the swap a type change rather than a rewrite.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StateSet {
    bits: Vec<u64>,
}

impl StateSet {
    /// Add `state` to the set.
    pub fn insert(&mut self, state: StateId) {
        let index = state.0 as usize;
        let word = index / 64;
        if word >= self.bits.len() {
            self.bits.resize(word + 1, 0);
        }
        self.bits[word] |= 1u64 << (index % 64);
    }

    /// Alias for [`StateSet::insert`], so `Vec::push` call sites still read.
    pub fn push(&mut self, state: StateId) {
        self.insert(state);
    }

    /// Whether `state` is in the set.
    pub fn contains(&self, state: &StateId) -> bool {
        let index = state.0 as usize;
        self.bits.get(index / 64).is_some_and(|w| (w >> (index % 64)) & 1 == 1)
    }

    /// Whether nothing is in the set.
    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|w| *w == 0)
    }
}

impl FromIterator<StateId> for StateSet {
    fn from_iter<I: IntoIterator<Item = StateId>>(iter: I) -> Self {
        let mut set = Self::default();
        for state in iter {
            set.insert(state);
        }
        set
    }
}
