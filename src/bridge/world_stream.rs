//! World streaming: iterate chunks of a Minecraft world, build chunks from
//! scratch, and write/patch worlds. Port of `ffi/world_stream.rs`.
//!
//! Omitted from port: `worldstream_free`, `worldchunkview_free`, `worldsink_free`
//! — destructors are generated (dropping an unfinished `WorldSink` abandons it,
//! matching the old `worldsink_free` semantics).

// The core WorldSink type only exists off-wasm (filesystem writer). The bridge
// opaque keeps a stable shape via this alias; all of its methods are gated
// `#[cfg(not(target_arch = "wasm32"))]`, matching the old wasm layer (which
// didn't expose world sinks at all).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type InnerWorldSink = Option<crate::formats::world_stream::WorldSink>;
#[cfg(target_arch = "wasm32")]
pub(crate) type InnerWorldSink = ();

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// A streaming iterator over the chunks of a world.
    #[diplomat::opaque_mut]
    pub struct WorldStream(crate::formats::world_stream::ChunkIter);

    /// A single decoded chunk (or a from-scratch chunk under construction).
    #[diplomat::opaque_mut]
    pub struct WorldChunkView(pub(crate) crate::formats::world_stream::WorldChunkView);

    /// A world writer. `finish` is consuming (PORTING rule 11): the inner sink is
    /// held in an `Option` and taken on `finish`; every method afterwards returns
    /// `AlreadyConsumed`. Dropping the handle without `finish` abandons the sink.
    #[diplomat::opaque_mut]
    pub struct WorldSink(super::InnerWorldSink);

    impl WorldStream {
        fn utf8(s: &[u8]) -> Result<&str, NucleationError> {
            std::str::from_utf8(s).map_err(|_| NucleationError::InvalidArgument)
        }

        /// Open a streaming iterator over a world directory.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn open_dir(path: &DiplomatStr) -> Result<Box<WorldStream>, NucleationError> {
            let path = Self::utf8(path)?;
            let source =
                crate::formats::world_stream::WorldSource::open_dir(std::path::Path::new(path))
                    .map_err(|_| NucleationError::Io)?;
            source
                .chunks()
                .map(|it| Box::new(WorldStream(it)))
                .map_err(|_| NucleationError::Io)
        }

        /// Open a streaming iterator over a world directory, bounded to the given
        /// block-coordinate box `[min_x..max_x, min_y..max_y, min_z..max_z]`.
        #[allow(clippy::too_many_arguments)]
        #[cfg(not(target_arch = "wasm32"))]
        pub fn open_dir_bounded(
            path: &DiplomatStr,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<WorldStream>, NucleationError> {
            let path = Self::utf8(path)?;
            let source =
                crate::formats::world_stream::WorldSource::open_dir(std::path::Path::new(path))
                    .map_err(|_| NucleationError::Io)?;
            source
                .chunks_bounded((min_x, min_y, min_z), (max_x, max_y, max_z))
                .map(|it| Box::new(WorldStream(it)))
                .map_err(|_| NucleationError::Io)
        }

        /// Open a streaming iterator from a zip archive in memory.
        pub fn from_zip(data: &[u8]) -> Result<Box<WorldStream>, NucleationError> {
            let source = crate::formats::world_stream::WorldSource::from_zip_bytes(data.to_vec())
                .map_err(|_| NucleationError::Parse)?;
            source
                .chunks()
                .map(|it| Box::new(WorldStream(it)))
                .map_err(|_| NucleationError::Parse)
        }

        /// Open a bounded streaming iterator from a zip archive in memory.
        #[allow(clippy::too_many_arguments)]
        pub fn from_zip_bounded(
            data: &[u8],
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<WorldStream>, NucleationError> {
            let source = crate::formats::world_stream::WorldSource::from_zip_bytes(data.to_vec())
                .map_err(|_| NucleationError::Parse)?;
            source
                .chunks_bounded((min_x, min_y, min_z), (max_x, max_y, max_z))
                .map(|it| Box::new(WorldStream(it)))
                .map_err(|_| NucleationError::Parse)
        }

        /// Advance the iterator and return the next chunk view. Errors with
        /// `NotFound` at end-of-stream (the old ABI returned NULL). Corrupt
        /// chunks are silently skipped, matching the old ABI.
        pub fn next(&mut self) -> Result<Box<WorldChunkView>, NucleationError> {
            loop {
                match self.0.next() {
                    None => return Err(NucleationError::NotFound),
                    Some(Ok(v)) => return Ok(Box::new(WorldChunkView(v))),
                    Some(Err(_)) => continue, // documented: corrupt chunks are skipped
                }
            }
        }
    }

    impl WorldChunkView {
        /// Create an empty chunk view at the given chunk coordinates — the
        /// starting point for generating worlds from scratch. Sections are
        /// created on demand by `set_block`. Serialized with
        /// `status = "minecraft:full"` (Minecraft will not regenerate over it)
        /// and the default data version.
        pub fn create(cx: i32, cz: i32) -> Box<WorldChunkView> {
            Box::new(WorldChunkView(
                crate::formats::world_stream::WorldChunkView::new(cx, cz),
            ))
        }

        /// The chunk X coordinate (in chunk units).
        pub fn cx(&self) -> i32 {
            self.0.cx()
        }

        /// The chunk Z coordinate (in chunk units).
        pub fn cz(&self) -> i32 {
            self.0.cz()
        }

        /// Convert the chunk view to a standalone schematic.
        pub fn to_schematic(&self) -> Box<Schematic> {
            Box::new(Schematic(self.0.to_schematic()))
        }

        /// Build a chunk view at (`cx`, `cz`) from a schematic — every non-air
        /// block whose world (x, z) falls in this chunk is copied in, the rest
        /// ignored. The write-side twin of `to_schematic`: this is how the
        /// schematic building tools become a *world generator*. Fill a schematic
        /// with any shape, SDF, brush, or footprint (intersect it with the
        /// chunk's cuboid to keep memory flat), then hand it here per chunk and
        /// `WorldSink.write_chunk` it. Also the transform step of a world filter:
        /// `to_schematic` a streamed chunk, edit it, rebuild with this.
        pub fn from_schematic(schematic: &Schematic, cx: i32, cz: i32) -> Box<WorldChunkView> {
            Box::new(WorldChunkView(
                crate::formats::world_stream::WorldChunkView::from_schematic(&schematic.0, cx, cz),
            ))
        }

        /// Set a block at absolute world coordinates inside this chunk view.
        /// `block_name` must be a valid Minecraft block identifier (e.g.
        /// `minecraft:stone`). Errors with `InvalidArgument` if (x, z) is outside
        /// this chunk's column.
        pub fn set_block(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            block_name: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let name =
                std::str::from_utf8(block_name).map_err(|_| NucleationError::InvalidArgument)?;
            let state = crate::BlockState::new(name.to_owned());
            if self.0.set_block(x, y, z, &state) {
                Ok(())
            } else {
                Err(NucleationError::InvalidArgument)
            }
        }

        /// Overwrite the biome of every currently-present section of the chunk
        /// view with `biome_name` (e.g. `minecraft:desert`). Sections are created
        /// lazily by `set_block`, so call this AFTER placing blocks.
        pub fn set_biome(&mut self, biome_name: &DiplomatStr) -> Result<(), NucleationError> {
            let biome =
                std::str::from_utf8(biome_name).map_err(|_| NucleationError::InvalidArgument)?;
            self.0.set_biome(biome);
            Ok(())
        }

        /// Deduped union of all sections' biome palette entries, in order of
        /// first appearance, written as a JSON array string (`[]` if no section
        /// carries biome data).
        pub fn biome_palette_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let names = self.0.biome_palette();
            let json = serde_json::to_string(&names).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }
    }

    impl WorldSink {
        /// Create a new world sink that writes fresh chunk data to `dir`.
        /// `options_json` is a serialized `WorldExportOptions` (empty string =
        /// defaults).
        #[cfg(not(target_arch = "wasm32"))]
        pub fn create(
            dir: &DiplomatStr,
            options_json: &DiplomatStr,
        ) -> Result<Box<WorldSink>, NucleationError> {
            let dir = std::str::from_utf8(dir).map_err(|_| NucleationError::InvalidArgument)?;
            let options_json =
                std::str::from_utf8(options_json).map_err(|_| NucleationError::InvalidArgument)?;
            let options = if options_json.is_empty() {
                None
            } else {
                Some(
                    serde_json::from_str::<crate::formats::world::WorldExportOptions>(options_json)
                        .map_err(|_| NucleationError::Parse)?,
                )
            };
            crate::formats::world_stream::WorldSink::create(std::path::Path::new(dir), options)
                .map(|sink| Box::new(WorldSink(Some(sink))))
                .map_err(|_| NucleationError::Io)
        }

        /// Open an existing world directory for patching via `put_chunk`.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn open_existing(dir: &DiplomatStr) -> Result<Box<WorldSink>, NucleationError> {
            let dir = std::str::from_utf8(dir).map_err(|_| NucleationError::InvalidArgument)?;
            crate::formats::world_stream::WorldSink::open_existing(std::path::Path::new(dir))
                .map(|sink| Box::new(WorldSink(Some(sink))))
                .map_err(|_| NucleationError::Io)
        }

        /// Write (append) a chunk view into the sink. The view is not consumed.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn write_chunk(&mut self, view: &WorldChunkView) -> Result<(), NucleationError> {
            let sink = self.0.as_mut().ok_or(NucleationError::AlreadyConsumed)?;
            sink.write_chunk(&view.0).map_err(|_| NucleationError::Io)
        }

        /// Overwrite the chunk at (`view.cx`, `view.cz`) of the sink's world with
        /// the supplied view's block data. Only valid on sinks opened with
        /// `open_existing`; errors with `Io` on a create-mode sink. The view is
        /// not consumed.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn put_chunk(&mut self, view: &WorldChunkView) -> Result<(), NucleationError> {
            let sink = self.0.as_mut().ok_or(NucleationError::AlreadyConsumed)?;
            let cx = view.0.cx();
            let cz = view.0.cz();
            // Clone the view's ChunkData so the closure can take ownership.
            let data_clone = view.0.data.clone();
            sink.patch_chunk(cx, cz, |chunk| {
                chunk.data = data_clone;
            })
            .map_err(|_| NucleationError::Io)
        }

        /// Finalise and flush all pending writes. Consuming (PORTING rule 11):
        /// afterwards every method on this sink returns `AlreadyConsumed`.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn finish(&mut self) -> Result<(), NucleationError> {
            let sink = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            sink.finish().map_err(|_| NucleationError::Io)
        }
    }
}
