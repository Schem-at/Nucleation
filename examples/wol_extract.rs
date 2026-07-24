//! M10 validation driver for the `world_segment` pipeline ("WOL world
//! segmentation" — ORE "World Of Logic"): runs the real pipeline against a
//! real Minecraft world tarball, using ORE plot rectangles as partition
//! hints, and reports aggregate stats so a human can sanity-check the run
//! against the pre-`world_segment` evidence baseline (a single
//! region-spanning 574,902,785-block substrate blob, 283k "builds", only 204
//! cross-tile stitches).
//!
//! This is a standalone example (the caller), so — unlike `src/world_segment/`
//! itself — it is allowed to name the platform it's built for (ORE, plots,
//! dynmap).
//!
//! Usage:
//!   cargo run --release --features world-segment --example wol_extract -- \
//!     [--tarball PATH] [--plots PATH] [--out DIR] [--limit N] [--sample N]
//!
//! Do NOT run this against the full 1.6 GB tarball without `--release`.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use nucleation::formats::schematic::to_schematic;
use nucleation::world_segment::partition::{PartitionHint, PartitionIndex, PartitionPolicy};
use nucleation::world_segment::profile::{ProfileParams, WorldProfile};
use nucleation::world_segment::runner::{MaterializedBuild, RunStats, SegmentJob, WorldSegmenter};
use nucleation::world_segment::score::{ScoreConfig, Tier};
use nucleation::world_segment::segment::SegConfig;
use nucleation::world_segment::source::{TileError, TileSource};
use nucleation::world_segment::targz_source::TarGzSource;
use nucleation::world_segment::tile::VoxelTile;

/// Old (pre-`world_segment`) evidence baseline this run is compared against.
const OLD_LARGEST_BLOCK_COUNT: u64 = 574_902_785;
const OLD_BUILDS: u64 = 283_000;
const OLD_CROSS_TILE: u64 = 204;

struct Cli {
    tarball: PathBuf,
    plots: PathBuf,
    out: PathBuf,
    limit: Option<usize>,
    sample: usize,
}

fn parse_args() -> Cli {
    let mut tarball =
        PathBuf::from("experiments/build-extractor/data/build_20260519080014.tar.gz");
    let mut plots = PathBuf::from("wol-project/data-ore-plots-build-20260723.json");
    let mut out = PathBuf::from("wol-project/m10-out");
    let mut limit: Option<usize> = None;
    let mut sample: usize = 30;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--tarball" => {
                tarball = PathBuf::from(args.get(i + 1).expect("--tarball needs a value"));
                i += 2;
            }
            "--plots" => {
                plots = PathBuf::from(args.get(i + 1).expect("--plots needs a value"));
                i += 2;
            }
            "--out" => {
                out = PathBuf::from(args.get(i + 1).expect("--out needs a value"));
                i += 2;
            }
            "--limit" => {
                limit = Some(
                    args.get(i + 1)
                        .expect("--limit needs a value")
                        .parse()
                        .expect("--limit must be a number"),
                );
                i += 2;
            }
            "--sample" => {
                sample = args
                    .get(i + 1)
                    .expect("--sample needs a value")
                    .parse()
                    .expect("--sample must be a number");
                i += 2;
            }
            other => {
                eprintln!("wol_extract: ignoring unrecognized argument {other}");
                i += 1;
            }
        }
    }

    Cli { tarball, plots, out, limit, sample }
}

/// One row of `wol-project/data-ore-plots-build-20260723.json`.
#[derive(serde::Deserialize)]
struct PlotRow {
    id: String,
    x0: i32,
    x1: i32,
    z0: i32,
    z1: i32,
}

fn load_plot_hints(path: &std::path::Path) -> Vec<PartitionHint> {
    let data = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read plots file {}: {e}", path.display()));
    let rows: Vec<PlotRow> = serde_json::from_str(&data)
        .unwrap_or_else(|e| panic!("failed to parse plots JSON {}: {e}", path.display()));
    rows.into_iter()
        .map(|r| PartitionHint { id: r.id, bbox_xz: (r.x0, r.x1, r.z0, r.z1), y_range: None })
        .collect()
}

