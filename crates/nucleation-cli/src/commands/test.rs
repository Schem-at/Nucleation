//! `nucleation test` — run schematic-embedded and sidecar test suites.
//!
//! Also home of the discovery and structure-loading helpers the other
//! subcommands share. See the crate docs for the discovery rules.

use std::path::{Path, PathBuf};
use std::time::Duration;

use mc_test::mc_tick::Structure;
use mc_test::{parse_suite, run_with, RunOptions};
use nucleation::formats::gametest::to_gametest_snbt;

use crate::usage_and_exit;

pub(crate) struct Options {
    pub(crate) filter: String,
    pub(crate) specs: Option<PathBuf>,
    pub(crate) json: bool,
    pub(crate) trace_window: u64,
    pub(crate) paths: Vec<PathBuf>,
}

pub(crate) enum FileOutcome {
    /// Every case's (name, ticks, wall, Ok / failure report).
    Ran(Vec<(String, u64, Duration, Result<(), String>)>),
    /// A structure with no spec anywhere.
    Unported,
    /// Could not be read at all.
    Broken(String),
}

pub(crate) fn test_main(args: impl Iterator<Item = String>) {
    let mut options = Options {
        filter: String::new(),
        specs: None,
        json: false,
        trace_window: 2,
        paths: Vec::new(),
    };
    let mut tui = false;
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--path" => options
                .paths
                .push(PathBuf::from(args.next().unwrap_or_else(|| usage_and_exit()))),
            "--filter" => options.filter = args.next().unwrap_or_else(|| usage_and_exit()),
            "--specs" => {
                options.specs = Some(PathBuf::from(args.next().unwrap_or_else(|| usage_and_exit())))
            }
            "--json" => options.json = true,
            "--tui" => tui = true,
            "--trace-window" => {
                options.trace_window = args
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or_else(|| usage_and_exit())
            }
            other if other.starts_with("--") => usage_and_exit(),
            other => options.paths.push(PathBuf::from(other)),
        }
    }
    if options.paths.is_empty() {
        usage_and_exit();
    }
    if tui {
        let screen = crate::tui::Screen::Tests(crate::tui::tests_screen::TestsState::new(
            options.paths.clone(),
            options.specs.clone(),
            options.trace_window,
        ));
        match crate::tui::run(screen) {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("terminal error: {e}");
                std::process::exit(2);
            }
        }
    }

    let files = discover(&options.paths, &options.filter);
    let mut rows: Vec<(PathBuf, FileOutcome)> = Vec::new();
    for (root, file) in files {
        rows.push((file.clone(), run_file_caught(&root, &file, &options)));
    }
    let code = if options.json { print_json(&rows) } else { print_grid(&rows) };
    std::process::exit(code);
}

/// [`run_file`], with the harness's panics (a structure it refuses to
/// half-simulate) turned into a broken row rather than the end of the run.
pub(crate) fn run_file_caught(root: &Path, file: &Path, options: &Options) -> FileOutcome {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_file(root, file, options)))
        .unwrap_or_else(|panic| {
            let why = panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("the harness panicked");
            FileOutcome::Broken(why.to_string())
        })
}

/// Walk each path; a directory yields its files recursively (sorted), a file
/// yields itself. Returns `(scan root, file)` so `--specs` can mirror the
/// relative path. Sidecar descriptors ride along with their structure rather
/// than being rows of their own.
pub(crate) fn discover(paths: &[PathBuf], filter: &str) -> Vec<(PathBuf, PathBuf)> {
    fn walk(root: &Path, dir: &Path, filter: &str, into: &mut Vec<(PathBuf, PathBuf)>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        let mut children: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        children.sort();
        for child in children {
            if child.is_dir() {
                // Hidden trees and build junk would dominate any walk from a
                // project root (`.git` alone is thousands of files).
                let name = child.file_name().map(|n| n.to_string_lossy().to_string());
                let name = name.unwrap_or_default();
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                walk(root, &child, filter, into);
            } else if is_candidate(&child)
                && (filter.is_empty() || child.to_string_lossy().contains(filter))
            {
                into.push((root.to_path_buf(), child));
            }
        }
    }
    fn is_candidate(path: &Path) -> bool {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if name.starts_with('.') {
            return false;
        }
        // A descriptor is a row of its own only when it stands alone; one with
        // its `.snbt` twin beside it rides along as that structure's sidecar.
        if name.ends_with(".test.json") {
            return !path.with_extension("").with_extension("snbt").is_file();
        }
        // Documentation and plain JSON are not carriers; everything else is
        // expected to load, and fails loudly if it does not.
        !name.ends_with(".md") && !name.ends_with(".json")
    }
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            walk(path, path, filter, &mut files);
        } else if path.is_file() {
            // A file named directly bypasses the filter; its parent is its root.
            let root = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            files.push((root, path.clone()));
        }
    }
    files
}

