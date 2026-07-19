//! Geospatial → schematic: extrude OSM-style building footprints, and raise
//! terrain from an elevation heightmap. Thin wrappers over `crate::geo`; the
//! caller does the fetching and lat/lon → block projection.

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;

    /// Namespace for the geodata entry points (no network — data goes in,
    /// blocks come out).
    #[diplomat::opaque]
    pub struct Geo;

    impl Geo {
        /// Extrude building footprints into a massed schematic. `buildings_json`
        /// is a JSON array of objects:
        /// `{"polygon": [[x, z], ...], "height": <blocks>, "block": "minecraft:...",
        /// "min_y": <optional base, default 1>}`. Footprints are stamped
        /// tallest-last, so overlaps keep the tallest occupant per column.
        /// `base_block` (empty string = none) lays a ground slab at y=0 under the
        /// whole extent. Errors `Parse` on bad JSON, `InvalidArgument` on non-UTF-8.
        pub fn extrude_footprints(
            buildings_json: &DiplomatStr,
            base_block: &DiplomatStr,
            name: &DiplomatStr,
        ) -> Result<Box<Schematic>, NucleationError> {
            let json = std::str::from_utf8(buildings_json)
                .map_err(|_| NucleationError::InvalidArgument)?;
            let base =
                std::str::from_utf8(base_block).map_err(|_| NucleationError::InvalidArgument)?;
            let name = std::str::from_utf8(name).map_err(|_| NucleationError::InvalidArgument)?;

            #[derive(serde::Deserialize)]
            struct RawFootprint {
                polygon: Vec<(f64, f64)>,
                height: i32,
                #[serde(default)]
                min_y: Option<i32>,
                block: String,
            }
            let raw: Vec<RawFootprint> =
                serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
            let footprints: Vec<crate::geo::Footprint> = raw
                .into_iter()
                .map(|b| crate::geo::Footprint {
                    polygon: b.polygon,
                    y_min: b.min_y.unwrap_or(1),
                    y_max: b.height,
                    block: b.block,
                })
                .collect();
            let base_opt = if base.is_empty() { None } else { Some(base) };
            let s = crate::geo::extrude_footprints(name, &footprints, base_opt);
            Ok(Box::new(Schematic(s)))
        }

        /// Raise terrain from a heightmap. `heights_json` is a flat row-major
        /// JSON array of per-column heights (blocks); `width` is columns per row.
        /// `surface_blocks_json` is a JSON array of block names — one entry (the
        /// same surface everywhere) or one per column, row-major and the same
        /// length as `heights`, for elevation/slope banding. Each column's top
        /// `surface_depth` blocks are its surface block, the rest are
        /// `subsurface_block`. Errors `Parse` on bad JSON, `InvalidArgument` on a
        /// non-positive width, empty surface list, or non-UTF-8.
        pub fn heightmap_terrain(
            heights_json: &DiplomatStr,
            width: i32,
            surface_blocks_json: &DiplomatStr,
            subsurface_block: &DiplomatStr,
            surface_depth: i32,
            name: &DiplomatStr,
        ) -> Result<Box<Schematic>, NucleationError> {
            let hj = std::str::from_utf8(heights_json)
                .map_err(|_| NucleationError::InvalidArgument)?;
            let sj = std::str::from_utf8(surface_blocks_json)
                .map_err(|_| NucleationError::InvalidArgument)?;
            let sub = std::str::from_utf8(subsurface_block)
                .map_err(|_| NucleationError::InvalidArgument)?;
            let name = std::str::from_utf8(name).map_err(|_| NucleationError::InvalidArgument)?;
            if width <= 0 {
                return Err(NucleationError::InvalidArgument);
            }
            let heights: Vec<i32> =
                serde_json::from_str(hj).map_err(|_| NucleationError::Parse)?;
            let surface_blocks: Vec<String> =
                serde_json::from_str(sj).map_err(|_| NucleationError::Parse)?;
            if surface_blocks.is_empty() {
                return Err(NucleationError::InvalidArgument);
            }
            let s = crate::geo::heightmap_terrain(
                name,
                &heights,
                width as usize,
                &surface_blocks,
                sub,
                surface_depth,
            );
            Ok(Box::new(Schematic(s)))
        }
    }
}
