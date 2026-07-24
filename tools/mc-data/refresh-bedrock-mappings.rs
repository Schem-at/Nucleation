//! Regenerates `data/blockpedia/geyser_mappings.json.gz` from GeyserMC's
//! canonical Java→Bedrock block mapping data (requires the `mc-data-refresh`
//! feature).
//!
//! GeyserMC moved its mapping data out of the old `mappings-generator` JSON
//! dumps into the `GeyserMC/mappings` repo, and switched the blockstate
//! mappings to gzipped NBT (`blocks.nbt`). The format is:
//!
//! - root compound → `bedrock_mappings`: a TAG_List of TAG_Compound with one
//!   entry per **Java blockstate, in Java runtime state-id order** (the Java
//!   side is implicit by index; Geyser zips the list against its own Java
//!   block registry, see `BlockRegistryPopulator#buildBedrockState`),
//! - each entry may have `bedrock_identifier` (TAG_String, no `minecraft:`
//!   prefix; **absent ⇒ same as the Java block's bare name**) and `state`
//!   (TAG_Compound of bedrock state overrides; absent ⇒ bedrock default
//!   state). State values are TAG_String / TAG_Int / TAG_Byte (byte = bool).
//!
//! This tool downloads `blocks.nbt`, reconstructs the Java side from the
//! in-tree `prismarinejs_blocks.json.gz` snapshot (which carries the vanilla
//! report's authoritative `minStateId`/`maxStateId` per block), and rewrites
//! `geyser_mappings.json.gz` in the same JSON schema `build.rs` already
//! consumes (`{"mappings": [{"java_state": {...}, "bedrock_state": {...}}],
//! "DataVersion": N}`), so no build.rs changes are needed.
//!
//! Java state-id reconstruction (validated against the vanilla 26.2 report,
//! 32,366/32,366 states exact): per block, properties sorted alphabetically
//! by name, cartesian product with the **last property varying fastest**,
//! bool values in vanilla order `true, false`; blocks laid out contiguously
//! from `minStateId`.
//!
//! Version-skew handling: if the upstream list length differs from the
//! snapshot's state count, the java sides can't be aligned by index alone.
//! Pass `--upstream-report <blocks.json>` (the vanilla data-generator report
//! for the *upstream* Java version) and the tool aligns by java blockstate
//! string instead. Snapshot states with no upstream mapping then get the
//! documented fallback: **identity mapping when the bare block name exists
//! in the in-tree Bedrock palette (`bedrock_block_states.json.gz`), else no
//! entry** (build.rs simply emits no key; `to_bedrock()` errors and callers
//! fall back to the raw Java name).
//!
//! Usage (repo root):
//! `cargo run --release --bin refresh-bedrock-mappings --features mc-data-refresh -- \
//!    [--source <url-or-path>] [--data-version N] [--upstream-report <blocks.json>]`
//!
//! `--data-version` is Java's world data version (`version.json` →
//! `world_version` inside the server jar; e.g. 26.2 = 4903). It is carried
//! into the output for provenance only; build.rs ignores it.

use anyhow::{bail, Context, Result};
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::Path;

const DEFAULT_SOURCE: &str =
    "https://raw.githubusercontent.com/GeyserMC/mappings/master/blocks.nbt";
const SNAPSHOT_PATH: &str = "data/blockpedia/prismarinejs_blocks.json.gz";
const BEDROCK_STATES_PATH: &str = "data/blockpedia/bedrock_block_states.json.gz";
const OUTPUT_PATH: &str = "data/blockpedia/geyser_mappings.json.gz";

/// One Java blockstate: block name (bare, no namespace) + property map, plus
/// its global runtime state id.
struct JavaState {
    bare_name: String,
    properties: Vec<(String, String)>,
}

/// The bedrock side of one mapping entry.
#[derive(Clone)]
struct BedrockState {
    identifier: String,
    /// `None` ⇒ omit the `state` key entirely (bedrock default state).
    state: Option<Map<String, Value>>,
}

