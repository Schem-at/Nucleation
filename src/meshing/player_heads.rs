//! Player-head skins.
//!
//! A player head's skin lives in its block entity, not its block state: a
//! `profile` (1.20.5+) or `SkullOwner` (older) compound holding the owner's
//! UUID and a base64 `textures` property whose JSON points at the skin on
//! `textures.minecraft.net`. The mesher only sees block properties, so this
//! module reads the profile and passes it on as:
//!
//! - `uuid`: picks vanilla's default skin when no other skin is available;
//! - `skin_texture`: a resource-pack texture to use as the skin, either the
//!   profile's own `texture` override or `nucleation:skins/<hash>` for a
//!   downloaded skin the caller registered with
//!   [`ResourcePackSource::add_player_skin`](super::ResourcePackSource::add_player_skin);
//! - `slim`: from the profile's `model` or the skin's metadata.
//!
//! Nucleation never fetches skins itself. [`UniversalSchematic::player_skin_urls`]
//! lists the URLs a build needs, so a caller can download and register them.

use base64::Engine;
use schematic_mesher::InputBlock;

use crate::nbt::{NbtMap, NbtValue};
use crate::{BlockState, Region, UniversalSchematic};

/// Namespace registered skins live under in the resource pack.
pub const SKIN_NAMESPACE: &str = "nucleation";

/// What a player head's profile says about its skin.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerProfile {
    /// Owner UUID, hyphenated lowercase.
    pub uuid: Option<String>,
    /// Owner name.
    pub name: Option<String>,
    /// Skin URL from the `textures` property.
    pub skin_url: Option<String>,
    /// `true` for the slim (Alex) arm model, from `model` or the skin metadata.
    pub slim: Option<bool>,
    /// Resource-pack texture the profile asks for instead of a downloaded skin
    /// (the 1.21.9+ `texture` field).
    pub texture_override: Option<String>,
}

/// Whether a block id is a player head (standing or on a wall).
pub fn is_player_head(block_name: &str) -> bool {
    let id = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    matches!(id, "player_head" | "player_wall_head")
}

/// The skin's texture hash: the last path segment of a `textures.minecraft.net`
/// URL, or the input itself when it already is a bare hash.
pub fn skin_hash(url_or_hash: &str) -> Option<String> {
    let s = url_or_hash.trim();
    let s = s.split(['?', '#']).next().unwrap_or(s);
    let last = s.trim_end_matches('/').rsplit('/').next()?;
    if !last.is_empty() && last.chars().all(|c| c.is_ascii_alphanumeric()) {
        Some(last.to_ascii_lowercase())
    } else {
        None
    }
}

/// Resource location a registered skin is stored under: `nucleation:skins/<hash>`.
pub fn skin_texture_location(hash: &str) -> String {
    format!("{SKIN_NAMESPACE}:skins/{hash}")
}

/// Read a head's profile from its block-entity NBT. `None` when the head has
/// no owner at all.
pub fn read_profile(nbt: &NbtMap) -> Option<PlayerProfile> {
    let value = nbt.get("profile").or_else(|| nbt.get("SkullOwner"))?;
    let mut profile = PlayerProfile::default();
    match value {
        // A bare name (both the old `SkullOwner` string and the 1.21.9+ name-only profile).
        NbtValue::String(name) => profile.name = Some(name.clone()),
        NbtValue::Compound(c) => {
            profile.uuid = c.get("id").or_else(|| c.get("Id")).and_then(uuid_from_nbt);
            profile.name = c
                .get("name")
                .or_else(|| c.get("Name"))
                .and_then(NbtValue::as_string)
                .cloned();
            if let Some(texture) = textures_value(c) {
                if let Some((url, slim)) = decode_textures(&texture) {
                    profile.skin_url = Some(url);
                    profile.slim = slim;
                }
            }
            if let Some(NbtValue::String(t)) = c.get("texture") {
                profile.texture_override = Some(t.clone());
            }
            if let Some(NbtValue::String(m)) = c.get("model") {
                profile.slim = Some(m == "slim");
            }
        }
        _ => return None,
    }
    Some(profile)
}

/// Copy a profile onto a player head's mesher input.
pub fn apply_profile(input: &mut InputBlock, profile: &PlayerProfile) {
    if let Some(uuid) = &profile.uuid {
        input.properties.insert("uuid".into(), uuid.clone());
    }
    let skin_texture = profile
        .texture_override
        .clone()
        .or_else(|| profile.skin_url.as_deref().and_then(skin_hash).map(|h| skin_texture_location(&h)));
    if let Some(t) = skin_texture {
        input.properties.insert("skin_texture".into(), t);
    }
    if let Some(slim) = profile.slim {
        input.properties.insert("slim".into(), slim.to_string());
    }
}

