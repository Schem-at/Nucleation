//! Downloads legacy block textures from the hueblocks repo into
//! `target/mc-data/textures/` (requires the `mc-data-refresh` feature).
//!
//! This is the fallback texture source; the primary pipeline is
//! `fetch-texture-colors`, which extracts textures straight from the
//! official client jar. Ported from blockpedia's download-textures bin,
//! converted from async tokio to blocking reqwest.
//!
//! Run from the repo root:
//! `cargo run --release --bin download-textures --features mc-data-refresh`

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    println!("Downloading Minecraft block textures from hueblocks...");

    let textures_dir = Path::new("target/mc-data/textures");
    fs::create_dir_all(textures_dir).context("Failed to create textures directory")?;

    let client = reqwest::blocking::Client::new();
    let url =
        "https://api.github.com/repos/1280px/hueblocks/contents/data/blocksets/blocks?ref=legacy";

    println!("Fetching texture list from GitHub API...");
    let response = client
        .get(url)
        .header("User-Agent", "nucleation-texture-downloader")
        .send()
        .context("Failed to fetch texture list")?;

    let files: Vec<Value> = response
        .json()
        .context("Failed to parse GitHub API response")?;

    println!("Found {} texture files", files.len());

    let mut downloaded = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for file in files {
        if let (Some(name), Some(download_url)) =
            (file["name"].as_str(), file["download_url"].as_str())
        {
            if !name.ends_with(".png") {
                continue;
            }

            let local_path = textures_dir.join(name);
            if local_path.exists() {
                skipped += 1;
                continue;
            }

            print!("Downloading {}... ", name);
            match download_texture(&client, download_url, &local_path) {
                Ok(_) => {
                    println!("ok");
                    downloaded += 1;
                }
                Err(e) => {
                    println!("failed: {}", e);
                    failed += 1;
                }
            }

            // Be nice to GitHub's API
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    println!();
    println!("Download summary:");
    println!("  Downloaded: {}", downloaded);
    println!("  Skipped (already exists): {}", skipped);
    println!("  Failed: {}", failed);
    println!("  Total files: {}", downloaded + skipped + failed);

    if failed > 0 {
        println!("Some downloads failed. Re-run this command to retry.");
    } else {
        println!("All textures downloaded.");
        println!();
        println!("Testing color extraction on a few textures...");
        test_color_extraction(textures_dir)?;
    }

    Ok(())
}

fn download_texture(
    client: &reqwest::blocking::Client,
    url: &str,
    local_path: &Path,
) -> Result<()> {
    let response = client.get(url).send().context("Failed to download texture")?;
    let bytes = response.bytes().context("Failed to read texture bytes")?;
    fs::write(local_path, bytes).context("Failed to write texture file")?;
    Ok(())
}

fn test_color_extraction(textures_dir: &Path) -> Result<()> {
    use nucleation::blockpedia::color::extract_dominant_color;

    let test_blocks = [
        "stone.png",
        "dirt.png",
        "grass_block_top.png",
        "oak_log.png",
    ];

    for block_name in test_blocks {
        let texture_path = textures_dir.join(block_name);
        if texture_path.exists() {
            match extract_dominant_color(&texture_path) {
                Ok(color) => {
                    println!(
                        "  {} -> RGB({}, {}, {}) | {} | HSL({:.0} deg, {:.1}%, {:.1}%)",
                        block_name,
                        color.rgb[0],
                        color.rgb[1],
                        color.rgb[2],
                        color.hex_string(),
                        color.hsl[0],
                        color.hsl[1] * 100.0,
                        color.hsl[2] * 100.0
                    );
                }
                Err(e) => {
                    println!("  {} -> Error: {}", block_name, e);
                }
            }
        }
    }

    Ok(())
}
