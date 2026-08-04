//! The `nucleation` CLI. Subcommands plus a TUI shell; the umbrella is the point.
//!
//! ```text
//! nucleation                    open the TUI shell (browser / inspector / tests)
//! nucleation test [--path P]... [--filter S] [--specs DIR] [--json] [--trace-window N] [paths...]
//! nucleation port [--path P]... [--specs DIR] [--max-ticks N] --out DIR
//! ```
//!
//! `test` runs schematic-embedded and sidecar test suites. `port` converts a
//! corpus of `.snbt` gametest structures into self-contained `.litematic`
//! tests: the descriptor (a hand-written spec when one exists, otherwise one
//! synthesized from the structure's own `test_block`s) travels inside the
//! file, so the output runs anywhere `nucleation test` runs.
//!
//! Test discovery, under each path (file or directory, recursively):
//!
//! - any schematic the `FormatManager` detects (`.litematic`, `.schem`, ...)
//!   whose root `NucleationTest` tag holds a suite — the carrier is anything
//!   that imports to a `UniversalSchematic`, never a hardcoded extension,
//! - `.snbt` structures with a `<stem>.test.json` sidecar (or, with `--specs`,
//!   a spec at the same relative path under that directory),
//! - `.snbt` structures with neither, reported as unported rather than
//!   silently skipped: a corpus's gaps are part of its report.
//!
//! One row per file, one glyph per case: `✓` pass, `✗` fail, `∅` unported,
//! `!` unreadable. Exit 0 only when nothing failed or errored; 2 for usage.

mod commands;
mod model;
mod tui;

pub(crate) fn usage_and_exit() -> ! {
    eprintln!(
        "usage: nucleation <command>\n\
         \n\
         commands:\n\
         \x20 test     run schematic-embedded and sidecar test suites\n\
         \x20 port     convert .snbt gametest structures to self-contained .litematic tests\n\
         \x20 inspect  view a schematic (TUI when on a terminal, text otherwise)\n\
         \x20 info     one file's overview on stdout: `info <file> [--json]`\n\
         \x20 convert  re-encode: `convert <in> -o <out> [--to FORMAT] [--format-version V]`\n\
         \x20 diff     compare two builds: `diff <a> <b> [--preset exact] [--json]`\n\
         \x20 render   PNG snapshot: `render <file> --pack <pack.zip> -o out.png [--width N] [--height N]`\n\
         \n\
         nucleation test [--path P]... [--filter S] [--specs DIR] [--json] [--trace-window N] [paths...]\n\
         \n\
         Runs every test suite found under the given paths (--path and bare\n\
         arguments are equivalent): schematics (any format the FormatManager\n\
         detects) carrying a NucleationTest tag, and .snbt structures with a\n\
         <stem>.test.json sidecar or a --specs match. Structures with no test\n\
         report as unported. Exit 1 on any failure.\n\
         \n\
         nucleation port [--path P]... [--specs DIR] [--max-ticks N] --out DIR\n\
         \n\
         Converts every .snbt under the given paths into OUT/<rel>.litematic\n\
         with its test embedded: a --specs/sidecar descriptor when one exists,\n\
         else one synthesized from the structure's test_blocks (start pulses\n\
         emulated, accept feeders must leave their off state). Structures with\n\
         neither are skipped and say so."
    );
    std::process::exit(2);
}

fn main() {
    use std::io::IsTerminal as _;
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        // `run` stays as a quiet alias for muscle memory from the harness's
        // first life as `nucleation-test run`.
        Some("test") | Some("run") => commands::test::test_main(args),
        Some("port") => commands::port::port_main(args),
        Some("info") => commands::info::info_main(args),
        Some("inspect") => {
            let rest: Vec<String> = args.collect();
            let wants_tui = std::io::stdout().is_terminal() && !rest.iter().any(|a| a == "--json");
            if !wants_tui {
                return commands::info::info_main(rest.into_iter());
            }
            let Some(file) = rest.iter().find(|a| !a.starts_with("--")) else { usage_and_exit() };
            match model::gather(std::path::Path::new(file)) {
                Ok(report) => {
                    let screen =
                        tui::Screen::Inspector(tui::inspector::InspectorState::new(report));
                    if let Err(e) = tui::run(screen) {
                        eprintln!("terminal error: {e}");
                        std::process::exit(2);
                    }
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(2);
                }
            }
        }
        Some("convert") => commands::convert::convert_main(args),
        Some("diff") => commands::diff::diff_main(args),
        #[cfg(feature = "render")]
        Some("render") => commands::render::render_main(args),
        #[cfg(not(feature = "render"))]
        Some("render") => {
            eprintln!(
                "the render subcommand is compiled out — rebuild with:\n\
                 \x20 cargo install --path crates/nucleation-cli --features render"
            );
            std::process::exit(2);
        }
        // Bare `nucleation` on a terminal opens the shell.
        None if std::io::stdout().is_terminal() => {
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let screen = tui::Screen::Browser(tui::browser::BrowserState::new(cwd));
            if let Err(e) = tui::run(screen) {
                eprintln!("terminal error: {e}");
                std::process::exit(2);
            }
        }
        _ => usage_and_exit(),
    }
}
