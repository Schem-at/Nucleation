//! Source-neutral generated world chunks for portable language bindings.

use std::sync::Arc;

use crate::world_generation::WorldGenerationError;

pub(crate) enum InnerWorldGenerator {
    Source(Arc<dyn crate::world_generation::ChunkSource>),
    /// Held behind an `Arc` so handing the composite to a stream or a generate
    /// call is a refcount bump rather than a deep copy of the layer list.
    /// `add_layer` uses `Arc::make_mut`, so the copy happens at most once after
    /// a snapshot has been taken.
    Composite(Arc<crate::world_generation::CompositeChunkSource>),
}

impl InnerWorldGenerator {
    fn source(&self) -> Arc<dyn crate::world_generation::ChunkSource> {
        match self {
            Self::Source(source) => source.clone(),
            Self::Composite(source) => source.clone(),
        }
    }
}

/// Map a world-generation failure onto the bridge's shared error enum.
///
/// Everything the caller can fix by passing different arguments collapses to
/// `InvalidArgument`; a source that fails on a well-formed request reports
/// `Generation`, so bindings can tell "you asked for something impossible"
/// apart from "the generator broke".
pub(crate) fn map_generation_error(
    error: WorldGenerationError,
) -> super::shared::ffi::NucleationError {
    use super::shared::ffi::NucleationError;
    match error {
        WorldGenerationError::InvalidYBounds
        | WorldGenerationError::YBoundsOutOfRange
        | WorldGenerationError::InvalidProvenance
        | WorldGenerationError::InvalidSdf(_)
        | WorldGenerationError::InvalidChunkBounds
        | WorldGenerationError::TooManyChunks
        | WorldGenerationError::TooManySourceLayers
        | WorldGenerationError::InvalidProjectedFootprints(_)
        | WorldGenerationError::InvalidCellularSource(_)
        | WorldGenerationError::CoordinateOverflow
        | WorldGenerationError::CoordinatePrecision => NucleationError::InvalidArgument,
        // `MismatchedChunkCoordinates` is a misbehaving source, not a bad request.
        // `WorldGenerationError` is `#[non_exhaustive]`, so anything added later
        // defaults to the conservative "the generator broke" answer.
        _ => NucleationError::Generation,
    }
}

#[diplomat::bridge]
pub mod ffi {
    use super::super::building::ffi::Brush;
    use super::super::sdf::ffi::Sdf;
    use super::super::shared::ffi::NucleationError;
    use super::super::world_stream::ffi::WorldChunkView;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;
    use std::sync::Arc;

    /// Coverage of a generated chunk by its source graph.
    pub enum GeneratedChunkCoverage {
        Complete,
        Partial,
        Outside,
    }

    impl GeneratedChunkCoverage {
        fn from_core(value: crate::world_generation::ChunkCoverage) -> Self {
            match value {
                crate::world_generation::ChunkCoverage::Complete => Self::Complete,
                crate::world_generation::ChunkCoverage::Partial => Self::Partial,
                crate::world_generation::ChunkCoverage::Outside => Self::Outside,
            }
        }
    }

    /// How a composite layer treats non-air blocks already emitted by earlier layers.
    pub enum GeneratedChunkOverlayMode {
        Replace,
        KeepExisting,
    }

    impl GeneratedChunkOverlayMode {
        fn to_core(self) -> crate::world_generation::ChunkOverlayMode {
            match self {
                Self::Replace => crate::world_generation::ChunkOverlayMode::Replace,
                Self::KeepExisting => crate::world_generation::ChunkOverlayMode::KeepExisting,
            }
        }
    }

    /// An immutable native chunk source graph.
    ///
    /// Generated bindings expose concrete source constructors rather than host
    /// callbacks, so SDF evaluation and block placement stay entirely in Rust.
    #[diplomat::opaque_mut]
    pub struct WorldGenerator(pub(crate) super::InnerWorldGenerator);

    /// A finite, lazy, canonical region-major traversal of a generator.
    #[diplomat::opaque_mut]
    pub struct GeneratedWorldStream(crate::world_generation::GeneratedChunkStream);

    /// One generated chunk plus coverage and source-version metadata.
    /// Call `take_view` once to move its chunk into the existing world-stream API.
    #[diplomat::opaque_mut]
    pub struct GeneratedChunk(Option<crate::world_generation::ChunkResult>);

    /// Immutable hashed-cell variation shared by coordinated SDF source layers.
    #[diplomat::opaque]
    pub struct CellularSdfConfig(pub(crate) crate::world_generation::CellularSdfConfig);

