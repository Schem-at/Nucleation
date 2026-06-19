//! The `MapType` / `ListType` equivalent for the Rust engine.
//!
//! DataConverter abstracts NBT and JSON behind `types/MapType.java` and
//! `types/ListType.java` so converters are backend-agnostic. We only ever need
//! the NBT backend for schematics, so instead of a trait-object indirection we
//! provide extension traits with the same *lenient* accessor semantics directly
//! over [`NbtMap`]/[`NbtValue`]: a getter for the wrong type returns `None`
//! (Java returns the default), numeric getters cast across number types
//! (MapType.java:47-93), and setters mirror `set*`/`remove`.

use crate::nbt::{NbtMap, NbtValue};

/// Lenient typed access over an [`NbtValue`], mirroring the scalar reads on
/// `MapType`/`ListType`.
pub trait ValueExt {
    fn as_str(&self) -> Option<&str>;
    fn as_compound_ref(&self) -> Option<&NbtMap>;
    fn as_compound_mut(&mut self) -> Option<&mut NbtMap>;
    fn as_list_ref(&self) -> Option<&[NbtValue]>;
    fn as_list_mut(&mut self) -> Option<&mut Vec<NbtValue>>;
    /// Any numeric tag widened to f64 (Byte/Short/Int/Long/Float/Double).
    fn as_number_f64(&self) -> Option<f64>;
    /// Any numeric tag narrowed to i64.
    fn as_number_i64(&self) -> Option<i64>;
}

