use crate::item::ItemStack;
use crate::utils::{NbtMap, NbtValue};
use quartz_nbt::NbtCompound;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// `BlockEntity` carries the tile-entity NBT for a positioned block (chest
/// inventory, sign text, jukebox record, etc.).
///
/// The `nbt` field is wrapped in an `Arc` so cloning a BlockEntity is a
/// refcount bump rather than a deep walk of the NBT tree. This is the
/// difference between ~250ns and ~5ns per clone, which is critical for
/// batch placement of identical tile entities (e.g. `set_blocks(positions,
/// "minecraft:chest{...}")` placing the same chest at N positions).
///
/// Mutations remain ergonomic: use `nbt_mut()` for in-place edits, which
/// uses `Arc::make_mut` to copy-on-write only when the NBT is actually
/// shared. For heavy mutation in tight loops, consider building a fresh
/// `NbtMap` and assigning via `set_nbt(...)`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockEntity {
    pub nbt: Arc<NbtMap>,
    pub id: String,
    pub position: (i32, i32, i32),
}

impl BlockEntity {
    pub fn new(id: String, position: (i32, i32, i32)) -> Self {
        BlockEntity {
            nbt: Arc::new(NbtMap::new()),
            id,
            position,
        }
    }

    /// Get a mutable reference to the NBT map. Copy-on-write: if other
    /// BlockEntity instances share this NBT (via `clone`), this clones
    /// it once and gives back a uniquely-owned mutable reference.
    #[inline]
    pub fn nbt_mut(&mut self) -> &mut NbtMap {
        Arc::make_mut(&mut self.nbt)
    }

    /// Replace the entire NBT map.
    #[inline]
    pub fn set_nbt(&mut self, nbt: NbtMap) {
        self.nbt = Arc::new(nbt);
    }

    pub fn with_nbt_data(mut self, key: String, value: NbtValue) -> Self {
        self.nbt_mut().insert(key, value);
        self
    }

    pub fn to_hashmap(&self) -> HashMap<String, NbtValue> {
        let mut map = HashMap::new();
        map.insert("Id".to_string(), NbtValue::String(self.id.clone()));
        map.insert(
            "Pos".to_string(),
            NbtValue::IntArray(vec![self.position.0, self.position.1, self.position.2]),
        );
        for (key, value) in self.nbt.iter() {
            map.insert(key.clone(), value.clone());
        }
        map
    }

    pub fn add_item_stack(&mut self, item: ItemStack) {
        let mut items = self
            .nbt
            .get("Items")
            .map(|items| {
                if let NbtValue::List(items) = items {
                    items.clone()
                } else {
                    vec![]
                }
            })
            .unwrap_or_default();
        items.push(item.to_nbt());
        self.nbt_mut()
            .insert("Items".to_string(), NbtValue::List(items));
    }

