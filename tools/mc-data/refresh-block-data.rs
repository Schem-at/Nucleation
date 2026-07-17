//! Regenerates `data/blockpedia/prismarinejs_blocks.json.gz` from Mojang's
//! own data generator (requires the `mc-data-refresh` feature).
//!
//! PrismarineJS minecraft-data stopped short of the 26.x era, so the block
//! list and blockstate property schema now come from the authoritative
//! source — the vanilla server jar's built-in report generator — while the
//! enrichment fields PrismarineJS used to provide (transparency, hardness,
//! bounding box, light, ...) are carried forward from the previous snapshot.
//!
//! Pipeline:
//!
//! 1. Resolves the target version (first CLI arg; defaults to the version
//!    manifest's latest release) and downloads the server jar.
//! 2. Runs `java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar
//!    --reports` (needs a JRE new enough for the jar; MC 26.x wants 25+) and
//!    reads `generated/reports/blocks.json`.
//! 3. Rebuilds every block entry in the PrismarineJS `blocks.json` schema
//!    that `build.rs` consumes:
//!    - block list, properties and state ids: from the vanilla report
//!      (authoritative),
//!    - enrichment fields: copied from the existing snapshot when the block
//!      id already exists there,
//!    - blocks new in this version: enriched from an *analogue* block (see
//!      `analogue_override` / `ROOT_ANALOGUES`) when one exists, otherwise
//!      from a model-shape heuristic over the client jar's
//!      blockstates/models assets (see `ModelClassifier`) plus conservative
//!      stone-like defaults (hardness 1.5, resistance 6.0, no light).
//! 4. Prints the added/removed/changed diff and the derived facts for every
//!    new block, then rewrites the gzipped snapshot.
//!
//! Run from the repo root:
//! `cargo run --release --bin refresh-block-data --features mc-data-refresh [-- <version>]`
//!
//! Afterwards run `fetch-texture-colors` (same version) to refresh the color
//! cache, then `cargo build` to bake both into the PHF tables.

use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

const SNAPSHOT_PATH: &str = "data/blockpedia/prismarinejs_blocks.json.gz";

