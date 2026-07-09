//! Streaming (constant-memory) world parsing: lazy chunk iteration over
//! Anvil worlds. Sits beside the eager `formats::world` API and the
//! SchematicImporter registry (which assume whole-schematic reads).
//!
//! # Memory model
//!
//! * **Directory** and **Mca** sources: peak memory is O(one decompressed
//!   chunk) — only the chunk being decoded is held in memory at any one time.
//! * **Zip** sources: each region entry is decompressed fully into memory
//!   before lazy chunk decode begins, so peak is O(one region file).  The
//!   underlying archive `Arc<Vec<u8>>` stays alive (shared with the source and
//!   every entry read) as long as the `WorldSource` or any `ChunkIter`
//!   derived from it is alive.

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::block_entity::BlockEntity;
use crate::entity::Entity;
use crate::formats::anvil::{floor_div, parse_entity_mca, ChunkData, RegionReader};
#[cfg(not(target_arch = "wasm32"))]
use crate::formats::anvil::{write_entity_mca, EntityChunkData, McaFile};
use crate::formats::error::Result;
#[cfg(not(target_arch = "wasm32"))]
use crate::formats::world::{generate_level_dat, WorldExportOptions};
use crate::formats::world::{load_chunk_into_schematic, parse_region_filename};
use crate::universal_schematic::UniversalSchematic;
use crate::BlockState;

/// Trait object alias so ChunkIter can hold file- or memory-backed readers.
trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

type Bounds = (i32, i32, i32, i32, i32, i32);

/// Canonical chunk ordering: region (x, z), then (z, x) within the region.
/// Both ChunkIter and diff_worlds depend on this exact ordering.
pub fn chunk_order_key(cx: i32, cz: i32) -> (i32, i32, i32, i32) {
    (
        floor_div(cx, 32),
        floor_div(cz, 32),
        cz - floor_div(cz, 32) * 32,
        cx - floor_div(cx, 32) * 32,
    )
}

/// Cursor over Arc<Vec<u8>> so readers are 'static without copying.
struct ArcCursor {
    data: Arc<Vec<u8>>,
    pos: u64,
}

impl Read for ArcCursor {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.data.len() as u64;
        if self.pos >= len {
            return Ok(0);
        }
        let start = self.pos as usize;
        let n = std::cmp::min(buf.len(), self.data.len() - start);
        buf[..n].copy_from_slice(&self.data[start..start + n]);
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for ArcCursor {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let (base, offset) = match pos {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(self.pos);
            }
            SeekFrom::End(n) => (self.data.len() as i64, n),
            SeekFrom::Current(n) => (self.pos as i64, n),
        };
        let new_pos = base.checked_add(offset).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "seek overflow")
        })?;
        if new_pos < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }
        self.pos = new_pos as u64;
        Ok(self.pos)
    }
}

#[derive(Clone)]
enum SourceKind {
    #[cfg(not(target_arch = "wasm32"))]
    Directory(PathBuf),
    Zip(Arc<Vec<u8>>),
    Mca(Arc<Vec<u8>>),
}

#[derive(Clone)]
pub struct WorldSource {
    kind: SourceKind,
}

/// One region's worth of lazily readable chunks plus its (eagerly parsed,
/// per-region) entity data.
struct CurrentRegion {
    reader: RegionReader<Box<dyn ReadSeek>>,
    /// Chunk positions still to yield, canonical order, bounds-filtered.
    positions: std::vec::IntoIter<(i32, i32)>,
    /// Entities from entities/r.X.Z.mca keyed by chunk position (1.17+).
    entities: HashMap<(i32, i32), Vec<Entity>>,
}

pub struct ChunkIter {
    kind: SourceKind,
    /// Region positions still to open, sorted by (x, z).
    regions: std::vec::IntoIter<(i32, i32)>,
    bounds: Option<Bounds>,
    current: Option<CurrentRegion>,
}

pub struct WorldChunkView {
    pub(crate) data: ChunkData,
}

impl WorldSource {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_dir(path: &Path) -> Result<Self> {
        if !path.join("region").is_dir() {
            return Err(format!("{} has no region/ subdirectory", path.display()).into());
        }
        Ok(Self {
            kind: SourceKind::Directory(path.to_path_buf()),
        })
    }

