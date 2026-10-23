//! Chunked layout: 16³ chunks, each either one value or 4096 cells. Used
//! for storage boxes too large to hold densely; never-written space and
//! solid fills cost one value per chunk. The grid is anchored at `origin`
//! and grows by whole chunks, so growing never moves a cell.

use crate::bounding_box::BoundingBox;
use std::borrow::Cow;

const SHIFT: i64 = 4;
const SIDE: usize = 16;
const MASK: usize = SIDE - 1;
const CELLS: usize = SIDE * SIDE * SIDE;

#[derive(Debug, Clone)]
enum Chunk {
    /// Every cell holds the value; kept as a full row so rows can be lent out.
    Uniform([u32; SIDE]),
    /// Cells y-major, then z, then x (x fastest).
    Cells(Box<[u32; CELLS]>),
}

impl Chunk {
    fn uniform(value: u32) -> Self {
        Chunk::Uniform([value; SIDE])
    }

    #[inline(always)]
    fn get(&self, local: usize) -> u32 {
        match self {
            Chunk::Uniform(row) => row[0],
            Chunk::Cells(cells) => cells[local],
        }
    }

    #[inline(always)]
    fn row(&self, row: usize) -> &[u32] {
        match self {
            Chunk::Uniform(values) => values,
            Chunk::Cells(cells) => &cells[row * SIDE..row * SIDE + SIDE],
        }
    }

    fn cells_mut(&mut self) -> &mut [u32; CELLS] {
        if let Chunk::Uniform(values) = self {
            *self = Chunk::Cells(Box::new([values[0]; CELLS]));
        }
        match self {
            Chunk::Cells(cells) => cells,
            Chunk::Uniform(_) => unreachable!(),
        }
    }

    fn uniform_value(&self) -> Option<u32> {
        match self {
            Chunk::Uniform(values) => Some(values[0]),
            Chunk::Cells(_) => None,
        }
    }
}

/// Chunk index and in-chunk cell index of a cell `local` cells from the grid
/// origin, in a grid `dims` chunks wide (x, y, z).
#[inline(always)]
fn locate((lx, ly, lz): (usize, usize, usize), (nx, _, nz): (usize, usize, usize)) -> (usize, usize) {
    let chunk = ((ly >> SHIFT) * nz + (lz >> SHIFT)) * nx + (lx >> SHIFT);
    let cell = ((ly & MASK) << 8) | ((lz & MASK) << 4) | (lx & MASK);
    (chunk, cell)
}

fn chunks_for(cells: i64) -> usize {
    ((cells + SIDE as i64 - 1) >> SHIFT) as usize
}

/// Segments of one row, one per chunk.
pub(crate) struct ChunkRow<'a> {
    chunks: &'a [Chunk],
    chunk: usize,
    row: usize,
    offset: usize,
    x: i32,
    x1: i32,
}

impl<'a> Iterator for ChunkRow<'a> {
    type Item = (i32, &'a [u32]);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x > self.x1 {
            return None;
        }
        let len = (SIDE - self.offset).min((self.x1 - self.x) as usize + 1);
        let cells = &self.chunks[self.chunk].row(self.row)[self.offset..self.offset + len];
        let start = self.x;
        self.x += len as i32;
        self.chunk += 1;
        self.offset = 0;
        Some((start, cells))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Chunked {
    bounds: BoundingBox,
    background: u32,
    /// World position of the grid's first cell; at or below `bounds.min`.
    origin: (i32, i32, i32),
    /// Grid size in chunks (x, y, z).
    dims: (usize, usize, usize),
    /// Chunks y-major, then z, then x.
    chunks: Vec<Chunk>,
}

impl Chunked {
    /// Storage over `bounds`, every cell `background`.
    pub(super) fn new(bounds: BoundingBox, background: u32) -> Result<Self, String> {
        let (w, h, l) = bounds.get_dimensions();
        let dims = (chunks_for(w as i64), chunks_for(h as i64), chunks_for(l as i64));
        let count = dims
            .0
            .checked_mul(dims.1)
            .and_then(|n| n.checked_mul(dims.2))
            .ok_or_else(|| "Region volume exceeds addressable memory".to_string())?;
        let mut chunks = Vec::new();
        chunks
            .try_reserve_exact(count)
            .map_err(|_| format!("Cannot allocate region with {count} chunks: out of memory"))?;
        chunks.resize(count, Chunk::uniform(background));
        Ok(Chunked {
            origin: bounds.min,
            bounds,
            background,
            dims,
            chunks,
        })
    }

