use quartz_nbt::{NbtCompound, NbtTag};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockState {
    pub name: SmolStr,
    pub properties: Vec<(SmolStr, SmolStr)>,
}

impl fmt::Display for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.properties.is_empty() {
            write!(f, "[")?;
            for (i, (key, value)) in self.properties.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}={}", key, value)?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

impl Hash for BlockState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        for (k, v) in &self.properties {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl BlockState {
    pub fn new(name: impl Into<SmolStr>) -> Self {
        BlockState {
            name: name.into(),
            properties: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn with_property(mut self, key: impl Into<SmolStr>, value: impl Into<SmolStr>) -> Self {
        self.set_property(key, value);
        self
    }

    pub fn with_properties(mut self, properties: Vec<(SmolStr, SmolStr)>) -> Self {
        self.properties = properties;
        self
    }

    pub fn set_property(&mut self, key: impl Into<SmolStr>, value: impl Into<SmolStr>) {
        let key = key.into();
        let value = value.into();
        for (k, v) in &mut self.properties {
            if *k == key {
                *v = value;
                return;
            }
        }
        self.properties.push((key, value));
    }

    pub fn remove_property(&mut self, key: &str) {
        self.properties.retain(|(k, _)| k != key);
    }

    pub fn get_property(&self, key: &str) -> Option<&SmolStr> {
        for (k, v) in &self.properties {
            if k == key {
                return Some(v);
            }
        }
        None
    }
    pub fn to_nbt(&self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.insert("Name", self.name.to_string());

        if !self.properties.is_empty() {
            let mut properties = NbtCompound::new();
            for (key, value) in &self.properties {
                properties.insert(key.to_string(), value.to_string());
            }
            compound.insert("Properties", properties);
        }

        NbtTag::Compound(compound)
    }

    pub fn from_nbt(compound: &NbtCompound) -> Result<Self, String> {
        let name: SmolStr = compound
            .get::<_, &String>("Name")
            .map_err(|e| format!("Failed to get Name: {}", e))?
            .into();

        let mut properties = Vec::new();
        if let Ok(props) = compound.get::<_, &NbtCompound>("Properties") {
            for (key, value) in props.inner() {
                if let NbtTag::String(value_str) = value {
                    properties.push((key.into(), value_str.into()));
                }
            }
        }

        Ok(BlockState { name, properties })
    }
}

#[cfg(test)]
mod tests {
    use super::BlockState;

    #[test]
    fn test_block_state_creation() {
        let block = BlockState::new("minecraft:stone").with_property("variant", "granite");

        assert_eq!(block.name, "minecraft:stone");
        assert_eq!(
            block.get_property("variant").map(|s| s.as_str()),
            Some("granite")
        );
    }
}
