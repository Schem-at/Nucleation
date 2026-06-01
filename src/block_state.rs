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

    /// Parse the `Display` form: `name` or `name[k=v,k=v]`. Inverse of `to_string`.
    pub fn from_block_string(s: &str) -> Result<Self, String> {
        let s = s.trim();
        let Some(open) = s.find('[') else {
            if s.is_empty() {
                return Err("empty block string".into());
            }
            return Ok(BlockState::new(s));
        };
        if !s.ends_with(']') {
            return Err(format!("unterminated properties in {s:?}"));
        }
        let name = &s[..open];
        let body = &s[open + 1..s.len() - 1];
        let mut properties: Vec<(SmolStr, SmolStr)> = Vec::new();
        for pair in body.split(',') {
            if pair.is_empty() {
                continue;
            }
            let mut it = pair.splitn(2, '=');
            let key = it.next().unwrap();
            let value = it
                .next()
                .ok_or_else(|| format!("property {pair:?} missing '='"))?;
            properties.push((key.into(), value.into()));
        }
        Ok(BlockState {
            name: name.into(),
            properties,
        })
    }

    pub fn with_property(mut self, key: impl Into<SmolStr>, value: impl Into<SmolStr>) -> Self {
        self.set_property(key, value);
        self
    }

    pub fn with_properties(mut self, mut properties: Vec<(SmolStr, SmolStr)>) -> Self {
        properties.sort_by(|a, b| a.0.cmp(&b.0));
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

        let mut properties: Vec<(SmolStr, SmolStr)> = Vec::new();
        if let Ok(props) = compound.get::<_, &NbtCompound>("Properties") {
            for (key, value) in props.inner() {
                if let NbtTag::String(value_str) = value {
                    properties.push((key.into(), value_str.into()));
                }
            }
        }
        properties.sort_by(|a, b| a.0.cmp(&b.0));

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

    #[test]
    fn block_string_round_trips() {
        let plain = BlockState::new("minecraft:glass");
        assert_eq!(
            BlockState::from_block_string(&plain.to_string()).unwrap(),
            plain
        );

        let props = BlockState::new("minecraft:redstone_wire")
            .with_property("east", "side")
            .with_property("power", "0");
        let s = props.to_string();
        assert_eq!(BlockState::from_block_string(&s).unwrap(), props);
    }

    #[test]
    fn block_string_rejects_malformed() {
        assert!(BlockState::from_block_string("minecraft:foo[bad").is_err());
        assert!(BlockState::from_block_string("minecraft:foo[a]").is_err());
    }
}