    pub fn from_zip_bytes(data: Vec<u8>) -> Result<Self> {
        // validate it opens as a zip containing region/*.mca
        let mut found = false;
        let mut archive = zip::ZipArchive::new(Cursor::new(&data))?;
        for i in 0..archive.len() {
            if let Ok(f) = archive.by_index_raw(i) {
                let name = f.name().to_lowercase();
                if name.contains("region/") && name.ends_with(".mca") {
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return Err("zip contains no region/*.mca entries".into());
        }
        Ok(Self {
            kind: SourceKind::Zip(Arc::new(data)),
        })
    }

    pub fn from_mca_bytes(data: Vec<u8>) -> Result<Self> {
        if data.len() < 8192 {
            return Err("MCA file too small".into());
        }
        Ok(Self {
            kind: SourceKind::Mca(Arc::new(data)),
        })
    }

    /// Region positions discovered from file/entry names — no chunk decode.
    pub fn region_positions(&self) -> Result<Vec<(i32, i32)>> {
        let mut positions: Vec<(i32, i32)> = match &self.kind {
            #[cfg(not(target_arch = "wasm32"))]
            SourceKind::Directory(dir) => {
                let mut out = Vec::new();
                for entry in std::fs::read_dir(dir.join("region"))? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "mca") {
                        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if let Some(pos) = parse_region_filename(filename) {
                            out.push(pos);
                        }
                    }
                }
                out
            }
            SourceKind::Zip(data) => {
                let mut out = Vec::new();
                let mut archive = zip::ZipArchive::new(Cursor::new(data.as_slice()))?;
                for i in 0..archive.len() {
                    if let Ok(f) = archive.by_index_raw(i) {
                        let lower = f.name().to_lowercase();
                        if lower.contains("region/") && lower.ends_with(".mca") {
                            if let Some(pos) = parse_region_filename(f.name()) {
                                out.push(pos);
                            }
                        }
                    }
                }
                out
            }
            SourceKind::Mca(data) => {
                let reader = RegionReader::new_auto(ArcCursor {
                    data: data.clone(),
                    pos: 0,
                })?;
                vec![reader.region_position()]
            }
        };
        positions.sort();
        positions.dedup();
        Ok(positions)
    }

    pub fn chunks(&self) -> Result<ChunkIter> {
        self.chunks_impl(None)
    }

    pub fn chunks_bounded(
        &self,
        min: (i32, i32, i32),
        max: (i32, i32, i32),
    ) -> Result<ChunkIter> {
        self.chunks_impl(Some((min.0, min.1, min.2, max.0, max.1, max.2)))
    }

    fn chunks_impl(&self, bounds: Option<Bounds>) -> Result<ChunkIter> {
        let mut regions = self.region_positions()?;
        if let Some((min_x, _, min_z, max_x, _, max_z)) = bounds {
            // A region spans 512 blocks; keep regions whose footprint intersects.
            regions.retain(|(rx, rz)| {
                let (bx0, bz0) = (rx * 512, rz * 512);
                let (bx1, bz1) = (bx0 + 511, bz0 + 511);
                bx1 >= min_x && bx0 <= max_x && bz1 >= min_z && bz0 <= max_z
            });
        }
        Ok(ChunkIter {
            kind: self.kind.clone(),
            regions: regions.into_iter(),
            bounds,
            current: None,
        })
    }
}

/// Does chunk (cx, cz)'s 16x16 footprint intersect the XZ extent of bounds?
fn chunk_in_bounds(cx: i32, cz: i32, bounds: &Option<Bounds>) -> bool {
    match bounds {
        None => true,
        Some((min_x, _, min_z, max_x, _, max_z)) => {
            let (bx0, bz0) = (cx * 16, cz * 16);
            let (bx1, bz1) = (bx0 + 15, bz0 + 15);
            bx1 >= *min_x && bx0 <= *max_x && bz1 >= *min_z && bz0 <= *max_z
        }
    }
}

/// Read a zip entry (under `subdir/`, basename `r.{rx}.{rz}.mca`) fully into memory.
fn read_zip_region_entry(
    data: &Arc<Vec<u8>>,
    subdir: &str,
    rx: i32,
    rz: i32,
) -> Result<Option<Vec<u8>>> {
    let wanted = format!("r.{}.{}.mca", rx, rz);
    let needle = format!("{}/", subdir);
    let slash_wanted = format!("/{}", wanted);
    let mut archive = zip::ZipArchive::new(Cursor::new(data.as_slice()))?;
    let mut entry_idx = None;
    for i in 0..archive.len() {
        if let Ok(f) = archive.by_index_raw(i) {
            let lower = f.name().to_lowercase();
            if lower.contains(&needle) && (lower.ends_with(&slash_wanted) || lower == wanted) {
                entry_idx = Some(i);
                break;
            }
        }
    }
    match entry_idx {
        Some(i) => {
            let mut file = archive.by_index(i)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            Ok(Some(bytes))
        }
        None => Ok(None),
    }
}

