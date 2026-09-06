//! The core `Schematic` opaque, wrapping [`crate::UniversalSchematic`], plus the
//! `BlockState` opaque. Port of `ffi/schematic.rs`.
//!
//! Omitted from port (obsolete by construction — destructors and error transport are
//! generated): `schematic_free`, `blockstate_free`, `free_file_map`, `free_entity_array`.
//! `schematic_new` is covered by [`ffi::Schematic::create`] (the old fn hard-coded the
//! name "Default"; pass any name here).

use crate::bridge::shared::ffi::NucleationError;

/// Validate a `&DiplomatStr` (raw UTF-8 bytes) into `&str`.
fn utf8(bytes: &[u8]) -> Result<&str, NucleationError> {
    std::str::from_utf8(bytes).map_err(|_| NucleationError::InvalidArgument)
}

/// Standard base64 encode, shared with other bridge modules that need it but
/// aren't otherwise gated on `meshing` (e.g. `mc_tick`'s `selection_schematic_b64`)
/// — this module is unconditionally part of `bridge`, so it's reachable from any
/// feature combination that has `bridge` on. Keep this the only encoder: a second
/// copy is how the two stop agreeing.
pub(crate) fn b64(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn parse_excluded_blocks(json: &str) -> Result<Vec<crate::BlockState>, NucleationError> {
    if json.is_empty() {
        return Ok(Vec::new());
    }
    let strings: Vec<String> = serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
    strings
        .iter()
        .map(|block| {
            crate::UniversalSchematic::parse_block_string(block)
                .map(|(state, _)| state)
                .map_err(|_| NucleationError::Parse)
        })
        .collect()
}

/// Parse optional world-export options JSON (empty string ⇒ defaults).
fn parse_world_options(
    json: &str,
) -> Result<Option<crate::formats::world::WorldExportOptions>, NucleationError> {
    if json.is_empty() {
        return Ok(None);
    }
    serde_json::from_str(json)
        .map(Some)
        .map_err(|_| NucleationError::Parse)
}

/// One block as JSON, shaped like the old `CBlock` (properties as serialized pairs).
fn block_json(
    pos: &crate::block_position::BlockPosition,
    block: &crate::BlockState,
) -> serde_json::Value {
    serde_json::json!({
        "x": pos.x,
        "y": pos.y,
        "z": pos.z,
        "name": block.name.as_str(),
        "properties": serde_json::to_value(&block.properties).unwrap_or(serde_json::Value::Null),
    })
}

#[diplomat::bridge]
pub mod ffi {
    use super::super::shared::ffi::{BlockPos, Dimensions, NucleationError};
    use super::{b64, block_json, parse_excluded_blocks, parse_world_options, utf8};
    use crate::formats::{litematic, manager::get_manager, mcstructure};
    use crate::universal_schematic::ChunkLoadingStrategy;
    use diplomat_runtime::DiplomatWrite;
    use std::collections::HashMap;
    use std::fmt::Write;

    #[diplomat::opaque_mut]
    pub struct Schematic(pub(crate) crate::UniversalSchematic);

    /// Deterministic, lossless pieces returned by
    /// [`Schematic::split_connected_attach_nearby`]. Pieces are ordered by their
    /// largest connected component, largest first.
    #[diplomat::opaque]
    pub struct SchematicSplitResult(pub(crate) Vec<crate::UniversalSchematic>);

    impl SchematicSplitResult {
        pub fn len(&self) -> u32 {
            self.0.len() as u32
        }

        /// Return an independently owned piece by zero-based index.
        pub fn piece(&self, index: u32) -> Result<Box<Schematic>, NucleationError> {
            self.0
                .get(index as usize)
                .cloned()
                .map(|schematic| Box::new(Schematic(schematic)))
                .ok_or(NucleationError::NotFound)
        }
    }

    impl Schematic {
        /// Create a new, empty schematic with the given name.
        pub fn create(name: &DiplomatStr) -> Box<Schematic> {
            Box::new(Schematic(crate::UniversalSchematic::new(
                String::from_utf8_lossy(name).into_owned(),
            )))
        }

        /// Release all block/entity storage immediately, keeping an empty valid
        /// schematic handle. JS consumers should call this when a parsed world or
        /// editing session is no longer needed instead of waiting for finalizers.
        pub fn clear_contents(&mut self) {
            self.0 = crate::UniversalSchematic::new(String::new());
        }

        /// Return an independent deep copy. Subsequent block, region, entity,
        /// metadata, or transform changes do not affect the original.
        pub fn deep_clone(&self) -> Box<Schematic> {
            Box::new(Schematic(self.0.clone()))
        }

        /// Inspect a versioned transform-plan JSON document without modifying
        /// this schematic. Writes a deterministic audit-report JSON document.
        pub fn inspect_transform_plan_json(
            &self,
            plan_json: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let plan =
                crate::processing::TransformPlan::from_json(utf8(plan_json)?).map_err(|error| {
                    crate::bridge::set_last_error_detail(error.to_string());
                    NucleationError::InvalidArgument
                })?;
            let report = plan.inspect(&self.0).map_err(|error| {
                crate::bridge::set_last_error_detail(error.to_string());
                NucleationError::InvalidArgument
            })?;
            let _ = write!(out, "{}", report.to_json());
            Ok(())
        }

        /// Atomically apply a versioned transform-plan JSON document. Policy
        /// rejection is represented by `report.rejected == true` and leaves the
        /// schematic unchanged; malformed plans raise `InvalidArgument`.
        pub fn apply_transform_plan_json(
            &mut self,
            plan_json: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let plan =
                crate::processing::TransformPlan::from_json(utf8(plan_json)?).map_err(|error| {
                    crate::bridge::set_last_error_detail(error.to_string());
                    NucleationError::InvalidArgument
                })?;
            let report = match plan.apply(&mut self.0) {
                Ok(report) | Err(crate::processing::TransformError::Rejected(report)) => report,
                Err(error) => {
                    crate::bridge::set_last_error_detail(error.to_string());
                    return Err(NucleationError::InvalidArgument);
                }
            };
            let _ = write!(out, "{}", report.to_json());
            Ok(())
        }

        /// Apply the bundled deterministic, lossless canonicalization preset.
        pub fn canonicalize_json(&mut self, out: &mut DiplomatWrite) {
            let report = crate::processing::TransformPlan::canonical()
                .apply(&mut self.0)
                .expect("the built-in canonical plan is valid and non-rejecting");
            let _ = write!(out, "{}", report.to_json());
        }

        /// Inspect the bundled public-registry policy without modifying this
        /// schematic. Applications should review `rejected` and `quarantined`
        /// before choosing whether to call `apply_transform_plan_json`.
        pub fn inspect_registry_safe_json(&self, out: &mut DiplomatWrite) {
            let report = crate::processing::TransformPlan::registry_safe()
                .inspect(&self.0)
                .expect("the built-in registry-safe plan is valid");
            let _ = write!(out, "{}", report.to_json());
        }

        /// Split spatially independent machines while keeping nearby tiny
        /// detached parts with their machine. Components at least
        /// `min_standalone_blocks` large always remain independent; smaller
        /// components attach only directly to a core within `max_air_gap`.
        /// Attachment is non-transitive and the operation is lossless.
        pub fn split_connected_attach_nearby(
            &self,
            min_standalone_blocks: u32,
            max_air_gap: u32,
        ) -> Box<SchematicSplitResult> {
            Box::new(SchematicSplitResult(self.0.split_connected_attach_nearby(
                crate::selection::Connectivity::Corner,
                min_standalone_blocks as usize,
                max_air_gap,
            )))
        }

        /// The allocated dimensions (width, height, length) of the schematic's
        /// bounding box.
        pub fn dimensions(&self) -> Dimensions {
            let (x, y, z) = self.0.get_dimensions();
            Dimensions { x, y, z }
        }

        /// Returns `true` if a block was placed (out-of-range coordinates extend the
        /// schematic rather than erroring, matching `UniversalSchematic::set_block`).
        pub fn set_block(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            block_name: &DiplomatStr,
        ) -> Result<bool, NucleationError> {
            let name =
                std::str::from_utf8(block_name).map_err(|_| NucleationError::InvalidArgument)?;
            self.0
                .try_set_block_str(x, y, z, name)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// The name of the block at a position. `NotFound` if the position is
        /// outside every region.
        pub fn get_block_name(
            &self,
            x: i32,
            y: i32,
            z: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            match self.0.get_block(x, y, z) {
                Some(state) => {
                    let _ = write!(out, "{}", state.name);
                    Ok(())
                }
                None => Err(NucleationError::NotFound),
            }
        }

        /// Save the schematic to a file, picking the format from the file
        /// extension (`.litematic`, `.schem`, `.schematic`, `.mcstructure`,
        /// `.nbt`, `.nusn`; unknown extensions write Litematic). For an
        /// explicit format or version, use `save_to_file_with_format`.
        /// Not available in JS: the WASM build has no filesystem — use
        /// `save_as_b64` there.
        #[diplomat::attr(js, disable)]
        pub fn save_to_file(&self, path: &DiplomatStr) -> Result<(), NucleationError> {
            let path = std::str::from_utf8(path).map_err(|_| NucleationError::InvalidArgument)?;
            let manager = get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let bytes = manager
                .write_auto_with_settings(path, &self.0, None, None)
                .map_err(|_| NucleationError::Serialize)?;
            std::fs::write(path, bytes).map_err(|_| NucleationError::Io)?;
            Ok(())
        }

        /// Convenience alias for `save_to_file`, matching the established
        /// Python API (`schematic.save("build.schem")`).
        #[diplomat::attr(js, disable)]
        pub fn save(&self, path: &DiplomatStr) -> Result<(), NucleationError> {
            self.save_to_file(path)
        }

        /// Load a schematic from a file, auto-detecting the format from the
        /// contents (any supported format, whatever the extension says).
        /// Not available in JS: the WASM build has no filesystem — read the
        /// bytes yourself and use `from_data`.
        #[diplomat::attr(js, disable)]
        pub fn load_from_file(path: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            let path = std::str::from_utf8(path).map_err(|_| NucleationError::InvalidArgument)?;
            let bytes = std::fs::read(path).map_err(|_| NucleationError::Io)?;
            Self::from_data(&bytes)
        }

        /// Convenience alias for `load_from_file`, matching the established
        /// Python API (`Schematic.open("build.schem")`).
        #[diplomat::attr(js, disable)]
        pub fn open(path: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            Self::load_from_file(path)
        }

        // --- Data I/O (old fns populated an existing schematic; these construct) ---

        /// Build a schematic from raw byte data, auto-detecting the format.
        /// Supports Litematic, Sponge Schematic, and McStructure (Bedrock) formats.
        /// `Parse` if a format was detected but failed to parse, `InvalidArgument` if
        /// no format was recognized.
        pub fn from_data(data: &[u8]) -> Result<Box<Schematic>, NucleationError> {
            let manager = get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            match manager.read(data) {
                Ok(res) => Ok(Box::new(Schematic(res))),
                Err(_) => {
                    if manager.detect_format(data).is_some() {
                        Err(NucleationError::Parse)
                    } else {
                        Err(NucleationError::InvalidArgument)
                    }
                }
            }
        }

        /// Decode untrusted bytes using a serialized `DecodeLimits` object.
        /// Empty JSON selects the conservative library defaults. Limits are
        /// enforced while decompressing/parsing and again before region
        /// allocations are accepted.
        pub fn from_data_bounded(
            data: &[u8],
            limits_json: &DiplomatStr,
        ) -> Result<Box<Schematic>, NucleationError> {
            let limits = if limits_json.is_empty() {
                crate::formats::limits::DecodeLimits::default()
            } else {
                serde_json::from_str(utf8(limits_json)?)
                    .map_err(|_| NucleationError::InvalidArgument)?
            };
            let manager = get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            manager
                .read_bounded(data, &limits)
                .map(|schematic| Box::new(Schematic(schematic)))
                .map_err(|error| {
                    crate::bridge::set_last_error_detail(error.to_string());
                    NucleationError::Parse
                })
        }

        /// Build a schematic from Litematic data.
        pub fn from_litematic(data: &[u8]) -> Result<Box<Schematic>, NucleationError> {
            litematic::from_litematic(data)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::Parse)
        }

        /// The schematic as Litematic bytes, base64-encoded.
        pub fn to_litematic_b64(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let data = litematic::to_litematic(&self.0).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", b64(&data));
            Ok(())
        }

        /// Build a schematic from classic `.schematic` data.
        pub fn from_schematic(data: &[u8]) -> Result<Box<Schematic>, NucleationError> {
            crate::formats::schematic::from_schematic(data)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::Parse)
        }

        /// The schematic as classic `.schematic` bytes, base64-encoded.
        pub fn to_schematic_b64(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let data = crate::formats::schematic::to_schematic(&self.0)
                .map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", b64(&data));
            Ok(())
        }

        /// Build a schematic from snapshot (fast binary) data.
        pub fn from_snapshot(data: &[u8]) -> Result<Box<Schematic>, NucleationError> {
            crate::formats::snapshot::from_snapshot(data)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::Parse)
        }

        /// The schematic as snapshot (fast binary) bytes, base64-encoded.
        pub fn to_snapshot_b64(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let data = crate::formats::snapshot::to_snapshot(&self.0)
                .map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", b64(&data));
            Ok(())
        }

        /// Build a schematic from McStructure (Bedrock) data.
        pub fn from_mcstructure(data: &[u8]) -> Result<Box<Schematic>, NucleationError> {
            mcstructure::from_mcstructure(data)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::Parse)
        }

        /// The schematic as McStructure (Bedrock) bytes, base64-encoded.
        pub fn to_mcstructure_b64(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let data =
                mcstructure::to_mcstructure(&self.0).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", b64(&data));
            Ok(())
        }

        // --- MCA / World Import/Export ---

        /// Import from a single MCA region file.
        pub fn from_mca(data: &[u8]) -> Result<Box<Schematic>, NucleationError> {
            crate::formats::world::from_mca(data)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::Parse)
        }

        /// Import from MCA with coordinate bounds.
        pub fn from_mca_bounded(
            data: &[u8],
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<Schematic>, NucleationError> {
            crate::formats::world::from_mca_bounded(data, min_x, min_y, min_z, max_x, max_y, max_z)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::Parse)
        }

        /// Import from a zipped world folder.
        pub fn from_world_zip(data: &[u8]) -> Result<Box<Schematic>, NucleationError> {
            crate::formats::world::from_world_zip(data)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::Parse)
        }

        /// Import from zipped world with coordinate bounds.
        pub fn from_world_zip_bounded(
            data: &[u8],
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<Schematic>, NucleationError> {
            crate::formats::world::from_world_zip_bounded(
                data, min_x, min_y, min_z, max_x, max_y, max_z,
            )
            .map(|s| Box::new(Schematic(s)))
            .map_err(|_| NucleationError::Parse)
        }

        /// Import from a Minecraft world directory path.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn from_world_directory(path: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            let path = utf8(path)?;
            crate::formats::world::from_world_directory(std::path::Path::new(path))
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::Parse)
        }

        /// Import from world directory with coordinate bounds.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn from_world_directory_bounded(
            path: &DiplomatStr,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<Schematic>, NucleationError> {
            let path = utf8(path)?;
            crate::formats::world::from_world_directory_bounded(
                std::path::Path::new(path),
                min_x,
                min_y,
                min_z,
                max_x,
                max_y,
                max_z,
            )
            .map(|s| Box::new(Schematic(s)))
            .map_err(|_| NucleationError::Parse)
        }

        /// Export the schematic as a Minecraft world: a JSON array of
        /// `{"path": <relative file path>, "data_b64": <base64 bytes>}` entries
        /// (the old `CFileMap`). `options_json` may be empty for defaults.
        pub fn to_world_json(
            &self,
            options_json: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let options = parse_world_options(utf8(options_json)?)?;
            let files = crate::formats::world::to_world(&self.0, options)
                .map_err(|_| NucleationError::Serialize)?;
            let items: Vec<serde_json::Value> = files
                .into_iter()
                .map(|(path, data)| serde_json::json!({ "path": path, "data_b64": b64(&data) }))
                .collect();
            let json = serde_json::to_string(&items).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Export and write world files to a directory. `options_json` may be empty.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn save_world(
            &self,
            directory: &DiplomatStr,
            options_json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let dir = utf8(directory)?;
            let options = parse_world_options(utf8(options_json)?)?;
            crate::formats::world::save_world(&self.0, std::path::Path::new(dir), options)
                .map_err(|_| NucleationError::Io)
        }

        /// Export the schematic as a zipped Minecraft world, base64-encoded.
        /// `options_json` may be empty for defaults.
        pub fn to_world_zip_b64(
            &self,
            options_json: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let options = parse_world_options(utf8(options_json)?)?;
            let bytes = crate::formats::world::to_world_zip(&self.0, options)
                .map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", b64(&bytes));
            Ok(())
        }

        // --- Block Manipulation ---

        /// Set a block with properties given as a JSON object of string→string
        /// (the old `CProperty` array).
        pub fn set_block_with_properties(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            block_name: &DiplomatStr,
            properties_json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let name = utf8(block_name)?;
            let props_str = utf8(properties_json)?;
            let props: Vec<(smol_str::SmolStr, smol_str::SmolStr)> = if props_str.is_empty() {
                Vec::new()
            } else {
                let map: serde_json::Map<String, serde_json::Value> =
                    serde_json::from_str(props_str).map_err(|_| NucleationError::Parse)?;
                let mut props = Vec::with_capacity(map.len());
                for (k, v) in map {
                    let v = v.as_str().ok_or(NucleationError::InvalidArgument)?;
                    props.push((k.into(), v.into()));
                }
                props
            };
            let block_state = crate::BlockState {
                name: name.into(),
                properties: props,
            };
            self.0.set_block(x, y, z, &block_state);
            Ok(())
        }

        /// Set a block from a full block string, e.g.
        /// `minecraft:chest[facing=north]{Items:[...]}`.
        pub fn set_block_from_string(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            block_string: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let block_str = utf8(block_string)?;
            self.0
                .set_block_from_string(x, y, z, block_str)
                .map(|_| ())
                .map_err(|_| NucleationError::Parse)
        }

        /// Pre-resolve a plain block name to a palette index for use with `place`.
        /// Pair them in hot loops with many unique block names to skip the per-call
        /// name → palette lookup.
        pub fn prepare_block(&mut self, block_name: &DiplomatStr) -> Result<i32, NucleationError> {
            let name = utf8(block_name)?;
            Ok(self.0.default_region.get_or_insert_palette_by_name(name) as i32)
        }

        /// Place a block by pre-resolved palette index (from `prepare_block`).
        pub fn place(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            palette_index: i32,
        ) -> Result<(), NucleationError> {
            if palette_index < 0 {
                return Err(NucleationError::InvalidArgument);
            }
            let region = &mut self.0.default_region;
            if (palette_index as usize) >= region.palette.len() {
                return Err(NucleationError::InvalidArgument);
            }
            if !region.is_in_region(x, y, z) {
                region.expand_to_fit(x, y, z);
            }
            region.set_block_at_index_unchecked(palette_index as usize, x, y, z);
            Ok(())
        }

        /// Batch-set blocks at multiple positions to the same block (name, block
        /// string with properties, or block string with NBT). `positions` is flat
        /// `[x0,y0,z0, x1,y1,z1, ...]` (length must be a multiple of 3).
        /// Returns the number of blocks set.
        pub fn set_blocks(
            &mut self,
            positions: &[i32],
            block_name: &DiplomatStr,
        ) -> Result<i32, NucleationError> {
            let block_name_str = utf8(block_name)?;
            if positions.len() % 3 != 0 {
                return Err(NucleationError::InvalidArgument);
            }
            let count = positions.len() / 3;
            if count == 0 {
                return Ok(0);
            }
            let count_i32 = i32::try_from(count).map_err(|_| NucleationError::InvalidArgument)?;
            let s = &mut self.0;

            let (mut min_x, mut min_y, mut min_z) = (positions[0], positions[1], positions[2]);
            let (mut max_x, mut max_y, mut max_z) = (min_x, min_y, min_z);
            for i in 1..count {
                let (x, y, z) = (positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]);
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                min_z = min_z.min(z);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                max_z = max_z.max(z);
            }

            // Validate before mutating any position. The shared setter caches the
            // parsed result after the first placement, while keeping replacement,
            // jukebox-state, and block-entity behavior identical to set_block.
            crate::UniversalSchematic::parse_block_string(block_name_str)
                .map_err(|_| NucleationError::InvalidArgument)?;
            s.default_region
                .ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
            for position in positions.chunks_exact(3) {
                s.set_block_from_string(position[0], position[1], position[2], block_name_str)
                    .map_err(|_| NucleationError::InvalidArgument)?;
            }
            Ok(count_i32)
        }

        /// Sequentially hand-place the same block at many positions in one
        /// local simulated component. `positions` is flat
        /// `[x0,y0,z0, x1,y1,z1, ...]`; placements run in that order and each
        /// settles before the next. Returns the number of final cells written
        /// back, including neighbours changed by redstone or pistons.
        ///
        /// Nearby passive blocks are loaded as environmental context, but the
        /// write-back is confined to the active component's effect window. Its
        /// runtime is therefore independent of unrelated schematic volume.
        /// Use `set_blocks_simulated_full_world` to opt into global updates.
        ///
        /// This is the efficient bulk form of repeated `{simulate=true}`:
        /// structure conversion and simulator wiring happen once for the
        /// complete sequence. Propagation is not constant-time—a placement can
        /// affect an arbitrarily large circuit—but fixed setup is amortized.
        pub fn set_blocks_simulated(
            &mut self,
            positions: &[i32],
            block_name: &DiplomatStr,
        ) -> Result<i32, NucleationError> {
            if positions.len() % 3 != 0 {
                return Err(NucleationError::InvalidArgument);
            }
            let descriptor = utf8(block_name)?;
            let placements: Vec<(i32, i32, i32)> = positions
                .chunks_exact(3)
                .map(|p| (p[0], p[1], p[2]))
                .collect();

            #[cfg(feature = "mc-tick")]
            {
                let written = crate::bridge::mc_tick::simulate_placements_into(
                    &mut self.0,
                    &placements,
                    descriptor,
                )
                .map_err(|detail| {
                    crate::bridge::set_last_error_detail(detail);
                    NucleationError::Simulation
                })?;
                i32::try_from(written).map_err(|_| NucleationError::InvalidArgument)
            }
            #[cfg(not(feature = "mc-tick"))]
            {
                let _ = (placements, descriptor);
                crate::bridge::set_last_error_detail(
                    "set_blocks_simulated needs the mc-tick feature (included in bridge-full)",
                );
                Err(NucleationError::Simulation)
            }
        }

        /// Explicit full-world counterpart to `set_blocks_simulated`.
        /// Unrelated schematic volume participates in setup and any resulting
        /// changes anywhere in the loaded world are written back.
        pub fn set_blocks_simulated_full_world(
            &mut self,
            positions: &[i32],
            block_name: &DiplomatStr,
        ) -> Result<i32, NucleationError> {
            if positions.len() % 3 != 0 {
                return Err(NucleationError::InvalidArgument);
            }
            let descriptor = utf8(block_name)?;
            let placements: Vec<(i32, i32, i32)> = positions
                .chunks_exact(3)
                .map(|p| (p[0], p[1], p[2]))
                .collect();

            #[cfg(feature = "mc-tick")]
            {
                let written = crate::bridge::mc_tick::simulate_placements_into_world(
                    &mut self.0,
                    &placements,
                    descriptor,
                )
                .map_err(|detail| {
                    crate::bridge::set_last_error_detail(detail);
                    NucleationError::Simulation
                })?;
                i32::try_from(written).map_err(|_| NucleationError::InvalidArgument)
            }
            #[cfg(not(feature = "mc-tick"))]
            {
                let _ = (placements, descriptor);
                crate::bridge::set_last_error_detail(
                    "set_blocks_simulated_full_world needs the mc-tick feature (included in bridge-full)",
                );
                Err(NucleationError::Simulation)
            }
        }

        /// Batch-get block names at multiple positions. `positions` is flat
        /// `[x0,y0,z0, ...]` (length must be a multiple of 3). Writes a JSON array,
        /// one entry per position: the block name string, or `null` for
        /// empty/out-of-bounds positions.
        pub fn get_blocks_json(
            &self,
            positions: &[i32],
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            if positions.len() % 3 != 0 {
                return Err(NucleationError::InvalidArgument);
            }
            let count = positions.len() / 3;
            let region = &self.0.default_region;
            let mut results: Vec<Option<&str>> = Vec::with_capacity(count);
            for i in 0..count {
                let (x, y, z) = (positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]);
                let name = if region.is_in_region(x, y, z) {
                    region.get_block_name(x, y, z)
                } else {
                    self.0.get_block(x, y, z).map(|bs| bs.name.as_str())
                };
                results.push(name);
            }
            let json = serde_json::to_string(&results).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Stamp a merged source box into the default region. Excluded blocks
        /// are skipped, preserving destination content. Empty string or `[]`
        /// means no exclusions.
        #[allow(clippy::too_many_arguments)]
        pub fn stamp_box(
            &mut self,
            source: &Schematic,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
            target_x: i32,
            target_y: i32,
            target_z: i32,
            excluded_blocks_json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let excluded = parse_excluded_blocks(utf8(excluded_blocks_json)?)?;
            let bounds = crate::BoundingBox::new((min_x, min_y, min_z), (max_x, max_y, max_z));
            self.0
                .stamp_box(
                    &source.0,
                    &bounds,
                    (target_x, target_y, target_z),
                    &excluded,
                )
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Stamp one explicitly named source region into the default region.
        /// The region's minimum corner is mapped to the target position.
        pub fn stamp_region(
            &mut self,
            source: &Schematic,
            source_region_name: &DiplomatStr,
            target_x: i32,
            target_y: i32,
            target_z: i32,
            excluded_blocks_json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let region_name = utf8(source_region_name)?;
            if !source.0.has_region(region_name) {
                return Err(NucleationError::NotFound);
            }
            let excluded = parse_excluded_blocks(utf8(excluded_blocks_json)?)?;
            self.0
                .stamp_region(
                    &source.0,
                    region_name,
                    (target_x, target_y, target_z),
                    &excluded,
                )
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Compatibility alias for `stamp_box`.
        #[allow(clippy::too_many_arguments)]
        pub fn copy_region(
            &mut self,
            source: &Schematic,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
            target_x: i32,
            target_y: i32,
            target_z: i32,
            excluded_blocks_json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            self.stamp_box(
                source,
                min_x,
                min_y,
                min_z,
                max_x,
                max_y,
                max_z,
                target_x,
                target_y,
                target_z,
                excluded_blocks_json,
            )
        }

        // --- Block & Entity Accessors ---

        /// The full block state at a position. `NotFound` if the position is
        /// outside every region.
        pub fn get_block(
            &self,
            x: i32,
            y: i32,
            z: i32,
        ) -> Result<Box<BlockState>, NucleationError> {
            self.0
                .get_block(x, y, z)
                .cloned()
                .map(BlockState)
                .map(Box::new)
                .ok_or(NucleationError::NotFound)
        }

        /// The block at a position with its properties, as a `BlockState`.
        /// Kept as an explicit alias for callers migrating from the older API.
        pub fn get_block_with_properties(
            &self,
            x: i32,
            y: i32,
            z: i32,
        ) -> Result<Box<BlockState>, NucleationError> {
            self.get_block(x, y, z)
        }

        /// The full block state at a position in one specific region. This
        /// avoids composite lookup ambiguity when regions overlap.
        pub fn get_block_in_region(
            &self,
            region_name: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
        ) -> Result<Box<BlockState>, NucleationError> {
            let name = utf8(region_name)?;
            self.0
                .get_block_from_region(name, x, y, z)
                .cloned()
                .map(BlockState)
                .map(Box::new)
                .ok_or(NucleationError::NotFound)
        }

        /// The block string at a position in one specific region.
        pub fn get_block_string_in_region(
            &self,
            region_name: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let block = self
                .0
                .get_block_string_in_region(utf8(region_name)?, x, y, z)
                .ok_or(NucleationError::NotFound)?;
            let _ = write!(out, "{}", block);
            Ok(())
        }

        /// The full block string (name, properties, NBT) at a position.
        pub fn get_block_string(
            &self,
            x: i32,
            y: i32,
            z: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            match self.0.get_block(x, y, z) {
                Some(bs) => {
                    let _ = write!(out, "{}", bs);
                    Ok(())
                }
                None => Err(NucleationError::NotFound),
            }
        }

        /// The block entity at a position as JSON
        /// `{"id": ..., "position": [x,y,z], "nbt": {...}}` (the old `CBlockEntity`).
        pub fn get_block_entity_json(
            &self,
            x: i32,
            y: i32,
            z: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let pos = crate::block_position::BlockPosition { x, y, z };
            match self.0.get_block_entity_owned(pos) {
                Some(be) => {
                    let json = serde_json::json!({
                        "id": be.id,
                        "position": [x, y, z],
                        "nbt": serde_json::to_value(&be.nbt).unwrap_or(serde_json::Value::Null),
                    });
                    let _ = write!(out, "{}", json);
                    Ok(())
                }
                None => Err(NucleationError::NotFound),
            }
        }

        /// The block entity at a position in one specific region as JSON.
        pub fn get_block_entity_json_in_region(
            &self,
            region_name: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let entity = self
                .0
                .get_block_entity_in_region(utf8(region_name)?, x, y, z)
                .ok_or(NucleationError::NotFound)?;
            let json = serde_json::json!({
                "id": entity.id,
                "position": [x, y, z],
                "nbt": serde_json::to_value(&entity.nbt).unwrap_or(serde_json::Value::Null),
            });
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Every block entity as a JSON array of
        /// `{"id": ..., "position": [x,y,z], "nbt": {...}}`.
        pub fn get_all_block_entities_json(&self, out: &mut DiplomatWrite) {
            let items: Vec<serde_json::Value> = self
                .0
                .get_block_entities_as_list()
                .into_iter()
                .map(|be| {
                    serde_json::json!({
                        "id": be.id,
                        "position": [be.position.0, be.position.1, be.position.2],
                        "nbt": serde_json::to_value(&be.nbt).unwrap_or(serde_json::Value::Null),
                    })
                })
                .collect();
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        /// The number of mobile entities (not block entities).
        pub fn entity_count(&self) -> u32 {
            self.0.default_region.entities.len() as u32
        }

        /// Every mobile entity as a JSON array of
        /// `{"id": ..., "position": [x,y,z], "nbt": {...}}` (the old `CEntityArray`).
        pub fn get_entities_json(&self, out: &mut DiplomatWrite) {
            let items: Vec<serde_json::Value> = self
                .0
                .default_region
                .entities
                .iter()
                .map(|entity| {
                    serde_json::json!({
                        "id": entity.id,
                        "position": [entity.position.0, entity.position.1, entity.position.2],
                        "nbt": serde_json::to_value(&entity.nbt).unwrap_or(serde_json::Value::Null),
                    })
                })
                .collect();
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        /// Add a mobile entity. `nbt_json` is a JSON object (may be empty).
        pub fn add_entity(
            &mut self,
            id: &DiplomatStr,
            x: f64,
            y: f64,
            z: f64,
            nbt_json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let id_str = utf8(id)?.to_string();
            let json = utf8(nbt_json)?;
            let mut entity = crate::entity::Entity::new(id_str, (x, y, z));
            if !json.is_empty() {
                if let Ok(nbt_map) = serde_json::from_str(json) {
                    entity.nbt = nbt_map;
                }
            }
            self.0.add_entity(entity);
            Ok(())
        }

        /// Add an armor stand without hand-authoring entity NBT.
        ///
        /// `armor_material` accepts `diamond`, `netherite`, `iron`, etc.; an
        /// empty string creates an unarmored stand. `yaw` uses Minecraft degrees.
        pub fn add_armor_stand(
            &mut self,
            x: f64,
            y: f64,
            z: f64,
            yaw: f32,
            armor_material: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let material = utf8(armor_material)?;
            let equipment = if material.is_empty() {
                crate::ArmorStandEquipment::default()
            } else {
                crate::ArmorStandEquipment::full_set(material)
            };
            self.0
                .add_entity(crate::Entity::armor_stand((x, y, z), yaw, equipment));
            Ok(())
        }

        /// Remove a mobile entity by index.
        pub fn remove_entity(&mut self, index: u32) -> Result<(), NucleationError> {
            self.0
                .remove_entity(index as usize)
                .map(|_| ())
                .ok_or(NucleationError::NotFound)
        }

        // --- Data-version conversion (datafixers) ---

        /// The canonical in-memory data version (the forward-conversion target).
        pub fn canonical_data_version() -> i32 {
            crate::dataconverter::CANONICAL_DATA_VERSION
        }

        /// Convert block/item/entity data between Minecraft data versions. Forward
        /// (`target >= source`) is lossless; reverse is lossy. Writes a JSON loss
        /// report (`[]` when lossless).
        pub fn convert_to_data_version(
            &mut self,
            target_data_version: i32,
            source_data_version: i32,
            out: &mut DiplomatWrite,
        ) {
            let json = if target_data_version == source_data_version {
                "[]".to_string()
            } else if target_data_version > source_data_version {
                crate::dataconverter::convert_schematic(
                    &mut self.0,
                    source_data_version,
                    target_data_version,
                );
                "[]".to_string()
            } else {
                crate::dataconverter::convert_schematic_reverse(
                    &mut self.0,
                    source_data_version,
                    target_data_version,
                )
                .to_json()
            };
            let _ = write!(out, "{}", json);
        }

        /// Convert to `target_data_version` using the schematic's captured source
        /// version (else `mc_version`, else canonical) as origin, updating metadata
        /// to the target. Writes a JSON loss report (`[]` when lossless).
        pub fn convert_to_version(&mut self, target_data_version: i32, out: &mut DiplomatWrite) {
            let json = self
                .0
                .convert_to_data_version(target_data_version)
                .to_json();
            let _ = write!(out, "{}", json);
        }

        /// The Minecraft data version of the file this schematic was loaded from, or
        /// `-1` if none was captured (versionless / freshly built).
        pub fn source_data_version(&self) -> i32 {
            self.0.metadata.source_data_version.unwrap_or(-1)
        }

        /// Override the source data version for formats that carry no Java data
        /// version, so the converter knows what to convert *from*.
        pub fn set_source_data_version(&mut self, version: i32) {
            self.0.metadata.source_data_version = Some(version);
        }

        /// Serialize a `.litematic` targeting a specific Minecraft data version. A
        /// COPY is converted and the matching Version header written; the schematic
        /// is left unchanged. Writes JSON
        /// `{"data_b64": <base64 .litematic>, "loss": <loss report>}`.
        pub fn to_litematic_for_version_json(
            &self,
            target_data_version: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let (data, report) =
                litematic::to_litematic_for_data_version(&self.0, target_data_version)
                    .map_err(|_| NucleationError::Serialize)?;
            let loss: serde_json::Value = serde_json::from_str(&report.to_json())
                .unwrap_or(serde_json::Value::Array(Vec::new()));
            let json = serde_json::json!({ "data_b64": b64(&data), "loss": loss });
            let _ = write!(out, "{}", json);
            Ok(())
        }

        // --- Faithful (SNBT) block-entity / entity access ---

        /// The block entity's NBT as a typed SNBT string. Round-trips losslessly.
        pub fn get_block_entity_snbt(
            &self,
            x: i32,
            y: i32,
            z: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let pos = crate::block_position::BlockPosition { x, y, z };
            match self.0.get_block_entity(pos) {
                Some(be) => {
                    let snbt = quartz_nbt::NbtTag::Compound(be.nbt.to_quartz_nbt()).to_snbt();
                    let _ = write!(out, "{}", snbt);
                    Ok(())
                }
                None => Err(NucleationError::NotFound),
            }
        }

        /// Set (or replace) a block entity at a position from a typed SNBT string.
        pub fn set_block_entity(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            id: &DiplomatStr,
            snbt: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let id_str = utf8(id)?.to_string();
            let snbt_str = utf8(snbt)?;
            let compound = quartz_nbt::snbt::parse(snbt_str).map_err(|_| NucleationError::Parse)?;
            let nbt = crate::nbt::NbtMap::from_quartz_nbt(&compound);
            let mut be = crate::block_entity::BlockEntity::new(id_str, (x, y, z));
            be.set_nbt(nbt);
            self.0
                .set_block_entity(crate::block_position::BlockPosition { x, y, z }, be);
            Ok(())
        }

        /// Remove the block entity at a position. `NotFound` if none was there.
        pub fn remove_block_entity(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
        ) -> Result<(), NucleationError> {
            self.0
                .remove_block_entity((x, y, z))
                .map(|_| ())
                .ok_or(NucleationError::NotFound)
        }

        /// Every block entity as a JSON array of `{id, position: [x,y,z], snbt}`.
        /// The `snbt` is the inner data only (no `Id`/`Pos`).
        pub fn get_all_block_entities_snbt_json(&self, out: &mut DiplomatWrite) {
            let items: Vec<serde_json::Value> = self
                .0
                .get_block_entities_as_list()
                .into_iter()
                .map(|be| {
                    let snbt = quartz_nbt::NbtTag::Compound(be.nbt.to_quartz_nbt()).to_snbt();
                    serde_json::json!({
                        "id": be.id,
                        "position": [be.position.0, be.position.1, be.position.2],
                        "snbt": snbt,
                    })
                })
                .collect();
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        /// Every mobile entity as a JSON array of typed SNBT strings (full compound
        /// incl. `id`/`Pos`).
        pub fn get_entities_snbt_json(&self, out: &mut DiplomatWrite) {
            let snbts: Vec<String> = self
                .0
                .get_entities_as_list()
                .iter()
                .map(|entity| entity.to_nbt().to_snbt())
                .collect();
            let json = serde_json::to_string(&snbts).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        /// Add a mobile entity from a full SNBT entity compound (must contain `id`
        /// and `Pos`).
        pub fn add_entity_from_snbt(&mut self, snbt: &DiplomatStr) -> Result<(), NucleationError> {
            let snbt_str = utf8(snbt)?;
            let compound = quartz_nbt::snbt::parse(snbt_str).map_err(|_| NucleationError::Parse)?;
            let entity =
                crate::entity::Entity::from_nbt(&compound).map_err(|_| NucleationError::Parse)?;
            self.0.add_entity(entity);
            Ok(())
        }

        /// Every IN-BOUNDS cell as a JSON array of
        /// `{"x", "y", "z", "name", "properties"}` (the old `CBlockArray`).
        /// Air cells are materialized too — on a large sparse build this
        /// dump is `volume()`-sized and can exhaust wasm memory; renderers
        /// and analyzers want `get_non_air_blocks_json`.
        ///
        /// Prefer `get_non_air_blocks_json` for a block list,
        /// `count_blocks_json` for a material tally and
        /// `non_air_blocks_packed_b64` for bulk transfer. This method is
        /// kept for compatibility and is the wrong tool at any real size.
        pub fn get_all_blocks_json(&self, out: &mut DiplomatWrite) {
            let items: Vec<serde_json::Value> = self
                .0
                .iter_blocks()
                .map(|(pos, block)| block_json(&pos, block))
                .collect();
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        /// Every non-air block of ONE named region (a flattened design names
        /// one per layer: `inst:{name}`, `bus:{name}`), same JSON shape as
        /// `get_all_blocks_json`. Unknown region names error.
        pub fn get_region_non_air_blocks_json(
            &self,
            region_name: &str,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let region = self.0.get_region(region_name).ok_or_else(|| {
                crate::bridge::set_last_error_detail(format!("no region named `{region_name}`"));
                NucleationError::NotFound
            })?;
            let items: Vec<serde_json::Value> = region
                .blocks
                .iter()
                .enumerate()
                .filter_map(|(index, block_index)| {
                    let block = &region.palette[*block_index];
                    if block.name == "minecraft:air" {
                        return None;
                    }
                    let (x, y, z) = region.index_to_coords(index);
                    Some(block_json(
                        &crate::block_position::BlockPosition { x, y, z },
                        block,
                    ))
                })
                .collect();
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Every non-air block, same JSON shape as `get_all_blocks_json`.
        /// `block_count()`-sized regardless of the bounding volume.
        pub fn get_non_air_blocks_json(&self, out: &mut DiplomatWrite) {
            let items: Vec<serde_json::Value> = self
                .0
                .iter_blocks()
                .filter(|(_, block)| !crate::universal_schematic::is_air(block.name.as_str()))
                .map(|(pos, block)| block_json(&pos, block))
                .collect();
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        /// Non-air blocks tallied by id: `{"minecraft:stone": 123, ...}`.
        /// One pass, no per block allocation, so a caller that only wants a
        /// material list never has to pull `get_non_air_blocks_json`. "Air"
        /// covers `minecraft:air`, `cave_air` and `void_air` alike.
        pub fn count_blocks_json(&self, out: &mut DiplomatWrite) {
            let mut counts: HashMap<&str, u64> = HashMap::new();
            for (_, block) in self.0.iter_blocks() {
                if crate::universal_schematic::is_air(block.name.as_str()) {
                    continue;
                }
                *counts.entry(block.name.as_str()).or_insert(0) += 1;
            }
            // BTreeMap for a stable key order, so two identical schematics
            // serialize to identical JSON.
            let ordered: std::collections::BTreeMap<&str, u64> = counts.into_iter().collect();
            let json = serde_json::to_string(&ordered).unwrap_or_else(|_| "{}".to_string());
            let _ = write!(out, "{}", json);
        }

        /// Apply a `{"from id": "to id"}` map in place and return how many
        /// blocks changed. Keys match on block id only, ignoring block
        /// states; values may carry states (`minecraft:oak_stairs[facing=north]`),
        /// but not NBT: `parse_block_string` only returns a `BlockState`, so
        /// any `{...}` payload on a `to` value is silently dropped rather
        /// than copied onto the replaced block.
        /// A block whose id is not a key is left alone, and so is one that
        /// already equals its target: the count is the number of blocks
        /// actually changed, so a map that rewrites stone to stone returns 0.
        /// Errors with `Parse` on malformed JSON or an unparseable target id.
        pub fn replace_blocks_json(
            &mut self,
            map_json: &DiplomatStr,
        ) -> Result<u64, NucleationError> {
            let raw: HashMap<String, String> =
                serde_json::from_str(utf8(map_json)?).map_err(|_| NucleationError::Parse)?;
            // The distinct targets, parsed once, plus a from-id to target
            // index map. There are as many targets as the caller wrote keys,
            // never as many as the schematic has blocks.
            let mut states: Vec<crate::BlockState> = Vec::with_capacity(raw.len());
            let mut targets: HashMap<String, u16> = HashMap::with_capacity(raw.len());
            if raw.len() > u16::MAX as usize {
                // The edit list indexes targets with a u16. No real material
                // map is anywhere near this large.
                return Err(NucleationError::InvalidArgument);
            }
            for (from, to) in raw {
                let (state, _) = crate::UniversalSchematic::parse_block_string(&to)
                    .map_err(|_| NucleationError::Parse)?;
                let index = match states.iter().position(|s| *s == state) {
                    Some(i) => i,
                    None => {
                        states.push(state);
                        states.len() - 1
                    }
                };
                targets.insert(from, index as u16);
            }
            // Collect first: iter_blocks borrows the schematic immutably. One
            // position and one index per changed block, no BlockState clone.
            let edits: Vec<(crate::block_position::BlockPosition, u16)> = self
                .0
                .iter_blocks()
                .filter_map(|(pos, block)| {
                    let &index = targets.get(block.name.as_str())?;
                    (states[index as usize] != *block).then_some((pos, index))
                })
                .collect();
            let changed = edits.len() as u64;
            for (pos, index) in edits {
                self.0
                    .set_block(pos.x, pos.y, pos.z, &states[index as usize]);
            }
            Ok(changed)
        }

        /// Every non-air block as a compact binary blob, base64 encoded
        /// (`DiplomatWrite` is UTF-8 only, see `to_litematic_b64`). Little
        /// endian throughout:
        ///
        /// ```text
        /// u32 count
        /// count * { i32 x, i32 y, i32 z, u16 palette_index }
        /// u32 palette_json_len
        /// u8[palette_json_len]   ["minecraft:stone", ...]
        /// ```
        ///
        /// Palette indices are assigned in first-seen order, so the same
        /// schematic always packs identically. About seven times smaller
        /// than `get_non_air_blocks_json` and free of per block JSON
        /// parsing on the far side.
        ///
        /// Palette indices are `u16`, so at most 65,535 distinct non-air
        /// block states can be addressed. A schematic with more than that
        /// writes **an empty string**, not a truncated palette: callers must
        /// treat an empty result as "too many distinct states, fall back to
        /// `get_non_air_blocks_json`". No real build has that many.
        pub fn non_air_blocks_packed_b64(&self, out: &mut DiplomatWrite) {
            let mut palette: Vec<&str> = Vec::new();
            let mut index_of: HashMap<&str, u16> = HashMap::new();
            let mut body: Vec<u8> = Vec::new();
            let mut count: u32 = 0;
            for (pos, block) in self.0.iter_blocks() {
                if crate::universal_schematic::is_air(block.name.as_str()) {
                    continue;
                }
                let name = block.name.as_str();
                let index = match index_of.get(name) {
                    Some(&i) => i,
                    None => {
                        if palette.len() >= u16::MAX as usize {
                            // Past what a u16 index can address: write nothing
                            // rather than truncate to a wrong palette.
                            return;
                        }
                        let i = palette.len() as u16;
                        palette.push(name);
                        index_of.insert(name, i);
                        i
                    }
                };
                body.extend_from_slice(&pos.x.to_le_bytes());
                body.extend_from_slice(&pos.y.to_le_bytes());
                body.extend_from_slice(&pos.z.to_le_bytes());
                body.extend_from_slice(&index.to_le_bytes());
                count += 1;
            }
            let palette_json = serde_json::to_vec(&palette).unwrap_or_else(|_| b"[]".to_vec());
            let mut packed = Vec::with_capacity(4 + body.len() + 4 + palette_json.len());
            packed.extend_from_slice(&count.to_le_bytes());
            packed.extend_from_slice(&body);
            packed.extend_from_slice(&(palette_json.len() as u32).to_le_bytes());
            packed.extend_from_slice(&palette_json);
            let _ = write!(out, "{}", b64(&packed));
        }

        /// All blocks within a sub-region (chunk) of the schematic, as the same
        /// JSON array shape as `get_all_blocks_json`.
        #[allow(clippy::too_many_arguments)]
        pub fn get_chunk_blocks_json(
            &self,
            offset_x: i32,
            offset_y: i32,
            offset_z: i32,
            width: i32,
            height: i32,
            length: i32,
            out: &mut DiplomatWrite,
        ) {
            let items: Vec<serde_json::Value> = self
                .0
                .iter_blocks()
                .filter(|(pos, _)| {
                    pos.x >= offset_x
                        && pos.x < offset_x + width
                        && pos.y >= offset_y
                        && pos.y < offset_y + height
                        && pos.z >= offset_z
                        && pos.z < offset_z + length
                })
                .map(|(pos, block)| block_json(&pos, block))
                .collect();
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        /// Non-air blocks inside a bounded box, retaining block-state properties.
        /// Visits only intersecting region cells, not the whole schematic. Bounds
        /// and arithmetic are checked; queries may scan at most 1,048,576 cells
        /// (including region overlaps) and emit at most 32 MiB of JSON. Renderers
        /// should request 16³ sections. Coordinates may be negative.
        #[allow(clippy::too_many_arguments)]
        pub fn get_chunk_non_air_blocks_json(
            &self,
            offset_x: i32,
            offset_y: i32,
            offset_z: i32,
            width: i32,
            height: i32,
            length: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            crate::bridge::clear_last_error_detail();
            let fail = |message: &str| {
                crate::bridge::set_last_error_detail(message.to_string());
                NucleationError::InvalidArgument
            };
            let dims = [width, height, length];
            if dims.iter().any(|&v| v <= 0)
                || dims
                    .iter()
                    .try_fold(1_u64, |v, &d| v.checked_mul(d as u64))
                    .is_none_or(|v| v > 1_048_576)
            {
                return Err(fail(
                    "Chunk queries require positive dimensions and at most 1,048,576 cells.",
                ));
            }
            let start = [offset_x as i64, offset_y as i64, offset_z as i64];
            let end = std::array::from_fn::<_, 3, _>(|a| start[a] + dims[a] as i64);
            let mut intersections = Vec::new();
            let mut visited = 0_u64;
            for region in
                std::iter::once(&self.0.default_region).chain(self.0.other_regions.values())
            {
                let bounds = region.get_bounding_box();
                let min = [
                    bounds.min.0 as i64,
                    bounds.min.1 as i64,
                    bounds.min.2 as i64,
                ];
                let max = [
                    bounds.max.0 as i64 + 1,
                    bounds.max.1 as i64 + 1,
                    bounds.max.2 as i64 + 1,
                ];
                let lo = std::array::from_fn::<_, 3, _>(|a| start[a].max(min[a]));
                let hi = std::array::from_fn::<_, 3, _>(|a| end[a].min(max[a]));
                if (0..3).any(|a| lo[a] >= hi[a]) {
                    continue;
                }
                visited += (0..3).map(|a| (hi[a] - lo[a]) as u64).product::<u64>();
                if visited > 1_048_576 {
                    return Err(fail(
                        "Overlapping regions exceed the chunk query working limit.",
                    ));
                }
                intersections.push((region, lo, hi));
            }
            #[derive(serde::Serialize)]
            struct JsonBlock<'a> {
                x: i32,
                y: i32,
                z: i32,
                name: &'a str,
                properties: &'a [(smol_str::SmolStr, smol_str::SmolStr)],
            }
            out.write_str("[")
                .map_err(|_| fail("Cannot write chunk data."))?;
            let mut first = true;
            let mut written = 2_usize;
            for (region, lo, hi) in intersections {
                for y in lo[1]..hi[1] {
                    for z in lo[2]..hi[2] {
                        for x in lo[0]..hi[0] {
                            let Some(block) = region.get_block(x as i32, y as i32, z as i32) else {
                                continue;
                            };
                            if crate::universal_schematic::is_air(block.name.as_str()) {
                                continue;
                            }
                            let json = serde_json::to_string(&JsonBlock {
                                x: x as i32,
                                y: y as i32,
                                z: z as i32,
                                name: block.name.as_str(),
                                properties: &block.properties,
                            })
                            .map_err(|_| fail("Cannot encode chunk data."))?;
                            written += json.len() + usize::from(!first);
                            if written > 32 * 1024 * 1024 {
                                return Err(fail(
                                    "Chunk JSON exceeds 32 MiB. Request a smaller box.",
                                ));
                            }
                            if !first {
                                out.write_str(",")
                                    .map_err(|_| fail("Cannot write chunk data."))?;
                            }
                            first = false;
                            out.write_str(&json)
                                .map_err(|_| fail("Cannot write chunk data."))?;
                        }
                    }
                }
            }
            out.write_str("]")
                .map_err(|_| fail("Cannot write chunk data."))
        }

        /// Storage metadata and full block states for palette-index streaming. Region order
        /// is default first, then sorted names (highest precedence first); indices are LOCAL to each region. Bounds
        /// describe allocated storage, never tight bounds. x is fastest, then z, then y.
        /// No block buffer is cloned or serialized here.
        pub fn render_regions_json(&self, out: &mut DiplomatWrite) {
            let names = self.0.get_region_names();
            let regions: Vec<serde_json::Value> = names.iter().map(|name| {
                let region = self.0.get_region(name).unwrap();
                let bounds = region.get_bounding_box();
                let content = region.get_tight_bounds().map(|b| serde_json::json!({
                    "min": [b.min.0, b.min.1, b.min.2],
                    "max": [b.max.0, b.max.1, b.max.2]
                }));
                serde_json::json!({
                    "name": name,
                    "min": [bounds.min.0, bounds.min.1, bounds.min.2],
                    "size": [bounds.max.0 as i64 - bounds.min.0 as i64 + 1,
                             bounds.max.1 as i64 - bounds.min.1 as i64 + 1,
                             bounds.max.2 as i64 - bounds.min.2 as i64 + 1],
                    "length": region.blocks.len(),
                    "contentBounds": content,
                    "palette": region.palette.iter().map(|b| serde_json::json!({
                        "name": b.name, "properties": b.properties
                    })).collect::<Vec<_>>()
                })
            }).collect();
            let _ = write!(out, "{}", serde_json::to_string(&regions).unwrap());
        }

        /// A bounded window of a region's dense palette indices (including air).
        /// At most 65,536 cells per call; no full-world scan, coordinate tuples, or
        /// intermediate Rust allocation. JS bindings copy the borrowed slice before
        /// returning, so callers may mutate the schematic or grow WASM memory safely.
        pub fn region_block_indices<'a>(
            &'a self, region_name: &str, start: u32, count: u32,
        ) -> Result<&'a [usize], NucleationError> {
            if count > 65_536 { return Err(NucleationError::InvalidArgument); }
            let region = self.0.get_region(region_name).ok_or(NucleationError::NotFound)?;
            let start = start as usize;
            let end = start.checked_add(count as usize).ok_or(NucleationError::InvalidArgument)?;
            region.blocks.get(start..end).ok_or(NucleationError::InvalidArgument)
        }

        // --- Chunking ---

        /// Split the schematic into chunks (default bottom-up strategy). Writes a
        /// JSON array of `{"chunk_x", "chunk_y", "chunk_z", "blocks": [...]}` where
        /// blocks have the `get_all_blocks_json` shape (the old `CChunkArray`).
        pub fn get_chunks_json(
            &self,
            chunk_width: i32,
            chunk_height: i32,
            chunk_length: i32,
            out: &mut DiplomatWrite,
        ) {
            self.get_chunks_with_strategy_json(
                chunk_width,
                chunk_height,
                chunk_length,
                b"",
                0.0,
                0.0,
                0.0,
                out,
            )
        }

        /// Split the schematic into chunks with a loading strategy: one of
        /// `distance_to_camera`, `top_down`, `bottom_up`, `center_outward`,
        /// `random` (anything else falls back to `bottom_up`). Camera coordinates
        /// are only used by `distance_to_camera`. Same JSON shape as
        /// `get_chunks_json`.
        #[allow(clippy::too_many_arguments)]
        pub fn get_chunks_with_strategy_json(
            &self,
            chunk_width: i32,
            chunk_height: i32,
            chunk_length: i32,
            strategy: &DiplomatStr,
            camera_x: f32,
            camera_y: f32,
            camera_z: f32,
            out: &mut DiplomatWrite,
        ) {
            let strategy_str = std::str::from_utf8(strategy).unwrap_or("");
            let strategy_enum = match strategy_str {
                "distance_to_camera" => {
                    ChunkLoadingStrategy::DistanceToCamera(camera_x, camera_y, camera_z)
                }
                "top_down" => ChunkLoadingStrategy::TopDown,
                "bottom_up" => ChunkLoadingStrategy::BottomUp,
                "center_outward" => ChunkLoadingStrategy::CenterOutward,
                "random" => ChunkLoadingStrategy::Random,
                _ => ChunkLoadingStrategy::BottomUp,
            };
            let chunks: Vec<serde_json::Value> = self
                .0
                .iter_chunks(chunk_width, chunk_height, chunk_length, Some(strategy_enum))
                .map(|chunk| {
                    let blocks: Vec<serde_json::Value> = chunk
                        .positions
                        .into_iter()
                        .filter_map(|pos| self.0.get_block(pos.x, pos.y, pos.z).map(|b| (pos, b)))
                        .map(|(pos, block)| block_json(&pos, block))
                        .collect();
                    serde_json::json!({
                        "chunk_x": chunk.chunk_x,
                        "chunk_y": chunk.chunk_y,
                        "chunk_z": chunk.chunk_z,
                        "blocks": blocks,
                    })
                })
                .collect();
            let json = serde_json::to_string(&chunks).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        // --- Metadata & Info ---

        /// The total number of non-air blocks in the schematic.
        pub fn block_count(&self) -> i32 {
            self.0.total_blocks()
        }

        /// The total volume of the schematic's bounding box.
        pub fn volume(&self) -> i32 {
            self.0.total_volume()
        }

        /// The names of all regions, as a JSON array of strings.
        pub fn region_names_json(&self, out: &mut DiplomatWrite) {
            let names = self.0.get_region_names();
            let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        // --- Debugging & Utility ---

        /// Basic debug info about the schematic (name + region count).
        pub fn debug_info(&self, out: &mut DiplomatWrite) {
            let _ = write!(
                out,
                "Schematic name: {}, Regions: {}",
                self.0.metadata.name.as_deref().unwrap_or("Unnamed"),
                self.0.other_regions.len() + 1 // +1 for the main region
            );
        }

        /// A formatted schematic layout string (old `schematic_print`).
        pub fn print_string(&self, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", crate::format_schematic(&self.0));
        }

        /// A formatted schematic layout string (old `schematic_print_schematic`;
        /// same output as `print_string`).
        pub fn print_schematic_string(&self, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", crate::format_schematic(&self.0));
        }

        /// A detailed debug string, including a visual layout (old `debug_schematic`).
        pub fn debug_string(&self, out: &mut DiplomatWrite) {
            let _ = write!(
                out,
                "Schematic name: {}, Regions: {}\n{}",
                self.0.metadata.name.as_deref().unwrap_or("Unnamed"),
                self.0.other_regions.len() + 1,
                crate::format_schematic(&self.0)
            );
        }

        /// A detailed debug string with a JSON layout (old `debug_json_schematic`).
        pub fn debug_json_string(&self, out: &mut DiplomatWrite) {
            let _ = write!(
                out,
                "Schematic name: {}, Regions: {}\n{}",
                self.0.metadata.name.as_deref().unwrap_or("Unnamed"),
                self.0.other_regions.len() + 1,
                crate::format_json_schematic(&self.0)
            );
        }

        // --- Metadata Accessors ---

        /// The schematic name, or the empty string if not set.
        ///
        /// Total, like every other metadata accessor: absence is a blank
        /// field, not an error — a file that simply doesn't carry the field
        /// (Sponge without attribution, a fresh schematic) reads as `""`,
        /// the same value a litematic round-trip of an unset field yields.
        pub fn name(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            if let Some(name) = &self.0.metadata.name {
                let _ = write!(out, "{}", name);
            }
            Ok(())
        }

        /// Set the schematic name.
        pub fn set_name(&mut self, name: &DiplomatStr) -> Result<(), NucleationError> {
            self.0.metadata.name = Some(utf8(name)?.to_string());
            Ok(())
        }

        /// The schematic author, or the empty string if not set.
        ///
        /// Total, like every other metadata accessor: absence is a blank
        /// field, not an error — a file that simply doesn't carry the field
        /// (Sponge without attribution, a fresh schematic) reads as `""`,
        /// the same value a litematic round-trip of an unset field yields.
        pub fn author(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            if let Some(author) = &self.0.metadata.author {
                let _ = write!(out, "{}", author);
            }
            Ok(())
        }

        /// Set the schematic author.
        pub fn set_author(&mut self, author: &DiplomatStr) -> Result<(), NucleationError> {
            self.0.metadata.author = Some(utf8(author)?.to_string());
            Ok(())
        }

        /// The schematic description, or the empty string if not set.
        ///
        /// Total, like every other metadata accessor: absence is a blank
        /// field, not an error — a file that simply doesn't carry the field
        /// (Sponge without attribution, a fresh schematic) reads as `""`,
        /// the same value a litematic round-trip of an unset field yields.
        pub fn description(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            if let Some(desc) = &self.0.metadata.description {
                let _ = write!(out, "{}", desc);
            }
            Ok(())
        }

        /// Set the schematic description.
        pub fn set_description(
            &mut self,
            description: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            self.0.metadata.description = Some(utf8(description)?.to_string());
            Ok(())
        }

        /// The creation timestamp (milliseconds since epoch), or `-1` if not set.
        pub fn created(&self) -> i64 {
            self.0.metadata.created.map(|v| v as i64).unwrap_or(-1)
        }

        /// Set the creation timestamp (milliseconds since epoch).
        pub fn set_created(&mut self, created: u64) {
            self.0.metadata.created = Some(created);
        }

        /// The modification timestamp (milliseconds since epoch), or `-1` if not set.
        pub fn modified(&self) -> i64 {
            self.0.metadata.modified.map(|v| v as i64).unwrap_or(-1)
        }

        /// Set the modification timestamp (milliseconds since epoch).
        pub fn set_modified(&mut self, modified: u64) {
            self.0.metadata.modified = Some(modified);
        }

        /// The Litematic format version, or `-1` if not set.
        pub fn lm_version(&self) -> i32 {
            self.0.metadata.lm_version.unwrap_or(-1)
        }

        /// Set the Litematic format version.
        pub fn set_lm_version(&mut self, version: i32) {
            self.0.metadata.lm_version = Some(version);
        }

        /// The Minecraft data version, or `-1` if not set.
        pub fn mc_version(&self) -> i32 {
            self.0.metadata.mc_version.unwrap_or(-1)
        }

        /// Set the Minecraft data version.
        pub fn set_mc_version(&mut self, version: i32) {
            self.0.metadata.mc_version = Some(version);
        }

        /// The WorldEdit version, or `-1` if not set.
        pub fn we_version(&self) -> i32 {
            self.0.metadata.we_version.unwrap_or(-1)
        }

        /// Set the WorldEdit version.
        pub fn set_we_version(&mut self, version: i32) {
            self.0.metadata.we_version = Some(version);
        }

        /// Standard embedded source provenance as canonical JSON. Returns an
        /// empty string when none is present.
        pub fn provenance_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            if let Some(provenance) = &self.0.metadata.provenance {
                let json = provenance
                    .to_json()
                    .map_err(|_| NucleationError::Serialize)?;
                let _ = write!(out, "{json}");
            }
            Ok(())
        }

        /// Validate and set standard embedded source provenance from JSON.
        pub fn set_provenance_json(&mut self, json: &DiplomatStr) -> Result<(), NucleationError> {
            let json = utf8(json)?;
            let provenance = crate::SchematicProvenance::from_json(json).map_err(|error| {
                crate::bridge::set_last_error_detail(error);
                NucleationError::Parse
            })?;
            self.0.metadata.provenance = Some(provenance);
            Ok(())
        }

        /// Remove embedded source provenance.
        pub fn clear_provenance(&mut self) {
            self.0.metadata.provenance = None;
        }

        /// Content-addressed processing history as a JSON array. This audit
        /// trail is deliberately separate from immutable source provenance.
        pub fn transformation_history_json(&self, out: &mut DiplomatWrite) {
            let json = serde_json::to_string(&self.0.metadata.transformation_history)
                .unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        /// Clear processing history without changing source provenance or
        /// schematic content. Intended for callers constructing a new artifact
        /// lineage, not for hiding registry audit records.
        pub fn clear_transformation_history(&mut self) {
            self.0.metadata.transformation_history.clear();
        }

        // --- Transformations ---

        /// Mirror the default region along the X axis (in place). Block
        /// orientations, block entities, and entities are mirrored too.
        pub fn flip_x(&mut self) {
            self.0.flip_x();
        }

        /// Mirror the default region along the Y axis (in place).
        pub fn flip_y(&mut self) {
            self.0.flip_y();
        }

        /// Mirror the default region along the Z axis (in place).
        pub fn flip_z(&mut self) {
            self.0.flip_z();
        }

        /// Rotate the default region about the X axis. +90° maps south (+Z)
        /// to down (-Y). Only multiples of 90 are accepted; invalid angles
        /// return `InvalidArgument` without changing the schematic. Negative
        /// values wrap.
        pub fn rotate_x(&mut self, degrees: i32) -> Result<(), NucleationError> {
            self.0
                .rotate_x(degrees)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Rotate the default region clockwise about the Y axis when viewed
        /// from above. +90° maps east (+X) to south (+Z).
        pub fn rotate_y(&mut self, degrees: i32) -> Result<(), NucleationError> {
            self.0
                .rotate_y(degrees)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Rotate the default region about the Z axis. +90° maps up (+Y) to
        /// west (-X).
        pub fn rotate_z(&mut self, degrees: i32) -> Result<(), NucleationError> {
            self.0
                .rotate_z(degrees)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Move the default region and all attached block entities/entities.
        pub fn translate(&mut self, dx: i32, dy: i32, dz: i32) -> Result<(), NucleationError> {
            self.0
                .translate(dx, dy, dz)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Mirror a named region along the X axis.
        pub fn flip_region_x(&mut self, region_name: &DiplomatStr) -> Result<(), NucleationError> {
            let name = utf8(region_name)?;
            if !self.0.has_region(name) {
                return Err(NucleationError::NotFound);
            }
            self.0
                .flip_region_x(name)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Mirror a named region along the Y axis.
        pub fn flip_region_y(&mut self, region_name: &DiplomatStr) -> Result<(), NucleationError> {
            let name = utf8(region_name)?;
            if !self.0.has_region(name) {
                return Err(NucleationError::NotFound);
            }
            self.0
                .flip_region_y(name)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Mirror a named region along the Z axis.
        pub fn flip_region_z(&mut self, region_name: &DiplomatStr) -> Result<(), NucleationError> {
            let name = utf8(region_name)?;
            if !self.0.has_region(name) {
                return Err(NucleationError::NotFound);
            }
            self.0
                .flip_region_z(name)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Rotate a named region about the X axis by a multiple of 90 degrees.
        pub fn rotate_region_x(
            &mut self,
            region_name: &DiplomatStr,
            degrees: i32,
        ) -> Result<(), NucleationError> {
            let name = utf8(region_name)?;
            if !self.0.has_region(name) {
                return Err(NucleationError::NotFound);
            }
            self.0
                .rotate_region_x(name, degrees)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Rotate a named region clockwise about the Y axis by a multiple of
        /// 90 degrees.
        pub fn rotate_region_y(
            &mut self,
            region_name: &DiplomatStr,
            degrees: i32,
        ) -> Result<(), NucleationError> {
            let name = utf8(region_name)?;
            if !self.0.has_region(name) {
                return Err(NucleationError::NotFound);
            }
            self.0
                .rotate_region_y(name, degrees)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Rotate a named region about the Z axis by a multiple of 90 degrees.
        pub fn rotate_region_z(
            &mut self,
            region_name: &DiplomatStr,
            degrees: i32,
        ) -> Result<(), NucleationError> {
            let name = utf8(region_name)?;
            if !self.0.has_region(name) {
                return Err(NucleationError::NotFound);
            }
            self.0
                .rotate_region_z(name, degrees)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Move one named region without affecting its siblings.
        pub fn translate_region(
            &mut self,
            region_name: &DiplomatStr,
            dx: i32,
            dy: i32,
            dz: i32,
        ) -> Result<(), NucleationError> {
            let name = utf8(region_name)?;
            if !self.0.has_region(name) {
                return Err(NucleationError::NotFound);
            }
            self.0
                .translate_region(name, dx, dy, dz)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Rotate every region as one rigid schematic around the shared bounds.
        pub fn rotate_schematic_x(&mut self, degrees: i32) -> Result<(), NucleationError> {
            self.0
                .rotate_schematic_x(degrees)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Rotate every region as one rigid schematic around the shared bounds.
        pub fn rotate_schematic_y(&mut self, degrees: i32) -> Result<(), NucleationError> {
            self.0
                .rotate_schematic_y(degrees)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Rotate every region as one rigid schematic around the shared bounds.
        pub fn rotate_schematic_z(&mut self, degrees: i32) -> Result<(), NucleationError> {
            self.0
                .rotate_schematic_z(degrees)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Mirror every region across the shared schematic X bounds.
        pub fn flip_schematic_x(&mut self) -> Result<(), NucleationError> {
            self.0
                .flip_schematic_x()
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Mirror every region across the shared schematic Y bounds.
        pub fn flip_schematic_y(&mut self) -> Result<(), NucleationError> {
            self.0
                .flip_schematic_y()
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Mirror every region across the shared schematic Z bounds.
        pub fn flip_schematic_z(&mut self) -> Result<(), NucleationError> {
            self.0
                .flip_schematic_z()
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Move every region by the same delta, preserving their relative layout.
        pub fn translate_schematic(
            &mut self,
            dx: i32,
            dy: i32,
            dz: i32,
        ) -> Result<(), NucleationError> {
            self.0
                .translate_schematic(dx, dy, dz)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        // --- Building ---

        /// Fill a cuboid with a block.
        #[allow(clippy::too_many_arguments)]
        pub fn fill_cuboid(
            &mut self,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
            block_name: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let name = utf8(block_name)?;
            self.0
                .fill_cuboid_str((min_x, min_y, min_z), (max_x, max_y, max_z), name);
            Ok(())
        }

        /// Fill a sphere with a block.
        pub fn fill_sphere(
            &mut self,
            cx: f32,
            cy: f32,
            cz: f32,
            radius: f32,
            block_name: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let name = utf8(block_name)?.to_string();
            let block = crate::BlockState::new(name);
            let shape = crate::building::ShapeEnum::Sphere(crate::building::Sphere::new(
                (cx as i32, cy as i32, cz as i32),
                radius as f64,
            ));
            let brush = crate::building::SolidBrush::new(block);
            let mut tool = crate::building::BuildingTool::new(&mut self.0);
            tool.fill(&shape, &brush);
            Ok(())
        }

        // --- Format management ---

        /// Serialize to a named format, base64-encoded. `version` and `settings`
        /// may be empty strings for defaults.
        pub fn save_as_b64(
            &self,
            format: &DiplomatStr,
            version: &DiplomatStr,
            settings: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let fmt = utf8(format)?;
            let ver = utf8(version)?;
            let ver = if ver.is_empty() { None } else { Some(ver) };
            let settings_str = utf8(settings)?;
            let settings_str = if settings_str.is_empty() {
                None
            } else {
                Some(settings_str)
            };
            let manager = get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let data = manager
                .write_with_settings(fmt, &self.0, ver, settings_str)
                .map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", b64(&data));
            Ok(())
        }

        /// Save to a file. If `format` is empty, the format is auto-detected from
        /// the file extension; `version` may be empty for the default.
        /// Not available in JS (no filesystem in WASM) — use `save_as_b64`.
        #[diplomat::attr(js, disable)]
        pub fn save_to_file_with_format(
            &self,
            path: &DiplomatStr,
            format: &DiplomatStr,
            version: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let path = utf8(path)?;
            let fmt = utf8(format)?;
            let ver = utf8(version)?;
            let ver = if ver.is_empty() { None } else { Some(ver) };
            let manager = get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let bytes = if fmt.is_empty() {
                manager.write_auto_with_settings(path, &self.0, ver, None)
            } else {
                manager.write_with_settings(fmt, &self.0, ver, None)
            }
            .map_err(|_| NucleationError::Serialize)?;
            std::fs::write(path, &bytes).map_err(|_| NucleationError::Io)
        }

        /// Serialize as a Sponge schematic targeting a specific format version,
        /// base64-encoded.
        pub fn to_schematic_version_b64(
            &self,
            version: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let ver = utf8(version)?;
            let manager = get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let data = manager
                .write("sponge", &self.0, Some(ver))
                .map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", b64(&data));
            Ok(())
        }

        /// The available Sponge schematic exporter versions, as a JSON array of
        /// strings.
        pub fn available_schematic_versions_json(
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let manager = get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let versions = manager.get_exporter_versions("sponge").unwrap_or_default();
            let json = serde_json::to_string(&versions).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        // --- More block setters ---

        /// Set a block with NBT data given as a JSON object of string→string
        /// (may be empty).
        pub fn set_block_with_nbt(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            block_name: &DiplomatStr,
            nbt_json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let name = utf8(block_name)?;
            let json = utf8(nbt_json)?;
            let nbt: HashMap<String, String> = if json.is_empty() {
                HashMap::new()
            } else {
                serde_json::from_str(json).unwrap_or_default()
            };
            self.0
                .set_block_with_nbt(x, y, z, name, nbt)
                .map(|_| ())
                .map_err(|_| NucleationError::Parse)
        }

        /// Set a block (by name) in a named region.
        pub fn set_block_in_region(
            &mut self,
            region_name: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
            block_name: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let region = utf8(region_name)?;
            let block = utf8(block_name)?;
            self.0
                .try_set_block_in_region_str(region, x, y, z, block)
                .and_then(|placed| {
                    placed
                        .then_some(())
                        .ok_or_else(|| "Block placement failed".to_string())
                })
                .map_err(|_| NucleationError::InvalidArgument)
        }

        // --- Palette / bounding box / info ---

        /// Whether a default or named schematic region exists.
        pub fn has_region(&self, region_name: &DiplomatStr) -> Result<bool, NucleationError> {
            Ok(self.0.has_region(utf8(region_name)?))
        }

        /// Create an empty named region. Its first block anchors its bounds.
        pub fn create_region(&mut self, region_name: &DiplomatStr) -> Result<(), NucleationError> {
            self.0
                .create_schematic_region(utf8(region_name)?)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Remove a named region. The default region cannot be removed.
        pub fn remove_region(&mut self, region_name: &DiplomatStr) -> Result<(), NucleationError> {
            let name = utf8(region_name)?;
            if !self.0.has_region(name) {
                return Err(NucleationError::NotFound);
            }
            self.0
                .remove_schematic_region(name)
                .map(|_| ())
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Rename a named region. The default region cannot be renamed.
        pub fn rename_region(
            &mut self,
            old_name: &DiplomatStr,
            new_name: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let old = utf8(old_name)?;
            let new = utf8(new_name)?;
            if !self.0.has_region(old) {
                return Err(NucleationError::NotFound);
            }
            self.0
                .rename_schematic_region(old, new)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// The schematic bounding box as a JSON array
        /// `[min_x, min_y, min_z, max_x, max_y, max_z]`.
        pub fn bounding_box_json(&self, out: &mut DiplomatWrite) {
            let bbox = self.0.get_bounding_box();
            let _ = write!(
                out,
                "[{},{},{},{},{},{}]",
                bbox.min.0, bbox.min.1, bbox.min.2, bbox.max.0, bbox.max.1, bbox.max.2
            );
        }

        /// A named region's bounding box as a JSON array
        /// `[min_x, min_y, min_z, max_x, max_y, max_z]`. `"default"`/`"Default"`
        /// address the default region.
        pub fn region_bounding_box_json(
            &self,
            region_name: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let name = utf8(region_name)?;
            let region = self
                .0
                .get_region(name)
                .or_else(|| {
                    (name == "default" || name == "Default").then_some(&self.0.default_region)
                })
                .ok_or(NucleationError::NotFound)?;
            // The tight content bounds (min/max of placed non-air blocks), not
            // the internal storage box — which over-allocates by up to 64 blocks
            // per axis and would otherwise leak allocation padding into the
            // reported bounds. Empty regions fall back to their (degenerate)
            // origin box.
            let bbox = region
                .get_tight_bounds()
                .unwrap_or_else(|| region.get_bounding_box());
            let _ = write!(
                out,
                "[{},{},{},{},{},{}]",
                bbox.min.0, bbox.min.1, bbox.min.2, bbox.max.0, bbox.max.1, bbox.max.2
            );
            Ok(())
        }

        /// The merged-region palette block names, as a JSON array of strings.
        pub fn palette_json(&self, out: &mut DiplomatWrite) {
            let merged = self.0.get_merged_region();
            let names: Vec<&str> = merged.palette.iter().map(|bs| bs.name.as_str()).collect();
            let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        /// The tight (content) dimensions.
        pub fn tight_dimensions(&self) -> Dimensions {
            let (x, y, z) = self.0.get_tight_dimensions();
            Dimensions { x, y, z }
        }

        /// The allocated dimensions (same as `dimensions`; named for parity with
        /// the old `schematic_get_allocated_dimensions`).
        pub fn allocated_dimensions(&self) -> Dimensions {
            let (x, y, z) = self.0.get_dimensions();
            Dimensions { x, y, z }
        }

        /// Every sign in the schematic, as a JSON array of
        /// `{"pos": [x,y,z], "text": [...]}`.
        pub fn extract_signs_json(&self, out: &mut DiplomatWrite) {
            let signs = crate::insign::extract_signs(&self.0);
            // SignInput doesn't derive Serialize, manually build JSON.
            let json_array: Vec<String> = signs
                .iter()
                .map(|sign| {
                    format!(
                        "{{\"pos\":[{},{},{}],\"text\":{}}}",
                        sign.pos[0],
                        sign.pos[1],
                        sign.pos[2],
                        serde_json::to_string(&sign.text).unwrap_or_default()
                    )
                })
                .collect();
            let _ = write!(out, "[{}]", json_array.join(","));
        }

        /// Compile the schematic's insign annotations to JSON.
        pub fn compile_insign_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let data = crate::insign::compile_schematic_insign(&self.0)
                .map_err(|_| NucleationError::Parse)?;
            let json = serde_json::to_string(&data).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Embed a `CellContract` (JSON) in the schematic's metadata,
        /// validating it parses first. The contract is carried through
        /// `.schem` save/open and autodetected on open — schematic +
        /// contract = one self-describing typed cell.
        pub fn set_cell_contract_json(
            &mut self,
            json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let json = core::str::from_utf8(json).map_err(|_| NucleationError::InvalidArgument)?;
            self.0.set_cell_contract_json(json).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::Parse
            })
        }

        /// The contract embedded in the schematic's metadata, as JSON.
        /// Errors with `NotFound` when none is embedded, `Parse` when an
        /// embedded string exists but is corrupt (loud, never silent).
        pub fn cell_contract_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let contract = self
                .0
                .embedded_cell_contract()
                .map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::Parse
                })?
                .ok_or(NucleationError::NotFound)?;
            let json = contract.to_json().map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{json}");
            Ok(())
        }

        /// Resolve the schematic's cell contract from its sources in
        /// strict precedence — embedded metadata over Insign signs — with
        /// loud conflict warnings. Writes `{"contract": ..., "warnings":
        /// [...]}`; errors with `NotFound` when no source defines one.
        pub fn resolve_cell_contract_json(
            &self,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let (contract, warnings) = self
                .0
                .resolve_cell_contract()
                .map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::Parse
                })?
                .ok_or(NucleationError::NotFound)?;
            let json = contract.to_json().map_err(|_| NucleationError::Serialize)?;
            let ws: Vec<String> = warnings.iter().map(|w| format!("{w:?}")).collect();
            let _ = write!(
                out,
                "{{\"contract\":{json},\"warnings\":[{}]}}",
                ws.join(",")
            );
            Ok(())
        }

        /// Parse the schematic's IO-contract insign annotations (`#cell`
        /// header, `bus.*` port annotations, `#route_zone` zones) to JSON:
        /// `{"cell": ..., "buses": [...], "route_zones": {...}}`.
        pub fn compile_io_contracts_json(
            &self,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let signs: Vec<([i32; 3], String)> = crate::insign::extract_signs(&self.0)
                .into_iter()
                .map(|s| (s.pos, s.text))
                .collect();
            let data = crate::io_contract::insign_ext::contracts_json(&signs)
                .map_err(|_| NucleationError::Parse)?;
            let json = serde_json::to_string(&data).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Every region's palette, as a JSON object mapping region name → array of
        /// block names (the default region under `"default"`).
        pub fn all_palettes_json(&self, out: &mut DiplomatWrite) {
            let mut palettes: HashMap<String, Vec<String>> = HashMap::new();
            let default_blocks: Vec<String> = self
                .0
                .default_region
                .palette
                .iter()
                .map(|bs| bs.name.to_string())
                .collect();
            palettes.insert("default".to_string(), default_blocks);
            for (name, region) in &self.0.other_regions {
                let blocks: Vec<String> = region
                    .palette
                    .iter()
                    .map(|bs| bs.name.to_string())
                    .collect();
                palettes.insert(name.clone(), blocks);
            }
            let json = serde_json::to_string(&palettes).unwrap_or_else(|_| "{}".to_string());
            let _ = write!(out, "{}", json);
        }

        /// The default region's palette block names, as a JSON array of strings.
        pub fn default_region_palette_json(&self, out: &mut DiplomatWrite) {
            let names: Vec<&str> = self
                .0
                .default_region
                .palette
                .iter()
                .map(|bs| bs.name.as_str())
                .collect();
            let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
        }

        /// A named region's palette block names, as a JSON array of strings.
        /// `"default"`/`"Default"` address the default region.
        pub fn region_palette_json(
            &self,
            region_name: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let name = utf8(region_name)?;
            let region = self
                .0
                .get_region(name)
                .or_else(|| {
                    (name == "default" || name == "Default").then_some(&self.0.default_region)
                })
                .ok_or(NucleationError::NotFound)?;
            let names: Vec<&str> = region.palette.iter().map(|bs| bs.name.as_str()).collect();
            let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// The minimum corner of the tight (content) bounds. `NotFound` when the
        /// schematic has no content.
        pub fn tight_bounds_min(&self) -> Result<BlockPos, NucleationError> {
            self.0
                .get_tight_bounds()
                .map(|bbox| BlockPos {
                    x: bbox.min.0,
                    y: bbox.min.1,
                    z: bbox.min.2,
                })
                .ok_or(NucleationError::NotFound)
        }

        /// The maximum corner of the tight (content) bounds. `NotFound` when the
        /// schematic has no content.
        pub fn tight_bounds_max(&self) -> Result<BlockPos, NucleationError> {
            self.0
                .get_tight_bounds()
                .map(|bbox| BlockPos {
                    x: bbox.max.0,
                    y: bbox.max.1,
                    z: bbox.max.2,
                })
                .ok_or(NucleationError::NotFound)
        }
    }

    /// A block state: a block name plus its properties. Port of the old
    /// `BlockStateWrapper` / `blockstate_*` fns.
    #[diplomat::opaque]
    pub struct BlockState(pub(crate) crate::BlockState);

    impl BlockState {
        /// Create a block state with the given name and no properties.
        pub fn create(name: &DiplomatStr) -> Box<BlockState> {
            Box::new(BlockState(crate::BlockState::new(
                String::from_utf8_lossy(name).into_owned(),
            )))
        }

        /// A copy of this block state with `key=value` set; the original is
        /// unchanged.
        pub fn with_property(
            &self,
            key: &DiplomatStr,
            value: &DiplomatStr,
        ) -> Result<Box<BlockState>, NucleationError> {
            let key = utf8(key)?;
            let value = utf8(value)?;
            Ok(Box::new(BlockState(
                self.0.clone().with_property(key, value),
            )))
        }

        /// The block name (e.g. `minecraft:stone`).
        pub fn name(&self, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", self.0.name);
        }

        /// The properties as a JSON object of string→string (the old
        /// `CPropertyArray`).
        pub fn properties_json(&self, out: &mut DiplomatWrite) {
            let mut map = serde_json::Map::new();
            for (k, v) in &self.0.properties {
                map.insert(k.to_string(), serde_json::Value::String(v.to_string()));
            }
            let json = serde_json::to_string(&serde_json::Value::Object(map))
                .unwrap_or_else(|_| "{}".to_string());
            let _ = write!(out, "{}", json);
        }
    }
}

#[cfg(test)]
mod file_convenience_alias_tests {
    use super::ffi::Schematic;

    #[test]
    fn open_and_save_round_trip_a_file() {
        let path = std::env::temp_dir().join(format!(
            "nucleation-python-open-save-{}.schem",
            std::process::id()
        ));
        let path_bytes = path.to_string_lossy();

        let mut schematic = Schematic::create(b"open-save-regression");
        schematic
            .set_block(0, 0, 0, b"minecraft:stone")
            .expect("place block");
        schematic.save(path_bytes.as_bytes()).expect("save alias");

        let loaded = Schematic::open(path_bytes.as_bytes()).expect("open alias");
        let dimensions = loaded.dimensions();
        assert_eq!((dimensions.x, dimensions.y, dimensions.z), (1, 1, 1));

        std::fs::remove_file(path).expect("remove test file");
    }
}
