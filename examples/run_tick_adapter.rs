//! Run the same named circuit operations as functional checks or a benchmark.
//! run_tick_adapter PROFILE SOURCE SUITE [SAMPLES ITERATIONS]
//! JSON on stdout; nonzero exit on any failed output, reset or loading check.
use nucleation::tick_adapter::{AdapterProfile, TickAdapter, Values};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{path::Path, time::Instant};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    inputs: Values,
    expected: Values,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    schema_version: u32,
    adapter: String,
    oracle: String,
    vectors: Vec<Vector>,
}
fn hash(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn execute(a: &mut TickAdapter, suite: &Suite) -> Result<Vec<Values>, String> {
    suite.vectors.iter().map(|v| a.execute(&v.inputs)).collect()
}
fn check(observed: &[Values], suite: &Suite) -> Result<(), String> {
    for (i, (actual, vector)) in observed.iter().zip(&suite.vectors).enumerate() {
        if actual != &vector.expected {
            return Err(format!(
                "vector {i}, inputs {:?}: expected {:?}, actual {actual:?}",
                vector.inputs, vector.expected
            ));
        }
    }
    if observed.len() != suite.vectors.len() {
        return Err("observation count mismatch".into());
    }
    Ok(())
}
fn run() -> Result<serde_json::Value, String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if ![3, 5].contains(&args.len()) {
        return Err("usage: run_tick_adapter PROFILE SOURCE SUITE [SAMPLES ITERATIONS]".into());
    }
    let read = |p: &str| std::fs::read(p).map_err(|e| format!("{p}: {e}"));
    let profile_bytes = read(&args[0])?;
    let source = read(&args[1])?;
    let suite_bytes = read(&args[2])?;
    let profile: AdapterProfile =
        serde_json::from_slice(&profile_bytes).map_err(|e| e.to_string())?;
    profile.validate()?;
    if hash(&source) != profile.source_sha256 {
        return Err("source SHA-256 differs from adapter".into());
    }
    let suite: Suite = serde_json::from_slice(&suite_bytes).map_err(|e| e.to_string())?;
    if suite.schema_version != 1
        || suite.adapter != profile.id
        || suite.vectors.is_empty()
        || suite.vectors.len() > 1_000_000
    {
        return Err("invalid suite version, adapter id or vector count".into());
    }
    for v in &suite.vectors {
        profile.validate_values(&v.inputs, false)?;
        profile.validate_values(&v.expected, true)?;
    }
    let samples: usize = args
        .get(3)
        .map(|s| s.parse())
        .transpose()
        .map_err(|_| "invalid sample count")?
        .unwrap_or(0);
    let iterations: usize = args
        .get(4)
        .map(|s| s.parse())
        .transpose()
        .map_err(|_| "invalid iteration count")?
        .unwrap_or(1);
    if samples > 1000 || iterations == 0 || iterations > 10_000 {
        return Err("benchmark bounds exceeded".into());
    }
    let schematic = nucleation::formats::manager::get_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .read(&source)
        .map_err(|e| e.to_string())?;
    let snbt = nucleation::formats::gametest::to_gametest_snbt(&schematic);
    let structure = mc_tick::Structure::parse(&snbt).map_err(|e| format!("{e:?}"))?;
    if structure.data_version != Some(profile.source_data_version) {
        return Err("source save DataVersion differs from adapter".into());
    }
    let [x, y, z] = profile.hash_origin;
    let sim = mc_test::try_build_sim(
        &structure,
        mc_tick::Pos::new(x, y, z),
        mc_test::SettleMode::InWorld,
        &[],
        &[],
        structure.data_version,
        &profile.id,
    )?;
    let mut adapter = TickAdapter::new(sim, profile.clone())?;
    let first = execute(&mut adapter, &suite)?;
    check(&first, &suite)?;
    let end = adapter.simulation().checkpoint();
    adapter.reset();
    let second = execute(&mut adapter, &suite)?;
    check(&second, &suite)?;
    if first != second || end != adapter.simulation().checkpoint() {
        return Err("checkpoint replay differs".into());
    }
    // One untimed full warmup, then measured cycles with reset and validation
    // outside the timer. Include typed drive, real ticks, observation and output
    // collection. No parsing, construction, assertions, rendering or export.
    let mut samples_ns = Vec::new();
    if samples > 0 {
        for sample in 0..=samples {
            let mut elapsed = 0u128;
            for _ in 0..iterations {
                adapter.reset();
                let start = Instant::now();
                let observations = execute(&mut adapter, &suite)?;
                elapsed += start.elapsed().as_nanos();
                check(&observations, &suite)?;
            }
            if sample > 0 {
                samples_ns.push(elapsed as u64);
            }
        }
    }
    Ok(
        json!({"schema_version":1,"adapter":profile.id,"status":"functional_pass",
        "source_sha256":hash(&source),"profile_sha256":hash(&profile_bytes),"suite_sha256":hash(&suite_bytes),
        "runner_sha256":hash(&std::fs::read(std::env::current_exe().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?),
        "source_file":Path::new(&args[1]).file_name().and_then(|n|n.to_str()),
        "source_data_version":structure.data_version,"oracle":suite.oracle,"vectors":suite.vectors.len(),
        "output_assertions_per_run":suite.vectors.len()*profile.outputs.len(),"reset_replay":"exact_checkpoint_and_outputs",
        "response_game_ticks":profile.response_ticks,"setup_game_ticks":profile.setup_ticks,
        "ticks_per_iteration":profile.response_ticks*suite.vectors.len() as u64,"rng_seed":profile.rng_seed,"random_tick_speed":0,
        "recording":"disabled","coordinates":"production export bounding-box-relative; explicit hash_origin preserves world-dependent order","hash_origin":profile.hash_origin,
        "scope":"engine endpoint functionality under this fixed protocol; not a vanilla comparison or minimal latency proof",
        "benchmark":{"clock":"elapsed_monotonic","samples_ns":samples_ns,"iterations":iterations,"warmups":if samples>0 {1} else {0},
            "includes":"typed drive, real simulation ticks, named output reads and result collection",
            "excludes":"load, setup, checkpoint restore, assertions, destruction, rendering, export"},
        "observations":first}),
    )
}
fn main() {
    match run() {
        Ok(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
        Err(error) => {
            println!("{}", json!({"status":"functional_fail","error":error}));
            std::process::exit(1);
        }
    }
}
