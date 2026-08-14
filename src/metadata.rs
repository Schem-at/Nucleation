use quartz_nbt::{NbtCompound, NbtTag};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Current version of Nucleation's embedded schematic provenance contract.
pub const SCHEMATIC_PROVENANCE_VERSION: u32 = 1;
/// On-disk NBT key used by every format that can carry custom metadata.
pub const SCHEMATIC_PROVENANCE_NBT_KEY: &str = "NucleationProvenance";
/// On-disk NBT key for processing history, kept separate from source provenance.
pub const TRANSFORMATION_HISTORY_NBT_KEY: &str = "NucleationTransformationHistory";

/// One applied, versioned processing plan. This deliberately contains no
/// timestamp: deterministic transforms should produce deterministic metadata.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransformationRecord {
    /// Transformation-record schema version.
    pub schema_version: u32,
    /// Schema version of the serialized plan that was applied.
    #[serde(default)]
    pub plan_schema_version: u32,
    pub plan_name: String,
    pub plan_id: String,
    pub lossless: bool,
    pub quarantined: bool,
    #[serde(default)]
    pub summary: BTreeMap<String, u64>,
    /// Machine-readable checks performed when this history record was made.
    /// Values are stable strings such as `passed`, `failed`, and
    /// `not_applicable` so older readers can preserve unknown checks.
    #[serde(default)]
    pub verification: BTreeMap<String, String>,
}

/// Inclusive world-space bounds for extracted schematic content.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProvenanceBounds {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

impl ProvenanceBounds {
    pub fn new(min: [i32; 3], max: [i32; 3]) -> Result<Self, String> {
        if min.iter().zip(max.iter()).any(|(lo, hi)| lo > hi) {
            return Err("provenance bounds must be inclusive min <= max on every axis".into());
        }
        Ok(Self { min, max })
    }
}

