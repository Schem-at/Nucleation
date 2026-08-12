//! Generic bounded world-to-schematics driver.
//!
//! Accepts an Anvil world directory or an uncompressed, gzip, or Zstandard tar
//! archive and an inclusive world-space rectangle. It derives a substrate profile from that selection,
//! segments it, and writes each non-debris build as Sponge `.schem` plus a
//! deterministic `provenance.jsonl` manifest.
//!
//! Usage:
//!   cargo run --release --features world-segment --example segment_world -- \
//!     <world.tar[.gz|.zst]> <out-dir> <min-x> <min-z> <max-x> <max-z>

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use nucleation::formats::schematic::to_schematic;
use nucleation::formats::world_stream::WorldSource;
use nucleation::world_segment::segment::DisconnectedSplit;
use nucleation::world_segment::{
    ContentId, MaterializedBuild, PartitionHint, PartitionIndex, PartitionPolicy, ProfileParams,
    RunStats, ScoreConfig, SegConfig, SegmentJob, StableBuildId, TarArchiveSource, Tier,
    TileSource, VoxelTile, WorldProfile, WorldSegmenter,
};
use nucleation::{Connectivity, ProvenanceBounds, SchematicProvenance};

struct Cli {
    input: String,
    world_prefix: Option<String>,
    output: String,
    rect: (i32, i32, i32, i32),
    source_id: String,
    world_name: String,
    map_name: String,
    dimension: String,
    snapshot_id: String,
    extracted_at: i64,
    substrate: Option<BTreeSet<String>>,
    substrate_band: Option<(i32, i32)>,
    grid: Option<Grid>,
    grid_index_bounds: Option<(i32, i32, i32, i32)>,
    partition_hints: Option<PathBuf>,
    drop_unpartitioned: Option<bool>,
    partition_floor_share: f32,
    partition_dense_layer_coverage: f32,
    split_min_blocks: u64,
    component_attach_mode: ComponentAttachMode,
    component_join_gap: u32,
    component_min_blocks: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComponentAttachMode {
    Exact,
    Nearby,
    Nearest,
}

#[derive(Clone, Copy)]
struct Grid {
    pitch: i32,
    size: i32,
    offset_x: i32,
    offset_z: i32,
}

fn parse_args() -> Cli {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 6 {
        eprintln!(
            "usage: segment_world <world-dir|world.tar[.gz|.zst]> <out-dir> <min-x> <min-z> <max-x> <max-z> \
             [--source-id ID] [--world-name NAME] [--map-name NAME] \
             [--dimension ID] [--snapshot-id ID] [--extracted-at UNIX_SECONDS] \
             [--substrate BLOCK[,BLOCK...]] [--substrate-band MIN,MAX] \
             [--grid-pitch BLOCKS] [--grid-size BLOCKS] \
             [--grid-offset-x X] [--grid-offset-z Z] \
             [--grid-index-bounds MIN_X,MIN_Z,MAX_X,MAX_Z] \
             [--partition-hints FILE.json] [--drop-unpartitioned true|false] \
             [--partition-floor-share FRACTION] \
             [--partition-dense-layer-coverage FRACTION] \
             [--split-min-blocks COUNT] \
             [--component-attach-mode exact|nearby|nearest] \
             [--component-join-gap BLOCKS] \
             [--component-min-blocks COUNT] \
             [--world-prefix STORE_KEY_TO_REGION_DIR]"
        );
        std::process::exit(2);
    }
    let number = |index: usize| {
        args[index]
            .parse::<i32>()
            .unwrap_or_else(|_| panic!("{} must be an integer", args[index]))
    };
    let (x0, z0, x1, z1) = (number(2), number(3), number(4), number(5));
    let input = args[0].clone();
    let input_path = PathBuf::from(&input);
    let filename = input_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("world")
        .to_string();
    let map_name = filename
        .strip_suffix(".tar.zst")
        .or_else(|| filename.strip_suffix(".tar.gz"))
        .or_else(|| filename.strip_suffix(".tar"))
        .unwrap_or(&filename)
        .to_string();
    let mut cli = Cli {
        input,
        world_prefix: None,
        output: args[1].clone(),
        rect: (x0.min(x1), z0.min(z1), x0.max(x1), z0.max(z1)),
        source_id: format!("map:{map_name}"),
        world_name: map_name.clone(),
        map_name,
        dimension: "minecraft:overworld".into(),
        snapshot_id: filename.clone(),
        extracted_at: 0,
        substrate: None,
        substrate_band: None,
        grid: None,
        grid_index_bounds: None,
        partition_hints: None,
        drop_unpartitioned: None,
        partition_floor_share: 0.30,
        partition_dense_layer_coverage: 0.80,
        split_min_blocks: 256,
        component_attach_mode: ComponentAttachMode::Nearby,
        component_join_gap: 3,
        component_min_blocks: 16,
    };
    let mut grid_pitch = None;
    let mut grid_size = None;
    let mut grid_offset_x = None;
    let mut grid_offset_z = None;
    let mut index = 6;
    while index < args.len() {
        let value = args
            .get(index + 1)
            .unwrap_or_else(|| panic!("{} requires a value", args[index]));
        match args[index].as_str() {
            "--source-id" => cli.source_id = value.clone(),
            "--world-name" => cli.world_name = value.clone(),
            "--map-name" => cli.map_name = value.clone(),
            "--dimension" => cli.dimension = value.clone(),
            "--snapshot-id" => cli.snapshot_id = value.clone(),
            "--extracted-at" => {
                cli.extracted_at = value.parse().expect("--extracted-at must be an integer")
            }
            "--substrate" => {
                let names = value
                    .split(',')
                    .filter(|name| !name.is_empty())
                    .map(str::to_string)
                    .collect::<BTreeSet<_>>();
                assert!(
                    !names.is_empty(),
                    "--substrate must name at least one block"
                );
                cli.substrate = Some(names);
            }
            "--substrate-band" => {
                let (lo, hi) = value
                    .split_once(',')
                    .unwrap_or_else(|| panic!("--substrate-band must be MIN,MAX"));
                let lo = lo
                    .parse::<i32>()
                    .expect("substrate minimum must be an integer");
                let hi = hi
                    .parse::<i32>()
                    .expect("substrate maximum must be an integer");
                cli.substrate_band = Some((lo.min(hi), lo.max(hi)));
            }
            "--grid-pitch" => {
                grid_pitch = Some(value.parse::<i32>().expect("grid pitch must be an integer"))
            }
            "--grid-size" => {
                grid_size = Some(value.parse::<i32>().expect("grid size must be an integer"))
            }
            "--grid-offset-x" => {
                grid_offset_x = Some(
                    value
                        .parse::<i32>()
                        .expect("grid X offset must be an integer"),
                )
            }
            "--grid-offset-z" => {
                grid_offset_z = Some(
                    value
                        .parse::<i32>()
                        .expect("grid Z offset must be an integer"),
                )
            }
            "--grid-index-bounds" => {
                let values = value
                    .split(',')
                    .map(|part| {
                        part.parse::<i32>()
                            .expect("grid index bounds must be integers")
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    values.len(),
                    4,
                    "--grid-index-bounds must have four comma-separated integers"
                );
                cli.grid_index_bounds = Some((
                    values[0].min(values[2]),
                    values[1].min(values[3]),
                    values[0].max(values[2]),
                    values[1].max(values[3]),
                ));
            }
            "--partition-hints" => cli.partition_hints = Some(PathBuf::from(value)),
            "--drop-unpartitioned" => {
                cli.drop_unpartitioned = Some(
                    value
                        .parse::<bool>()
                        .expect("--drop-unpartitioned must be true or false"),
                )
            }
            "--partition-floor-share" => {
                cli.partition_floor_share = value
                    .parse::<f32>()
                    .expect("partition floor share must be a number")
            }
            "--partition-dense-layer-coverage" => {
                cli.partition_dense_layer_coverage = value
                    .parse::<f32>()
                    .expect("partition dense-layer coverage must be a number")
            }
            "--split-min-blocks" => {
                cli.split_min_blocks = value
                    .parse::<u64>()
                    .expect("split minimum blocks must be a non-negative integer")
            }
            "--component-attach-mode" => {
                cli.component_attach_mode = match value.as_str() {
                    "exact" => ComponentAttachMode::Exact,
                    "nearby" => ComponentAttachMode::Nearby,
                    "nearest" => ComponentAttachMode::Nearest,
                    _ => panic!("--component-attach-mode must be exact, nearby, or nearest"),
                }
            }
            "--component-join-gap" => {
                cli.component_join_gap = value
                    .parse::<u32>()
                    .expect("component join gap must be a non-negative integer")
            }
            "--component-min-blocks" => {
                cli.component_min_blocks = value
                    .parse::<usize>()
                    .expect("component minimum blocks must be a non-negative integer")
            }
            "--world-prefix" => cli.world_prefix = Some(value.clone()),
            other => panic!("unknown argument {other}"),
        }
        index += 2;
    }
    if [grid_pitch, grid_size, grid_offset_x, grid_offset_z]
        .iter()
        .any(Option::is_some)
    {
        let grid = Grid {
            pitch: grid_pitch.expect("--grid-pitch is required with grid options"),
            size: grid_size.expect("--grid-size is required with grid options"),
            offset_x: grid_offset_x.expect("--grid-offset-x is required with grid options"),
            offset_z: grid_offset_z.expect("--grid-offset-z is required with grid options"),
        };
        assert!(grid.pitch > 0, "--grid-pitch must be positive");
        assert!(
            grid.size > 0 && grid.size <= grid.pitch,
            "--grid-size must be in 1..=pitch"
        );
        cli.grid = Some(grid);
    }
    assert_eq!(
        cli.substrate.is_some(),
        cli.substrate_band.is_some(),
        "--substrate and --substrate-band must be supplied together"
    );
    assert!(
        cli.grid.is_some() || cli.grid_index_bounds.is_none(),
        "--grid-index-bounds requires grid options"
    );
    assert!(
        cli.grid.is_none() || cli.partition_hints.is_none(),
        "grid options and --partition-hints are mutually exclusive"
    );
    assert!(
        cli.partition_floor_share >= 0.0 && cli.partition_floor_share <= 1.0,
        "--partition-floor-share must be in 0..=1"
    );
    assert!(
        cli.partition_dense_layer_coverage >= 0.0 && cli.partition_dense_layer_coverage <= 1.0,
        "--partition-dense-layer-coverage must be in 0..=1"
    );
    assert_eq!(
        cli.input.contains("://"),
        cli.world_prefix.is_some(),
        "store inputs require --world-prefix; local inputs must omit it"
    );
    cli
}

fn source(cli: &Cli) -> Box<dyn TileSource> {
    if let Some(prefix) = &cli.world_prefix {
        let store = nucleation::store::open(&cli.input)
            .unwrap_or_else(|e| panic!("failed to open input store {}: {e}", cli.input));
        Box::new(
            nucleation::world_segment::StoreRegionTiles::new(store, prefix, -64, 320)
                .unwrap_or_else(|e| panic!("failed to open region prefix {prefix}: {e}"))
                .with_world_rect(cli.rect.0, cli.rect.1, cli.rect.2, cli.rect.3),
        )
    } else if PathBuf::from(&cli.input).is_dir() {
        let source = WorldSource::open_dir(PathBuf::from(&cli.input).as_path())
            .unwrap_or_else(|e| panic!("failed to open {}: {e}", cli.input));
        Box::new(
            nucleation::world_segment::WorldSourceTiles::new(source, -64, 320)
                .with_world_rect(cli.rect.0, cli.rect.1, cli.rect.2, cli.rect.3),
        )
    } else {
        Box::new(
            TarArchiveSource::open(&cli.input, -64, 320)
                .unwrap_or_else(|e| panic!("failed to open {}: {e}", cli.input))
                .with_world_rect(cli.rect.0, cli.rect.1, cli.rect.2, cli.rect.3)
                .quiet_filtered_entries(),
        )
    }
}

fn grid_axis_range(min: i32, max: i32, offset: i32, grid: Grid) -> std::ops::RangeInclusive<i32> {
    let first = (min - (offset + grid.size - 1) + grid.pitch - 1).div_euclid(grid.pitch);
    let last = (max - offset).div_euclid(grid.pitch);
    first..=last
}

/// Generic JSON row for caller-owned spatial partitions. Extra scalar fields
/// (owner, tags, etc.) are captured as attribution metadata, so a richer
/// catalogue can be used directly without teaching Nucleation its domain.
#[derive(serde::Deserialize)]
struct PartitionRow {
    id: String,
    x0: i32,
    x1: i32,
    z0: i32,
    z1: i32,
    #[serde(default)]
    y0: Option<i32>,
    #[serde(default)]
    y1: Option<i32>,
    #[serde(flatten)]
    metadata: BTreeMap<String, serde_json::Value>,
}

struct PartitionCatalog {
    hints: Vec<PartitionHint>,
    metadata_by_id: BTreeMap<String, BTreeMap<String, String>>,
    content_hash: String,
}

fn load_partition_catalog(path: &std::path::Path) -> PartitionCatalog {
    let bytes = std::fs::read(path).unwrap_or_else(|error| {
        panic!("failed to read partition hints {}: {error}", path.display())
    });
    let rows: Vec<PartitionRow> = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "failed to parse partition hints {}: {error}",
            path.display()
        )
    });
    assert!(!rows.is_empty(), "partition-hints file must not be empty");
    let mut hints = Vec::with_capacity(rows.len());
    let mut values_by_id = BTreeMap::<String, BTreeMap<String, BTreeSet<String>>>::new();
    for row in rows {
        let y_range = match (row.y0, row.y1) {
            (None, None) => None,
            (Some(y0), Some(y1)) => Some((y0.min(y1), y0.max(y1))),
            _ => panic!("partition {} must provide both y0 and y1", row.id),
        };
        for (key, value) in row.metadata {
            // Geometry bookkeeping is not attribution.  Preserve scalar
            // caller metadata under stable provenance keys; ignore nested
            // values rather than stuffing unbounded JSON into every schematic.
            if matches!(key.as_str(), "key" | "k" | "corners") {
                continue;
            }
            let rendered = match value {
                serde_json::Value::String(value) if !value.is_empty() && value != "None" => value,
                serde_json::Value::Bool(value) => value.to_string(),
                serde_json::Value::Number(value) => value.to_string(),
                _ => continue,
            };
            let normalized_key: String = key
                .chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() {
                        ch.to_ascii_lowercase()
                    } else {
                        '_'
                    }
                })
                .collect();
            values_by_id
                .entry(row.id.clone())
                .or_default()
                .entry(normalized_key)
                .or_default()
                .insert(rendered);
        }
        hints.push(PartitionHint {
            id: row.id,
            bbox_xz: (
                row.x0.min(row.x1),
                row.x0.max(row.x1),
                row.z0.min(row.z1),
                row.z0.max(row.z1),
            ),
            y_range,
        });
    }
    let metadata_by_id = values_by_id
        .into_iter()
        .map(|(id, fields)| {
            let fields = fields
                .into_iter()
                .map(|(key, values)| (key, values.into_iter().collect::<Vec<_>>().join(",")))
                .collect();
            (id, fields)
        })
        .collect();
    PartitionCatalog {
        hints,
        metadata_by_id,
        content_hash: ContentId::of(&[&bytes]).to_string(),
    }
}

