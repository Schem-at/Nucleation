//! Streaming NBT encoder, byte-for-byte equal to `quartz_nbt::io::write_nbt`
//! on a canonicalized (key-sorted) tree.
//!
//! Exporters build the small parts of a file as a quartz tree and leave the
//! block-entity lists empty; [`write_root`] asks a hook to fill each list as
//! it reaches it, and the hook encodes every entity straight from its
//! [`NbtMap`] instead of building a per-entity tree first.

use crate::block_entity::BlockEntity;
use crate::nbt::{NbtMap, NbtValue};
use quartz_nbt::{NbtCompound, NbtTag};
use std::io::{self, Write};

const END: u8 = 0;
const BYTE: u8 = 1;
const SHORT: u8 = 2;
const INT: u8 = 3;
const LONG: u8 = 4;
const FLOAT: u8 = 5;
const DOUBLE: u8 = 6;
const BYTE_ARRAY: u8 = 7;
const STRING: u8 = 8;
const LIST: u8 = 9;
const COMPOUND: u8 = 10;
const INT_ARRAY: u8 = 11;
const LONG_ARRAY: u8 = 12;

/// Writes an unnamed root compound. `hook(path, writer)` is offered every
/// list reached through compound keys (`path` is those keys); returning
/// `true` means it wrote the list payload itself.
pub(crate) fn write_root<W, F>(w: &mut W, root: &NbtCompound, mut hook: F) -> io::Result<()>
where
    W: Write,
    F: FnMut(&[&str], &mut W) -> io::Result<bool>,
{
    w.write_all(&[COMPOUND])?;
    write_str(w, "")?;
    let mut path = Vec::new();
    write_compound(w, root, &mut path, &mut hook)
}

fn write_compound<'a, W, F>(
    w: &mut W,
    compound: &'a NbtCompound,
    path: &mut Vec<&'a str>,
    hook: &mut F,
) -> io::Result<()>
where
    W: Write,
    F: FnMut(&[&str], &mut W) -> io::Result<bool>,
{
    for (key, tag) in compound.inner() {
        w.write_all(&[tag_id(tag)])?;
        write_str(w, key)?;
        path.push(key);
        match tag {
            NbtTag::Compound(inner) => write_compound(w, inner, path, hook)?,
            NbtTag::List(_) if hook(path, w)? => {}
            _ => write_tag(w, tag)?,
        }
        path.pop();
    }
    w.write_all(&[END])
}

/// Writes a list of `len` compounds, each encoded by `write_item`.
pub(crate) fn write_compound_list<W: Write, T>(
    w: &mut W,
    len: usize,
    items: impl IntoIterator<Item = T>,
    mut write_item: impl FnMut(&mut W, T) -> io::Result<()>,
) -> io::Result<()> {
    write_list_header(w, COMPOUND, len)?;
    for item in items {
        write_item(w, item)?;
    }
    Ok(())
}

/// Sponge v2 block entity: the entity's NBT at the root, `Id` unless the
/// NBT has one, `Pos` relative to the schematic origin.
pub(crate) fn write_block_entity_v2<W: Write>(
    w: &mut W,
    be: &BlockEntity,
    pos: [i32; 3],
) -> io::Result<()> {
    write_merged(
        w,
        &be.nbt,
        &[("Id", Field::Str(&be.id))],
        &[("Pos", Field::Pos(pos))],
    )
}

/// Sponge v3 block entity: `Id`, relative `Pos`, and the NBT under `Data`.
pub(crate) fn write_block_entity_v3<W: Write>(
    w: &mut W,
    be: &BlockEntity,
    pos: [i32; 3],
    data_version: Option<i32>,
) -> io::Result<()> {
    if be.nbt.iter().next().is_some() {
        write_key(w, COMPOUND, "Data")?;
        let defaults: &[(&str, Field)] = if be.v3_injects_components(data_version) {
            &[
                ("components", Field::EmptyCompound),
                ("id", Field::Str(&be.id)),
            ]
        } else {
            &[]
        };
        write_merged(w, &be.nbt, defaults, &[])?;
    }
    write_key(w, STRING, "Id")?;
    write_str(w, &be.id)?;
    write_key(w, INT_ARRAY, "Pos")?;
    write_int_array(w, &pos)?;
    w.write_all(&[END])
}

/// Litematica tile entity: the entity's NBT, `Id` unless the NBT has one,
/// and the region-relative position as both `x`/`y`/`z` and `Pos`.
pub(crate) fn write_tile_entity_litematic<W: Write>(
    w: &mut W,
    be: &BlockEntity,
    pos: [i32; 3],
) -> io::Result<()> {
    write_merged(
        w,
        &be.nbt,
        &[("Id", Field::Str(&be.id))],
        &[
            ("Pos", Field::Pos(pos)),
            ("x", Field::Int(pos[0])),
            ("y", Field::Int(pos[1])),
            ("z", Field::Int(pos[2])),
        ],
    )
}