    /// Storage adopting `cells`, laid out over `bounds` x fastest, then z,
    /// then y. `cells.len()` must be the volume of `bounds`.
    pub(super) fn from_dense(bounds: BoundingBox, cells: Vec<u32>, background: u32) -> Self {
        let mut storage = Self::new(bounds.clone(), background)
            .expect("the dense cells already occupy the volume");
        let (w, _, l) = bounds.get_dimensions();
        let (w, l) = (w as usize, l as usize);
        let (min, max) = (bounds.min, bounds.max);
        for y in min.1..=max.1 {
            for z in min.2..=max.2 {
                let start = ((y - min.1) as usize * l + (z - min.2) as usize) * w;
                storage.write_row(y, z, min.0, &cells[start..start + w]);
            }
        }
        storage
    }

    pub(super) fn bounds(&self) -> &BoundingBox {
        &self.bounds
    }

    #[inline(always)]
    fn local(&self, x: i32, y: i32, z: i32) -> (usize, usize, usize) {
        // i64: a box may span more than i32::MAX cells along an axis.
        (
            (x as i64 - self.origin.0 as i64) as usize,
            (y as i64 - self.origin.1 as i64) as usize,
            (z as i64 - self.origin.2 as i64) as usize,
        )
    }

    /// The value at a cell inside the bounds.
    #[inline(always)]
    pub(super) fn get(&self, x: i32, y: i32, z: i32) -> u32 {
        let (chunk, cell) = locate(self.local(x, y, z), self.dims);
        self.chunks[chunk].get(cell)
    }

    /// Write a cell inside the bounds, returning its previous value.
    #[inline(always)]
    pub(super) fn set(&mut self, x: i32, y: i32, z: i32, value: u32) -> u32 {
        let (chunk, cell) = locate(self.local(x, y, z), self.dims);
        let chunk = &mut self.chunks[chunk];
        if chunk.uniform_value() == Some(value) {
            return value;
        }
        std::mem::replace(&mut chunk.cells_mut()[cell], value)
    }

    /// `(chunk index, cell row within the chunk, first x offset in the chunk)`
    /// for the start of row `(y, z)` at `x`.
    #[inline(always)]
    fn row_start(&self, x: i32, y: i32, z: i32) -> (usize, usize, usize) {
        let (lx, ly, lz) = self.local(x, y, z);
        let (chunk, _) = locate((lx, ly, lz), self.dims);
        (chunk, ((ly & MASK) << 4) | (lz & MASK), lx & MASK)
    }

    /// The cells `x0..=x1` of row `(y, z)` as `(first x, slice)` segments, in
    /// x order. The row must lie inside the bounds.
    #[inline]
    pub(super) fn row(&self, y: i32, z: i32, x0: i32, x1: i32) -> ChunkRow<'_> {
        let (chunk, row, offset) = self.row_start(x0, y, z);
        ChunkRow { chunks: &self.chunks, chunk, row, offset, x: x0, x1 }
    }

    /// Mutable counterpart of [`Self::row`], visiting each segment.
    pub(super) fn row_mut(&mut self, y: i32, z: i32, x0: i32, x1: i32, mut f: impl FnMut(i32, &mut [u32])) {
        let (mut chunk, row, mut offset) = self.row_start(x0, y, z);
        let mut x = x0;
        while x <= x1 {
            let len = (SIDE - offset).min((x1 - x) as usize + 1);
            let start = row * SIDE + offset;
            f(x, &mut self.chunks[chunk].cells_mut()[start..start + len]);
            x += len as i32;
            chunk += 1;
            offset = 0;
        }
    }

    /// Write `values` into row `(y, z)` from `x0`, leaving uniform chunks
    /// that already hold the values untouched.
    pub(super) fn write_row(&mut self, y: i32, z: i32, x0: i32, values: &[u32]) {
        let (mut chunk, row, mut offset) = self.row_start(x0, y, z);
        let mut done = 0;
        while done < values.len() {
            let len = (SIDE - offset).min(values.len() - done);
            let part = &values[done..done + len];
            let target = &mut self.chunks[chunk];
            if target.uniform_value().is_none_or(|v| part.iter().any(|&p| p != v)) {
                let start = row * SIDE + offset;
                target.cells_mut()[start..start + len].copy_from_slice(part);
            }
            done += len;
            chunk += 1;
            offset = 0;
        }
    }

    fn read_row(&self, y: i32, z: i32, x0: i32, x1: i32, out: &mut Vec<u32>) {
        out.clear();
        for (_, cells) in self.row(y, z, x0, x1) {
            out.extend_from_slice(cells);
        }
    }

    /// Reverse the cells `x0..=x1` of row `(y, z)`.
    pub(super) fn reverse_row(&mut self, y: i32, z: i32, x0: i32, x1: i32) {
        let mut cells = Vec::with_capacity((x1 - x0 + 1) as usize);
        self.read_row(y, z, x0, x1, &mut cells);
        if cells.iter().all(|&cell| cell == cells[0]) {
            return;
        }
        cells.reverse();
        self.write_row(y, z, x0, &cells);
    }

