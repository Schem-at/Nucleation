use crate::universal_schematic::UniversalSchematic;
use std::error::Error;
use std::sync::{Arc, Mutex, OnceLock};

pub trait SchematicImporter: Send + Sync {
    fn name(&self) -> String;
    fn detect(&self, data: &[u8]) -> bool;
    fn read(&self, data: &[u8]) -> Result<UniversalSchematic, Box<dyn Error>>;
}

pub trait SchematicExporter: Send + Sync {
    fn name(&self) -> String;
    fn extensions(&self) -> Vec<String>;
    fn available_versions(&self) -> Vec<String>;
    fn default_version(&self) -> String;
    fn write(
        &self,
        schematic: &UniversalSchematic,
        version: Option<&str>,
    ) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub struct FormatManager {
    importers: Vec<Box<dyn SchematicImporter>>,
    exporters: Vec<Box<dyn SchematicExporter>>,
}

impl FormatManager {
    pub fn new() -> Self {
        Self {
            importers: Vec::new(),
            exporters: Vec::new(),
        }
    }

    pub fn register_importer<I: SchematicImporter + 'static>(&mut self, importer: I) {
        self.importers.push(Box::new(importer));
    }

    pub fn register_exporter<E: SchematicExporter + 'static>(&mut self, exporter: E) {
        self.exporters.push(Box::new(exporter));
    }

    pub fn detect_format(&self, data: &[u8]) -> Option<String> {
        for importer in &self.importers {
            if importer.detect(data) {
                return Some(importer.name());
            }
        }
        None
    }

    pub fn read(&self, data: &[u8]) -> Result<UniversalSchematic, Box<dyn Error>> {
        for importer in &self.importers {
            if importer.detect(data) {
                return importer.read(data);
            }
        }
        Err("Unknown or unsupported schematic format".into())
    }

    pub fn write(
        &self,
        format: &str,
        schematic: &UniversalSchematic,
        version: Option<&str>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        for exporter in &self.exporters {
            if exporter.name().eq_ignore_ascii_case(format) {
                return exporter.write(schematic, version);
            }
        }
        Err(format!("Unsupported export format: {}", format).into())
    }

    pub fn write_auto(
        &self,
        path: &str,
        schematic: &UniversalSchematic,
        version: Option<&str>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        for exporter in &self.exporters {
            if exporter.extensions().contains(&extension) {
                return exporter.write(schematic, version);
            }
        }
        Err(format!("Could not determine format from extension: .{}", extension).into())
    }

    pub fn list_importers(&self) -> Vec<String> {
        self.importers.iter().map(|i| i.name()).collect()
    }

    pub fn list_exporters(&self) -> Vec<String> {
        self.exporters.iter().map(|e| e.name()).collect()
    }

    pub fn get_exporter_versions(&self, format: &str) -> Option<Vec<String>> {
        for exporter in &self.exporters {
            if exporter.name().eq_ignore_ascii_case(format) {
                return Some(exporter.available_versions());
            }
        }
        None
    }

    pub fn get_exporter_default_version(&self, format: &str) -> Option<String> {
        for exporter in &self.exporters {
            if exporter.name().eq_ignore_ascii_case(format) {
                return Some(exporter.default_version());
            }
        }
        None
    }
}

pub static MANAGER: OnceLock<Arc<Mutex<FormatManager>>> = OnceLock::new();

pub fn get_manager() -> Arc<Mutex<FormatManager>> {
    MANAGER
        .get_or_init(|| {
            let mut manager = FormatManager::new();
            manager.register_importer(crate::formats::litematic::LitematicFormat);
            manager.register_exporter(crate::formats::litematic::LitematicFormat);
            manager.register_importer(crate::formats::schematic::SchematicFormat);
            manager.register_exporter(crate::formats::schematic::SchematicFormat);
            manager.register_importer(crate::formats::mcstructure::McStructureFormat);
            manager.register_exporter(crate::formats::mcstructure::McStructureFormat);
            Arc::new(Mutex::new(manager))
        })
        .clone()
}
