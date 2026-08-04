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
}

/// Load `path` through the `FormatManager` and summarise it.
pub(crate) fn gather(path: &Path) -> Result<FileReport, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let manager = nucleation::formats::manager::get_manager();
    let manager = manager.lock().map_err(|e| format!("format manager: {e}"))?;
    let format = manager
        .detect_format(&bytes)
        .ok_or_else(|| format!("{}: no importer recognises it", path.display()))?;
    let schematic =
        manager.read(&bytes).map_err(|e| format!("{}: unreadable: {e:?}", path.display()))?;
    Ok(from_schematic(path.to_path_buf(), bytes.len() as u64, format, &schematic))
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
            format!("{} @ ({:.1}, {:.1}, {:.1})", e.id, e.position.0, e.position.1, e.position.2)
        })
        .collect();
    let block_entities = schematic
        .get_block_entities_as_list()
        .into_iter()
        .map(|be| {
            format!("{} @ ({}, {}, {})", be.id, be.position.0, be.position.1, be.position.2)
        })
        .collect();

    let embedded_test = schematic.metadata.embedded_test.as_deref().map(|spec| {
        let pretty = serde_json::from_str::<serde_json::Value>(spec)
            .ok()
            .and_then(|v| serde_json::to_string_pretty(&v).ok());
        match mc_test::parse_suite(spec, &path.display().to_string()) {
            Ok(cases) => TestSummary {
                cases: cases.len(),
                names: cases.iter().map(|c| c.name.clone()).collect(),
                parse_error: None,
                pretty,
            },
            Err(e) => TestSummary { cases: 0, names: Vec::new(), parse_error: Some(e), pretty },
        }
    });

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
        let report =
            from_schematic(PathBuf::from("sample.litematic"), 123, "litematic".into(), &sample());
        assert_eq!(report.total_blocks, 4);
        assert_eq!(report.palette[0], ("minecraft:stone".to_string(), 3), "sorted by count");
        assert_eq!(report.palette[1].0, "minecraft:glass");
        assert_eq!(report.entities.len(), 1);
        assert!(report.entities[0].starts_with("minecraft:pig"), "{}", report.entities[0]);
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
        let test = report.embedded_test.expect("the tag exists, so it is reported");
        assert_eq!(test.cases, 0);
        assert!(test.parse_error.is_some());
    }
}
