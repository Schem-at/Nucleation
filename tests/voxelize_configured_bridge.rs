#![cfg(all(feature = "bridge", feature = "voxelize"))]
use diplomat_runtime::{rust_interop::RustWriteVec, DiplomatWrite};
use nucleation::bridge::{building::ffi::Palette, voxelize::ffi::Voxelizer};
fn written(fill: impl FnOnce(&mut DiplomatWrite)) -> String {
    let mut out = RustWriteVec::with_capacity(256);
    fill(unsafe { out.borrow_mut() });
    String::from_utf8(out.borrow().as_bytes().to_vec()).unwrap()
}
#[test]
fn configured_glb_bridge_loads_estimates_and_exports() {
    use base64::Engine;
    let bytes = include_bytes!("fixtures/voxelize-transformed-scan.glb");
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let model = Voxelizer::load_glb_base64(encoded.as_bytes()).unwrap();
    let options = br#"{"target_size":384,"axis":"y","hollow":true}"#;
    let plan = written(|w| model.plan_json(options, w).unwrap());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&plan).unwrap()["dimensions"],
        serde_json::json!([96, 384, 24])
    );
    let out = model
        .to_schematic(options, &Palette::solid(), b"scan")
        .unwrap();
    assert_eq!(out.block_count(), 94760);
    assert!(model
        .to_schematic(br#"{"target_size":8192}"#, &Palette::solid(), b"large")
        .is_err());
    assert!(written(Voxelizer::last_error_detail).contains("working limit"));
    written(|w| model.plan_json(options, w).unwrap());
    assert!(written(Voxelizer::last_error_detail).is_empty());
}
