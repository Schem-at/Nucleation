//! Regression bench for the block-placement hot paths.
//!
//! Runs each scenario against `UniversalSchematic` directly (no Python/JS
//! boundary) to isolate the core engine's throughput. Numbers here are the
//! ceiling that the language wrappers can approach.
//!
//! Run with: `cargo bench --bench block_placement`
//! Quick mode:  `cargo bench --bench block_placement -- --quick`

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use nucleation::UniversalSchematic;

const N: usize = 100_000;

/// A deterministic permutation of a 64-cubed volume. This has random-looking
/// locality without putting entropy generation inside the timed section, and
/// the odd multiplier guarantees that the first 2^18 indices are unique.
fn sparse_positions(count: usize) -> Vec<(i32, i32, i32)> {
    assert!(count <= 64 * 64 * 64);
    (0..count)
        .map(|i| {
            let shuffled = (i * 73 + 19) & ((64 * 64 * 64) - 1);
            (
                (shuffled & 63) as i32,
                ((shuffled >> 12) & 63) as i32,
                ((shuffled >> 6) & 63) as i32,
            )
        })
        .collect()
}

fn bench_set_block_plain(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_block_plain");
    group.throughput(Throughput::Elements(N as u64));
    group.bench_function("per_call", |b| {
        b.iter(|| {
            let mut s = UniversalSchematic::new("bench".into());
            for i in 0..N as i32 {
                s.set_block_str(black_box(i), 0, 0, "minecraft:stone");
            }
        })
    });
    group.finish();
}

fn bench_set_block_complex_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_block_complex_state");
    group.throughput(Throughput::Elements(N as u64));
    group.bench_function("per_call_set_block_from_string", |b| {
        b.iter(|| {
            let mut s = UniversalSchematic::new("bench".into());
            for i in 0..N as i32 {
                let _ = s.set_block_from_string(
                    black_box(i),
                    0,
                    0,
                    "minecraft:repeater[delay=4,facing=east]",
                );
            }
        })
    });
    group.finish();
}

fn bench_set_block_chest_per_call(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_block_chest_nbt");
    group.throughput(Throughput::Elements(N as u64));
    group.bench_function("per_call_set_block_from_string", |b| {
        b.iter(|| {
            let mut s = UniversalSchematic::new("bench".into());
            let chest = "minecraft:chest[facing=west]\
                         {Items:[{Slot:0b,id:\"minecraft:diamond\",Count:64b}]}";
            for i in 0..N as i32 {
                let _ = s.set_block_from_string(black_box(i), 0, 0, chest);
            }
        })
    });
    group.finish();
}

fn bench_realistic_sparse_placement(c: &mut Criterion) {
    const COUNT: usize = 10_000;
    const REGION_COUNT: usize = 32;
    let positions = sparse_positions(COUNT);
    let palette = [
        "minecraft:stone",
        "minecraft:oak_planks",
        "minecraft:redstone_wire[power=0,north=none,east=none,south=none,west=none]",
        "minecraft:repeater[delay=2,facing=east,locked=false,powered=false]",
        "minecraft:comparator[facing=north,mode=compare,powered=false]",
    ];

    // Smoke the workload before Criterion can report a plausible number for
    // an accidentally empty or duplicate-heavy scenario.
    let mut validation = UniversalSchematic::new("validation".into());
    for (i, &(x, y, z)) in positions.iter().enumerate() {
        assert!(validation.set_block_str(x, y, z, palette[i % palette.len()]));
    }
    assert_eq!(validation.total_blocks(), COUNT as i32);

    let mut group = c.benchmark_group("realistic_sparse_placement");
    group.throughput(Throughput::Elements(COUNT as u64));
    group.bench_function("mixed_palette_default_region_10k", |b| {
        b.iter(|| {
            let mut schematic = UniversalSchematic::new("bench".into());
            for (i, &(x, y, z)) in positions.iter().enumerate() {
                assert!(schematic.set_block_str(
                    black_box(x),
                    black_box(y),
                    black_box(z),
                    palette[i % palette.len()],
                ));
            }
            black_box(schematic);
        })
    });

    // Region names and coordinates are prepared outside the timer, but region
    // creation and all individual writes are included. Each region receives a
    // random-looking 16-cubed working set, like a multi-build editor session.
    let region_names: Vec<String> = (0..REGION_COUNT).map(|i| format!("build_{i:02}")).collect();
    let region_positions: Vec<(i32, i32, i32)> = (0..COUNT)
        .map(|i| {
            let local_i = i / REGION_COUNT;
            let shuffled = (local_i * 73 + 19) & ((16 * 16 * 16) - 1);
            (
                (shuffled & 15) as i32,
                ((shuffled >> 8) & 15) as i32,
                ((shuffled >> 4) & 15) as i32,
            )
        })
        .collect();

    group.bench_function("mixed_palette_32_named_regions_10k", |b| {
        b.iter(|| {
            let mut schematic = UniversalSchematic::new("bench".into());
            for name in &region_names {
                schematic
                    .create_schematic_region(name)
                    .expect("unique benchmark region");
            }
            for (i, &(x, y, z)) in region_positions.iter().enumerate() {
                assert!(schematic.set_block_in_region_str(
                    &region_names[i % REGION_COUNT],
                    black_box(x),
                    black_box(y),
                    black_box(z),
                    palette[i % palette.len()],
                ));
            }
            black_box(schematic);
        })
    });
    group.finish();
}

