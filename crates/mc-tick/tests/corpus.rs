//! The data-driven corpus runner.
//!
//! One fixed test binary that discovers cases on disk **at runtime**. Adding a
//! case means adding a file — no recompilation, no `#[test]` to write, no
//! rebuild of this harness. Combined with the workspace split (edit→test ~0.7s
//! instead of ~28s), that is what makes a long grind through hundreds of
//! behaviours affordable.
//!
//! # Case format
//!
//! A `.case` file is a line-oriented script: setup directives, then a run
//! directive, then expectations. Blank lines and `#` comments are ignored.
//!
//! ```text
//! bounds 0 0 0 7 7 7
//! set 1 1 1 minecraft:stone
//! schedule 2 2 2 delay=3 priority=normal
//! run 5
//! expect tick 5
//! expect block 1 1 1 minecraft:stone
//! expect quiescent
//! ```
//!
//! Deliberately a tiny hand-writable DSL rather than JSON: these cases are read
//! and diffed by people far more often than they are generated, and a failing
//! assertion should be legible without tooling.
//!
//! # Structures
//!
//! `load <name>.snbt` reads a Java structure from `tests/corpus/structures/` — the
//! same format the vanilla oracle in `tools/gametest` runs. That is the point: the
//! engine and the game consume the *identical file*, so a disagreement is about
//! behaviour rather than about two different inputs.
//!
//! A loaded structure sizes the region itself, with [`STRUCTURE_MARGIN`] of
//! padding, because out-of-bounds neighbours read as air and a contraption flush
//! against the edge would simulate differently than it does in game.

use mc_tick::{Bounds, Pos, Simulation, StopReason, TickPriority};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Padding around a loaded structure. Out-of-bounds neighbours read as air,
/// so a contraption flush against the edge would simulate differently.
const STRUCTURE_MARGIN: i32 = 4;

/// Root of the case corpus, relative to this crate.
const CORPUS_DIR: &str = "tests/corpus";

// ---------------------------------------------------------------------------
// Case model
// ---------------------------------------------------------------------------

/// One line of a case script.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Directive {
    Bounds(Bounds),
    Load(String),
    Set { pos: Pos, descriptor: String },
    Schedule { pos: Pos, delay: u64, priority: TickPriority },
    Event { pos: Pos, id: u8, param: u8 },
    Checkpoint,
    Restore,
    Reset,
    MarkInitial,
    Step,
    Run(u64),
    RunUntilQuiescent(u64),
    ExpectTick(u64),
    ExpectBlock { pos: Pos, descriptor: String },
    ExpectQuiescent(bool),
    ExpectNonAirCount(usize),
    ExpectStop(StopReason),
}

/// A parse failure, reported with the line that caused it.
#[derive(Debug)]
struct ParseError {
    line_number: usize,
    line: String,
    reason: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line {}: {}\n  {}",
            self.line_number, self.reason, self.line
        )
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn parse_i32(token: &str) -> Result<i32, String> {
    token
        .parse()
        .map_err(|_| format!("expected an integer, got {token:?}"))
}

fn parse_pos(tokens: &[&str]) -> Result<Pos, String> {
    if tokens.len() < 3 {
        return Err(format!("expected 3 coordinates, got {}", tokens.len()));
    }
    Ok(Pos::new(
        parse_i32(tokens[0])?,
        parse_i32(tokens[1])?,
        parse_i32(tokens[2])?,
    ))
}

fn parse_priority(name: &str) -> Result<TickPriority, String> {
    Ok(match name {
        "extremely_high" => TickPriority::ExtremelyHigh,
        "very_high" => TickPriority::VeryHigh,
        "high" => TickPriority::High,
        "normal" => TickPriority::Normal,
        "low" => TickPriority::Low,
        "very_low" => TickPriority::VeryLow,
        "extremely_low" => TickPriority::ExtremelyLow,
        other => return Err(format!("unknown priority {other:?}")),
    })
}

fn parse_stop_reason(name: &str) -> Result<StopReason, String> {
    Ok(match name {
        "completed" => StopReason::Completed,
        "quiescent" => StopReason::Quiescent,
        "budget_exhausted" => StopReason::BudgetExhausted,
        "event_chain_limit" => StopReason::EventChainLimit,
        other => return Err(format!("unknown stop reason {other:?}")),
    })
}