impl ChunkIter {
    fn open_region(&self, rx: i32, rz: i32) -> Result<CurrentRegion> {
        let (reader, entity_chunks): (RegionReader<Box<dyn ReadSeek>>, Vec<_>) = match &self.kind {
            #[cfg(not(target_arch = "wasm32"))]
            SourceKind::Directory(dir) => {
                let region_path = dir.join("region").join(format!("r.{}.{}.mca", rx, rz));
                let file = std::fs::File::open(&region_path)?;
                let reader = RegionReader::new(Box::new(file) as Box<dyn ReadSeek>, rx, rz)?;
                let entity_path = dir.join("entities").join(format!("r.{}.{}.mca", rx, rz));
                let entity_chunks = if entity_path.is_file() {
                    let bytes = std::fs::read(&entity_path)?;
                    parse_entity_mca(&bytes, rx, rz).unwrap_or_default()
                } else {
                    Vec::new()
                };
                (reader, entity_chunks)
            }
            SourceKind::Zip(data) => {
                let region_bytes = read_zip_region_entry(data, "region", rx, rz)?
                    .ok_or_else(|| format!("region r.{}.{}.mca not found in zip", rx, rz))?;
                let reader = RegionReader::new(
                    Box::new(Cursor::new(region_bytes)) as Box<dyn ReadSeek>,
                    rx,
                    rz,
                )?;
                let entity_chunks = match read_zip_region_entry(data, "entities", rx, rz)? {
                    Some(bytes) => parse_entity_mca(&bytes, rx, rz).unwrap_or_default(),
                    None => Vec::new(),
                };
                (reader, entity_chunks)
            }
            SourceKind::Mca(data) => {
                let reader = RegionReader::new_auto(Box::new(ArcCursor {
                    data: data.clone(),
                    pos: 0,
                }) as Box<dyn ReadSeek>)?;
                (reader, Vec::new())
            }
        };

        let mut positions: Vec<(i32, i32)> = reader
            .chunk_positions()
            .into_iter()
            .filter(|(cx, cz)| chunk_in_bounds(*cx, *cz, &self.bounds))
            .collect();
        positions.sort_by_key(|(cx, cz)| chunk_order_key(*cx, *cz));

        let mut entities: HashMap<(i32, i32), Vec<Entity>> = HashMap::new();
        for chunk in entity_chunks {
            if !chunk_in_bounds(chunk.chunk_x, chunk.chunk_z, &self.bounds) {
                continue;
            }
            entities
                .entry((chunk.chunk_x, chunk.chunk_z))
                .or_default()
                .extend(chunk.entities);
        }

        Ok(CurrentRegion {
            reader,
            positions: positions.into_iter(),
            entities,
        })
    }
}

impl Iterator for ChunkIter {
    type Item = Result<WorldChunkView>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(cur) = self.current.as_mut() {
                if let Some((cx, cz)) = cur.positions.next() {
                    match cur.reader.read_chunk(cx, cz) {
                        Ok(Some(mut chunk)) => {
                            if let Some(extra) = cur.entities.remove(&(cx, cz)) {
                                chunk.entities.extend(extra);
                            }
                            return Some(Ok(WorldChunkView { data: chunk }));
                        }
                        Ok(None) => continue, // listed but absent — skip
                        Err(e) => return Some(Err(e)), // corrupt chunk: yield error, continue next call
                    }
                }
                self.current = None;
            }
            let (rx, rz) = self.regions.next()?;
            match self.open_region(rx, rz) {
                Ok(region) => self.current = Some(region),
                Err(e) => return Some(Err(e)), // unreadable region file: one error item, then move on
            }
        }
    }
}

impl WorldChunkView {
    pub fn cx(&self) -> i32 {
        self.data.x
    }
    pub fn cz(&self) -> i32 {
        self.data.z
    }

