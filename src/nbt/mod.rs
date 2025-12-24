use quartz_nbt::{self, NbtCompound, NbtTag};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use std::io::{Error, ErrorKind, Read, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endian {
    Big,
    Little,
}

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

    pub fn inner(&self) -> &HashMap<String, NbtValue> {
        &self.0
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
    pub fn to_modern_item_format(&self) -> NbtValue {
        match self {
            NbtValue::Compound(map) => {
                let mut new_map = NbtMap::new();
                for (key, value) in map.iter() {
                    if key == "Count" {
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
    pub fn to_legacy_item_format(&self) -> NbtValue {
        match self {
            NbtValue::Compound(map) => {
                let mut new_map = NbtMap::new();
                for (key, value) in map.iter() {
                    if key == "count" {
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
                NbtValue::Long(v) => JsValue::from(*v as f64),
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
                        arr.push(&JsValue::from(long as f64));
                    }
                    arr.into()
                }
            }
        }
    }
}

// --- IO Logic ---

pub mod io {
    use super::*;
    use std::io::{Error, ErrorKind, Read, Result as IoResult, Write};

    fn read_u8<R: Read>(r: &mut R) -> IoResult<u8> {
        let mut buf = [0; 1];
        r.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_i16<R: Read>(r: &mut R, endian: Endian) -> IoResult<i16> {
        let mut buf = [0; 2];
        r.read_exact(&mut buf)?;
        match endian {
            Endian::Big => Ok(i16::from_be_bytes(buf)),
            Endian::Little => Ok(i16::from_le_bytes(buf)),
        }
    }

    fn read_i32<R: Read>(r: &mut R, endian: Endian) -> IoResult<i32> {
        let mut buf = [0; 4];
        r.read_exact(&mut buf)?;
        match endian {
            Endian::Big => Ok(i32::from_be_bytes(buf)),
            Endian::Little => Ok(i32::from_le_bytes(buf)),
        }
    }

    fn read_i64<R: Read>(r: &mut R, endian: Endian) -> IoResult<i64> {
        let mut buf = [0; 8];
        r.read_exact(&mut buf)?;
        match endian {
            Endian::Big => Ok(i64::from_be_bytes(buf)),
            Endian::Little => Ok(i64::from_le_bytes(buf)),
        }
    }

    fn read_f32<R: Read>(r: &mut R, endian: Endian) -> IoResult<f32> {
        let mut buf = [0; 4];
        r.read_exact(&mut buf)?;
        match endian {
            Endian::Big => Ok(f32::from_be_bytes(buf)),
            Endian::Little => Ok(f32::from_le_bytes(buf)),
        }
    }

    fn read_f64<R: Read>(r: &mut R, endian: Endian) -> IoResult<f64> {
        let mut buf = [0; 8];
        r.read_exact(&mut buf)?;
        match endian {
            Endian::Big => Ok(f64::from_be_bytes(buf)),
            Endian::Little => Ok(f64::from_le_bytes(buf)),
        }
    }

    fn read_string<R: Read>(r: &mut R, endian: Endian) -> IoResult<String> {
        let len = read_i16(r, endian)? as usize;
        let mut buf = vec![0; len];
        r.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    fn read_payload<R: Read>(r: &mut R, type_id: u8, endian: Endian) -> IoResult<NbtValue> {
        match type_id {
            1 => Ok(NbtValue::Byte(read_u8(r)? as i8)),
            2 => Ok(NbtValue::Short(read_i16(r, endian)?)),
            3 => Ok(NbtValue::Int(read_i32(r, endian)?)),
            4 => Ok(NbtValue::Long(read_i64(r, endian)?)),
            5 => Ok(NbtValue::Float(read_f32(r, endian)?)),
            6 => Ok(NbtValue::Double(read_f64(r, endian)?)),
            7 => {
                // ByteArray
                let len = read_i32(r, endian)? as usize;
                let mut buf = vec![0u8; len];
                r.read_exact(&mut buf)?;
                Ok(NbtValue::ByteArray(buf.iter().map(|&b| b as i8).collect()))
            }
            8 => Ok(NbtValue::String(read_string(r, endian)?)),
            9 => {
                // List
                let tag_id = read_u8(r)?;
                let len = read_i32(r, endian)? as usize;
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    list.push(read_payload(r, tag_id, endian)?);
                }
                Ok(NbtValue::List(list))
            }
            10 => {
                // Compound
                let mut map = NbtMap::new();
                loop {
                    let tag_id = read_u8(r)?;
                    if tag_id == 0 {
                        break;
                    }
                    let name = read_string(r, endian)?;
                    let tag = read_payload(r, tag_id, endian)?;
                    map.insert(name, tag);
                }
                Ok(NbtValue::Compound(map))
            }
            11 => {
                // IntArray
                let len = read_i32(r, endian)? as usize;
                let mut buf = Vec::with_capacity(len);
                for _ in 0..len {
                    buf.push(read_i32(r, endian)?);
                }
                Ok(NbtValue::IntArray(buf))
            }
            12 => {
                // LongArray
                let len = read_i32(r, endian)? as usize;
                let mut buf = Vec::with_capacity(len);
                for _ in 0..len {
                    buf.push(read_i64(r, endian)?);
                }
                Ok(NbtValue::LongArray(buf))
            }
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unknown tag id: {}", type_id),
            )),
        }
    }

    pub fn read_nbt<R: Read>(r: &mut R, endian: Endian) -> IoResult<NbtValue> {
        let tag_id = read_u8(r)?;
        if tag_id != 10 {
            // Must be compound
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Root tag must be compound",
            ));
        }
        let _name = read_string(r, endian)?; // Root name
        read_payload(r, 10, endian)
    }

    fn write_u8<W: Write>(w: &mut W, v: u8) -> IoResult<()> {
        w.write_all(&[v])
    }

    fn write_i16<W: Write>(w: &mut W, v: i16, endian: Endian) -> IoResult<()> {
        match endian {
            Endian::Big => w.write_all(&v.to_be_bytes()),
            Endian::Little => w.write_all(&v.to_le_bytes()),
        }
    }

    fn write_i32<W: Write>(w: &mut W, v: i32, endian: Endian) -> IoResult<()> {
        match endian {
            Endian::Big => w.write_all(&v.to_be_bytes()),
            Endian::Little => w.write_all(&v.to_le_bytes()),
        }
    }

    fn write_i64<W: Write>(w: &mut W, v: i64, endian: Endian) -> IoResult<()> {
        match endian {
            Endian::Big => w.write_all(&v.to_be_bytes()),
            Endian::Little => w.write_all(&v.to_le_bytes()),
        }
    }

    fn write_f32<W: Write>(w: &mut W, v: f32, endian: Endian) -> IoResult<()> {
        match endian {
            Endian::Big => w.write_all(&v.to_be_bytes()),
            Endian::Little => w.write_all(&v.to_le_bytes()),
        }
    }

    fn write_f64<W: Write>(w: &mut W, v: f64, endian: Endian) -> IoResult<()> {
        match endian {
            Endian::Big => w.write_all(&v.to_be_bytes()),
            Endian::Little => w.write_all(&v.to_le_bytes()),
        }
    }

    fn write_string<W: Write>(w: &mut W, v: &str, endian: Endian) -> IoResult<()> {
        write_i16(w, v.len() as i16, endian)?;
        w.write_all(v.as_bytes())
    }

    fn get_tag_id(tag: &NbtValue) -> u8 {
        match tag {
            NbtValue::Byte(_) => 1,
            NbtValue::Short(_) => 2,
            NbtValue::Int(_) => 3,
            NbtValue::Long(_) => 4,
            NbtValue::Float(_) => 5,
            NbtValue::Double(_) => 6,
            NbtValue::ByteArray(_) => 7,
            NbtValue::String(_) => 8,
            NbtValue::List(_) => 9,
            NbtValue::Compound(_) => 10,
            NbtValue::IntArray(_) => 11,
            NbtValue::LongArray(_) => 12,
        }
    }

    fn write_payload<W: Write>(w: &mut W, tag: &NbtValue, endian: Endian) -> IoResult<()> {
        match tag {
            NbtValue::Byte(v) => write_u8(w, *v as u8),
            NbtValue::Short(v) => write_i16(w, *v, endian),
            NbtValue::Int(v) => write_i32(w, *v, endian),
            NbtValue::Long(v) => write_i64(w, *v, endian),
            NbtValue::Float(v) => write_f32(w, *v, endian),
            NbtValue::Double(v) => write_f64(w, *v, endian),
            NbtValue::ByteArray(v) => {
                write_i32(w, v.len() as i32, endian)?;
                for &b in v {
                    write_u8(w, b as u8)?;
                }
                Ok(())
            }
            NbtValue::String(v) => write_string(w, v, endian),
            NbtValue::List(v) => {
                let type_id = if v.is_empty() { 1 } else { get_tag_id(&v[0]) };
                write_u8(w, type_id)?;
                write_i32(w, v.len() as i32, endian)?;
                for item in v.iter() {
                    write_payload(w, item, endian)?;
                }
                Ok(())
            }
            NbtValue::Compound(v) => {
                for (name, tag) in v.iter() {
                    write_u8(w, get_tag_id(tag))?;
                    write_string(w, name, endian)?;
                    write_payload(w, tag, endian)?;
                }
                write_u8(w, 0)?; // End tag
                Ok(())
            }
            NbtValue::IntArray(v) => {
                write_i32(w, v.len() as i32, endian)?;
                for &i in v {
                    write_i32(w, i, endian)?;
                }
                Ok(())
            }
            NbtValue::LongArray(v) => {
                write_i32(w, v.len() as i32, endian)?;
                for &l in v {
                    write_i64(w, l, endian)?;
                }
                Ok(())
            }
        }
    }

    pub fn write_nbt<W: Write>(
        w: &mut W,
        root: &NbtMap,
        root_name: &str,
        endian: Endian,
    ) -> IoResult<()> {
        write_u8(w, 10)?; // Root Compound
        write_string(w, root_name, endian)?;
        write_payload(w, &NbtValue::Compound(root.clone()), endian)
    }
}
