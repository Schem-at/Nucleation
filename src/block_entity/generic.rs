use crate::item::ItemStack;
use crate::utils::{NbtMap, NbtValue};
use quartz_nbt::NbtCompound;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockEntity {
    pub nbt: NbtMap,
    pub id: String,
    pub position: (i32, i32, i32),
}

impl BlockEntity {
    pub fn new(id: String, position: (i32, i32, i32)) -> Self {
        BlockEntity {
            nbt: NbtMap::new(),
            id,
            position,
        }
    }

    pub fn with_nbt_data(mut self, key: String, value: NbtValue) -> Self {
        self.nbt.insert(key, value);
        self
    }

    pub fn to_hashmap(&self) -> HashMap<String, NbtValue> {
        let mut map = HashMap::new();
        map.insert("Id".to_string(), NbtValue::String(self.id.clone()));
        map.insert(
            "Pos".to_string(),
            NbtValue::IntArray(vec![self.position.0, self.position.1, self.position.2]),
        );
        for (key, value) in &self.nbt {
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
            .unwrap_or_else(|| vec![]);
        items.push(item.to_nbt());
        self.nbt.insert("Items".to_string(), NbtValue::List(items));
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
        let id = nbt_map
            .get("Id")
            .and_then(|v| v.as_string())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let position = nbt_map
            .get("Pos")
            .and_then(|v| v.as_int_array())
            .map(|v| (v[0], v[1], v[2]))
            .unwrap_or_else(|| (0, 0, 0));
        BlockEntity {
            nbt: nbt_map,
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
        for (key, value) in &self.nbt {
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
    pub fn to_nbt_v3(&self) -> NbtCompound {
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

            if is_container {
                // Add empty components compound (required in 1.20.5+)
                data_compound.insert(
                    "components",
                    NbtValue::Compound(NbtMap::new()).to_quartz_nbt(),
                );
                // Add id field (matches the block entity type)
                data_compound.insert("id", NbtValue::String(self.id.clone()).to_quartz_nbt());
            }

            // Add all NBT data
            for (key, value) in &self.nbt {
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
    use crate::item::ItemStack;

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
}
