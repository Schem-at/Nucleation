//! Generates two .schem files to verify the entity "Id" casing issue in
//! the Sponge Schematic v3 format.
//!
//! The example exports a base schematic via nucleation (which currently
//! drops every entity on export because of a validator mismatch), then
//! injects two entity compounds directly into the NBT:
//!
//! `entity_id_buggy.schem`         — entity stored with lowercase top-level
//!                                   `"id"` (what our Entity::to_nbt emits).
//!                                   The v3 spec requires capital `"Id"` +
//!                                   `"Pos"` at the top and any extra
//!                                   fields nested under `"Data"`.
//!                                   Expected: WorldEdit / Litematica /
//!                                   any spec-compliant loader will DROP
//!                                   the entities on load.
//!
//! `entity_id_spec_correct.schem`  — same entities encoded per the v3 spec:
//!                                       { "Id": "minecraft:creeper",
//!                                         "Pos": [x, y, z],
//!                                         "Data": { "id": "...", ...rest } }
//!                                   Expected: both entities load correctly.
//!
//! Run with: `cargo run --example entity_id_schem_test`

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use nucleation::{schematic, BlockState, UniversalSchematic};
use quartz_nbt::io::{read_nbt, write_nbt, Flavor};
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use std::io::Read;

const ENTITIES: &[(&str, f64, f64, f64)] = &[
    ("minecraft:creeper", 0.5, 1.0, 0.5),
    ("minecraft:villager", 2.5, 1.0, 2.5),
];

fn build_schem() -> UniversalSchematic {
    let mut s = UniversalSchematic::new("entity_id_test".to_string());
    s.set_block(
        0,
        0,
        0,
        &BlockState::new("minecraft:diamond_block".to_string()),
    );
    // A few air blocks so the region is larger than 1x1x1 and the entities
    // land inside the region bbox.
    s.set_block(3, 3, 3, &BlockState::new("minecraft:stone".to_string()));
    s
}

fn read_schem_nbt(bytes: &[u8]) -> NbtCompound {
    let mut gz = GzDecoder::new(bytes);
    let mut raw = Vec::new();
    gz.read_to_end(&mut raw).expect("gunzip");
    let (root, _) = read_nbt(&mut std::io::Cursor::new(raw), Flavor::Uncompressed).expect("nbt");
    root
}

fn write_schem_nbt(root: &NbtCompound) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    write_nbt(&mut encoder, None, root, Flavor::Uncompressed).expect("write nbt");
    encoder.finish().expect("finish gz")
}

fn pos_list(x: f64, y: f64, z: f64) -> NbtList {
    let mut list = NbtList::new();
    list.push(NbtTag::Double(x));
    list.push(NbtTag::Double(y));
    list.push(NbtTag::Double(z));
    list
}

/// Buggy shape — lowercase top-level `id`, no `Data` wrapper.
fn buggy_entity(id: &str, x: f64, y: f64, z: f64) -> NbtCompound {
    let mut c = NbtCompound::new();
    c.insert("id", NbtTag::String(id.to_string()));
    c.insert("Pos", NbtTag::List(pos_list(x, y, z)));
    c
}

/// Spec-correct shape per Sponge v3 + the minimum vanilla MC entity NBT that
/// WorldEdit's SpongeSchematicV3Reader actually requires (Rotation, Motion,
/// Pos inside Data).
fn spec_entity(id: &str, x: f64, y: f64, z: f64) -> NbtCompound {
    let mut motion = NbtList::new();
    motion.push(NbtTag::Double(0.0));
    motion.push(NbtTag::Double(0.0));
    motion.push(NbtTag::Double(0.0));

    let mut rotation = NbtList::new();
    rotation.push(NbtTag::Float(0.0));
    rotation.push(NbtTag::Float(0.0));

    let mut data = NbtCompound::new();
    data.insert("id", NbtTag::String(id.to_string()));
    data.insert("Pos", NbtTag::List(pos_list(x, y, z)));
    data.insert("Motion", NbtTag::List(motion));
    data.insert("Rotation", NbtTag::List(rotation));

    let mut c = NbtCompound::new();
    c.insert("Id", NbtTag::String(id.to_string()));
    c.insert("Pos", NbtTag::List(pos_list(x, y, z)));
    c.insert("Data", NbtTag::Compound(data));
    c
}

fn inject_entities(root: &mut NbtCompound, entities: NbtList) {
    let schem: &mut NbtCompound = root.get_mut("Schematic").expect("Schematic");
    schem.insert("Entities", NbtTag::List(entities));
}

fn dump_entity_nbt(tag: &NbtTag, label: &str) {
    println!("   [{}] {:?}", label, tag);
}

fn main() {
    let schem = build_schem();
    let base_bytes = schematic::to_schematic(&schem).expect("export");

    // Build the two entity lists.
    let mut buggy_list = NbtList::new();
    let mut spec_list = NbtList::new();
    for &(id, x, y, z) in ENTITIES {
        buggy_list.push(NbtTag::Compound(buggy_entity(id, x, y, z)));
        spec_list.push(NbtTag::Compound(spec_entity(id, x, y, z)));
    }

    // 1) Buggy file.
    let mut buggy_root = read_schem_nbt(&base_bytes);
    inject_entities(&mut buggy_root, buggy_list);
    let buggy_bytes = write_schem_nbt(&buggy_root);
    std::fs::write("entity_id_buggy.schem", &buggy_bytes).expect("write buggy");
    println!(
        "wrote entity_id_buggy.schem ({} bytes — {} entities with lowercase top-level \"id\")",
        buggy_bytes.len(),
        ENTITIES.len()
    );
    let buggy_root = read_schem_nbt(&buggy_bytes);
    let schem_tag: &NbtCompound = buggy_root.get("Schematic").expect("Schematic");
    let ents: &NbtList = schem_tag.get("Entities").expect("Entities");
    if let Some(first) = ents.iter().next() {
        dump_entity_nbt(first, "buggy first entity");
    }

    // 2) Spec-correct file.
    let mut spec_root = read_schem_nbt(&base_bytes);
    inject_entities(&mut spec_root, spec_list);
    let spec_bytes = write_schem_nbt(&spec_root);
    std::fs::write("entity_id_spec_correct.schem", &spec_bytes).expect("write spec");
    println!(
        "wrote entity_id_spec_correct.schem ({} bytes — {} entities with capital \"Id\" + nested \"Data\")",
        spec_bytes.len(),
        ENTITIES.len()
    );
    let spec_root = read_schem_nbt(&spec_bytes);
    let schem_tag: &NbtCompound = spec_root.get("Schematic").expect("Schematic");
    let ents: &NbtList = schem_tag.get("Entities").expect("Entities");
    if let Some(first) = ents.iter().next() {
        dump_entity_nbt(first, "spec first entity");
    }

    println!();
    println!("Load both in WorldEdit (//schem load) or any Sponge v3 loader.");
    println!("Prediction:");
    println!("  entity_id_buggy.schem        -> block placed, entities dropped (spec rejects)");
    println!("  entity_id_spec_correct.schem -> block placed, creeper + villager present");
}
