#![cfg(feature = "meshing")]
//! Player heads render with the skin named in their block entity.

use base64::Engine;
use nucleation::block_entity::BlockEntity;
use nucleation::block_position::BlockPosition;
use nucleation::meshing::{build_global_atlas, MeshConfig, ResourcePackSource};
use nucleation::nbt::{NbtMap, NbtValue};
use nucleation::UniversalSchematic;
use schematic_mesher::resource_pack::{ResourcePack, TextureData};

const SKIN_URL: &str =
    "http://textures.minecraft.net/texture/7fd9ba42a7c81eeea22f1524271ae85a8e045ce0af5a6ae16c6406ae917e68b5";
const MAGENTA: [u8; 4] = [255, 0, 255, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

fn solid(rgba: [u8; 4], h: u32) -> TextureData {
    TextureData::new(64, h, rgba.iter().copied().cycle().take((64 * h * 4) as usize).collect())
}

fn pack() -> ResourcePackSource {
    let mut pack = ResourcePack::new();
    pack.add_texture("minecraft", "entity/player/wide/steve", solid(BLUE, 64));
    ResourcePackSource::from_resource_pack(pack)
}

fn profile_nbt(url: &str) -> NbtValue {
    let json = format!(r#"{{"textures":{{"SKIN":{{"url":"{url}"}}}}}}"#);
    let value = base64::engine::general_purpose::STANDARD.encode(json);
    let mut texture_prop = NbtMap::new();
    texture_prop.insert("name".into(), NbtValue::String("textures".into()));
    texture_prop.insert("value".into(), NbtValue::String(value));
    let mut profile = NbtMap::new();
    profile.insert("name".into(), NbtValue::String("Someone".into()));
    profile.insert("properties".into(), NbtValue::List(vec![NbtValue::Compound(texture_prop)]));
    NbtValue::Compound(profile)
}

fn schematic_with_heads() -> UniversalSchematic {
    let mut s = UniversalSchematic::new("heads".to_string());
    s.set_block_from_string(0, 0, 0, "minecraft:player_head[rotation=0]").unwrap();
    s.set_block_from_string(2, 0, 0, "minecraft:player_wall_head[facing=north]").unwrap();
    for (x, z) in [(0, 0), (2, 0)] {
        let be = BlockEntity::new("minecraft:skull".to_string(), (x, 0, z))
            .with_nbt_data("profile".to_string(), profile_nbt(SKIN_URL));
        assert!(s.set_block_entity(BlockPosition::new(x, 0, z), be));
    }
    s
}

fn atlas_has(pixels: &[u8], rgba: [u8; 4]) -> bool {
    pixels.chunks(4).any(|p| p == rgba)
}

#[test]
fn lists_the_skin_urls_a_build_needs() {
    assert_eq!(schematic_with_heads().player_skin_urls(), vec![SKIN_URL.to_string()]);
}

#[test]
fn registered_skin_is_used_and_unregistered_falls_back() {
    let s = schematic_with_heads();
    let config = MeshConfig::default();

    // Not registered: vanilla's default skin.
    let out = s.to_mesh(&pack(), &config).unwrap();
    assert!(atlas_has(&out.atlas.pixels, BLUE));
    assert!(!atlas_has(&out.atlas.pixels, MAGENTA));

    // Registered: the downloaded skin.
    let mut pack = pack();
    let png = solid(MAGENTA, 64).to_png().unwrap();
    let location = pack.add_player_skin(SKIN_URL, &png).unwrap();
    assert_eq!(location, "nucleation:skins/7fd9ba42a7c81eeea22f1524271ae85a8e045ce0af5a6ae16c6406ae917e68b5");
    let out = s.to_mesh(&pack, &config).unwrap();
    assert!(atlas_has(&out.atlas.pixels, MAGENTA));
    assert!(!atlas_has(&out.atlas.pixels, BLUE), "both heads share the registered skin");
}

#[test]
fn chunked_meshing_with_a_shared_atlas_finds_the_skin() {
    let s = schematic_with_heads();
    let config = MeshConfig::default();
    let mut pack = pack();
    pack.add_player_skin(SKIN_URL, &solid(MAGENTA, 64).to_png().unwrap()).unwrap();

    let atlas = build_global_atlas(&s, &pack, &config).unwrap();
    assert!(atlas_has(&atlas.pixels, MAGENTA), "global atlas holds the skin");
    let meshes: Vec<_> = s
        .mesh_chunks_with_atlas(&pack, &config, 16, atlas)
        .collect::<Result<_, _>>()
        .unwrap();
    let triangles: usize = meshes.iter().map(|m| m.total_triangles()).sum();
    assert!(triangles > 0);
}

#[test]
fn legacy_64x32_skins_are_padded_and_odd_sizes_rejected() {
    let mut pack = pack();
    pack.add_player_skin("abc123", &solid(MAGENTA, 32).to_png().unwrap()).unwrap();
    let info = pack.get_texture_info("nucleation:skins/abc123").unwrap();
    assert_eq!((info.0, info.1), (64, 64));

    let err = pack.add_player_skin("def456", &TextureData::new(16, 16, vec![0; 16 * 16 * 4]).to_png().unwrap());
    assert!(err.is_err());
    assert!(pack.add_player_skin("not a hash!", &solid(MAGENTA, 64).to_png().unwrap()).is_err());
}
