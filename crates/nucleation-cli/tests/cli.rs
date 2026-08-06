//! The CLI end to end: discovery, the grid, exit codes, --json.

use std::path::Path;
use std::process::Command;

use nucleation::formats::gametest::to_gametest_snbt;
use nucleation::formats::litematic::to_litematic;
use nucleation::{BlockState, UniversalSchematic};

fn stone_build(name: &str) -> UniversalSchematic {
    let mut schem = UniversalSchematic::new(name.into());
    schem.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));
    schem
}

const PASSING: &str = r#"{"name":"stone stays","checks":[{"tick":1,"expect":"blocks","blocks":{"0,0,0":"minecraft:stone"}}]}"#;
const FAILING: &str = r#"{"name":"stone is not glass","checks":[{"tick":1,"expect":"blocks","blocks":{"0,0,0":"minecraft:glass"}}]}"#;

fn write_corpus(dir: &Path) {
    // 1. A passing self-testing litematic, with a two-case suite.
    let mut passing = stone_build("passing");
    passing.metadata.embedded_test =
        Some(format!(r#"{{"format":1,"cases":[{PASSING},{PASSING}]}}"#));
    std::fs::write(
        dir.join("passing.litematic"),
        to_litematic(&passing).unwrap(),
    )
    .unwrap();

    // 2. A failing snbt + sidecar pair.
    let failing = stone_build("failing");
    std::fs::write(dir.join("failing.snbt"), to_gametest_snbt(&failing)).unwrap();
    std::fs::write(dir.join("failing.test.json"), FAILING).unwrap();

    // 3. A structure with no test at all.
    let unported = stone_build("unported");
    std::fs::write(dir.join("unported.snbt"), to_gametest_snbt(&unported)).unwrap();
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nucleation"))
        .args(args)
        .output()
        .expect("the CLI runs")
}

#[test]
fn the_grid_names_every_file_and_the_exit_code_says_fail() {
    let dir = std::env::temp_dir().join(format!("mc-test-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    write_corpus(&dir);

    let out = run_cli(&["test", "--path", dir.to_str().unwrap()]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(1),
        "a failing case must fail the run:\n{text}"
    );
    assert!(text.contains("passing.litematic"), "{text}");
    assert!(text.contains("failing.snbt"), "{text}");
    assert!(text.contains("unported.snbt"), "{text}");
    assert!(text.contains("unported"), "{text}");
    assert!(
        text.contains("stone is not glass"),
        "the failure report must reach the user:\n{text}"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_passing_corpus_exits_zero_and_json_is_parseable() {
    let dir = std::env::temp_dir().join(format!("mc-test-cli-ok-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut passing = stone_build("passing");
    passing.metadata.embedded_test = Some(PASSING.to_string());
    std::fs::write(
        dir.join("passing.litematic"),
        to_litematic(&passing).unwrap(),
    )
    .unwrap();

    let out = run_cli(&["test", "--json", dir.to_str().unwrap()]);
    let text = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(out.status.code(), Some(0), "{text}");
    let doc: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("--json emits one JSON document");
    assert_eq!(doc["summary"]["fail"], 0);
    assert_eq!(doc["summary"]["pass"], 1);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn info_json_reports_what_the_file_holds() {
    let dir = std::env::temp_dir().join(format!("nucleation-info-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut build = stone_build("info");
    build.metadata.embedded_test = Some(PASSING.to_string());
    let file = dir.join("info.litematic");
    std::fs::write(&file, to_litematic(&build).unwrap()).unwrap();

    let out = run_cli(&["info", "--json", file.to_str().unwrap()]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    assert_eq!(doc["format"], "litematic");
    assert_eq!(doc["total_blocks"], 1);
    assert_eq!(doc["palette"][0][0], "minecraft:stone");
    assert_eq!(doc["embedded_test"]["cases"], 1);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn convert_round_trips_across_formats() {
    let dir = std::env::temp_dir().join(format!("nucleation-convert-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("build.litematic");
    std::fs::write(&source, to_litematic(&stone_build("convert")).unwrap()).unwrap();
    let schem = dir.join("build.schem");
    let back = dir.join("back.litematic");

    let out = run_cli(&[
        "convert",
        source.to_str().unwrap(),
        "-o",
        schem.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = run_cli(&[
        "convert",
        schem.to_str().unwrap(),
        "-o",
        back.to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let doc = run_cli(&["info", "--json", back.to_str().unwrap()]);
    let doc: serde_json::Value = serde_json::from_slice(&doc.stdout).expect("parses");
    assert_eq!(doc["total_blocks"], 1, "the stone survived two conversions");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn diff_exit_codes_tell_identical_from_different() {
    let dir = std::env::temp_dir().join(format!("nucleation-diff-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.litematic");
    std::fs::write(&a, to_litematic(&stone_build("a")).unwrap()).unwrap();
    let mut other = stone_build("b");
    other.set_block(
        1,
        0,
        0,
        &nucleation::BlockState::new("minecraft:glass".to_string()),
    );
    let b = dir.join("b.litematic");
    std::fs::write(&b, to_litematic(&other).unwrap()).unwrap();

    let same = run_cli(&["diff", a.to_str().unwrap(), a.to_str().unwrap()]);
    assert_eq!(
        same.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&same.stderr)
    );

    let different = run_cli(&["diff", a.to_str().unwrap(), b.to_str().unwrap()]);
    assert_eq!(
        different.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&different.stderr)
    );
    let text = String::from_utf8_lossy(&different.stdout);
    assert!(text.contains("+1"), "one added block must show: {text}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn unknown_flags_are_usage_errors() {
    let out = run_cli(&["test", "--no-such-flag", "."]);
    assert_eq!(out.status.code(), Some(2), "unknown flags are usage errors");
    let out = run_cli(&["no-such-command"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "unknown subcommands are usage errors"
    );
}

/// `-` is stdin and stdout, ffmpeg-style: a schematic piped through convert
/// comes out as valid payload bytes on stdout with the status on stderr,
/// and `info -` reads the pipe.
#[test]
fn dash_pipes_a_schematic_through_convert_and_info() {
    use std::io::Write as _;
    use std::process::Stdio;

    let dir = std::env::temp_dir().join("nucleation-pipe-test");
    std::fs::create_dir_all(&dir).expect("temp dir");
    let src = dir.join("in.litematic");
    let bytes = nucleation::formats::litematic::to_litematic(&stone_build("pipe"))
        .expect("writes litematic");
    std::fs::write(&src, &bytes).expect("writes file");

    // convert: stdin -> stdout, explicit --to since a pipe has no extension.
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleation"))
        .args(["convert", "-", "--to", "schematic", "-o", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawns");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(&bytes)
        .expect("feeds stdin");
    let out = child.wait_with_output().expect("runs");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.stdout.is_empty(), "payload bytes must land on stdout");
    let status_line = String::from_utf8_lossy(&out.stderr);
    assert!(
        status_line.contains("<stdin>"),
        "status goes to stderr, got {status_line:?}"
    );

    // The bytes that came out are a readable schematic: feed them to info -.
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleation"))
        .args(["info", "-", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawns");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(&out.stdout)
        .expect("feeds stdin");
    let info = child.wait_with_output().expect("runs");
    assert!(info.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&info.stdout).expect("info --json is parseable");
    assert!(json["palette"].is_array() || json["palette"].is_object());
}

/// `test -` takes the corpus file list from stdin, one path per line — the
/// find(1) pipeline shape.
#[test]
fn test_dash_reads_the_path_list_from_stdin() {
    use std::io::Write as _;
    use std::process::Stdio;

    let dir = std::env::temp_dir().join("nucleation-pipe-list-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("corpus dir");
    write_corpus(&dir);
    let list: String = std::fs::read_dir(&dir)
        .expect("corpus dir")
        .flatten()
        .map(|e| format!("{}\n", e.path().display()))
        .collect();

    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleation"))
        .args(["test", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawns");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(list.as_bytes())
        .expect("feeds list");
    let out = child.wait_with_output().expect("runs");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("files:"),
        "the grid summary prints, got {text:?}"
    );
}

/// The embedded man page is real mdoc that mandoc accepts.
#[test]
fn the_man_page_is_mandoc_clean() {
    let out = run_cli(&["man"]);
    let roff = String::from_utf8_lossy(&out.stdout);
    assert!(roff.starts_with(".\\\""), "roff comes out when piped");
    assert!(roff.contains(".Dt NUCLEATION 1"), "a real mdoc document");
    // When mandoc is present (macOS ships it), it must parse without errors.
    if let Ok(mut child) = Command::new("mandoc")
        .arg("-T")
        .arg("lint")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        use std::io::Write as _;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(roff.as_bytes())
            .expect("feeds mandoc");
        let lint = child.wait_with_output().expect("mandoc runs");
        let complaints = String::from_utf8_lossy(&lint.stderr);
        assert!(
            !complaints.contains("ERROR") && !complaints.contains("FATAL"),
            "mandoc must accept the page, said: {complaints}"
        );
    }
}
