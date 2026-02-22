use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nucleation::formats::litematic::{from_litematic, to_litematic};
use nucleation::formats::schematic::{from_schematic, to_schematic};
use nucleation::formats::snapshot::{from_snapshot, to_snapshot};
use nucleation::{BlockState, Region, UniversalSchematic};
use std::time::Duration;

fn make_solid_schematic(size: i32) -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new("Bench".to_string());
    let region = Region::new("Main".to_string(), (0, 0, 0), (size, size, size));
    schematic.add_region(region);
    let stone = BlockState::new("minecraft:stone".to_string());
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                schematic.set_block(x, y, z, &stone);
            }
        }
    }
    schematic
}

fn make_sparse_schematic(size: i32, fill_pct: f64) -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new("Bench".to_string());
    let region = Region::new("Main".to_string(), (0, 0, 0), (size, size, size));
    schematic.add_region(region);
    let stone = BlockState::new("minecraft:stone".to_string());
    let threshold = (fill_pct * u32::MAX as f64) as u32;
    // Deterministic pseudo-random fill using a simple LCG
    let mut rng: u32 = 12345;
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                if rng < threshold {
                    schematic.set_block(x, y, z, &stone);
                }
            }
        }
    }
    schematic
}

fn make_diverse_schematic(size: i32) -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new("Bench".to_string());
    let region = Region::new("Main".to_string(), (0, 0, 0), (size, size, size));
    schematic.add_region(region);
    let blocks: Vec<BlockState> = [
        "minecraft:stone",
        "minecraft:dirt",
        "minecraft:grass_block",
        "minecraft:cobblestone",
        "minecraft:oak_planks",
        "minecraft:spruce_planks",
        "minecraft:birch_planks",
        "minecraft:sand",
        "minecraft:gravel",
        "minecraft:gold_ore",
        "minecraft:iron_ore",
        "minecraft:coal_ore",
        "minecraft:oak_log",
        "minecraft:spruce_log",
        "minecraft:glass",
        "minecraft:lapis_ore",
        "minecraft:sandstone",
        "minecraft:white_wool",
        "minecraft:bricks",
        "minecraft:bookshelf",
    ]
    .iter()
    .map(|n| BlockState::new(n.to_string()))
    .collect();

    let mut idx = 0usize;
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                schematic.set_block(x, y, z, &blocks[idx % blocks.len()]);
                idx += 1;
            }
        }
    }
    schematic
}

