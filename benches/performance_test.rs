// benches/performance_test
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nucleation::{BlockState, Region};
use std::collections::HashMap;

fn fill_cube_no_cache_small_region(size: i32) {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 1, 1));

    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                let stone = BlockState::new("minecraft:stone".to_string());
                region.set_block(x, y, z, stone);
            }
        }
    }
}

fn fill_cube_with_cache_small_region(size: i32) {
    let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 1, 1));
    let stone = BlockState::new("minecraft:stone".to_string());

    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                region.set_block(x, y, z, stone.clone());
            }
        }
    }
}

fn fill_cube_with_cache_large_region(size: i32) {
    let margin = 16;
    let mut region = Region::new(
        "Test".to_string(),
        (-margin, -margin, -margin),
        (size + 2 * margin, size + 2 * margin, size + 2 * margin),
    );
    let stone = BlockState::new("minecraft:stone".to_string());

    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                region.set_block(x, y, z, stone.clone());
            }
        }
    }
}

fn fill_cube_sparse_hashmap(size: i32) {
    let mut blocks: HashMap<(i32, i32, i32), usize> = HashMap::new();
    let stone_id = 1;

    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                blocks.insert((x, y, z), stone_id);
            }
        }
    }
    black_box(blocks);
}

fn bench_nucleation_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("nucleation_performance");

    for size in [5, 10, 15].iter() {
        group.bench_with_input(
            BenchmarkId::new("no_cache_small_region", size),
            size,
            |b, &size| {
                b.iter(|| fill_cube_no_cache_small_region(black_box(size)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_cache_small_region", size),
            size,
            |b, &size| {
                b.iter(|| fill_cube_with_cache_small_region(black_box(size)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_cache_large_region", size),
            size,
            |b, &size| {
                b.iter(|| fill_cube_with_cache_large_region(black_box(size)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sparse_hashmap_python_style", size),
            size,
            |b, &size| {
                b.iter(|| fill_cube_sparse_hashmap(black_box(size)));
            },
        );
    }

    group.finish();
}

fn bench_block_state_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_state_creation");

    group.bench_function("create_1000_new_block_states", |b| {
        b.iter(|| {
            let mut states = Vec::new();
            for _i in 0..1000 {
                states.push(BlockState::new("minecraft:stone".to_string()));
            }
            black_box(states);
        });
    });

    group.bench_function("clone_1_block_state_1000_times", |b| {
        b.iter(|| {
            let stone = BlockState::new("minecraft:stone".to_string());
            let mut states = Vec::new();
            for _i in 0..1000 {
                states.push(stone.clone());
            }
            black_box(states);
        });
    });

    group.finish();
}

fn bench_region_expansion(c: &mut Criterion) {
    let mut group = c.benchmark_group("region_expansion");

    group.bench_function("frequent_expansions", |b| {
        b.iter(|| {
            let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 1, 1));
            let stone = BlockState::new("minecraft:stone".to_string());

            for i in 0..20 {
                region.set_block(i, i, i, stone.clone());
            }
            black_box(region);
        });
    });

    group.bench_function("no_expansions_needed", |b| {
        b.iter(|| {
            let mut region = Region::new("Test".to_string(), (0, 0, 0), (30, 30, 30));
            let stone = BlockState::new("minecraft:stone".to_string());

            for i in 0..20 {
                region.set_block(i, i, i, stone.clone());
            }
            black_box(region);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_nucleation_strategies,
    bench_block_state_creation,
    bench_region_expansion
);
criterion_main!(benches);
