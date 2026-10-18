//! Storage layer for tile entities (BlockEntity) within a region.
//!
//! A palette of unique `Arc<BlockEntity>` templates, each with the number of
//! positions that reference it, plus a `FxHashMap<(i32,i32,i32), u32>` from
//! position to palette slot.
//!
//! ## Why
//!
//! When placing many copies of the same tile entity (chests with identical
//! contents, signs with the same text), a plain `BlockEntity` map pays the
//! full per-position cost of `BlockEntity::clone()` and stores a large value
//! per key. The palette collapses repeated templates:
//! `insert_template(positions, Arc<BE>)` performs ONE palette push and N
//! small u32 inserts.
//!
//! Slots are reference counted: when the last position referencing a
//! template is overwritten or removed, the template is dropped and its slot
//! reused. (An earlier version never released slots, so overwriting one chest
//! a million times kept a million dead templates alive.) `insert` and
//! `remove` hand back the previous entity by moving it out of its `Arc` when
//! nothing else shares it, so replacing or clearing a block entity does not
//! deep-clone its NBT.
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
    /// Templates and how many positions reference each. `None` is a free
    /// slot, listed in `free` for reuse.
    palette: Vec<Option<(Arc<BlockEntity>, u32)>>,
    free: Vec<u32>,
    /// Position -> palette slot.
    by_pos: FxHashMap<(i32, i32, i32), u32>,
}

/// Take ownership of a stored entity, cloning only if something else still
/// shares it.
#[inline]
fn into_owned(be: Arc<BlockEntity>) -> BlockEntity {
    Arc::try_unwrap(be).unwrap_or_else(|shared| (*shared).clone())
}