    /// Swap the full-width rows `(ya, za)` and `(yb, zb)`.
    pub(super) fn swap_rows(&mut self, (ya, za): (i32, i32), (yb, zb): (i32, i32)) {
        if (ya, za) == (yb, zb) {
            return;
        }
        let (x0, x1) = (self.bounds.min.0, self.bounds.max.0);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        self.read_row(ya, za, x0, x1, &mut a);
        self.read_row(yb, zb, x0, x1, &mut b);
        if a != b {
            self.write_row(ya, za, x0, &b);
            self.write_row(yb, zb, x0, &a);
        }
    }

    /// Swap the full layers `ya` and `yb`.
    pub(super) fn swap_layers(&mut self, ya: i32, yb: i32) {
        for z in self.bounds.min.2..=self.bounds.max.2 {
            self.swap_rows((ya, z), (yb, z));
        }
    }

    /// Set every cell to `value`.
    pub(super) fn fill(&mut self, value: u32) {
        self.chunks.fill(Chunk::uniform(value));
    }

    /// Apply `f` to every stored value, and to the background.
    pub(super) fn map_values(&mut self, mut f: impl FnMut(u32) -> u32) {
        self.background = f(self.background);
        for chunk in &mut self.chunks {
            match chunk {
                Chunk::Uniform(values) => *values = [f(values[0]); SIDE],
                Chunk::Cells(cells) => cells.iter_mut().for_each(|cell| *cell = f(*cell)),
            }
        }
    }

