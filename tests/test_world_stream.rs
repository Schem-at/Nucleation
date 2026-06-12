use nucleation::formats::anvil::{McaFile, RegionReader};
use nucleation::formats::world;
use nucleation::{BlockState, UniversalSchematic};
use std::io::Cursor;

/// Build a small schematic spanning two chunks, export it as world files,
/// and return the bytes of the single region file r.0.0.mca.
fn region_file_fixture() -> Vec<u8> {
    let mut schem = UniversalSchematic::new("fixture".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());
    let gold = BlockState::new("minecraft:gold_block".to_string());
    schem.set_block(0, 64, 0, &stone);
    schem.set_block(5, 65, 5, &gold);
    schem.set_block(20, 70, 3, &stone); // second chunk (cx=1)
    let files = world::to_world(&schem, None).expect("to_world");
    files
        .iter()
        .find(|(path, _)| path.ends_with("r.0.0.mca"))
        .map(|(_, data)| data.clone())
        .expect("region file present")
}

#[test]
fn region_reader_matches_eager_mca_parse() {
    let mca_bytes = region_file_fixture();
    let eager = McaFile::from_bytes(&mca_bytes, 0, 0).expect("eager parse");
    let eager_positions: Vec<(i32, i32)> =
        eager.chunks.iter().flatten().map(|c| (c.x, c.z)).collect();

    let mut reader = RegionReader::new(Cursor::new(mca_bytes), 0, 0).expect("reader");
    let lazy_positions = reader.chunk_positions();
    // Header table may list chunks that fail NBT parse; every eager chunk
    // must be present and readable lazily with identical content.
    for (cx, cz) in &eager_positions {
        assert!(lazy_positions.contains(&(*cx, *cz)));
        let lazy = reader
            .read_chunk(*cx, *cz)
            .expect("read ok")
            .expect("chunk present");
        let eager_chunk = eager
            .chunks
            .iter()
            .flatten()
            .find(|c| c.x == *cx && c.z == *cz)
            .unwrap();
        assert_eq!(lazy.sections.len(), eager_chunk.sections.len());
        assert_eq!(lazy.data_version, eager_chunk.data_version);
    }
    // Absent chunk reads as None, not an error.
    assert!(reader.read_chunk(31, 31).expect("read ok").is_none());
}

#[test]
fn region_reader_auto_detects_coordinates() {
    let mca_bytes = region_file_fixture();
    let mut reader = RegionReader::new_auto(Cursor::new(mca_bytes)).expect("auto");
    assert!(reader.read_chunk(0, 0).expect("ok").is_some());
}

use nucleation::formats::world_stream::WorldSource;

/// Two-region world: blocks at cx 0..2 in r.0.0 plus one at cx=33 in r.1.0.
fn world_zip_fixture() -> Vec<u8> {
    let mut schem = UniversalSchematic::new("fixture".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());
    let gold = BlockState::new("minecraft:gold_block".to_string());
    schem.set_block(0, 64, 0, &stone);
    schem.set_block(5, 65, 5, &gold);
    schem.set_block(20, 70, 3, &stone); // cx=1, cz=0
    schem.set_block(530, 64, 7, &gold); // cx=33 -> region r.1.0
    world::to_world_zip(&schem, None).expect("to_world_zip")
}

#[test]
fn streaming_collect_equals_eager_import() {
    let zip = world_zip_fixture();
    let eager = world::from_world_zip(&zip).expect("eager");

    let source = WorldSource::from_zip_bytes(zip).expect("source");
    let mut streamed = UniversalSchematic::new("streamed".to_string());
    let mut chunk_count = 0usize;
    for chunk in source.chunks().expect("chunks") {
        let view = chunk.expect("chunk ok");
        view.load_into(&mut streamed);
        chunk_count += 1;
    }
    assert!(
        chunk_count >= 3,
        "expected at least 3 chunks, got {}",
        chunk_count
    );

    // Same non-air blocks at the same world coordinates.
    let bb = eager.default_region.get_bounding_box();
    for y in bb.min.1..=bb.max.1 {
        for z in bb.min.2..=bb.max.2 {
            for x in bb.min.0..=bb.max.0 {
                let a = eager
                    .default_region
                    .get_block(x, y, z)
                    .map(|b| b.name.clone());
                let b = streamed
                    .default_region
                    .get_block(x, y, z)
                    .map(|b| b.name.clone());
                assert_eq!(a, b, "block mismatch at ({}, {}, {})", x, y, z);
            }
        }
    }
}

