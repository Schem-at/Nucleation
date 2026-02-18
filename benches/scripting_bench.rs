use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

// --- Benchmark scenarios ---
// Each scenario is tested across: Rust (native), Lua, JS
// Python + mcschematic are benchmarked via the companion Python script.

/// Scenario 1: Set N individual blocks
fn rust_set_blocks(n: i32) {
    let mut s = nucleation::scripting::ScriptingSchematic::new(Some("bench".into()));
    for i in 0..n {
        s.set_block(i, 0, 0, "minecraft:stone");
    }
}

fn lua_set_blocks(n: i32) {
    let code = format!(
        r#"
        local s = Schematic.new("bench")
        for i = 0, {} do
            s:set_block(i, 0, 0, "minecraft:stone")
        end
        "#,
        n - 1
    );
    nucleation::scripting::lua_engine::run_lua_code(&code).unwrap();
}

fn js_set_blocks(n: i32) {
    let code = format!(
        r#"
        let s = new Schematic("bench");
        for (let i = 0; i < {}; i++) {{
            s.set_block(i, 0, 0, "minecraft:stone");
        }}
        "#,
        n
    );
    nucleation::scripting::js_engine::run_js_code(&code).unwrap();
}

/// Scenario 2: Fill a cuboid of size NxNxN
fn rust_fill_cuboid(n: i32) {
    let mut s = nucleation::scripting::ScriptingSchematic::new(Some("bench".into()));
    s.fill_cuboid((0, 0, 0), (n - 1, n - 1, n - 1), "minecraft:stone");
}

fn lua_fill_cuboid(n: i32) {
    let code = format!(
        r#"
        local s = Schematic.new("bench")
        s:fill_cuboid(0, 0, 0, {}, {}, {}, "minecraft:stone")
        "#,
        n - 1,
        n - 1,
        n - 1
    );
    nucleation::scripting::lua_engine::run_lua_code(&code).unwrap();
}

fn js_fill_cuboid(n: i32) {
    let code = format!(
        r#"
        let s = new Schematic("bench");
        s.fill_cuboid(0, 0, 0, {}, {}, {}, "minecraft:stone");
        "#,
        n - 1,
        n - 1,
        n - 1
    );
    nucleation::scripting::js_engine::run_js_code(&code).unwrap();
}

/// Scenario 3: Fill cuboid + export to .schem bytes
fn rust_fill_and_export(n: i32) {
    let mut s = nucleation::scripting::ScriptingSchematic::new(Some("bench".into()));
    s.fill_cuboid((0, 0, 0), (n - 1, n - 1, n - 1), "minecraft:stone");
    let _ = s.to_schematic().unwrap();
}

fn lua_fill_and_export(n: i32) {
    let code = format!(
        r#"
        local s = Schematic.new("bench")
        s:fill_cuboid(0, 0, 0, {}, {}, {}, "minecraft:stone")
        local bytes = s:to_schematic()
        "#,
        n - 1,
        n - 1,
        n - 1
    );
    nucleation::scripting::lua_engine::run_lua_code(&code).unwrap();
}

fn js_fill_and_export(n: i32) {
    let code = format!(
        r#"
        let s = new Schematic("bench");
        s.fill_cuboid(0, 0, 0, {}, {}, {}, "minecraft:stone");
        let bytes = s.to_schematic();
        "#,
        n - 1,
        n - 1,
        n - 1
    );
    nucleation::scripting::js_engine::run_js_code(&code).unwrap();
}

// --- Criterion benchmarks ---

fn bench_set_blocks(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_blocks");
    for n in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("rust", n), &n, |b, &n| {
            b.iter(|| rust_set_blocks(n))
        });
        group.bench_with_input(BenchmarkId::new("lua", n), &n, |b, &n| {
            b.iter(|| lua_set_blocks(n))
        });
        group.bench_with_input(BenchmarkId::new("js", n), &n, |b, &n| {
            b.iter(|| js_set_blocks(n))
        });
    }
    group.finish();
}

fn bench_fill_cuboid(c: &mut Criterion) {
    let mut group = c.benchmark_group("fill_cuboid");
    for n in [10, 32, 64] {
        group.bench_with_input(BenchmarkId::new("rust", n), &n, |b, &n| {
            b.iter(|| rust_fill_cuboid(n))
        });
        group.bench_with_input(BenchmarkId::new("lua", n), &n, |b, &n| {
            b.iter(|| lua_fill_cuboid(n))
        });
        group.bench_with_input(BenchmarkId::new("js", n), &n, |b, &n| {
            b.iter(|| js_fill_cuboid(n))
        });
    }
    group.finish();
}

fn bench_fill_and_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("fill_and_export");
    for n in [10, 32] {
        group.bench_with_input(BenchmarkId::new("rust", n), &n, |b, &n| {
            b.iter(|| rust_fill_and_export(n))
        });
        group.bench_with_input(BenchmarkId::new("lua", n), &n, |b, &n| {
            b.iter(|| lua_fill_and_export(n))
        });
        group.bench_with_input(BenchmarkId::new("js", n), &n, |b, &n| {
            b.iter(|| js_fill_and_export(n))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_set_blocks,
    bench_fill_cuboid,
    bench_fill_and_export
);
criterion_main!(benches);
