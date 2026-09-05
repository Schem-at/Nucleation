#![cfg(feature = "bridge")]
//! Round trips for the bulk block export and edit methods added for
//! schemat.io, which used to tally and replace blocks by shipping the whole
//! schematic through get_all_blocks_json.

use diplomat_runtime::rust_interop::RustWriteVec;
use diplomat_runtime::DiplomatWrite;
use nucleation::bridge::schematic::ffi::Schematic;

/// Run a `DiplomatWrite`-returning method and collect what it wrote.
fn written(fill: impl FnOnce(&mut DiplomatWrite)) -> String {
    let mut buffer = RustWriteVec::with_capacity(256);
    // Safety: this is the only `DiplomatWrite` in scope.
    fill(unsafe { buffer.borrow_mut() });
    String::from_utf8(buffer.borrow().as_bytes().to_vec()).expect("UTF-8 out")
}

fn sample() -> Box<Schematic> {
    let mut s = Schematic::create(b"bulk");
    s.set_block(0, 0, 0, b"minecraft:stone").unwrap();
    s.set_block(1, 0, 0, b"minecraft:stone").unwrap();
    s.set_block(2, 0, 0, b"minecraft:dirt").unwrap();
    s
}

/// A schematic with a stateful block and a `cave_air` cell, for asserting
/// that the bulk queries tally by name (ignoring block state) and treat
/// every air variant as air, not just `minecraft:air`.
fn sample_with_state_and_cave_air() -> Box<Schematic> {
    let mut s = Schematic::create(b"bulk-state");
    s.set_block(0, 0, 0, b"minecraft:stone").unwrap();
    s.set_block_from_string(1, 0, 0, b"minecraft:oak_stairs[facing=north]")
        .unwrap();
    s.set_block(2, 0, 0, b"minecraft:cave_air").unwrap();
    s
}

#[test]
fn count_blocks_json_tallies_non_air_blocks() {
    let s = sample();
    let out = written(|w| s.count_blocks_json(w));
    let counts: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert_eq!(counts["minecraft:stone"], 2);
    assert_eq!(counts["minecraft:dirt"], 1);
    assert!(counts.get("minecraft:air").is_none(), "air is excluded");
}

#[test]
fn count_blocks_json_keys_by_name_and_excludes_every_air_variant() {
    let s = sample_with_state_and_cave_air();
    let out = written(|w| s.count_blocks_json(w));
    let counts: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert_eq!(counts["minecraft:stone"], 1);
    // Keyed by id only: the `[facing=north]` state does not split the tally
    // into a separate key.
    assert_eq!(counts["minecraft:oak_stairs"], 1);
    assert!(counts.get("minecraft:air").is_none());
    assert!(
        counts.get("minecraft:cave_air").is_none(),
        "cave_air is air too"
    );
    assert_eq!(counts.as_object().expect("object").len(), 2);
}

#[test]
fn replace_blocks_json_rewrites_and_counts() {
    let mut s = sample();
    let changed = s
        .replace_blocks_json(br#"{"minecraft:stone":"minecraft:glass"}"#)
        .expect("valid map");
    assert_eq!(changed, 2);
    let out = written(|w| s.count_blocks_json(w));
    let counts: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert_eq!(counts["minecraft:glass"], 2);
    assert!(counts.get("minecraft:stone").is_none());
    assert!(s.replace_blocks_json(b"not json").is_err());
}

#[test]
fn packed_export_round_trips() {
    use base64::Engine as _;
    let s = sample();
    let out = written(|w| s.non_air_blocks_packed_b64(w));
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&out)
        .expect("valid base64");

    let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    assert_eq!(count, 3);
    let mut seen = Vec::new();
    for i in 0..count {
        let at = 4 + i * 14;
        let x = i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let y = i32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap());
        let z = i32::from_le_bytes(bytes[at + 8..at + 12].try_into().unwrap());
        let p = u16::from_le_bytes(bytes[at + 12..at + 14].try_into().unwrap());
        seen.push((x, y, z, p));
    }
    let at = 4 + count * 14;
    let len = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
    let palette: Vec<String> =
        serde_json::from_slice(&bytes[at + 4..at + 4 + len]).expect("palette JSON");
    assert_eq!(bytes.len(), at + 4 + len, "no trailing bytes");

    assert_eq!(seen.len(), 3);
    for (x, y, z, p) in seen {
        let name = &palette[p as usize];
        let expected = if x == 2 {
            "minecraft:dirt"
        } else {
            "minecraft:stone"
        };
        assert_eq!(name, expected, "block at {x},{y},{z}");
    }
}
