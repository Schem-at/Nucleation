//! Integration tests for the public `autostack` API (detection + resize).
//! Run with: `cargo test --features autostack --test autostack_test`
#![cfg(feature = "autostack")]

use nucleation::autostack::{
    detect_structures, detect_structures_json, resize, resize_1d, resize_2d, resize_json,
};
use nucleation::{BlockState, UniversalSchematic};

const AIR: [&str; 3] = ["minecraft:air", "minecraft:cave_air", "minecraft:void_air"];

fn non_air(s: &UniversalSchematic) -> usize {
    s.iter_blocks()
        .filter(|(_, b)| !AIR.contains(&b.get_name()))
        .count()
}

/// A stone/glass bar repeating along X with period 2.
fn bar(n: i32) -> UniversalSchematic {
    let mut s = UniversalSchematic::new("bar".into());
    let stone = BlockState::new("minecraft:stone");
    let glass = BlockState::new("minecraft:glass");
    for i in 0..n {
        s.set_block(i * 2, 0, 0, &stone);
        s.set_block(i * 2 + 1, 0, 0, &glass);
    }
    s
}

/// A 2D tiling in the Y-Z plane, period 2 in both.
fn screen(ny: i32, nz: i32) -> UniversalSchematic {
    let mut s = UniversalSchematic::new("screen".into());
    let stone = BlockState::new("minecraft:stone");
    let glass = BlockState::new("minecraft:glass");
    for y in 0..2 * ny {
        for z in 0..2 * nz {
            let b = if y % 2 == 0 && z % 2 == 0 {
                &stone
            } else {
                &glass
            };
            s.set_block(0, y, z, b);
        }
    }
    s
}

#[test]
fn detect_1d_and_resize() {
    let s = bar(6);
    let structs = detect_structures(&s);
    assert!(!structs.is_empty());
    assert_eq!(structs[0].mode, "1d");
    assert_eq!(structs[0].vectors[0], [2, 0, 0]);
    // resize via the Structure dispatcher
    let bigger = resize(&s, &structs[0], &[10]).unwrap();
    assert_eq!(non_air(&bigger), 20);
    // re-detects with the same period
    assert_eq!(detect_structures(&bigger)[0].vectors[0], [2, 0, 0]);
}

#[test]
fn detect_2d_and_resize() {
    let s = screen(6, 6);
    let structs = detect_structures(&s);
    assert_eq!(structs[0].mode, "2d");
    let r = resize(&s, &structs[0], &[4, 8]).unwrap();
    assert_eq!(non_air(&r), 4 * 8 * 4); // 4x8 cells, 4 blocks each
}

#[test]
fn diagonal_resize() {
    // resize directly along a diagonal vector
    let mut s = UniversalSchematic::new("diag".into());
    let stone = BlockState::new("minecraft:stone");
    let glass = BlockState::new("minecraft:glass");
    for i in 0..6 {
        s.set_block(i * 2, i, 0, &stone);
        s.set_block(i * 2 + 1, i, 0, &glass);
    }
    let r = resize_1d(&s, [2, 1, 0], 10).unwrap();
    assert_eq!(non_air(&r), 20);
}

#[test]
fn json_api_roundtrip() {
    let s = bar(6);
    let json = detect_structures_json(&s);
    assert!(json.contains("\"1d\""));
    let arr: serde_json::Value = serde_json::from_str(&json).unwrap();
    let one = serde_json::to_string(&arr[0]).unwrap();
    let r = resize_json(&s, &one, &[8]).unwrap();
    assert_eq!(non_air(&r), 16);
}

#[test]
fn explicit_2d_resize_fn() {
    let s = screen(5, 5);
    let r = resize_2d(&s, [0, 2, 0], [0, 0, 2], 3, 7).unwrap();
    assert_eq!(non_air(&r), 3 * 7 * 4);
}

#[test]
fn bad_units_errors() {
    let s = bar(4);
    let st = &detect_structures(&s)[0];
    assert!(resize(&s, st, &[0]).is_err());
}
