//! `nucleation diff <a> <b> [--preset P] [--json]` — the diff engine's verdict
//! on two builds. Exit 0 identical, 1 different, 2 unusable input.

use std::path::{Path, PathBuf};

use nucleation::diff::{diff, DiffSpec, SpecOverrides};
use nucleation::UniversalSchematic;

use crate::usage_and_exit;

pub(crate) fn diff_main(args: impl Iterator<Item = String>) {
    let mut json = false;
    let mut preset = "exact".to_string();
    let mut files: Vec<PathBuf> = Vec::new();
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--preset" => preset = args.next().unwrap_or_else(|| usage_and_exit()),
            other if other.starts_with("--") => usage_and_exit(),
            other => files.push(PathBuf::from(other)),
        }
    }
    let [a_path, b_path] = files.as_slice() else { usage_and_exit() };

    let Some(spec) = DiffSpec::resolve(&preset, &SpecOverrides::default()) else {
        eprintln!("unknown preset {preset:?} — try \"exact\"");
        std::process::exit(2);
    };
    let a = load(a_path);
    let b = load(b_path);
    let result = diff(&a, &b, &spec);
    let summary = result.summary_json();

    // "Identical" is read off the summary the engine itself publishes, so the
    // CLI never re-derives what different means.
    let doc: serde_json::Value = serde_json::from_str(&summary).unwrap_or_default();
    let count = |key: &str| doc["counts"].get(key).and_then(|v| v.as_u64()).unwrap_or(0);
    let (added, removed, changed, swapped) =
        (count("added"), count("removed"), count("changed"), count("swapped"));
    let identical = added + removed + changed + swapped == 0;

    if json {
        println!("{summary}");
    } else if identical {
        println!("identical under preset {preset:?}");
    } else {
        println!(
            "{} vs {}: +{added} -{removed} ~{changed} swapped {swapped}  (preset {preset:?})",
            a_path.display(),
            b_path.display()
        );
    }
    std::process::exit(i32::from(!identical));
}

fn load(path: &Path) -> UniversalSchematic {
    let result = (|| -> Result<UniversalSchematic, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let manager = nucleation::formats::manager::get_manager();
        let manager = manager.lock().map_err(|e| format!("format manager: {e}"))?;
        manager.read(&bytes).map_err(|e| format!("{}: unreadable: {e:?}", path.display()))
    })();
    result.unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    })
}