fn main() -> Result<()> {
    let mut source = DEFAULT_SOURCE.to_string();
    let mut data_version: Option<u64> = None;
    let mut upstream_report: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => source = args.next().context("--source needs a value")?,
            "--data-version" => {
                data_version = Some(
                    args.next()
                        .context("--data-version needs a value")?
                        .parse()
                        .context("--data-version must be an integer")?,
                )
            }
            "--upstream-report" => {
                upstream_report = Some(args.next().context("--upstream-report needs a value")?)
            }
            other => bail!("Unknown argument: {other}"),
        }
    }

    // --- upstream mapping list (gzipped NBT) ---
    let nbt_bytes = fetch_source(&source)?;
    let upstream = parse_blocks_nbt(&nbt_bytes)?;
    println!(
        "Upstream blocks.nbt: {} bedrock_mappings entries",
        upstream.len()
    );

    // --- Java blockstates, in runtime state-id order, from the snapshot ---
    let java_states = enumerate_java_states()?;
    println!(
        "In-tree Java snapshot: {} blockstates across {} blocks",
        java_states.len(),
        java_states
            .iter()
            .map(|s| s.bare_name.as_str())
            .collect::<HashSet<_>>()
            .len()
    );

    // --- Bedrock palette names (fallback + coverage check) ---
    let palette: HashSet<String> = {
        let raw = read_gz(Path::new(BEDROCK_STATES_PATH))?;
        let states: Value = serde_json::from_str(&raw).context("bad bedrock_block_states.json")?;
        states
            .as_array()
            .context("bedrock_block_states.json is not an array")?
            .iter()
            .filter_map(|e| e["name"].as_str().map(String::from))
            .collect()
    };

    // --- align java states with upstream entries ---
    let mut fallback_identity = 0usize;
    let mut unmapped = 0usize;
    let aligned: Vec<Option<BedrockState>> = if upstream.len() == java_states.len() {
        // Same Java version on both sides: direct zip by state id.
        java_states
            .iter()
            .zip(&upstream)
            .map(|(java, entry)| Some(resolve_entry(java, entry)))
            .collect()
    } else {
        // Version skew: align by java blockstate string via the upstream
        // vanilla report, then apply the identity fallback.
        let report_path = upstream_report.with_context(|| {
            format!(
                "Upstream has {} entries but the snapshot has {} Java states — \
                 the versions differ, so index alignment is impossible. Rerun with \
                 --upstream-report <blocks.json> (vanilla data-generator report for \
                 the upstream Java version).",
                upstream.len(),
                java_states.len()
            )
        })?;
        let by_string = align_via_report(&report_path, &upstream)?;
        java_states
            .iter()
            .map(|java| {
                if let Some(bedrock) = by_string.get(&state_key(java)) {
                    Some(bedrock.clone())
                } else if palette.contains(&java.bare_name) {
                    // Documented fallback: identity mapping (default state).
                    fallback_identity += 1;
                    Some(BedrockState {
                        identifier: java.bare_name.clone(),
                        state: None,
                    })
                } else {
                    unmapped += 1;
                    None
                }
            })
            .collect()
    };

    // --- build the output JSON in the existing schema ---
    let mut mappings = Vec::with_capacity(java_states.len());
    let mut identity_ids = 0usize;
    let mut missing_in_palette: HashSet<String> = HashSet::new();
    for (java, bedrock) in java_states.iter().zip(&aligned) {
        let Some(bedrock) = bedrock else { continue };
        if bedrock.identifier == java.bare_name {
            identity_ids += 1;
        }
        let bare = bedrock
            .identifier
            .rsplit(':')
            .next()
            .unwrap_or(&bedrock.identifier);
        if !palette.contains(bare) {
            missing_in_palette.insert(bedrock.identifier.clone());
        }

        let mut java_obj = Map::new();
        java_obj.insert(
            "Name".into(),
            json!(format!("minecraft:{}", java.bare_name)),
        );
        if !java.properties.is_empty() {
            let props: Map<String, Value> = java
                .properties
                .iter()
                .map(|(k, v)| (k.clone(), json!(v)))
                .collect();
            java_obj.insert("Properties".into(), Value::Object(props));
        }

        let mut bedrock_obj = Map::new();
        bedrock_obj.insert("bedrock_identifier".into(), json!(bedrock.identifier));
        if let Some(state) = &bedrock.state {
            bedrock_obj.insert("state".into(), Value::Object(state.clone()));
        }

        mappings.push(json!({
            "java_state": Value::Object(java_obj),
            "bedrock_state": Value::Object(bedrock_obj),
        }));
    }

    // --- diff against the previous snapshot before overwriting ---
    diff_against_previous(&mappings)?;

    let mut root = Map::new();
    root.insert("mappings".into(), Value::Array(mappings.clone()));
    match data_version {
        Some(v) => {
            root.insert("DataVersion".into(), json!(v));
        }
        None => println!("Note: no --data-version given; omitting the DataVersion field."),
    }

    let pretty = serde_json::to_string_pretty(&Value::Object(root))?;
    write_gz(Path::new(OUTPUT_PATH), &pretty)?;

    println!();
    println!("Wrote {OUTPUT_PATH}: {} mappings", mappings.len());
    println!("  identity bedrock ids (same name as Java): {identity_ids}");
    println!("  identity-fallback states (skew only):     {fallback_identity}");
    println!("  unmapped states left without an entry:    {unmapped}");
    if missing_in_palette.is_empty() {
        println!(
            "  bedrock palette coverage: every bedrock_identifier exists in {BEDROCK_STATES_PATH}"
        );
    } else {
        println!(
            "  WARNING: {} bedrock ids missing from {BEDROCK_STATES_PATH}: {:?}",
            missing_in_palette.len(),
            missing_in_palette
        );
    }
    println!("Next: `cargo build` (build.rs bakes the PHF maps), then `cargo test bedrock`.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Upstream blocks.nbt
// ---------------------------------------------------------------------------

fn fetch_source(source: &str) -> Result<Vec<u8>> {
    if source.starts_with("http://") || source.starts_with("https://") {
        println!("Downloading {source}...");
        let client = reqwest::blocking::Client::builder()
            .user_agent("nucleation-bedrock-mappings-refresh")
            .timeout(std::time::Duration::from_secs(120))
            .build()?;
        let bytes = client.get(source).send()?.error_for_status()?.bytes()?;
        Ok(bytes.to_vec())
    } else {
        fs::read(source).with_context(|| format!("Failed to read {source}"))
    }
}

/// Parses blocks.nbt into the per-java-state bedrock entries, in list order.
fn parse_blocks_nbt(bytes: &[u8]) -> Result<Vec<NbtCompound>> {
    let mut cursor = std::io::Cursor::new(bytes);
    let (root, _) = quartz_nbt::io::read_nbt(&mut cursor, quartz_nbt::io::Flavor::GzCompressed)
        .context("Failed to parse blocks.nbt (expected gzipped NBT)")?;
    let list: &NbtList = root
        .get("bedrock_mappings")
        .context("blocks.nbt has no bedrock_mappings list")?;
    let mut entries = Vec::with_capacity(list.len());
    for tag in list.iter() {
        match tag {
            NbtTag::Compound(c) => entries.push(c.clone()),
            other => bail!("bedrock_mappings entry is not a compound: {other:?}"),
        }
    }
    Ok(entries)
}

/// Resolves one NBT entry against its Java state (Geyser semantics: missing
/// `bedrock_identifier` ⇒ the Java block's bare name; missing `state` ⇒
/// bedrock default state).
fn resolve_entry(java: &JavaState, entry: &NbtCompound) -> BedrockState {
    let identifier = entry
        .get::<_, &str>("bedrock_identifier")
        .map(String::from)
        .unwrap_or_else(|_| java.bare_name.clone());
    let state = entry.get::<_, &NbtCompound>("state").ok().map(|c| {
        c.inner()
            .iter()
            .map(|(k, v)| (k.clone(), nbt_state_value_to_json(v)))
            .collect()
    });
    BedrockState { identifier, state }
}

/// Bedrock state values in blocks.nbt are String / Int / Byte; the JSON
/// schema (from the old mappings-generator dumps) represents bytes as bools.
fn nbt_state_value_to_json(tag: &NbtTag) -> Value {
    match tag {
        NbtTag::Byte(b) if *b == 0 || *b == 1 => json!(*b == 1),
        NbtTag::Byte(b) => json!(b),
        NbtTag::Short(v) => json!(v),
        NbtTag::Int(v) => json!(v),
        NbtTag::Long(v) => json!(v),
        NbtTag::String(s) => json!(s),
        other => json!(other.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Java blockstate enumeration (snapshot-driven)
// ---------------------------------------------------------------------------

/// All Java blockstates in runtime state-id order, reconstructed from the
/// PrismarineJS-schema snapshot (see the module docs for the ordering rules).
fn enumerate_java_states() -> Result<Vec<JavaState>> {
    let raw = read_gz(Path::new(SNAPSHOT_PATH))?;
    let blocks: Vec<Value> = serde_json::from_str(&raw).context("bad prismarinejs snapshot")?;

    let mut ordered: Vec<&Value> = blocks.iter().collect();
    ordered.sort_by_key(|b| b["minStateId"].as_u64().unwrap_or(u64::MAX));

    let mut out = Vec::new();
    let mut expected_id = 0u64;
    for block in ordered {
        let name = block["name"].as_str().context("block missing name")?;
        let min_id = block["minStateId"]
            .as_u64()
            .context("block missing minStateId")?;
        if min_id != expected_id {
            bail!("state-id gap before minecraft:{name}: expected {expected_id}, got {min_id}");
        }

        // Properties sorted alphabetically (matches vanilla StateDefinition).
        let mut props: Vec<(String, Vec<String>)> = Vec::new();
        if let Some(states) = block["states"].as_array() {
            for s in states {
                let pname = s["name"]
                    .as_str()
                    .context("state missing name")?
                    .to_string();
                let values = property_values(s)
                    .with_context(|| format!("minecraft:{name}: bad property {pname}"))?;
                props.push((pname, values));
            }
        }
        props.sort_by(|a, b| a.0.cmp(&b.0));

        // Cartesian product, last property varying fastest.
        let total: usize = props.iter().map(|(_, v)| v.len()).product::<usize>().max(1);
        for i in 0..total {
            let mut rem = i;
            let mut assignment = vec![String::new(); props.len()];
            for (slot, (_, values)) in props.iter().enumerate().rev() {
                assignment[slot] = values[rem % values.len()].clone();
                rem /= values.len();
            }
            out.push(JavaState {
                bare_name: name.to_string(),
                properties: props
                    .iter()
                    .map(|(n, _)| n.clone())
                    .zip(assignment)
                    .collect(),
            });
        }
        expected_id += total as u64;
    }
    Ok(out)
}

/// Value list for one snapshot property, in vanilla state-id order:
/// bools are `true, false`; ints/enums use the snapshot's `values` array
/// (vanilla ordinal order), bare ints fall back to `0..num_values`.
fn property_values(state: &Value) -> Result<Vec<String>> {
    match state["type"].as_str() {
        Some("bool") => Ok(vec!["true".into(), "false".into()]),
        Some("int") | Some("enum") => {
            if let Some(values) = state["values"].as_array() {
                Ok(values
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        other => other.to_string(),
                    })
                    .collect())
            } else if state["type"].as_str() == Some("int") {
                let n = state["num_values"]
                    .as_u64()
                    .context("int property without num_values")?;
                Ok((0..n).map(|i| i.to_string()).collect())
            } else {
                bail!("enum property without values array")
            }
        }
        other => bail!("unknown property type {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Version-skew alignment + diffing
// ---------------------------------------------------------------------------

/// Canonical "name[k=v,...]" key for a Java state (sorted properties).
fn state_key(java: &JavaState) -> String {
    let mut props: Vec<String> = java
        .properties
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    props.sort();
    format!("minecraft:{}[{}]", java.bare_name, props.join(","))
}

/// Builds java-state-string → bedrock mapping by zipping the upstream vanilla
/// report's states (which carry explicit ids) with the NBT entries.
fn align_via_report(
    report_path: &str,
    upstream: &[NbtCompound],
) -> Result<HashMap<String, BedrockState>> {
    let report: Value = serde_json::from_str(
        &fs::read_to_string(report_path)
            .with_context(|| format!("Failed to read {report_path}"))?,
    )?;
    let report = report.as_object().context("report is not an object")?;

    // state id -> (bare name, sorted property string)
    let mut by_id: HashMap<u64, (String, JavaState)> = HashMap::new();
    for (id, entry) in report {
        let bare = id.strip_prefix("minecraft:").unwrap_or(id);
        for s in entry["states"]
            .as_array()
            .context("report entry without states")?
        {
            let sid = s["id"].as_u64().context("report state without id")?;
            let properties = s["properties"]
                .as_object()
                .map(|p| {
                    p.iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let js = JavaState {
                bare_name: bare.to_string(),
                properties,
            };
            by_id.insert(sid, (state_key(&js), js));
        }
    }
    if by_id.len() != upstream.len() {
        bail!(
            "Upstream report has {} states but blocks.nbt has {} entries — \
             the report is for the wrong Java version.",
            by_id.len(),
            upstream.len()
        );
    }
    let mut out = HashMap::with_capacity(by_id.len());
    for (sid, (key, js)) in by_id {
        let entry = &upstream[sid as usize];
        out.insert(key, resolve_entry(&js, entry));
    }
    Ok(out)
}

/// Prints a carried-over/changed/gained/lost summary against the previous
/// geyser_mappings snapshot (best-effort; a missing old file is fine).
fn diff_against_previous(new_mappings: &[Value]) -> Result<()> {
    let Ok(old_raw) = read_gz(Path::new(OUTPUT_PATH)) else {
        println!("No previous {OUTPUT_PATH} to diff against.");
        return Ok(());
    };
    let old: Value = serde_json::from_str(&old_raw)?;
    let canonical = |m: &Value| -> Option<(String, String)> {
        let js = m.get("java_state")?;
        let mut jprops: Vec<String> = js
            .get("Properties")
            .and_then(|p| p.as_object())
            .map(|p| {
                p.iter()
                    .map(|(k, v)| format!("{k}={}", v.as_str().unwrap_or_default()))
                    .collect()
            })
            .unwrap_or_default();
        jprops.sort();
        let jkey = format!("{}[{}]", js.get("Name")?.as_str()?, jprops.join(","));
        let bs = m.get("bedrock_state")?;
        let mut bprops: Vec<String> = bs
            .get("state")
            .and_then(|p| p.as_object())
            .map(|p| {
                p.iter()
                    .map(|(k, v)| {
                        let vs = match v {
                            Value::Bool(b) => b.to_string(),
                            Value::Number(n) => n.to_string(),
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        format!("{k}={vs}")
                    })
                    .collect()
            })
            .unwrap_or_default();
        bprops.sort();
        let bkey = format!(
            "{}[{}]",
            bs.get("bedrock_identifier")?.as_str()?,
            bprops.join(",")
        );
        Some((jkey, bkey))
    };

    let old_map: HashMap<String, String> = old["mappings"]
        .as_array()
        .map(|a| a.iter().filter_map(canonical).collect())
        .unwrap_or_default();
    let new_map: HashMap<String, String> = new_mappings.iter().filter_map(canonical).collect();

    let mut identical = 0usize;
    let mut changed = 0usize;
    let mut changed_samples: Vec<&String> = Vec::new();
    for (k, v) in &new_map {
        match old_map.get(k) {
            Some(old_v) if old_v == v => identical += 1,
            Some(_) => {
                changed += 1;
                if changed_samples.len() < 10 {
                    changed_samples.push(k);
                }
            }
            None => {}
        }
    }
    let gained = new_map.keys().filter(|k| !old_map.contains_key(*k)).count();
    let lost = old_map.keys().filter(|k| !new_map.contains_key(*k)).count();
    println!(
        "Diff vs previous snapshot: {} carried over identical, {} changed, {} gained, {} lost",
        identical, changed, gained, lost
    );
    for k in changed_samples {
        println!("  changed: {k}: {} -> {}", old_map[k], new_map[k]);
    }
    Ok(())
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