/// Standard source metadata carried by a schematic independently of its file
/// format. Coordinates are absolute Minecraft world coordinates; `origin` is
/// the world position corresponding to schematic-local `(0, 0, 0)`.
///
/// The typed common fields make catalogs interoperable. `attributes` is the
/// namespaced extension point for source-specific identifiers without changing
/// the schema (for example `example.org:claim_id`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SchematicProvenance {
    pub schema_version: u32,
    pub source_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimension: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_bbox: Option<ProvenanceBounds>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<[i32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partition_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_build_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extracted_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_hash: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

impl SchematicProvenance {
    pub fn new(source_id: impl Into<String>) -> Result<Self, String> {
        let source_id = source_id.into();
        if source_id.trim().is_empty() {
            return Err("provenance source_id must not be empty".into());
        }
        Ok(Self {
            schema_version: SCHEMATIC_PROVENANCE_VERSION,
            source_id,
            world_name: None,
            map_name: None,
            dimension: None,
            snapshot_id: None,
            world_bbox: None,
            origin: None,
            partition_id: None,
            stable_build_id: None,
            extracted_at: None,
            config_hash: None,
            profile_hash: None,
            attributes: BTreeMap::new(),
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != SCHEMATIC_PROVENANCE_VERSION {
            return Err(format!(
                "unsupported schematic provenance schema version {}",
                self.schema_version
            ));
        }
        if self.source_id.trim().is_empty() {
            return Err("provenance source_id must not be empty".into());
        }
        if let Some(bounds) = &self.world_bbox {
            ProvenanceBounds::new(bounds.min, bounds.max)?;
        }
        if self.attributes.keys().any(|key| !key.contains(':')) {
            return Err("provenance attribute keys must be namespaced (contain ':')".into());
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|error| error.to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let provenance: Self = serde_json::from_str(json).map_err(|error| error.to_string())?;
        provenance.validate()?;
        Ok(provenance)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct Metadata {
    pub name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub created: Option<u64>,
    pub modified: Option<u64>,
    pub lm_version: Option<i32>,
    pub mc_version: Option<i32>,
    pub we_version: Option<i32>,
    /// Standard origin/source metadata. Native Nucleation serialization keeps
    /// the typed value; `.schem` and `.litematic` store its canonical JSON at
    /// `NucleationProvenance` in their Metadata compound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<SchematicProvenance>,
    /// Applied processing plans. Source provenance remains immutable; this is
    /// a separate append-only, content-addressed audit trail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transformation_history: Vec<TransformationRecord>,
    /// The Minecraft data version of the *file this schematic was loaded from*,
    /// captured by importers. Distinct from `mc_version` (which doubles as the
    /// export fallback). Drives forward-conversion to canonical on load and is
    /// the `from` version for reverse-conversion on save. `None` for formats with
    /// no Java data version (classic `.schematic`, Bedrock `.mcstructure`) or a
    /// freshly-constructed schematic. Not serialized — it is purely transient
    /// load-time provenance.
    #[serde(default, skip)]
    pub source_data_version: Option<i32>,
    /// A test scenario carried *inside* the schematic file, as the JSON
    /// descriptor `crates/mc-tick/tests/cases/README.md` documents.
    ///
    /// This is what makes a build its own regression test: a scenario is a file
    /// you drop in a folder rather than Rust somebody has to compile. Only
    /// `.litematic` carries it today, in a root-level `NucleationTest` compound
    /// beside `Metadata` — root-level so Litematica ignores it, and preserved
    /// on re-save so a build edited in-game and saved again does not silently
    /// lose its test.
    ///
    /// Not serialized: it is file-carried provenance like
    /// [`Metadata::source_data_version`], written and read by the format
    /// modules that know where to put it.
    #[serde(default, skip)]
    pub embedded_test: Option<String>,
    /// The schematic's embedded [`CellContract`](crate::io_contract::CellContract)
    /// as JSON — what makes a saved build a self-describing typed cell.
    ///
    /// Carried in the `.schem` `Metadata` compound as the
    /// `NucleationCellContract` string (beside `NucleationDefinitions`) and
    /// autodetected on open. File-carried provenance like
    /// [`Metadata::embedded_test`]: not serialized here, written and read by
    /// the format modules that know where to put it.
    #[serde(default, skip)]
    pub cell_contract: Option<String>,
}

impl Metadata {
    pub fn new(
        name: Option<String>,
        author: Option<String>,
        description: Option<String>,
        created: Option<u64>,
        modified: Option<u64>,
        lm_version: Option<i32>,
        mc_version: Option<i32>,
        we_version: Option<i32>,
    ) -> Self {
        Metadata {
            name,
            author,
            description,
            created,
            modified,
            lm_version,
            mc_version,
            we_version,
            provenance: None,
            transformation_history: Vec::new(),
            source_data_version: None,
            embedded_test: None,
            cell_contract: None,
        }
    }

    pub fn to_nbt(&self) -> NbtTag {
        let mut compound = NbtCompound::new();

        if let Some(name) = &self.name {
            compound.insert("Name", NbtTag::String(name.clone()));
        }
        if let Some(author) = &self.author {
            compound.insert("Author", NbtTag::String(author.clone()));
        }
        if let Some(description) = &self.description {
            compound.insert("Description", NbtTag::String(description.clone()));
        }
        if let Some(created) = self.created {
            compound.insert("TimeCreated", NbtTag::Long(created as i64));
        }
        if let Some(modified) = self.modified {
            compound.insert("TimeModified", NbtTag::Long(modified as i64));
        }
        if let Some(lm_version) = self.lm_version {
            compound.insert("lm_version", NbtTag::Int(lm_version));
        }
        if let Some(mc_version) = self.mc_version {
            compound.insert("mc_version", NbtTag::Int(mc_version));
        }
        if let Some(we_version) = self.we_version {
            compound.insert("we_version", NbtTag::Int(we_version));
        }
        if let Some(provenance) = &self.provenance {
            if let Ok(json) = provenance.to_json() {
                compound.insert(SCHEMATIC_PROVENANCE_NBT_KEY, NbtTag::String(json));
            }
        }
        if !self.transformation_history.is_empty() {
            if let Ok(json) = serde_json::to_string(&self.transformation_history) {
                compound.insert(TRANSFORMATION_HISTORY_NBT_KEY, NbtTag::String(json));
            }
        }

        NbtTag::Compound(compound)
    }

    pub fn from_nbt(nbt: &NbtCompound) -> Result<Self, String> {
        let name = nbt
            .get::<_, &str>("Name")
            .map_err(|_| "")
            .ok()
            .map(|s| s.to_string());
        let author = nbt
            .get::<_, &str>("Author")
            .map_err(|_| "")
            .ok()
            .map(|s| s.to_string());
        let description = nbt
            .get::<_, &str>("Description")
            .map_err(|_| "")
            .ok()
            .map(|s| s.to_string());
        let created = nbt
            .get::<_, i64>("TimeCreated")
            .map_err(|_| 0)
            .ok()
            .map(|v| v as u64);
        let modified = nbt
            .get::<_, i64>("TimeModified")
            .map_err(|_| 0)
            .ok()
            .map(|v| v as u64);
        let lm_version = nbt.get::<_, i32>("lm_version").map_err(|_| 0).ok();
        let mc_version = nbt.get::<_, i32>("mc_version").map_err(|_| 0).ok();
        let we_version = nbt.get::<_, i32>("we_version").map_err(|_| 0).ok();

        let provenance = nbt
            .get::<_, &str>(SCHEMATIC_PROVENANCE_NBT_KEY)
            .ok()
            .and_then(|json| SchematicProvenance::from_json(json).ok());

        let mut metadata = Metadata::new(
            name,
            author,
            description,
            created,
            modified,
            lm_version,
            mc_version,
            we_version,
        );
        metadata.provenance = provenance;
        metadata.transformation_history = nbt
            .get::<_, &str>(TRANSFORMATION_HISTORY_NBT_KEY)
            .ok()
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default();
        Ok(metadata)
    }
}
