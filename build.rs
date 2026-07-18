//! Generates the vendored blockpedia block tables (see `src/blockpedia/`).
//!
//! Reads the gzipped data snapshots in `data/blockpedia/` (Java 26.2 block
//! states in the PrismarineJS schema, generated from Mojang's data-generator
//! reports; official block semantics — kind/base/tags/full-cube; Bedrock
//! block states; Geyser blockstate mappings; and the texture-derived color
//! cache) and writes two files into `OUT_DIR`:
//!
//!   - `block_table.rs`     — `BLOCKS` PHF map of `BlockFacts` + color query helpers
//!   - `bedrock_mappings.rs` — Java<->Bedrock blockstate string PHF maps
//!
//! This is the prebuilt-data path only: no network access, ever. Refreshing
//! the snapshots is done by the `mc-data-refresh` tools (`tools/mc-data/`),
//! which rewrite the `.json.gz` files; a normal build then picks them up.
//!
//! Ported from the standalone blockpedia crate's build.rs, minus its
//! network/build-data/texture-extraction paths (the committed color cache is
//! the product of the full texture pipeline and is always preferred).

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::path::Path;

/// Unified block data structure (same shape the blockpedia build used).
#[derive(Debug, Clone)]
struct UnifiedBlockData {
    id: String,
    properties: HashMap<String, Vec<String>>,
    default_state: HashMap<String, String>,
    transparent: bool,
    emit_light: u8,
    kind: String,
    base_block: Option<String>,
    tags: Vec<String>,
    full_cube: bool,
    has_block_entity: bool,
    bedrock_id: Option<String>,
    bedrock_properties: Option<HashMap<String, Vec<String>>>,
    bedrock_default_state: Option<HashMap<String, String>>,
}

/// Official block semantics parsed from `block_semantics.json.gz`
/// (kind / base block / tags / full-cube geometry / block entity;
/// see tools/mc-data/).
struct BlockSemantics {
    kind: String,
    base_block: Option<String>,
    tags: Vec<String>,
    full_cube: bool,
    block_entity: bool,
}

