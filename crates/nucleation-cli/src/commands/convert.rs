//! `nucleation convert <in> -o <out> [--to FORMAT] [--format-version V]` —
//! any supported format to any exporter the `FormatManager` knows.
//!
//! `--to` names the exporter (`litematic`, `schematic`, `structure_snbt`, ...);
//! without it the output extension decides.

use std::path::PathBuf;

use crate::usage_and_exit;

pub(crate) fn convert_main(args: impl Iterator<Item = String>) {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut to: Option<String> = None;
    let mut version: Option<String> = None;
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--out" => {
                output = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--to" => to = Some(args.next().unwrap_or_else(|| usage_and_exit())),
            "--format-version" => version = Some(args.next().unwrap_or_else(|| usage_and_exit())),
            other if other.starts_with("--") => usage_and_exit(),
            other => input = Some(PathBuf::from(other)),
        }
    }
    let (Some(input), Some(output)) = (input, output) else {
        usage_and_exit()
    };
    let piping_out = super::io::is_stdio(&output);
    if piping_out && to.is_none() {
        eprintln!("writing to stdout needs --to <format>: a pipe has no file extension");
        std::process::exit(2);
    }

    let result = (|| -> Result<(String, usize, usize), String> {
        let bytes = super::io::read_input(&input)?;
        let manager = nucleation::formats::manager::get_manager();
        let manager = manager.lock().map_err(|e| format!("format manager: {e}"))?;
        let from = manager
            .detect_format(&bytes)
            .unwrap_or_else(|| "unknown".to_string());
        let schematic = manager
            .read(&bytes)
            .map_err(|e| format!("{}: unreadable: {e:?}", super::io::display_name(&input)))?;
        let written = match &to {
            Some(format) => manager
                .write(format, &schematic, version.as_deref())
                .map_err(|e| format!("writing as {format}: {e:?}"))?,
            None => manager
                .write_auto(
                    &output.display().to_string(),
                    &schematic,
                    version.as_deref(),
                )
                .map_err(|e| format!("writing {}: {e:?}", output.display()))?,
        };
        let size = written.len();
        super::io::write_output(&output, &written)?;
        Ok((from, bytes.len(), size))
    })();

    match result {
        Ok((from, in_bytes, out_bytes)) => {
            super::io::status(
                &format!(
                    "{} ({from}, {in_bytes} bytes) -> {} ({out_bytes} bytes)",
                    super::io::display_name(&input),
                    super::io::display_out(&output)
                ),
                piping_out,
            );
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