#[derive(Clone, Copy)]
enum Field<'a> {
    Str(&'a str),
    Int(i32),
    Pos([i32; 3]),
    EmptyCompound,
    Value(&'a NbtValue),
}

/// Writes the compound body of `nbt` plus `defaults` (only for keys `nbt`
/// lacks) and `overrides` (always win), keys sorted.
fn write_merged<W: Write>(
    w: &mut W,
    nbt: &NbtMap,
    defaults: &[(&str, Field)],
    overrides: &[(&str, Field)],
) -> io::Result<()> {
    let mut fields: Vec<(&str, Field)> =
        Vec::with_capacity(nbt.inner().len() + defaults.len() + overrides.len());
    fields.extend(
        nbt.iter()
            .filter(|(key, _)| !overrides.iter().any(|(k, _)| k == key))
            .map(|(key, value)| (key.as_str(), Field::Value(value))),
    );
    fields.extend(
        defaults
            .iter()
            .filter(|(key, _)| nbt.get(key).is_none())
            .copied(),
    );
    fields.extend(overrides.iter().copied());
    fields.sort_unstable_by(|a, b| a.0.cmp(b.0));

    for (key, field) in fields {
        match field {
            Field::Str(s) => {
                write_key(w, STRING, key)?;
                write_str(w, s)?;
            }
            Field::Int(v) => {
                write_key(w, INT, key)?;
                w.write_all(&v.to_be_bytes())?;
            }
            Field::Pos(p) => {
                write_key(w, INT_ARRAY, key)?;
                write_int_array(w, &p)?;
            }
            Field::EmptyCompound => {
                write_key(w, COMPOUND, key)?;
                w.write_all(&[END])?;
            }
            Field::Value(v) => {
                write_key(w, value_id(v), key)?;
                write_value(w, v)?;
            }
        }
    }
    w.write_all(&[END])
}

fn write_value<W: Write>(w: &mut W, value: &NbtValue) -> io::Result<()> {
    match value {
        NbtValue::Byte(v) => w.write_all(&v.to_be_bytes()),
        NbtValue::Short(v) => w.write_all(&v.to_be_bytes()),
        NbtValue::Int(v) => w.write_all(&v.to_be_bytes()),
        NbtValue::Long(v) => w.write_all(&v.to_be_bytes()),
        NbtValue::Float(v) => w.write_all(&v.to_be_bytes()),
        NbtValue::Double(v) => w.write_all(&v.to_be_bytes()),
        NbtValue::ByteArray(v) => write_byte_array(w, v),
        NbtValue::String(v) => write_str(w, v),
        NbtValue::List(items) => {
            let list_type = items.first().map_or(END, value_id);
            write_list_header(w, list_type, items.len())?;
            for item in items {
                check_list_item(list_type, value_id(item))?;
                write_value(w, item)?;
            }
            Ok(())
        }
        NbtValue::Compound(map) => {
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_unstable_by(|a, b| a.0.cmp(b.0));
            for (key, value) in entries {
                write_key(w, value_id(value), key)?;
                write_value(w, value)?;
            }
            w.write_all(&[END])
        }
        NbtValue::IntArray(v) => write_int_array(w, v),
        NbtValue::LongArray(v) => write_long_array(w, v),
    }
}

fn write_tag<W: Write>(w: &mut W, tag: &NbtTag) -> io::Result<()> {
    match tag {
        NbtTag::Byte(v) => w.write_all(&v.to_be_bytes()),
        NbtTag::Short(v) => w.write_all(&v.to_be_bytes()),
        NbtTag::Int(v) => w.write_all(&v.to_be_bytes()),
        NbtTag::Long(v) => w.write_all(&v.to_be_bytes()),
        NbtTag::Float(v) => w.write_all(&v.to_be_bytes()),
        NbtTag::Double(v) => w.write_all(&v.to_be_bytes()),
        NbtTag::ByteArray(v) => write_byte_array(w, v),
        NbtTag::String(v) => write_str(w, v),
        NbtTag::List(list) => {
            let items: &[NbtTag] = list.as_ref();
            let list_type = items.first().map_or(END, tag_id);
            write_list_header(w, list_type, items.len())?;
            for item in items {
                check_list_item(list_type, tag_id(item))?;
                write_tag(w, item)?;
            }
            Ok(())
        }
        NbtTag::Compound(compound) => {
            for (key, tag) in compound.inner() {
                write_key(w, tag_id(tag), key)?;
                write_tag(w, tag)?;
            }
            w.write_all(&[END])
        }
        NbtTag::IntArray(v) => write_int_array(w, v),
        NbtTag::LongArray(v) => write_long_array(w, v),
    }
}

fn check_list_item(list_type: u8, item_type: u8) -> io::Result<()> {
    if item_type == list_type {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("non-homogenous NBT list: type {list_type} holds type {item_type}"),
        ))
    }
}

fn write_list_header<W: Write>(w: &mut W, list_type: u8, len: usize) -> io::Result<()> {
    let list_type = if len == 0 { END } else { list_type };
    w.write_all(&[list_type])?;
    w.write_all(&(len as i32).to_be_bytes())
}

fn write_key<W: Write>(w: &mut W, id: u8, key: &str) -> io::Result<()> {
    w.write_all(&[id])?;
    write_str(w, key)
}