/// Mesher inputs for every player head in `region` that has a profile, keyed
/// by world position. These replace the plain block-state inputs.
pub(super) fn player_head_inputs(region: &Region) -> Vec<((i32, i32, i32), InputBlock)> {
    let mut out = Vec::new();
    for (pos, be) in region.block_entities.iter() {
        let Some(state) = region.get_block(pos.0, pos.1, pos.2) else { continue };
        if !is_player_head(&state.name) {
            continue;
        }
        let Some(profile) = read_profile(&be.nbt) else { continue };
        let mut input = super::block_state_to_input_block(state);
        apply_profile(&mut input, &profile);
        out.push((pos, input));
    }
    out.sort_by_key(|(p, _)| *p);
    out
}

fn head_profiles<'a>(region: &'a Region) -> impl Iterator<Item = PlayerProfile> + 'a {
    region.block_entities.iter().filter_map(move |(pos, be)| {
        let state: &BlockState = region.get_block(pos.0, pos.1, pos.2)?;
        if !is_player_head(&state.name) {
            return None;
        }
        read_profile(&be.nbt)
    })
}

impl UniversalSchematic {
    /// Skin URLs of the player heads in this schematic that need a downloaded
    /// skin (sorted, without duplicates). Fetch each and register it with
    /// [`ResourcePackSource::add_player_skin`](super::ResourcePackSource::add_player_skin)
    /// to render the heads with their real skins; unregistered ones fall back
    /// to vanilla's default skin for the owner's UUID.
    pub fn player_skin_urls(&self) -> Vec<String> {
        let mut urls: Vec<String> = std::iter::once(&self.default_region)
            .chain(self.other_regions.values())
            .flat_map(head_profiles)
            .filter(|p| p.texture_override.is_none())
            .filter_map(|p| p.skin_url)
            .collect();
        urls.sort();
        urls.dedup();
        urls
    }
}

/// `[I; a, b, c, d]` (Java `UUIDUtil` order) or a string, as a hyphenated UUID.
fn uuid_from_nbt(v: &NbtValue) -> Option<String> {
    match v {
        NbtValue::IntArray(a) if a.len() == 4 => {
            let most = ((a[0] as u32 as u64) << 32) | a[1] as u32 as u64;
            let least = ((a[2] as u32 as u64) << 32) | a[3] as u32 as u64;
            let hex = format!("{most:016x}{least:016x}");
            Some(format!(
                "{}-{}-{}-{}-{}",
                &hex[0..8],
                &hex[8..12],
                &hex[12..16],
                &hex[16..20],
                &hex[20..32]
            ))
        }
        NbtValue::String(s) => {
            let hex: String = s.chars().filter(|c| *c != '-').collect();
            (hex.len() == 32 && hex.chars().all(|c| c.is_ascii_hexdigit())).then(|| s.to_ascii_lowercase())
        }
        _ => None,
    }
}