pub(crate) fn run_file(root: &Path, path: &Path, options: &Options) -> FileOutcome {
    let what = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    if what.ends_with(".test.json") {
        // A standalone descriptor: its structures are the ones it names
        // (`structure` per case, defaulting to its own `<stem>.snbt`).
        run_spec(path, &path.with_extension("").with_extension("snbt"), &what, options)
    } else if path.extension().is_some_and(|e| e == "snbt") {
        run_snbt(root, path, &what, options)
    } else {
        run_schematic(path, &what, options)
    }
}

/// Any schematic the `FormatManager` reads: the suite is wherever the
/// importer put `metadata.embedded_test`, and the format never matters here.
fn run_schematic(path: &Path, what: &str, options: &Options) -> FileOutcome {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => return FileOutcome::Broken(format!("reading it: {e}")),
    };
    let schematic = {
        let manager = nucleation::formats::manager::get_manager();
        let manager = match manager.lock() {
            Ok(manager) => manager,
            Err(e) => return FileOutcome::Broken(format!("format manager: {e}")),
        };
        match manager.read(&bytes) {
            Ok(schematic) => schematic,
            Err(e) => return FileOutcome::Broken(format!("no importer recognises it: {e:?}")),
        }
    };
    let Some(spec) = schematic.metadata.embedded_test.as_deref() else {
        return FileOutcome::Unported;
    };
    let cases = match parse_suite(spec, what) {
        Ok(cases) => cases,
        Err(e) => return FileOutcome::Broken(e),
    };
    let snbt = to_gametest_snbt(&schematic);
    let structure = match Structure::parse(&snbt) {
        Ok(structure) => structure,
        Err(e) => return FileOutcome::Broken(format!("the engine refused it: {e:?}")),
    };
    let run_options = RunOptions { trace_window: options.trace_window };
    let mut results = Vec::new();
    for case in &cases {
        if case.structure.is_some() {
            return FileOutcome::Broken(format!(
                "\"{}\" names a separate structure file, but the carrier *is* the structure — \
                 drop the field",
                case.name
            ));
        }
        // The file's stated data version is the authority on `Entity.load`
        // motion semantics — see tests/embedded_cases.rs for why.
        let result =
            run_with(&structure, case, schematic.metadata.source_data_version, &run_options);
        results.push((result.name, result.ticks, result.wall, result.outcome));
    }
    FileOutcome::Ran(results)
}

/// An `.snbt` structure with its descriptor beside it (`<stem>.test.json`) or
/// mirrored under `--specs`. Neither is not an error: it is an unported row.
fn run_snbt(root: &Path, path: &Path, what: &str, options: &Options) -> FileOutcome {
    let sidecar = path.with_extension("").with_extension("test.json");
    let spec_path = if sidecar.is_file() {
        Some(sidecar)
    } else {
        options.specs.as_ref().and_then(|specs| {
            let rel = path.strip_prefix(root).unwrap_or(path);
            let mirrored = specs.join(rel).with_extension("").with_extension("test.json");
            mirrored.is_file().then_some(mirrored)
        })
    };
    let Some(spec_path) = spec_path else {
        return FileOutcome::Unported;
    };
    run_spec(&spec_path, path, what, options)
}

/// A descriptor file against its structures: each case's `structure` path is
/// relative to the spec — the `cases.rs` carrier behaviour — with
/// `default_structure` when the field is absent.
fn run_spec(
    spec_path: &Path,
    default_structure: &Path,
    what: &str,
    options: &Options,
) -> FileOutcome {
    let spec = match std::fs::read_to_string(spec_path) {
        Ok(spec) => spec,
        Err(e) => return FileOutcome::Broken(format!("reading {}: {e}", spec_path.display())),
    };
    let cases = match parse_suite(&spec, what) {
        Ok(cases) => cases,
        Err(e) => return FileOutcome::Broken(e),
    };
    let run_options = RunOptions { trace_window: options.trace_window };
    let mut results = Vec::new();
    for case in &cases {
        let structure_path = match &case.structure {
            Some(rel) => spec_path.parent().unwrap_or(Path::new(".")).join(rel),
            None => default_structure.to_path_buf(),
        };
        let structure = match load_structure(&structure_path) {
            Ok(structure) => structure,
            Err(e) => return FileOutcome::Broken(e),
        };
        // An SNBT carrier has no save version of its own to speak for; these
        // are captures of contraptions, not of a world's entities.
        let result = run_with(&structure, case, None, &run_options);
        results.push((result.name, result.ticks, result.wall, result.outcome));
    }
    FileOutcome::Ran(results)
}

