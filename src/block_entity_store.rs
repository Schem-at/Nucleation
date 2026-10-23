//! Storage layer for tile entities (BlockEntity) within a region.
//!
//! A `FxHashMap<(i32,i32,i32), BlockEntity>`. Each entry is its own
//! `BlockEntity` with the correct `position`; the NBT payload is an
//! `Arc<NbtMap>`, so many copies of the same chest share one payload and
//! copying an entity costs a reference count, not a deep NBT clone.
//!
//! ## API shape
//!
//! Mirrors the HashMap API closely so most existing call sites work
//! unchanged: `.iter()`, `.values()`, `.keys()`, `.len()`, `.is_empty()`,
//! `.contains_key()`, `.get()`, `.insert()`, `.remove()`, `.clear()`,
//! `.reserve()`, `.drain()`, `.extend()`.

use crate::block_entity::BlockEntity;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct BlockEntityStore {
    by_pos: FxHashMap<(i32, i32, i32), BlockEntity>,
}

impl BlockEntityStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a tile entity at the given position (its `position` is set to
    /// `pos`). Returns the previously-stored BlockEntity at this position, if
    /// any.
    #[inline]
    pub fn insert(&mut self, pos: (i32, i32, i32), mut be: BlockEntity) -> Option<BlockEntity> {
        be.position = pos;
        self.by_pos.insert(pos, be)
    }

    /// Insert a tile entity, dropping any previous one.
    #[inline]
    pub fn set(&mut self, pos: (i32, i32, i32), be: BlockEntity) {
        self.insert(pos, be);
    }

    /// Store a copy of `template` at every position, each with its own
    /// `position` and all sharing the template's NBT payload.
    pub fn insert_template(&mut self, positions: &[(i32, i32, i32)], template: Arc<BlockEntity>) {
        self.by_pos.reserve(positions.len());
        for &position in positions {
            self.by_pos.insert(
                position,
                BlockEntity {
                    id: template.id.clone(),
                    position,
                    nbt: Arc::clone(&template.nbt),
                },
            );
        }
    }

    #[inline]
    pub fn get(&self, pos: &(i32, i32, i32)) -> Option<&BlockEntity> {
        self.by_pos.get(pos)
    }

    #[inline]
    pub fn contains_key(&self, pos: &(i32, i32, i32)) -> bool {
        self.by_pos.contains_key(pos)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.by_pos.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.by_pos.is_empty()
    }

    #[inline]
    pub fn remove(&mut self, pos: &(i32, i32, i32)) -> Option<BlockEntity> {
        self.by_pos.remove(pos)
    }

    /// Remove the entity at `pos`. Returns whether one was present.
    #[inline]
    pub fn discard(&mut self, pos: &(i32, i32, i32)) -> bool {
        self.by_pos.remove(pos).is_some()
    }

    /// Keep only the entries for which `keep` returns true.
    pub fn retain<F>(&mut self, mut keep: F)
    where
        F: FnMut((i32, i32, i32), &BlockEntity) -> bool,
    {
        self.by_pos.retain(|&pos, be| keep(pos, be));
    }

    pub fn clear(&mut self) {
        self.by_pos.clear();
    }

    pub fn reserve(&mut self, n: usize) {
        self.by_pos.reserve(n);
    }

    /// Iterate `(position, &BlockEntity)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = ((i32, i32, i32), &BlockEntity)> + '_ {
        self.by_pos.iter().map(|(&pos, be)| (pos, be))
    }

    pub fn values(&self) -> impl Iterator<Item = &BlockEntity> + '_ {
        self.by_pos.values()
    }

    pub fn keys(&self) -> impl Iterator<Item = &(i32, i32, i32)> + '_ {
        self.by_pos.keys()
    }

    /// Remap every stored position via `f` (flip, rotate, translate), keeping
    /// each entity's `position` in step. No NBT cloning happens.
    pub fn remap_positions<F>(&mut self, f: F)
    where
        F: Fn((i32, i32, i32)) -> (i32, i32, i32),
    {
        let old = std::mem::take(&mut self.by_pos);
        self.by_pos.reserve(old.len());
        for (pos, mut be) in old {
            let new_pos = f(pos);
            be.position = new_pos;
            self.by_pos.insert(new_pos, be);
        }
    }

    /// Drain all entries. Used by transforms (rotate/flip) that need
    /// ownership to mutate.
    pub fn drain(&mut self) -> Vec<((i32, i32, i32), BlockEntity)> {
        std::mem::take(&mut self.by_pos).into_iter().collect()
    }

    /// Bulk-insert, setting each entity's `position` to its key.
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = ((i32, i32, i32), BlockEntity)>,
    {
        let iter = iter.into_iter();
        let (lo, _) = iter.size_hint();
        self.by_pos.reserve(lo);
        for (pos, be) in iter {
            self.insert(pos, be);
        }
    }
}

impl Serialize for BlockEntityStore {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let entries: Vec<&BlockEntity> = self.by_pos.values().collect();
        entries.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BlockEntityStore {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let entries: Vec<BlockEntity> = Vec::deserialize(deserializer)?;
        let mut store = BlockEntityStore::new();
        store.reserve(entries.len());
        for be in entries {
            store.set(be.position, be);
        }
        Ok(store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stored_entity_carries_its_own_position() {
        let mut store = BlockEntityStore::new();
        let template = Arc::new(BlockEntity::new("minecraft:chest".to_string(), (9, 9, 9)));
        let positions = [(1, 0, 0), (2, 0, 0), (3, 0, 0)];
        store.insert_template(&positions, template.clone());
        store.insert((7, 7, 7), BlockEntity::new("minecraft:barrel".to_string(), (0, 0, 0)));
        store.remap_positions(|(x, y, z)| (x + 10, y, z));
        for (pos, be) in store.iter() {
            assert_eq!(be.position, pos);
        }
        // Copies share the template's NBT payload.
        assert!(Arc::ptr_eq(&store.get(&(11, 0, 0)).unwrap().nbt, &template.nbt));
    }
}
