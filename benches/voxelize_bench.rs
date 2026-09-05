//! Mesh voxelization benchmarks. Sizes 32 and 64 always run; 128 is behind
//! NUCLEATION_BENCH_LARGE=1 because before the surface field lands it takes
//! about half an hour per sample.
//!
//!   cargo bench --features bridge,voxelize --bench voxelize_bench -- \
//!     --warm-up-time 1 --measurement-time 3

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use nucleation::building::{BlockPalette, BuildingTool, SolidBrush};
use nucleation::voxelize::{test_meshes::sphere_5k, voxelize_textured, MeshModel, MeshShape};
use nucleation::{BlockState, UniversalSchematic};

fn sphere(size: f32) -> MeshShape {
    let mut model = MeshModel::from_obj_str(&sphere_5k()).expect("sphere parses");
    model.fit(size);
    MeshShape::new(model)
}

fn textured_cube(size: f32) -> MeshShape {
    let bytes = std::fs::read("tests/samples/BoxTextured.glb").expect("committed sample");
    let mut model = MeshModel::from_glb_bytes(&bytes).expect("BoxTextured loads");
    model.fit(size);
    MeshShape::new(model)
}

fn fill(shape: &MeshShape) -> usize {
    let mut schematic = UniversalSchematic::new("bench".to_string());
    let brush = SolidBrush::new(BlockState::new("minecraft:stone"));
    BuildingTool::new(&mut schematic).fill(shape, &brush);
    schematic.total_blocks() as usize
}

fn sizes() -> Vec<f32> {
    let mut s = vec![32.0, 64.0];
    if std::env::var("NUCLEATION_BENCH_LARGE").as_deref() == Ok("1") {
        s.push(128.0);
    }
    s
}

fn bench(c: &mut Criterion) {
    let mut g = c.benchmark_group("voxelize");
    g.sample_size(10);
    for size in sizes() {
        let solid = sphere(size);
        g.bench_with_input(
            BenchmarkId::new("sphere_solid", size as i32),
            &solid,
            |b, s| {
                // A fresh MeshShape per iteration: the mask and the surface field
                // are cached per shape, and we are timing the whole fill.
                b.iter_batched(
                    || s.clone_uncached(),
                    |s| fill(&s),
                    criterion::BatchSize::SmallInput,
                )
            },
        );
        let shell = sphere(size).with_surface_shell(1.0);
        g.bench_with_input(
            BenchmarkId::new("sphere_shell", size as i32),
            &shell,
            |b, s| {
                b.iter_batched(
                    || s.clone_uncached(),
                    |s| fill(&s),
                    criterion::BatchSize::SmallInput,
                )
            },
        );
        let palette = BlockPalette::new_wool();
        let cube = textured_cube(size);
        g.bench_with_input(
            BenchmarkId::new("textured_cube", size as i32),
            &cube,
            |b, s| {
                b.iter_batched(
                    || s.clone_uncached(),
                    |s| voxelize_textured(&s, &palette, "bench").total_blocks(),
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    g.finish();
}

criterion_group!(voxelize_benches, bench);
criterion_main!(voxelize_benches);