/// Pull a `key=value` option out of the trailing tokens.
fn option<'a>(tokens: &[&'a str], key: &str) -> Option<&'a str> {
    tokens
        .iter()
        .find_map(|t| t.strip_prefix(key).and_then(|r| r.strip_prefix('=')))
}

fn parse_directive(tokens: &[&str]) -> Result<Directive, String> {
    match tokens[0] {
        "bounds" => {
            if tokens.len() < 7 {
                return Err("bounds needs 6 coordinates".into());
            }
            Ok(Directive::Bounds(Bounds::new(
                parse_pos(&tokens[1..4])?,
                parse_pos(&tokens[4..7])?,
            )))
        }
        "set" => Ok(Directive::Set {
            pos: parse_pos(&tokens[1..])?,
            descriptor: tokens
                .get(4)
                .ok_or("set needs a block descriptor")?
                .to_string(),
        }),
        "schedule" => Ok(Directive::Schedule {
            pos: parse_pos(&tokens[1..])?,
            delay: option(tokens, "delay")
                .unwrap_or("1")
                .parse()
                .map_err(|_| "delay must be a number".to_string())?,
            priority: parse_priority(option(tokens, "priority").unwrap_or("normal"))?,
        }),
        "event" => Ok(Directive::Event {
            pos: parse_pos(&tokens[1..])?,
            id: option(tokens, "id")
                .unwrap_or("0")
                .parse()
                .map_err(|_| "id must be 0-255".to_string())?,
            param: option(tokens, "param")
                .unwrap_or("0")
                .parse()
                .map_err(|_| "param must be 0-255".to_string())?,
        }),
        "checkpoint" => Ok(Directive::Checkpoint),
        "restore" => Ok(Directive::Restore),
        "reset" => Ok(Directive::Reset),
        "mark_initial" => Ok(Directive::MarkInitial),
        "step" => Ok(Directive::Step),
        "run" => Ok(Directive::Run(
            tokens
                .get(1)
                .ok_or("run needs a tick count")?
                .parse()
                .map_err(|_| "tick count must be a number".to_string())?,
        )),
        "run_until_quiescent" => Ok(Directive::RunUntilQuiescent(
            tokens.get(1).unwrap_or(&"1000").parse().unwrap_or(1000),
        )),
        "load" => Ok(Directive::Load(
            tokens.get(1).ok_or("load needs a structure file")?.to_string(),
        )),
        "expect" => parse_expectation(&tokens[1..]),
        other => Err(format!("unknown directive {other:?}")),
    }
}

fn parse_expectation(tokens: &[&str]) -> Result<Directive, String> {
    let what = tokens.first().ok_or("expect needs a subject")?;
    match *what {
        "tick" => Ok(Directive::ExpectTick(
            tokens
                .get(1)
                .ok_or("expect tick needs a number")?
                .parse()
                .map_err(|_| "tick must be a number".to_string())?,
        )),
        "block" => Ok(Directive::ExpectBlock {
            pos: parse_pos(&tokens[1..])?,
            descriptor: tokens
                .get(4)
                .ok_or("expect block needs a descriptor")?
                .to_string(),
        }),
        "quiescent" => Ok(Directive::ExpectQuiescent(true)),
        "not_quiescent" => Ok(Directive::ExpectQuiescent(false)),
        "non_air_count" => Ok(Directive::ExpectNonAirCount(
            tokens
                .get(1)
                .ok_or("expect non_air_count needs a number")?
                .parse()
                .map_err(|_| "count must be a number".to_string())?,
        )),
        "stop" => Ok(Directive::ExpectStop(parse_stop_reason(
            tokens.get(1).ok_or("expect stop needs a reason")?,
        )?)),
        other => Err(format!("unknown expectation {other:?}")),
    }
}

fn parse_case(source: &str) -> Result<Vec<Directive>, ParseError> {
    let mut directives = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let directive = parse_directive(&tokens).map_err(|reason| ParseError {
            line_number: index + 1,
            line: raw.to_string(),
            reason,
        })?;
        directives.push(directive);
    }
    Ok(directives)
}