/// Wraps a `TarGzSource` and stops invoking the inner callback after `limit`
/// tiles have been seen, without aborting the underlying archive walk with an
/// error (a real error would surface at the caller as a hard failure — we
/// just want to stop *processing* early). `TarGzSource` is forward-only and
/// streams the whole tar regardless, so this only bounds how much work the
/// pipeline itself does downstream of the callback, not how much of the
/// tarball gets decompressed.
struct LimitedSource<'a> {
    inner: &'a TarGzSource,
    limit: usize,
}

impl<'a> TileSource for LimitedSource<'a> {
    fn access(&self) -> nucleation::world_segment::source::Access {
        self.inner.access()
    }

    fn tile_ids(&self) -> Result<Vec<nucleation::world_segment::ids::TileId>, TileError> {
        self.inner.tile_ids()
    }

    fn tile(
        &self,
        id: nucleation::world_segment::ids::TileId,
    ) -> Result<Option<VoxelTile>, TileError> {
        self.inner.tile(id)
    }

    fn for_each_tile(
        &self,
        f: &mut dyn FnMut(VoxelTile) -> Result<(), TileError>,
    ) -> Result<(), TileError> {
        let mut seen = 0usize;
        let limit = self.limit;
        self.inner.for_each_tile(&mut |tile| {
            if seen >= limit {
                return Ok(());
            }
            seen += 1;
            f(tile)
        })
    }
}

