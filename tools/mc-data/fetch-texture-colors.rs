//! End-to-end block color pipeline for the vendored blockpedia module
//! (requires the `mc-data-refresh` feature).
//!
//! 1. Fetches Mojang's version manifest and resolves the target version
//!    (first CLI arg; defaults to the manifest's latest release — keep it in
//!    sync with the Java block-data pin in `data/blockpedia/`, i.e. run
//!    `refresh-block-data` first).
//! 2. Downloads the client jar and extracts
//!    `assets/minecraft/textures/block/*.png` into `target/mc-data/textures/`.
//! 3. Maps every known block (from the crate's own PHF table, i.e. the
//!    committed prismarine data) to a representative texture via
//!    `nucleation::blockpedia::color::texture_mapping::resolve_texture`.
//! 4. Computes an alpha-weighted average color per block and applies the
//!    default plains-biome tints for grayscale-tinted blocks (grass, leaves,
//!    water, vines, stems, ...; see `texture_mapping` for the constants).
//! 5. Regenerates `data/blockpedia/color_cache.json.gz`, which `build.rs`
//!    bakes into the generated block table on the next build.
//!
//! Run from the repo root:
//! `cargo run --release --bin fetch-texture-colors --features mc-data-refresh [-- <version>]`

use anyhow::{Context, Result};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::Path;

use nucleation::blockpedia::color::extraction::{ColorExtractor, ExtractionMethod};
use nucleation::blockpedia::color::texture_mapping::{default_biome_tint, resolve_texture};

const VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

fn main() -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("nucleation-color-pipeline")
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    // 1. Resolve the target version (first CLI arg or the manifest's latest
    // release). Must match the Java block-data pin in data/blockpedia/.
    println!("Fetching version manifest...");
    let manifest: serde_json::Value = client
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

    let work_dir = Path::new("target/mc-data");
    let textures_dir = work_dir.join("textures");
    let jar_path = work_dir.join(format!("client-{version}.jar"));

    // 2. Download the client jar (cached) and extract block textures
    if !jar_path.exists() {
        let version_url = manifest["versions"]
            .as_array()
            .context("manifest has no versions array")?
            .iter()
            .find(|v| v["id"].as_str() == Some(&version))
            .and_then(|v| v["url"].as_str())
            .with_context(|| format!("Version {version} not found in manifest"))?
            .to_string();

        println!("Fetching version metadata...");
        let version_meta: serde_json::Value = client
            .get(&version_url)
            .send()?
            .error_for_status()?
            .json()
            .context("Failed to parse version metadata")?;

        let client_url = version_meta["downloads"]["client"]["url"]
            .as_str()
            .context("No client jar download in version metadata")?;

        println!("Downloading client jar from {client_url}...");
        let jar_bytes = client
            .get(client_url)
            .send()?
            .error_for_status()?
            .bytes()
            .context("Failed to download client jar")?;

        fs::create_dir_all(work_dir)?;
        fs::write(&jar_path, &jar_bytes)?;
        println!(
            "Saved {} ({:.1} MB)",
            jar_path.display(),
            jar_bytes.len() as f64 / 1e6
        );
    } else {
        println!("Using cached client jar {}", jar_path.display());
    }

    println!("Extracting block textures...");
    fs::create_dir_all(&textures_dir)?;
    let jar_file = fs::File::open(&jar_path)?;
    let mut archive = zip::ZipArchive::new(jar_file).context("Client jar is not a valid zip")?;

    let mut extracted = 0usize;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();
        if let Some(file_name) = entry_name.strip_prefix("assets/minecraft/textures/block/") {
            if file_name.ends_with(".png") && !file_name.contains('/') {
                let mut buf = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut buf)?;
                fs::write(textures_dir.join(file_name), &buf)?;
                extracted += 1;
            }
        }
    }
    println!(
        "Extracted {extracted} block textures to {}",
        textures_dir.display()
    );

    // 3+4. Resolve a texture per block and extract colors
    let available: HashSet<String> = fs::read_dir(&textures_dir)?
        .filter_map(|e| {
            let path = e.ok()?.path();
            if path.extension()? == "png" {
                path.file_stem()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect();

    let extractor = ColorExtractor::new(ExtractionMethod::AlphaWeighted);
    let mut cache: BTreeMap<String, (u8, u8, u8, f32, f32, f32)> = BTreeMap::new();
    let mut unresolved: Vec<&str> = Vec::new();
    let mut failed: Vec<(&str, String)> = Vec::new();

    let total_blocks = nucleation::blockpedia::all_blocks().count();
    for block in nucleation::blockpedia::all_blocks() {
        let name = block.id().strip_prefix("minecraft:").unwrap_or(block.id());
        let Some(texture) = resolve_texture(name, &available) else {
            unresolved.push(block.id());
            continue;
        };

        let texture_path = textures_dir.join(format!("{texture}.png"));
        let img = match image::open(&texture_path) {
            Ok(img) => img,
            Err(e) => {
                failed.push((block.id(), e.to_string()));
                continue;
            }
        };
        let color = match extractor.extract_color(&img) {
            Ok(c) => c,
            Err(e) => {
                failed.push((block.id(), e.to_string()));
                continue;
            }
        };

        let [mut r, mut g, mut b] = color.rgb;
        if let Some(tint) = default_biome_tint(name) {
            r = ((r as u16 * tint[0] as u16) / 255) as u8;
            g = ((g as u16 * tint[1] as u16) / 255) as u8;
            b = ((b as u16 * tint[2] as u16) / 255) as u8;
        }

        let (l, a, b_val) = rgb_to_simple_oklab(r, g, b);
        cache.insert(block.id().to_string(), (r, g, b, l, a, b_val));
    }

    // 5. Write the cache (gzipped, as build.rs consumes it)
    let cache_path = Path::new("data/blockpedia/color_cache.json.gz");
    write_gz(cache_path, &serde_json::to_string_pretty(&cache)?)?;

    println!();
    println!("Color cache written to {}", cache_path.display());
    println!(
        "Coverage: {}/{} blocks ({:.1}%)",
        cache.len(),
        total_blocks,
        cache.len() as f64 / total_blocks as f64 * 100.0
    );
    if !failed.is_empty() {
        println!("Extraction failures ({}):", failed.len());
        for (id, e) in &failed {
            println!("  {id}: {e}");
        }
    }
    if !unresolved.is_empty() {
        println!("Blocks without a resolved texture ({}):", unresolved.len());
        for id in &unresolved {
            println!("  {id}");
        }
    }
    println!("Rebuild the crate to bake the new colors in (cargo build).");

    Ok(())
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

/// Same simplified RGB -> Oklab used by build.rs when generating the table;
/// the cache stores both so the two never disagree.
fn rgb_to_simple_oklab(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let a = (r - g) * 0.5;
    let b_val = (r + g - 2.0 * b) * 0.25;
    (l, a, b_val)
}
