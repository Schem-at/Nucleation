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

fn b64(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(bytes)
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
fn block_json(pos: &crate::block_position::BlockPosition, block: &crate::BlockState) -> serde_json::Value {
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
    use super::{b64, block_json, parse_world_options, utf8};
    use crate::formats::{litematic, manager::get_manager, mcstructure};
    use crate::universal_schematic::ChunkLoadingStrategy;
    use diplomat_runtime::DiplomatWrite;
    use std::collections::HashMap;
    use std::fmt::Write;

    #[diplomat::opaque_mut]
    pub struct Schematic(pub(crate) crate::UniversalSchematic);

    impl Schematic {
        /// Create a new, empty schematic with the given name.
        pub fn create(name: &DiplomatStr) -> Box<Schematic> {
            Box::new(Schematic(crate::UniversalSchematic::new(
                String::from_utf8_lossy(name).into_owned(),
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
            Ok(self.0.set_block_str(x, y, z, name))
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

        /// Save the schematic to a file, always in Litematic format (the
        /// extension is not consulted; use `save_to_file_with_format` for
        /// other formats). Not available in JS: the WASM build has no
        /// filesystem — use `to_litematic_b64` / `save_as_b64` there.
        #[diplomat::attr(js, disable)]
        pub fn save_to_file(&self, path: &DiplomatStr) -> Result<(), NucleationError> {
            let path = std::str::from_utf8(path).map_err(|_| NucleationError::InvalidArgument)?;
            let bytes =
                crate::litematic::to_litematic(&self.0).map_err(|_| NucleationError::Serialize)?;
            std::fs::write(path, bytes).map_err(|_| NucleationError::Io)?;
            Ok(())
        }

        /// Load a schematic from a Litematic file (this path is
        /// Litematic-only; use `from_data` for format auto-detection).
        /// Not available in JS: the WASM build has no filesystem — read the
        /// bytes yourself and use `from_data`.
        #[diplomat::attr(js, disable)]
        pub fn load_from_file(path: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            let path = std::str::from_utf8(path).map_err(|_| NucleationError::InvalidArgument)?;
            let bytes = std::fs::read(path).map_err(|_| NucleationError::Io)?;
            let inner =
                crate::litematic::from_litematic(&bytes).map_err(|_| NucleationError::Parse)?;
            Ok(Box::new(Schematic(inner)))
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
            let data = mcstructure::to_mcstructure(&self.0)
                .map_err(|_| NucleationError::Serialize)?;
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

            // Complex block strings: parse once, apply many.
            if block_name_str.contains('[') || block_name_str.ends_with('}') {
                let (mut block_state, nbt_data) =
                    crate::UniversalSchematic::parse_block_string(block_name_str)
                        .map_err(|_| NucleationError::Parse)?;
                if block_state.name.contains("jukebox") {
                    if let Some(ref nbt) = nbt_data {
                        let has_record = nbt.contains_key("RecordItem");
                        block_state.set_property("has_record", has_record.to_string());
                    }
                }

                let block_name_owned = block_state.name.to_string();
                let proto: Option<crate::block_entity::BlockEntity> = nbt_data.as_ref().map(|nbt| {
                    let mut be =
                        crate::block_entity::BlockEntity::new(block_name_owned.clone(), (0, 0, 0));
                    for (k, v) in nbt {
                        be = be.with_nbt_data(k.clone(), v.clone());
                    }
                    be
                });

                let region = &mut s.default_region;
                region.ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
                let palette_index = region.get_or_insert_palette_by_state(&block_state);
                for i in 0..count {
                    let (x, y, z) =
                        (positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]);
                    region.set_block_at_index_unchecked(palette_index, x, y, z);
                }

                if let Some(ref template) = proto {
                    for i in 0..count {
                        let (x, y, z) =
                            (positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]);
                        let mut be = template.clone();
                        be.position = (x, y, z);
                        s.set_block_entity(crate::block_position::BlockPosition { x, y, z }, be);
                    }
                }
                return Ok(count as i32);
            }

            let region = &mut s.default_region;
            region.ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
            let palette_index = region.get_or_insert_palette_by_name(block_name_str);
            for i in 0..count {
                let (x, y, z) = (positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]);
                region.set_block_at_index_unchecked(palette_index, x, y, z);
            }
            Ok(count as i32)
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

        /// Copy a region from `source` into this schematic. `excluded_blocks_json`
        /// is a JSON array of block strings to skip (empty string or `[]` for none).
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
            let excluded_str = utf8(excluded_blocks_json)?;
            let mut excluded = Vec::new();
            if !excluded_str.is_empty() {
                let strings: Vec<String> =
                    serde_json::from_str(excluded_str).map_err(|_| NucleationError::Parse)?;
                for block_str in &strings {
                    let (bs, _) = crate::UniversalSchematic::parse_block_string(block_str)
                        .map_err(|_| NucleationError::Parse)?;
                    excluded.push(bs);
                }
            }
            let bounds =
                crate::BoundingBox::new((min_x, min_y, min_z), (max_x, max_y, max_z));
            self.0
                .copy_region(&source.0, &bounds, (target_x, target_y, target_z), &excluded)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        // --- Block & Entity Accessors ---

        /// The block at a position with its properties, as a `BlockState`.
        pub fn get_block_with_properties(
            &self,
            x: i32,
            y: i32,
            z: i32,
        ) -> Result<Box<BlockState>, NucleationError> {
            self.0
                .get_block(x, y, z)
                .cloned()
                .map(|bs| Box::new(BlockState(bs)))
                .ok_or(NucleationError::NotFound)
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
            match self.0.get_block_entity(pos) {
                Some(be) => {
                    let json = serde_json::json!({
                        "id": be.id,
                        "position": [be.position.0, be.position.1, be.position.2],
                        "nbt": serde_json::to_value(&be.nbt).unwrap_or(serde_json::Value::Null),
                    });
                    let _ = write!(out, "{}", json);
                    Ok(())
                }
                None => Err(NucleationError::NotFound),
            }
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
            let json = self.0.convert_to_data_version(target_data_version).to_json();
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
            let compound =
                quartz_nbt::snbt::parse(snbt_str).map_err(|_| NucleationError::Parse)?;
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
            let compound =
                quartz_nbt::snbt::parse(snbt_str).map_err(|_| NucleationError::Parse)?;
            let entity = crate::entity::Entity::from_nbt(&compound)
                .map_err(|_| NucleationError::Parse)?;
            self.0.add_entity(entity);
            Ok(())
        }

        /// Every non-air block as a JSON array of
        /// `{"x", "y", "z", "name", "properties"}` (the old `CBlockArray`).
        pub fn get_all_blocks_json(&self, out: &mut DiplomatWrite) {
            let items: Vec<serde_json::Value> = self
                .0
                .iter_blocks()
                .map(|(pos, block)| block_json(&pos, block))
                .collect();
            let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
            let _ = write!(out, "{}", json);
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
                        .filter_map(|pos| {
                            self.0.get_block(pos.x, pos.y, pos.z).map(|b| (pos, b))
                        })
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
                self.0
                    .metadata
                    .name
                    .as_deref()
                    .unwrap_or("Unnamed"),
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

        /// The schematic name. `NotFound` if not set.
        pub fn name(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            match &self.0.metadata.name {
                Some(name) => {
                    let _ = write!(out, "{}", name);
                    Ok(())
                }
                None => Err(NucleationError::NotFound),
            }
        }

        /// Set the schematic name.
        pub fn set_name(&mut self, name: &DiplomatStr) -> Result<(), NucleationError> {
            self.0.metadata.name = Some(utf8(name)?.to_string());
            Ok(())
        }

        /// The schematic author. `NotFound` if not set.
        pub fn author(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            match &self.0.metadata.author {
                Some(author) => {
                    let _ = write!(out, "{}", author);
                    Ok(())
                }
                None => Err(NucleationError::NotFound),
            }
        }

        /// Set the schematic author.
        pub fn set_author(&mut self, author: &DiplomatStr) -> Result<(), NucleationError> {
            self.0.metadata.author = Some(utf8(author)?.to_string());
            Ok(())
        }

        /// The schematic description. `NotFound` if not set.
        pub fn description(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            match &self.0.metadata.description {
                Some(desc) => {
                    let _ = write!(out, "{}", desc);
                    Ok(())
                }
                None => Err(NucleationError::NotFound),
            }
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

        // --- Transformations ---

        /// Mirror the default region along the X axis (in place). Block
        /// orientations (e.g. `facing` properties), block entities, and
        /// entities are mirrored too.
        pub fn flip_x(&mut self) {
            self.0.flip_x();
        }

        /// Mirror the default region along the Y axis (in place). Block
        /// orientations, block entities, and entities are mirrored too.
        pub fn flip_y(&mut self) {
            self.0.flip_y();
        }

        /// Mirror the default region along the Z axis (in place). Block
        /// orientations, block entities, and entities are mirrored too.
        pub fn flip_z(&mut self) {
            self.0.flip_z();
        }

        /// Rotate the default region about the X axis. `degrees` must be a
        /// multiple of 90 (anything else is a no-op; negative values wrap).
        /// +90° maps +Z onto +Y (south face rotates up). The region keeps its
        /// minimum corner; block orientations and entities are updated.
        pub fn rotate_x(&mut self, degrees: i32) {
            self.0.rotate_x(degrees);
        }

        /// Rotate the default region about the Y axis (horizontal plane).
        /// `degrees` must be a multiple of 90 (anything else is a no-op;
        /// negative values wrap). +90° maps +X onto -Z (east to north, i.e.
        /// counterclockwise seen from above). The region keeps its minimum
        /// corner; block orientations and entities are updated.
        pub fn rotate_y(&mut self, degrees: i32) {
            self.0.rotate_y(degrees);
        }

        /// Rotate the default region about the Z axis. `degrees` must be a
        /// multiple of 90 (anything else is a no-op; negative values wrap).
        /// +90° maps +Y onto +X (up rotates east). The region keeps its
        /// minimum corner; block orientations and entities are updated.
        pub fn rotate_z(&mut self, degrees: i32) {
            self.0.rotate_z(degrees);
        }

        /// Mirror a named region along the X axis (like `flip_x`). `NotFound`
        /// if no region has that name.
        pub fn flip_region_x(&mut self, region_name: &DiplomatStr) -> Result<(), NucleationError> {
            self.0
                .flip_region_x(utf8(region_name)?)
                .map_err(|_| NucleationError::NotFound)
        }

        /// Mirror a named region along the Y axis (like `flip_y`). `NotFound`
        /// if no region has that name.
        pub fn flip_region_y(&mut self, region_name: &DiplomatStr) -> Result<(), NucleationError> {
            self.0
                .flip_region_y(utf8(region_name)?)
                .map_err(|_| NucleationError::NotFound)
        }

        /// Mirror a named region along the Z axis (like `flip_z`). `NotFound`
        /// if no region has that name.
        pub fn flip_region_z(&mut self, region_name: &DiplomatStr) -> Result<(), NucleationError> {
            self.0
                .flip_region_z(utf8(region_name)?)
                .map_err(|_| NucleationError::NotFound)
        }

        /// Rotate a named region about the X axis by a multiple of 90 degrees
        /// (same semantics as `rotate_x`). `NotFound` if no region has that
        /// name.
        pub fn rotate_region_x(
            &mut self,
            region_name: &DiplomatStr,
            degrees: i32,
        ) -> Result<(), NucleationError> {
            self.0
                .rotate_region_x(utf8(region_name)?, degrees)
                .map_err(|_| NucleationError::NotFound)
        }

        /// Rotate a named region about the Y axis by a multiple of 90 degrees
        /// (same semantics as `rotate_y`). `NotFound` if no region has that
        /// name.
        pub fn rotate_region_y(
            &mut self,
            region_name: &DiplomatStr,
            degrees: i32,
        ) -> Result<(), NucleationError> {
            self.0
                .rotate_region_y(utf8(region_name)?, degrees)
                .map_err(|_| NucleationError::NotFound)
        }

        /// Rotate a named region about the Z axis by a multiple of 90 degrees
        /// (same semantics as `rotate_z`). `NotFound` if no region has that
        /// name.
        pub fn rotate_region_z(
            &mut self,
            region_name: &DiplomatStr,
            degrees: i32,
        ) -> Result<(), NucleationError> {
            self.0
                .rotate_region_z(utf8(region_name)?, degrees)
                .map_err(|_| NucleationError::NotFound)
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
            let name = utf8(block_name)?.to_string();
            let block = crate::BlockState::new(name);
            let shape = crate::building::ShapeEnum::Cuboid(crate::building::Cuboid::new(
                (min_x, min_y, min_z),
                (max_x, max_y, max_z),
            ));
            let brush = crate::building::SolidBrush::new(block);
            let mut tool = crate::building::BuildingTool::new(&mut self.0);
            tool.fill(&shape, &brush);
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
            if self.0.set_block_in_region_str(region, x, y, z, block) {
                Ok(())
            } else {
                Err(NucleationError::InvalidArgument)
            }
        }

        // --- Palette / bounding box / info ---

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
            let bbox = if name == "default" || name == "Default" {
                self.0.default_region.get_bounding_box()
            } else {
                self.0
                    .other_regions
                    .get(name)
                    .ok_or(NucleationError::NotFound)?
                    .get_bounding_box()
            };
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
            let region = if name == "default" || name == "Default" {
                &self.0.default_region
            } else {
                self.0
                    .other_regions
                    .get(name)
                    .ok_or(NucleationError::NotFound)?
            };
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
                map.insert(
                    k.to_string(),
                    serde_json::Value::String(v.to_string()),
                );
            }
            let json = serde_json::to_string(&serde_json::Value::Object(map))
                .unwrap_or_else(|_| "{}".to_string());
            let _ = write!(out, "{}", json);
        }
    }
}
