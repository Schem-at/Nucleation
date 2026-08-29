//! The animated GLB of a build animation: named group nodes with textured
//! meshes, TRS tracks, an anchor child, and `extras.nucleation`.
#![cfg(feature = "meshing")]
use nucleation::animation::{presets, AnimationEffect, BuildAnimation, Easing, Power, Property};
use nucleation::meshing::{MeshConfig, ResourcePackSource};

/// The vanilla pack the docs generators use (`render_work/pack.zip`, or
/// `NUCLEATION_PACK`); `apps/shared-pack` lacks the beacon and gold models.
fn pack() -> Option<ResourcePackSource> {
    let path = std::env::var("NUCLEATION_PACK")
        .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/render_work/pack.zip").into());
    match std::fs::read(&path) {
        Ok(bytes) => Some(ResourcePackSource::from_bytes(&bytes).unwrap()),
        Err(_) => {
            eprintln!("skipping: no resource pack at {path} (set NUCLEATION_PACK)");
            None
        }
    }
}

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
    animation.add_anchor("beacon", 0.0, 1.5, 0.0).unwrap();
    let camera =
        AnimationEffect::new(2_400.0).tween(Property::RotY, -4.0, 4.0, Easing::InOut(Power::Sine));
    animation.animate_camera(camera.clip().clone(), 0.0);
    animation
}

fn glb_json(glb: &[u8]) -> serde_json::Value {
    assert_eq!(&glb[0..4], b"glTF");
    assert_eq!(
        u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize,
        glb.len()
    );
    let json_len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    serde_json::from_slice(&glb[20..20 + json_len]).unwrap()
}

#[test]
fn beacon_exports_groups_tracks_anchor_and_extras() {
    let Some(pack) = pack() else { return };
    let animation = beacon();
    let glb = animation
        .to_animated_glb(&pack, &MeshConfig::default(), 30.0)
        .unwrap();
    let json = glb_json(&glb);
    let nodes = json["nodes"].as_array().unwrap();
    assert_eq!(nodes[0]["name"], "build:beacon");
    let root = &nodes[0]["extras"]["nucleation"];
    assert_eq!(root["groups"], 10);
    assert!((root["durationMs"].as_f64().unwrap() - 2400.0).abs() < 1e-3);
    assert_eq!(
        root["camera"]["yaw"].as_array().unwrap().len(),
        root["camera"]["times"].as_array().unwrap().len()
    );
    let group_names: Vec<&str> = nodes
        .iter()
        .filter_map(|n| n["name"].as_str())
        .filter(|n| n.starts_with("group:"))
        .collect();
    assert_eq!(group_names.len(), 10);

    let anchor = nodes
        .iter()
        .find(|n| n["name"] == "anchor:beacon")
        .expect("anchor node");
    assert_eq!(anchor["translation"], serde_json::json!([0.0, 1.5, 0.0]));
    assert_eq!(anchor["extras"]["nucleation"]["group"], 9);
    let beacon_group = nodes.iter().find(|n| n["name"] == "group:9").unwrap();
    assert!(beacon_group["mesh"].is_number());
    assert_eq!(beacon_group["extras"]["nucleation"]["blocks"], 1);

    // Group meshes use the engine's block space — a block is centred on its
    // integer coordinate — so the gold block at (-1, 0, -1) spans -1.5..-0.5.
    let first = nodes.iter().find(|n| n["name"] == "group:0").unwrap();
    let mesh = first["mesh"].as_u64().unwrap() as usize;
    let position = json["meshes"][mesh]["primitives"][0]["attributes"]["POSITION"]
        .as_u64()
        .unwrap() as usize;
    let min = json["accessors"][position]["min"].as_array().unwrap();
    let min: Vec<f64> = min.iter().map(|v| v.as_f64().unwrap()).collect();
    assert!(
        (min[0] + 1.5).abs() < 0.01 && (min[1] + 0.5).abs() < 0.01 && (min[2] + 1.5).abs() < 0.01,
        "group:0 position min {min:?}"
    );

    let animation_json = &json["animations"][0];
    assert_eq!(animation_json["name"], "beacon");
    let channels = animation_json["channels"].as_array().unwrap();
    assert!(
        channels.len() >= 20,
        "every group gets at least translation + scale: {}",
        channels.len()
    );
    assert_eq!(json["materials"].as_array().unwrap().len(), 10);
}

#[test]
fn constant_runs_are_deduplicated() {
    let Some(pack) = pack() else { return };
    let animation = beacon();
    let glb = animation
        .to_animated_glb(&pack, &MeshConfig::default(), 30.0)
        .unwrap();
    let json = glb_json(&glb);
    // A 2400 ms timeline at 30 fps is 73 samples; the first gold block settles
    // at 480 ms, so its deduplicated tracks hold far fewer keys.
    let sampler = &json["animations"][0]["samplers"][0];
    let input = sampler["input"].as_u64().unwrap() as usize;
    let count = json["accessors"][input]["count"].as_u64().unwrap();
    assert!(count < 40, "expected deduplicated keys, got {count}");
}
