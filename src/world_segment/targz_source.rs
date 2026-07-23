//! Forward-only tile source over a streaming `.tar.gz` world archive.
//!
//! A gzipped tar cannot be seeked, so this is `Access::Forward`: it walks the
//! archive once, parses each `region/*.mca` entry into a tile, and streams it.
//! Everything rejected (backups, stray level files, junk coordinates) is
//! reported, never silently dropped.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use crate::world_segment::ids::TileId;
use crate::world_segment::source::{Access, TileError, TileSource};
use crate::world_segment::tile::VoxelTile;

/// Region coords beyond this are name-mangling artifacts, not real regions
/// (a real MC world spans about +/-117_000 blocks / 512).
pub const MAX_REASONABLE_REGION_COORD: i32 = 120_000;

/// `.../region/r.<x>.<z>.mca` -> `(x, z)`. Anything else -> `None`.
pub fn parse_region_coords(name: &str) -> Option<(i32, i32)> {
    // Must be inside a `region/` dir and end in `.mca` exactly (not .bak/.backup).
    if !name.contains("/region/") && !name.starts_with("region/") {
        return None;
    }
    let file = name.rsplit('/').next()?;
    let stem = file.strip_suffix(".mca")?;
    let mut parts = stem.split('.');
    if parts.next()? != "r" {
        return None;
    }
    let x: i32 = parts.next()?.parse().ok()?;
    let z: i32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None; // trailing junk => not a bare .mca
    }
    Some((x, z))
}

pub fn is_out_of_range(x: i32, z: i32) -> bool {
    x.abs() > MAX_REASONABLE_REGION_COORD || z.abs() > MAX_REASONABLE_REGION_COORD
}

/// True if region `(rx,rz)`'s 512-block span lies entirely outside
/// `[-border, border]` on either axis.
pub fn region_outside_border(rx: i32, rz: i32, border: i32) -> bool {
    let span = |r: i32| (r * 512, r * 512 + 511);
    let (x0, x1) = span(rx);
    let (z0, z1) = span(rz);
    x1 < -border || x0 > border || z1 < -border || z0 > border
}

pub struct TarGzSource {
    path: PathBuf,
    min_y: i32,
    max_y: i32,
    world_border: Option<i32>,
}

impl TarGzSource {
    pub fn open(path: impl AsRef<Path>, min_y: i32, max_y: i32) -> Result<Self, TileError> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(TileError::Io(format!("{} does not exist", path.display())));
        }
        Ok(TarGzSource { path, min_y, max_y, world_border: None })
    }

    pub fn with_world_border(mut self, border_abs: i32) -> Self {
        self.world_border = Some(border_abs.abs());
        self
    }

    /// Classify an entry name. `Ok(coords)` to process; `Err(reason)` to reject
    /// (caller logs). Non-region entries return `Err` too.
    fn classify(&self, name: &str) -> Result<(i32, i32), String> {
        let (x, z) = parse_region_coords(name).ok_or_else(|| "not a region .mca".to_string())?;
        if is_out_of_range(x, z) {
            return Err(format!("out-of-range coords ({x},{z})"));
        }
        if let Some(border) = self.world_border {
            if region_outside_border(x, z, border) {
                return Err(format!("outside world border ({x},{z})"));
            }
        }
        Ok((x, z))
    }
}

impl TileSource for TarGzSource {
    fn access(&self) -> Access {
        Access::Forward
    }

    fn tile_ids(&self) -> Result<Vec<TileId>, TileError> {
        // Ids are not known without a full pass; callers should stream.
        Ok(Vec::new())
    }

    fn tile(&self, _id: TileId) -> Result<Option<VoxelTile>, TileError> {
        Err(TileError::NotRandomAccess)
    }

