//! `FileReport`: everything the inspector and `info` show, gathered once.
//!
//! One loader feeds three consumers — `nucleation info` (text/JSON), the
//! non-TTY `inspect` fallback, and the TUI inspector — so they can never
//! disagree about what a file contains.

use std::path::{Path, PathBuf};

use nucleation::UniversalSchematic;

/// One region's line in the overview.
pub(crate) struct RegionReport {
    pub(crate) name: String,
    pub(crate) dimensions: (i32, i32, i32),
    pub(crate) blocks: usize,
}

/// The embedded test, summarised. `parse_error` set means the tag exists but
/// its JSON does not parse — worth showing, never worth hiding.
pub(crate) struct TestSummary {
    pub(crate) cases: usize,
    pub(crate) names: Vec<String>,
    pub(crate) parse_error: Option<String>,
    /// The suite re-serialised with indentation, for the inspector's test tab.
    pub(crate) pretty: Option<String>,
    /// One entry per case: what it does in plain terms, then its own JSON —
    /// the navigable view for suites with more than one case.
    pub(crate) case_views: Vec<CaseView>,
}

/// One case of the embedded suite, rendered for reading.
pub(crate) struct CaseView {
    pub(crate) name: String,
    /// Digest first (settle, seed, ticks, what is claimed), JSON after.
    pub(crate) text: String,
}

/// A dense colour-per-cell grid for the terminal 3D preview. `0` is empty;
/// anything else is `0xRRGGBB` for the block occupying the cell.
pub(crate) struct VoxelGrid {
    pub(crate) dims: (usize, usize, usize),
    pub(crate) cells: Vec<u32>,
}

/// Builds bigger than this many cells skip the preview grid rather than
/// stalling every `gather` — the inspector says so instead of guessing.
const PREVIEW_CELL_CAP: i64 = 8_000_000;

/// Split a parsed suite into its per-case JSON values, whatever carrier
/// shape it uses (bare case, array, or `{"format":1,"cases":[...]}`).
fn case_values(spec: &serde_json::Value) -> Vec<serde_json::Value> {
    match spec {
        serde_json::Value::Array(cases) => cases.clone(),
        serde_json::Value::Object(map) => match map.get("cases") {
            Some(serde_json::Value::Array(cases)) => cases.clone(),
            _ => vec![spec.clone()],
        },
        _ => Vec::new(),
    }
}

/// The plain-terms half of a [`CaseView`]: what the case sets up and what
/// it claims, before the reader ever has to look at JSON.
fn case_digest(case: &serde_json::Value) -> String {
    let mut out = String::new();
    let field = |key: &str| case.get(key);
    let mut line = String::new();
    if let Some(v) = field("settle") {
        line.push_str(&format!("settle {}  ", v.as_str().unwrap_or("?")));
    }
    if let Some(v) = field("seed").and_then(|v| v.as_i64()) {
        line.push_str(&format!("seed {v}  "));
    }
    if let Some(v) = field("random_ticks").and_then(|v| v.as_i64()) {
        line.push_str(&format!("randomTickSpeed {v}  "));
    }
    if let Some(v) = field("setup").and_then(|v| v.as_i64()) {
        line.push_str(&format!("{v} setup tick(s)  "));
    }
    if let Some(v) = field("origin") {
        line.push_str(&format!("origin {v}  "));
    }
    if !line.is_empty() {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    if let Some(actions) = field("actions").and_then(|v| v.as_array()) {
        out.push_str(&format!("{} action(s):\n", actions.len()));
        for action in actions {
            let at = action
                .get("tick")
                .map(|t| format!("t{t}"))
                .unwrap_or_default();
            let what = action
                .get("place")
                .map(|p| format!("place {p}"))
                .or_else(|| action.get("set").map(|s| format!("set {s}")))
                .or_else(|| action.get("remove").map(|r| format!("remove {r}")))
                .unwrap_or_else(|| action.to_string());
            let pos = action
                .get("pos")
                .map(|p| format!(" at {p}"))
                .unwrap_or_default();
            out.push_str(&format!(
                "  {at} {what}{pos}
"
            ));
        }
    }
    if let Some(inert) = field("inert").and_then(|v| v.as_array()) {
        let names: Vec<String> = inert
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        out.push_str(&format!("asserted inert: {}\n", names.join(", ")));
    }
    if let Some(checks) = field("checks").and_then(|v| v.as_array()) {
        out.push_str(&format!("{} check(s):\n", checks.len()));
        for check in checks {
            let at = check
                .get("tick")
                .map(|t| format!("t{t}"))
                .unwrap_or_default();
            let expect = check.get("expect").and_then(|e| e.as_str()).unwrap_or("?");
            let detail = match expect {
                "blocks" => check
                    .get("blocks")
                    .and_then(|b| b.as_object())
                    .map(|b| format!(" — {} cell(s)", b.len()))
                    .unwrap_or_default(),
                "entities" => check
                    .get("entities")
                    .and_then(|e| e.as_array())
                    .map(|e| format!(" — {} kind(s)", e.len()))
                    .unwrap_or_default(),
                _ => String::new(),
            };
            out.push_str(&format!(
                "  {at} expect {expect}{detail}
"
            ));
        }
    }
    if let Some(events) = field("events").and_then(|v| v.as_array()) {
        out.push_str(&format!("{} block-change claim(s):\n", events.len()));
        for event in events {
            let pos = event.get("pos").map(|p| p.to_string()).unwrap_or_default();
            let to = event.get("to").and_then(|t| t.as_str()).unwrap_or("?");
            let when = event
                .get("tick")
                .map(|t| format!(" at t{t}"))
                .or_else(|| event.get("after").map(|a| format!(" after t{a}")))
                .unwrap_or_default();
            out.push_str(&format!(
                "  {pos} becomes {to}{when}
"
            ));
        }
    }
    out
}

/// Everything `info`/`inspect` show about one file.
pub(crate) struct FileReport {
    pub(crate) path: PathBuf,
    pub(crate) bytes: u64,
    pub(crate) format: String,
    pub(crate) name: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) data_version: Option<i32>,
    pub(crate) dimensions: (i32, i32, i32),
    pub(crate) total_blocks: i32,
    pub(crate) total_volume: i32,
    pub(crate) regions: Vec<RegionReport>,
    /// Descriptor → count, sorted by count descending then name.
    pub(crate) palette: Vec<(String, usize)>,
    /// `"minecraft:pig @ (1.5, 2.0, 3.5)"`.
    pub(crate) entities: Vec<String>,
    /// `"minecraft:chest @ (1, 2, 3)"`.
    pub(crate) block_entities: Vec<String>,
    pub(crate) embedded_test: Option<TestSummary>,
    /// Colour-per-cell grid for the 3D view tab; `None` when the build is
    /// over the preview cap, with the tab saying why.
    pub(crate) voxels: Option<VoxelGrid>,
}