    pub fn create_chest(position: (i32, i32, i32), items: Vec<ItemStack>) -> BlockEntity {
        let mut chest = BlockEntity::new("minecraft:chest".to_string(), position);
        for item_stack in items {
            chest.add_item_stack(item_stack);
        }
        chest
    }

    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        let nbt_map = NbtMap::from_quartz_nbt(nbt);
        // The id key is `Id` in Sponge v3 / Nucleation's own litematic writer, but
        // vanilla and Litematica use lowercase `id` — accept both.
        let id = nbt_map
            .get("Id")
            .or_else(|| nbt_map.get("id"))
            .and_then(|v| v.as_string())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        // Position is `Pos` (int[3]) in Sponge/Nucleation, but Litematica's
        // TileEntities carry relative `x`/`y`/`z` ints instead — fall back to those
        // (the caller adds the region offset). Without this, every block entity in a
        // Litematica region parsed to (0,0,0) and collapsed onto the region origin.
        let position = nbt_map
            .get("Pos")
            .and_then(|v| v.as_int_array())
            .map(|v| (v[0], v[1], v[2]))
            .or_else(|| {
                match (
                    nbt_map.get("x").and_then(|v| v.as_i32()),
                    nbt_map.get("y").and_then(|v| v.as_i32()),
                    nbt_map.get("z").and_then(|v| v.as_i32()),
                ) {
                    (Some(x), Some(y), Some(z)) => Some((x, y, z)),
                    _ => None,
                }
            })
            .unwrap_or((0, 0, 0));
        BlockEntity {
            nbt: Arc::new(nbt_map),
            id,
            position,
        }
    }

    pub fn to_nbt(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        // Store the core BlockEntity fields
        nbt.insert("Id", NbtValue::String(self.id.clone()).to_quartz_nbt());
        nbt.insert(
            "Pos",
            NbtValue::IntArray(vec![self.position.0, self.position.1, self.position.2])
                .to_quartz_nbt(),
        );

        // Store the rest of the NBT data
        for (key, value) in self.nbt.iter() {
            nbt.insert(key, value.to_quartz_nbt());
        }
        nbt
    }

    /// Converts BlockEntity to NBT format for Sponge Schematic v3
    ///
    /// According to the Sponge Schematic v3 spec, block entities should have:
    /// - Id: string (required)
    /// - Pos: int[3] (required)
    /// - Data: compound (optional) - contains all block-specific NBT data
    ///
    /// This differs from to_nbt() which puts all data at root level (used for litematic)
    ///
    /// `data_version` is the target Minecraft data version: the empty
    /// `components`/`id` fields are a 1.20.5+ container convention, so they are
    /// only injected when the target is ≥ 1.20.5 (3837), or when the version is
    /// unknown (`None`, preserving the historical default). For an older target
    /// — e.g. a schematic reverse-converted down past the components rework —
    /// they are omitted so the export does not re-introduce shape the converter
    /// deliberately removed.
    pub fn to_nbt_v3(&self, data_version: Option<i32>) -> NbtCompound {
        const COMPONENTS_VERSION: i32 = 3837; // 1.20.5
        let inject_components = data_version.is_none_or(|dv| dv >= COMPONENTS_VERSION);

        let mut nbt = NbtCompound::new();

        // Required fields at root level
        nbt.insert("Id", NbtValue::String(self.id.clone()).to_quartz_nbt());
        nbt.insert(
            "Pos",
            NbtValue::IntArray(vec![self.position.0, self.position.1, self.position.2])
                .to_quartz_nbt(),
        );

        // Wrap all block-specific NBT data in a "Data" compound
        // Only add Data compound if there's actually NBT data to include
        let has_nbt_data = self.nbt.iter().next().is_some();
        if has_nbt_data {
            let mut data_compound = NbtCompound::new();

            // Add modern format fields for containers and jukeboxes (1.20.5+)
            let is_container = self.id.contains("barrel")
                || self.id.contains("chest")
                || self.id.contains("hopper")
                || self.id.contains("dropper")
                || self.id.contains("dispenser")
                || self.id.contains("jukebox");

            if is_container && inject_components {
                // Add empty components compound (required in 1.20.5+)
                data_compound.insert(
                    "components",
                    NbtValue::Compound(NbtMap::new()).to_quartz_nbt(),
                );
                // Add id field (matches the block entity type)
                data_compound.insert("id", NbtValue::String(self.id.clone()).to_quartz_nbt());
            }

            // Add all NBT data
            for (key, value) in self.nbt.iter() {
                data_compound.insert(key, value.to_quartz_nbt());
            }
            nbt.insert("Data", quartz_nbt::NbtTag::Compound(data_compound));
        }

        nbt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_entity_creation() {
        let block_entity = BlockEntity::new("minecraft:chest".to_string(), (1, 2, 3));
        assert_eq!(block_entity.id, "minecraft:chest");
        assert_eq!(block_entity.position, (1, 2, 3));
    }

    #[test]
    fn test_block_entity_with_nbt_data() {
        let block_entity = BlockEntity::new("minecraft:chest".to_string(), (1, 2, 3))
            .with_nbt_data(
                "CustomName".to_string(),
                NbtValue::String("Test".to_string()),
            );
        assert_eq!(
            block_entity.nbt.get("CustomName"),
            Some(&NbtValue::String("Test".to_string()))
        );
    }

    // Litematica / vanilla TileEntities use lowercase `id` and relative `x`/`y`/`z`
    // ints rather than the `Id`/`Pos` keys Nucleation's own writer emits. from_nbt
    // must read both spellings, else block entities parse as id="unknown" at (0,0,0)
    // (which collapsed every block entity in a region onto the region origin).
    #[test]
    fn from_nbt_reads_litematica_lowercase_id_and_xyz() {
        let mut compound = NbtCompound::new();
        compound.insert(
            "id",
            quartz_nbt::NbtTag::String("minecraft:dispenser".to_string()),
        );
        compound.insert("x", quartz_nbt::NbtTag::Int(3));
        compound.insert("y", quartz_nbt::NbtTag::Int(4));
        compound.insert("z", quartz_nbt::NbtTag::Int(5));
        let be = BlockEntity::from_nbt(&compound);
        assert_eq!(be.id, "minecraft:dispenser");
        assert_eq!(be.position, (3, 4, 5));
    }

    // The `Id`/`Pos` spelling (Sponge v3 / Nucleation's writer) must still win.
    #[test]
    fn from_nbt_still_reads_capitalized_id_and_pos() {
        let mut compound = NbtCompound::new();
        compound.insert(
            "Id",
            quartz_nbt::NbtTag::String("minecraft:chest".to_string()),
        );
        compound.insert("Pos", quartz_nbt::NbtTag::IntArray(vec![7, 8, 9]));
        let be = BlockEntity::from_nbt(&compound);
        assert_eq!(be.id, "minecraft:chest");
        assert_eq!(be.position, (7, 8, 9));
    }
}