/// Parse `block_semantics.json.gz`: `id -> {kind, base?, tags[], full_cube}`.
fn parse_semantics(json_data: &str) -> Result<HashMap<String, BlockSemantics>> {
    let parsed: Value =
        serde_json::from_str(json_data).context("Failed to parse block semantics JSON")?;
    let map = parsed
        .as_object()
        .context("Block semantics JSON is not an object")?;
    let mut out = HashMap::new();
    for (id, entry) in map {
        out.insert(
            id.clone(),
            BlockSemantics {
                kind: entry["kind"]
                    .as_str()
                    .unwrap_or("minecraft:block")
                    .to_string(),
                base_block: entry["base"].as_str().map(String::from),
                tags: entry["tags"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|t| t.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                full_cube: entry["full_cube"].as_bool().unwrap_or(false),
                block_entity: entry["block_entity"].as_bool().unwrap_or(false),
            },
        );
    }
    Ok(out)
}

/// Attach the official semantics to the Java block list.
fn merge_semantics(java_blocks: &mut [UnifiedBlockData], semantics: &HashMap<String, BlockSemantics>) {
    let mut missing = 0usize;
    for block in java_blocks {
        match semantics.get(&block.id) {
            Some(s) => {
                block.kind = s.kind.clone();
                block.base_block = s.base_block.clone();
                block.tags = s.tags.clone();
                block.full_cube = s.full_cube;
                block.has_block_entity = s.block_entity;
            }
            None => missing += 1,
        }
    }
    if missing > 0 {
        println!("cargo:warning=blockpedia data: {missing} blocks missing from block_semantics.json.gz");
    }
}

fn read_gz(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open data file {}", path.display()))?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut out = String::new();
    decoder
        .read_to_string(&mut out)
        .with_context(|| format!("Failed to gunzip {}", path.display()))?;
    Ok(out)
}

/// Parse PrismarineJS pc blocks.json (array of block objects).
fn parse_prismarine(json_data: &str) -> Result<Vec<UnifiedBlockData>> {
    let parsed: Value = serde_json::from_str(json_data).context("Failed to parse PrismarineJS JSON")?;
    let blocks_array = parsed.as_array().context("PrismarineJS JSON is not an array")?;

    let mut unified_blocks = Vec::new();
    for block in blocks_array {
        let block_obj = block.as_object().context("Block is not an object")?;
        let name = block_obj
            .get("name")
            .and_then(|n| n.as_str())
            .context("Block missing name field")?;
        let id = format!("minecraft:{name}");

        let mut properties = HashMap::new();
        if let Some(states) = block_obj.get("states").and_then(|s| s.as_array()) {
            for state in states {
                if let Some(state_obj) = state.as_object() {
                    if let (Some(prop_name), Some(prop_type), Some(num_values)) = (
                        state_obj.get("name").and_then(|n| n.as_str()),
                        state_obj.get("type").and_then(|t| t.as_str()),
                        state_obj.get("num_values").and_then(|n| n.as_u64()),
                    ) {
                        let values = match prop_type {
                            "bool" => vec!["false".to_string(), "true".to_string()],
                            "int" | "enum" => {
                                if let Some(values_array) =
                                    state_obj.get("values").and_then(|v| v.as_array())
                                {
                                    values_array
                                        .iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                } else if prop_type == "int" {
                                    (0..num_values).map(|i| i.to_string()).collect()
                                } else {
                                    (0..num_values).map(|i| format!("value_{i}")).collect()
                                }
                            }
                            _ => vec!["unknown".to_string()],
                        };
                        properties.insert(prop_name.to_string(), values);
                    }
                }
            }
        }

        let transparent = block_obj
            .get("transparent")
            .and_then(|t| t.as_bool())
            .unwrap_or(false);

        let emit_light = block_obj
            .get("emitLight")
            .and_then(|l| l.as_u64())
            .unwrap_or(0)
            .min(15) as u8;

        // Default state property values, from the vanilla report's state
        // flagged `"default": true` (converted to `defaultProperties` by
        // tools/mc-data/refresh-block-data.rs).
        let mut default_state = HashMap::new();
        if let Some(defaults) = block_obj.get("defaultProperties").and_then(|d| d.as_object()) {
            for (name, value) in defaults {
                if let Some(value) = value.as_str() {
                    default_state.insert(name.clone(), value.to_string());
                }
            }
        }

        unified_blocks.push(UnifiedBlockData {
            id,
            properties,
            default_state,
            transparent,
            emit_light,
            kind: "minecraft:block".to_string(),
            base_block: None,
            tags: Vec::new(),
            full_cube: false,
            has_block_entity: false,
            bedrock_id: None,
            bedrock_properties: None,
            bedrock_default_state: None,
        });
    }
    Ok(unified_blocks)
}

/// Parse the Bedrock blockStates.json array into per-block property sets.
fn parse_bedrock_states(json_data: &str) -> Result<Vec<UnifiedBlockData>> {
    let parsed: Value =
        serde_json::from_str(json_data).context("Failed to parse Bedrock blockStates.json")?;
    let states_array = parsed
        .as_array()
        .context("Bedrock blockStates.json is not an array")?;

    type PropInfo = (HashMap<String, Vec<String>>, HashMap<String, String>);
    let mut block_info_map: HashMap<String, PropInfo> = HashMap::new();

    for state_entry in states_array {
        if let Some(state_obj) = state_entry.as_object() {
            if let Some(name) = state_obj.get("name").and_then(|n| n.as_str()) {
                let info = block_info_map
                    .entry(name.to_string())
                    .or_insert_with(|| (HashMap::new(), HashMap::new()));

                if let Some(states) = state_obj.get("states").and_then(|s| s.as_object()) {
                    for (prop_name, prop_val_obj) in states {
                        if let Some(val) = prop_val_obj.get("value") {
                            let val_str = match val {
                                Value::Bool(b) => b.to_string(),
                                Value::Number(n) => n.to_string(),
                                Value::String(s) => s.clone(),
                                _ => continue,
                            };
                            let values = info.0.entry(prop_name.clone()).or_default();
                            if !values.contains(&val_str) {
                                values.push(val_str.clone());
                            }
                            // First seen value becomes the default (heuristic)
                            info.1.entry(prop_name.clone()).or_insert(val_str);
                        }
                    }
                }
            }
        }
    }

    Ok(block_info_map
        .into_iter()
        .map(|(name, (properties, default_state))| {
            let id = format!("minecraft:{name}");
            UnifiedBlockData {
                id: id.clone(),
                properties: properties.clone(),
                default_state: default_state.clone(),
                transparent: false,
                emit_light: 0,
                kind: "minecraft:block".to_string(),
                base_block: None,
                tags: Vec::new(),
                full_cube: false,
                has_block_entity: false,
                bedrock_id: Some(id),
                bedrock_properties: Some(properties),
                bedrock_default_state: Some(default_state),
            }
        })
        .collect())
}

/// Attach Bedrock ids/properties to the Java block list (same heuristics as
/// the blockpedia build).
fn merge_bedrock_data(java_blocks: &mut [UnifiedBlockData], bedrock_blocks: Vec<UnifiedBlockData>) {
    let bedrock_map: HashMap<String, UnifiedBlockData> = bedrock_blocks
        .into_iter()
        .map(|b| (b.id.clone(), b))
        .collect();

    for java_block in java_blocks {
        let target_bedrock_id = match java_block.id.as_str() {
            "minecraft:wall_torch" => "minecraft:torch",
            "minecraft:redstone_wall_torch" => "minecraft:redstone_torch",
            "minecraft:soul_wall_torch" => "minecraft:soul_torch",
            "minecraft:grass_block" => "minecraft:grass",
            "minecraft:repeater" => "minecraft:unpowered_repeater",
            "minecraft:comparator" => "minecraft:unpowered_comparator",
            id => id,
        };
        if let Some(bedrock_block) = bedrock_map.get(target_bedrock_id) {
            java_block.bedrock_id = Some(bedrock_block.id.clone());
            java_block.bedrock_properties = Some(bedrock_block.properties.clone());
            java_block.bedrock_default_state = Some(bedrock_block.default_state.clone());
        }
    }
}

/// RGB (0-255) + simplified Oklab, as stored in the color cache.
type CachedColor = (u8, u8, u8, f32, f32, f32);

/// Load the texture-derived color cache and fill remaining holes by
/// inheriting from base materials (stairs/slabs/walls/fences/doors/...).
fn load_colors(data_dir: &Path, available_block_ids: &[String]) -> Result<HashMap<String, CachedColor>> {
    let cache_path = data_dir.join("color_cache.json.gz");
    let cache_data = read_gz(&cache_path)?;
    let mut colors: HashMap<String, CachedColor> =
        serde_json::from_str(&cache_data).context("Failed to parse color_cache.json")?;

    // Inheritance pass: blocks without their own texture-derived color take
    // the base material's color.
    let existing = colors.clone();
    let mut inherited = 0usize;
    for block_id in available_block_ids {
        if existing.contains_key(block_id) {
            continue;
        }
        if let Some(base) = base_material_for_block(block_id) {
            if let Some(color) = existing.get(&base) {
                colors.insert(block_id.clone(), *color);
                inherited += 1;
            }
        }
    }
    println!("cargo:warning=blockpedia data: {} cached colors, {} inherited", existing.len(), inherited);
    Ok(colors)
}

/// Base material lookup for color inheritance (verbatim from blockpedia).
fn base_material_for_block(block_id: &str) -> Option<String> {
    let block_name = block_id.strip_prefix("minecraft:").unwrap_or(block_id);

    if block_name.ends_with("_stairs") {
        let base = block_name.replace("_stairs", "");
        return Some(format!("minecraft:{base}"));
    }

    if block_name.ends_with("_slab") {
        let base = block_name.replace("_slab", "");
        return Some(match base.as_str() {
            "petrified_oak" => "minecraft:oak_planks".to_string(),
            "smooth_stone" => "minecraft:stone".to_string(),
            "cut_copper" | "waxed_cut_copper" => "minecraft:copper_block".to_string(),
            "exposed_cut_copper" | "waxed_exposed_cut_copper" => "minecraft:exposed_copper".to_string(),
            "weathered_cut_copper" | "waxed_weathered_cut_copper" => "minecraft:weathered_copper".to_string(),
            "oxidized_cut_copper" | "waxed_oxidized_cut_copper" => "minecraft:oxidized_copper".to_string(),
            "cut_red_sandstone" => "minecraft:red_sandstone".to_string(),
            "cut_sandstone" => "minecraft:sandstone".to_string(),
            "prismarine_brick" => "minecraft:prismarine_bricks".to_string(),
            "nether_brick" => "minecraft:nether_bricks".to_string(),
            "red_nether_brick" => "minecraft:red_nether_bricks".to_string(),
            "polished_blackstone_brick" => "minecraft:polished_blackstone_bricks".to_string(),
            "end_stone_brick" => "minecraft:end_stone_bricks".to_string(),
            "stone_brick" => "minecraft:stone_bricks".to_string(),
            "mossy_stone_brick" => "minecraft:mossy_stone_bricks".to_string(),
            "deepslate_brick" => "minecraft:deepslate_bricks".to_string(),
            "deepslate_tile" => "minecraft:deepslate_tiles".to_string(),
            "tuff_brick" => "minecraft:tuff_bricks".to_string(),
            "bamboo_mosaic" => "minecraft:bamboo_planks".to_string(),
            _ => format!("minecraft:{base}"),
        });
    }

    if block_name.ends_with("_wall") {
        let base = block_name.replace("_wall", "");
        return Some(match base.as_str() {
            "stone_brick" => "minecraft:stone_bricks".to_string(),
            "mossy_stone_brick" => "minecraft:mossy_stone_bricks".to_string(),
            "deepslate_brick" => "minecraft:deepslate_bricks".to_string(),
            "deepslate_tile" => "minecraft:deepslate_tiles".to_string(),
            "brick" => "minecraft:bricks".to_string(),
            "mud_brick" => "minecraft:mud_bricks".to_string(),
            "nether_brick" => "minecraft:nether_bricks".to_string(),
            "red_nether_brick" => "minecraft:red_nether_bricks".to_string(),
            "polished_blackstone_brick" => "minecraft:polished_blackstone_bricks".to_string(),
            "end_stone_brick" => "minecraft:end_stone_bricks".to_string(),
            "tuff_brick" => "minecraft:tuff_bricks".to_string(),
            _ => format!("minecraft:{base}"),
        });
    }

    if block_name.ends_with("_fence") && !block_name.ends_with("_fence_gate") {
        let base = block_name.replace("_fence", "");
        return Some(match base.as_str() {
            "nether_brick" => "minecraft:nether_bricks".to_string(),
            _ => format!("minecraft:{base}_planks"),
        });
    }

    if block_name.ends_with("_fence_gate") {
        let base = block_name.replace("_fence_gate", "");
        return Some(format!("minecraft:{base}_planks"));
    }

    if block_name.ends_with("_door") || block_name.ends_with("_trapdoor") {
        let suffix = if block_name.ends_with("_trapdoor") { "_trapdoor" } else { "_door" };
        let base = block_name.replace(suffix, "");
        return Some(match base.as_str() {
            "iron" => "minecraft:iron_block".to_string(),
            "copper" | "waxed_copper" => "minecraft:copper_block".to_string(),
            "exposed_copper" | "waxed_exposed_copper" => "minecraft:exposed_copper".to_string(),
            "weathered_copper" | "waxed_weathered_copper" => "minecraft:weathered_copper".to_string(),
            "oxidized_copper" | "waxed_oxidized_copper" => "minecraft:oxidized_copper".to_string(),
            _ => format!("minecraft:{base}_planks"),
        });
    }

    None
}

fn rust_ident_for(block_id: &str) -> String {
    block_id
        .replace(':', "_")
        .replace('-', "_")
        .replace(['\'', '!'], "")
        .replace('.', "_")
        .to_uppercase()
}

/// Emit `block_table.rs`: statics + PHF map + color query helpers.
///
/// All type paths are `crate::blockpedia::...` because the file is included
/// from `src/blockpedia/mod.rs` inside the nucleation crate.
fn generate_block_table(
    out_dir: &str,
    unified_blocks: &[UnifiedBlockData],
    colors: &HashMap<String, CachedColor>,
) -> Result<()> {
    let table_path = Path::new(out_dir).join("block_table.rs");
    let mut file = std::fs::File::create(&table_path).context("Failed to create block_table.rs")?;

    writeln!(file, "// Auto-generated PHF table from unified block data")?;
    writeln!(file, "use phf::{{phf_map, Map}};")?;
    writeln!(file)?;

    for block_data in unified_blocks {
        let block_id = &block_data.id;
        let safe_name = rust_ident_for(block_id);

        writeln!(
            file,
            "static {}: crate::blockpedia::BlockFacts = crate::blockpedia::BlockFacts {{",
            safe_name
        )?;
        writeln!(file, "    id: \"{}\",", block_id)?;
        writeln!(file, "    transparent: {},", block_data.transparent)?;
        writeln!(file, "    emit_light: {},", block_data.emit_light)?;
        writeln!(file, "    kind: \"{}\",", block_data.kind)?;
        match &block_data.base_block {
            Some(base) => writeln!(file, "    base_block: Some(\"{}\"),", base)?,
            None => writeln!(file, "    base_block: None,")?,
        }
        // Identical string literals are pooled by rustc, so repeating tag
        // names across blocks costs one copy each in rodata.
        write!(file, "    tags: &[")?;
        for (i, tag) in block_data.tags.iter().enumerate() {
            if i > 0 {
                write!(file, ", ")?;
            }
            write!(file, "\"{}\"", tag)?;
        }
        writeln!(file, "],")?;
        writeln!(file, "    full_cube: {},", block_data.full_cube)?;
        writeln!(file, "    has_block_entity: {},", block_data.has_block_entity)?;

        writeln!(file, "    properties: &[")?;
        for (prop_name, prop_values) in &block_data.properties {
            write!(file, "        (\"{}\", &[", prop_name)?;
            for (i, value) in prop_values.iter().enumerate() {
                if i > 0 {
                    write!(file, ", ")?;
                }
                write!(file, "\"{}\"", value)?;
            }
            writeln!(file, "]),")?;
        }
        writeln!(file, "    ],")?;

        writeln!(file, "    default_state: &[")?;
        for (state_name, state_value) in &block_data.default_state {
            writeln!(file, "        (\"{}\", \"{}\"),", state_name, state_value)?;
        }
        writeln!(file, "    ],")?;

        write!(file, "    extras: crate::blockpedia::Extras {{ mock_data: None,")?;

        if let Some((r, g, b, l, a, b_val)) = colors.get(block_id) {
            // Nudge values that would trip clippy::approx_constant
            let adjust = |v: f32| {
                if (v - std::f32::consts::FRAC_1_PI).abs() < 0.001 {
                    v + 0.001
                } else {
                    v
                }
            };
            write!(
                file,
                " color: Some(crate::blockpedia::ColorData {{ rgb: [{}, {}, {}], oklab: [{:.3}, {:.3}, {:.3}] }}),",
                r, g, b, adjust(*l), adjust(*a), adjust(*b_val)
            )?;
        } else {
            write!(file, " color: None,")?;
        }

        if let Some(ref bedrock_id) = block_data.bedrock_id {
            writeln!(file, " bedrock: Some(crate::blockpedia::BedrockData {{")?;
            writeln!(file, "     id: \"{}\",", bedrock_id)?;
            writeln!(file, "     properties: &[")?;
            if let Some(ref props) = block_data.bedrock_properties {
                for (prop_name, prop_values) in props {
                    write!(file, "         (\"{}\", &[", prop_name)?;
                    for (i, value) in prop_values.iter().enumerate() {
                        if i > 0 {
                            write!(file, ", ")?;
                        }
                        write!(file, "\"{}\"", value)?;
                    }
                    writeln!(file, "]),")?;
                }
            }
            writeln!(file, "     ],")?;
            writeln!(file, "     default_state: &[")?;
            if let Some(ref def_state) = block_data.bedrock_default_state {
                for (prop_name, prop_value) in def_state {
                    writeln!(file, "         (\"{}\", \"{}\"),", prop_name, prop_value)?;
                }
            }
            writeln!(file, "     ],")?;
            write!(file, " }}),")?;
        } else {
            write!(file, " bedrock: None,")?;
        }

        writeln!(file, " }},")?;
        writeln!(file, "}};")?;
        writeln!(file)?;
    }

    writeln!(
        file,
        "pub static BLOCKS: Map<&'static str, &'static crate::blockpedia::BlockFacts> = phf_map! {{"
    )?;
    for block_data in unified_blocks {
        let safe_name = rust_ident_for(&block_data.id);
        writeln!(file, "    \"{}\" => &{},", block_data.id, safe_name)?;
    }
    writeln!(file, "}};")?;
    writeln!(file)?;

    // Tag index: tag name -> block ids carrying it (drives `blocks_by_tag`).
    let mut tag_index: std::collections::BTreeMap<&str, Vec<&str>> = std::collections::BTreeMap::new();
    for block_data in unified_blocks {
        for tag in &block_data.tags {
            tag_index.entry(tag).or_default().push(&block_data.id);
        }
    }
    writeln!(
        file,
        "pub static BLOCK_TAGS: Map<&'static str, &'static [&'static str]> = phf_map! {{"
    )?;
    for (tag, ids) in &tag_index {
        write!(file, "    \"{}\" => &[", tag)?;
        for (i, id) in ids.iter().enumerate() {
            if i > 0 {
                write!(file, ", ")?;
            }
            write!(file, "\"{}\"", id)?;
        }
        writeln!(file, "],")?;
    }
    writeln!(file, "}};")?;
    writeln!(file)?;

    // Color query helpers (same generated API as blockpedia)
    writeln!(file, "// Generated query helper functions")?;
    writeln!(file, "impl crate::blockpedia::BlockFacts {{")?;
    writeln!(file, "    pub fn closest_to_color(target_rgb: [u8; 3]) -> Option<&'static Self> {{")?;
    writeln!(file, "        let target_oklab = rgb_to_oklab(target_rgb);")?;
    writeln!(file, "        let mut best_block = None;")?;
    writeln!(file, "        let mut best_distance = f32::INFINITY;")?;
    writeln!(file, "        for block in crate::blockpedia::all_blocks() {{")?;
    writeln!(file, "            if let Some(ref color) = block.extras.color {{")?;
    writeln!(file, "                let distance = oklab_distance(target_oklab, color.oklab);")?;
    writeln!(file, "                if distance < best_distance {{")?;
    writeln!(file, "                    best_distance = distance;")?;
    writeln!(file, "                    best_block = Some(block);")?;
    writeln!(file, "                }}")?;
    writeln!(file, "            }}")?;
    writeln!(file, "        }}")?;
    writeln!(file, "        best_block")?;
    writeln!(file, "    }}")?;
    writeln!(file)?;
    writeln!(file, "    pub fn blocks_in_color_range(center_rgb: [u8; 3], max_distance: f32) -> Vec<&'static Self> {{")?;
    writeln!(file, "        let center_oklab = rgb_to_oklab(center_rgb);")?;
    writeln!(file, "        let mut result = Vec::new();")?;
    writeln!(file, "        for block in crate::blockpedia::all_blocks() {{")?;
    writeln!(file, "            if let Some(ref color) = block.extras.color {{")?;
    writeln!(file, "                let distance = oklab_distance(center_oklab, color.oklab);")?;
    writeln!(file, "                if distance <= max_distance {{")?;
    writeln!(file, "                    result.push(block);")?;
    writeln!(file, "                }}")?;
    writeln!(file, "            }}")?;
    writeln!(file, "        }}")?;
    writeln!(file, "        result")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;
    writeln!(file)?;
    writeln!(file, "fn rgb_to_oklab(rgb: [u8; 3]) -> [f32; 3] {{")?;
    writeln!(file, "    // Simplified RGB to Oklab conversion for build-time")?;
    writeln!(file, "    let r = rgb[0] as f32 / 255.0;")?;
    writeln!(file, "    let g = rgb[1] as f32 / 255.0;")?;
    writeln!(file, "    let b = rgb[2] as f32 / 255.0;")?;
    writeln!(file, "    let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;")?;
    writeln!(file, "    let a = (r - g) * 0.5;")?;
    writeln!(file, "    let b_val = (r + g - 2.0 * b) * 0.25;")?;
    writeln!(file, "    [l, a, b_val]")?;
    writeln!(file, "}}")?;
    writeln!(file)?;
    writeln!(file, "fn oklab_distance(a: [f32; 3], b: [f32; 3]) -> f32 {{")?;
    writeln!(file, "    let dl = a[0] - b[0];")?;
    writeln!(file, "    let da = a[1] - b[1];")?;
    writeln!(file, "    let db = a[2] - b[2];")?;
    writeln!(file, "    (dl * dl + da * da + db * db).sqrt()")?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    println!(
        "cargo:warning=blockpedia data: generated PHF table with {} blocks",
        unified_blocks.len()
    );
    Ok(())
}

/// Emit `bedrock_mappings.rs` from the Geyser mappings snapshot.
fn generate_bedrock_mappings(out_dir: &str, data_dir: &Path) -> Result<()> {
    let mappings_path = Path::new(out_dir).join("bedrock_mappings.rs");
    let mut file =
        std::fs::File::create(&mappings_path).context("Failed to create bedrock_mappings.rs")?;

    writeln!(file, "// Auto-generated bedrock blockstate mappings")?;
    writeln!(file)?;

    let geyser_data = read_gz(&data_dir.join("geyser_mappings.json.gz"))?;
    let parsed: Value =
        serde_json::from_str(&geyser_data).context("Failed to parse geyser_mappings.json")?;
    let mappings = parsed
        .get("mappings")
        .and_then(|m| m.as_array())
        .context("Invalid Geyser mappings format (missing 'mappings' array)")?;

    writeln!(
        file,
        "pub static BEDROCK_J2B_MAP: phf::Map<&'static str, &'static str> = phf_map! {{"
    )?;

    let mut b2j_map: HashMap<String, String> = HashMap::new();

    for mapping in mappings {
        let java_state_obj = mapping.get("java_state").and_then(|s| s.as_object());
        let bedrock_state_obj = mapping.get("bedrock_state").and_then(|s| s.as_object());

        if let (Some(java), Some(bedrock)) = (java_state_obj, bedrock_state_obj) {
            let java_name = java.get("Name").and_then(|n| n.as_str()).unwrap_or("");
            let java_props = java.get("Properties").and_then(|p| p.as_object());
            let java_state_str = if let Some(props) = java_props {
                let mut props_vec: Vec<String> = props
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v.as_str().unwrap_or("")))
                    .collect();
                props_vec.sort();
                format!("{}[{}]", java_name, props_vec.join(","))
            } else {
                format!("{}[]", java_name)
            };

            // Geyser identifiers often lack the "minecraft:" prefix for vanilla
            let bedrock_id_raw = bedrock
                .get("bedrock_identifier")
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let bedrock_id = if !bedrock_id_raw.contains(':') {
                format!("minecraft:{bedrock_id_raw}")
            } else {
                bedrock_id_raw.to_string()
            };

            let bedrock_props = bedrock.get("state").and_then(|s| s.as_object());
            let bedrock_state_str = if let Some(props) = bedrock_props {
                let mut props_vec: Vec<String> = props
                    .iter()
                    .map(|(k, v)| {
                        let val_str = match v {
                            Value::Bool(b) => b.to_string(),
                            Value::Number(n) => n.to_string(),
                            Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        };
                        format!("{}={}", k, val_str)
                    })
                    .collect();
                props_vec.sort();
                format!("{}[{}]", bedrock_id, props_vec.join(","))
            } else {
                format!("{}[]", bedrock_id)
            };

            writeln!(file, "    r#\"{}\"# => r#\"{}\"#,", java_state_str, bedrock_state_str)?;
            b2j_map.entry(bedrock_state_str).or_insert(java_state_str);
        }
    }
    writeln!(file, "}};")?;
    writeln!(file)?;

    writeln!(
        file,
        "pub static BEDROCK_B2J_MAP: phf::Map<&'static str, &'static str> = phf_map! {{"
    )?;
    for (bedrock_state, java_state) in &b2j_map {
        writeln!(file, "    r#\"{}\"# => r#\"{}\"#,", bedrock_state, java_state)?;
    }
    writeln!(file, "}};")?;

    println!(
        "cargo:warning=blockpedia data: generated {} Java->Bedrock / {} Bedrock->Java mappings",
        mappings.len(),
        b2j_map.len()
    );
    Ok(())
}

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let data_dir = Path::new(&manifest_dir).join("data").join("blockpedia");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/blockpedia/prismarinejs_blocks.json.gz");
    println!("cargo:rerun-if-changed=data/blockpedia/bedrock_block_states.json.gz");
    println!("cargo:rerun-if-changed=data/blockpedia/geyser_mappings.json.gz");
    println!("cargo:rerun-if-changed=data/blockpedia/color_cache.json.gz");
    println!("cargo:rerun-if-changed=data/blockpedia/block_semantics.json.gz");

    let prismarine_json = read_gz(&data_dir.join("prismarinejs_blocks.json.gz"))?;
    let mut java_blocks = parse_prismarine(&prismarine_json)?;

    let semantics_json = read_gz(&data_dir.join("block_semantics.json.gz"))?;
    let semantics = parse_semantics(&semantics_json)?;
    merge_semantics(&mut java_blocks, &semantics);

    let bedrock_json = read_gz(&data_dir.join("bedrock_block_states.json.gz"))?;
    let bedrock_blocks = parse_bedrock_states(&bedrock_json)?;
    merge_bedrock_data(&mut java_blocks, bedrock_blocks);

    let available_block_ids: Vec<String> = java_blocks.iter().map(|b| b.id.clone()).collect();
    let colors = load_colors(&data_dir, &available_block_ids)?;

    generate_block_table(&out_dir, &java_blocks, &colors)?;
    generate_bedrock_mappings(&out_dir, &data_dir)?;

    Ok(())
}