    fn for_each_tile(
        &self,
        f: &mut dyn FnMut(VoxelTile) -> Result<(), TileError>,
    ) -> Result<(), TileError> {
        use flate2::read::GzDecoder;
        let file = File::open(&self.path).map_err(|e| TileError::Io(e.to_string()))?;
        let gz = GzDecoder::new(BufReader::new(file));
        let mut archive = tar::Archive::new(gz);
        let entries = archive.entries().map_err(|e| TileError::Io(e.to_string()))?;
        let mut buf: Vec<u8> = Vec::new();
        for entry in entries {
            let mut entry = entry.map_err(|e| TileError::Io(e.to_string()))?;
            let name = entry
                .path()
                .map_err(|e| TileError::Io(e.to_string()))?
                .to_string_lossy()
                .to_string();
            let (rx, rz) = match self.classify(&name) {
                Ok(c) => c,
                Err(reason) => {
                    // Report, do not silently drop. eprintln keeps the core
                    // free of a logging dependency; callers can capture stderr.
                    eprintln!("world_segment: skipping {name}: {reason}");
                    continue;
                }
            };
            buf.clear();
            entry.read_to_end(&mut buf).map_err(|e| TileError::Io(e.to_string()))?;
            if buf.is_empty() {
                eprintln!("world_segment: skipping {name}: empty entry");
                continue;
            }
            // The filename coords (rx,rz) are used ONLY for junk filtering above
            // (the sign-extension artifact is a filename problem). The tile's
            // actual id comes from the decoded region via region_positions(),
            // which reads the region header — this avoids trusting a possibly
            // mangled filename for identity.
            //
            // Per-entry decode failures below are reported and skipped, not
            // propagated: a single malformed region must never abort the rest
            // of the archive (see module docs).
            let source = match crate::formats::world_stream::WorldSource::from_mca_bytes(buf.clone())
            {
                Ok(source) => source,
                Err(e) => {
                    eprintln!("world_segment: skipping {name}: malformed region data ({e})");
                    continue;
                }
            };
            let positions = match source.region_positions() {
                Ok(positions) => positions,
                Err(e) => {
                    eprintln!("world_segment: skipping {name}: could not read region positions ({e})");
                    continue;
                }
            };
            let (tx, tz) = match positions.first() {
                Some(&p) => p,
                None => {
                    eprintln!("world_segment: skipping {name}: no region position in header");
                    continue;
                }
            };
            let _ = (rx, rz); // filename coords already served their filtering purpose
            let tiles = crate::world_segment::world_source::WorldSourceTiles::new(
                source, self.min_y, self.max_y,
            );
            // A corrupt chunk inside an otherwise valid region header surfaces
            // here as TileError::Malformed from collect_tile(); report and
            // skip this region, same as the decode failures above, rather
            // than aborting the whole archive. `f(tile)?` intentionally still
            // propagates — a callback error is a legitimate caller-level
            // abort, not a per-tile decode failure.
            match tiles.tile(TileId { x: tx, z: tz }) {
                Ok(Some(tile)) => f(tile)?,
                Ok(None) => {}
                Err(e) => {
                    eprintln!("world_segment: skipping {name}: {e}");
                    continue;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_region_names() {
        assert_eq!(parse_region_coords("world/region/r.0.0.mca"), Some((0, 0)));
        assert_eq!(parse_region_coords("build/region/r.-1.-2.mca"), Some((-1, -2)));
        assert_eq!(parse_region_coords("region/r.5.-3.mca"), Some((5, -3)));
    }

    #[test]
    fn rejects_non_region_and_backup_entries() {
        assert_eq!(parse_region_coords("world/region/r.0.0.mca.bak"), None);
        assert_eq!(parse_region_coords("world/region/r.-2.-8.mca.5409229131383617463.backup"), None);
        assert_eq!(parse_region_coords("world/level.dat"), None);
        assert_eq!(parse_region_coords("world/entities/r.0.0.mca"), None); // not /region/
        assert_eq!(parse_region_coords("world/region/notaregion.mca"), None);
    }

    #[test]
    fn flags_sign_extension_coordinates() {
        // 4194303 = 2^22 - 1 = -1 misread as unsigned 22-bit. Parsed, then
        // rejected by the range guard.
        assert!(is_out_of_range(4194303, -3));
        assert!(!is_out_of_range(5, -3));
    }

    #[test]
    fn world_border_excludes_far_regions() {
        // "Outside" means the span lies ENTIRELY beyond [-border, border].
        // border 8192: region 17 covers blocks 8704..9215, fully beyond -> excluded.
        // region 16 covers 8192..8703 and block 8192 is ON the border, so it
        // TOUCHES and must be kept. region 0 is well inside.
        assert!(region_outside_border(17, 0, 8192), "fully beyond the border");
        assert!(!region_outside_border(16, 0, 8192), "touches the border, keep it");
        assert!(!region_outside_border(15, 0, 8192), "inside");
        assert!(!region_outside_border(0, 0, 8192), "inside");
    }

    /// Builds a gzipped tar (in memory) containing several entries that each
    /// get skipped for a *different* reason, then streams it through
    /// `for_each_tile`. Pins the continue-on-error contract: none of these
    /// entries is a valid region, so the callback must never fire, yet
    /// `for_each_tile` must still return `Ok(())` rather than aborting on the
    /// first (malformed-region) entry.
    ///
    /// No `.mca` fixture is needed: `from_mca_bytes` rejects anything under
    /// 8192 bytes, so a short byte string is enough to drive the Malformed
    /// path that Finding 1 fixes.
    #[test]
    fn for_each_tile_skips_bad_entries_and_keeps_streaming() {
        use std::io::Write;

        // Build the archive bytes in memory first.
        let mut tar_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);

            // 1. Looks like a region file by name, but is far too small to be
            //    a real .mca -> from_mca_bytes() returns Err (Finding 1 path).
            let tiny = b"not a real region file";
            let mut header = tar::Header::new_gnu();
            header.set_size(tiny.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "world/region/r.0.0.mca", &tiny[..])
                .unwrap();

            // 2. Non-region entry -> rejected by classify().
            let leveldat = b"junk level.dat contents";
            let mut header = tar::Header::new_gnu();
            header.set_size(leveldat.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "world/level.dat", &leveldat[..])
                .unwrap();

            // 3. Backup file -> also rejected by classify().
            let backup = b"backup region bytes";
            let mut header = tar::Header::new_gnu();
            header.set_size(backup.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "world/region/r.0.0.mca.backup", &backup[..])
                .unwrap();

            builder.finish().unwrap();
        }

        // Gzip it.
        let mut gz_bytes = Vec::new();
        {
            let mut encoder =
                flate2::write::GzEncoder::new(&mut gz_bytes, flate2::Compression::default());
            encoder.write_all(&tar_bytes).unwrap();
            encoder.finish().unwrap();
        }

        // Write to a fixed, unique temp path (no tempfile dep: not present in
        // [dev-dependencies], so use std::env::temp_dir() and clean up after).
        let path = std::env::temp_dir()
            .join("nucleation_test_targz_source_skips_bad_entries.tar.gz");
        std::fs::write(&path, &gz_bytes).unwrap();

        let source = TarGzSource::open(&path, -64, 320).unwrap();
        let mut call_count = 0usize;
        let result = source.for_each_tile(&mut |_tile| {
            call_count += 1;
            Ok(())
        });

        std::fs::remove_file(&path).ok();

        // Every entry above is rejected for a different reason; none is a
        // usable tile.
        assert_eq!(call_count, 0, "no entry in this archive is a valid region");
        // The critical assertion: a malformed per-entry decode (entry 1) must
        // not abort the whole stream. Before Finding 1's fix, the `?` on
        // `from_mca_bytes` propagated out of `for_each_tile`, turning this
        // into `Err(..)`.
        assert!(
            result.is_ok(),
            "for_each_tile must skip a malformed region entry, not abort the archive: {result:?}"
        );

        // NOTE: the corrupt-CHUNK-in-an-otherwise-valid-region-header path
        // (WorldSourceTiles::collect_tile returning TileError::Malformed from
        // a bad chunk, not a bad header) is NOT exercised here: it needs a
        // real `.mca` fixture with a valid region header but one corrupt
        // chunk, which isn't cheap to synthesize in-memory. That branch is
        // covered by code inspection only (see the match on `tiles.tile(...)`
        // above in `for_each_tile`), and this gap is recorded intentionally
        // rather than left silent.
        //
        // Same fixture gap for the "callback error still propagates" case:
        // every entry in this in-memory archive is rejected before a tile is
        // ever decoded (see call_count == 0 above), so the callback can never
        // run here either, and a cheap in-memory extension can't drive it.
        // That behaviour is instead guarded structurally: `f(tile)?` in
        // `for_each_tile` still uses `?` (only `tiles.tile(...)`'s own Err
        // was changed to report-and-continue), so any Err the callback
        // returns is untouched and propagates by construction. Confirmed by
        // inspection of the match arms above, not by a running test.
    }
}