fn bench_content_shorthands(c: &mut Criterion) {
    const COUNT: usize = 5_000;
    let positions = sparse_positions(COUNT);
    let barrels: Vec<String> = (1..=15)
        .map(|signal| format!("minecraft:barrel[facing=up]{{signal={signal}}}"))
        .collect();
    let jukeboxes = [
        "minecraft:jukebox{record=pigstep}",
        "minecraft:jukebox{record=cat}",
        "minecraft:jukebox{record=blocks}",
        "minecraft:jukebox{record=chirp}",
    ];

    let mut validation = UniversalSchematic::new("validation".into());
    assert!(validation
        .set_block_from_string(0, 0, 0, &barrels[12])
        .expect("barrel signal shorthand parses"));
    assert!(validation
        .set_block_from_string(1, 0, 0, jukeboxes[0])
        .expect("jukebox record shorthand parses"));
    assert_eq!(validation.total_blocks(), 2);
    assert_eq!(validation.default_region.block_entities.len(), 2);

    let mut group = c.benchmark_group("content_shorthands");
    group.throughput(Throughput::Elements(COUNT as u64));
    group.bench_function("barrel_signal_5k", |b| {
        b.iter(|| {
            let mut schematic = UniversalSchematic::new("bench".into());
            for (i, &(x, y, z)) in positions.iter().enumerate() {
                assert!(schematic
                    .set_block_from_string(
                        black_box(x),
                        black_box(y),
                        black_box(z),
                        &barrels[i % barrels.len()],
                    )
                    .expect("validated barrel descriptor"));
            }
            black_box(schematic);
        })
    });

    group.bench_function("jukebox_record_5k", |b| {
        b.iter(|| {
            let mut schematic = UniversalSchematic::new("bench".into());
            for (i, &(x, y, z)) in positions.iter().enumerate() {
                assert!(schematic
                    .set_block_from_string(
                        black_box(x),
                        black_box(y),
                        black_box(z),
                        jukeboxes[i % jukeboxes.len()],
                    )
                    .expect("validated jukebox descriptor"));
            }
            black_box(schematic);
        })
    });

    // A common editor action: replace content-bearing blocks with ordinary
    // blocks. This must pay for block-entity removal and must not leave stale
    // inventory or RecordItem data behind.
    group.throughput(Throughput::Elements((COUNT * 2) as u64));
    group.bench_function(
        "barrel_then_plain_replacement_5k_positions_10k_writes",
        |b| {
            b.iter(|| {
                let mut schematic = UniversalSchematic::new("bench".into());
                for (i, &(x, y, z)) in positions.iter().enumerate() {
                    assert!(schematic
                        .set_block_from_string(x, y, z, &barrels[i % barrels.len()])
                        .expect("validated barrel descriptor"));
                }
                for &(x, y, z) in &positions {
                    assert!(schematic.set_block_str(
                        black_box(x),
                        black_box(y),
                        black_box(z),
                        "minecraft:stone",
                    ));
                }
                assert!(schematic.default_region.block_entities.is_empty());
                black_box(schematic);
            })
        },
    );
    group.finish();
}

fn bench_axis_aligned_run(c: &mut Criterion) {
    // Manual `fill` analog at the core API level — placing the same plain
    // block across N positions to set the engine's per-block ceiling.
    let mut group = c.benchmark_group("axis_aligned_run");
    group.throughput(Throughput::Elements(N as u64));
    group.bench_function("set_block_str_loop", |b| {
        b.iter(|| {
            let mut s = UniversalSchematic::new("bench".into());
            for i in 0..N as i32 {
                s.set_block_str(black_box(i), 0, 0, "minecraft:stone");
            }
        })
    });
    group.finish();
}

