//! A region's voxel storage: one palette index per cell of a storage box.
//!
//! Cells are addressed by world coordinates inside [`Storage::bounds`]; rows
//! run along x and are handed out as segments, so callers never assume one
//! contiguous buffer. Cells that were never written hold the background value
//! (the region's air index).

use crate::bounding_box::BoundingBox;
use crate::region::try_zeroed_blocks;

#[derive(Debug, Clone)]
pub(crate) struct Storage {
    bounds: BoundingBox,
    background: u32,
    width: i64,
    length: i64,
    layer: i64,
    cells: Vec<u32>,
}

impl Storage {
    /// Storage over `bounds`, every cell `background`.
    pub fn new(bounds: BoundingBox, background: u32) -> Result<Self, String> {
        let volume = usize::try_from(bounds.volume())
            .map_err(|_| "Region volume exceeds addressable memory".to_string())?;
        let mut cells = try_zeroed_blocks(volume)?;
        if background != 0 {
            cells.fill(background);
        }
        Ok(Self::from_dense(bounds, cells, background))
    }

    /// Storage adopting `cells`, laid out over `bounds` x fastest, then z,
    /// then y. `cells.len()` must be the volume of `bounds`.
    pub fn from_dense(bounds: BoundingBox, cells: Vec<u32>, background: u32) -> Self {
        let (w, _, l) = bounds.get_dimensions();
        let (width, length) = (w as i64, l as i64);
        Storage {
            bounds,
            background,
            width,
            length,
            // i64: w × l can exceed i32::MAX (e.g. 50k × 50k = 2.5e9).
            layer: width * length,
            cells,
        }
    }

    pub fn bounds(&self) -> &BoundingBox {
        &self.bounds
    }

    #[inline(always)]
    fn index(&self, x: i32, y: i32, z: i32) -> usize {
        // i64 before the multiplications: for a 600M-cell box the y term
        // alone overflows i32.
        let dx = (x - self.bounds.min.0) as i64;
        let dy = (y - self.bounds.min.1) as i64;
        let dz = (z - self.bounds.min.2) as i64;
        (dx + dz * self.width + dy * self.layer) as usize
    }

    /// The value at a cell inside the bounds.
    #[inline(always)]
    pub fn get(&self, x: i32, y: i32, z: i32) -> u32 {
        self.cells[self.index(x, y, z)]
    }

    /// Write a cell inside the bounds, returning its previous value.
    #[inline(always)]
    pub fn set(&mut self, x: i32, y: i32, z: i32, value: u32) -> u32 {
        let index = self.index(x, y, z);
        std::mem::replace(&mut self.cells[index], value)
    }

    /// The cells `x0..=x1` of row `(y, z)` as `(first x, slice)` segments, in
    /// x order. The row must lie inside the bounds.
    #[inline]
    pub fn row(&self, y: i32, z: i32, x0: i32, x1: i32) -> impl Iterator<Item = (i32, &[u32])> {
        let start = self.index(x0, y, z);
        std::iter::once((x0, &self.cells[start..start + (x1 - x0 + 1) as usize]))
    }

    /// Mutable counterpart of [`Self::row`], visiting each segment.
    #[inline]
    pub fn row_mut(&mut self, y: i32, z: i32, x0: i32, x1: i32, mut f: impl FnMut(i32, &mut [u32])) {
        let start = self.index(x0, y, z);
        f(x0, &mut self.cells[start..start + (x1 - x0 + 1) as usize]);
    }

    /// Reverse the cells `x0..=x1` of row `(y, z)`.
    pub fn reverse_row(&mut self, y: i32, z: i32, x0: i32, x1: i32) {
        self.row_mut(y, z, x0, x1, |_, cells| cells.reverse());
    }

    /// Swap the full-width rows `(ya, za)` and `(yb, zb)`.
    pub fn swap_rows(&mut self, (ya, za): (i32, i32), (yb, zb): (i32, i32)) {
        let w = self.width as usize;
        let (a, b) = (self.index(self.bounds.min.0, ya, za), self.index(self.bounds.min.0, yb, zb));
        if a == b {
            return;
        }
        let (lo, hi) = (a.min(b), a.max(b));
        let (low, high) = self.cells.split_at_mut(hi);
        low[lo..lo + w].swap_with_slice(&mut high[..w]);
    }

