use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nucleation::{BlockState, Region};
use std::time::Duration;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_region_solid(size: i32) -> Region {
    let mut r = Region::new("bench".to_string(), (0, 0, 0), (size, size, size));
    let stone = BlockState::new("minecraft:stone".to_string());
    for y in 0..size {
        for z in 0..size {
            for x in 0..size {
                r.set_block(x, y, z, &stone);
            }
        }
    }
    r
}

fn make_region_sparse(size: i32, pct: f64) -> Region {
    let mut r = Region::new("bench".to_string(), (0, 0, 0), (size, size, size));
    let stone = BlockState::new("minecraft:stone".to_string());
    let threshold = (pct * 100.0) as i32;
    let mut counter = 0i32;
    for y in 0..size {
        for z in 0..size {
            for x in 0..size {
                counter = counter.wrapping_mul(1103515245).wrapping_add(12345);
                if (counter.unsigned_abs() % 100) < threshold as u32 {
                    r.set_block(x, y, z, &stone);
                }
            }
        }
    }
    r
}

// ── Benchmarks ───────────────────────────────────────────────────────────────

fn bench_set_block(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_block");
    group.measurement_time(Duration::from_secs(3));

    for &size in &[16, 32] {
        group.bench_function(&format!("{}_solid", size), |b| {
            b.iter(|| {
                let mut r = Region::new("bench".to_string(), (0, 0, 0), (size, size, size));
                let stone = BlockState::new("minecraft:stone".to_string());
                for y in 0..size {
                    for z in 0..size {
                        for x in 0..size {
                            r.set_block(x, y, z, &stone);
                        }
                    }
                }
                black_box(r);
            });
        });
    }
    group.finish();
}

fn bench_get_block(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_block");
    group.measurement_time(Duration::from_secs(3));

    for &size in &[16, 32] {
        let region = make_region_solid(size);
        group.bench_function(&format!("{}_solid", size), |b| {
            b.iter(|| {
                let mut sum = 0usize;
                for y in 0..size {
                    for z in 0..size {
                        for x in 0..size {
                            if region.get_block(x, y, z).is_some() {
                                sum += 1;
                            }
                        }
                    }
                }
                black_box(sum);
            });
        });
    }
    group.finish();
}

fn bench_coords_to_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("coords_to_index");
    group.measurement_time(Duration::from_secs(2));

    let region = make_region_solid(32);
    group.bench_function("32_1M", |b| {
        b.iter(|| {
            let mut sum = 0usize;
            for _ in 0..1_000_000 {
                sum += region.coords_to_index(16, 16, 16);
            }
            black_box(sum);
        });
    });
    group.finish();
}

fn bench_flips(c: &mut Criterion) {
    let mut group = c.benchmark_group("flip");
    group.measurement_time(Duration::from_secs(3));

    for &size in &[16, 32] {
        let region = make_region_solid(size);

        group.bench_function(&format!("x_{}", size), |b| {
            b.iter_batched(
                || region.clone(),
                |mut r| {
                    r.flip_x();
                    black_box(r);
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_function(&format!("y_{}", size), |b| {
            b.iter_batched(
                || region.clone(),
                |mut r| {
                    r.flip_y();
                    black_box(r);
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_function(&format!("z_{}", size), |b| {
            b.iter_batched(
                || region.clone(),
                |mut r| {
                    r.flip_z();
                    black_box(r);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_rotate_y(c: &mut Criterion) {
    let mut group = c.benchmark_group("rotate_y_90");
    group.measurement_time(Duration::from_secs(3));

    for &size in &[16, 32] {
        let region = make_region_solid(size);
        group.bench_function(&format!("{}", size), |b| {
            b.iter_batched(
                || region.clone(),
                |mut r| {
                    r.rotate_y(90);
                    black_box(r);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_to_compact(c: &mut Criterion) {
    let mut group = c.benchmark_group("to_compact");
    group.measurement_time(Duration::from_secs(3));

    for &size in &[32] {
        let region_sparse = make_region_sparse(size, 0.1);
        group.bench_function(&format!("{}_sparse10", size), |b| {
            b.iter(|| black_box(region_sparse.to_compact()));
        });

        let region_half = make_region_sparse(size, 0.5);
        group.bench_function(&format!("{}_half", size), |b| {
            b.iter(|| black_box(region_half.to_compact()));
        });
    }
    group.finish();
}

fn bench_expand_to_fit(c: &mut Criterion) {
    let mut group = c.benchmark_group("expand_to_fit");
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("16_to_32", |b| {
        b.iter_batched(
            || make_region_solid(16),
            |mut r| {
                r.expand_to_fit(31, 31, 31);
                black_box(r);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn bench_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge");
    group.measurement_time(Duration::from_secs(3));

    let size = 16;
    let r1 = make_region_solid(size);
    let mut r2 = Region::new("bench2".to_string(), (size, 0, 0), (size, size, size));
    let stone = BlockState::new("minecraft:stone".to_string());
    for y in 0..size {
        for z in 0..size {
            for x in size..size * 2 {
                r2.set_block(x, y, z, &stone);
            }
        }
    }

    group.bench_function("16", |b| {
        b.iter_batched(
            || (r1.clone(), r2.clone()),
            |(mut a, b)| {
                a.merge(&b);
                black_box(a);
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn bench_count_blocks(c: &mut Criterion) {
    let mut group = c.benchmark_group("count_blocks");
    group.measurement_time(Duration::from_secs(2));

    let solid = make_region_solid(32);
    group.bench_function("32_solid", |b| {
        b.iter(|| black_box(solid.count_blocks()));
    });
    group.finish();
}

fn bench_is_empty(c: &mut Criterion) {
    let mut group = c.benchmark_group("is_empty");
    group.measurement_time(Duration::from_secs(2));

    let empty = Region::new("bench".to_string(), (0, 0, 0), (32, 32, 32));
    group.bench_function("32_empty", |b| {
        b.iter(|| black_box(empty.is_empty()));
    });

    let non_empty = make_region_solid(32);
    group.bench_function("32_non_empty", |b| {
        b.iter(|| black_box(non_empty.is_empty()));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_set_block,
    bench_get_block,
    bench_coords_to_index,
    bench_flips,
    bench_rotate_y,
    bench_to_compact,
    bench_expand_to_fit,
    bench_merge,
    bench_count_blocks,
    bench_is_empty,
);
criterion_main!(benches);