    /// (min_y, max_y) world Y covered by present sections.
    pub fn y_range(&self) -> (i32, i32) {
        let min = self
            .data
            .sections
            .iter()
            .map(|s| (s.y as i32) * 16)
            .min()
            .unwrap_or(0);
        let max = self
            .data
            .sections
            .iter()
            .map(|s| (s.y as i32) * 16 + 15)
            .max()
            .unwrap_or(0);
        (min, max)
    }

    /// Block at absolute world coordinates; None outside this chunk/sections.
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<&BlockState> {
        let local_x = x - self.data.x * 16;
        let local_z = z - self.data.z * 16;
        if !(0..16).contains(&local_x) || !(0..16).contains(&local_z) {
            return None;
        }
        let section = self.data.sections.iter().find(|s| {
            let base = (s.y as i32) * 16;
            y >= base && y < base + 16
        })?;
        let local_y = y - (section.y as i32) * 16;
        let idx = (local_y * 256 + local_z * 16 + local_x) as usize;
        let palette_idx = *section.block_states.get(idx)? as usize;
        section.palette.get(palette_idx)
    }

    /// Iterator over non-air blocks as (world_x, world_y, world_z, state).
    pub fn blocks(&self) -> impl Iterator<Item = (i32, i32, i32, &BlockState)> + '_ {
        let cx16 = self.data.x * 16;
        let cz16 = self.data.z * 16;
        self.data.sections.iter().flat_map(move |section| {
            let base_y = (section.y as i32) * 16;
            section
                .block_states
                .iter()
                .enumerate()
                .filter_map(move |(idx, pal)| {
                    let block = section.palette.get(*pal as usize)?;
                    if matches!(
                        block.name.as_str(),
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                    ) {
                        return None;
                    }
                    let local_y = (idx / 256) as i32;
                    let local_z = ((idx / 16) % 16) as i32;
                    let local_x = (idx % 16) as i32;
                    Some((cx16 + local_x, base_y + local_y, cz16 + local_z, block))
                })
        })
    }

    pub fn block_entities(&self) -> &[BlockEntity] {
        &self.data.block_entities
    }
    pub fn entities(&self) -> &[Entity] {
        &self.data.entities
    }

    /// Merge this chunk into an existing schematic at world coordinates.
    pub fn load_into(&self, schematic: &mut UniversalSchematic) {
        load_chunk_into_schematic(&self.data, schematic, None);
    }

    /// This chunk alone as a schematic (bridge to diff/fingerprint/mesh).
    pub fn to_schematic(&self) -> UniversalSchematic {
        let mut s = UniversalSchematic::new(format!("chunk_{}_{}", self.data.x, self.data.z));
        self.load_into(&mut s);
        s
    }

    /// Create an empty chunk at the given chunk coordinates — the starting
    /// point for generating worlds from scratch. Sections are created on
    /// demand by `set_block`. Serialized with `status = "minecraft:full"`
    /// (Minecraft will not regenerate over it) and the default data version;
    /// biomes default to the sink's world-default biome (plains unless
    /// overridden via `WorldExportOptions::biome` or [`set_biome`]) and
    /// lighting is recalculated by the game on first load.
    ///
    /// [`set_biome`]: WorldChunkView::set_biome
    pub fn new(cx: i32, cz: i32) -> Self {
        WorldChunkView {
            data: ChunkData {
                x: cx,
                z: cz,
                data_version: crate::formats::world::default_data_version(),
                status: "minecraft:full".to_string(),
                sections: Vec::new(),
                block_entities: Vec::new(),
                entities: Vec::new(),
                y_pos: -4,
            },
        }
    }

    /// Set a block at absolute world coordinates. Returns false if (x, z) is
    /// outside this chunk. Creates the section if the Y level has none.
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: &BlockState) -> bool {
        let local_x = x - self.data.x * 16;
        let local_z = z - self.data.z * 16;
        if !(0..16).contains(&local_x) || !(0..16).contains(&local_z) {
            return false;
        }
        let section_y = floor_div(y, 16) as i8;
        if self.data.sections.iter().all(|s| s.y != section_y) {
            self.data
                .sections
                .push(crate::formats::anvil::ChunkSection {
                    y: section_y,
                    palette: vec![BlockState::new("minecraft:air".to_string())],
                    block_states: vec![0u16; 4096],
                    biomes: None,
                });
        }
        let section = self
            .data
            .sections
            .iter_mut()
            .find(|s| s.y == section_y)
            .unwrap();
        let palette_idx = match section.palette.iter().position(|b| b == block) {
            Some(i) => i,
            None => {
                section.palette.push(block.clone());
                section.palette.len() - 1
            }
        };
        let local_y = y - (section_y as i32) * 16;
        let idx = (local_y * 256 + local_z * 16 + local_x) as usize;
        section.block_states[idx] = palette_idx as u16;
        true
    }

    /// Overwrite the biome of every currently-present section with a
    /// single-entry palette of `biome_name` (e.g. `"minecraft:desert"`).
    ///
    /// Applies to sections existing at call time — `set_block` creates
    /// sections lazily, so call this AFTER placing blocks. This is coarse
    /// chunk-level control; sub-chunk 3D biome editing is future work, but
    /// existing multi-biome data round-trips losslessly if you don't call
    /// this.
    pub fn set_biome(&mut self, biome_name: &str) {
        let compound = crate::formats::anvil::single_biome_compound(biome_name);
        for section in &mut self.data.sections {
            section.biomes = Some(compound.clone());
        }
    }

    /// Deduped union of all sections' biome palette entries, in order of
    /// first appearance. Empty if no section carries biome data.
    pub fn biome_palette(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for section in &self.data.sections {
            let Some(biomes) = &section.biomes else {
                continue;
            };
            let Ok(palette) = biomes.get::<_, &quartz_nbt::NbtList>("palette") else {
                continue;
            };
            for tag in palette.iter() {
                if let quartz_nbt::NbtTag::String(name) = tag {
                    if !out.iter().any(|n| n == name) {
                        out.push(name.clone());
                    }
                }
            }
        }
        out
    }
}