/// Load `path` through the `FormatManager` and summarise it. `-` reads a
/// schematic from stdin, so `something | nucleation info -` works.
pub(crate) fn gather(path: &Path) -> Result<FileReport, String> {
    let bytes = crate::commands::io::read_input(path)?;
    let manager = nucleation::formats::manager::get_manager();
    let manager = manager.lock().map_err(|e| format!("format manager: {e}"))?;
    let format = manager
        .detect_format(&bytes)
        .ok_or_else(|| format!("{}: no importer recognises it", path.display()))?;
    let schematic = manager
        .read(&bytes)
        .map_err(|e| format!("{}: unreadable: {e:?}", path.display()))?;
    Ok(from_schematic(
        path.to_path_buf(),
        bytes.len() as u64,
        format,
        &schematic,
    ))
}

/// Summarise an already-loaded schematic (the testable half of [`gather`]).
pub(crate) fn from_schematic(
    path: PathBuf,
    bytes: u64,
    format: String,
    schematic: &UniversalSchematic,
) -> FileReport {
    // Regions materialise air densely, so `count_block_types` reports it in
    // chunk-volume quantities; a palette listing is about the build.
    let mut palette: Vec<(String, usize)> = schematic
        .count_block_types()
        .into_iter()
        .filter(|(state, _)| state.name != "minecraft:air")
        .map(|(state, count)| (state.to_string(), count))
        .collect();
    palette.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut regions: Vec<RegionReport> = schematic
        .get_all_regions()
        .iter()
        .map(|(name, region)| RegionReport {
            name: name.to_string(),
            dimensions: region.get_dimensions(),
            blocks: region.count_blocks(),
        })
        .collect();
    regions.sort_by(|a, b| a.name.cmp(&b.name));

    let entities = schematic
        .get_entities_as_list()
        .into_iter()
        .map(|e| {
            format!(
                "{} @ ({:.1}, {:.1}, {:.1})",
                e.id, e.position.0, e.position.1, e.position.2
            )
        })
        .collect();
    let block_entities = schematic
        .get_block_entities_as_list()
        .into_iter()
        .map(|be| {
            format!(
                "{} @ ({}, {}, {})",
                be.id, be.position.0, be.position.1, be.position.2
            )
        })
        .collect();

    let embedded_test = schematic.metadata.embedded_test.as_deref().map(|spec| {
        let value = serde_json::from_str::<serde_json::Value>(spec).ok();
        let pretty = value
            .as_ref()
            .and_then(|v| serde_json::to_string_pretty(v).ok());
        let case_views = value
            .as_ref()
            .map(|v| {
                case_values(v)
                    .into_iter()
                    .enumerate()
                    .map(|(index, case)| {
                        let name = case
                            .get("name")
                            .and_then(|n| n.as_str())
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("case {}", index + 1));
                        let digest = case_digest(&case);
                        let json = serde_json::to_string_pretty(&case).unwrap_or_default();
                        CaseView {
                            name,
                            text: format!(
                                "{digest}
{json}"
                            ),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        match mc_test::parse_suite(spec, &path.display().to_string()) {
            Ok(cases) => TestSummary {
                cases: cases.len(),
                names: cases.iter().map(|c| c.name.clone()).collect(),
                parse_error: None,
                pretty,
                case_views,
            },
            Err(e) => TestSummary {
                cases: 0,
                names: Vec::new(),
                parse_error: Some(e),
                pretty,
                case_views,
            },
        }
    });

    // The preview grid: bounding box scanned once, colour per occupied cell.
    let bbox = schematic.get_bounding_box();
    let (dx, dy, dz) = (
        i64::from(bbox.max.0 - bbox.min.0) + 1,
        i64::from(bbox.max.1 - bbox.min.1) + 1,
        i64::from(bbox.max.2 - bbox.min.2) + 1,
    );
    let voxels = if dx > 0 && dy > 0 && dz > 0 && dx * dy * dz <= PREVIEW_CELL_CAP {
        let dims = (dx as usize, dy as usize, dz as usize);
        let mut cells = vec![0u32; dims.0 * dims.1 * dims.2];
        for (pos, block) in schematic.iter_blocks() {
            if block.name == "minecraft:air" {
                continue;
            }
            let (x, y, z) = (
                (pos.x - bbox.min.0) as usize,
                (pos.y - bbox.min.1) as usize,
                (pos.z - bbox.min.2) as usize,
            );
            if x < dims.0 && y < dims.1 && z < dims.2 {
                cells[(y * dims.2 + z) * dims.0 + x] = crate::tui::voxel::block_color(&block.name);
            }
        }
        Some(VoxelGrid { dims, cells })
    } else {
        None
    };

    FileReport {
        path,
        bytes,
        format,
        name: schematic.metadata.name.clone(),
        author: schematic.metadata.author.clone(),
        description: schematic.metadata.description.clone(),
        data_version: schematic.metadata.mc_version,
        dimensions: schematic.get_dimensions(),
        total_blocks: schematic.total_blocks(),
        total_volume: schematic.total_volume(),
        regions,
        palette,
        entities,
        block_entities,
        embedded_test,
        voxels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nucleation::{BlockState, UniversalSchematic};

    fn sample() -> UniversalSchematic {
        let mut schem = UniversalSchematic::new("sample".into());
        let stone = BlockState::new("minecraft:stone".to_string());
        for x in 0..3 {
            schem.set_block(x, 0, 0, &stone);
        }
        schem.set_block(0, 1, 0, &BlockState::new("minecraft:glass".to_string()));
        schem.add_entity(nucleation::Entity::new(
            "minecraft:pig".to_string(),
            (0.5, 1.0, 0.5),
        ));
        schem.metadata.embedded_test = Some(
            r#"[{"name":"a","checks":[{"tick":0,"expect":"quiescent"}]},
                {"name":"b","checks":[{"tick":0,"expect":"quiescent"}]}]"#
                .to_string(),
        );
        schem
    }

    #[test]
    fn a_report_counts_what_the_file_holds() {
        let report = from_schematic(
            PathBuf::from("sample.litematic"),
            123,
            "litematic".into(),
            &sample(),
        );
        assert_eq!(report.total_blocks, 4);
        assert_eq!(
            report.palette[0],
            ("minecraft:stone".to_string(), 3),
            "sorted by count"
        );
        assert_eq!(report.palette[1].0, "minecraft:glass");
        assert_eq!(report.entities.len(), 1);
        assert!(
            report.entities[0].starts_with("minecraft:pig"),
            "{}",
            report.entities[0]
        );
        let test = report.embedded_test.expect("the suite is summarised");
        assert_eq!(test.cases, 2);
        assert_eq!(test.names, vec!["a", "b"]);
        assert!(test.parse_error.is_none());
    }

    #[test]
    fn a_broken_embedded_test_reports_its_error_rather_than_vanishing() {
        let mut schem = sample();
        schem.metadata.embedded_test = Some("{not json".to_string());
        let report = from_schematic(PathBuf::from("x"), 0, "litematic".into(), &schem);
        let test = report
            .embedded_test
            .expect("the tag exists, so it is reported");
        assert_eq!(test.cases, 0);
        assert!(test.parse_error.is_some());
    }
}
