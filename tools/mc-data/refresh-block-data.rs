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
//! 5. Extracts official block *semantics* into
//!    `data/blockpedia/block_semantics.json.gz` (id -> kind/base/tags/
//!    full_cube/block_entity):
//!    - `kind`/`base`: the report's `definition.type` and
//!      `definition.base_state.Name` (official variant linkage),
//!    - `tags`: every `data/minecraft/tags/block/**.json` from the server
//!      jar (the bundler's inner jar), `#tag` references resolved,
//!    - `full_cube`: the client jar's blockstate models resolve to a
//!      cube-family template or a full 16x16x16 element (plus
//!      `full_cube_override` for model-ambiguous blocks like the huge
//!      mushrooms),
//!    - `block_entity`: the `block_entity_type` registry
//!      (`generated/reports/registries.json`) joined to the report's
//!      `definition.type` kinds,
//!    - blocks without `base_state` are linked to a base by their resolved
//!      model texture set (e.g. `oak_slab` uses exactly `block/oak_planks`,
//!      which `oak_planks` owns).
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
const SEMANTICS_PATH: &str = "data/blockpedia/block_semantics.json.gz";
/// Plain-text marker of the Minecraft version the snapshots were generated
/// from; the weekly `data-refresh` workflow compares it against the version
/// manifest's `latest.release` to decide whether a refresh is needed.
const DATA_VERSION_PATH: &str = "data/blockpedia/DATA_VERSION";

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
        let Some(old) = old_by_name.get(name) else {
            continue;
        };
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
        let (min_id, max_id, default_id) =
            state_ids(entry).with_context(|| format!("{id}: bad states in report"))?;

        let mut block = match old_by_name.get(name) {
            // Existing block: keep all enrichment fields, refresh the schema.
            Some(old) => (*old).clone(),
            // New block: enrich from an analogue or the model heuristic.
            None => {
                let model = classifier.classify(name);
                let analogue = find_analogue(name, &old_by_name);
                let (
                    transparent,
                    bounding_box,
                    filter_light,
                    emit_light,
                    hardness,
                    resistance,
                    material,
                    diggable,
                    stack_size,
                ) = match &analogue {
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
        // The default state's property map, straight from the report state
        // flagged `"default": true` (the numeric `defaultState` id above is
        // useless to consumers that key states by property values).
        obj.insert(
            "defaultProperties".into(),
            Value::Object(default_state_properties(entry)),
        );
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

    // --- block semantics (kind / base / tags / full_cube / block_entity) ---
    let gen_dir = work_dir.join(format!("gen-{version}"));
    generate_semantics(&report, &server_jar, &mut classifier, &gen_dir)?;

    // --- version marker (drives the automated data-refresh workflow) ---
    fs::write(DATA_VERSION_PATH, format!("{version}\n"))
        .with_context(|| format!("Failed to write {DATA_VERSION_PATH}"))?;
    println!("Wrote {DATA_VERSION_PATH} ({version}).");

    println!();
    println!("Next: `cargo run --release --bin fetch-texture-colors --features mc-data-refresh -- {version}`");
    println!("then `cargo build` to bake the new tables in.");

    Ok(())
}

// ---------------------------------------------------------------------------
// Block semantics (kind / base / tags / full_cube)
// ---------------------------------------------------------------------------

/// Builds `block_semantics.json.gz` from official data only:
///
/// - `kind`: the vanilla report's `definition.type`
/// - `base`: `definition.base_state.Name` when present (stairs); otherwise a
///   model-texture linkage — a block whose blockstate models resolve to
///   exactly the texture set owned by a single full-cube block gets that
///   block as its base (`oak_slab` -> `oak_planks`, `stone_brick_wall` ->
///   `stone_bricks`). Ambiguous texture ownership is broken by a texture
///   whose file stem names the owner (`block/stone` -> `minecraft:stone`,
///   not `infested_stone`); still-ambiguous links are dropped.
/// - `tags`: `data/minecraft/tags/block/**.json` from the server jar,
///   nested `#tag` references resolved recursively.
/// - `full_cube`: every model of the blockstate roots in a cube-family
///   template or carries a full 16x16x16 element (catches `grass_block`,
///   `command_block` and other non-template cubes). Blocks whose render
///   geometry is not classifiable from models alone are settled by
///   `full_cube_override` (mushroom blocks render as a multipart shell of
///   face planes — indistinguishable from vines — but are full cubes).
/// - `block_entity`: the block carries a block entity ("tile entity") —
///   derived by joining the `minecraft:block_entity_type` registry
///   (`generated/reports/registries.json`) to the blocks report's
///   `definition.type` kinds (see `block_entity_kinds`).
fn generate_semantics(
    report: &Map<String, Value>,
    server_jar: &Path,
    classifier: &mut ModelClassifier,
    gen_dir: &Path,
) -> Result<()> {
    println!();
    println!("Extracting block semantics (kind / base / tags / full_cube / block_entity)...");

    let tags_by_block = extract_block_tags(server_jar)?;
    let entity_kinds = block_entity_kinds(gen_dir)?;

    // Pass 1: kind + full_cube for every block; collect texture ownership of
    // full cubes for the base linkage below.
    let mut kinds: HashMap<&str, &str> = HashMap::new();
    let mut full_cubes: BTreeSet<&str> = BTreeSet::new();
    let mut cube_textures: HashMap<&str, BTreeSet<String>> = HashMap::new();
    for (id, entry) in report {
        let name = id.strip_prefix("minecraft:").unwrap_or(id);
        let kind = entry["definition"]["type"]
            .as_str()
            .unwrap_or("minecraft:block");
        kinds.insert(id.as_str(), kind);
        let full_cube = full_cube_override(name).unwrap_or_else(|| classifier.is_full_cube(name));
        if full_cube {
            full_cubes.insert(id.as_str());
            let textures = classifier.texture_set(name);
            if !textures.is_empty() {
                cube_textures.insert(id.as_str(), textures);
            }
        }
    }

    // Texture set -> owning full-cube blocks (for exact-set matches).
    let mut owners_by_set: HashMap<&BTreeSet<String>, Vec<&str>> = HashMap::new();
    for (id, set) in &cube_textures {
        owners_by_set.entry(set).or_default().push(id);
    }

    // Pass 2: emit each block's record.
    let mut semantics = Map::new();
    let mut base_from_report = 0usize;
    let mut base_from_textures = 0usize;
    let mut block_entities = 0usize;
    let mut matched_entity_kinds: BTreeSet<&str> = BTreeSet::new();
    for (id, entry) in report {
        let name = id.strip_prefix("minecraft:").unwrap_or(id);
        let kind = kinds[id.as_str()];
        let full_cube = full_cubes.contains(id.as_str());
        let block_entity = entity_kinds.contains(kind);
        if block_entity {
            block_entities += 1;
            matched_entity_kinds.insert(kind);
        }

        let mut base: Option<String> = entry["definition"]["base_state"]["Name"]
            .as_str()
            .map(String::from);
        if base.is_some() {
            base_from_report += 1;
        } else if !full_cube {
            let textures = classifier.texture_set(name);
            if let Some(owner) = base_by_textures(&textures, &owners_by_set, &cube_textures) {
                base = Some(owner.to_string());
                base_from_textures += 1;
            }
        }

        let tags: Vec<&String> = tags_by_block
            .get(id.as_str())
            .map(|t| t.iter().collect())
            .unwrap_or_default();

        let mut record = Map::new();
        record.insert("kind".into(), json!(kind));
        if let Some(base) = base {
            record.insert("base".into(), json!(base));
        }
        record.insert("tags".into(), json!(tags));
        record.insert("full_cube".into(), json!(full_cube));
        record.insert("block_entity".into(), json!(block_entity));
        semantics.insert(id.clone(), Value::Object(record));
    }

    // Every kind derived from the block_entity_type registry must have
    // matched at least one block — a miss means a rename in the report or a
    // stale `BLOCK_ENTITY_KIND_OVERRIDES` table.
    let unmatched: Vec<&str> = entity_kinds
        .iter()
        .map(String::as_str)
        .filter(|k| !matched_entity_kinds.contains(k))
        .collect();
    if !unmatched.is_empty() {
        bail!(
            "block_entity_type kinds matched no block (update BLOCK_ENTITY_KIND_OVERRIDES): {}",
            unmatched.join(", ")
        );
    }

    let tagged = semantics
        .values()
        .filter(|v| !v["tags"].as_array().map(|a| a.is_empty()).unwrap_or(true))
        .count();
    println!(
        "Semantics: {} blocks, {} full cubes, {} tagged, {} block entities, base links: {} from report + {} from model textures",
        semantics.len(),
        full_cubes.len(),
        tagged,
        block_entities,
        base_from_report,
        base_from_textures,
    );

    let pretty = serde_json::to_string_pretty(&Value::Object(semantics))?;
    write_gz(Path::new(SEMANTICS_PATH), &pretty)?;
    println!("Wrote {SEMANTICS_PATH}.");
    Ok(())
}

/// Explicit full-cube overrides for blocks whose render geometry cannot be
/// classified from the client-jar models (same pattern as
/// `texture_mapping::texture_override` and `analogue_override`).
///
/// The huge-mushroom blocks render as a six-face multipart shell of
/// individual face planes — model geometry identical to vines, which are
/// genuinely non-full — but they collide, occlude and light as full opaque
/// cubes in game.
fn full_cube_override(name: &str) -> Option<bool> {
    Some(match name {
        "brown_mushroom_block" | "red_mushroom_block" | "mushroom_stem" => true,
        _ => return None,
    })
}

/// `block_entity_type` registry entries whose blocks are NOT simply "every
/// block whose report `definition.type` kind carries the same name". Maps a
/// registry entry to the report kinds it owns; entries absent here map to
/// the kind with the identical name (`minecraft:barrel` -> kind
/// `minecraft:barrel`).
const BLOCK_ENTITY_KIND_OVERRIDES: &[(&str, &[&str])] = &[
    (
        "minecraft:banner",
        &["minecraft:banner", "minecraft:wall_banner"],
    ),
    ("minecraft:brushable_block", &["minecraft:brushable"]),
    (
        "minecraft:chest",
        &[
            "minecraft:chest",
            "minecraft:copper_chest",
            "minecraft:weathering_copper_chest",
        ],
    ),
    (
        "minecraft:chiseled_bookshelf",
        &["minecraft:chiseled_book_shelf"],
    ),
    ("minecraft:command_block", &["minecraft:command"]),
    (
        "minecraft:copper_golem_statue",
        &[
            "minecraft:copper_golem_statue",
            "minecraft:weathering_copper_golem_statue",
        ],
    ),
    (
        "minecraft:enchanting_table",
        &["minecraft:enchantment_table"],
    ),
    (
        "minecraft:hanging_sign",
        &[
            "minecraft:ceiling_hanging_sign",
            "minecraft:wall_hanging_sign",
        ],
    ),
    ("minecraft:mob_spawner", &["minecraft:spawner"]),
    // The `piston` block entity belongs to the in-motion technical block,
    // not the piston bases (which carry no block entity).
    ("minecraft:piston", &["minecraft:moving_piston"]),
    (
        "minecraft:sign",
        &["minecraft:standing_sign", "minecraft:wall_sign"],
    ),
    (
        "minecraft:skull",
        &[
            "minecraft:skull",
            "minecraft:wall_skull",
            "minecraft:wither_skull",
            "minecraft:wither_wall_skull",
            "minecraft:player_head",
            "minecraft:player_wall_head",
            "minecraft:piglinwallskull",
        ],
    ),
    ("minecraft:structure_block", &["minecraft:structure"]),
    ("minecraft:test_block", &["minecraft:test"]),
    (
        "minecraft:test_instance_block",
        &["minecraft:test_instance"],
    ),
];

/// The set of report `definition.type` kinds whose blocks carry a block
/// entity, derived from the authoritative `minecraft:block_entity_type`
/// registry in `generated/reports/registries.json`: each registry entry
/// maps to the kind of the same name unless `BLOCK_ENTITY_KIND_OVERRIDES`
/// says otherwise.
fn block_entity_kinds(gen_dir: &Path) -> Result<BTreeSet<String>> {
    let registries_path = gen_dir.join("generated/reports/registries.json");
    let registries: Value = serde_json::from_str(
        &fs::read_to_string(&registries_path)
            .with_context(|| format!("Failed to read {}", registries_path.display()))?,
    )
    .context("Failed to parse registries.json report")?;
    let entries = registries["minecraft:block_entity_type"]["entries"]
        .as_object()
        .context("registries.json has no minecraft:block_entity_type registry")?;

    let overrides: HashMap<&str, &[&str]> = BLOCK_ENTITY_KIND_OVERRIDES.iter().copied().collect();
    let mut kinds = BTreeSet::new();
    for entry in entries.keys() {
        match overrides.get(entry.as_str()) {
            Some(mapped) => kinds.extend(mapped.iter().map(|k| k.to_string())),
            None => {
                kinds.insert(entry.clone());
            }
        }
    }
    println!(
        "Block entities: {} registry entries -> {} block kinds",
        entries.len(),
        kinds.len()
    );
    Ok(kinds)
}

/// Resolves a variant block's base by its model texture set: the unique
/// full-cube block owning exactly (or, failing that, a superset of) the
/// variant's textures. Ambiguities are broken by a texture file stem naming
/// the owner; otherwise no base is emitted.
fn base_by_textures<'a>(
    textures: &BTreeSet<String>,
    owners_by_set: &HashMap<&BTreeSet<String>, Vec<&'a str>>,
    cube_textures: &HashMap<&'a str, BTreeSet<String>>,
) -> Option<&'a str> {
    if textures.is_empty() {
        return None;
    }
    let exact: Vec<&str> = owners_by_set
        .get(textures)
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let candidates: Vec<&str> = if !exact.is_empty() {
        exact
    } else {
        cube_textures
            .iter()
            .filter(|(_, set)| textures.is_subset(set))
            .map(|(id, _)| *id)
            .collect()
    };
    match candidates.len() {
        0 => None,
        1 => Some(candidates[0]),
        _ => {
            // Tie-break: a texture like `block/stone` names its owner.
            let stems: BTreeSet<&str> = textures
                .iter()
                .filter_map(|t| t.rsplit('/').next())
                .collect();
            let mut named: Vec<&str> = candidates
                .iter()
                .filter(|id| {
                    let name = id.strip_prefix("minecraft:").unwrap_or(id);
                    stems.contains(name)
                })
                .copied()
                .collect();
            named.sort();
            match named.len() {
                1 => Some(named[0]),
                _ => None,
            }
        }
    }
}

