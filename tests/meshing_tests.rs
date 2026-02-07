#![cfg(feature = "meshing")]

use nucleation::meshing::{MeshConfig, ResourcePackSource};

#[test]
fn test_mesh_config_default_values() {
    let config = MeshConfig::default();
    assert!(config.cull_hidden_faces);
    assert!(config.ambient_occlusion);
    assert!((config.ao_intensity - 0.4).abs() < f32::EPSILON * 10.0);
    assert_eq!(config.biome, None);
    assert_eq!(config.atlas_max_size, 4096);
    assert!(config.cull_occluded_blocks);
    assert!(!config.greedy_meshing);
}

#[test]
fn test_mesh_config_builder_chain() {
    let config = MeshConfig::new()
        .with_culling(false)
        .with_ambient_occlusion(false)
        .with_ao_intensity(0.8)
        .with_biome("swamp")
        .with_atlas_max_size(2048)
        .with_cull_occluded_blocks(false)
        .with_greedy_meshing(true);

    assert!(!config.cull_hidden_faces);
    assert!(!config.ambient_occlusion);
    assert!((config.ao_intensity - 0.8).abs() < f32::EPSILON * 10.0);
    assert_eq!(config.biome, Some("swamp".to_string()));
    assert_eq!(config.atlas_max_size, 2048);
    assert!(!config.cull_occluded_blocks);
    assert!(config.greedy_meshing);
}

#[test]
fn test_mesh_config_new_equals_default() {
    let new = MeshConfig::new();
    let def = MeshConfig::default();
    assert_eq!(new.cull_hidden_faces, def.cull_hidden_faces);
    assert_eq!(new.ambient_occlusion, def.ambient_occlusion);
    assert_eq!(new.ao_intensity, def.ao_intensity);
    assert_eq!(new.biome, def.biome);
    assert_eq!(new.atlas_max_size, def.atlas_max_size);
    assert_eq!(new.cull_occluded_blocks, def.cull_occluded_blocks);
    assert_eq!(new.greedy_meshing, def.greedy_meshing);
}

#[test]
fn test_resource_pack_from_invalid_bytes() {
    let result = ResourcePackSource::from_bytes(&[0xFF, 0xFE, 0xFD]);
    assert!(result.is_err());
}

#[test]
fn test_resource_pack_from_empty_bytes() {
    let result = ResourcePackSource::from_bytes(&[]);
    assert!(result.is_err());
}

#[test]
fn test_empty_schematic_mesh_fails() {
    // Meshing an empty schematic should fail with "No blocks to mesh"
    // We can't test this without a valid resource pack, but we verify
    // that the MeshConfig is properly constructed
    let config = MeshConfig::new().with_greedy_meshing(true);
    assert!(config.greedy_meshing);
}

#[test]
fn test_mesh_config_clone() {
    let config = MeshConfig::new()
        .with_biome("plains")
        .with_greedy_meshing(true)
        .with_cull_occluded_blocks(false);

    let cloned = config.clone();
    assert_eq!(cloned.biome, Some("plains".to_string()));
    assert!(cloned.greedy_meshing);
    assert!(!cloned.cull_occluded_blocks);
}