    /// The cells of `area` (inside the bounds), x fastest, then z, then y.
    pub(super) fn dense(&self, area: &BoundingBox) -> Cow<'_, [u32]> {
        let mut out = Vec::with_capacity(area.volume() as usize);
        for y in area.min.1..=area.max.1 {
            for z in area.min.2..=area.max.2 {
                for (_, cells) in self.row(y, z, area.min.0, area.max.0) {
                    out.extend_from_slice(cells);
                }
            }
        }
        Cow::Owned(out)
    }

    /// Grow to `new_bounds`, which must contain the current bounds; new
    /// cells hold the background. Only the chunk grid is rebuilt: the grid
    /// extends by whole chunks, so existing chunks move as a whole.
    pub(super) fn grow(&mut self, new_bounds: BoundingBox) {
        if new_bounds == self.bounds {
            return;
        }
        debug_assert!(new_bounds.contains(self.bounds.min) && new_bounds.contains(self.bounds.max));
        // Whole chunks to prepend per axis so the grid reaches the new min.
        let ahead = |origin: i32, min: i32| chunks_for(origin as i64 - min as i64);
        let shift = (
            ahead(self.origin.0, new_bounds.min.0),
            ahead(self.origin.1, new_bounds.min.1),
            ahead(self.origin.2, new_bounds.min.2),
        );
        let origin = (
            (self.origin.0 as i64 - (shift.0 * SIDE) as i64) as i32,
            (self.origin.1 as i64 - (shift.1 * SIDE) as i64) as i32,
            (self.origin.2 as i64 - (shift.2 * SIDE) as i64) as i32,
        );
        let span = |max: i32, origin: i32| chunks_for(max as i64 - origin as i64 + 1);
        let dims = (
            span(new_bounds.max.0, origin.0),
            span(new_bounds.max.1, origin.1),
            span(new_bounds.max.2, origin.2),
        );
        if dims != self.dims || shift != (0, 0, 0) {
            let mut chunks = vec![Chunk::uniform(self.background); dims.0 * dims.1 * dims.2];
            let (nx, ny, nz) = self.dims;
            let mut old = std::mem::take(&mut self.chunks).into_iter();
            for cy in 0..ny {
                for cz in 0..nz {
                    for cx in 0..nx {
                        let at = ((cy + shift.1) * dims.2 + cz + shift.2) * dims.0 + cx + shift.0;
                        chunks[at] = old.next().expect("chunk per grid cell");
                    }
                }
            }
            self.chunks = chunks;
            self.dims = dims;
            self.origin = origin;
        }
        self.bounds = new_bounds;
    }

    /// Copy the cells of `area` (inside both boxes) from `src`.
    pub(super) fn copy_box_from(&mut self, src: &Chunked, area: &BoundingBox) {
        let mut row = Vec::with_capacity((area.max.0 - area.min.0 + 1) as usize);
        for y in area.min.1..=area.max.1 {
            for z in area.min.2..=area.max.2 {
                src.read_row(y, z, area.min.0, area.max.0, &mut row);
                self.write_row(y, z, area.min.0, &row);
            }
        }
    }

    /// Move the box by an offset; cells keep their values.
    pub(super) fn translate(&mut self, dx: i32, dy: i32, dz: i32) {
        let (min, max) = (self.bounds.min, self.bounds.max);
        self.bounds = BoundingBox::new(
            (min.0 + dx, min.1 + dy, min.2 + dz),
            (max.0 + dx, max.1 + dy, max.2 + dz),
        );
        self.origin = (self.origin.0 + dx, self.origin.1 + dy, self.origin.2 + dz);
    }

    /// Count of cells inside the bounds that are not `air`, and the box they
    /// span. Uniform chunks are counted without visiting their cells.
    pub(super) fn content(&self, air: u32) -> (usize, Option<BoundingBox>) {
        let mut count = 0usize;
        let mut span: Option<BoundingBox> = None;
        let mut extend = |b: BoundingBox| {
            span = Some(match span.take() {
                Some(s) => s.union(&b),
                None => b,
            });
        };
        let (nx, _, nz) = self.dims;
        for (index, chunk) in self.chunks.iter().enumerate() {
            if chunk.uniform_value() == Some(air) {
                continue;
            }
            let (cx, cz, cy) = (index % nx, (index / nx) % nz, index / (nx * nz));
            let lo = (
                self.origin.0 as i64 + (cx * SIDE) as i64,
                self.origin.1 as i64 + (cy * SIDE) as i64,
                self.origin.2 as i64 + (cz * SIDE) as i64,
            );
            let clip = |lo: i64, b_min: i32, b_max: i32| {
                (lo.max(b_min as i64) as i32, (lo + SIDE as i64 - 1).min(b_max as i64) as i32)
            };
            let (x0, x1) = clip(lo.0, self.bounds.min.0, self.bounds.max.0);
            let (y0, y1) = clip(lo.1, self.bounds.min.1, self.bounds.max.1);
            let (z0, z1) = clip(lo.2, self.bounds.min.2, self.bounds.max.2);
            if x0 > x1 || y0 > y1 || z0 > z1 {
                continue;
            }
            if chunk.uniform_value().is_some() {
                let cells = (x1 - x0 + 1) as usize * (y1 - y0 + 1) as usize * (z1 - z0 + 1) as usize;
                count += cells;
                extend(BoundingBox::new((x0, y0, z0), (x1, y1, z1)));
                continue;
            }
            let (ox, oy, oz) = (lo.0 as i32, lo.1 as i32, lo.2 as i32);
            for y in y0..=y1 {
                for z in z0..=z1 {
                    let row = chunk.row((((y - oy) as usize) << 4) | (z - oz) as usize);
                    let row = &row[(x0 - ox) as usize..=(x1 - ox) as usize];
                    let row_count = row.iter().filter(|&&b| b != air).count();
                    if row_count == 0 {
                        continue;
                    }
                    count += row_count;
                    let first = row.iter().position(|&b| b != air).unwrap_or(0) as i32;
                    let last = row.iter().rposition(|&b| b != air).unwrap_or(0) as i32;
                    extend(BoundingBox::new((x0 + first, y, z), (x0 + last, y, z)));
                }
            }
        }
        (count, span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: index arithmetic must not wrap for a box far wider than
    /// i32 cells (the build-extractor mega-terrain repro, ~600M cells).
    #[test]
    fn locate_does_not_overflow_for_huge_boxes() {
        let dims = (82448 / 16 + 1, 384 / 16, 18944 / 16);
        let (chunk, cell) = locate((82447, 383, 18943), dims);
        assert_eq!(chunk, ((383 / 16) * dims.2 + 18943 / 16) * dims.0 + 82447 / 16);
        assert_eq!(cell, (15 << 8) | (15 << 4) | 15);
    }

    #[test]
    fn rows_cross_chunks_and_growth_keeps_cells() {
        let mut storage = Chunked::new(BoundingBox::new((-5, 0, 0), (40, 3, 2)), 0).unwrap();
        for x in -5..=40 {
            storage.set(x, 1, 2, (x + 10) as u32);
        }
        let row: Vec<u32> = storage.row(1, 2, -5, 40).flat_map(|(_, c)| c.to_vec()).collect();
        assert_eq!(row, (5..=50).collect::<Vec<u32>>());
        storage.grow(BoundingBox::new((-40, -20, -3), (70, 3, 2)));
        assert_eq!(storage.get(-5, 1, 2), 5);
        assert_eq!(storage.get(40, 1, 2), 50);
        assert_eq!(storage.get(-40, -20, -3), 0);
        storage.reverse_row(1, 2, -5, 40);
        assert_eq!(storage.get(-5, 1, 2), 50);
        assert_eq!(storage.content(0).0, 46);
    }
}