/// Reads every block tag from the server jar and resolves nested `#tag`
/// references, producing `block id -> sorted tag names`.
///
/// The distributed server jar is a *bundler*: `META-INF/versions.list` points
/// at the real server jar inside `META-INF/versions/`, which carries the
/// vanilla datapack (`data/minecraft/tags/block/**.json`, including
/// subdirectories like `mineable/`).
fn extract_block_tags(server_jar: &Path) -> Result<HashMap<String, BTreeSet<String>>> {
    let file = fs::File::open(server_jar)
        .with_context(|| format!("Failed to open {}", server_jar.display()))?;
    let mut outer = zip::ZipArchive::new(file).context("Server jar is not a valid zip")?;

    // Locate + extract the inner jar (bundler format).
    let mut versions_list = String::new();
    outer
        .by_name("META-INF/versions.list")
        .context("Server jar has no META-INF/versions.list (not a bundler?)")?
        .read_to_string(&mut versions_list)?;
    let inner_rel = versions_list
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().last())
        .context("META-INF/versions.list is empty")?;
    let mut inner_bytes = Vec::new();
    outer
        .by_name(&format!("META-INF/versions/{inner_rel}"))
        .with_context(|| format!("Inner jar META-INF/versions/{inner_rel} missing"))?
        .read_to_end(&mut inner_bytes)?;
    let mut jar = zip::ZipArchive::new(std::io::Cursor::new(inner_bytes))
        .context("Inner server jar is not a valid zip")?;

    // Raw tag files: tag name (e.g. `wool`, `mineable/pickaxe`) -> entries.
    const PREFIX: &str = "data/minecraft/tags/block/";
    let mut raw: HashMap<String, Vec<String>> = HashMap::new();
    for i in 0..jar.len() {
        let mut entry = jar.by_index(i)?;
        let path = entry.name().to_string();
        let Some(tag_name) = path
            .strip_prefix(PREFIX)
            .and_then(|p| p.strip_suffix(".json"))
            .map(String::from)
        else {
            continue;
        };
        let mut buf = String::new();
        entry.read_to_string(&mut buf)?;
        let parsed: Value =
            serde_json::from_str(&buf).with_context(|| format!("Bad tag json: {path}"))?;
        let values = parsed["values"]
            .as_array()
            .with_context(|| format!("Tag {tag_name} has no values array"))?
            .iter()
            .filter_map(|v| match v {
                // Entries are either plain ids/`#tag` refs or
                // `{"id": ..., "required": bool}` objects.
                Value::String(s) => Some(s.clone()),
                Value::Object(o) => o.get("id").and_then(|s| s.as_str()).map(String::from),
                _ => None,
            })
            .collect();
        raw.insert(tag_name, values);
    }
    if raw.is_empty() {
        bail!("No block tags found in the server jar");
    }

    // Resolve nested `#minecraft:tag` references to block-id sets.
    fn resolve(
        tag: &str,
        raw: &HashMap<String, Vec<String>>,
        memo: &mut HashMap<String, BTreeSet<String>>,
        visiting: &mut Vec<String>,
    ) -> BTreeSet<String> {
        if let Some(done) = memo.get(tag) {
            return done.clone();
        }
        if visiting.iter().any(|t| t == tag) {
            return BTreeSet::new(); // cycle guard
        }
        visiting.push(tag.to_string());
        let mut blocks = BTreeSet::new();
        for value in raw.get(tag).map(|v| v.as_slice()).unwrap_or(&[]) {
            if let Some(nested) = value.strip_prefix('#') {
                let nested = nested.strip_prefix("minecraft:").unwrap_or(nested);
                blocks.extend(resolve(nested, raw, memo, visiting));
            } else {
                blocks.insert(value.clone());
            }
        }
        visiting.pop();
        memo.insert(tag.to_string(), blocks.clone());
        blocks
    }

    let mut memo = HashMap::new();
    let mut by_block: HashMap<String, BTreeSet<String>> = HashMap::new();
    let tag_names: Vec<String> = raw.keys().cloned().collect();
    for tag in &tag_names {
        for block in resolve(tag, &raw, &mut memo, &mut Vec::new()) {
            by_block
                .entry(block)
                .or_default()
                .insert(format!("minecraft:{tag}"));
        }
    }
    println!(
        "Block tags: {} tags covering {} blocks",
        tag_names.len(),
        by_block.len()
    );
    Ok(by_block)
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
    println!(
        "Saved {} ({:.1} MB)",
        jar_path.display(),
        bytes.len() as f64 / 1e6
    );
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