fn bench_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize");
    group.measurement_time(Duration::from_secs(3));

    for &size in &[32, 64] {
        let schematic = make_solid_schematic(size);
        let label = format!("{}³_solid", size);

        group.bench_with_input(BenchmarkId::new("snapshot", &label), &schematic, |b, s| {
            b.iter(|| to_snapshot(black_box(s)).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("schematic", &label), &schematic, |b, s| {
            b.iter(|| to_schematic(black_box(s)).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("litematic", &label), &schematic, |b, s| {
            b.iter(|| to_litematic(black_box(s)).unwrap())
        });
    }

    // 128³ solid
    let big = make_solid_schematic(128);
    group.bench_with_input(BenchmarkId::new("snapshot", "128³_solid"), &big, |b, s| {
        b.iter(|| to_snapshot(black_box(s)).unwrap())
    });
    group.bench_with_input(
        BenchmarkId::new("schematic", "128³_solid"),
        &big,
        |b, s| b.iter(|| to_schematic(black_box(s)).unwrap()),
    );
    group.bench_with_input(
        BenchmarkId::new("litematic", "128³_solid"),
        &big,
        |b, s| b.iter(|| to_litematic(black_box(s)).unwrap()),
    );

    // Sparse — different compression and varint characteristics
    for &pct in &[0.1, 0.5] {
        let label = format!("64³_{}pct", (pct * 100.0) as u32);
        let schematic = make_sparse_schematic(64, pct);
        group.bench_with_input(BenchmarkId::new("snapshot", &label), &schematic, |b, s| {
            b.iter(|| to_snapshot(black_box(s)).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("schematic", &label), &schematic, |b, s| {
            b.iter(|| to_schematic(black_box(s)).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("litematic", &label), &schematic, |b, s| {
            b.iter(|| to_litematic(black_box(s)).unwrap())
        });
    }

    // Diverse palette
    let diverse = make_diverse_schematic(32);
    group.bench_with_input(
        BenchmarkId::new("snapshot", "32³_diverse"),
        &diverse,
        |b, s| b.iter(|| to_snapshot(black_box(s)).unwrap()),
    );
    group.bench_with_input(
        BenchmarkId::new("schematic", "32³_diverse"),
        &diverse,
        |b, s| b.iter(|| to_schematic(black_box(s)).unwrap()),
    );
    group.bench_with_input(
        BenchmarkId::new("litematic", "32³_diverse"),
        &diverse,
        |b, s| b.iter(|| to_litematic(black_box(s)).unwrap()),
    );

    group.finish();
}

fn bench_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize");
    group.measurement_time(Duration::from_secs(3));

    for &size in &[32, 64] {
        let schematic = make_solid_schematic(size);
        let snap_bytes = to_snapshot(&schematic).unwrap();
        let schem_bytes = to_schematic(&schematic).unwrap();
        let lit_bytes = to_litematic(&schematic).unwrap();
        let label = format!("{}³_solid", size);

        group.bench_with_input(
            BenchmarkId::new("snapshot", &label),
            &snap_bytes,
            |b, data| b.iter(|| from_snapshot(black_box(data)).unwrap()),
        );
        group.bench_with_input(
            BenchmarkId::new("schematic", &label),
            &schem_bytes,
            |b, data| b.iter(|| from_schematic(black_box(data)).unwrap()),
        );
        group.bench_with_input(
            BenchmarkId::new("litematic", &label),
            &lit_bytes,
            |b, data| b.iter(|| from_litematic(black_box(data)).unwrap()),
        );
    }

    // 128³ solid
    let big = make_solid_schematic(128);
    let snap_bytes = to_snapshot(&big).unwrap();
    let schem_bytes = to_schematic(&big).unwrap();
    let lit_bytes = to_litematic(&big).unwrap();
    group.bench_with_input(
        BenchmarkId::new("snapshot", "128³_solid"),
        &snap_bytes,
        |b, data| b.iter(|| from_snapshot(black_box(data)).unwrap()),
    );
    group.bench_with_input(
        BenchmarkId::new("schematic", "128³_solid"),
        &schem_bytes,
        |b, data| b.iter(|| from_schematic(black_box(data)).unwrap()),
    );
    group.bench_with_input(
        BenchmarkId::new("litematic", "128³_solid"),
        &lit_bytes,
        |b, data| b.iter(|| from_litematic(black_box(data)).unwrap()),
    );

    // Sparse
    for &pct in &[0.1, 0.5] {
        let label = format!("64³_{}pct", (pct * 100.0) as u32);
        let schematic = make_sparse_schematic(64, pct);
        let snap_bytes = to_snapshot(&schematic).unwrap();
        let schem_bytes = to_schematic(&schematic).unwrap();
        let lit_bytes = to_litematic(&schematic).unwrap();
        group.bench_with_input(
            BenchmarkId::new("snapshot", &label),
            &snap_bytes,
            |b, data| b.iter(|| from_snapshot(black_box(data)).unwrap()),
        );
        group.bench_with_input(
            BenchmarkId::new("schematic", &label),
            &schem_bytes,
            |b, data| b.iter(|| from_schematic(black_box(data)).unwrap()),
        );
        group.bench_with_input(
            BenchmarkId::new("litematic", &label),
            &lit_bytes,
            |b, data| b.iter(|| from_litematic(black_box(data)).unwrap()),
        );
    }

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");
    group.measurement_time(Duration::from_secs(3));

    for &size in &[32, 64] {
        let schematic = make_solid_schematic(size);
        let label = format!("{}³_solid", size);

        group.bench_with_input(BenchmarkId::new("snapshot", &label), &schematic, |b, s| {
            b.iter(|| {
                let bytes = to_snapshot(black_box(s)).unwrap();
                from_snapshot(&bytes).unwrap()
            })
        });
        group.bench_with_input(BenchmarkId::new("schematic", &label), &schematic, |b, s| {
            b.iter(|| {
                let bytes = to_schematic(black_box(s)).unwrap();
                from_schematic(&bytes).unwrap()
            })
        });
        group.bench_with_input(BenchmarkId::new("litematic", &label), &schematic, |b, s| {
            b.iter(|| {
                let bytes = to_litematic(black_box(s)).unwrap();
                from_litematic(&bytes).unwrap()
            })
        });
    }

    // Sparse 64³ at 50% — worst case for compression (max entropy)
    let sparse = make_sparse_schematic(64, 0.5);
    group.bench_with_input(
        BenchmarkId::new("snapshot", "64³_50pct"),
        &sparse,
        |b, s| {
            b.iter(|| {
                let bytes = to_snapshot(black_box(s)).unwrap();
                from_snapshot(&bytes).unwrap()
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("schematic", "64³_50pct"),
        &sparse,
        |b, s| {
            b.iter(|| {
                let bytes = to_schematic(black_box(s)).unwrap();
                from_schematic(&bytes).unwrap()
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("litematic", "64³_50pct"),
        &sparse,
        |b, s| {
            b.iter(|| {
                let bytes = to_litematic(black_box(s)).unwrap();
                from_litematic(&bytes).unwrap()
            })
        },
    );

    group.finish();
}

fn bench_deserialize_fixture(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize_fixture");
    group.measurement_time(Duration::from_secs(3));

    if let Ok(data) = std::fs::read("tests/samples/cutecounter.schem") {
        group.bench_with_input(
            BenchmarkId::new("schematic", "cutecounter"),
            &data,
            |b, data| b.iter(|| from_schematic(black_box(data)).unwrap()),
        );
    }

    if let Ok(data) = std::fs::read("tests/samples/sample.litematic") {
        group.bench_with_input(BenchmarkId::new("litematic", "sample"), &data, |b, data| {
            b.iter(|| from_litematic(black_box(data)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_serialize,
    bench_deserialize,
    bench_roundtrip,
    bench_deserialize_fixture
);
criterion_main!(benches);