#[test]
fn chunk_view_get_block_uses_world_coords() {
    let zip = world_zip_fixture();
    let source = WorldSource::from_zip_bytes(zip).expect("source");
    let mut found_gold = false;
    for chunk in source.chunks().expect("chunks") {
        let view = chunk.expect("ok");
        if view.cx() == 0 && view.cz() == 0 {
            assert_eq!(
                view.get_block(5, 65, 5).map(|b| b.name.as_str()),
                Some("minecraft:gold_block")
            );
            found_gold = true;
        }
    }
    assert!(found_gold);
}

#[test]
fn bounded_iteration_skips_outside_regions_and_chunks() {
    let zip = world_zip_fixture();
    let source = WorldSource::from_zip_bytes(zip).expect("source");
    // Box covering only chunk (0,0)
    let views: Vec<_> = source
        .chunks_bounded((0, 0, 0), (15, 255, 15))
        .expect("bounded")
        .collect::<Result<Vec<_>, _>>()
        .expect("all ok");
    assert_eq!(views.len(), 1);
    assert_eq!((views[0].cx(), views[0].cz()), (0, 0));
}

#[test]
fn iteration_order_is_canonical() {
    let zip = world_zip_fixture();
    let source = WorldSource::from_zip_bytes(zip).expect("source");
    let keys: Vec<_> = source
        .chunks()
        .expect("chunks")
        .map(|c| {
            let v = c.expect("ok");
            (v.cx(), v.cz())
        })
        .collect();
    let mut sorted = keys.clone();
    sorted.sort_by_key(|(cx, cz)| nucleation::formats::world_stream::chunk_order_key(*cx, *cz));
    assert_eq!(keys, sorted);
}

#[test]
fn corrupt_chunk_yields_error_item_and_stream_continues() {
    // Take a valid region file and truncate one chunk's payload bytes
    // mid-sector: zero out 16 bytes shortly after the first chunk's
    // 5-byte header so decompression fails for that chunk only.
    let mut mca = region_file_fixture();

    // Verify the fixture has at least 2 chunks so the "stream continues"
    // assertion below is meaningful.
    let clean = McaFile::from_bytes_auto(&mca).expect("clean parse");
    let clean_count = clean.chunks.iter().flatten().count();
    assert!(
        clean_count >= 2,
        "fixture must have >= 2 chunks (has {})",
        clean_count
    );

    // First location entry points at sector >= 2; find the first present
    // chunk's byte offset from the header table.
    let mut payload_off = None;
    for i in 0..1024usize {
        let o = i * 4;
        let loc = ((mca[o] as usize) << 16) | ((mca[o + 1] as usize) << 8) | (mca[o + 2] as usize);
        if loc >= 2 && mca[o + 3] > 0 {
            payload_off = Some(loc * 4096);
            break;
        }
    }
    let off = payload_off.expect("at least one chunk");
    for b in &mut mca[off + 5..off + 21] {
        *b = 0;
    }

    let source = WorldSource::from_mca_bytes(mca).expect("source");
    let results: Vec<_> = source.chunks().expect("chunks").collect();
    assert_eq!(
        results.iter().filter(|r| r.is_err()).count(),
        1,
        "expected exactly one corrupt-chunk error"
    );
    assert!(
        results.iter().any(|r| r.is_ok()),
        "stream must continue past the error"
    );
}

#[test]
fn directory_source_streams_chunks() {
    let mut schem = UniversalSchematic::new("dirfix".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());
    schem.set_block(0, 64, 0, &stone);
    let dir = std::env::temp_dir().join("nucleation_ws_test_dir_src");
    let _ = std::fs::remove_dir_all(&dir);
    world::save_world(&schem, &dir, None).expect("save_world");

    let source = WorldSource::open_dir(&dir).expect("open");
    let count = source
        .chunks()
        .expect("chunks")
        .filter(|c| c.is_ok())
        .count();
    assert!(count >= 1);
    let _ = std::fs::remove_dir_all(&dir);
}

// ─── Task 4: WorldSink (write/patch path) ───────────────────────────────────

use nucleation::formats::world_stream::WorldSink;