/// The property map of the report state flagged `"default": true` (falls
/// back to the first state, mirroring `state_ids`). Empty for
/// property-less blocks.
fn default_state_properties(entry: &Value) -> Map<String, Value> {
    entry["states"]
        .as_array()
        .and_then(|states| {
            states
                .iter()
                .find(|s| s["default"].as_bool() == Some(true))
                .or_else(|| states.first())
        })
        .and_then(|s| s["properties"].as_object().cloned())
        .unwrap_or_default()
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
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(String::from))
                                .collect()
                        })
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
            let Some(name) = s["name"].as_str() else {
                continue;
            };
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
        Ok(Self {
            archive,
            model_cache: HashMap::new(),
        })
    }

    fn read_json(&mut self, path: &str) -> Option<Value> {
        let mut entry = self.archive.by_name(path).ok()?;
        let mut buf = String::new();
        entry.read_to_string(&mut buf).ok()?;
        serde_json::from_str(&buf).ok()
    }

    fn model(&mut self, model_ref: &str) -> Option<Value> {
        let key = model_ref
            .strip_prefix("minecraft:")
            .unwrap_or(model_ref)
            .to_string();
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
        let mut chain = vec![model_ref
            .strip_prefix("minecraft:")
            .unwrap_or(model_ref)
            .to_string()];
        for _ in 0..8 {
            let Some(m) = self.model(chain.last().unwrap().clone().as_str()) else {
                break;
            };
            let Some(parent) = m["parent"].as_str() else {
                break;
            };
            chain.push(
                parent
                    .strip_prefix("minecraft:")
                    .unwrap_or(parent)
                    .to_string(),
            );
        }
        chain
    }

    /// True if any texture referenced by the model has meaningfully
    /// translucent pixels (>1% below alpha 250).
    fn has_translucent_texture(&mut self, model_ref: &str) -> bool {
        let Some(m) = self.model(model_ref) else {
            return false;
        };
        let Some(textures) = m["textures"].as_object() else {
            return false;
        };
        for tex in textures.values().filter_map(|t| t.as_str()) {
            if tex.starts_with('#') {
                continue;
            }
            let tex = tex.strip_prefix("minecraft:").unwrap_or(tex);
            let path = format!("assets/minecraft/textures/{tex}.png");
            let Ok(mut entry) = self.archive.by_name(&path) else {
                continue;
            };
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

    /// True when every model referenced by the block's blockstate is a full
    /// cube: rooted in a cube-family template, or (for non-template models
    /// like `grass_block` and `command_block`) carrying a full
    /// 16x16x16 element.
    fn is_full_cube(&mut self, name: &str) -> bool {
        let models = self.blockstate_models(name);
        if models.is_empty() {
            return false;
        }
        models.iter().all(|m| self.model_is_full_cube(m))
    }

    fn model_is_full_cube(&mut self, model_ref: &str) -> bool {
        let chain = self.parent_chain(model_ref);
        if chain.iter().any(|c| CUBE_TEMPLATES.contains(&c.as_str())) {
            return true;
        }
        // Nearest model in the chain that defines elements wins (child
        // elements override the parent's entirely).
        for m in &chain {
            let Some(model) = self.model(m) else { continue };
            let Some(elements) = model["elements"].as_array() else {
                continue;
            };
            let full = |v: &Value, expected: f64| {
                v.as_array()
                    .map(|a| a.iter().all(|c| c.as_f64() == Some(expected)))
                    .unwrap_or(false)
            };
            return elements
                .iter()
                .any(|e| full(&e["from"], 0.0) && full(&e["to"], 16.0));
        }
        false
    }

    /// The resolved (non-reference, non-particle) texture set used across
    /// all models of a block's blockstate, normalized without `minecraft:`.
    fn texture_set(&mut self, name: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for model_ref in self.blockstate_models(name) {
            // Merge texture maps along the parent chain (child wins).
            let mut merged: HashMap<String, String> = HashMap::new();
            for m in self.parent_chain(&model_ref) {
                let Some(model) = self.model(&m) else {
                    continue;
                };
                let Some(textures) = model["textures"].as_object() else {
                    continue;
                };
                for (k, v) in textures {
                    if let Some(v) = v.as_str() {
                        merged.entry(k.clone()).or_insert_with(|| v.to_string());
                    }
                }
            }
            for (k, v) in &merged {
                if k == "particle" || v.starts_with('#') {
                    continue;
                }
                out.insert(v.strip_prefix("minecraft:").unwrap_or(v).to_string());
            }
        }
        out
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
        let in_set =
            |chain: &[String], set: &[&str]| chain.iter().any(|c| set.contains(&c.as_str()));

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
    let file =
        fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut out = String::new();
    decoder.read_to_string(&mut out)?;
    Ok(out)
}

fn write_gz(path: &Path, contents: &str) -> Result<()> {
    use std::io::Write;
    let file =
        fs::File::create(path).with_context(|| format!("Failed to create {}", path.display()))?;
    let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::best());
    encoder.write_all(contents.as_bytes())?;
    encoder.finish()?;
    Ok(())
}