/// Region-buffered world writer: write streamed/edited chunks into a world
/// directory (`create`) or patch chunks of an existing world in place
/// (`open_existing`). Chunks written via [`WorldSink::write_chunk`] are
/// buffered one region at a time and flushed to `region/r.X.Z.mca` whenever
/// the region changes. Out-of-order region writes are supported but slower
/// (each region switch triggers a flush with a read-merge against any existing
/// file). Buffered data and `level.dat` are only written when [`finish`] is
/// called — dropping a sink without calling `finish` discards any unflushed
/// region buffer. Entities carried by the views are split into
/// `entities/r.X.Z.mca` files, mirroring the eager `world::to_world` export
/// (1.17+ layout).
///
/// [`finish`]: WorldSink::finish
#[cfg(not(target_arch = "wasm32"))]
pub struct WorldSink {
    dir: PathBuf,
    options: Option<WorldExportOptions>,
    write_level_dat: bool,
    /// Region currently being assembled: (rx, rz, chunks by local index).
    current: Option<(i32, i32, Vec<Option<ChunkData>>)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl WorldSink {
    /// Start a new world at `dir`. `finish` writes a `level.dat` generated
    /// from `options` (defaults if `None`).
    pub fn create(dir: &Path, options: Option<WorldExportOptions>) -> Result<Self> {
        std::fs::create_dir_all(dir.join("region"))?;
        Ok(Self {
            dir: dir.to_path_buf(),
            options,
            write_level_dat: true,
            current: None,
        })
    }

    /// Open an existing world for in-place patching. Does not touch level.dat.
    pub fn open_existing(dir: &Path) -> Result<Self> {
        if !dir.join("region").is_dir() {
            return Err(format!("{} has no region/ subdirectory", dir.display()).into());
        }
        Ok(Self {
            dir: dir.to_path_buf(),
            options: None,
            write_level_dat: false,
            current: None,
        })
    }

    /// Write a chunk into the sink. The chunk is buffered until the active
    /// region changes, at which point the buffered region is flushed (with a
    /// read-merge if the file already exists, so earlier writes to the same
    /// region are preserved across out-of-order sequences).
    pub fn write_chunk(&mut self, view: &WorldChunkView) -> Result<()> {
        let rx = floor_div(view.cx(), 32);
        let rz = floor_div(view.cz(), 32);
        match &mut self.current {
            Some((crx, crz, _)) if *crx == rx && *crz == rz => {}
            _ => {
                self.flush_current()?;
                self.current = Some((rx, rz, (0..1024).map(|_| None).collect()));
            }
        }
        let (_, _, chunks) = self.current.as_mut().unwrap();
        let local = ((view.cz() - rz * 32) * 32 + (view.cx() - rx * 32)) as usize;
        chunks[local] = Some(view.data.clone());
        Ok(())
    }

    /// Read chunk (cx, cz) from its region file, apply `f` to a mutable view,
    /// and rewrite the region file. Other chunks in the region are untouched.
    ///
    /// Only valid on [`open_existing`] sinks. Returns an error if called on a
    /// `create`-mode sink, because the sink may hold unflushed buffered chunks
    /// for the same region that would be silently discarded.
    ///
    /// [`open_existing`]: WorldSink::open_existing
    pub fn patch_chunk(
        &mut self,
        cx: i32,
        cz: i32,
        f: impl FnOnce(&mut WorldChunkView),
    ) -> Result<()> {
        if self.write_level_dat {
            return Err("patch_chunk is only supported on WorldSink::open_existing sinks".into());
        }
        let rx = floor_div(cx, 32);
        let rz = floor_div(cz, 32);
        let path = self.dir.join("region").join(format!("r.{}.{}.mca", rx, rz));
        let bytes = std::fs::read(&path)?;
        let mut mca = McaFile::from_bytes(&bytes, rx, rz)?;
        let local = ((cz - rz * 32) * 32 + (cx - rx * 32)) as usize;
        let chunk = mca.chunks[local]
            .take()
            .ok_or_else(|| format!("chunk ({}, {}) not present in {}", cx, cz, path.display()))?;
        let mut view = WorldChunkView { data: chunk };
        f(&mut view);
        mca.chunks[local] = Some(view.data);
        std::fs::write(&path, mca.to_bytes()?)?;
        Ok(())
    }

    fn flush_current(&mut self) -> Result<()> {
        if let Some((rx, rz, mut buffered_chunks)) = self.current.take() {
            // World-default biome (create-mode sinks only): fill in sections
            // that carry no biome data. Sections with existing biome data are
            // never touched; open_existing sinks have no options → pure
            // passthrough.
            if let Some(opts) = &self.options {
                for chunk in buffered_chunks.iter_mut().flatten() {
                    for section in &mut chunk.sections {
                        if section.biomes.is_none() {
                            section.biomes =
                                Some(crate::formats::anvil::single_biome_compound(&opts.biome));
                        }
                    }
                }
            }
            // Entities are not part of region chunk NBT (1.17+ layout, same
            // as world::to_world): split them into entities/r.X.Z.mca.
            let data_version = buffered_chunks
                .iter()
                .flatten()
                .map(|c| c.data_version)
                .next()
                .unwrap_or_else(|| self.options.clone().unwrap_or_default().data_version);

            // Read-merge: if the region file already exists, read it and
            // overlay only the buffered Some(...) slots onto it.  This
            // preserves chunks written in earlier flushes for the same region
            // (i.e. out-of-order A → B → A sequences lose nothing).
            let region_path = self.dir.join("region").join(format!("r.{}.{}.mca", rx, rz));
            let final_chunks = if region_path.is_file() {
                let existing_bytes = std::fs::read(&region_path)?;
                let mut existing_mca = McaFile::from_bytes(&existing_bytes, rx, rz)?;
                for (i, slot) in buffered_chunks.into_iter().enumerate() {
                    if slot.is_some() {
                        existing_mca.chunks[i] = slot;
                    }
                }
                existing_mca.chunks
            } else {
                buffered_chunks
            };

            let entity_chunks: Vec<EntityChunkData> = final_chunks
                .iter()
                .flatten()
                .filter(|c| !c.entities.is_empty())
                .map(|c| EntityChunkData {
                    chunk_x: c.x,
                    chunk_z: c.z,
                    entities: c.entities.clone(),
                })
                .collect();

            let mca = McaFile {
                chunks: final_chunks,
                region_x: rx,
                region_z: rz,
            };
            std::fs::write(&region_path, mca.to_bytes()?)?;

            // Read-merge for the entities file as well.
            let entities_dir = self.dir.join("entities");
            let entity_path = entities_dir.join(format!("r.{}.{}.mca", rx, rz));
            let merged_entity_chunks = if entity_path.is_file() {
                let existing_bytes = std::fs::read(&entity_path)?;
                let mut existing: Vec<EntityChunkData> =
                    parse_entity_mca(&existing_bytes, rx, rz).unwrap_or_default();
                // Buffered entries replace same-position existing entries.
                for buf_chunk in &entity_chunks {
                    if let Some(pos) = existing.iter().position(|e| {
                        e.chunk_x == buf_chunk.chunk_x && e.chunk_z == buf_chunk.chunk_z
                    }) {
                        existing[pos] = buf_chunk.clone();
                    } else {
                        existing.push(buf_chunk.clone());
                    }
                }
                existing
            } else {
                entity_chunks
            };

            if !merged_entity_chunks.is_empty() {
                std::fs::create_dir_all(&entities_dir)?;
                let bytes = write_entity_mca(&merged_entity_chunks, rx, rz, data_version)?;
                std::fs::write(&entity_path, bytes)?;
            }
        }
        Ok(())
    }

    /// Flush the buffered region and (for `create` sinks) write `level.dat`.
    /// Dropping a sink without calling `finish` discards any unflushed region
    /// buffer and skips `level.dat` generation.
    pub fn finish(mut self) -> Result<()> {
        self.flush_current()?;
        if self.write_level_dat {
            let opts = self.options.clone().unwrap_or_default();
            let level_dat = generate_level_dat(&opts)?;
            std::fs::write(self.dir.join("level.dat"), level_dat)?;
        }
        Ok(())
    }
}

/// One differing chunk produced by [`diff_worlds`]: the chunk position plus
/// the block-level [`crate::diff::Diff`] between the two worlds' copies.
pub struct ChunkDiff {
    pub cx: i32,
    pub cz: i32,
    pub diff: crate::diff::Diff,
}

/// Lockstep merge-join over two canonically ordered chunk streams.
/// Chunks present in only one world diff against an empty chunk schematic.
/// Identical chunks (distance == 0) are skipped. Stream errors from either
/// side are yielded once as error items; iteration continues afterwards.
pub fn diff_worlds(
    a: &WorldSource,
    b: &WorldSource,
    preset: &str,
) -> Result<impl Iterator<Item = Result<ChunkDiff>>> {
    let spec = crate::diff::DiffSpec::resolve(preset, &Default::default())
        .ok_or_else(|| format!("unknown diff preset: {}", preset))?;
    let mut ia = a.chunks()?.peekable();
    let mut ib = b.chunks()?.peekable();
    Ok(std::iter::from_fn(move || {
        loop {
            // Propagate stream errors as items (one per call, then continue).
            if matches!(ia.peek(), Some(Err(_))) {
                match ia.next() {
                    Some(Err(e)) => return Some(Err(e)),
                    _ => unreachable!("peeked Err"),
                }
            }
            if matches!(ib.peek(), Some(Err(_))) {
                match ib.next() {
                    Some(Err(e)) => return Some(Err(e)),
                    _ => unreachable!("peeked Err"),
                }
            }
            // Both fronts are Ok (or exhausted) — the Err guards above just
            // consumed any error item, so every unwrap below (on peeked keys
            // and on next()) is on a Some(Ok(_)) front. Keep the guards and
            // these unwraps in sync if this loop is restructured.
            let ka = ia
                .peek()
                .map(|r| r.as_ref().map(|v| chunk_order_key(v.cx(), v.cz())).unwrap());
            let kb = ib
                .peek()
                .map(|r| r.as_ref().map(|v| chunk_order_key(v.cx(), v.cz())).unwrap());
            let (va, vb) = match (ka, kb) {
                (None, None) => return None,
                (Some(_), None) => (Some(ia.next().unwrap().unwrap()), None),
                (None, Some(_)) => (None, Some(ib.next().unwrap().unwrap())),
                (Some(x), Some(y)) if x < y => (Some(ia.next().unwrap().unwrap()), None),
                (Some(x), Some(y)) if x > y => (None, Some(ib.next().unwrap().unwrap())),
                (Some(_), Some(_)) => (
                    Some(ia.next().unwrap().unwrap()),
                    Some(ib.next().unwrap().unwrap()),
                ),
            };
            let (cx, cz) = va
                .as_ref()
                .or(vb.as_ref())
                .map(|v| (v.cx(), v.cz()))
                .unwrap();
            let sa = va
                .map(|v| v.to_schematic())
                .unwrap_or_else(|| UniversalSchematic::new("empty".to_string()));
            let sb = vb
                .map(|v| v.to_schematic())
                .unwrap_or_else(|| UniversalSchematic::new("empty".to_string()));
            let d = crate::diff::diff_identity(&sa, &sb, &spec);
            if d.distance == 0 {
                continue; // identical chunk — emit nothing
            }
            return Some(Ok(ChunkDiff { cx, cz, diff: d }));
        }
    }))
}
