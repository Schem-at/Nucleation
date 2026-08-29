//! The native engine is the source of truth for the WASM/JS and Python parity
//! suites (`tools/verify-build-animation.sh`). Run with
//! `NUCLEATION_WRITE_FIXTURES=1` to (re)generate the fixtures under
//! tests/fixtures/build-animation; without it the test asserts that the engine
//! still produces exactly the committed fixtures.
use nucleation::animation::{presets, AnimationEffect, BuildAnimation, Easing, Power, Property};
use serde_json::{json, Value};
use std::path::PathBuf;

const SAMPLE_TIMES_MS: [f32; 5] = [0.0, 450.0, 1000.0, 1500.0, 2400.0];

fn beacon() -> BuildAnimation {
    let mut animation = BuildAnimation::new("beacon");
    animation.set_step_ms(140.0);
    for x in -1..=1 {
        for z in -1..=1 {
            animation
                .set_block(x, 0, z, "minecraft:gold_block")
                .unwrap();
        }
    }
    animation
        .with_effect(presets::spin_in(680.0, 1.0))
        .set_block(0, 1, 0, "minecraft:beacon")
        .unwrap();
    let camera =
        AnimationEffect::new(2_400.0).tween(Property::RotY, -4.0, 4.0, Easing::InOut(Power::Sine));
    animation.animate_camera(camera.clip().clone(), 0.0);
    animation
}

fn crafting_nook() -> BuildAnimation {
    let mut animation = BuildAnimation::new("crafting_nook");
    animation.set_step_ms(520.0);
    animation.begin_group(None).unwrap();
    for x in 0..5 {
        for z in 0..5 {
            animation
                .set_block(x, 0, z, "minecraft:spruce_planks")
                .unwrap();
        }
    }
    animation.end_group().unwrap();
    animation.begin_group(None).unwrap();
    for y in 1..=3 {
        for x in 0..5 {
            let block = if x == 2 && y == 2 {
                "minecraft:light_blue_stained_glass"
            } else if x == 0 || x == 4 {
                "minecraft:stripped_spruce_log[axis=y]"
            } else {
                "minecraft:oak_planks"
            };
            animation.set_block(x, y, 0, block).unwrap();
        }
        for z in 1..5 {
            let block = if z == 2 && y == 2 {
                "minecraft:light_blue_stained_glass"
            } else if z == 4 {
                "minecraft:stripped_spruce_log[axis=y]"
            } else {
                "minecraft:oak_planks"
            };
            animation.set_block(0, y, z, block).unwrap();
        }
    }
    animation.end_group().unwrap();
    animation
        .with_effect(presets::spin_in(620.0, 1.0))
        .set_block(1, 1, 1, "minecraft:crafting_table")
        .unwrap();
    animation
        .set_block(3, 1, 1, "minecraft:chest[facing=south]")
        .unwrap();
    animation.begin_group(None).unwrap();
    animation
        .set_block(4, 2, 1, "minecraft:wall_torch[facing=south]")
        .unwrap();
    animation
        .set_block(1, 2, 4, "minecraft:wall_torch[facing=east]")
        .unwrap();
    animation.end_group().unwrap();
    let camera =
        AnimationEffect::new(3_000.0).tween(Property::RotY, -5.0, 6.0, Easing::InOut(Power::Sine));
    animation.animate_camera(camera.clip().clone(), 0.0);
    animation
}

fn fixture(name: &str, animation: &BuildAnimation) -> Value {
    let frames: Vec<Value> = SAMPLE_TIMES_MS
        .iter()
        .map(|t| serde_json::to_value(animation.frame_at(*t)).unwrap())
        .collect();
    json!({
        "name": name,
        "groupCount": animation.groups().len(),
        "durationMs": animation.duration_ms(),
        "sampleTimesMs": SAMPLE_TIMES_MS,
        "frames": frames,
    })
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/build-animation")
        .join(format!("{name}.json"))
}

fn check(name: &str, animation: BuildAnimation) {
    let value = fixture(name, &animation);
    let path = fixture_path(name);
    if std::env::var_os("NUCLEATION_WRITE_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
        return;
    }
    let committed: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "missing fixture {}; run with NUCLEATION_WRITE_FIXTURES=1",
                path.display()
            )
        }))
        .unwrap();
    // Both sides go through the same text round-trip, so f32 → JSON → f64 is compared
    // exactly the way the JS and Python suites see it.
    let fresh: Value = serde_json::from_str(&serde_json::to_string(&value).unwrap()).unwrap();
    if let Some(diff) = first_difference(&fresh, &committed, "$") {
        panic!("{name}: engine output drifted from the committed fixture at {diff}");
    }
}

fn first_difference(a: &Value, b: &Value, path: &str) -> Option<String> {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            let mut keys: Vec<&String> = x.keys().chain(y.keys()).collect();
            keys.sort();
            keys.dedup();
            keys.into_iter()
                .find_map(|key| match (x.get(key), y.get(key)) {
                    (Some(av), Some(bv)) => first_difference(av, bv, &format!("{path}.{key}")),
                    _ => Some(format!("{path}.{key} (missing on one side)")),
                })
        }
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                return Some(format!("{path} (length {} vs {})", x.len(), y.len()));
            }
            x.iter()
                .zip(y)
                .enumerate()
                .find_map(|(i, (av, bv))| first_difference(av, bv, &format!("{path}[{i}]")))
        }
        _ => (a != b).then(|| format!("{path}: {a} vs {b}")),
    }
}

#[test]
fn beacon_matches_fixture() {
    let animation = beacon();
    assert_eq!(animation.groups().len(), 10);
    check("beacon", animation);
}

#[test]
fn crafting_nook_matches_fixture() {
    let animation = crafting_nook();
    assert_eq!(animation.groups().len(), 5);
    check("crafting-nook", animation);
}

#[test]
fn sampling_is_pure() {
    let animation = beacon();
    let a = serde_json::to_string(&animation.frame_at(450.0)).unwrap();
    let _later = animation.frame_at(2_000.0);
    let b = serde_json::to_string(&animation.frame_at(450.0)).unwrap();
    assert_eq!(a, b, "sampling later times must not change earlier frames");
}
