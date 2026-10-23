//! A region's voxel storage: one palette index per cell of a storage box.
//!
//! Cells are addressed by world coordinates inside [`Storage::bounds`]; rows
//! run along x and are handed out as segments, so callers never assume one
//! contiguous buffer. Cells never written hold the background value (the
//! region's air index).
//!
//! Boxes up to [`DENSE_MAX_CELLS`] are one contiguous buffer, the fastest
//! layout for the boxes schematics usually span. Larger boxes switch to 16³
//! chunks, where empty and uniform space costs one value per chunk instead
//! of four bytes per cell (two blocks 600 apart: under 10 MB instead of
//! 870 MB).

mod chunked;
mod dense;

use crate::bounding_box::BoundingBox;
use chunked::{ChunkRow, Chunked};
use dense::Dense;
use std::borrow::Cow;

/// Largest box held densely: 16M cells (64 MB).
const DENSE_MAX_CELLS: u64 = 1 << 24;

#[derive(Debug, Clone)]
pub(crate) enum Storage {
    Dense(Dense),
    Chunked(Chunked),
}

/// Segments of one row.
pub(crate) enum Row<'a> {
    Dense(std::iter::Once<(i32, &'a [u32])>),
    Chunked(ChunkRow<'a>),
}

impl<'a> Iterator for Row<'a> {
    type Item = (i32, &'a [u32]);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Row::Dense(row) => row.next(),
            Row::Chunked(row) => row.next(),
        }
    }
}

macro_rules! each {
    ($self:expr, $s:ident => $body:expr) => {
        match $self {
            Storage::Dense($s) => $body,
            Storage::Chunked($s) => $body,
        }
    };
}

// The chunked paths stay out of line so that the dense hot path, inlined
// into every per-cell caller, stays small.
#[inline(never)]
fn chunked_get(s: &Chunked, x: i32, y: i32, z: i32) -> u32 {
    s.get(x, y, z)
}

#[inline(never)]
fn chunked_set(s: &mut Chunked, x: i32, y: i32, z: i32, value: u32) -> u32 {
    s.set(x, y, z, value)
}

impl Storage {
    /// Storage over `bounds`, every cell `background`.
    pub fn new(bounds: BoundingBox, background: u32) -> Result<Self, String> {
        if bounds.volume() <= DENSE_MAX_CELLS {
            Ok(Storage::Dense(Dense::new(bounds, background)?))
        } else {
            Ok(Storage::Chunked(Chunked::new(bounds, background)?))
        }
    }

    /// Storage adopting `cells`, laid out over `bounds` x fastest, then z,
    /// then y. `cells.len()` must be the volume of `bounds`.
    pub fn from_dense(bounds: BoundingBox, cells: Vec<u32>, background: u32) -> Self {
        if bounds.volume() <= DENSE_MAX_CELLS {
            Storage::Dense(Dense::from_dense(bounds, cells, background))
        } else {
            Storage::Chunked(Chunked::from_dense(bounds, cells, background))
        }
    }

    /// Placeholder with one cell, for moving a storage out temporarily.
    pub fn empty() -> Self {
        Storage::Dense(Dense::from_dense(BoundingBox::new((0, 0, 0), (0, 0, 0)), vec![0], 0))
    }

    pub fn bounds(&self) -> &BoundingBox {
        each!(self, s => s.bounds())
    }

    /// The value at a cell inside the bounds.
    #[inline(always)]
    pub fn get(&self, x: i32, y: i32, z: i32) -> u32 {
        match self {
            Storage::Dense(s) => s.get(x, y, z),
            Storage::Chunked(s) => chunked_get(s, x, y, z),
        }
    }

    /// Write a cell inside the bounds, returning its previous value.
    #[inline(always)]
    pub fn set(&mut self, x: i32, y: i32, z: i32, value: u32) -> u32 {
        match self {
            Storage::Dense(s) => s.set(x, y, z, value),
            Storage::Chunked(s) => chunked_set(s, x, y, z, value),
        }
    }

    /// The cells `x0..=x1` of row `(y, z)` as `(first x, slice)` segments, in
    /// x order. The row must lie inside the bounds.
    #[inline]
    pub fn row(&self, y: i32, z: i32, x0: i32, x1: i32) -> Row<'_> {
        match self {
            Storage::Dense(s) => Row::Dense(s.row(y, z, x0, x1)),
            Storage::Chunked(s) => Row::Chunked(s.row(y, z, x0, x1)),
        }
    }

    /// Mutable counterpart of [`Self::row`], visiting each segment.
    #[inline]
    pub fn row_mut(&mut self, y: i32, z: i32, x0: i32, x1: i32, f: impl FnMut(i32, &mut [u32])) {
        each!(self, s => s.row_mut(y, z, x0, x1, f))
    }

    /// Reverse the cells `x0..=x1` of row `(y, z)`.
    pub fn reverse_row(&mut self, y: i32, z: i32, x0: i32, x1: i32) {
        each!(self, s => s.reverse_row(y, z, x0, x1))
    }

    /// Swap the full-width rows `(ya, za)` and `(yb, zb)`.
    pub fn swap_rows(&mut self, a: (i32, i32), b: (i32, i32)) {
        each!(self, s => s.swap_rows(a, b))
    }

    /// Swap the full layers `ya` and `yb`.
    pub fn swap_layers(&mut self, ya: i32, yb: i32) {
        each!(self, s => s.swap_layers(ya, yb))
    }

    /// Set every cell to `value`.
    pub fn fill(&mut self, value: u32) {
        each!(self, s => s.fill(value))
    }

    /// Apply `f` to every stored value, and to the background.
    pub fn map_values(&mut self, f: impl FnMut(u32) -> u32) {
        each!(self, s => s.map_values(f))
    }

    /// The cells of `area` (inside the bounds), x fastest, then z, then y.
    pub fn dense(&self, area: &BoundingBox) -> Cow<'_, [u32]> {
        each!(self, s => s.dense(area))
    }

    /// Grow to `new_bounds`, which must contain the current bounds; new
    /// cells hold the background. A dense box that outgrows
    /// [`DENSE_MAX_CELLS`] becomes chunked.
    #[inline(never)]
    pub fn grow(&mut self, new_bounds: BoundingBox) {
        if let Storage::Dense(dense) = self {
            if new_bounds.volume() <= DENSE_MAX_CELLS {
                dense.grow(new_bounds);
                return;
            }
            let (bounds, cells, background) =
                std::mem::replace(dense, Dense::from_dense(BoundingBox::new((0, 0, 0), (0, 0, 0)), vec![0], 0))
                    .into_parts();
            *self = Storage::Chunked(Chunked::from_dense(bounds, cells, background));
        }
        if let Storage::Chunked(chunked) = self {
            chunked.grow(new_bounds);
        }
    }

    /// Copy the cells of `area` (inside both boxes) from `src`.
    pub fn copy_box_from(&mut self, src: &Storage, area: &BoundingBox) {
        match (&mut *self, src) {
            (Storage::Dense(dst), Storage::Dense(src)) => dst.copy_box_from(src, area),
            (Storage::Chunked(dst), Storage::Chunked(src)) => dst.copy_box_from(src, area),
            _ => {
                let mut row = Vec::with_capacity((area.max.0 - area.min.0 + 1) as usize);
                for y in area.min.1..=area.max.1 {
                    for z in area.min.2..=area.max.2 {
                        row.clear();
                        for (_, cells) in src.row(y, z, area.min.0, area.max.0) {
                            row.extend_from_slice(cells);
                        }
                        each!(&mut *self, s => s.write_row(y, z, area.min.0, &row));
                    }
                }
            }
        }
    }

    /// Move the box by an offset; cells keep their values.
    pub fn translate(&mut self, dx: i32, dy: i32, dz: i32) {
        each!(self, s => s.translate(dx, dy, dz))
    }

    /// Count of cells inside the bounds that are not `air`, and the box they
    /// span.
    pub fn content(&self, air: u32) -> (usize, Option<BoundingBox>) {
        each!(self, s => s.content(air))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dense_box_turns_chunked_when_it_outgrows_the_limit() {
        let mut storage = Storage::new(BoundingBox::new((0, 0, 0), (9, 9, 9)), 0).unwrap();
        storage.set(3, 4, 5, 7);
        assert!(matches!(storage, Storage::Dense(_)));
        storage.grow(BoundingBox::new((-300, 0, 0), (300, 300, 300)));
        assert!(matches!(storage, Storage::Chunked(_)));
        assert_eq!(storage.get(3, 4, 5), 7);
        assert_eq!(storage.content(0), (1, Some(BoundingBox::new((3, 4, 5), (3, 4, 5)))));
    }
}