#[test]
fn sink_round_trips_streamed_chunks() {
    let zip = world_zip_fixture();
    let source = WorldSource::from_zip_bytes(zip).expect("source");

    let dir = std::env::temp_dir().join("nucleation_ws_test_sink");
    let _ = std::fs::remove_dir_all(&dir);
    let mut sink = WorldSink::create(&dir, None).expect("create");
    for chunk in source.chunks().expect("chunks") {
        sink.write_chunk(&chunk.expect("ok")).expect("write");
    }
    sink.finish().expect("finish");
    assert!(dir.join("level.dat").is_file());

    // Re-stream and compare block content via eager import equivalence.
    let original = world::from_world_zip(&world_zip_fixture()).expect("orig");
    let rewritten = world::from_world_directory(&dir).expect("reread");
    let bb = original.default_region.get_bounding_box();
    for y in bb.min.1..=bb.max.1 {
        for z in bb.min.2..=bb.max.2 {
            for x in bb.min.0..=bb.max.0 {
                assert_eq!(
                    original.default_region.get_block(x, y, z).map(|b| &b.name),
                    rewritten.default_region.get_block(x, y, z).map(|b| &b.name),
                    "mismatch at ({}, {}, {})",
                    x,
                    y,
                    z
                );
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn patch_chunk_changes_only_target_chunk() {
    let mut schem = UniversalSchematic::new("patchfix".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());
    schem.set_block(0, 64, 0, &stone);
    schem.set_block(20, 64, 0, &stone); // cx=1
    let dir = std::env::temp_dir().join("nucleation_ws_test_patch");
    let _ = std::fs::remove_dir_all(&dir);
    world::save_world(&schem, &dir, None).expect("save");

    let beacon = BlockState::new("minecraft:beacon".to_string());
    let mut sink = WorldSink::open_existing(&dir).expect("open");
    sink.patch_chunk(0, 0, |view| {
        assert!(view.set_block(0, 64, 0, &beacon));
    })
    .expect("patch");
    sink.finish().expect("finish");

    let reread = world::from_world_directory(&dir).expect("reread");
    assert_eq!(
        reread
            .default_region
            .get_block(0, 64, 0)
            .map(|b| b.name.as_str()),
        Some("minecraft:beacon")
    );
    assert_eq!(
        reread
            .default_region
            .get_block(20, 64, 0)
            .map(|b| b.name.as_str()),
        Some("minecraft:stone")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sink_out_of_order_region_writes_lose_nothing() {
    // Chunks in two regions, written A, B, A — the second A flush must not
    // discard the first A chunk.
    let mut schem = UniversalSchematic::new("ooo".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());
    schem.set_block(0, 64, 0, &stone); // region (0,0), chunk (0,0)
    schem.set_block(530, 64, 7, &stone); // region (1,0), chunk (33,0)
    schem.set_block(20, 64, 0, &stone); // region (0,0), chunk (1,0)
    let zip = world::to_world_zip(&schem, None).expect("zip");
    let source = WorldSource::from_zip_bytes(zip).expect("source");
    let views: Vec<_> = source
        .chunks()
        .expect("chunks")
        .collect::<Result<Vec<_>, _>>()
        .expect("ok");
    // Force non-canonical order: (0,0), (33,0), (1,0) — sandwiches r.1.0 between r.0.0 chunks.
    let a0 = views
        .iter()
        .position(|v| v.cx() == 0 && v.cz() == 0)
        .unwrap();
    let b = views.iter().position(|v| v.cx() == 33).unwrap();
    let a1 = views
        .iter()
        .position(|v| v.cx() == 1 && v.cz() == 0)
        .unwrap();
    let order = [a0, b, a1];

    let dir = std::env::temp_dir().join("nucleation_ws_test_ooo");
    let _ = std::fs::remove_dir_all(&dir);
    let mut sink = WorldSink::create(&dir, None).expect("create");
    for &i in &order {
        sink.write_chunk(&views[i]).expect("write");
    }
    sink.finish().expect("finish");

    let reread = world::from_world_directory(&dir).expect("reread");
    assert_eq!(
        reread
            .default_region
            .get_block(0, 64, 0)
            .map(|b| b.name.as_str()),
        Some("minecraft:stone"),
        "block at (0,64,0) missing"
    );
    assert_eq!(
        reread
            .default_region
            .get_block(20, 64, 0)
            .map(|b| b.name.as_str()),
        Some("minecraft:stone"),
        "block at (20,64,0) missing"
    );
    assert_eq!(
        reread
            .default_region
            .get_block(530, 64, 7)
            .map(|b| b.name.as_str()),
        Some("minecraft:stone"),
        "block at (530,64,7) missing"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// ─── Task 5: diff_worlds (lockstep streaming diff) ──────────────────────────

use nucleation::formats::world_stream::diff_worlds;

#[test]
fn diff_worlds_finds_chunk_deltas_and_replays() {
    // World A
    let mut a = UniversalSchematic::new("a".to_string());
    let stone = BlockState::new("minecraft:stone".to_string());
    let iron = BlockState::new("minecraft:iron_ore".to_string());
    a.set_block(0, 64, 0, &stone);
    a.set_block(3, 64, 3, &iron); // "mod ore" — only in A
    a.set_block(20, 64, 0, &stone); // cx=1, identical in both

    // World B: same minus the ore
    let mut b = UniversalSchematic::new("b".to_string());
    b.set_block(0, 64, 0, &stone);
    b.set_block(20, 64, 0, &stone);

    let zip_a = world::to_world_zip(&a, None).expect("zip a");
    let zip_b = world::to_world_zip(&b, None).expect("zip b");
    let src_a = WorldSource::from_zip_bytes(zip_a).expect("a");
    let src_b = WorldSource::from_zip_bytes(zip_b).expect("b");

    let diffs: Vec<_> = diff_worlds(&src_a, &src_b, "exact")
        .expect("diff_worlds")
        .collect::<Result<Vec<_>, _>>()
        .expect("all ok");

    // Only chunk (0,0) differs; identical chunk (1,0) is skipped.
    assert_eq!(diffs.len(), 1);
    assert_eq!((diffs[0].cx, diffs[0].cz), (0, 0));
    assert!(diffs[0].diff.distance > 0);

    // Replay: apply removals from the diff onto a copy of world A.
    let dir = std::env::temp_dir().join("nucleation_ws_test_replay");
    let _ = std::fs::remove_dir_all(&dir);
    world::save_world(&a, &dir, None).expect("save a");
    let mut sink = WorldSink::open_existing(&dir).expect("open");
    let air = BlockState::new("minecraft:air".to_string());
    for d in &diffs {
        sink.patch_chunk(d.cx, d.cz, |view| {
            // removed = blocks present in A but not B → erase them
            for (pos, _state) in &d.diff.removed {
                view.set_block(pos.0, pos.1, pos.2, &air);
            }
            for (pos, state) in &d.diff.added {
                view.set_block(pos.0, pos.1, pos.2, state);
            }
        })
        .expect("patch");
    }
    sink.finish().expect("finish");

    let result = world::from_world_directory(&dir).expect("reread");
    assert_eq!(result.default_region.get_block(3, 64, 3), None); // ore gone
    assert_eq!(
        result
            .default_region
            .get_block(0, 64, 0)
            .map(|b| b.name.as_str()),
        Some("minecraft:stone")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn diff_worlds_does_not_translate_align_shifted_chunks() {
    // Same shape, shifted one block in X within the same chunk: at absolute
    // coordinates this is one removal + one addition, NOT a zero diff.
    let stone = BlockState::new("minecraft:stone".to_string());
    let mut a = UniversalSchematic::new("a".to_string());
    a.set_block(4, 64, 4, &stone);
    let mut b = UniversalSchematic::new("b".to_string());
    b.set_block(5, 64, 4, &stone);

    let src_a = WorldSource::from_zip_bytes(world::to_world_zip(&a, None).unwrap()).unwrap();
    let src_b = WorldSource::from_zip_bytes(world::to_world_zip(&b, None).unwrap()).unwrap();
    let diffs: Vec<_> = diff_worlds(&src_a, &src_b, "exact")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(
        diffs.len(),
        1,
        "shifted content must produce a non-empty chunk diff"
    );
    let d = &diffs[0].diff;
    assert!(d.distance > 0);
    // The reported transform must stay at the identity (no translation search).
    assert_eq!(d.transform.translate, (0, 0, 0));
    // Absolute positions: stone removed at x=4, added at x=5 (allow either
    // added/removed or changed-style accounting, but positions must be absolute).
    let mentions = |x: i32| {
        d.removed
            .iter()
            .map(|(p, _)| p)
            .chain(d.added.iter().map(|(p, _)| p))
            .chain(d.changed.iter().map(|(p, _, _)| p))
            .any(|p| p.0 == x)
    };
    assert!(
        mentions(4) && mentions(5),
        "deltas must reference absolute x=4 and x=5: removed={:?} added={:?}",
        d.removed,
        d.added
    );
}

// ─── World generation from scratch (fabricated chunks) ──────────────────────

#[test]
fn generate_world_from_scratch_chunk_by_chunk() {
    use nucleation::formats::world_stream::WorldChunkView;
    let stone = BlockState::new("minecraft:stone".to_string());
    let gold = BlockState::new("minecraft:gold_block".to_string());

    let dir = std::env::temp_dir().join("nucleation_ws_test_gen");
    let _ = std::fs::remove_dir_all(&dir);
    let mut sink = WorldSink::create(&dir, None).expect("create");
    // 2x2 chunks of flat "terrain" with one marker block, constant memory.
    for cx in 0i32..2 {
        for cz in 0i32..2 {
            let mut chunk = WorldChunkView::new(cx, cz);
            for lx in 0i32..16 {
                for lz in 0i32..16 {
                    let (wx, wz) = (cx * 16 + lx, cz * 16 + lz);
                    for y in 60..=64 {
                        assert!(chunk.set_block(wx, y, wz, &stone));
                    }
                }
            }
            if (cx, cz) == (1, 1) {
                chunk.set_block(20, 65, 20, &gold);
            }
            sink.write_chunk(&chunk).expect("write");
        }
    }
    sink.finish().expect("finish");

    let world = world::from_world_directory(&dir).expect("reread");
    assert_eq!(
        world
            .default_region
            .get_block(0, 64, 0)
            .map(|b| b.name.as_str()),
        Some("minecraft:stone")
    );
    assert_eq!(
        world
            .default_region
            .get_block(31, 60, 31)
            .map(|b| b.name.as_str()),
        Some("minecraft:stone")
    );
    assert_eq!(
        world
            .default_region
            .get_block(20, 65, 20)
            .map(|b| b.name.as_str()),
        Some("minecraft:gold_block")
    );
    // Nothing was placed above the terrain in chunk (0, 0): in-bounds unset
    // positions read back as air (region palette default).
    let above = world
        .default_region
        .get_block(0, 65, 0)
        .map(|b| b.name.as_str());
    assert!(above.is_none() || above == Some("minecraft:air"));

    // A fabricated chunk also streams back out with correct coords.
    let source = WorldSource::open_dir(&dir).expect("open");
    let count = source
        .chunks()
        .expect("chunks")
        .filter(|c| c.is_ok())
        .count();
    assert_eq!(count, 4);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn biome_set_and_default_round_trip() {
    use nucleation::formats::world_stream::{WorldChunkView, WorldSink};
    let stone = BlockState::new("minecraft:stone".to_string());
    let dir = std::env::temp_dir().join("nucleation_ws_test_biome");
    let _ = std::fs::remove_dir_all(&dir);
    // Chunk 0,0: explicit desert. Chunk 1,0: no set_biome -> world default (jungle).
    let opts = r#"{"biome": "minecraft:jungle"}"#;
    let options: world::WorldExportOptions = serde_json::from_str(opts).unwrap();
    let mut sink = WorldSink::create(&dir, Some(options)).expect("create");
    let mut c0 = WorldChunkView::new(0, 0);
    c0.set_block(0, 64, 0, &stone);
    c0.set_biome("minecraft:desert");
    sink.write_chunk(&c0).expect("write");
    let mut c1 = WorldChunkView::new(1, 0);
    c1.set_block(16, 64, 0, &stone);
    sink.write_chunk(&c1).expect("write");
    sink.finish().expect("finish");

    let source = WorldSource::open_dir(&dir).expect("open");
    let mut saw = std::collections::HashMap::new();
    for chunk in source.chunks().expect("chunks") {
        let v = chunk.expect("ok");
        saw.insert((v.cx(), v.cz()), v.biome_palette());
    }
    assert_eq!(saw[&(0, 0)], vec!["minecraft:desert".to_string()]);
    assert_eq!(saw[&(1, 0)], vec!["minecraft:jungle".to_string()]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn patch_chunk_preserves_biomes() {
    use nucleation::formats::world_stream::{WorldChunkView, WorldSink};
    let stone = BlockState::new("minecraft:stone".to_string());
    let beacon = BlockState::new("minecraft:beacon".to_string());
    let dir = std::env::temp_dir().join("nucleation_ws_test_biome_patch");
    let _ = std::fs::remove_dir_all(&dir);
    let mut sink = WorldSink::create(&dir, None).expect("create");
    let mut c = WorldChunkView::new(0, 0);
    c.set_block(0, 64, 0, &stone);
    c.set_biome("minecraft:desert");
    sink.write_chunk(&c).expect("write");
    sink.finish().expect("finish");

    // Patch one block; biome must survive (this used to reset to plains).
    let mut sink = WorldSink::open_existing(&dir).expect("open");
    sink.patch_chunk(0, 0, |view| {
        view.set_block(1, 64, 1, &beacon);
    })
    .expect("patch");
    sink.finish().expect("finish");

    let source = WorldSource::open_dir(&dir).expect("open");
    let v = source
        .chunks()
        .expect("chunks")
        .next()
        .unwrap()
        .expect("ok");
    assert_eq!(v.biome_palette(), vec!["minecraft:desert".to_string()]);
    assert_eq!(
        v.get_block(1, 64, 1).map(|b| b.name.as_str()),
        Some("minecraft:beacon")
    );
    let _ = std::fs::remove_dir_all(&dir);
}
