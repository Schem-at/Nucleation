//! Definition regions: named sub-volumes of a schematic with metadata, set
//! algebra, and block filtering. Port of `ffi/definition_region.rs`.
//!
//! Omitted from port: `definitionregion_free` — destructor is generated.
//! Omitted from port: `free_float_array` (+ the `CFloatArray` type) — buffer
//! memory management is obsolete (`center_f32` writes a JSON array).
//!
//! Alias collapsing (each old alias maps to the same bridge method as its
//! target): `definitionregion_with_metadata` and
//! `definitionregion_set_metadata_mut` → `set_metadata`;
//! `definitionregion_clone_region` → `copy`; `definitionregion_intersect` and
//! `definitionregion_intersected` (identical semantics in the old ABI: both
//! returned a new region) → `intersected`; `definitionregion_subtract` and
//! `definitionregion_subtracted` (likewise identical) → `subtracted`.

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::{BlockPos, Dimensions, NucleationError};
    use diplomat_runtime::DiplomatWrite;
    use std::collections::HashMap;
    use std::fmt::Write;

    /// An inclusive block-coordinate box (a definition region is a union of
    /// these).
    #[diplomat::attr(auto, abi_compatible)]
    #[derive(Copy, Clone)]
    pub struct RegionBounds {
        pub min_x: i32,
        pub min_y: i32,
        pub min_z: i32,
        pub max_x: i32,
        pub max_y: i32,
        pub max_z: i32,
    }

    /// A named sub-volume of a schematic: a union of boxes plus a metadata map.
    #[diplomat::opaque_mut]
    pub struct DefinitionRegion(pub(crate) crate::definition_region::DefinitionRegion);

    impl DefinitionRegion {
        fn utf8(s: &[u8]) -> Result<&str, NucleationError> {
            std::str::from_utf8(s).map_err(|_| NucleationError::InvalidArgument)
        }

        /// Create a new empty region (no boxes, no metadata).
        pub fn create() -> Box<DefinitionRegion> {
            Box::new(DefinitionRegion(
                crate::definition_region::DefinitionRegion::new(),
            ))
        }

        /// A region consisting of a single inclusive box. Min/max are swapped
        /// per axis if given out of order.
        pub fn from_bounds(
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Box<DefinitionRegion> {
            Box::new(DefinitionRegion(
                crate::definition_region::DefinitionRegion::from_bounds(
                    (min_x, min_y, min_z),
                    (max_x, max_y, max_z),
                ),
            ))
        }

        /// Build a region from single-block positions crossing as flat `[i32]`
        /// chunked in threes (PORTING rule 7). Errors with `InvalidArgument` if
        /// the length is not a multiple of 3.
        pub fn from_positions(positions: &[i32]) -> Result<Box<DefinitionRegion>, NucleationError> {
            if positions.len() % 3 != 0 {
                return Err(NucleationError::InvalidArgument);
            }
            let pts: Vec<(i32, i32, i32)> =
                positions.chunks(3).map(|c| (c[0], c[1], c[2])).collect();
            Ok(Box::new(DefinitionRegion(
                crate::definition_region::DefinitionRegion::from_positions(&pts),
            )))
        }

        /// Build a region from bounding boxes crossing as flat `[i32]` chunked in
        /// sixes (`min_x, min_y, min_z, max_x, max_y, max_z` per box). Errors
        /// with `InvalidArgument` if the length is not a multiple of 6.
        pub fn from_bounding_boxes(boxes: &[i32]) -> Result<Box<DefinitionRegion>, NucleationError> {
            if boxes.len() % 6 != 0 {
                return Err(NucleationError::InvalidArgument);
            }
            let bbs: Vec<((i32, i32, i32), (i32, i32, i32))> = boxes
                .chunks(6)
                .map(|c| ((c[0], c[1], c[2]), (c[3], c[4], c[5])))
                .collect();
            Ok(Box::new(DefinitionRegion(
                crate::definition_region::DefinitionRegion::from_bounding_boxes(bbs),
            )))
        }

        /// Add an inclusive box to the region. Min/max are swapped per axis if
        /// given out of order.
        pub fn add_bounds(
            &mut self,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) {
            self.0
                .add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
        }

        /// Add a single block position (a 1x1x1 box) to the region.
        pub fn add_point(&mut self, x: i32, y: i32, z: i32) {
            self.0.add_point(x, y, z);
        }

        /// Set a metadata entry (insert or overwrite the key).
        pub fn set_metadata(
            &mut self,
            key: &DiplomatStr,
            value: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let key = Self::utf8(key)?.to_owned();
            let value = Self::utf8(value)?.to_owned();
            self.0.set_metadata(key, value);
            Ok(())
        }

        /// Errors with `NotFound` when the key is absent.
        pub fn get_metadata(
            &self,
            key: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let key = Self::utf8(key)?;
            let value = self.0.get_metadata(key).ok_or(NucleationError::NotFound)?;
            let _ = write!(out, "{}", value);
            Ok(())
        }

        /// The full metadata map, written as a JSON object string (the old ABI
        /// returned an array of `"key=value"` strings).
        pub fn all_metadata_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let json =
                serde_json::to_string(&self.0.metadata).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// The metadata keys, written as a JSON array string.
        pub fn metadata_keys_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let keys: Vec<&String> = self.0.metadata_keys();
            let json = serde_json::to_string(&keys).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Store a filter expression in the region's metadata under the `filter`
        /// key.
        pub fn add_filter(&mut self, filter: &DiplomatStr) -> Result<(), NucleationError> {
            let filter = Self::utf8(filter)?.to_owned();
            self.0.set_metadata("filter", filter);
            Ok(())
        }

        /// `true` if the region contains no boxes.
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// The total volume in blocks, summed box by box: positions covered by
        /// several overlapping boxes are counted once per box.
        pub fn volume(&self) -> u64 {
            self.0.volume()
        }

        /// Whether the position lies inside any of the region's boxes.
        pub fn contains(&self, x: i32, y: i32, z: i32) -> bool {
            self.0.contains(x, y, z)
        }

        /// Translate every box by (`dx`, `dy`, `dz`) in place.
        pub fn shift(&mut self, dx: i32, dy: i32, dz: i32) {
            self.0.shift(dx, dy, dz);
        }

        /// Grow every box in place by (`x`, `y`, `z`) outward on both sides of
        /// each axis. Negative values contract; boxes that shrink away are
        /// removed.
        pub fn expand(&mut self, x: i32, y: i32, z: i32) {
            self.0.expand(x, y, z);
        }

        /// Shrink every box in place by `amount` on all sides (the inverse of
        /// a uniform `expand`); boxes that shrink away are removed.
        pub fn contract(&mut self, amount: i32) {
            self.0.contract(amount);
        }

        /// A new region: the intersection of `self` and `other`.
        pub fn intersected(&self, other: &DefinitionRegion) -> Box<DefinitionRegion> {
            Box::new(DefinitionRegion(self.0.intersected(&other.0)))
        }

        /// A new region: the union of `self` and `other`.
        pub fn union_with(&self, other: &DefinitionRegion) -> Box<DefinitionRegion> {
            Box::new(DefinitionRegion(self.0.union(&other.0)))
        }

        /// A new region: `self` minus `other`.
        pub fn subtracted(&self, other: &DefinitionRegion) -> Box<DefinitionRegion> {
            Box::new(DefinitionRegion(self.0.subtracted(&other.0)))
        }

        /// Merge `other`'s boxes and metadata into `self`.
        pub fn merge(&mut self, other: &DefinitionRegion) {
            self.0.merge(&other.0);
        }

        /// Union `other`'s boxes into `self` in place.
        pub fn union_into(&mut self, other: &DefinitionRegion) {
            self.0.union_into(&other.0);
        }

        /// The overall bounding box. Errors with `NotFound` when the region is
        /// empty.
        pub fn bounds(&self) -> Result<RegionBounds, NucleationError> {
            let bbox = self.0.get_bounds().ok_or(NucleationError::NotFound)?;
            Ok(RegionBounds {
                min_x: bbox.min.0,
                min_y: bbox.min.1,
                min_z: bbox.min.2,
                max_x: bbox.max.0,
                max_y: bbox.max.1,
                max_z: bbox.max.2,
            })
        }

        /// The (width, height, length) of the overall bounding box; all zeros
        /// when the region is empty.
        pub fn dimensions(&self) -> Dimensions {
            let (x, y, z) = self.0.dimensions();
            Dimensions { x, y, z }
        }

        /// The center block position. Errors with `NotFound` when the region is
        /// empty.
        pub fn center(&self) -> Result<BlockPos, NucleationError> {
            let (x, y, z) = self.0.center().ok_or(NucleationError::NotFound)?;
            Ok(BlockPos { x, y, z })
        }

        /// The exact (fractional) center, written as a JSON `[x, y, z]` array of
        /// floats. Errors with `NotFound` when the region is empty.
        pub fn center_f32_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let (x, y, z) = self.0.center_f32().ok_or(NucleationError::NotFound)?;
            let json =
                serde_json::to_string(&[x, y, z]).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Every contained position, written as a flat JSON array of ints
        /// (`[x0, y0, z0, x1, y1, z1, …]`), deduplicated, in box order.
        pub fn positions_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let mut flat: Vec<i32> = Vec::new();
            for (x, y, z) in self.0.iter_positions() {
                flat.extend_from_slice(&[x, y, z]);
            }
            let json = serde_json::to_string(&flat).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Every contained position in sorted (y, z, x) order, written as a flat
        /// JSON array of ints.
        pub fn positions_sorted_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let sorted = self.0.iter_positions_sorted();
            let mut flat: Vec<i32> = Vec::with_capacity(sorted.len() * 3);
            for (x, y, z) in sorted {
                flat.extend_from_slice(&[x, y, z]);
            }
            let json = serde_json::to_string(&flat).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// The number of boxes making up this region.
        pub fn box_count(&self) -> u32 {
            self.0.box_count() as u32
        }

        /// The box at `index`. Errors with `NotFound` when out of range.
        pub fn get_box(&self, index: u32) -> Result<RegionBounds, NucleationError> {
            let (min, max) = self
                .0
                .get_box(index as usize)
                .ok_or(NucleationError::NotFound)?;
            Ok(RegionBounds {
                min_x: min.0,
                min_y: min.1,
                min_z: min.2,
                max_x: max.0,
                max_y: max.1,
                max_z: max.2,
            })
        }

        /// Every box, written as a flat JSON array of ints (six ints per box:
        /// `min_x, min_y, min_z, max_x, max_y, max_z`).
        pub fn boxes_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let boxes = self.0.get_boxes();
            let mut flat: Vec<i32> = Vec::with_capacity(boxes.len() * 6);
            for (min, max) in boxes {
                flat.extend_from_slice(&[min.0, min.1, min.2, max.0, max.1, max.2]);
            }
            let json = serde_json::to_string(&flat).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Whether all positions form a single face-connected (6-connectivity)
        /// component. `true` for empty and single-block regions.
        pub fn is_contiguous(&self) -> bool {
            self.0.is_contiguous()
        }

        /// The number of face-connected (6-connectivity) components; 0 when
        /// the region is empty.
        pub fn connected_components(&self) -> u32 {
            self.0.connected_components() as u32
        }

        /// Merge overlapping/adjacent boxes into a minimal representation.
        pub fn simplify(&mut self) {
            self.0.simplify();
        }

        /// A new region containing only the positions where `schematic` has a
        /// block named `block_name`.
        pub fn filter_by_block(
            &self,
            schematic: &Schematic,
            block_name: &DiplomatStr,
        ) -> Result<Box<DefinitionRegion>, NucleationError> {
            let name = Self::utf8(block_name)?;
            let mut filtered = self.0.clone();
            filtered.filter_by_block(&schematic.0, name);
            Ok(Box::new(DefinitionRegion(filtered)))
        }

        /// A new region containing only the positions where the block in
        /// `schematic` matches every property in `properties_json` (a JSON
        /// object of property name → value strings).
        pub fn filter_by_properties(
            &self,
            schematic: &Schematic,
            properties_json: &DiplomatStr,
        ) -> Result<Box<DefinitionRegion>, NucleationError> {
            let json = Self::utf8(properties_json)?;
            let props: HashMap<String, String> =
                serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
            Ok(Box::new(DefinitionRegion(
                self.0.filter_by_properties(&schematic.0, &props),
            )))
        }

        /// Remove every position where `schematic` has a block named
        /// `block_name` (in place).
        pub fn exclude_block(
            &mut self,
            schematic: &Schematic,
            block_name: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(block_name)?;
            self.0.exclude_block(&schematic.0, name);
            Ok(())
        }

        /// Whether any of the region's boxes intersects the given inclusive
        /// box (useful for frustum culling).
        pub fn intersects_bounds(
            &self,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> bool {
            self.0
                .intersects_bounds((min_x, min_y, min_z), (max_x, max_y, max_z))
        }

        /// A new region shifted by (`dx`, `dy`, `dz`).
        pub fn shifted(&self, dx: i32, dy: i32, dz: i32) -> Box<DefinitionRegion> {
            Box::new(DefinitionRegion(self.0.shifted(dx, dy, dz)))
        }

        /// A new region expanded by (`x`, `y`, `z`) on each axis.
        pub fn expanded(&self, x: i32, y: i32, z: i32) -> Box<DefinitionRegion> {
            Box::new(DefinitionRegion(self.0.expanded(x, y, z)))
        }

        /// A new region contracted by `amount` on every axis.
        pub fn contracted(&self, amount: i32) -> Box<DefinitionRegion> {
            Box::new(DefinitionRegion(self.0.contracted(amount)))
        }

        /// A deep copy of this region.
        pub fn copy(&self) -> Box<DefinitionRegion> {
            Box::new(DefinitionRegion(self.0.copy()))
        }

        /// Store a display color (`0xRRGGBB`) in the region's metadata.
        pub fn set_color(&mut self, color: u32) {
            self.0.set_color(color);
        }

        /// The blocks of `schematic` inside this region, written as a JSON array
        /// of `{"x", "y", "z", "name", "properties"}` objects (the old ABI
        /// returned a `CBlockArray`).
        pub fn blocks_json(
            &self,
            schematic: &Schematic,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let blocks: Vec<serde_json::Value> = self
                .0
                .iter_positions()
                .filter_map(|(x, y, z)| {
                    schematic.0.get_block(x, y, z).map(|block| {
                        serde_json::json!({
                            "x": x,
                            "y": y,
                            "z": z,
                            "name": block.name,
                            "properties": block.properties,
                        })
                    })
                })
                .collect();
            let json = serde_json::to_string(&blocks).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Write this region into `schematic`'s definition-region map under
        /// `name` (insert or overwrite).
        pub fn sync(
            &self,
            schematic: &mut Schematic,
            name: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?.to_owned();
            schematic.0.definition_regions.insert(name, self.0.clone());
            Ok(())
        }
    }

    /// Namespace type for the schematic-attached definition-region operations
    /// (PORTING rule 12; the `Schematic` opaque lives in another module, so
    /// these are statics taking it explicitly, like `Autostack`).
    #[diplomat::opaque]
    pub struct SchematicRegions;

    impl SchematicRegions {
        fn utf8(s: &[u8]) -> Result<&str, NucleationError> {
            std::str::from_utf8(s).map_err(|_| NucleationError::InvalidArgument)
        }

        /// Insert (or overwrite) `region` under `name`.
        pub fn add(
            schematic: &mut Schematic,
            name: &DiplomatStr,
            region: &DefinitionRegion,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?.to_owned();
            schematic.0.definition_regions.insert(name, region.0.clone());
            Ok(())
        }

        /// Overwrite the region stored under `name` (identical to `add` in the
        /// old ABI too; kept as a separate method for 1:1 coverage of
        /// `schematic_update_region`).
        pub fn update(
            schematic: &mut Schematic,
            name: &DiplomatStr,
            region: &DefinitionRegion,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?.to_owned();
            schematic.0.definition_regions.insert(name, region.0.clone());
            Ok(())
        }

        /// A copy of the region stored under `name`. Errors with `NotFound` when
        /// absent.
        pub fn get(
            schematic: &Schematic,
            name: &DiplomatStr,
        ) -> Result<Box<DefinitionRegion>, NucleationError> {
            let name = Self::utf8(name)?;
            schematic
                .0
                .definition_regions
                .get(name)
                .map(|r| Box::new(DefinitionRegion(r.clone())))
                .ok_or(NucleationError::NotFound)
        }

        /// Remove the region stored under `name`. Errors with `NotFound` when
        /// absent.
        pub fn remove(
            schematic: &mut Schematic,
            name: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?;
            schematic
                .0
                .definition_regions
                .remove(name)
                .map(|_| ())
                .ok_or(NucleationError::NotFound)
        }

        /// The names of every definition region, written as a JSON array string.
        pub fn names_json(
            schematic: &Schematic,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let names: Vec<&String> = schematic.0.definition_regions.keys().collect();
            let json = serde_json::to_string(&names).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Create an empty region under `name`.
        pub fn create(schematic: &mut Schematic, name: &DiplomatStr) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?.to_owned();
            schematic
                .0
                .definition_regions
                .insert(name, crate::definition_region::DefinitionRegion::new());
            Ok(())
        }

        /// Create a single-point region under `name`.
        pub fn create_from_point(
            schematic: &mut Schematic,
            name: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?.to_owned();
            let mut region = crate::definition_region::DefinitionRegion::new();
            region.add_point(x, y, z);
            schematic.0.definition_regions.insert(name, region);
            Ok(())
        }

        /// Create a single-box region under `name`.
        #[allow(clippy::too_many_arguments)]
        pub fn create_from_bounds(
            schematic: &mut Schematic,
            name: &DiplomatStr,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?.to_owned();
            let mut region = crate::definition_region::DefinitionRegion::new();
            region.add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
            schematic.0.definition_regions.insert(name, region);
            Ok(())
        }

        /// Create a single-box region under `name` and return a copy of it.
        #[allow(clippy::too_many_arguments)]
        pub fn create_region(
            schematic: &mut Schematic,
            name: &DiplomatStr,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<DefinitionRegion>, NucleationError> {
            let name = Self::utf8(name)?.to_owned();
            let mut region = crate::definition_region::DefinitionRegion::new();
            region.add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
            schematic.0.definition_regions.insert(name, region.clone());
            Ok(Box::new(DefinitionRegion(region)))
        }

        /// Add a box to the region stored under `name`. Errors with `NotFound`
        /// when absent.
        #[allow(clippy::too_many_arguments)]
        pub fn add_bounds_to(
            schematic: &mut Schematic,
            name: &DiplomatStr,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?;
            let region = schematic
                .0
                .definition_regions
                .get_mut(name)
                .ok_or(NucleationError::NotFound)?;
            region.add_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
            Ok(())
        }

        /// Add a point to the region stored under `name`. Errors with `NotFound`
        /// when absent.
        pub fn add_point_to(
            schematic: &mut Schematic,
            name: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?;
            let region = schematic
                .0
                .definition_regions
                .get_mut(name)
                .ok_or(NucleationError::NotFound)?;
            region.add_point(x, y, z);
            Ok(())
        }

        /// Set a metadata entry on the region stored under `name`. Errors with
        /// `NotFound` when absent.
        pub fn set_metadata_on(
            schematic: &mut Schematic,
            name: &DiplomatStr,
            key: &DiplomatStr,
            value: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?;
            let key = Self::utf8(key)?.to_owned();
            let value = Self::utf8(value)?.to_owned();
            let region = schematic
                .0
                .definition_regions
                .get_mut(name)
                .ok_or(NucleationError::NotFound)?;
            region.metadata.insert(key, value);
            Ok(())
        }

        /// Shift the region stored under `name` by (`dx`, `dy`, `dz`). Errors
        /// with `NotFound` when absent.
        pub fn shift_region(
            schematic: &mut Schematic,
            name: &DiplomatStr,
            dx: i32,
            dy: i32,
            dz: i32,
        ) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?;
            let region = schematic
                .0
                .definition_regions
                .get_mut(name)
                .ok_or(NucleationError::NotFound)?;
            region.shift(dx, dy, dz);
            Ok(())
        }
    }
}
