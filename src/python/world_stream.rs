//! Python bindings for the streaming world API.
//!
//! Exposes `WorldSource`, `WorldChunkIter`, `WorldChunkView`, `WorldSink`,
//! `ChunkDiff`, and module-level `diff_worlds` to Python.

use pyo3::exceptions::{PyIOError, PyStopIteration, PyValueError};
use pyo3::prelude::*;

use super::diff::PyDiff;
use super::schematic::PySchematic;
use crate::diff::Diff;
use crate::formats::world_stream::{ChunkDiff, ChunkIter, WorldChunkView, WorldSink, WorldSource};

// ─── WorldSource ─────────────────────────────────────────────────────────────

#[pyclass(name = "WorldSource")]
pub struct PyWorldSource {
    inner: WorldSource,
}

#[pymethods]
impl PyWorldSource {
    /// Open a Minecraft world directory (Anvil region files).
    #[staticmethod]
    pub fn open_dir(path: &str) -> PyResult<Self> {
        WorldSource::open_dir(std::path::Path::new(path))
            .map(|inner| Self { inner })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Load a world from a ZIP archive provided as raw bytes.
    #[staticmethod]
    pub fn from_zip_bytes(data: Vec<u8>) -> PyResult<Self> {
        WorldSource::from_zip_bytes(data)
            .map(|inner| Self { inner })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Load a world from a single `.mca` region file provided as raw bytes.
    #[staticmethod]
    pub fn from_mca_bytes(data: Vec<u8>) -> PyResult<Self> {
        WorldSource::from_mca_bytes(data)
            .map(|inner| Self { inner })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Return `(region_x, region_z)` pairs for every region present.
    pub fn region_positions(&self) -> PyResult<Vec<(i32, i32)>> {
        self.inner
            .region_positions()
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Return a lazy iterator over every chunk in the world.
    pub fn chunks(&self) -> PyResult<PyWorldChunkIter> {
        self.inner
            .chunks()
            .map(|it| PyWorldChunkIter { inner: it })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Return a lazy iterator restricted to chunks overlapping the given
    /// block-coordinate bounding box `[min_x..max_x, min_y..max_y, min_z..max_z]`.
    #[pyo3(signature = (min_x, min_y, min_z, max_x, max_y, max_z))]
    pub fn chunks_bounded(
        &self,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> PyResult<PyWorldChunkIter> {
        self.inner
            .chunks_bounded((min_x, min_y, min_z), (max_x, max_y, max_z))
            .map(|it| PyWorldChunkIter { inner: it })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Iterating a `WorldSource` directly is equivalent to calling `.chunks()`.
    pub fn __iter__(&self) -> PyResult<PyWorldChunkIter> {
        self.chunks()
    }
}

// ─── WorldChunkIter ──────────────────────────────────────────────────────────

#[pyclass(name = "WorldChunkIter", unsendable)]
pub struct PyWorldChunkIter {
    inner: ChunkIter,
}

#[pymethods]
impl PyWorldChunkIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(&mut self) -> PyResult<PyWorldChunkView> {
        match self.inner.next() {
            None => Err(PyStopIteration::new_err(())),
            Some(Ok(view)) => Ok(PyWorldChunkView { inner: view }),
            Some(Err(e)) => Err(PyIOError::new_err(e.to_string())),
        }
    }
}

// ─── WorldChunkView ──────────────────────────────────────────────────────────

#[pyclass(name = "WorldChunkView")]
pub struct PyWorldChunkView {
    pub(crate) inner: WorldChunkView,
}

#[pymethods]
impl PyWorldChunkView {
    /// Create an empty chunk at the given chunk coordinates — the starting
    /// point for generating worlds from scratch. Sections are created on
    /// demand by `set_block`. Serialized with `status = "minecraft:full"`
    /// (Minecraft will not regenerate over it) and the default data version;
    /// sections with no biome data receive the world-default biome
    /// (`WorldExportOptions::biome`, `"minecraft:plains"` unless overridden);
    /// lighting is recalculated by the game on first load.
    #[new]
    pub fn new(cx: i32, cz: i32) -> Self {
        Self {
            inner: WorldChunkView::new(cx, cz),
        }
    }

    /// Chunk X coordinate (in chunk units, not block units).
    #[getter]
    pub fn cx(&self) -> i32 {
        self.inner.cx()
    }

    /// Chunk Z coordinate (in chunk units, not block units).
    #[getter]
    pub fn cz(&self) -> i32 {
        self.inner.cz()
    }

    /// `(min_y, max_y)` block-coordinate range present in this chunk.
    pub fn y_range(&self) -> (i32, i32) {
        self.inner.y_range()
    }

    /// Return the block name at `(x, y, z)`, or `None` if the position is
    /// out of range / not stored.
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.inner.get_block(x, y, z).map(|b| b.name.to_string())
    }

    /// Set the block at `(x, y, z)` by name. Returns `True` on success.
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_name: &str) -> bool {
        self.inner
            .set_block(x, y, z, &crate::BlockState::new(block_name.to_string()))
    }

    /// Return all non-air blocks as a list of `(x, y, z, name)` tuples.
    pub fn blocks(&self) -> Vec<(i32, i32, i32, String)> {
        self.inner
            .blocks()
            .map(|(x, y, z, b)| (x, y, z, b.name.to_string()))
            .collect()
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

    /// Convert this chunk to a `Schematic`.
    pub fn to_schematic(&self) -> PySchematic {
        PySchematic::from_inner(self.inner.to_schematic())
    }
}

// ─── WorldSink ───────────────────────────────────────────────────────────────

#[pyclass(name = "WorldSink")]
pub struct PyWorldSink {
    /// `None` after `finish()` has been called.
    inner: Option<WorldSink>,
    /// `true` when opened via `create()`, `false` via `open_existing()`.
    create_mode: bool,
}

#[pymethods]
impl PyWorldSink {
    /// Create a new world in `path`. `options_json` is an optional JSON
    /// string deserialized into `WorldExportOptions` (same schema as
    /// `Schematic.to_world`); omit it for defaults.
    #[staticmethod]
    #[pyo3(signature = (path, options_json = None))]
    pub fn create(path: &str, options_json: Option<&str>) -> PyResult<Self> {
        let options = options_json
            .map(serde_json::from_str::<crate::formats::world::WorldExportOptions>)
            .transpose()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        WorldSink::create(std::path::Path::new(path), options)
            .map(|s| Self {
                inner: Some(s),
                create_mode: true,
            })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Open an existing world directory for patching.
    #[staticmethod]
    pub fn open_existing(path: &str) -> PyResult<Self> {
        WorldSink::open_existing(std::path::Path::new(path))
            .map(|s| Self {
                inner: Some(s),
                create_mode: false,
            })
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Write (append) a chunk to the sink.
    pub fn write_chunk(&mut self, view: &PyWorldChunkView) -> PyResult<()> {
        self.inner
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("WorldSink already finished"))?
            .write_chunk(&view.inner)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Overwrite a chunk in an `open_existing` sink with the data from `view`.
    ///
    /// `view` is typically a chunk retrieved from the same (or another) world
    /// and mutated via `set_block` before being passed here.
    ///
    /// The target chunk must already exist in the destination world's region
    /// file; patching a never-generated chunk raises `IOError`.
    pub fn put_chunk(&mut self, view: &PyWorldChunkView) -> PyResult<()> {
        if self.create_mode {
            return Err(PyValueError::new_err(
                "put_chunk requires a sink opened with WorldSink.open_existing(); \
                 create-mode sinks only support write_chunk()",
            ));
        }
        let sink = self
            .inner
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("WorldSink already finished"))?;
        let data = view.inner.data.clone();
        sink.patch_chunk(view.inner.cx(), view.inner.cz(), |c| c.data = data)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Flush all buffered region data to disk and close the sink.
    /// Raises `ValueError` if called more than once.
    pub fn finish(&mut self) -> PyResult<()> {
        self.inner
            .take()
            .ok_or_else(|| PyValueError::new_err("WorldSink already finished"))?
            .finish()
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }
}

// ─── ChunkDiff ───────────────────────────────────────────────────────────────

#[pyclass(name = "ChunkDiff")]
pub struct PyChunkDiff {
    pub cx: i32,
    pub cz: i32,
    /// `None` after `diff()` has been called (ownership moved to Python).
    inner_diff: Option<Diff>,
}

#[pymethods]
impl PyChunkDiff {
    /// Chunk X coordinate of the differing chunk.
    #[getter]
    pub fn cx(&self) -> i32 {
        self.cx
    }

    /// Chunk Z coordinate of the differing chunk.
    #[getter]
    pub fn cz(&self) -> i32 {
        self.cz
    }

    /// Return the `Diff` for this chunk as a JSON string (always available).
    pub fn diff_json(&self) -> PyResult<String> {
        self.inner_diff
            .as_ref()
            .map(|d| d.to_json())
            .ok_or_else(|| PyValueError::new_err("diff already consumed by .diff()"))
    }

    /// Return a `Diff` object for this chunk.
    ///
    /// This moves ownership — calling `diff()` a second time raises `ValueError`.
    /// Use `diff_json()` for read-only access that can be called repeatedly.
    pub fn diff(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let d = self
            .inner_diff
            .take()
            .ok_or_else(|| PyValueError::new_err("diff already consumed"))?;
        Py::new(py, PyDiff { inner: d }).map(|p| p.into_bound(py).into_any().unbind())
    }
}

// ─── diff_worlds (module-level function) ─────────────────────────────────────

/// Compare two worlds chunk-by-chunk using `preset` (one of `"exact"`,
/// `"shape"`, `"structural"`, `"redstone"`, `"redstone_survival"`).
///
/// Returns an eager list of `ChunkDiff` objects ordered by chunk position —
/// one per chunk that differs. Identical chunks are omitted. Chunks present
/// in only one world are included (treated as all-added / all-removed with
/// respect to the other world).
///
/// **Fail-fast semantics:** raises `IOError` on the first unreadable
/// chunk or region encountered in either world, discarding any prior
/// results accumulated in that run. An unknown `preset` name raises
/// `ValueError` before any I/O is attempted.
#[pyfunction]
pub fn diff_worlds(
    a: &PyWorldSource,
    b: &PyWorldSource,
    preset: &str,
) -> PyResult<Vec<PyChunkDiff>> {
    use crate::formats::world_stream::diff_worlds as core_diff;

    // Unknown preset → ValueError (no I/O attempted yet).
    let iter =
        core_diff(&a.inner, &b.inner, preset).map_err(|e| PyValueError::new_err(e.to_string()))?;

    // Per-chunk / stream errors → IOError (fail-fast, prior results discarded).
    iter.map(|r: Result<ChunkDiff, _>| {
        r.map(|cd| PyChunkDiff {
            cx: cd.cx,
            cz: cd.cz,
            inner_diff: Some(cd.diff),
        })
        .map_err(|e| PyIOError::new_err(e.to_string()))
    })
    .collect()
}
