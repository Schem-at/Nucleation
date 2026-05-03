//! Storage layer for tile entities (BlockEntity) within a region.
//!
//! Replaces the previous `FxHashMap<(i32,i32,i32), BlockEntity>` with a
//! palette-indexed layout: a `Vec<Arc<BlockEntity>>` holds the unique
//! templates, and a `FxHashMap<(i32,i32,i32), u32>` maps each position to
//! a palette index.
//!
//! ## Why
//!
//! When placing many copies of the same tile entity (chests with identical
//! contents, signs with the same text), the previous storage paid the full
//! per-position cost of `BlockEntity::clone()` and a HashMap insert keyed
//! by an owned BlockEntity value. The palette layout collapses repeated
//! templates: `insert_template(positions, Arc<BE>)` performs ONE palette
//! push and N small u32 inserts.
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
    /// Palette of unique BlockEntity templates. Each insert (without
    /// `insert_template`) appends a fresh entry; templates inserted via
    /// `insert_template` are shared by all positions referencing them.
    palette: Vec<Arc<BlockEntity>>,
    /// Position -> palette index lookup.
    by_pos: FxHashMap<(i32, i32, i32), u32>,
}

impl BlockEntityStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a tile entity at the given position. Each call appends a new
    /// palette entry — use `insert_template` if you want template sharing.
    /// Returns the previously-stored BlockEntity at this position, if any.
    #[inline]
    pub fn insert(&mut self, pos: (i32, i32, i32), be: BlockEntity) -> Option<BlockEntity> {
        let idx = self.palette.len() as u32;
        self.palette.push(Arc::new(be));
        let prev = self.by_pos.insert(pos, idx);
        prev.map(|old_idx| (*self.palette[old_idx as usize]).clone())
    }

    /// Hot batch path: store ONE shared template at all `positions`.
    /// Skips the per-position BlockEntity clone and palette push entirely.
    pub fn insert_template(&mut self, positions: &[(i32, i32, i32)], template: Arc<BlockEntity>) {
        if positions.is_empty() {
            return;
        }
        let idx = self.palette.len() as u32;
        self.palette.push(template);
        self.by_pos.reserve(positions.len());
        for &pos in positions {
            self.by_pos.insert(pos, idx);
        }
    }

    #[inline]
    pub fn get(&self, pos: &(i32, i32, i32)) -> Option<&BlockEntity> {
        self.by_pos
            .get(pos)
            .map(|&idx| self.palette[idx as usize].as_ref())
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

    pub fn remove(&mut self, pos: &(i32, i32, i32)) -> Option<BlockEntity> {
        self.by_pos
            .remove(pos)
            .map(|idx| (*self.palette[idx as usize]).clone())
    }

    pub fn clear(&mut self) {
        self.palette.clear();
        self.by_pos.clear();
    }

    pub fn reserve(&mut self, n: usize) {
        self.by_pos.reserve(n);
    }

    /// Iterate `(position, &BlockEntity)` pairs. **The position from the
    /// iterator is the canonical position; do not use `be.position` for
    /// serialization** — it may be stale on shared templates.
    pub fn iter(&self) -> impl Iterator<Item = ((i32, i32, i32), &BlockEntity)> + '_ {
        self.by_pos
            .iter()
            .map(|(&pos, &idx)| (pos, self.palette[idx as usize].as_ref()))
    }

    /// Iterate just the values. Templates shared across positions are
    /// yielded once per position.
    pub fn values(&self) -> impl Iterator<Item = &BlockEntity> + '_ {
        self.by_pos
            .values()
            .map(|&idx| self.palette[idx as usize].as_ref())
    }

    pub fn keys(&self) -> impl Iterator<Item = &(i32, i32, i32)> + '_ {
        self.by_pos.keys()
    }

    /// Remap every stored position via `f` without touching the palette.
    /// Far cheaper than `drain` + `insert` when transforms only shuffle
    /// coordinates (flip, rotate, translate). Templates remain shared and
    /// no NBT cloning happens.
    pub fn remap_positions<F>(&mut self, f: F)
    where
        F: Fn((i32, i32, i32)) -> (i32, i32, i32),
    {
        let old = std::mem::take(&mut self.by_pos);
        self.by_pos.reserve(old.len());
        for (pos, idx) in old {
            let new_pos = f(pos);
            self.by_pos.insert(new_pos, idx);
        }
    }

    /// Drain all entries, materializing each into an owned BlockEntity with
    /// its position field set from the storage key. Used by transforms
    /// (rotate/flip) that need ownership to mutate.
    pub fn drain(&mut self) -> Vec<((i32, i32, i32), BlockEntity)> {
        let palette = std::mem::take(&mut self.palette);
        let by_pos = std::mem::take(&mut self.by_pos);
        by_pos
            .into_iter()
            .map(|(pos, idx)| {
                let mut be = (*palette[idx as usize]).clone();
                be.position = pos;
                (pos, be)
            })
            .collect()
    }

    /// Bulk-insert. Each entry adds a new palette template — use
    /// `insert_template` for template-shared batches.
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = ((i32, i32, i32), BlockEntity)>,
    {
        let iter = iter.into_iter();
        let (lo, _) = iter.size_hint();
        self.by_pos.reserve(lo);
        self.palette.reserve(lo);
        for (pos, be) in iter {
            self.insert(pos, be);
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
        store.palette.reserve(entries.len());
        store.by_pos.reserve(entries.len());
        for be in entries {
            let pos = be.position;
            let idx = store.palette.len() as u32;
            store.palette.push(Arc::new(be));
            store.by_pos.insert(pos, idx);
        }
        Ok(store)
    }
}
