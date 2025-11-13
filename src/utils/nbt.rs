use quartz_nbt::{self, NbtCompound, NbtTag};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<NbtValue>),
    Compound(NbtMap),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NbtMap(HashMap<String, NbtValue>);

impl Default for NbtMap {
    fn default() -> Self {
        Self::new()
    }
}

impl NbtMap {
    pub fn new() -> Self {
        NbtMap(HashMap::new())
    }

    pub fn insert(&mut self, key: String, value: NbtValue) -> Option<NbtValue> {
        self.0.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&NbtValue> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut NbtValue> {
        self.0.get_mut(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<NbtValue> {
        self.0.remove(key)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, NbtValue> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<String, NbtValue> {
        self.0.iter_mut()
    }

    pub fn from_quartz_nbt(compound: &NbtCompound) -> Self {
        let mut map = NbtMap::new();
        for (key, value) in compound.inner().iter() {
            let nbt_value = NbtValue::from_quartz_nbt(value);
            map.insert(key.clone(), nbt_value);
        }
        map
    }

    pub fn to_quartz_nbt(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        for (key, value) in self.iter() {
            compound.insert(key, value.to_quartz_nbt());
        }
        compound
    }
}

impl IntoIterator for NbtMap {
    type Item = (String, NbtValue);
    type IntoIter = std::collections::hash_map::IntoIter<String, NbtValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a NbtMap {
    type Item = (&'a String, &'a NbtValue);
    type IntoIter = std::collections::hash_map::Iter<'a, String, NbtValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut NbtMap {
    type Item = (&'a String, &'a mut NbtValue);
    type IntoIter = std::collections::hash_map::IterMut<'a, String, NbtValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

// Conversion functions
impl NbtValue {
    pub fn from_quartz_nbt(tag: &NbtTag) -> Self {
        match tag {
            NbtTag::Byte(v) => NbtValue::Byte(*v),
            NbtTag::Short(v) => NbtValue::Short(*v),
            NbtTag::Int(v) => NbtValue::Int(*v),
            NbtTag::Long(v) => NbtValue::Long(*v),
            NbtTag::Float(v) => NbtValue::Float(*v),
            NbtTag::Double(v) => NbtValue::Double(*v),
            NbtTag::ByteArray(v) => NbtValue::ByteArray(v.clone()),
            NbtTag::String(v) => NbtValue::String(v.clone()),
            NbtTag::List(v) => NbtValue::List(v.iter().map(NbtValue::from_quartz_nbt).collect()),
            NbtTag::Compound(v) => NbtValue::Compound(NbtMap::from_quartz_nbt(v)),
            NbtTag::IntArray(v) => NbtValue::IntArray(v.clone()),
            NbtTag::LongArray(v) => NbtValue::LongArray(v.clone()),
        }
    }

    pub fn to_quartz_nbt(&self) -> NbtTag {
        match self {
            NbtValue::Byte(v) => NbtTag::Byte(*v),
            NbtValue::Short(v) => NbtTag::Short(*v),
            NbtValue::Int(v) => NbtTag::Int(*v),
            NbtValue::Long(v) => NbtTag::Long(*v),
            NbtValue::Float(v) => NbtTag::Float(*v),
            NbtValue::Double(v) => NbtTag::Double(*v),
            NbtValue::ByteArray(v) => NbtTag::ByteArray(v.clone()),
            NbtValue::String(v) => NbtTag::String(v.clone()),
            NbtValue::List(v) => NbtTag::List(quartz_nbt::NbtList::from(
                v.iter().map(|x| x.to_quartz_nbt()).collect::<Vec<_>>(),
            )),
            NbtValue::Compound(v) => NbtTag::Compound(v.to_quartz_nbt()),
            NbtValue::IntArray(v) => NbtTag::IntArray(v.clone()),
            NbtValue::LongArray(v) => NbtTag::LongArray(v.clone()),
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let NbtValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            NbtValue::Byte(v) => Some(*v as i32),
            NbtValue::Short(v) => Some(*v as i32),
            NbtValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            NbtValue::Float(v) => Some(*v as f64),
            NbtValue::Double(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_compound(&self) -> Option<&NbtMap> {
        if let NbtValue::Compound(map) = self {
            Some(map)
        } else {
            None
        }
    }

    pub fn as_int_array(&self) -> Option<&Vec<i32>> {
        if let NbtValue::IntArray(arr) = self {
            Some(arr)
        } else {
            None
        }
    }

    /// Converts item NBT from legacy format (Count: Byte) to modern format (count: Int)
    /// Used when exporting schematics to ensure compatibility with modern Minecraft
    pub fn to_modern_item_format(&self) -> NbtValue {
        match self {
            NbtValue::Compound(map) => {
                let mut new_map = NbtMap::new();
                for (key, value) in map.iter() {
                    if key == "Count" {
                        // Convert Count: Byte to count: Int
                        if let NbtValue::Byte(b) = value {
                            new_map.insert("count".to_string(), NbtValue::Int(*b as i32));
                        } else {
                            new_map.insert(key.clone(), value.to_modern_item_format());
                        }
                    } else {
                        new_map.insert(key.clone(), value.to_modern_item_format());
                    }
                }
                NbtValue::Compound(new_map)
            }
            NbtValue::List(list) => {
                NbtValue::List(list.iter().map(|v| v.to_modern_item_format()).collect())
            }
            _ => self.clone(),
        }
    }

    /// Converts item NBT from modern format (count: Int) to legacy format (Count: Byte)
    /// Used when preparing data for MCHPRS which expects the legacy format
    pub fn to_legacy_item_format(&self) -> NbtValue {
        match self {
            NbtValue::Compound(map) => {
                let mut new_map = NbtMap::new();
                for (key, value) in map.iter() {
                    if key == "count" {
                        // Convert count: Int to Count: Byte
                        if let NbtValue::Int(i) = value {
                            new_map.insert("Count".to_string(), NbtValue::Byte(*i as i8));
                        } else {
                            new_map.insert(key.clone(), value.to_legacy_item_format());
                        }
                    } else {
                        new_map.insert(key.clone(), value.to_legacy_item_format());
                    }
                }
                NbtValue::Compound(new_map)
            }
            NbtValue::List(list) => {
                NbtValue::List(list.iter().map(|v| v.to_legacy_item_format()).collect())
            }
            _ => self.clone(),
        }
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use super::*;
    use js_sys::{Array, Object};
    use wasm_bindgen::JsValue;

    impl NbtMap {
        pub fn to_js_value(&self) -> JsValue {
            let obj = Object::new();
            for (key, value) in self.iter() {
                js_sys::Reflect::set(&obj, &key.into(), &value.to_js_value()).unwrap();
            }
            obj.into()
        }
    }

    impl NbtValue {
        pub fn to_js_value(&self) -> JsValue {
            match self {
                NbtValue::Byte(v) => JsValue::from(*v),
                NbtValue::Short(v) => JsValue::from(*v),
                NbtValue::Int(v) => JsValue::from(*v),
                NbtValue::Long(v) => JsValue::from(*v as f64), // JavaScript doesn't have 64-bit integers
                NbtValue::Float(v) => JsValue::from(*v),
                NbtValue::Double(v) => JsValue::from(*v),
                NbtValue::ByteArray(v) => {
                    let arr = Array::new();
                    for &byte in v {
                        arr.push(&JsValue::from(byte));
                    }
                    arr.into()
                }
                NbtValue::String(v) => JsValue::from_str(v),
                NbtValue::List(v) => {
                    let arr = Array::new();
                    for item in v {
                        arr.push(&item.to_js_value());
                    }
                    arr.into()
                }
                NbtValue::Compound(v) => v.to_js_value(),
                NbtValue::IntArray(v) => {
                    let arr = Array::new();
                    for &int in v {
                        arr.push(&JsValue::from(int));
                    }
                    arr.into()
                }
                NbtValue::LongArray(v) => {
                    let arr = Array::new();
                    for &long in v {
                        arr.push(&JsValue::from(long as f64)); // JavaScript doesn't have 64-bit integers
                    }
                    arr.into()
                }
            }
        }
    }
}
