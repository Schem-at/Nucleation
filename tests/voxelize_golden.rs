#![cfg(feature = "voxelize")]
//! Byte identity of the voxelizer output against the v0.10.16 baseline.
//!
//! The fixture is generated once from unchanged code by the ignored test at
//! the bottom of this file, then committed. Every later change must keep
//! these three hashes.

use nucleation::building::{BlockPalette, BuildingTool, SolidBrush};
use nucleation::voxelize::{test_meshes::sphere_5k, voxelize_textured, MeshModel, MeshShape};
use nucleation::{BlockState, UniversalSchematic};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

fn sphere(size: f32) -> MeshShape {
    let mut model = MeshModel::from_obj_str(&sphere_5k()).expect("sphere parses");
    model.fit(size);
    MeshShape::new(model)
}

fn textured_cube(size: f32) -> MeshShape {
    let bytes = std::fs::read("tests/samples/BoxTextured.glb").expect("committed sample");
    let mut model = MeshModel::from_glb_bytes(&bytes).expect("BoxTextured loads");
    model.fit(size);
    MeshShape::new(model)
}

/// `x,y,z,id` for every non air block, sorted, joined by newlines, hashed.
fn digest(schematic: &UniversalSchematic) -> (usize, String) {
    let mut lines: Vec<String> = schematic
        .iter_blocks()
        .filter(|(_, block)| block.name != "minecraft:air")
        .map(|(pos, block)| format!("{},{},{},{}", pos.x, pos.y, pos.z, block.name))
        .collect();
    lines.sort();
    let mut hasher = Sha256::new();
    hasher.update(lines.join("\n").as_bytes());
    (lines.len(), format!("{:x}", hasher.finalize()))
}

fn fill_solid(shape: &MeshShape, name: &str) -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new(name.to_string());
    let brush = SolidBrush::new(BlockState::new("minecraft:stone"));
    BuildingTool::new(&mut schematic).fill(shape, &brush);
    schematic
}

/// The three pinned cases, as `case name -> (block count, sha256)`.
fn cases() -> BTreeMap<String, (usize, String)> {
    let mut out = BTreeMap::new();
    out.insert(
        "sphere_solid_32".to_string(),
        digest(&fill_solid(&sphere(32.0), "sphere_solid_32")),
    );
    out.insert(
        "sphere_shell_32".to_string(),
        digest(&fill_solid(
            &sphere(32.0).with_surface_shell(1.0),
            "sphere_shell_32",
        )),
    );
    let palette = BlockPalette::new_wool();
    out.insert(
        "textured_cube_32".to_string(),
        digest(&voxelize_textured(
            &textured_cube(32.0),
            &palette,
            "textured_cube_32",
        )),
    );
    out
}

const FIXTURE: &str = "tests/fixtures/voxelize_golden.json";

#[test]
fn voxelize_output_matches_the_v0_10_16_golden() {
    let raw = std::fs::read_to_string(FIXTURE).expect(
        "tests/fixtures/voxelize_golden.json is missing; regenerate it with \
         `cargo test --release --features voxelize --test voxelize_golden \
         write_golden_fixture -- --ignored`",
    );
    let expected: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&raw).expect("fixture parses");
    for (name, (count, hash)) in cases() {
        let want = expected
            .get(&name)
            .unwrap_or_else(|| panic!("no golden for {name}"));
        assert_eq!(
            want["blocks"].as_u64().unwrap() as usize,
            count,
            "{name}: block count moved"
        );
        assert_eq!(
            want["sha256"].as_str().unwrap(),
            hash,
            "{name}: block set moved"
        );
    }
}

/// One off generator. Run with `-- --ignored` on unchanged code, then commit
/// the fixture. Never run it again to "fix" a failure: a failure means the
/// output moved, which is the thing this file exists to catch.
#[test]
#[ignore]
fn write_golden_fixture() {
    let mut out = serde_json::Map::new();
    for (name, (count, hash)) in cases() {
        out.insert(name, serde_json::json!({ "blocks": count, "sha256": hash }));
    }
    let json = serde_json::to_string_pretty(&serde_json::Value::Object(out)).unwrap();
    std::fs::write(FIXTURE, format!("{json}\n")).expect("fixture written");
}
