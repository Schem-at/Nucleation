//! Storage layer for tile entities (BlockEntity) within a region.
//!
//! A `FxHashMap<(i32,i32,i32), Arc<BlockEntity>>`: each position owns a
//! reference-counted entity.
//!
//! ## Why
//!
//! When placing many copies of the same tile entity (chests with identical
//! contents, signs with the same text), a plain `BlockEntity` map pays the
//! full per-position cost of `BlockEntity::clone()`. Sharing one `Arc`
//! across positions makes `insert_template(positions, Arc<BE>)` N reference
//! count increments instead of N deep NBT clones.
//!
//! Entries are freed as soon as no position references them. (An earlier
//! palette-indexed layout appended a template on every insert and never
//! reclaimed it, so overwriting one chest a million times kept a million
//! dead templates alive.) `insert` and `remove` hand back the previous
//! entity by moving it out of its `Arc` when it was not shared, so replacing
//! or clearing a block entity does not deep-clone its NBT.
//!
//! ## Position invariant
//!
//! `BlockEntity::position` is the on-disk source of truth for serialization,
//! but it MAY be stale for templates shared across positions. Serializers
//! that emit positions must use the storage key (returned by `iter()`), not
//! `be.position`. The included `serialize_block_entity_store` helper does
//! this override automatically.
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
    by_pos: FxHashMap<(i32, i32, i32), Arc<BlockEntity>>,
}

/// Take ownership of a stored entity, cloning only if another position
/// still shares it.
#[inline]
fn into_owned(be: Arc<BlockEntity>) -> BlockEntity {
    Arc::try_unwrap(be).unwrap_or_else(|shared| (*shared).clone())
}

impl BlockEntityStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a tile entity at the given position.
    /// Returns the previously-stored BlockEntity at this position, if any.
    #[inline]
    pub fn insert(&mut self, pos: (i32, i32, i32), be: BlockEntity) -> Option<BlockEntity> {
        self.by_pos.insert(pos, Arc::new(be)).map(into_owned)
    }

    /// Insert a tile entity, dropping (not materializing) any previous one.
    /// Use this instead of `insert` when the old value is not needed.
    #[inline]
    pub fn set(&mut self, pos: (i32, i32, i32), be: BlockEntity) {
        self.by_pos.insert(pos, Arc::new(be));
    }

    /// Hot batch path: store ONE shared template at all `positions`.
    /// Skips the per-position BlockEntity clone entirely.
    pub fn insert_template(&mut self, positions: &[(i32, i32, i32)], template: Arc<BlockEntity>) {
        self.by_pos.reserve(positions.len());
        for &pos in positions {
            self.by_pos.insert(pos, Arc::clone(&template));
        }
    }

    #[inline]
    pub fn get(&self, pos: &(i32, i32, i32)) -> Option<&BlockEntity> {
        self.by_pos.get(pos).map(Arc::as_ref)
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
        self.by_pos.remove(pos).map(into_owned)
    }

    /// Remove the entity at `pos` without materializing it. Returns whether
    /// one was present. Use this when the old value is not needed.
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

    /// Iterate `(position, &BlockEntity)` pairs. **The position from the
    /// iterator is the canonical position; do not use `be.position` for
    /// serialization** — it may be stale on shared templates.
    pub fn iter(&self) -> impl Iterator<Item = ((i32, i32, i32), &BlockEntity)> + '_ {
        self.by_pos.iter().map(|(&pos, be)| (pos, be.as_ref()))
    }

    /// Iterate just the values. Templates shared across positions are
    /// yielded once per position.
    pub fn values(&self) -> impl Iterator<Item = &BlockEntity> + '_ {
        self.by_pos.values().map(Arc::as_ref)
    }

    pub fn keys(&self) -> impl Iterator<Item = &(i32, i32, i32)> + '_ {
        self.by_pos.keys()
    }

    /// Remap every stored position via `f`. Far cheaper than `drain` +
    /// `insert` when transforms only shuffle coordinates (flip, rotate,
    /// translate). Templates remain shared and no NBT cloning happens.
    pub fn remap_positions<F>(&mut self, f: F)
    where
        F: Fn((i32, i32, i32)) -> (i32, i32, i32),
    {
        let old = std::mem::take(&mut self.by_pos);
        self.by_pos.reserve(old.len());
        for (pos, be) in old {
            self.by_pos.insert(f(pos), be);
        }
    }

    /// Drain all entries, materializing each into an owned BlockEntity with
    /// its position field set from the storage key. Used by transforms
    /// (rotate/flip) that need ownership to mutate.
    pub fn drain(&mut self) -> Vec<((i32, i32, i32), BlockEntity)> {
        std::mem::take(&mut self.by_pos)
            .into_iter()
            .map(|(pos, be)| {
                let mut be = into_owned(be);
                be.position = pos;
                (pos, be)
            })
            .collect()
    }

    /// Bulk-insert. Use `insert_template` for template-shared batches.
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = ((i32, i32, i32), BlockEntity)>,
    {
        let iter = iter.into_iter();
        let (lo, _) = iter.size_hint();
        self.by_pos.reserve(lo);
        for (pos, be) in iter {
            self.by_pos.insert(pos, Arc::new(be));
        }
    }
}

impl Serialize for BlockEntityStore {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Emit a Vec<BlockEntity> where each entry's position field is set
        // to the storage key — needed because templates shared across
        // positions store stale position fields.
        let owned: Vec<BlockEntity> = self
            .iter()
            .map(|(pos, be)| {
                let mut cloned = be.clone();
                cloned.position = pos;
                cloned
            })
            .collect();
        owned.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BlockEntityStore {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let entries: Vec<BlockEntity> = Vec::deserialize(deserializer)?;
        let mut store = BlockEntityStore::new();
        store.by_pos.reserve(entries.len());
        for be in entries {
            store.by_pos.insert(be.position, Arc::new(be));
        }
        Ok(store)
    }
}