fn main() -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("nucleation-block-data-refresh")
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    println!("Fetching version manifest...");
    let manifest: Value = client
        .get(VERSION_MANIFEST_URL)
        .send()?
        .error_for_status()?
        .json()
        .context("Failed to parse version manifest")?;

    let version = match std::env::args().nth(1) {
        Some(v) => v,
        None => manifest["latest"]["release"]
            .as_str()
            .context("manifest has no latest.release")?
            .to_string(),
    };
    println!("Target Minecraft version: {version}");

    let version_url = manifest["versions"]
        .as_array()
        .context("manifest has no versions array")?
        .iter()
        .find(|v| v["id"].as_str() == Some(&version))
        .and_then(|v| v["url"].as_str())
        .with_context(|| format!("Version {version} not found in manifest"))?;
    let meta: Value = client
        .get(version_url)
        .send()?
        .error_for_status()?
        .json()
        .context("Failed to parse version metadata")?;

    let work_dir = Path::new("target/mc-data");
    fs::create_dir_all(work_dir)?;

    // --- vanilla blocks report (server jar data generator) ---
    let server_jar = download_jar(&client, &meta, "server", &version, work_dir)?;
    let report = vanilla_blocks_report(&server_jar, &version, work_dir)?;

    // --- client jar for the model/texture transparency heuristic ---
    // (same cache path fetch-texture-colors uses)
    let client_jar = download_jar(&client, &meta, "client", &version, work_dir)?;
    let mut classifier = ModelClassifier::open(&client_jar)?;

    // --- previous snapshot (enrichment carry-forward source) ---
    let old_json = read_gz(Path::new(SNAPSHOT_PATH))?;
    let old_blocks: Vec<Value> = serde_json::from_str(&old_json).context("bad old snapshot")?;
    let old_by_name: HashMap<&str, &Value> = old_blocks
        .iter()
        .filter_map(|b| b["name"].as_str().map(|n| (n, b)))
        .collect();

    // --- id-set diff ---
    let old_names: BTreeSet<&str> = old_by_name.keys().copied().collect();
    let new_names: BTreeSet<&str> = report
        .keys()
        .map(|k| k.strip_prefix("minecraft:").unwrap_or(k))
        .collect();
    let added: Vec<&str> = new_names.difference(&old_names).copied().collect();
    let removed: Vec<&str> = old_names.difference(&new_names).copied().collect();

    println!();
    println!(
        "Vanilla {version} report: {} blocks ({} added, {} removed vs snapshot)",
        new_names.len(),
        added.len(),
        removed.len()
    );
    if !removed.is_empty() {
        println!("REMOVED blocks (check palettes/tests; a rename shows up as remove+add):");
        for name in &removed {
            println!("  minecraft:{name}");
        }
    }

    // --- property-schema changes on carried-over blocks ---
    let mut prop_changes = 0usize;
    for (id, entry) in &report {
        let name = id.strip_prefix("minecraft:").unwrap_or(id);
        let Some(old) = old_by_name.get(name) else { continue };
        let new_props = property_sets(entry);
        let old_props = old_property_sets(old);
        if new_props != old_props {
            prop_changes += 1;
            println!("Property schema changed: {id}");
            for key in new_props
                .keys()
                .chain(old_props.keys())
                .collect::<BTreeSet<_>>()
            {
                if new_props.get(key) != old_props.get(key) {
                    println!(
                        "    {key}: {:?} -> {:?}",
                        old_props.get(key),
                        new_props.get(key)
                    );
                }
            }
        }
    }
    if prop_changes == 0 {
        println!("No property-schema changes on carried-over blocks.");
    }

    // --- rebuild every entry ---
    let mut out = Vec::with_capacity(report.len());
    let mut new_block_report: Vec<String> = Vec::new();

    for (index, (id, entry)) in report.iter().enumerate() {
        let name = id.strip_prefix("minecraft:").unwrap_or(id);
        let states = states_from_report(entry);
        let (min_id, max_id, default_id) = state_ids(entry)
            .with_context(|| format!("{id}: bad states in report"))?;

        let mut block = match old_by_name.get(name) {
            // Existing block: keep all enrichment fields, refresh the schema.
            Some(old) => (*old).clone(),
            // New block: enrich from an analogue or the model heuristic.
            None => {
                let model = classifier.classify(name);
                let analogue = find_analogue(name, &old_by_name);
                let (transparent, bounding_box, filter_light, emit_light, hardness, resistance, material, diggable, stack_size) =
                    match &analogue {
                        Some((_, a)) => (
                            a["transparent"].as_bool().unwrap_or(false),
                            a["boundingBox"].as_str().unwrap_or("block").to_string(),
                            a["filterLight"].as_u64().unwrap_or(0),
                            a["emitLight"].as_u64().unwrap_or(0),
                            a["hardness"].clone(),
                            a["resistance"].clone(),
                            a["material"].as_str().unwrap_or("default").to_string(),
                            a["diggable"].as_bool().unwrap_or(true),
                            a["stackSize"].as_u64().unwrap_or(64),
                        ),
                        None => (
                            model.transparent,
                            model.bounding_box.to_string(),
                            model.filter_light,
                            0,
                            json!(1.5),
                            json!(6.0),
                            "mineable/pickaxe".to_string(),
                            true,
                            64,
                        ),
                    };
                new_block_report.push(format!(
                    "  {id}: transparent={transparent} boundingBox={bounding_box} filterLight={filter_light} \
                     hardness={hardness} material={material} \
                     [source: {}; model heuristic says transparent={} ({})]",
                    analogue
                        .as_ref()
                        .map(|(n, _)| format!("analogue minecraft:{n}"))
                        .unwrap_or_else(|| "model heuristic + defaults".to_string()),
                    model.transparent,
                    model.basis,
                ));
                json!({
                    "name": name,
                    "displayName": display_name(name),
                    "hardness": hardness,
                    "resistance": resistance,
                    "stackSize": stack_size,
                    "diggable": diggable,
                    "material": material,
                    "transparent": transparent,
                    "emitLight": emit_light,
                    "filterLight": filter_light,
                    "drops": [],
                    "boundingBox": bounding_box,
                })
            }
        };

        let obj = block.as_object_mut().context("block entry not an object")?;
        obj.insert("id".into(), json!(index));
        obj.insert("states".into(), Value::Array(states));
        obj.insert("minStateId".into(), json!(min_id));
        obj.insert("maxStateId".into(), json!(max_id));
        obj.insert("defaultState".into(), json!(default_id));
        out.push(block);
    }

    if !new_block_report.is_empty() {
        println!();
        println!("NEW blocks in {version} and their derived facts:");
        for line in &new_block_report {
            println!("{line}");
        }
    }

    // --- write the snapshot ---
    let pretty = serde_json::to_string_pretty(&out)?;
    write_gz(Path::new(SNAPSHOT_PATH), &pretty)?;
    println!();
    println!("Wrote {SNAPSHOT_PATH} ({} blocks).", out.len());
    println!("Next: `cargo run --release --bin fetch-texture-colors --features mc-data-refresh -- {version}`");
    println!("then `cargo build` to bake the new tables in.");

    Ok(())
}

