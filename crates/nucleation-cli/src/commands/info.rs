//! `nucleation info <file> [--json]` — the inspector's overview, on stdout.
//!
//! The scriptable twin of the TUI inspector: both consume the same
//! [`FileReport`](crate::model::FileReport), so they cannot disagree.

use std::path::PathBuf;

use crate::model::{gather, FileReport};
use crate::usage_and_exit;

pub(crate) fn info_main(args: impl Iterator<Item = String>) {
    let mut json = false;
    let mut file: Option<PathBuf> = None;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            other if other.starts_with("--") => usage_and_exit(),
            other => file = Some(PathBuf::from(other)),
        }
    }
    let Some(file) = file else { usage_and_exit() };
    let report = match gather(&file) {
        Ok(report) => report,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    if json {
        println!("{}", to_json(&report));
    } else {
        print!("{}", to_text(&report));
    }
}

/// The overview as one JSON document with stable keys.
pub(crate) fn to_json(report: &FileReport) -> String {
    let embedded_test = report.embedded_test.as_ref().map(|t| {
        serde_json::json!({
            "cases": t.cases,
            "names": t.names,
            "parse_error": t.parse_error,
        })
    });
    serde_json::json!({
        "path": report.path.display().to_string(),
        "format": report.format,
        "bytes": report.bytes,
        "name": report.name,
        "author": report.author,
        "description": report.description,
        "data_version": report.data_version,
        "dimensions": [report.dimensions.0, report.dimensions.1, report.dimensions.2],
        "total_blocks": report.total_blocks,
        "total_volume": report.total_volume,
        "regions": report.regions.iter().map(|r| serde_json::json!({
            "name": r.name,
            "dimensions": [r.dimensions.0, r.dimensions.1, r.dimensions.2],
            "blocks": r.blocks,
        })).collect::<Vec<_>>(),
        "palette": report.palette.iter().map(|(d, n)| serde_json::json!([d, n])).collect::<Vec<_>>(),
        "entities": report.entities,
        "block_entities": report.block_entities,
        "embedded_test": embedded_test,
    })
    .to_string()
}

/// The overview as human text. Palette truncated to the top 20; a build's
/// long tail belongs in the TUI, not a terminal dump.
pub(crate) fn to_text(report: &FileReport) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let _ = writeln!(out, "{}", report.path.display());
    let _ = writeln!(
        out,
        "  format      {} ({} bytes)",
        report.format, report.bytes
    );
    if let Some(name) = &report.name {
        let _ = writeln!(out, "  name        {name}");
    }
    if let Some(author) = &report.author {
        let _ = writeln!(out, "  author      {author}");
    }
    if let Some(description) = &report.description {
        let _ = writeln!(out, "  description {description}");
    }
    if let Some(dv) = report.data_version {
        let _ = writeln!(out, "  data ver    {dv}");
    }
    let (w, h, l) = report.dimensions;
    let _ = writeln!(
        out,
        "  size        {w}x{h}x{l}  ({} blocks / {} volume)",
        report.total_blocks, report.total_volume
    );
    if report.regions.len() > 1 {
        let _ = writeln!(out, "  regions:");
        for region in &report.regions {
            let (w, h, l) = region.dimensions;
            let _ = writeln!(
                out,
                "    {}  {w}x{h}x{l}  {} blocks",
                region.name, region.blocks
            );
        }
    }
    match &report.embedded_test {
        Some(test) if test.parse_error.is_some() => {
            let _ = writeln!(
                out,
                "  test        embedded, but unreadable: {}",
                test.parse_error.as_deref().unwrap_or("?")
            );
        }
        Some(test) => {
            let _ = writeln!(
                out,
                "  test        {} case(s): {}",
                test.cases,
                test.names.join("; ")
            );
        }
        None => {
            let _ = writeln!(out, "  test        none embedded");
        }
    }
    let _ = writeln!(out, "  palette     ({} states):", report.palette.len());
    for (descriptor, count) in report.palette.iter().take(20) {
        let _ = writeln!(out, "    {count:>8}  {descriptor}");
    }
    if report.palette.len() > 20 {
        let _ = writeln!(out, "    … {} more", report.palette.len() - 20);
    }
    let _ = writeln!(out, "  entities    {}", report.entities.len());
    for entity in report.entities.iter().take(10) {
        let _ = writeln!(out, "    {entity}");
    }
    if report.entities.len() > 10 {
        let _ = writeln!(out, "    … {} more", report.entities.len() - 10);
    }
    let _ = writeln!(out, "  block entities  {}", report.block_entities.len());
    for be in report.block_entities.iter().take(10) {
        let _ = writeln!(out, "    {be}");
    }
    if report.block_entities.len() > 10 {
        let _ = writeln!(out, "    … {} more", report.block_entities.len() - 10);
    }
    out
}