/// The base64 `textures` property value, from either
/// `properties: [{name: "textures", value}]` (1.20.5+) or
/// `Properties: {textures: [{Value}]}` (older).
fn textures_value(profile: &NbtMap) -> Option<String> {
    if let Some(NbtValue::List(props)) = profile.get("properties") {
        for p in props {
            if let NbtValue::Compound(p) = p {
                if p.get("name").and_then(NbtValue::as_string).map(String::as_str) == Some("textures") {
                    if let Some(v) = p.get("value").and_then(NbtValue::as_string) {
                        return Some(v.clone());
                    }
                }
            }
        }
    }
    // Some writers keep the 1.20.5 properties as a name → list map.
    for key in ["properties", "Properties"] {
        if let Some(NbtValue::Compound(props)) = profile.get(key) {
            if let Some(NbtValue::List(list)) = props.get("textures") {
                for entry in list {
                    if let NbtValue::Compound(e) = entry {
                        if let Some(v) = e.get("Value").or_else(|| e.get("value")).and_then(NbtValue::as_string) {
                            return Some(v.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Decode a `textures` property: base64 JSON
/// `{"textures": {"SKIN": {"url": ..., "metadata": {"model": "slim"}}}}`.
fn decode_textures(value: &str) -> Option<(String, Option<bool>)> {
    let engines = [
        base64::engine::general_purpose::STANDARD,
        base64::engine::general_purpose::STANDARD_NO_PAD,
        base64::engine::general_purpose::URL_SAFE,
        base64::engine::general_purpose::URL_SAFE_NO_PAD,
    ];
    let bytes = engines.iter().find_map(|e| e.decode(value.trim()).ok())?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let skin = json.get("textures")?.get("SKIN")?;
    let url = skin.get("url")?.as_str()?.to_string();
    let slim = skin
        .get("metadata")
        .and_then(|m| m.get("model"))
        .and_then(|m| m.as_str())
        .map(|m| m == "slim");
    Some((url, slim))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn textures_b64(url: &str, slim: bool) -> String {
        let meta = if slim { r#","metadata":{"model":"slim"}"# } else { "" };
        let json = format!(r#"{{"timestamp":1,"textures":{{"SKIN":{{"url":"{url}"{meta}}}}}}}"#);
        base64::engine::general_purpose::STANDARD.encode(json)
    }

    fn compound(entries: Vec<(&str, NbtValue)>) -> NbtMap {
        let mut m = NbtMap::new();
        for (k, v) in entries {
            m.insert(k.to_string(), v);
        }
        m
    }

    const URL: &str = "http://textures.minecraft.net/texture/7fd9ba42a7c81eeea22f1524271ae85a8e045ce0af5a6ae16c6406ae917e68b5";

    #[test]
    fn reads_a_modern_profile() {
        let profile = compound(vec![
            ("id", NbtValue::IntArray(vec![0x069a79f4, 0x44e94726u32 as i32, 0xa5befca9u32 as i32, 0x0e38aaf5])),
            ("name", NbtValue::String("Notch".into())),
            (
                "properties",
                NbtValue::List(vec![NbtValue::Compound(compound(vec![
                    ("name", NbtValue::String("textures".into())),
                    ("value", NbtValue::String(textures_b64(URL, true))),
                ]))]),
            ),
        ]);
        let nbt = compound(vec![("profile", NbtValue::Compound(profile))]);
        let p = read_profile(&nbt).unwrap();
        assert_eq!(p.uuid.as_deref(), Some("069a79f4-44e9-4726-a5be-fca90e38aaf5"));
        assert_eq!(p.name.as_deref(), Some("Notch"));
        assert_eq!(p.skin_url.as_deref(), Some(URL));
        assert_eq!(p.slim, Some(true));

        let mut input = InputBlock::new("minecraft:player_head");
        apply_profile(&mut input, &p);
        assert_eq!(
            input.properties.get("skin_texture").map(String::as_str),
            Some("nucleation:skins/7fd9ba42a7c81eeea22f1524271ae85a8e045ce0af5a6ae16c6406ae917e68b5")
        );
        assert_eq!(input.properties.get("uuid").map(String::as_str), Some("069a79f4-44e9-4726-a5be-fca90e38aaf5"));
        assert_eq!(input.properties.get("slim").map(String::as_str), Some("true"));
    }

    #[test]
    fn reads_a_legacy_skull_owner() {
        let owner = compound(vec![
            ("Id", NbtValue::String("069A79F4-44E9-4726-A5BE-FCA90E38AAF5".into())),
            ("Name", NbtValue::String("Notch".into())),
            (
                "Properties",
                NbtValue::Compound(compound(vec![(
                    "textures",
                    NbtValue::List(vec![NbtValue::Compound(compound(vec![(
                        "Value",
                        NbtValue::String(textures_b64(URL, false)),
                    )]))]),
                )])),
            ),
        ]);
        let p = read_profile(&compound(vec![("SkullOwner", NbtValue::Compound(owner))])).unwrap();
        assert_eq!(p.uuid.as_deref(), Some("069a79f4-44e9-4726-a5be-fca90e38aaf5"));
        assert_eq!(p.skin_url.as_deref(), Some(URL));
        assert_eq!(p.slim, None);
    }

    #[test]
    fn texture_override_wins_over_the_downloaded_skin() {
        let profile = compound(vec![
            ("texture", NbtValue::String("minecraft:entity/player/wide/zuri".into())),
            ("model", NbtValue::String("wide".into())),
        ]);
        let p = read_profile(&compound(vec![("profile", NbtValue::Compound(profile))])).unwrap();
        let mut input = InputBlock::new("minecraft:player_wall_head");
        apply_profile(&mut input, &p);
        assert_eq!(
            input.properties.get("skin_texture").map(String::as_str),
            Some("minecraft:entity/player/wide/zuri")
        );
        assert_eq!(input.properties.get("slim").map(String::as_str), Some("false"));
    }

    #[test]
    fn name_only_and_ownerless_heads() {
        let p = read_profile(&compound(vec![("profile", NbtValue::String("jeb_".into()))])).unwrap();
        assert_eq!(p.name.as_deref(), Some("jeb_"));
        assert!(p.uuid.is_none() && p.skin_url.is_none());
        assert!(read_profile(&NbtMap::new()).is_none());
    }

    #[test]
    fn skin_hash_from_url_or_hash() {
        assert_eq!(skin_hash(URL).as_deref(), Some("7fd9ba42a7c81eeea22f1524271ae85a8e045ce0af5a6ae16c6406ae917e68b5"));
        assert_eq!(skin_hash("https://textures.minecraft.net/texture/ABC123?x=1").as_deref(), Some("abc123"));
        assert_eq!(skin_hash("abc123").as_deref(), Some("abc123"));
        assert_eq!(skin_hash("https://example.com/skin.png"), None);
        assert_eq!(skin_hash(""), None);
    }
}