    impl CellularSdfConfig {
        /// Validates every field up front, so a config that constructs here is
        /// never rejected for its own values by a later `cellular_sdf` call.
        #[allow(clippy::too_many_arguments)]
        pub fn create(
            cell_size_x: i32,
            cell_size_z: i32,
            seed: u64,
            max_jitter_x: f32,
            max_jitter_z: f32,
            max_yaw_degrees: f32,
            min_scale: f32,
            max_scale: f32,
            min_y_offset: i32,
            max_y_offset: i32,
            presence_numerator: u32,
            presence_denominator: u32,
            feature_salt: u64,
        ) -> Result<Box<CellularSdfConfig>, NucleationError> {
            let config = crate::world_generation::CellularSdfConfig {
                cell_size_x,
                cell_size_z,
                seed,
                max_jitter_x,
                max_jitter_z,
                max_yaw_degrees,
                min_scale,
                max_scale,
                min_y_offset,
                max_y_offset,
                presence_numerator,
                presence_denominator,
                feature_salt,
            };
            config.validate().map_err(super::map_generation_error)?;
            Ok(Box::new(CellularSdfConfig(config)))
        }
    }

    impl WorldGenerator {
        fn utf8(value: &[u8]) -> Result<&str, NucleationError> {
            std::str::from_utf8(value).map_err(|_| NucleationError::InvalidArgument)
        }

        /// Create an SDF-backed source evaluated at voxel centers over the inclusive
        /// Y range. `source_id` and `version` become chunk provenance/cache metadata.
        pub fn sdf(
            volume: &Sdf,
            material: &Brush,
            min_y: i32,
            max_y: i32,
            source_id: &DiplomatStr,
            version: &DiplomatStr,
        ) -> Result<Box<WorldGenerator>, NucleationError> {
            let provenance = crate::world_generation::SourceProvenance::new(
                Self::utf8(source_id)?,
                Self::utf8(version)?,
            )
            .map_err(super::map_generation_error)?;
            let source = crate::world_generation::SdfChunkSource::new(
                volume.0.clone(),
                material.0.clone(),
                min_y,
                max_y,
                provenance,
            )
            .map_err(super::map_generation_error)?;
            Ok(Box::new(WorldGenerator(
                super::InnerWorldGenerator::Source(Arc::new(source)),
            )))
        }

        /// Create a sparse infinite source by placing a bounded SDF motif once per
        /// deterministically transformed cell. Reuse `config` across layers to keep
        /// terrain, water, vegetation, paths, and structures coordinated.
        pub fn cellular_sdf(
            volume: &Sdf,
            material: &Brush,
            min_y: i32,
            max_y: i32,
            config: &CellularSdfConfig,
            source_id: &DiplomatStr,
            version: &DiplomatStr,
        ) -> Result<Box<WorldGenerator>, NucleationError> {
            let provenance = crate::world_generation::SourceProvenance::new(
                Self::utf8(source_id)?,
                Self::utf8(version)?,
            )
            .map_err(super::map_generation_error)?;
            let source = crate::world_generation::CellularSdfChunkSource::new(
                volume.0.clone(),
                material.0.clone(),
                min_y,
                max_y,
                config.0,
                provenance,
            )
            .map_err(super::map_generation_error)?;
            Ok(Box::new(WorldGenerator(
                super::InnerWorldGenerator::Source(Arc::new(source)),
            )))
        }

        /// Create a sparse source from projected building footprints, including
        /// caller-projected OSM-derived data.
        /// `buildings_json` uses the same schema as `Geo.extrude_footprints`:
        /// `[{"polygon":[[x,z],...],"height":40,"min_y":1,
        /// "block":"minecraft:bricks"}]`. `height` is the absolute top Y, matching
        /// `Geo.extrude_footprints`. Fetching and lat/lon projection stay
        /// caller-controlled; this source rasterizes only requested chunks.
        pub fn projected_footprints(
            buildings_json: &DiplomatStr,
            base_block: &DiplomatStr,
            source_id: &DiplomatStr,
            version: &DiplomatStr,
        ) -> Result<Box<WorldGenerator>, NucleationError> {
            const MAX_JSON_BYTES: usize = 64 * 1024 * 1024;
            if buildings_json.len() > MAX_JSON_BYTES {
                return Err(NucleationError::InvalidArgument);
            }
            let json = Self::utf8(buildings_json)?;
            let base = Self::utf8(base_block)?;
            let footprints =
                crate::geo::parse_footprints_json(json).map_err(|_| NucleationError::Parse)?;
            let provenance = crate::world_generation::SourceProvenance::new(
                Self::utf8(source_id)?,
                Self::utf8(version)?,
            )
            .map_err(super::map_generation_error)?;
            let source = crate::world_generation::ProjectedFootprintChunkSource::new(
                footprints,
                if base.is_empty() {
                    None
                } else {
                    Some(base.to_string())
                },
                provenance,
            )
            .map_err(super::map_generation_error)?;
            Ok(Box::new(WorldGenerator(
                super::InnerWorldGenerator::Source(Arc::new(source)),
            )))
        }