impl BlockEntityStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store `template` in a free (or new) slot with `refs` references.
    fn alloc(&mut self, template: Arc<BlockEntity>, refs: u32) -> u32 {
        match self.free.pop() {
            Some(slot) => {
                self.palette[slot as usize] = Some((template, refs));
                slot
            }
            None => {
                self.palette.push(Some((template, refs)));
                (self.palette.len() - 1) as u32
            }
        }
    }

    /// Drop one reference to `slot`, returning the template if that was the
    /// last one (the slot is then free).
    fn release(&mut self, slot: u32) -> Option<Arc<BlockEntity>> {
        let entry = self.palette[slot as usize]
            .as_mut()
            .expect("live block-entity slot");
        entry.1 -= 1;
        if entry.1 > 0 {
            return None;
        }
        self.free.push(slot);
        self.palette[slot as usize].take().map(|(template, _)| template)
    }

    /// Drop one reference to `slot` and return the entity it held.
    fn take(&mut self, slot: u32) -> BlockEntity {
        let shared = &self.palette[slot as usize]
            .as_ref()
            .expect("live block-entity slot")
            .0;
        if self.palette[slot as usize].as_ref().is_some_and(|e| e.1 > 1) {
            let copy = (**shared).clone();
            self.release(slot);
            return copy;
        }
        into_owned(self.release(slot).expect("last reference"))
    }

    #[inline]
    fn template(&self, slot: u32) -> &BlockEntity {
        &self.palette[slot as usize]
            .as_ref()
            .expect("live block-entity slot")
            .0
    }

    /// Insert a tile entity at the given position.
    /// Returns the previously-stored BlockEntity at this position, if any.
    #[inline]
    pub fn insert(&mut self, pos: (i32, i32, i32), be: BlockEntity) -> Option<BlockEntity> {
        let slot = self.alloc(Arc::new(be), 1);
        self.by_pos.insert(pos, slot).map(|old| self.take(old))
    }

    /// Insert a tile entity, dropping (not materializing) any previous one.
    /// Use this instead of `insert` when the old value is not needed.
    #[inline]
    pub fn set(&mut self, pos: (i32, i32, i32), be: BlockEntity) {
        let slot = self.alloc(Arc::new(be), 1);
        if let Some(old) = self.by_pos.insert(pos, slot) {
            self.release(old);
        }
    }

    /// Hot batch path: store ONE shared template at all `positions`.
    /// Skips the per-position BlockEntity clone and palette push entirely.
    pub fn insert_template(&mut self, positions: &[(i32, i32, i32)], template: Arc<BlockEntity>) {
        if positions.is_empty() {
            return;
        }
        let slot = self.alloc(template, positions.len() as u32);
        self.by_pos.reserve(positions.len());
        for &pos in positions {
            if let Some(old) = self.by_pos.insert(pos, slot) {
                self.release(old);
            }
        }
    }

    #[inline]
    pub fn get(&self, pos: &(i32, i32, i32)) -> Option<&BlockEntity> {
        self.by_pos.get(pos).map(|&slot| self.template(slot))
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
        self.by_pos.remove(pos).map(|slot| self.take(slot))
    }

    /// Remove the entity at `pos` without materializing it. Returns whether
    /// one was present. Use this when the old value is not needed.
    #[inline]
    pub fn discard(&mut self, pos: &(i32, i32, i32)) -> bool {
        match self.by_pos.remove(pos) {
            Some(slot) => {
                self.release(slot);
                true
            }
            None => false,
        }
    }

    /// Keep only the entries for which `keep` returns true.
    pub fn retain<F>(&mut self, mut keep: F)
    where
        F: FnMut((i32, i32, i32), &BlockEntity) -> bool,
    {
        let palette = &self.palette;
        let mut dropped = Vec::new();
        self.by_pos.retain(|&pos, &mut slot| {
            let template = &palette[slot as usize]
                .as_ref()
                .expect("live block-entity slot")
                .0;
            let kept = keep(pos, template);
            if !kept {
                dropped.push(slot);
            }
            kept
        });
        for slot in dropped {
            self.release(slot);
        }
    }

    pub fn clear(&mut self) {
        self.palette.clear();
        self.free.clear();
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
            .map(|(&pos, &slot)| (pos, self.template(slot)))
    }

    /// Iterate just the values. Templates shared across positions are
    /// yielded once per position.
    pub fn values(&self) -> impl Iterator<Item = &BlockEntity> + '_ {
        self.by_pos.values().map(|&slot| self.template(slot))
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
        for (pos, slot) in old {
            if let Some(replaced) = self.by_pos.insert(f(pos), slot) {
                self.release(replaced);
            }
        }
    }

    /// Drain all entries, materializing each into an owned BlockEntity with
    /// its position field set from the storage key. Used by transforms
    /// (rotate/flip) that need ownership to mutate.
    pub fn drain(&mut self) -> Vec<((i32, i32, i32), BlockEntity)> {
        let by_pos = std::mem::take(&mut self.by_pos);
        let drained = by_pos
            .into_iter()
            .map(|(pos, slot)| {
                let mut be = self.take(slot);
                be.position = pos;
                (pos, be)
            })
            .collect();
        self.clear();
        drained
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
            self.set(pos, be);
        }
    }

    /// Number of live templates (test support for the slot accounting).
    #[cfg(test)]
    fn live_templates(&self) -> usize {
        self.palette.iter().filter(|slot| slot.is_some()).count()
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

    fn chest(items: i8) -> BlockEntity {
        BlockEntity::new("minecraft:chest".to_string(), (0, 0, 0)).with_nbt_data(
            "Count".to_string(),
            crate::utils::NbtValue::Byte(items),
        )
    }

    #[test]
    fn overwrites_and_removals_release_templates() {
        let mut store = BlockEntityStore::new();
        for i in 0..1000 {
            store.set((0, 0, 0), chest((i % 100) as i8));
        }
        assert_eq!(store.len(), 1);
        assert_eq!(store.live_templates(), 1);

        let template = Arc::new(chest(7));
        let positions: Vec<_> = (0..50).map(|x| (x, 0, 0)).collect();
        store.insert_template(&positions, template.clone());
        assert_eq!(store.live_templates(), 1, "the overwritten chest is freed");
        for &pos in &positions[..49] {
            assert!(store.discard(&pos));
        }
        assert_eq!(store.live_templates(), 1);
        // The last reference moves out; the caller's Arc still shares it.
        assert_eq!(store.remove(&(49, 0, 0)), Some(chest(7)));
        assert_eq!(store.live_templates(), 0);
        assert!(store.is_empty());

        // Freed slots are reused rather than growing the palette.
        store.set((1, 1, 1), chest(1));
        store.set((2, 2, 2), chest(2));
        assert_eq!(store.palette.len(), 2);
    }

    #[test]
    fn shared_templates_survive_partial_removal_and_retain() {
        let mut store = BlockEntityStore::new();
        let positions: Vec<_> = (0..10).map(|x| (x, 0, 0)).collect();
        store.insert_template(&positions, Arc::new(chest(3)));
        store.retain(|(x, _, _), _| x % 2 == 0);
        assert_eq!(store.len(), 5);
        assert_eq!(store.live_templates(), 1);
        assert_eq!(store.get(&(4, 0, 0)), Some(&chest(3)));
        let drained = store.drain();
        assert_eq!(drained.len(), 5);
        assert!(drained.iter().all(|(pos, be)| be.position == *pos));
        assert_eq!(store.live_templates(), 0);
    }
}