fn bench_cuboid_fill_and_export(c: &mut Criterion) {
    const EDGE: i32 = 64;
    let volume = (EDGE as u64).pow(3);

    let mut fill = c.benchmark_group("cuboid_fill");
    fill.throughput(Throughput::Elements(volume));
    fill.bench_function("fill_uniform_64cubed", |b| {
        b.iter(|| {
            let mut schematic = UniversalSchematic::new("bench".into());
            schematic.fill_cuboid_str((0, 0, 0), (EDGE - 1, EDGE - 1, EDGE - 1), "minecraft:stone");
            black_box(schematic);
        })
    });
    fill.finish();

    let mut schematic = UniversalSchematic::new("bench".into());
    schematic.fill_cuboid_str((0, 0, 0), (31, 31, 31), "minecraft:stone");
    let mut export = c.benchmark_group("schematic_export");
    export.throughput(Throughput::Elements(32_u64.pow(3)));
    export.bench_function("compact_32cubed", |b| {
        b.iter(|| {
            let bytes = nucleation::schematic::to_schematic(black_box(&schematic)).unwrap();
            black_box(bytes);
        })
    });
    export.finish();
}

fn bench_clone_block_entity(c: &mut Criterion) {
    // Direct micro-bench of the deep clone cost we're targeting.
    use nucleation::block_entity::BlockEntity;
    use nucleation::utils::NbtValue;

    let mut proto = BlockEntity::new("minecraft:chest".to_string(), (0, 0, 0));
    let item = {
        let mut m = nucleation::nbt::NbtMap::new();
        m.insert("Slot".to_string(), NbtValue::Byte(0));
        m.insert(
            "id".to_string(),
            NbtValue::String("minecraft:diamond".into()),
        );
        m.insert("Count".to_string(), NbtValue::Byte(64));
        NbtValue::Compound(m)
    };
    proto = proto.with_nbt_data("Items".to_string(), NbtValue::List(vec![item]));

    let mut group = c.benchmark_group("block_entity");
    group.throughput(Throughput::Elements(1));
    group.bench_function("clone_chest_with_one_item", |b| {
        b.iter(|| {
            let cloned = black_box(proto.clone());
            black_box(cloned);
        })
    });
    group.finish();
}