    /// Swap the full layers `ya` and `yb`.
    pub fn swap_layers(&mut self, ya: i32, yb: i32) {
        let l = self.layer as usize;
        let (a, b) = (
            (ya - self.bounds.min.1) as usize * l,
            (yb - self.bounds.min.1) as usize * l,
        );
        if a == b {
            return;
        }
        let (lo, hi) = (a.min(b), a.max(b));
        let (low, high) = self.cells.split_at_mut(hi);
        low[lo..lo + l].swap_with_slice(&mut high[..l]);
    }

    /// Set every cell to `value`.
    pub fn fill(&mut self, value: u32) {
        self.cells.fill(value);
    }

    /// Apply `f` to every stored value, and to the background.
    pub fn map_values(&mut self, mut f: impl FnMut(u32) -> u32) {
        self.background = f(self.background);
        for cell in &mut self.cells {
            *cell = f(*cell);
        }
    }

    /// The cells of `area` (inside the bounds), x fastest, then z, then y.
    pub fn dense(&self, area: &BoundingBox) -> std::borrow::Cow<'_, [u32]> {
        if *area == self.bounds {
            return std::borrow::Cow::Borrowed(&self.cells);
        }
        let mut out = Vec::with_capacity(area.volume() as usize);
        for y in area.min.1..=area.max.1 {
            for z in area.min.2..=area.max.2 {
                for (_, cells) in self.row(y, z, area.min.0, area.max.0) {
                    out.extend_from_slice(cells);
                }
            }
        }
        std::borrow::Cow::Owned(out)
    }

    /// Grow to `new_bounds`, which must contain the current bounds; new
    /// cells hold the background.
    ///
    /// Works in place: the vector is extended (a realloc, which for large
    /// buffers remaps the existing pages instead of copying them), the old
    /// rows are moved back-to-front to their new offsets, and only the cells
    /// that were not part of the old box are reset. Compared with allocating
    /// a second full buffer, this halves peak memory during growth and avoids
    /// faulting in a fresh copy of every resident page.
    pub fn grow(&mut self, new_bounds: BoundingBox) {
        if new_bounds == self.bounds {
            return;
        }
        debug_assert!(new_bounds.contains(self.bounds.min) && new_bounds.contains(self.bounds.max));
        let fill = self.background;
        let (old_w, old_h, old_l) = self.bounds.get_dimensions();
        let (old_w, old_h, old_l) = (old_w as usize, old_h as usize, old_l as usize);
        let (new_w, new_h, new_l) = new_bounds.get_dimensions();
        let (new_w, new_h, new_l) = (new_w as usize, new_h as usize, new_l as usize);
        let x_off = (self.bounds.min.0 - new_bounds.min.0) as usize;
        let y_off = (self.bounds.min.1 - new_bounds.min.1) as usize;
        let z_off = (self.bounds.min.2 - new_bounds.min.2) as usize;

        let old_len = self.cells.len();
        self.cells.resize(new_bounds.volume() as usize, fill);

        // Growing only towards +y appends whole layers: the old cells are
        // already in place.
        let in_place = x_off == 0 && y_off == 0 && z_off == 0 && new_w == old_w && new_l == old_l;
        if !in_place {
            // Every old row moves to an equal or higher index, and rows keep
            // their order, so moving the last row first never overwrites a
            // row that has not been moved yet.
            for dy in (0..old_h).rev() {
                for dz in (0..old_l).rev() {
                    let src = dy * old_w * old_l + dz * old_w;
                    let dst = (dy + y_off) * new_w * new_l + (dz + z_off) * new_w + x_off;
                    if dst != src {
                        self.cells.copy_within(src..src + old_w, dst);
                    }
                }
            }
            // Every cell outside the moved rows takes the background. Only
            // cells below the old length can hold a stale value: `resize`
            // already wrote the background past it.
            'rows: for dy in 0..new_h {
                for dz in 0..new_l {
                    let row_start = (dy * new_l + dz) * new_w;
                    if row_start >= old_len {
                        break 'rows;
                    }
                    let row = &mut self.cells[row_start..][..new_w.min(old_len - row_start)];
                    let holds_old_row =
                        (y_off..y_off + old_h).contains(&dy) && (z_off..z_off + old_l).contains(&dz);
                    if holds_old_row {
                        let end = row.len();
                        row[..x_off.min(end)].fill(fill);
                        row[(x_off + old_w).min(end)..].fill(fill);
                    } else {
                        row.fill(fill);
                    }
                }
            }
        }
        *self = Self::from_dense(new_bounds, std::mem::take(&mut self.cells), fill);
    }

    /// Copy the cells of `area` (inside both boxes) from `src`.
    pub fn copy_box_from(&mut self, src: &Storage, area: &BoundingBox) {
        let (min, max) = (area.min, area.max);
        let row_len = (max.0 - min.0 + 1) as usize;
        #[cfg(not(target_arch = "wasm32"))]
        {
            use rayon::prelude::*;
            // Destination layers are disjoint slices, so rows copy in parallel.
            let layer = self.layer as usize;
            let dst_min_y = self.bounds.min.1;
            let (width, dst_min) = (self.width as usize, self.bounds.min);
            self.cells
                .par_chunks_mut(layer)
                .enumerate()
                .for_each(|(dy, dst_layer)| {
                    let y = dst_min_y + dy as i32;
                    if y < min.1 || y > max.1 {
                        return;
                    }
                    for z in min.2..=max.2 {
                        let start = (z - dst_min.2) as usize * width + (min.0 - dst_min.0) as usize;
                        let src_start = src.index(min.0, y, z);
                        dst_layer[start..start + row_len]
                            .copy_from_slice(&src.cells[src_start..src_start + row_len]);
                    }
                });
        }
        #[cfg(target_arch = "wasm32")]
        for y in min.1..=max.1 {
            for z in min.2..=max.2 {
                let (dst, from) = (self.index(min.0, y, z), src.index(min.0, y, z));
                self.cells[dst..dst + row_len].copy_from_slice(&src.cells[from..from + row_len]);
            }
        }
    }

    /// Placeholder with no cells, for moving a storage out temporarily.
    pub fn empty() -> Self {
        Self::from_dense(BoundingBox::new((0, 0, 0), (0, 0, 0)), vec![0], 0)
    }

    /// Move the box by an offset; cells keep their values.
    pub fn translate(&mut self, dx: i32, dy: i32, dz: i32) {
        let (min, max) = (self.bounds.min, self.bounds.max);
        self.bounds = BoundingBox::new(
            (min.0 + dx, min.1 + dy, min.2 + dz),
            (max.0 + dx, max.1 + dy, max.2 + dz),
        );
    }

    /// Count of cells that are not `air`, and the box they span.
    ///
    /// Scans x-rows as slices, so there is no per-voxel coordinate division
    /// and the comparisons vectorize; each occupied row touches the bounds
    /// once.
    pub fn content(&self, air: u32) -> (usize, Option<BoundingBox>) {
        let w = self.width as usize;
        let l = self.length as usize;
        let mut count = 0usize;
        let (mut lo_x, mut lo_y, mut lo_z) = (usize::MAX, usize::MAX, usize::MAX);
        let (mut hi_x, mut hi_y, mut hi_z) = (0usize, 0usize, 0usize);
        if w > 0 && l > 0 {
            for (row_index, row) in self.cells.chunks_exact(w).enumerate() {
                let row_count = row.iter().filter(|&&b| b != air).count();
                if row_count == 0 {
                    continue;
                }
                count += row_count;
                let first = row.iter().position(|&b| b != air).unwrap_or(0);
                let last = row.iter().rposition(|&b| b != air).unwrap_or(0);
                let (dy, dz) = (row_index / l, row_index % l);
                lo_x = lo_x.min(first);
                hi_x = hi_x.max(last);
                lo_y = lo_y.min(dy);
                hi_y = hi_y.max(dy);
                lo_z = lo_z.min(dz);
                hi_z = hi_z.max(dz);
            }
        }
        let (bx, by, bz) = self.bounds.min;
        let bounds = (count > 0).then(|| {
            BoundingBox::new(
                (bx + lo_x as i32, by + lo_y as i32, bz + lo_z as i32),
                (bx + hi_x as i32, by + hi_y as i32, bz + hi_z as i32),
            )
        });
        (count, bounds)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: with i32 arithmetic, an index inside a region whose
    /// footprint exceeded ~50k × 19k blocks wrapped to a negative value and
    /// the cell access panicked out of bounds.
    #[test]
    fn index_does_not_overflow_for_huge_boxes() {
        // The build-extractor mega-terrain repro. No cells are allocated:
        // only the index arithmetic is under test.
        let bounds = BoundingBox::new((-47472, -64, -5216), (34975, 319, 13727));
        let storage = Storage::from_dense(bounds, Vec::new(), 0);
        let (width, length) = (82448i64, 18944i64);
        let layer = width * length;
        assert_eq!(storage.index(-47472, 236, -5216), (300 * layer) as usize);
        assert_eq!(
            storage.index(34975, 319, 13727),
            ((width - 1) + (length - 1) * width + 383 * layer) as usize
        );
    }
}
