use crate::formats::manager::{SchematicExporter, SchematicImporter};
use crate::universal_schematic::UniversalSchematic;
use std::error::Error;

const MAGIC: &[u8; 4] = b"NUSN";
const VERSION: u32 = 1;

pub struct SnapshotFormat;

impl SchematicImporter for SnapshotFormat {
    fn name(&self) -> String {
        "snapshot".to_string()
    }

    fn detect(&self, data: &[u8]) -> bool {
        data.len() >= 4 && &data[0..4] == MAGIC
    }

    fn read(&self, data: &[u8]) -> Result<UniversalSchematic, Box<dyn Error>> {
        from_snapshot(data)
    }
}

impl SchematicExporter for SnapshotFormat {
    fn name(&self) -> String {
        "snapshot".to_string()
    }

    fn extensions(&self) -> Vec<String> {
        vec!["nusn".to_string()]
    }

    fn available_versions(&self) -> Vec<String> {
        vec!["1".to_string()]
    }

    fn default_version(&self) -> String {
        "1".to_string()
    }

    fn write(
        &self,
        schematic: &UniversalSchematic,
        _version: Option<&str>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        to_snapshot(schematic)
    }
}

pub fn to_snapshot(schematic: &UniversalSchematic) -> Result<Vec<u8>, Box<dyn Error>> {
    let payload = bincode::serialize(schematic)?;
    let mut buf = Vec::with_capacity(8 + payload.len());
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    buf.extend_from_slice(&payload);
    Ok(buf)
}

pub fn from_snapshot(data: &[u8]) -> Result<UniversalSchematic, Box<dyn Error>> {
    if data.len() < 8 {
        return Err("Snapshot data too short".into());
    }
    if &data[0..4] != MAGIC {
        return Err("Invalid snapshot magic bytes".into());
    }
    let version = u32::from_le_bytes(data[4..8].try_into()?);
    if version != VERSION {
        return Err(format!("Unsupported snapshot version: {}", version).into());
    }
    let mut schematic: UniversalSchematic = bincode::deserialize(&data[8..])?;

    // Rebuild cached fields that are #[serde(skip)] on Region
    rebuild_region(&mut schematic.default_region);
    for region in schematic.other_regions.values_mut() {
        rebuild_region(region);
    }

    Ok(schematic)
}

fn rebuild_region(region: &mut crate::region::Region) {
    region.rebuild_bbox();
    region.rebuild_palette_index();
    region.rebuild_air_index();
    region.rebuild_non_air_count();
    region.rebuild_tight_bounds();
}