fn bench_transform_with_block_entities(c: &mut Criterion) {
    // Simulates a "schematic with N chests, then transform" workflow —
    // every transform clones every block entity into a fresh map. The
    // Arc-shared NBT change should make this dramatically cheaper.
    let chest =
        "minecraft:chest[facing=west]{Items:[{Slot:0b,id:\"minecraft:diamond\",Count:64b}]}";
    let n: usize = 5_000;

    let mut group = c.benchmark_group("transform_with_chests");
    group.throughput(Throughput::Elements(n as u64));

    group.bench_function("flip_x_5k_chests", |b| {
        let mut s = UniversalSchematic::new("bench".into());
        for i in 0..n as i32 {
            let _ = s.set_block_from_string(i, 0, 0, chest);
        }
        b.iter_batched(
            || s.clone(),
            |mut schem| {
                schem.flip_x();
                black_box(schem);
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("rotate_y_5k_chests", |b| {
        let mut s = UniversalSchematic::new("bench".into());
        for i in 0..n as i32 {
            let _ = s.set_block_from_string(i, 0, 0, chest);
        }
        b.iter_batched(
            || s.clone(),
            |mut schem| {
                schem.rotate_y(90).expect("90 is a valid quarter-turn");
                black_box(schem);
            },
            criterion::BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_clone_schematic_with_chests(c: &mut Criterion) {
    // Cloning a schematic clones every BlockEntity. With Arc'd NBT this
    // should be dramatically cheaper than baseline.
    let chest =
        "minecraft:chest[facing=west]{Items:[{Slot:0b,id:\"minecraft:diamond\",Count:64b}]}";
    let n: usize = 10_000;
    let mut s = UniversalSchematic::new("bench".into());
    for i in 0..n as i32 {
        let _ = s.set_block_from_string(i, 0, 0, chest);
    }

    let mut group = c.benchmark_group("schematic_clone");
    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("clone_10k_chests", |b| {
        b.iter(|| {
            let cloned = black_box(s.clone());
            black_box(cloned);
        })
    });
    group.finish();
}

fn bench_chest_batch_components(c: &mut Criterion) {
    // Profile sub-phases of the chest-batch hot path to identify the
    // remaining floor.
    use nucleation::block_entity::BlockEntity;
    use nucleation::block_entity_store::BlockEntityStore;
    use nucleation::utils::NbtValue;
    use std::sync::Arc;

    const N: usize = 100_000;

    let positions: Vec<(i32, i32, i32)> = (0..N as i32).map(|i| (i, 0, 0)).collect();
    let template = {
        let mut be = BlockEntity::new("minecraft:chest".to_string(), (0, 0, 0));
        let item = {
            let mut m = nucleation::nbt::NbtMap::new();
            m.insert("Slot".to_string(), NbtValue::Byte(0));
            m.insert(
                "id".to_string(),
                NbtValue::String("minecraft:diamond".into()),
            );
            m.insert("Count".to_string(), NbtValue::Byte(64));
            NbtValue::Compound(m)
        };
        be = be.with_nbt_data("Items".to_string(), NbtValue::List(vec![item]));
        Arc::new(be)
    };

    let mut group = c.benchmark_group("chest_batch_phases");
    group.throughput(Throughput::Elements(N as u64));

    group.bench_function("store_insert_template_only", |b| {
        b.iter_batched(
            || (BlockEntityStore::default(), template.clone()),
            |(mut store, tpl)| {
                store.insert_template(black_box(&positions), tpl);
                black_box(store);
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("set_blocks_full_batch_chest", |b| {
        b.iter(|| {
            let mut s = UniversalSchematic::new("bench".into());
            let chest =
                "minecraft:chest[facing=west]{Items:[{Slot:0b,id:\"minecraft:diamond\",Count:64b}]}";
            // Use UniversalSchematic's parse-once batch at the core API
            // by going through set_block_from_string repeatedly.
            for &(x, y, z) in &positions {
                let _ = s.set_block_from_string(black_box(x), y, z, chest);
            }
        })
    });

    group.finish();
}

fn bench_copy_region(c: &mut Criterion) {
    // Palette-mapped fast path (single-region source) vs the per-block
    // set_block slow path (forced by adding a second source region).
    use nucleation::BoundingBox;

    const SIZE: i32 = 64;
    let n = (SIZE * SIZE * SIZE) as u64;

    let build_source = |extra_region: bool| {
        let mut src = UniversalSchematic::new("src".into());
        for y in 0..SIZE {
            for z in 0..SIZE {
                for x in 0..SIZE {
                    let block = if (x + y + z) % 7 == 0 {
                        "minecraft:stone"
                    } else if (x + y + z) % 3 == 0 {
                        "minecraft:dirt"
                    } else {
                        "minecraft:oak_planks"
                    };
                    src.set_block_str(x, y, z, block);
                }
            }
        }
        if extra_region {
            src.set_block_in_region_str("Extra", 1000, 1000, 1000, "minecraft:gold_block");
        }
        src
    };

    let bounds = BoundingBox::new((0, 0, 0), (SIZE - 1, SIZE - 1, SIZE - 1));

    let mut group = c.benchmark_group("copy_region_64cubed");
    group.throughput(Throughput::Elements(n));
    group.sample_size(10);

    let single = build_source(false);
    group.bench_function("fast_path_palette_mapped", |b| {
        b.iter(|| {
            let mut target = UniversalSchematic::new("target".into());
            target
                .copy_region(black_box(&single), &bounds, (0, 0, 0), &[])
                .unwrap();
            black_box(target);
        })
    });

    let multi = build_source(true);
    group.bench_function("slow_path_per_block", |b| {
        b.iter(|| {
            let mut target = UniversalSchematic::new("target".into());
            target
                .copy_region(black_box(&multi), &bounds, (0, 0, 0), &[])
                .unwrap();
            black_box(target);
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_set_block_plain,
    bench_set_block_complex_state,
    bench_set_block_chest_per_call,
    bench_realistic_sparse_placement,
    bench_content_shorthands,
    bench_axis_aligned_run,
    bench_cuboid_fill_and_export,
    bench_clone_block_entity,
    bench_transform_with_block_entities,
    bench_clone_schematic_with_chests,
    bench_chest_batch_components,
    bench_copy_region,
);
criterion_main!(benches);