fn partitions(
    cli: &Cli,
) -> (
    PartitionIndex,
    BTreeMap<String, BTreeMap<String, String>>,
    Option<String>,
) {
    if let Some(path) = &cli.partition_hints {
        let catalog = load_partition_catalog(path);
        return (
            PartitionIndex::new(catalog.hints),
            catalog.metadata_by_id,
            Some(catalog.content_hash),
        );
    }
    let Some(grid) = cli.grid else {
        return (
            PartitionIndex::new(vec![PartitionHint {
                id: "selection".to_string(),
                bbox_xz: (cli.rect.0, cli.rect.2, cli.rect.1, cli.rect.3),
                y_range: None,
            }]),
            BTreeMap::new(),
            None,
        );
    };
    let (gx_range, gz_range) = match cli.grid_index_bounds {
        Some((gx0, gz0, gx1, gz1)) => (gx0..=gx1, gz0..=gz1),
        None => (
            grid_axis_range(cli.rect.0, cli.rect.2, grid.offset_x, grid),
            grid_axis_range(cli.rect.1, cli.rect.3, grid.offset_z, grid),
        ),
    };
    let mut hints = Vec::new();
    for gx in gx_range {
        for gz in gz_range.clone() {
            let min_x = grid.offset_x + gx * grid.pitch;
            let min_z = grid.offset_z + gz * grid.pitch;
            hints.push(PartitionHint {
                id: format!("grid:{gx}:{gz}"),
                bbox_xz: (min_x, min_x + grid.size - 1, min_z, min_z + grid.size - 1),
                y_range: None,
            });
        }
    }
    (PartitionIndex::new(hints), BTreeMap::new(), None)
}

