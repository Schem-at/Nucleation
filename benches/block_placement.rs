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
                schem.rotate_y(90);
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

criterion_group!(
    benches,
    bench_set_block_plain,
    bench_set_block_complex_state,
    bench_set_block_chest_per_call,
    bench_axis_aligned_run,
    bench_clone_block_entity,
    bench_transform_with_block_entities,
    bench_clone_schematic_with_chests,
    bench_chest_batch_components,
);
criterion_main!(benches);