fn main() {
    let cli = parse_args();

    // 1. Load plots -> partition hints.
    let hints = load_plot_hints(&cli.plots);
    println!("wol_extract: loaded {} plot partition hints from {}", hints.len(), cli.plots.display());

    // 2. Derive the world profile from a fresh, sample-limited pass over the
    //    tarball. `TarGzSource` is forward-only, so this source is consumed
    //    entirely by sampling and CANNOT be reused for the main run below.
    let sample_cap = match cli.limit {
        Some(l) => cli.sample.min(l),
        None => cli.sample,
    };
    println!(
        "wol_extract: deriving world profile from up to {sample_cap} sample tiles ({})",
        cli.tarball.display()
    );
    let profile_source = TarGzSource::open(&cli.tarball, -64, 320)
        .unwrap_or_else(|e| panic!("failed to open tarball {} for sampling: {e}", cli.tarball.display()));
    let mut samples: Vec<VoxelTile> = Vec::new();
    profile_source
        .for_each_tile(&mut |tile| {
            if samples.len() >= sample_cap {
                return Ok(());
            }
            samples.push(tile);
            Ok(())
        })
        .expect("sampling pass over tarball failed");
    println!("wol_extract: collected {} sample tiles for profile derivation", samples.len());

    let profile = WorldProfile::derive(&samples, &ProfileParams::default());
    let palette_preview: Vec<&String> = profile.substrate_palette.iter().take(10).collect();
    println!(
        "wol_extract: derived profile — substrate_y_band = {:?}, palette ({} names, first 10) = {:?}",
        profile.substrate_y_band,
        profile.substrate_palette.len(),
        palette_preview
    );
    drop(samples);

    // 3. Build the segmentation job.
    let job = SegmentJob {
        config: SegConfig {
            cell_size: 4,
            closing_radius: 2,
            partition_policy: PartitionPolicy::HardCut,
            ..SegConfig::default()
        },
        score_config: ScoreConfig::default(),
        source_id: "ore-build".to_string(),
        snapshot_id: "build_20260519080014".to_string(),
        min_y: -64,
        max_y: 320,
        extracted_at: 1_747_800_000,
        match_iou: 0.5,
    };
    let partitions = PartitionIndex::new(hints);

    // 4. Open a FRESH TarGzSource for the run (the sampling one above is
    //    fully consumed and cannot be rewound).
    println!("wol_extract: opening fresh source for the main run");
    let run_source = TarGzSource::open(&cli.tarball, job.min_y, job.max_y)
        .unwrap_or_else(|e| panic!("failed to open tarball {} for the run: {e}", cli.tarball.display()));

    std::fs::create_dir_all(&cli.out)
        .unwrap_or_else(|e| panic!("failed to create output dir {}: {e}", cli.out.display()));
    let provenance_path = cli.out.join("provenance.jsonl");
    let mut provenance_file = File::create(&provenance_path)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", provenance_path.display()));

    let mut build_count: u64 = 0;
    let mut schem_written: u64 = 0;
    let mut seen_stable_ids: BTreeSet<String> = BTreeSet::new();

    let mut emit = |mb: MaterializedBuild| {
        build_count += 1;
        let MaterializedBuild { schematic, provenance } = mb;

        let line = serde_json::to_string(&provenance).expect("provenance must serialize to JSON");
        writeln!(provenance_file, "{line}").expect("failed to write provenance line");

        let stable_id = provenance.stable_build_id.to_string();
        seen_stable_ids.insert(stable_id.clone());

        // Only write non-Debris builds to .schem, to keep the smoke output
        // small; the schematic writer (`to_schematic`) is directly callable,
        // so this is not a TODO-skip.
        if provenance.tier != Tier::Debris {
            match to_schematic(&schematic) {
                Ok(bytes) => {
                    let schem_path = cli.out.join(format!("{stable_id}.schem"));
                    if let Err(e) = std::fs::write(&schem_path, &bytes) {
                        eprintln!("wol_extract: failed to write {}: {e}", schem_path.display());
                    } else {
                        schem_written += 1;
                    }
                }
                Err(e) => {
                    eprintln!("wol_extract: failed to serialize build {stable_id} to .schem: {e}");
                }
            }
        }
    };

    println!("wol_extract: running WorldSegmenter::run_streaming...");
    let stats: RunStats = if let Some(limit) = cli.limit {
        println!("wol_extract: --limit {limit} set, wrapping source in LimitedSource");
        let limited = LimitedSource { inner: &run_source, limit };
        WorldSegmenter::run_streaming(&limited, &profile, &partitions, &job, &[], &mut emit)
    } else {
        WorldSegmenter::run_streaming(&run_source, &profile, &partitions, &job, &[], &mut emit)
    };

    println!();
    println!("wol_extract: run complete.");
    println!("wol_extract: RunStats = {stats:?}");
    println!(
        "wol_extract: emitted {build_count} builds ({} distinct stable ids), wrote {schem_written} .schem files",
        seen_stable_ids.len()
    );
    println!("wol_extract: provenance written to {}", provenance_path.display());

    println!();
    println!("wol_extract: --- comparison vs pre-world_segment evidence baseline ---");
    println!(
        "wol_extract: builds total          = {} (old baseline: ~{OLD_BUILDS} ; expect low thousands, not hundreds of thousands)",
        stats.builds
    );
    println!(
        "wol_extract: tier breakdown         = confident={} probable={} debris={}",
        stats.tier_confident, stats.tier_probable, stats.tier_debris
    );
    println!(
        "wol_extract: cross_tile count       = {} (old baseline: {OLD_CROSS_TILE} ; expect well above that)",
        stats.cross_tile
    );
    println!(
        "wol_extract: largest_block_count    = {} (old baseline: {OLD_LARGEST_BLOCK_COUNT} ; expect FAR below that unless legitimately huge)",
        stats.largest_block_count
    );
    let blob_absent = stats.largest_block_count < OLD_LARGEST_BLOCK_COUNT / 10;
    println!(
        "wol_extract: region-spanning substrate blob ABSENT? {} (largest_block_count < 10% of old baseline)",
        if blob_absent { "YES" } else { "NO -- investigate" }
    );
}
