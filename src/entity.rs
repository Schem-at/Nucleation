use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NbtValue {
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Byte(i8),
    Short(i16),
    Boolean(bool),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    ByteArray(Vec<i8>),
    List(Vec<NbtValue>),
    Compound(HashMap<String, NbtValue>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub position: (f64, f64, f64),
    pub nbt: HashMap<String, NbtValue>,
}

impl Entity {
    pub fn new(id: String, position: (f64, f64, f64)) -> Self {
        Entity {
            id,
            position,
            nbt: HashMap::new(),
        }
    }

    pub fn with_nbt_data(mut self, key: String, value: String) -> Self {
        self.nbt.insert(key, NbtValue::String(value));
        self
    }

    fn nbt_tag_to_value(tag: &NbtTag) -> NbtValue {
        match tag {
            NbtTag::String(s) => NbtValue::String(s.clone()),
            NbtTag::Int(i) => NbtValue::Int(*i),
            NbtTag::Long(l) => NbtValue::Long(*l),
            NbtTag::Float(f) => NbtValue::Float(*f),
            NbtTag::Double(d) => NbtValue::Double(*d),
            NbtTag::Byte(b) => NbtValue::Byte(*b),
            NbtTag::Short(s) => NbtValue::Short(*s),
            NbtTag::IntArray(arr) => NbtValue::IntArray(arr.clone()),
            NbtTag::LongArray(arr) => NbtValue::LongArray(arr.clone()),
            NbtTag::ByteArray(arr) => NbtValue::ByteArray(arr.clone()),
            NbtTag::List(list) => {
                let values: Vec<NbtValue> =
                    list.iter().map(|tag| Self::nbt_tag_to_value(tag)).collect();
                NbtValue::List(values)
            }
            NbtTag::Compound(compound) => {
                let mut map = HashMap::new();
                for (key, value) in compound.inner() {
                    map.insert(key.clone(), Self::nbt_tag_to_value(value));
                }
                NbtValue::Compound(map)
            }
        }
    }

    fn value_to_nbt_tag(value: &NbtValue) -> NbtTag {
        match value {
            NbtValue::String(s) => NbtTag::String(s.clone()),
            NbtValue::Int(i) => NbtTag::Int(*i),
            NbtValue::Long(l) => NbtTag::Long(*l),
            NbtValue::Float(f) => NbtTag::Float(*f),
            NbtValue::Double(d) => NbtTag::Double(*d),
            NbtValue::Byte(b) => NbtTag::Byte(*b),
            NbtValue::Short(s) => NbtTag::Short(*s),
            NbtValue::Boolean(b) => NbtTag::Byte(if *b { 1 } else { 0 }),
            NbtValue::IntArray(arr) => NbtTag::IntArray(arr.clone()),
            NbtValue::LongArray(arr) => NbtTag::LongArray(arr.clone()),
            NbtValue::ByteArray(arr) => NbtTag::ByteArray(arr.clone()),
            NbtValue::List(list) => {
                let tags: Vec<NbtTag> = list
                    .iter()
                    .map(|value| Self::value_to_nbt_tag(value))
                    .collect();
                NbtTag::List(NbtList::from(tags))
            }
            NbtValue::Compound(map) => {
                let mut compound = NbtCompound::new();
                for (key, value) in map {
                    compound.insert(key, Self::value_to_nbt_tag(value));
                }
                NbtTag::Compound(compound)
            }
        }
    }

    pub fn to_nbt(&self) -> NbtTag {
        let mut compound = NbtCompound::new();

        // Always store the full minecraft:id format
        let full_id = if self.id.starts_with("minecraft:") {
            self.id.clone()
        } else {
            format!("minecraft:{}", self.id)
        };
        compound.insert("id", NbtTag::String(full_id));

        // Add position
        let pos_list = NbtList::from(vec![
            NbtTag::Double(self.position.0),
            NbtTag::Double(self.position.1),
            NbtTag::Double(self.position.2),
        ]);
        compound.insert("Pos", NbtTag::List(pos_list));

        // Convert HashMap<String, NbtValue> to NbtCompound
        if !self.nbt.is_empty() {
            let mut nbt_compound = NbtCompound::new();
            for (key, value) in &self.nbt {
                nbt_compound.insert(key, Self::value_to_nbt_tag(value));
            }
            compound.insert("NBT", NbtTag::Compound(nbt_compound));
        }

        NbtTag::Compound(compound)
    }

    pub fn from_nbt(nbt: &NbtCompound) -> Result<Self, String> {
        // Handle both id cases, but preserve the minecraft: prefix
        let id = match nbt.get::<_, &str>("id") {
            Ok(id) => id.to_string(),
            Err(_) => match nbt.get::<_, &str>("Id") {
                Ok(id) => id.to_string(),
                Err(e) => return Err(format!("Failed to get Entity id: {}", e)),
            },
        };

        // Don't strip the minecraft: prefix anymore
        let id = if id.starts_with("minecraft:") {
            id
        } else {
            format!("minecraft:{}", id)
        };

        let position = nbt
            .get::<_, &NbtList>("Pos")
            .map_err(|e| format!("Failed to get Entity position: {}", e))?;
        let position = if position.len() == 3 {
            (
                position
                    .get::<f64>(0)
                    .map_err(|e| format!("Failed to get X position: {}", e))?,
                position
                    .get::<f64>(1)
                    .map_err(|e| format!("Failed to get Y position: {}", e))?,
                position
                    .get::<f64>(2)
                    .map_err(|e| format!("Failed to get Z position: {}", e))?,
            )
        } else {
            return Err("Invalid position data".to_string());
        };

        // Get NBT data if it exists and convert it to HashMap<String, NbtValue>
        let mut nbt_map = HashMap::new();
        if let Ok(entity_nbt) = nbt.get::<_, &NbtCompound>("NBT") {
            for (key, value) in entity_nbt.inner() {
                nbt_map.insert(key.clone(), Self::nbt_tag_to_value(value));
            }
        }

        Ok(Entity {
            id,
            position,
            nbt: nbt_map,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entity() {
        let entity = Entity::new("minecraft:creeper".to_string(), (1.0, 2.0, 3.0));
        assert_eq!(entity.id, "minecraft:creeper");
        assert_eq!(entity.position, (1.0, 2.0, 3.0));
        assert!(entity.nbt.is_empty());
    }

    #[test]
    fn test_with_nbt_data() {
        let entity = Entity::new("minecraft:creeper".to_string(), (1.0, 2.0, 3.0))
            .with_nbt_data("CustomName".to_string(), "Bob".to_string());

        assert_eq!(entity.nbt.len(), 1);
        assert_eq!(
            entity.nbt.get("CustomName"),
            Some(&NbtValue::String("Bob".to_string()))
        );
    }

    #[test]
    fn test_entity_serialization() {
        let mut entity = Entity::new("minecraft:creeper".to_string(), (1.0, 2.0, 3.0));
        entity
            .nbt
            .insert("Health".to_string(), NbtValue::Float(20.0));
        entity.nbt.insert(
            "CustomName".to_string(),
            NbtValue::String("Bob".to_string()),
        );

        let nbt = entity.to_nbt();

        if let NbtTag::Compound(compound) = nbt {
            assert_eq!(compound.get::<_, &str>("id").unwrap(), "minecraft:creeper");

            let pos = compound.get::<_, &NbtList>("Pos").unwrap();
            assert_eq!(pos.get::<f64>(0).unwrap(), 1.0);
            assert_eq!(pos.get::<f64>(1).unwrap(), 2.0);
            assert_eq!(pos.get::<f64>(2).unwrap(), 3.0);

            let nbt_data = compound.get::<_, &NbtCompound>("NBT").unwrap();
            assert_eq!(nbt_data.get::<_, f32>("Health").unwrap(), 20.0);
            assert_eq!(nbt_data.get::<_, &str>("CustomName").unwrap(), "Bob");
        } else {
            panic!("Expected Compound NBT tag");
        }
    }

    #[test]
    fn test_entity_deserialization() {
        let mut compound = NbtCompound::new();
        compound.insert("id", NbtTag::String("minecraft:creeper".to_string()));

        let pos_list = NbtList::from(vec![
            NbtTag::Double(1.0),
            NbtTag::Double(2.0),
            NbtTag::Double(3.0),
        ]);
        compound.insert("Pos", NbtTag::List(pos_list));

        let mut nbt_data = NbtCompound::new();
        nbt_data.insert("Health", NbtTag::Float(20.0));
        nbt_data.insert("CustomName", NbtTag::String("Bob".to_string()));
        compound.insert("NBT", NbtTag::Compound(nbt_data));

        let entity = Entity::from_nbt(&compound).unwrap();

        assert_eq!(entity.id, "minecraft:creeper");
        assert_eq!(entity.position, (1.0, 2.0, 3.0));
        assert_eq!(entity.nbt.get("Health"), Some(&NbtValue::Float(20.0)));
        assert_eq!(
            entity.nbt.get("CustomName"),
            Some(&NbtValue::String("Bob".to_string()))
        );
    }

    #[test]
    fn test_complex_nbt_values() {
        let mut entity = Entity::new("minecraft:item".to_string(), (0.0, 0.0, 0.0));

        // Test array types
        entity
            .nbt
            .insert("IntArray".to_string(), NbtValue::IntArray(vec![1, 2, 3]));
        entity
            .nbt
            .insert("LongArray".to_string(), NbtValue::LongArray(vec![1, 2, 3]));
        entity
            .nbt
            .insert("ByteArray".to_string(), NbtValue::ByteArray(vec![1, 2, 3]));

        // Test nested compound
        let mut nested_map = HashMap::new();
        nested_map.insert(
            "NestedString".to_string(),
            NbtValue::String("test".to_string()),
        );
        entity
            .nbt
            .insert("Compound".to_string(), NbtValue::Compound(nested_map));

        // Test list
        entity.nbt.insert(
            "List".to_string(),
            NbtValue::List(vec![
                NbtValue::String("a".to_string()),
                NbtValue::String("b".to_string()),
            ]),
        );

        let nbt = entity.to_nbt();
        if let NbtTag::Compound(compound) = nbt {
            let deserialized = Entity::from_nbt(&compound).unwrap();
            assert_eq!(entity, deserialized);
        } else {
            panic!("Expected Compound NBT tag");
        }
    }

    #[test]
    fn test_id_prefix_handling() {
        // Test with minecraft: prefix
        let entity1 = Entity::new("minecraft:creeper".to_string(), (0.0, 0.0, 0.0));
        let nbt1 = entity1.to_nbt();
        if let NbtTag::Compound(compound) = nbt1 {
            let deserialized1 = Entity::from_nbt(&compound).unwrap();
            assert_eq!(deserialized1.id, "minecraft:creeper");
        } else {
            panic!("Expected Compound NBT tag");
        }

        // Test without minecraft: prefix
        let entity2 = Entity::new("creeper".to_string(), (0.0, 0.0, 0.0));
        let nbt2 = entity2.to_nbt();
        if let NbtTag::Compound(compound) = nbt2 {
            let deserialized2 = Entity::from_nbt(&compound).unwrap();
            assert_eq!(deserialized2.id, "minecraft:creeper");
        } else {
            panic!("Expected Compound NBT tag");
        }
    }

    #[test]
    fn test_invalid_nbt() {
        // Test missing id
        let mut compound = NbtCompound::new();
        compound.insert(
            "Pos",
            NbtTag::List(NbtList::from(vec![
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
            ])),
        );
        assert!(Entity::from_nbt(&compound).is_err());

        // Test invalid position
        let mut compound = NbtCompound::new();
        compound.insert("id", NbtTag::String("minecraft:creeper".to_string()));
        compound.insert(
            "Pos",
            NbtTag::List(NbtList::from(vec![
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
            ])),
        );
        assert!(Entity::from_nbt(&compound).is_err());
    }
}
