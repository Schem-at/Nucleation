#![cfg(all(feature = "world-segment", feature = "store-fs"))]

use nucleation::formats::world_stream::{WorldChunkView, WorldSink};
use nucleation::store::{FsStore, MemStore, Store};
use nucleation::world_segment::snapshot::{index_snapshot, SnapshotManifest, SnapshotTiles};
use nucleation::world_segment::{TileId, TileSource};
use nucleation::BlockState;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleation-snapshot-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn world(&self, position: i32) {
        let mut sink = WorldSink::create(&self.0.join("world"), None).unwrap();
        for cx in [0, 40] {
            let mut chunk = WorldChunkView::new(cx, 0);
            chunk.set_block(
                cx * 16 + position,
                4,
                2,
                &BlockState::new("minecraft:redstone_block"),
            );
            sink.write_chunk(&chunk).unwrap();
        }
        sink.finish().unwrap();
    }
    fn index(&self, store: &dyn Store, previous: Option<&SnapshotManifest>) -> SnapshotManifest {
        index_snapshot(
            self.0.join("world").to_str().unwrap(),
            store,
            "test:world",
            "minecraft:overworld",
            None,
            previous,
            &mut |_| {},
        )
        .unwrap()
        .0
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn identical_snapshots_share_objects_and_manifest() {
    let fixture = Fixture::new();
    fixture.world(1);
    let store = MemStore::new();
    let first = fixture.index(&store, None);
    let mut reused = 0;
    let (second, _) = index_snapshot(
        fixture.0.join("world").to_str().unwrap(),
        &store,
        "test:world",
        "minecraft:overworld",
        None,
        Some(&first),
        &mut |p| reused = p.reused_regions,
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.hash().unwrap(), second.hash().unwrap());
    assert_eq!(reused, 2);
    assert_eq!(store.list("objects/").unwrap().len(), 2);
    assert_eq!(store.list("manifests/").unwrap().len(), 1);
}

#[test]
fn save_timestamps_change_raw_observation_but_not_extraction_dependencies() {
    let fixture = Fixture::new();
    fixture.world(1);
    let store = MemStore::new();
    let first = fixture.index(&store, None);
    let path = fixture.0.join("world/region/r.0.0.mca");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[4096..4100].copy_from_slice(&123456u32.to_be_bytes());
    std::fs::write(path, bytes).unwrap();
    let second = fixture.index(&store, Some(&first));
    assert_ne!(first.hash().unwrap(), second.hash().unwrap());
    assert_eq!(
        first.rectangle_hash((0, 0, 511, 511)),
        second.rectangle_hash((0, 0, 511, 511))
    );
    assert_eq!(
        first.regions["r.0.0.mca"].chunks,
        second.regions["r.0.0.mca"].chunks
    );
}

#[test]
fn moving_content_within_a_chunk_invalidates_its_rectangle_only() {
    let first_world = Fixture::new();
    first_world.world(1);
    let second_world = Fixture::new();
    second_world.world(2);
    std::fs::copy(
        first_world.0.join("world/region/r.1.0.mca"),
        second_world.0.join("world/region/r.1.0.mca"),
    )
    .unwrap();
    let store = MemStore::new();
    let first = first_world.index(&store, None);
    let second = second_world.index(&store, Some(&first));
    assert_ne!(
        first.rectangle_hash((0, 0, 511, 511)),
        second.rectangle_hash((0, 0, 511, 511))
    );
    assert_eq!(
        first.rectangle_hash((512, 0, 1023, 511)),
        second.rectangle_hash((512, 0, 1023, 511))
    );
    assert_eq!(
        first.rectangle_hash((16, 0, 31, 15)),
        second.rectangle_hash((16, 0, 31, 15))
    );
}

#[test]
fn corrupt_input_never_publishes_a_complete_manifest() {
    let fixture = Fixture::new();
    fixture.world(1);
    std::fs::write(fixture.0.join("world/region/r.0.0.mca"), b"broken").unwrap();
    let store = MemStore::new();
    let result = index_snapshot(
        fixture.0.join("world").to_str().unwrap(),
        &store,
        "test:world",
        "minecraft:overworld",
        None,
        None,
        &mut |_| {},
    );
    assert!(result.is_err());
    assert!(store.list("manifests/").unwrap().is_empty());
}

#[test]
fn archival_capture_preserves_unreadable_regions_but_never_extracts_them_as_empty() {
    use nucleation::world_segment::snapshot::index_snapshot_with_policy;
    let fixture = Fixture::new();
    fixture.world(1);
    std::fs::write(fixture.0.join("world/region/r.0.0.mca"), b"").unwrap();
    let store_path = fixture.0.join("objects");
    let store = FsStore::new(&store_path);
    let (manifest, progress) = index_snapshot_with_policy(
        fixture.0.join("world").to_str().unwrap(),
        &store,
        "test:world",
        "minecraft:overworld",
        None,
        None,
        true,
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(progress.unreadable_regions, 1);
    assert!(manifest.regions["r.0.0.mca"].error.is_some());
    assert_eq!(
        store
            .get(&manifest.regions["r.0.0.mca"].object_key)
            .unwrap(),
        Some(vec![])
    );
    let damaged = SnapshotTiles::new(
        manifest.clone(),
        Box::new(FsStore::new(&store_path)),
        (0, 0, 511, 511),
    )
    .unwrap();
    assert!(damaged.tile(TileId { x: 0, z: 0 }).is_err());
    let acknowledged = SnapshotTiles::with_empty_region_policy(
        manifest.clone(),
        Box::new(FsStore::new(&store_path)),
        (0, 0, 511, 511),
        nucleation::world_segment::snapshot::EmptyRegionPolicy::AcknowledgeZeroByte,
    )
    .unwrap();
    assert!(acknowledged.tile(TileId { x: 0, z: 0 }).unwrap().is_none());
    // The policy does not authorize missing objects or modified bytes.
    store
        .put(&manifest.regions["r.0.0.mca"].object_key, b"corrupt")
        .unwrap();
    assert!(acknowledged.tile(TileId { x: 0, z: 0 }).is_err());
    let readable = SnapshotTiles::new(
        manifest,
        Box::new(FsStore::new(&store_path)),
        (512, 0, 1023, 511),
    )
    .unwrap();
    assert!(readable.tile(TileId { x: 1, z: 0 }).unwrap().is_some());
}

#[test]
fn snapshot_reads_are_independent_of_original_world_and_verify_objects() {
    let fixture = Fixture::new();
    fixture.world(1);
    let object_root = fixture.0.join("objects");
    let store = FsStore::new(&object_root);
    let manifest = fixture.index(&store, None);
    let source = SnapshotTiles::new(
        manifest.clone(),
        Box::new(FsStore::new(&object_root)),
        (0, 0, 511, 511),
    )
    .unwrap();
    std::fs::remove_dir_all(fixture.0.join("world")).unwrap();
    assert!(source.tile(TileId { x: 0, z: 0 }).unwrap().is_some());
    store
        .put(&manifest.regions["r.0.0.mca"].object_key, b"corrupt")
        .unwrap();
    assert!(source.tile(TileId { x: 0, z: 0 }).is_err());
}

#[test]
fn zero_byte_acknowledgement_never_skips_nonempty_corruption() {
    use nucleation::world_segment::snapshot::{index_snapshot_with_policy, EmptyRegionPolicy};
    let fixture = Fixture::new();
    fixture.world(1);
    std::fs::write(fixture.0.join("world/region/r.0.0.mca"), b"broken").unwrap();
    let store_path = fixture.0.join("objects");
    let (manifest, _) = index_snapshot_with_policy(
        fixture.0.join("world").to_str().unwrap(),
        &FsStore::new(&store_path),
        "test:world",
        "minecraft:overworld",
        None,
        None,
        true,
        &mut |_| {},
    )
    .unwrap();
    let source = SnapshotTiles::with_empty_region_policy(
        manifest,
        Box::new(FsStore::new(&store_path)),
        (0, 0, 511, 511),
        EmptyRegionPolicy::AcknowledgeZeroByte,
    )
    .unwrap();
    assert!(source.tile(TileId { x: 0, z: 0 }).is_err());
}

#[test]
fn archive_and_directory_produce_the_same_manifest() {
    let fixture = Fixture::new();
    fixture.world(1);
    let store = MemStore::new();
    let expected = fixture.index(&store, None);
    let archive = fixture.0.join("world.tar.gz");
    let gzip = flate2::write::GzEncoder::new(
        std::fs::File::create(&archive).unwrap(),
        flate2::Compression::fast(),
    );
    let mut tar = tar::Builder::new(gzip);
    tar.append_dir_all("world", fixture.0.join("world"))
        .unwrap();
    tar.into_inner().unwrap().finish().unwrap();
    let (actual, _) = index_snapshot(
        archive.to_str().unwrap(),
        &store,
        "test:world",
        "minecraft:overworld",
        None,
        None,
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn worker_cli_indexes_extracts_and_certifies_an_empty_rectangle() {
    let fixture = Fixture::new();
    fixture.world(1);
    let objects = fixture.0.join("objects");
    let indexed = std::process::Command::new(env!("CARGO_BIN_EXE_segment_world"))
        .args([
            "index",
            fixture.0.join("world").to_str().unwrap(),
            objects.to_str().unwrap(),
            "--source-id",
            "test:world",
        ])
        .output()
        .unwrap();
    assert!(
        indexed.status.success(),
        "{}",
        String::from_utf8_lossy(&indexed.stderr)
    );
    let completed: serde_json::Value = String::from_utf8(indexed.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|value| value["event"] == "snapshot_completed")
        .unwrap();
    let manifest = objects.join(completed["manifest_key"].as_str().unwrap());
    for (bounds, expected) in [
        (["-32", "-32", "511", "511"], 1),
        (["1024", "0", "1535", "511"], 0),
    ] {
        let output = fixture.0.join(format!("extraction-{expected}"));
        let run = std::process::Command::new(env!("CARGO_BIN_EXE_segment_world"))
            .args([manifest.to_str().unwrap(), output.to_str().unwrap()])
            .args(bounds)
            .args([
                "--snapshot-store",
                objects.to_str().unwrap(),
                "--source-id",
                "test:world",
                "--snapshot-id",
                "fixture-snapshot",
                "--substrate",
                "minecraft:stone",
                "--substrate-band",
                "0,0",
                "--split-min-blocks",
                "1",
                "--component-min-blocks",
                "1",
                "--progress-json",
                "true",
            ])
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(output.join("completion.json")).unwrap())
                .unwrap();
        assert_eq!(report["complete"], true);
        assert_eq!(report["source_hash"], "fixture-snapshot");
        assert_eq!(report["builds"], expected);
        assert!(output.join(report["catalog"].as_str().unwrap()).is_file());
    }
    let clipped = fixture.0.join("clipped");
    let run = std::process::Command::new(env!("CARGO_BIN_EXE_segment_world"))
        .args([
            manifest.to_str().unwrap(),
            clipped.to_str().unwrap(),
            "0",
            "0",
            "511",
            "511",
            "--snapshot-store",
            objects.to_str().unwrap(),
            "--source-id",
            "test:world",
            "--substrate",
            "minecraft:stone",
            "--substrate-band",
            "0,0",
            "--split-min-blocks",
            "100000",
        ])
        .output()
        .unwrap();
    assert!(
        !run.status.success(),
        "implicit selection cannot certify a clipped build"
    );
    assert!(String::from_utf8_lossy(&run.stderr).contains("uncertified extraction boundary"));
    assert!(!clipped.join("completion.json").exists());
}

#[test]
fn deleting_a_region_changes_its_dependencies_but_keeps_previous_bytes_readable() {
    let fixture = Fixture::new();
    fixture.world(1);
    let store = MemStore::new();
    let first = fixture.index(&store, None);
    std::fs::remove_file(fixture.0.join("world/region/r.0.0.mca")).unwrap();
    let second = fixture.index(&store, Some(&first));
    assert_ne!(
        first.rectangle_hash((0, 0, 511, 511)),
        second.rectangle_hash((0, 0, 511, 511))
    );
    assert_eq!(
        first.rectangle_hash((512, 0, 1023, 511)),
        second.rectangle_hash((512, 0, 1023, 511))
    );
    assert!(store
        .get(&first.regions["r.0.0.mca"].object_key)
        .unwrap()
        .is_some());
}

#[test]
fn worker_cli_requires_acknowledgement_and_receipts_zero_byte_gaps() {
    use nucleation::world_segment::snapshot::index_snapshot_with_policy;
    let fixture = Fixture::new();
    fixture.world(1);
    std::fs::write(fixture.0.join("world/region/r.0.0.mca"), b"").unwrap();
    let objects = fixture.0.join("objects");
    let (_, progress) = index_snapshot_with_policy(
        fixture.0.join("world").to_str().unwrap(),
        &FsStore::new(&objects),
        "test:world",
        "minecraft:overworld",
        None,
        None,
        true,
        &mut |_| {},
    )
    .unwrap();
    let manifest = objects.join(progress.manifest_key.unwrap());
    for (policy, success) in [("reject", false), ("acknowledge-zero-byte", true)] {
        let output = fixture.0.join(policy);
        let run = std::process::Command::new(env!("CARGO_BIN_EXE_segment_world"))
            .args([
                manifest.to_str().unwrap(),
                output.to_str().unwrap(),
                "0",
                "0",
                "511",
                "511",
                "--snapshot-store",
                objects.to_str().unwrap(),
                "--source-id",
                "test:world",
                "--empty-region-policy",
                policy,
                "--substrate",
                "minecraft:stone",
                "--substrate-band",
                "0,0",
            ])
            .output()
            .unwrap();
        assert_eq!(
            run.status.success(),
            success,
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        if success {
            let report: serde_json::Value =
                serde_json::from_slice(&std::fs::read(output.join("completion.json")).unwrap())
                    .unwrap();
            assert_eq!(report["coverage"], "acknowledged_gaps");
            assert_eq!(
                report["acknowledged_zero_byte_regions"],
                serde_json::json!([[0, 0]])
            );
            assert_eq!(report["builds"], 0);
        } else {
            assert!(!output.join("completion.json").exists());
        }
    }
}

#[test]
fn sign_text_changes_invalidate_extraction_without_block_state_changes() {
    use nucleation::block_entity::BlockEntity;
    use nucleation::block_position::BlockPosition;
    use nucleation::utils::NbtValue;
    use nucleation::UniversalSchematic;
    let fixture = Fixture::new();
    let store = MemStore::new();
    let mut schematic = UniversalSchematic::new("Sign".into());
    schematic.set_block(1, 4, 2, &BlockState::new("minecraft:oak_sign"));
    let sign = |text: &str| {
        BlockEntity::new("minecraft:sign".into(), (1, 4, 2))
            .with_nbt_data("Text1".into(), NbtValue::String(text.into()))
    };
    schematic.set_block_entity(BlockPosition::new(1, 4, 2), sign("first label"));
    nucleation::formats::world::save_world(&schematic, &fixture.0.join("world"), None).unwrap();
    let first = fixture.index(&store, None);
    schematic.set_block_entity(BlockPosition::new(1, 4, 2), sign("changed label"));
    nucleation::formats::world::save_world(&schematic, &fixture.0.join("world"), None).unwrap();
    let second = fixture.index(&store, Some(&first));
    assert_ne!(
        first.rectangle_hash((0, 0, 511, 511)),
        second.rectangle_hash((0, 0, 511, 511))
    );
}

#[test]
fn manifest_rejects_object_path_substitution() {
    let fixture = Fixture::new();
    fixture.world(1);
    let mut manifest = fixture.index(&MemStore::new(), None);
    manifest.regions.get_mut("r.0.0.mca").unwrap().object_key = "../../somewhere".into();
    assert!(SnapshotManifest::from_bytes(&serde_json::to_vec(&manifest).unwrap()).is_err());
}

#[test]
fn coverage_rejects_clipped_content_before_filtering_but_accepts_complete_hard_partitions() {
    use nucleation::world_segment::coverage::CoverageCheckedTiles;
    use nucleation::world_segment::{PartitionHint, PartitionIndex, WorldProfile};
    let fixture = Fixture::new();
    fixture.world(1);
    let store = MemStore::new();
    let manifest = fixture.index(&store, None);
    let source = SnapshotTiles::new(manifest, Box::new(store), (0, 0, 31, 31)).unwrap();
    let profile = WorldProfile::new(Default::default(), (0, 0));
    let empty = PartitionIndex::new(vec![]);
    let mut checked = CoverageCheckedTiles {
        source: &source,
        profile: &profile,
        partitions: &empty,
        rect: (0, 0, 31, 31),
        margin: 20,
        drop_unpartitioned: true,
    };
    assert!(checked
        .for_each_tile(&mut |_| Ok(()))
        .unwrap_err()
        .to_string()
        .contains("uncertified extraction boundary"));
    let partitions = PartitionIndex::new(vec![PartitionHint {
        id: "plot".into(),
        bbox_xz: (0, 31, 0, 31),
        y_range: None,
    }]);
    checked.partitions = &partitions;
    assert!(checked.for_each_tile(&mut |_| Ok(())).is_ok());
    let clipped = PartitionIndex::new(vec![PartitionHint {
        id: "plot".into(),
        bbox_xz: (0, 100, 0, 31),
        y_range: None,
    }]);
    checked.partitions = &clipped;
    assert!(checked.for_each_tile(&mut |_| Ok(())).is_err());
}