/// Read an `.snbt` structure in either flavor: the engine's own (`blocks` +
/// compound palette), or the modern `data:`-list gametest flavor (1.21.5+,
/// what lithium ships) via nucleation's structure-SNBT importer and a re-emit
/// in the engine flavor — anything that imports to a `UniversalSchematic` is
/// a carrier.
pub(crate) fn load_structure(path: &Path) -> Result<Structure, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("reading {}: {e}", path.display()))?;
    let engine_err = match Structure::parse(&text) {
        Ok(structure) => return Ok(structure),
        Err(e) => e,
    };
    match nucleation::formats::structure_snbt::from_structure_snbt(text.as_bytes()) {
        Ok(schematic) => {
            let snbt = to_gametest_snbt(&schematic);
            Structure::parse(&snbt)
                .map_err(|e| format!("the engine refused the re-emitted structure: {e:?}"))
        }
        Err(_) => Err(format!("the engine refused it: {engine_err:?}")),
    }
}

pub(crate) fn display_path(path: &Path) -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    path.strip_prefix(&cwd).unwrap_or(path).display().to_string()
}

/// The human grid: failures in full first, then one row per file, then the
/// summary. Returns the exit code.
fn print_grid(rows: &[(PathBuf, FileOutcome)]) -> i32 {
    let (mut pass, mut fail, mut unported, mut broken) = (0usize, 0usize, 0usize, 0usize);
    let mut reports: Vec<String> = Vec::new();
    let mut grid: Vec<(String, String)> = Vec::new();

    for (path, outcome) in rows {
        let shown = display_path(path);
        match outcome {
            FileOutcome::Ran(results) => {
                let glyphs: String = results
                    .iter()
                    .map(|(_, _, _, outcome)| if outcome.is_ok() { '✓' } else { '✗' })
                    .collect();
                let wall: Duration = results.iter().map(|(_, _, wall, _)| *wall).sum();
                for (_, _, _, outcome) in results {
                    match outcome {
                        Ok(()) => pass += 1,
                        Err(report) => {
                            fail += 1;
                            reports.push(format!("--- {shown}\n{report}"));
                        }
                    }
                }
                grid.push((
                    shown,
                    format!("{glyphs}  {} case(s)  {}ms", results.len(), wall.as_millis()),
                ));
            }
            FileOutcome::Unported => {
                unported += 1;
                grid.push((shown, "∅ unported".to_string()));
            }
            FileOutcome::Broken(why) => {
                broken += 1;
                let first_line = why.lines().next().unwrap_or("unreadable");
                grid.push((shown, format!("! {first_line}")));
            }
        }
    }

    for report in &reports {
        println!("{report}\n");
    }
    let width = grid.iter().map(|(path, _)| path.chars().count()).max().unwrap_or(0);
    for (path, cell) in &grid {
        println!("{path:width$}  {cell}");
    }
    println!(
        "\n{} files: {pass} pass, {fail} fail, {unported} unported, {broken} broken",
        rows.len()
    );
    i32::from(fail > 0 || broken > 0)
}

/// The same verdicts as one JSON document on stdout. Returns the exit code.
fn print_json(rows: &[(PathBuf, FileOutcome)]) -> i32 {
    let (mut pass, mut fail, mut unported, mut broken) = (0usize, 0usize, 0usize, 0usize);
    let files: Vec<serde_json::Value> = rows
        .iter()
        .map(|(path, outcome)| {
            let shown = display_path(path);
            match outcome {
                FileOutcome::Ran(results) => {
                    let cases: Vec<serde_json::Value> = results
                        .iter()
                        .map(|(name, ticks, wall, outcome)| {
                            match outcome {
                                Ok(()) => pass += 1,
                                Err(_) => fail += 1,
                            }
                            let mut case = serde_json::json!({
                                "name": name,
                                "ok": outcome.is_ok(),
                                "ticks": ticks,
                                "ms": wall.as_millis() as u64,
                            });
                            if let Err(report) = outcome {
                                case["report"] = serde_json::json!(report);
                            }
                            case
                        })
                        .collect();
                    serde_json::json!({"path": shown, "status": "ran", "cases": cases})
                }
                FileOutcome::Unported => {
                    unported += 1;
                    serde_json::json!({"path": shown, "status": "unported"})
                }
                FileOutcome::Broken(why) => {
                    broken += 1;
                    serde_json::json!({"path": shown, "status": "broken", "error": why})
                }
            }
        })
        .collect();
    let doc = serde_json::json!({
        "files": files,
        "summary": {
            "files": rows.len(),
            "pass": pass,
            "fail": fail,
            "unported": unported,
            "broken": broken,
        },
    });
    println!("{doc}");
    i32::from(fail > 0 || broken > 0)
}