fn write_str<W: Write>(w: &mut W, s: &str) -> io::Result<()> {
    let bytes = cesu8::to_java_cesu8(s);
    w.write_all(&(bytes.len() as u16).to_be_bytes())?;
    w.write_all(&bytes)
}

fn write_byte_array<W: Write>(w: &mut W, v: &[i8]) -> io::Result<()> {
    w.write_all(&(v.len() as i32).to_be_bytes())?;
    // SAFETY: i8 and u8 have identical size, alignment, and representation
    w.write_all(unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len()) })
}

fn write_int_array<W: Write>(w: &mut W, v: &[i32]) -> io::Result<()> {
    w.write_all(&(v.len() as i32).to_be_bytes())?;
    v.iter().try_for_each(|x| w.write_all(&x.to_be_bytes()))
}

fn write_long_array<W: Write>(w: &mut W, v: &[i64]) -> io::Result<()> {
    w.write_all(&(v.len() as i32).to_be_bytes())?;
    v.iter().try_for_each(|x| w.write_all(&x.to_be_bytes()))
}

fn tag_id(tag: &NbtTag) -> u8 {
    match tag {
        NbtTag::Byte(_) => BYTE,
        NbtTag::Short(_) => SHORT,
        NbtTag::Int(_) => INT,
        NbtTag::Long(_) => LONG,
        NbtTag::Float(_) => FLOAT,
        NbtTag::Double(_) => DOUBLE,
        NbtTag::ByteArray(_) => BYTE_ARRAY,
        NbtTag::String(_) => STRING,
        NbtTag::List(_) => LIST,
        NbtTag::Compound(_) => COMPOUND,
        NbtTag::IntArray(_) => INT_ARRAY,
        NbtTag::LongArray(_) => LONG_ARRAY,
    }
}

fn value_id(value: &NbtValue) -> u8 {
    match value {
        NbtValue::Byte(_) => BYTE,
        NbtValue::Short(_) => SHORT,
        NbtValue::Int(_) => INT,
        NbtValue::Long(_) => LONG,
        NbtValue::Float(_) => FLOAT,
        NbtValue::Double(_) => DOUBLE,
        NbtValue::ByteArray(_) => BYTE_ARRAY,
        NbtValue::String(_) => STRING,
        NbtValue::List(_) => LIST,
        NbtValue::Compound(_) => COMPOUND,
        NbtValue::IntArray(_) => INT_ARRAY,
        NbtValue::LongArray(_) => LONG_ARRAY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quartz_bytes(mut compound: NbtCompound) -> Vec<u8> {
        crate::nbt::canonicalize_compound(&mut compound);
        let mut out = Vec::new();
        quartz_nbt::io::write_nbt(
            &mut out,
            None,
            &compound,
            quartz_nbt::io::Flavor::Uncompressed,
        )
        .unwrap();
        out
    }

    fn streamed(write: impl Fn(&mut Vec<u8>)) -> Vec<u8> {
        let mut out = vec![COMPOUND];
        write_str(&mut out, "").unwrap();
        write(&mut out);
        out
    }

    #[test]
    fn block_entities_match_the_quartz_tree() {
        let parse = |snbt: &str| BlockEntity::from_nbt(&quartz_nbt::snbt::parse(snbt).unwrap());
        // No id in the NBT, so the writers' defaults apply.
        let mut chest = parse(
            r#"{Items:[{Slot:0b,id:"minecraft:stone",count:2,components:{"minecraft:custom_name":"é𝄞"}}],Empty:[],L:[L;5l],B:[B;1b],F:1.5f,D:2.5d,S:3s,N:{b:[[1,2],[3]],a:{}}}"#,
        );
        chest.id = "minecraft:chest".into();
        // NBT keys colliding with the written ones.
        let own = parse(r#"{Id:"own",id:"minecraft:barrel",components:{a:1},x:9,Pos:[I;7,7,7]}"#);
        for be in [&chest, &own] {
            check(be);
        }
    }

    fn check(be: &BlockEntity) {
        let pos = [1, -2, 3];
        for dv in [None, Some(3700), Some(4671)] {
            let mut tree = be.to_nbt_v3(dv);
            tree.insert("Pos", NbtTag::IntArray(pos.to_vec()));
            let bytes = streamed(|w| write_block_entity_v3(w, be, pos, dv).unwrap());
            assert_eq!(bytes, quartz_bytes(tree), "v3 {dv:?}");
        }

        let mut tree = be.to_nbt();
        tree.insert("Pos", NbtTag::IntArray(pos.to_vec()));
        let bytes = streamed(|w| write_block_entity_v2(w, be, pos).unwrap());
        assert_eq!(bytes, quartz_bytes(tree.clone()), "v2");

        for (key, v) in [("x", pos[0]), ("y", pos[1]), ("z", pos[2])] {
            tree.insert(key, NbtTag::Int(v));
        }
        let bytes = streamed(|w| write_tile_entity_litematic(w, be, pos).unwrap());
        assert_eq!(bytes, quartz_bytes(tree), "litematic");
    }
}
