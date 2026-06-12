use std::iter::Peekable;

use wasm_bindgen::prelude::*;

use crate::formats::world_stream::{ChunkIter, WorldChunkView, WorldSource};

use super::schematic::SchematicWrapper;

#[wasm_bindgen]
pub struct WorldSourceWrapper {
    inner: WorldSource,
}

#[wasm_bindgen]
impl WorldSourceWrapper {
    pub fn from_zip_bytes(data: &[u8]) -> Result<WorldSourceWrapper, JsValue> {
        WorldSource::from_zip_bytes(data.to_vec())
            .map(|inner| WorldSourceWrapper { inner })
            .map_err(|e| JsValue::from_str(&format!("World zip error: {}", e)))
    }

    pub fn from_mca_bytes(data: &[u8]) -> Result<WorldSourceWrapper, JsValue> {
        WorldSource::from_mca_bytes(data.to_vec())
            .map(|inner| WorldSourceWrapper { inner })
            .map_err(|e| JsValue::from_str(&format!("MCA error: {}", e)))
    }

    pub fn chunks(&self) -> Result<WorldChunkIterWrapper, JsValue> {
        self.inner
            .chunks()
            .map(|it| WorldChunkIterWrapper {
                inner: it.peekable(),
            })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn chunks_bounded(
        &self,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Result<WorldChunkIterWrapper, JsValue> {
        self.inner
            .chunks_bounded((min_x, min_y, min_z), (max_x, max_y, max_z))
            .map(|it| WorldChunkIterWrapper {
                inner: it.peekable(),
            })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[wasm_bindgen]
pub struct WorldChunkIterWrapper {
    inner: Peekable<ChunkIter>,
}

#[wasm_bindgen]
impl WorldChunkIterWrapper {
    pub fn has_next(&mut self) -> bool {
        self.inner.peek().is_some()
    }

    /// Returns the next chunk, or `null`/`undefined` at the end of the stream.
    /// Throws if the chunk is corrupt or unreadable.
    pub fn next(&mut self) -> Result<Option<WorldChunkViewWrapper>, JsValue> {
        match self.inner.next() {
            None => Ok(None),
            Some(Ok(v)) => Ok(Some(WorldChunkViewWrapper { inner: v })),
            Some(Err(e)) => Err(JsValue::from_str(&e.to_string())),
        }
    }
}

#[wasm_bindgen]
pub struct WorldChunkViewWrapper {
    inner: WorldChunkView,
}

#[wasm_bindgen]
impl WorldChunkViewWrapper {
    /// Create an empty chunk at the given chunk coordinates — the starting
    /// point for generating chunks from scratch. Sections are created on
    /// demand by `set_block`. In WASM there is no `WorldSink` (no filesystem),
    /// so fabricated chunks are used via `to_schematic()` or direct inspection
    /// (`get_block`, `y_range`, ...).
    #[wasm_bindgen(constructor)]
    pub fn new(cx: i32, cz: i32) -> WorldChunkViewWrapper {
        WorldChunkViewWrapper {
            inner: WorldChunkView::new(cx, cz),
        }
    }

    pub fn cx(&self) -> i32 {
        self.inner.cx()
    }

    pub fn cz(&self) -> i32 {
        self.inner.cz()
    }

    /// Returns `[min_y, max_y]` (inclusive on both ends).
    pub fn y_range(&self) -> Vec<i32> {
        let (min_y, max_y) = self.inner.y_range();
        vec![min_y, max_y]
    }

    /// Returns the block name at world coordinates (e.g. `"minecraft:air"` for air cells),
    /// or `undefined`/`null` if the coordinates fall outside the chunk's loaded sections.
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.inner.get_block(x, y, z).map(|b| b.name.to_string())
    }

    /// Set the block at world coordinates `(x, y, z)` by name. Returns `true`
    /// on success, `false` if `(x, z)` falls outside this chunk. Creates the
    /// section if the Y level has none.
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_name: &str) -> bool {
        self.inner
            .set_block(x, y, z, &crate::BlockState::new(block_name.to_string()))
    }

    /// Overwrite the biome of every currently-present section with
    /// `biome_name` (e.g. `"minecraft:desert"`). Sections are created lazily
    /// by `set_block`, so call this AFTER placing blocks. Coarse chunk-level
    /// control; existing multi-biome data round-trips losslessly if you
    /// don't call this.
    pub fn set_biome(&mut self, biome_name: &str) {
        self.inner.set_biome(biome_name)
    }

    /// Deduped union of all sections' biome palette entries, in order of
    /// first appearance. Empty if no section carries biome data.
    pub fn biome_palette(&self) -> Vec<String> {
        self.inner.biome_palette()
    }

    pub fn to_schematic(&self) -> SchematicWrapper {
        SchematicWrapper(self.inner.to_schematic())
    }
}
