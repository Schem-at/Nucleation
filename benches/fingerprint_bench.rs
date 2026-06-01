use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use nucleation::fingerprint::{fingerprint, footprint, signature, FingerprintSpec};
use nucleation::{BlockState, UniversalSchematic}; // re-exported at crate root

fn build(edge: i32) -> UniversalSchematic {
    let mut s = UniversalSchematic::new("b".into());
    let bs = BlockState::new("minecraft:stone");
    for x in 0..edge {
        for y in 0..edge {
            for z in 0..edge {
                s.set_block(x, y, z, &bs);
            }
        }
    }
    s
}

fn bench(c: &mut Criterion) {
    let spec = FingerprintSpec::structural();
    let mut g = c.benchmark_group("fingerprint");
    for edge in [8, 16, 24] {
        let s = build(edge);
        g.bench_with_input(BenchmarkId::new("fingerprint", edge), &s, |b, s| {
            b.iter(|| fingerprint(s, &spec))
        });
        g.bench_with_input(BenchmarkId::new("signature", edge), &s, |b, s| {
            b.iter(|| signature(s, &spec))
        });
        g.bench_with_input(BenchmarkId::new("footprint", edge), &s, |b, s| {
            b.iter(|| footprint(s, &spec))
        });
    }
    g.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