// ---------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------

/// Run a case, collecting every failed expectation.
///
/// Collects rather than stopping at the first failure: when a timing change
/// breaks a case, seeing all the consequences at once is far more informative
/// than being told about them one run at a time.
fn run_case(directives: &[Directive]) -> Vec<String> {
    let mut failures = Vec::new();

    // A `load` sizes the region from the structure itself, with padding — the
    // out-of-bounds-reads-air divergence makes an unpadded load simulate
    // differently than the game would.
    let loaded: Option<(mc_tick::Structure, PathBuf)> = directives.iter().find_map(|d| match d {
        Directive::Load(name) => {
            let path = corpus_root().join("structures").join(name);
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|text| mc_tick::Structure::parse(&text).ok())
                .map(|s| (s, path))
        }
        _ => None,
    });

    // A default region, so a case that only cares about scheduling need not
    // declare bounds.
    let bounds = directives
        .iter()
        .find_map(|d| match d {
            Directive::Bounds(b) => Some(*b),
            _ => None,
        })
        .or_else(|| loaded.as_ref().map(|(s, _)| s.bounds(STRUCTURE_MARGIN)))
        .unwrap_or_else(|| Bounds::new(Pos::new(0, 0, 0), Pos::new(15, 15, 15)));

    let mut sim = Simulation::new(bounds);
    let mut saved = None;
    let mut last_stop = StopReason::Completed;

    for directive in directives {
        match directive {
            Directive::Bounds(_) => {}

            Directive::Load(name) => {
                let path = corpus_root().join("structures").join(name);
                match std::fs::read_to_string(&path) {
                    Err(e) => failures.push(format!("load {name}: {e}")),
                    Ok(text) => match mc_tick::Structure::parse(&text) {
                        Err(e) => failures.push(format!("load {name}: {e}")),
                        Ok(structure) => {
                            let (registry, world) = sim.registry_and_world_mut();
                            structure.place(world, registry, Pos::new(0, 0, 0));
                            sim.mark_initial();
                        }
                    },
                }
            }

            Directive::Set { pos, descriptor } => {
                match sim.registry_mut().intern(descriptor) {
                    Ok(id) => {
                        if sim.world_mut().set(*pos, id).is_none() {
                            failures.push(format!(
                                "set {pos:?} {descriptor}: outside the region {:?}",
                                bounds
                            ));
                        }
                    }
                    Err(e) => failures.push(format!("intern {descriptor}: {e}")),
                }
            }

            Directive::Schedule { pos, delay, priority } => {
                sim.schedule_tick(*pos, *delay, *priority);
            }

            Directive::Event { pos, id, param } => {
                sim.queue_event(mc_tick::BlockEvent { pos: *pos, id: *id, param: *param });
            }

            Directive::Checkpoint => saved = Some(sim.checkpoint()),
            Directive::Restore => match &saved {
                Some(c) => sim.restore(c),
                None => failures.push("restore with no prior checkpoint".into()),
            },
            Directive::Reset => sim.reset(),
            Directive::MarkInitial => sim.mark_initial(),

            Directive::Step => last_stop = sim.step(),
            Directive::Run(n) => last_stop = sim.run(*n),
            Directive::RunUntilQuiescent(budget) => {
                last_stop = sim.run_until_quiescent(*budget)
            }

            Directive::ExpectTick(expected) => {
                let actual = sim.tick_count();
                if actual != *expected {
                    failures.push(format!("expected tick {expected}, got {actual}"));
                }
            }

            Directive::ExpectBlock { pos, descriptor } => {
                let actual = sim.world().get(*pos);
                let actual_descriptor = sim.registry().descriptor(actual).unwrap_or("<unknown>");
                if actual_descriptor != descriptor {
                    failures.push(format!(
                        "expected {descriptor} at {pos:?}, found {actual_descriptor}"
                    ));
                }
            }

            Directive::ExpectQuiescent(want) => {
                let actual = sim.is_quiescent();
                if actual != *want {
                    failures.push(format!(
                        "expected quiescent={want}, got quiescent={actual}"
                    ));
                }
            }

            Directive::ExpectNonAirCount(expected) => {
                let actual = sim.world().non_air_count();
                if actual != *expected {
                    failures.push(format!(
                        "expected {expected} non-air blocks, found {actual}"
                    ));
                }
            }

            Directive::ExpectStop(expected) => {
                if last_stop != *expected {
                    failures.push(format!(
                        "expected stop {expected:?}, got {last_stop:?}"
                    ));
                }
            }
        }
    }

    failures
}

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_DIR)
}