impl ValueExt for NbtValue {
    fn as_str(&self) -> Option<&str> {
        match self {
            NbtValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
    fn as_compound_ref(&self) -> Option<&NbtMap> {
        match self {
            NbtValue::Compound(m) => Some(m),
            _ => None,
        }
    }
    fn as_compound_mut(&mut self) -> Option<&mut NbtMap> {
        match self {
            NbtValue::Compound(m) => Some(m),
            _ => None,
        }
    }
    fn as_list_ref(&self) -> Option<&[NbtValue]> {
        match self {
            NbtValue::List(l) => Some(l.as_slice()),
            _ => None,
        }
    }
    fn as_list_mut(&mut self) -> Option<&mut Vec<NbtValue>> {
        match self {
            NbtValue::List(l) => Some(l),
            _ => None,
        }
    }
    fn as_number_f64(&self) -> Option<f64> {
        match self {
            NbtValue::Byte(v) => Some(*v as f64),
            NbtValue::Short(v) => Some(*v as f64),
            NbtValue::Int(v) => Some(*v as f64),
            NbtValue::Long(v) => Some(*v as f64),
            NbtValue::Float(v) => Some(*v as f64),
            NbtValue::Double(v) => Some(*v),
            _ => None,
        }
    }
    fn as_number_i64(&self) -> Option<i64> {
        match self {
            NbtValue::Byte(v) => Some(*v as i64),
            NbtValue::Short(v) => Some(*v as i64),
            NbtValue::Int(v) => Some(*v as i64),
            NbtValue::Long(v) => Some(*v),
            NbtValue::Float(v) => Some(*v as i64),
            NbtValue::Double(v) => Some(*v as i64),
            _ => None,
        }
    }
}

/// Lenient typed access over an [`NbtMap`], mirroring `MapType`.
pub trait MapExt {
    fn has_key(&self, key: &str) -> bool;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    /// A snapshot of the keys, safe to iterate while mutating the map.
    fn keys(&self) -> Vec<String>;

    fn get_string(&self, key: &str) -> Option<&str>;
    fn get_map(&self, key: &str) -> Option<&NbtMap>;
    fn get_map_mut(&mut self, key: &str) -> Option<&mut NbtMap>;
    fn get_list(&self, key: &str) -> Option<&[NbtValue]>;
    fn get_list_mut(&mut self, key: &str) -> Option<&mut Vec<NbtValue>>;
    fn get_i32(&self, key: &str) -> Option<i32>;
    fn get_i64(&self, key: &str) -> Option<i64>;
    fn get_f64(&self, key: &str) -> Option<f64>;
    /// Byte tag read as a boolean (non-zero == true), like vanilla.
    fn get_bool(&self, key: &str) -> Option<bool>;

    fn set_string(&mut self, key: &str, v: impl Into<String>);
    fn set_byte(&mut self, key: &str, v: i8);
    fn set_short(&mut self, key: &str, v: i16);
    fn set_i32(&mut self, key: &str, v: i32);
    fn set_i64(&mut self, key: &str, v: i64);
    fn set_f32(&mut self, key: &str, v: f32);
    fn set_f64(&mut self, key: &str, v: f64);
    fn set_bool(&mut self, key: &str, v: bool);
    fn set_map(&mut self, key: &str, v: NbtMap);
    fn set_list(&mut self, key: &str, v: Vec<NbtValue>);
    fn set_generic(&mut self, key: &str, v: NbtValue);

    /// Remove and return the value at `key` (MapType.remove).
    fn take(&mut self, key: &str) -> Option<NbtValue>;

    /// Rename a key in place, preserving the value (used by many field-move
    /// converters). No-op if `from` is absent. If `to` already exists it is
    /// overwritten (matches `RenameHelper.renameSingle`).
    fn rename_key(&mut self, from: &str, to: &str);
}

impl MapExt for NbtMap {
    fn has_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
    fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
    fn len(&self) -> usize {
        self.iter().count()
    }
    fn keys(&self) -> Vec<String> {
        self.iter().map(|(k, _)| k.clone()).collect()
    }
    fn get_string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(ValueExt::as_str)
    }
    fn get_map(&self, key: &str) -> Option<&NbtMap> {
        self.get(key).and_then(ValueExt::as_compound_ref)
    }
    fn get_map_mut(&mut self, key: &str) -> Option<&mut NbtMap> {
        self.get_mut(key).and_then(ValueExt::as_compound_mut)
    }
    fn get_list(&self, key: &str) -> Option<&[NbtValue]> {
        self.get(key).and_then(ValueExt::as_list_ref)
    }
    fn get_list_mut(&mut self, key: &str) -> Option<&mut Vec<NbtValue>> {
        self.get_mut(key).and_then(ValueExt::as_list_mut)
    }
    fn get_i32(&self, key: &str) -> Option<i32> {
        self.get(key).and_then(ValueExt::as_number_i64).map(|v| v as i32)
    }
    fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(ValueExt::as_number_i64)
    }
    fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(ValueExt::as_number_f64)
    }
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(ValueExt::as_number_i64).map(|v| v != 0)
    }

    fn set_string(&mut self, key: &str, v: impl Into<String>) {
        self.insert(key.to_string(), NbtValue::String(v.into()));
    }
    fn set_byte(&mut self, key: &str, v: i8) {
        self.insert(key.to_string(), NbtValue::Byte(v));
    }
    fn set_short(&mut self, key: &str, v: i16) {
        self.insert(key.to_string(), NbtValue::Short(v));
    }
    fn set_i32(&mut self, key: &str, v: i32) {
        self.insert(key.to_string(), NbtValue::Int(v));
    }
    fn set_i64(&mut self, key: &str, v: i64) {
        self.insert(key.to_string(), NbtValue::Long(v));
    }
    fn set_f32(&mut self, key: &str, v: f32) {
        self.insert(key.to_string(), NbtValue::Float(v));
    }
    fn set_f64(&mut self, key: &str, v: f64) {
        self.insert(key.to_string(), NbtValue::Double(v));
    }
    fn set_bool(&mut self, key: &str, v: bool) {
        self.insert(key.to_string(), NbtValue::Byte(if v { 1 } else { 0 }));
    }
    fn set_map(&mut self, key: &str, v: NbtMap) {
        self.insert(key.to_string(), NbtValue::Compound(v));
    }
    fn set_list(&mut self, key: &str, v: Vec<NbtValue>) {
        self.insert(key.to_string(), NbtValue::List(v));
    }
    fn set_generic(&mut self, key: &str, v: NbtValue) {
        self.insert(key.to_string(), v);
    }
    fn take(&mut self, key: &str) -> Option<NbtValue> {
        self.remove(key)
    }
    fn rename_key(&mut self, from: &str, to: &str) {
        if from == to {
            return;
        }
        if let Some(v) = self.remove(from) {
            self.insert(to.to_string(), v);
        }
    }
}