fn output_store(
    cli: &Cli,
) -> Result<Box<dyn nucleation::store::Store>, Box<dyn std::error::Error>> {
    if cli.output.contains("://") {
        Ok(nucleation::store::open(&cli.output)?)
    } else {
        Ok(Box::new(nucleation::store::FsStore::new(&cli.output)))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_args();
    let store = output_store(&cli)?;
    store.health()?;

    println!(
        "segment_world: sampling {} within x[{},{}] z[{},{}]",
        cli.input, cli.rect.0, cli.rect.2, cli.rect.1, cli.rect.3
    );
    let mut samples: Vec<VoxelTile> = Vec::new();
    let mut profile = if let (Some(palette), Some(band)) = (&cli.substrate, cli.substrate_band) {
        println!("segment_world: using supplied substrate y={band:?}, palette={palette:?}");
        WorldProfile::new(palette.clone(), band)
    } else {
        source(&cli).for_each_tile(&mut |tile| {
            samples.push(tile);
            Ok(())
        })?;
        if samples.is_empty() {
            return Err("the selected rectangle contained no readable region data".into());
        }
        WorldProfile::derive(
            &samples,
            &ProfileParams {
                sample_stride: 1,
                min_slab_coverage: 0.75,
                y_scan: (-64, 320),
                palette_min_share: 0.02,
            },
        )
    };
    // A selection with builds in nearly every column can defeat slab-density
    // inference. Fall back to its single most common block name at the most
    // populated Y level, rather than silently treating the entire floor as a
    // build. This is generic and based only on the selected content.
    if profile.substrate_palette.is_empty() {
        let mut by_y_name =
            std::collections::BTreeMap::<i32, std::collections::BTreeMap<String, u64>>::new();
        for tile in &samples {
            for ((_x, y, _z), state) in tile.blocks() {
                *by_y_name
                    .entry(y)
                    .or_default()
                    .entry(state.get_name().to_string())
                    .or_default() += 1;
            }
        }
        if let Some((y, names)) = by_y_name
            .iter()
            .max_by_key(|(_, names)| names.values().sum::<u64>())
        {
            if let Some((name, _)) = names.iter().max_by_key(|(_, count)| *count) {
                profile = WorldProfile::new(BTreeSet::from([name.clone()]), (*y, *y));
            }
        }
    }
    println!(
        "segment_world: substrate y={:?}, palette={:?}",
        profile.substrate_y_band, profile.substrate_palette
    );
    drop(samples);

    let (partitions, partition_metadata, partition_catalog_hash) = partitions(&cli);
    let partition_floor_share =
        (cli.partition_floor_share > 0.0).then_some(cli.partition_floor_share);
    let partition_dense_layer_coverage =
        (cli.partition_dense_layer_coverage > 0.0).then_some(cli.partition_dense_layer_coverage);
    let split_disconnected = (cli.split_min_blocks > 0).then_some(DisconnectedSplit {
        min_component_blocks: cli.split_min_blocks,
        ..DisconnectedSplit::default()
    });
    let drop_unpartitioned = cli.drop_unpartitioned.unwrap_or(cli.grid.is_some());
    let job = SegmentJob {
        config: SegConfig {
            cell_size: 4,
            closing_radius: 2,
            partition_policy: PartitionPolicy::HardCut,
            partition_floor_share,
            partition_dense_layer_coverage,
            split_disconnected,
            drop_unpartitioned,
            ..SegConfig::default()
        },
        score_config: ScoreConfig::default(),
        source_id: cli.source_id.clone(),
        snapshot_id: cli.snapshot_id.clone(),
        min_y: -64,
        max_y: 320,
        extracted_at: cli.extracted_at,
        match_iou: 0.5,
    };

    let mut manifest = Vec::<String>::new();
    let mut written = 0u64;
    let mut emit = |build: MaterializedBuild| {
        let (pieces, attach_mode): (Vec<_>, &str) = match cli.component_attach_mode {
            ComponentAttachMode::Exact => (
                build.schematic.split_connected(Connectivity::Corner),
                "exact",
            ),
            ComponentAttachMode::Nearby => (
                build.schematic.split_connected_attach_nearby(
                    Connectivity::Corner,
                    cli.component_min_blocks,
                    cli.component_join_gap,
                ),
                "nearby",
            ),
            ComponentAttachMode::Nearest => (
                build
                    .schematic
                    .split_connected_attach(Connectivity::Corner, cli.component_min_blocks),
                "nearest",
            ),
        };
        let split = pieces.len() > 1;
        for mut piece in pieces {
            let mut provenance = build.provenance.clone();
            provenance.config_hash = ContentId::of(&[
                b"segment.output.v2",
                provenance.config_hash.as_bytes(),
                &(cli.component_min_blocks as u64).to_le_bytes(),
                &cli.component_join_gap.to_le_bytes(),
                attach_mode.as_bytes(),
            ]);
            if split {
                let bbox = piece.get_bounding_box();
                let origin = build.provenance.origin_offset;
                provenance.world_bbox = (
                    (
                        origin.0 + bbox.min.0,
                        origin.1 + bbox.min.1,
                        origin.2 + bbox.min.2,
                    ),
                    (
                        origin.0 + bbox.max.0,
                        origin.1 + bbox.max.1,
                        origin.2 + bbox.max.2,
                    ),
                );
                let bbox_words = [
                    provenance.world_bbox.0 .0,
                    provenance.world_bbox.0 .1,
                    provenance.world_bbox.0 .2,
                    provenance.world_bbox.1 .0,
                    provenance.world_bbox.1 .1,
                    provenance.world_bbox.1 .2,
                ];
                let bbox_bytes = bbox_words
                    .iter()
                    .flat_map(|value| value.to_le_bytes())
                    .collect::<Vec<_>>();
                let piece_fingerprint = nucleation::fingerprint::fingerprint(
                    &piece,
                    &nucleation::fingerprint::FingerprintSpec::exact(),
                )
                .0;
                provenance.stable_build_id = StableBuildId(ContentId::of(&[
                    b"segment.component.v3",
                    build.provenance.stable_build_id.0.as_bytes(),
                    &bbox_bytes,
                    &piece_fingerprint.to_le_bytes(),
                ]));
                provenance.block_count = piece.total_blocks().max(0) as u64;
                provenance.fingerprint = piece_fingerprint;
                if provenance.block_count <= job.score_config.debris_max_blocks {
                    provenance.tier = Tier::Debris;
                }
            }
            let mut embedded = piece
                .metadata
                .provenance
                .take()
                .unwrap_or_else(|| SchematicProvenance::new(&cli.source_id).unwrap());
            embedded.source_id = cli.source_id.clone();
            embedded.world_name = Some(cli.world_name.clone());
            embedded.map_name = Some(cli.map_name.clone());
            embedded.dimension = Some(cli.dimension.clone());
            embedded.snapshot_id = Some(cli.snapshot_id.clone());
            embedded.world_bbox = Some(
                ProvenanceBounds::new(
                    [
                        provenance.world_bbox.0 .0,
                        provenance.world_bbox.0 .1,
                        provenance.world_bbox.0 .2,
                    ],
                    [
                        provenance.world_bbox.1 .0,
                        provenance.world_bbox.1 .1,
                        provenance.world_bbox.1 .2,
                    ],
                )
                .expect("piece world bbox is ordered"),
            );
            embedded.origin = Some([
                provenance.origin_offset.0,
                provenance.origin_offset.1,
                provenance.origin_offset.2,
            ]);
            embedded.partition_id = provenance.partition_id.clone();
            if let Some(partition_id) = provenance.partition_id.as_deref() {
                if let Some(fields) = partition_metadata.get(partition_id) {
                    for (key, value) in fields {
                        embedded
                            .attributes
                            .insert(format!("nucleation:partition_{key}"), value.clone());
                    }
                }
            }
            if let Some(hash) = &partition_catalog_hash {
                embedded.attributes.insert(
                    "nucleation:partition_catalog_hash".to_string(),
                    hash.clone(),
                );
            }
            embedded.stable_build_id = Some(provenance.stable_build_id.to_string());
            embedded.extracted_at = Some(cli.extracted_at);
            embedded.config_hash = Some(provenance.config_hash.to_string());
            embedded.profile_hash = Some(provenance.profile_hash.to_string());
            embedded.attributes.insert(
                "nucleation:component_join_gap".to_string(),
                cli.component_join_gap.to_string(),
            );
            embedded.attributes.insert(
                "nucleation:component_min_blocks".to_string(),
                cli.component_min_blocks.to_string(),
            );
            embedded.attributes.insert(
                "nucleation:component_attach_mode".to_string(),
                attach_mode.to_string(),
            );
            embedded.attributes.insert(
                "nucleation:partition_floor_share".to_string(),
                cli.partition_floor_share.to_string(),
            );
            embedded.attributes.insert(
                "nucleation:partition_dense_layer_coverage".to_string(),
                cli.partition_dense_layer_coverage.to_string(),
            );
            embedded.attributes.insert(
                "nucleation:split_min_blocks".to_string(),
                cli.split_min_blocks.to_string(),
            );
            piece.metadata.provenance = Some(embedded);
            // `Provenance::fingerprint` is a full u128. JSON numbers cannot
            // represent every u128 and serde_json deliberately rejects values
            // above u64::MAX when building a Value. IDs and fingerprints are
            // content addresses anyway, so the queryable catalog uses their
            // canonical hexadecimal strings.
            let mut catalog_provenance = serde_json::json!({
                "stable_build_id": provenance.stable_build_id.to_string(),
                "snapshot_build_id": provenance.snapshot_build_id.to_string(),
                "source_id": &provenance.source_id,
                "snapshot_id": &provenance.snapshot_id,
                "world_bbox": provenance.world_bbox,
                "origin_offset": provenance.origin_offset,
                "partition_id": &provenance.partition_id,
                "block_count": provenance.block_count,
                "cluster_count": provenance.cluster_count,
                "fingerprint": format!("{:032x}", provenance.fingerprint),
                "tier": provenance.tier,
                "config_hash": provenance.config_hash.to_string(),
                "profile_hash": provenance.profile_hash.to_string(),
                "extracted_at": provenance.extracted_at,
            });
            if let Some(object) = catalog_provenance.as_object_mut() {
                if let Some(partition_id) = provenance.partition_id.as_deref() {
                    if let Some(fields) = partition_metadata.get(partition_id) {
                        object.insert(
                            "partition_metadata".to_string(),
                            serde_json::to_value(fields).expect("serialize partition metadata"),
                        );
                    }
                }
                if let Some(hash) = &partition_catalog_hash {
                    object.insert(
                        "partition_catalog_hash".to_string(),
                        serde_json::Value::String(hash.clone()),
                    );
                }
            }
            manifest.push(
                serde_json::to_string(&catalog_provenance).expect("serialize catalog provenance"),
            );
            let provenance_key = format!("provenance/{}.json", provenance.stable_build_id);
            let embedded_json = piece
                .metadata
                .provenance
                .as_ref()
                .expect("embedded provenance")
                .to_json()
                .expect("serialize embedded provenance");
            store
                .put(&provenance_key, embedded_json.as_bytes())
                .expect("store provenance");
            let key = format!("schematics/{}.schem", provenance.stable_build_id);
            let bytes = to_schematic(&piece).expect("serialize schematic");
            store.put(&key, &bytes).expect("store schematic");
            written += 1;
            println!(
                "segment_world: wrote {} ({} blocks, {:?}, {} block entities)",
                key,
                provenance.block_count,
                provenance.tier,
                piece.get_block_entities_as_list().len()
            );
        }
    };

    let stats: RunStats = WorldSegmenter::run_streaming(
        source(&cli).as_ref(),
        &profile,
        &partitions,
        &job,
        &[],
        &mut emit,
    );
    let catalog_key = format!(
        "catalog/x{}_{}_z{}_{}.jsonl",
        cli.rect.0, cli.rect.2, cli.rect.1, cli.rect.3
    );
    let mut catalog = manifest.join("\n");
    if !catalog.is_empty() {
        catalog.push('\n');
    }
    store.put(&catalog_key, catalog.as_bytes())?;
    println!("segment_world: {stats:?}; wrote {written} schematics");
    Ok(())
}