/// Every `.case` file under the corpus root, sorted for stable reporting.
fn discover(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "case") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

#[test]
fn corpus_cases_pass() {
    let root = corpus_root();
    let cases = discover(&root);

    assert!(
        !cases.is_empty(),
        "no .case files under {}. The corpus is the point of this harness; an \
         empty one means discovery is broken, not that everything passes.",
        root.display()
    );

    let mut report = String::new();
    let mut failed = 0usize;

    for path in &cases {
        let name = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                failed += 1;
                let _ = writeln!(report, "\n{name}: unreadable: {e}");
                continue;
            }
        };

        match parse_case(&source) {
            Err(e) => {
                failed += 1;
                let _ = writeln!(report, "\n{name}: parse error\n  {e}");
            }
            Ok(directives) => {
                let failures = run_case(&directives);
                if !failures.is_empty() {
                    failed += 1;
                    let _ = writeln!(report, "\n{name}:");
                    for failure in failures {
                        let _ = writeln!(report, "  - {failure}");
                    }
                }
            }
        }
    }

    assert!(
        failed == 0,
        "{failed} of {} corpus cases failed:{report}",
        cases.len()
    );

    eprintln!("corpus: {} cases passed", cases.len());
}

#[test]
fn adding_a_case_requires_no_recompilation() {
    // The harness discovers by walking the filesystem, so this test is really a
    // statement about the mechanism: nothing here is generated at compile time,
    // and no macro enumerates the corpus.
    let root = corpus_root();
    assert!(root.is_dir(), "corpus root {} must exist", root.display());
    let discovered = discover(&root);
    assert!(!discovered.is_empty());

    // A file dropped in now would be picked up by the same call, with no build
    // step in between.
    let synthetic = parse_case("bounds 0 0 0 1 1 1\nrun 2\nexpect tick 2\n")
        .expect("the DSL parses without any compile-time support");
    assert!(run_case(&synthetic).is_empty());
}

#[test]
fn a_malformed_case_reports_its_line() {
    let error = parse_case("bounds 0 0 0 7 7 7\nnonsense 1 2 3\n")
        .expect_err("unknown directives must not be silently skipped");
    assert_eq!(error.line_number, 2);
    assert!(error.reason.contains("nonsense"), "{}", error.reason);
}

#[test]
fn expectation_failures_are_all_reported_not_just_the_first() {
    let directives = parse_case(
        "bounds 0 0 0 3 3 3\nrun 1\nexpect tick 99\nexpect non_air_count 5\n",
    )
    .unwrap();
    let failures = run_case(&directives);
    assert_eq!(failures.len(), 2, "got: {failures:?}");
}

#[test]
fn load_reads_the_same_structures_the_oracle_runs() {
    // The whole point of the reader: engine and game consume the identical file.
    let path = corpus_root().join("structures").join("piston_qc.snbt");
    let text = std::fs::read_to_string(&path).expect("structure must exist");
    let structure = mc_tick::Structure::parse(&text).expect("must parse");

    assert_eq!(structure.size, (3, 3, 1));
    assert!(
        structure.palette.iter().any(|d| d.starts_with("minecraft:piston")),
        "palette: {:?}",
        structure.palette
    );
}

#[test]
fn a_missing_structure_fails_loudly_rather_than_simulating_nothing() {
    // A silently-ignored load would let a structure corpus report green while
    // running an empty world — the worst failure mode available to this project.
    let directives = parse_case("load does_not_exist.snbt\nrun 1\n").unwrap();
    let failures = run_case(&directives);
    assert!(
        failures.iter().any(|f| f.contains("does_not_exist")),
        "must report the missing file: {failures:?}"
    );
}