        /// Create an initially empty ordered source composition.
        pub fn composite(
            source_id: &DiplomatStr,
            version: &DiplomatStr,
        ) -> Result<Box<WorldGenerator>, NucleationError> {
            let provenance = crate::world_generation::SourceProvenance::new(
                Self::utf8(source_id)?,
                Self::utf8(version)?,
            )
            .map_err(super::map_generation_error)?;
            Ok(Box::new(WorldGenerator(
                super::InnerWorldGenerator::Composite(Arc::new(
                    crate::world_generation::CompositeChunkSource::new(provenance),
                )),
            )))
        }

        /// Append a source to a composite. Later `Replace` layers win at occupied
        /// voxels; `KeepExisting` layers only fill air. Errors on non-composites.
        ///
        /// Streams already created from this generator keep the layer list they
        /// were built with; only later `generate`/`stream` calls see the addition.
        pub fn add_layer(
            &mut self,
            source: &WorldGenerator,
            mode: GeneratedChunkOverlayMode,
        ) -> Result<(), NucleationError> {
            let composite = match &mut self.0 {
                super::InnerWorldGenerator::Composite(composite) => composite,
                super::InnerWorldGenerator::Source(_) => {
                    return Err(NucleationError::InvalidArgument)
                }
            };
            Arc::make_mut(composite)
                .add_layer(source.0.source(), mode.to_core())
                .map_err(super::map_generation_error)
        }

        /// Generate one random-access chunk.
        pub fn generate(&self, cx: i32, cz: i32) -> Result<Box<GeneratedChunk>, NucleationError> {
            self.0
                .source()
                .generate(crate::world_generation::ChunkRequest::new(cx, cz))
                .map(|result| Box::new(GeneratedChunk(Some(result))))
                .map_err(super::map_generation_error)
        }

        /// Traverse an inclusive chunk rectangle lazily in canonical region-major
        /// order. The stream snapshots the generator's sources at creation, so
        /// later `add_layer` calls do not affect a stream already in flight.
        pub fn stream(
            &self,
            min_cx: i32,
            min_cz: i32,
            max_cx: i32,
            max_cz: i32,
        ) -> Result<Box<GeneratedWorldStream>, NucleationError> {
            let bounds = crate::world_generation::ChunkBounds::new(min_cx, min_cz, max_cx, max_cz)
                .map_err(super::map_generation_error)?;
            Ok(Box::new(GeneratedWorldStream(
                crate::world_generation::GeneratedChunkStream::new(self.0.source(), bounds),
            )))
        }
    }

    impl GeneratedWorldStream {
        /// Number of chunks not yet requested from the source.
        pub fn remaining(&self) -> u64 {
            self.0.remaining()
        }

        /// Generate and return the next chunk. Returns `NotFound` at end-of-stream,
        /// and `Generation` if the underlying source failed on a valid request.
        pub fn next(&mut self) -> Result<Box<GeneratedChunk>, NucleationError> {
            match self.0.next() {
                Some(Ok(result)) => Ok(Box::new(GeneratedChunk(Some(result)))),
                Some(Err(error)) => Err(super::map_generation_error(error)),
                None => Err(NucleationError::NotFound),
            }
        }
    }

    impl GeneratedChunk {
        fn result(&self) -> Result<&crate::world_generation::ChunkResult, NucleationError> {
            self.0.as_ref().ok_or(NucleationError::AlreadyConsumed)
        }

        pub fn cx(&self) -> Result<i32, NucleationError> {
            Ok(self.result()?.chunk().cx())
        }

        pub fn cz(&self) -> Result<i32, NucleationError> {
            Ok(self.result()?.chunk().cz())
        }

        pub fn coverage(&self) -> Result<GeneratedChunkCoverage, NucleationError> {
            Ok(GeneratedChunkCoverage::from_core(self.result()?.coverage()))
        }

        pub fn source_id(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let _ = write!(out, "{}", self.result()?.provenance().source_id());
            Ok(())
        }

        pub fn version(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let _ = write!(out, "{}", self.result()?.provenance().version());
            Ok(())
        }

        /// Consume the generated chunk payload. Metadata access and a second call
        /// return `AlreadyConsumed` afterwards.
        pub fn take_view(&mut self) -> Result<Box<WorldChunkView>, NucleationError> {
            let result = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            Ok(Box::new(WorldChunkView(result.into_chunk())))
        }
    }
}