// ---------------------------------------------------------------------------
// Vanilla report handling
// ---------------------------------------------------------------------------

fn download_jar(
    client: &reqwest::blocking::Client,
    meta: &Value,
    which: &str,
    version: &str,
    work_dir: &Path,
) -> Result<PathBuf> {
    let jar_path = work_dir.join(format!("{which}-{version}.jar"));
    if jar_path.exists() {
        println!("Using cached {which} jar {}", jar_path.display());
        return Ok(jar_path);
    }
    let url = meta["downloads"][which]["url"]
        .as_str()
        .with_context(|| format!("No {which} jar download in version metadata"))?;
    println!("Downloading {which} jar from {url}...");
    let bytes = client.get(url).send()?.error_for_status()?.bytes()?;
    fs::write(&jar_path, &bytes)?;
    println!("Saved {} ({:.1} MB)", jar_path.display(), bytes.len() as f64 / 1e6);
    Ok(jar_path)
}

/// Runs the server jar's bundled data generator and parses
/// `generated/reports/blocks.json` (cached per version).
fn vanilla_blocks_report(
    server_jar: &Path,
    version: &str,
    work_dir: &Path,
) -> Result<Map<String, Value>> {
    let gen_dir = work_dir.join(format!("gen-{version}"));
    let report_path = gen_dir.join("generated/reports/blocks.json");

    if !report_path.exists() {
        fs::create_dir_all(&gen_dir)?;
        let server_jar = server_jar.canonicalize()?;
        println!("Running the vanilla data generator (java -DbundlerMainClass=net.minecraft.data.Main)...");
        let output = Command::new("java")
            .arg("-DbundlerMainClass=net.minecraft.data.Main")
            .arg("-jar")
            .arg(&server_jar)
            .arg("--reports")
            .current_dir(&gen_dir)
            .output()
            .context("Failed to launch `java` (is a JRE on PATH?)")?;
        if !output.status.success() || !report_path.exists() {
            bail!(
                "Data generator failed (MC 26.x needs Java 25+ on PATH):\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        println!("Using cached vanilla report {}", report_path.display());
    }

    let report: Value = serde_json::from_str(&fs::read_to_string(&report_path)?)
        .context("Failed to parse generated blocks.json report")?;
    match report {
        Value::Object(map) => Ok(map),
        _ => bail!("blocks.json report is not an object"),
    }
}

/// Report `properties` object -> PrismarineJS `states` array. Property order
/// is alphabetical (serde_json's map ordering), matching the old snapshots.
fn states_from_report(entry: &Value) -> Vec<Value> {
    let Some(props) = entry["properties"].as_object() else {
        return Vec::new();
    };
    props
        .iter()
        .map(|(name, vals)| {
            let vals: Vec<&str> = vals
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();
            if vals.len() == 2 && vals.contains(&"true") && vals.contains(&"false") {
                json!({"name": name, "type": "bool", "num_values": 2})
            } else if vals.iter().all(|v| v.parse::<i64>().is_ok()) {
                json!({"name": name, "type": "int", "num_values": vals.len(), "values": vals})
            } else {
                json!({"name": name, "type": "enum", "num_values": vals.len(), "values": vals})
            }
        })
        .collect()
}

/// (minStateId, maxStateId, defaultState) from the report's `states` array —
/// these are the vanilla global blockstate ids.
fn state_ids(entry: &Value) -> Option<(u64, u64, u64)> {
    let states = entry["states"].as_array()?;
    let ids: Vec<u64> = states.iter().filter_map(|s| s["id"].as_u64()).collect();
    let default = states
        .iter()
        .find(|s| s["default"].as_bool() == Some(true))
        .and_then(|s| s["id"].as_u64())
        .or_else(|| ids.first().copied())?;
    Some((*ids.iter().min()?, *ids.iter().max()?, default))
}

/// Property name -> sorted value set, from a report entry.
fn property_sets(entry: &Value) -> HashMap<String, BTreeSet<String>> {
    entry["properties"]
        .as_object()
        .map(|props| {
            props
                .iter()
                .map(|(k, v)| {
                    let vals = v
                        .as_array()
                        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    (k.clone(), vals)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Property name -> sorted value set, from an old snapshot entry.
fn old_property_sets(block: &Value) -> HashMap<String, BTreeSet<String>> {
    let mut out = HashMap::new();
    if let Some(states) = block["states"].as_array() {
        for s in states {
            let Some(name) = s["name"].as_str() else { continue };
            let vals: BTreeSet<String> = if s["type"].as_str() == Some("bool") {
                ["true".to_string(), "false".to_string()].into()
            } else {
                s["values"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| match v {
                                Value::String(s) => Some(s.clone()),
                                Value::Number(n) => Some(n.to_string()),
                                _ => None,
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            };
            out.insert(name.to_string(), vals);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Enrichment for new blocks
// ---------------------------------------------------------------------------

/// Explicit per-block analogue overrides for new blocks whose closest
/// existing relative can't be derived by the `ROOT_ANALOGUES` swap (same
/// pattern as `texture_mapping::texture_override`).
fn analogue_override(name: &str) -> Option<&'static str> {
    Some(match name {
        // 26.x (cinnabar/sulfur update)
        "sulfur_spike" => "pointed_dripstone",
        "potent_sulfur" => "tuff",
        _ => return None,
    })
}

/// New material families mapped onto their closest existing family; applied
/// as a substring swap so shape variants (slabs/stairs/walls/bricks/
/// polished/chiseled/potted) resolve automatically, e.g.
/// `polished_cinnabar_slab` -> `polished_tuff_slab`.
const ROOT_ANALOGUES: &[(&str, &str)] = &[
    ("golden_dandelion", "dandelion"),
    ("cinnabar", "tuff"),
    ("sulfur", "tuff"),
];

/// Finds the enrichment analogue for a new block, if any.
fn find_analogue<'a>(
    name: &str,
    old_by_name: &HashMap<&str, &'a Value>,
) -> Option<(String, &'a Value)> {
    if let Some(over) = analogue_override(name) {
        if let Some(block) = old_by_name.get(over) {
            return Some((over.to_string(), block));
        }
    }
    for (root, analogue) in ROOT_ANALOGUES {
        if name.contains(root) {
            let candidate = name.replace(root, analogue);
            if let Some(block) = old_by_name.get(candidate.as_str()) {
                return Some((candidate, block));
            }
        }
    }
    None
}

fn display_name(name: &str) -> String {
    name.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Model-shape transparency heuristic (client jar assets)
// ---------------------------------------------------------------------------

/// Derived render/shape facts for a block with no analogue.
struct ModelDerived {
    transparent: bool,
    bounding_box: &'static str,
    filter_light: u64,
    /// Human-readable basis for the decision (printed in the report).
    basis: String,
}

/// Model templates that always mean "full opaque cube" (unless the texture
/// itself is translucent).
const CUBE_TEMPLATES: &[&str] = &[
    "block/cube",
    "block/cube_all",
    "block/cube_all_inner_faces",
    "block/cube_column",
    "block/cube_column_horizontal",
    "block/cube_bottom_top",
    "block/cube_directional",
    "block/cube_north_west_mirrored",
    "block/cube_mirrored_all",
];

/// Partial-but-solid shapes: PrismarineJS marks these `transparent: false`
/// with `filterLight: 0` (stairs/slabs/walls).
const PARTIAL_SOLID_TEMPLATES: &[&str] = &[
    "block/slab",
    "block/slab_top",
    "block/stairs",
    "block/inner_stairs",
    "block/outer_stairs",
    "block/template_wall_post",
    "block/template_wall_side",
    "block/template_wall_side_tall",
];

/// Plant-style cross models: transparent, no collision shape.
const CROSS_TEMPLATES: &[&str] = &["block/cross", "block/tinted_cross"];
const POT_TEMPLATES: &[&str] = &["block/flower_pot_cross", "block/tinted_flower_pot_cross"];

/// Classifies a block by walking its blockstate's model parent chains inside
/// the client jar:
///
/// - every variant rooted in a `cube*` template and fully opaque textures
///   -> opaque full cube (`transparent: false`, `filterLight: 15`)
/// - `cube*` template but a texture with translucent pixels -> glass-like
///   (`transparent: true`, `filterLight: 0`)
/// - any slab/stairs/wall template -> partial solid (`transparent: false`,
///   `filterLight: 0`, PrismarineJS convention)
/// - cross/flower-pot templates -> plant (`transparent: true`, bounding box
///   `empty` for bare crosses)
/// - anything else (custom models) -> conservative `transparent: true`,
///   `filterLight: 0`
struct ModelClassifier {
    archive: zip::ZipArchive<fs::File>,
    model_cache: HashMap<String, Option<Value>>,
}

impl ModelClassifier {
    fn open(client_jar: &Path) -> Result<Self> {
        let file = fs::File::open(client_jar)
            .with_context(|| format!("Failed to open {}", client_jar.display()))?;
        let archive = zip::ZipArchive::new(file).context("Client jar is not a valid zip")?;
        Ok(Self { archive, model_cache: HashMap::new() })
    }

    fn read_json(&mut self, path: &str) -> Option<Value> {
        let mut entry = self.archive.by_name(path).ok()?;
        let mut buf = String::new();
        entry.read_to_string(&mut buf).ok()?;
        serde_json::from_str(&buf).ok()
    }

    fn model(&mut self, model_ref: &str) -> Option<Value> {
        let key = model_ref.strip_prefix("minecraft:").unwrap_or(model_ref).to_string();
        if let Some(cached) = self.model_cache.get(&key) {
            return cached.clone();
        }
        let value = self.read_json(&format!("assets/minecraft/models/{key}.json"));
        self.model_cache.insert(key, value.clone());
        value
    }

    /// All model refs used by a block's blockstate definition.
    fn blockstate_models(&mut self, name: &str) -> Vec<String> {
        let Some(bs) = self.read_json(&format!("assets/minecraft/blockstates/{name}.json")) else {
            return Vec::new();
        };
        let mut models = BTreeSet::new();
        collect_model_refs(&bs, &mut models);
        models.into_iter().collect()
    }

    /// The parent chain of a model (normalized, without `minecraft:`).
    fn parent_chain(&mut self, model_ref: &str) -> Vec<String> {
        let mut chain = vec![model_ref.strip_prefix("minecraft:").unwrap_or(model_ref).to_string()];
        for _ in 0..8 {
            let Some(m) = self.model(chain.last().unwrap().clone().as_str()) else { break };
            let Some(parent) = m["parent"].as_str() else { break };
            chain.push(parent.strip_prefix("minecraft:").unwrap_or(parent).to_string());
        }
        chain
    }

    /// True if any texture referenced by the model has meaningfully
    /// translucent pixels (>1% below alpha 250).
    fn has_translucent_texture(&mut self, model_ref: &str) -> bool {
        let Some(m) = self.model(model_ref) else { return false };
        let Some(textures) = m["textures"].as_object() else { return false };
        for tex in textures.values().filter_map(|t| t.as_str()) {
            if tex.starts_with('#') {
                continue;
            }
            let tex = tex.strip_prefix("minecraft:").unwrap_or(tex);
            let path = format!("assets/minecraft/textures/{tex}.png");
            let Ok(mut entry) = self.archive.by_name(&path) else { continue };
            let mut buf = Vec::new();
            if entry.read_to_end(&mut buf).is_err() {
                continue;
            }
            drop(entry);
            if let Ok(img) = image::load_from_memory(&buf) {
                let rgba = img.to_rgba8();
                let total = rgba.pixels().len().max(1);
                let translucent = rgba.pixels().filter(|p| p[3] < 250).count();
                if translucent * 100 > total {
                    return true;
                }
            }
        }
        false
    }

    fn classify(&mut self, name: &str) -> ModelDerived {
        let models = self.blockstate_models(name);
        if models.is_empty() {
            return ModelDerived {
                transparent: true,
                bounding_box: "block",
                filter_light: 0,
                basis: "no blockstate asset; conservative default".to_string(),
            };
        }

        let chains: Vec<Vec<String>> = models.iter().map(|m| self.parent_chain(m)).collect();
        let in_set = |chain: &[String], set: &[&str]| chain.iter().any(|c| set.contains(&c.as_str()));

        if chains.iter().all(|c| in_set(c, CUBE_TEMPLATES)) {
            if models.iter().any(|m| self.has_translucent_texture(m)) {
                return ModelDerived {
                    transparent: true,
                    bounding_box: "block",
                    filter_light: 0,
                    basis: "cube model with translucent texture".to_string(),
                };
            }
            return ModelDerived {
                transparent: false,
                bounding_box: "block",
                filter_light: 15,
                basis: "all variants are cube models with opaque textures".to_string(),
            };
        }
        if chains.iter().any(|c| in_set(c, PARTIAL_SOLID_TEMPLATES)) {
            return ModelDerived {
                transparent: false,
                bounding_box: "block",
                filter_light: 0,
                basis: "slab/stairs/wall template (partial solid)".to_string(),
            };
        }
        if chains.iter().any(|c| in_set(c, POT_TEMPLATES)) {
            return ModelDerived {
                transparent: true,
                bounding_box: "block",
                filter_light: 0,
                basis: "flower pot template".to_string(),
            };
        }
        if chains.iter().any(|c| in_set(c, CROSS_TEMPLATES)) {
            return ModelDerived {
                transparent: true,
                bounding_box: "empty",
                filter_light: 0,
                basis: "cross (plant) template".to_string(),
            };
        }
        ModelDerived {
            transparent: true,
            bounding_box: "block",
            filter_light: 0,
            basis: "custom model; conservative default".to_string(),
        }
    }
}

/// Recursively collects `"model": "..."` refs from a blockstate definition.
fn collect_model_refs(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if k == "model" {
                    if let Some(s) = v.as_str() {
                        out.insert(s.to_string());
                    }
                }
                collect_model_refs(v, out);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_model_refs(v, out);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// gz helpers
// ---------------------------------------------------------------------------

fn read_gz(path: &Path) -> Result<String> {
    let file = fs::File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut out = String::new();
    decoder.read_to_string(&mut out)?;
    Ok(out)
}

fn write_gz(path: &Path, contents: &str) -> Result<()> {
    use std::io::Write;
    let file = fs::File::create(path)
        .with_context(|| format!("Failed to create {}", path.display()))?;
    let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::best());
    encoder.write_all(contents.as_bytes())?;
    encoder.finish()?;
    Ok(())
}
