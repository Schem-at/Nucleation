//! Dense layout: one contiguous buffer over the storage box, x fastest,
//! then z, then y.

use crate::bounding_box::BoundingBox;
use crate::region::try_zeroed_blocks;

#[derive(Debug, Clone)]
pub(crate) struct Dense {
    bounds: BoundingBox,
    background: u32,
    width: i64,
    length: i64,
    layer: i64,
    cells: Vec<u32>,
}

impl Dense {
    /// Storage over `bounds`, every cell `background`.
    pub(super) fn new(bounds: BoundingBox, background: u32) -> Result<Self, String> {
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
    pub(super) fn from_dense(bounds: BoundingBox, cells: Vec<u32>, background: u32) -> Self {
        let (w, _, l) = bounds.get_dimensions();
        let (width, length) = (w as i64, l as i64);
        Dense {
            bounds,
            background,
            width,
            length,
            // i64: w × l can exceed i32::MAX (e.g. 50k × 50k = 2.5e9).
            layer: width * length,
            cells,
        }
    }

    pub(super) fn bounds(&self) -> &BoundingBox {
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
    pub(super) fn get(&self, x: i32, y: i32, z: i32) -> u32 {
        self.cells[self.index(x, y, z)]
    }

    /// Write a cell inside the bounds, returning its previous value.
    #[inline(always)]
    pub(super) fn set(&mut self, x: i32, y: i32, z: i32, value: u32) -> u32 {
        let index = self.index(x, y, z);
        std::mem::replace(&mut self.cells[index], value)
    }

    /// The cells `x0..=x1` of row `(y, z)` as `(first x, slice)` segments, in
    /// x order. The row must lie inside the bounds.
    #[inline]
    pub(super) fn row(&self, y: i32, z: i32, x0: i32, x1: i32) -> std::iter::Once<(i32, &[u32])> {
        let start = self.index(x0, y, z);
        std::iter::once((x0, &self.cells[start..start + (x1 - x0 + 1) as usize]))
    }

    /// Mutable counterpart of [`Self::row`], visiting each segment.
    #[inline]
    pub(super) fn row_mut(&mut self, y: i32, z: i32, x0: i32, x1: i32, mut f: impl FnMut(i32, &mut [u32])) {
        let start = self.index(x0, y, z);
        f(x0, &mut self.cells[start..start + (x1 - x0 + 1) as usize]);
    }

    /// Flip all cells inside `bbox` along the X axis in place.
    pub(super) fn flip_x(&mut self, bbox: &BoundingBox) {
        let w = self.width as usize;
        if *bbox == self.bounds {
            if w > 1 {
                for row in self.cells.chunks_exact_mut(w) {
                    row.reverse();
                }
            }
            return;
        }
        let (min, max) = (bbox.min, bbox.max);
        let row_len = (max.0 - min.0 + 1) as usize;
        if row_len <= 1 {
            return;
        }
        let layer = self.layer as usize;
        let x_off = (min.0 - self.bounds.min.0) as usize;
        let z0 = (min.2 - self.bounds.min.2) as usize;
        let z1 = (max.2 - self.bounds.min.2) as usize;
        let y0 = (min.1 - self.bounds.min.1) as usize;
        let y1 = (max.1 - self.bounds.min.1) as usize;
        for dy in y0..=y1 {
            let layer_base = dy * layer;
            for dz in z0..=z1 {
                let start = layer_base + dz * w + x_off;
                self.cells[start..start + row_len].reverse();
            }
        }
    }

    /// Flip all cells inside `bbox` along the Y axis in place.
    pub(super) fn flip_y(&mut self, bbox: &BoundingBox) {
        let l = self.layer as usize;
        let y0 = (bbox.min.1 - self.bounds.min.1) as usize;
        let y1 = (bbox.max.1 - self.bounds.min.1) as usize;
        let half = (y1 - y0 + 1) / 2;
        for k in 0..half {
            let a = (y0 + k) * l;
            let b = (y1 - k) * l;
            let (low, high) = self.cells.split_at_mut(b);
            low[a..a + l].swap_with_slice(&mut high[..l]);
        }
    }

    /// Flip all cells inside `bbox` along the Z axis in place.
    pub(super) fn flip_z(&mut self, bbox: &BoundingBox) {
        let w = self.width as usize;
        let layer = self.layer as usize;
        let z0 = (bbox.min.2 - self.bounds.min.2) as usize;
        let z1 = (bbox.max.2 - self.bounds.min.2) as usize;
        let half = (z1 - z0 + 1) / 2;
        if half == 0 {
            return;
        }
        let y0 = (bbox.min.1 - self.bounds.min.1) as usize;
        let y1 = (bbox.max.1 - self.bounds.min.1) as usize;
        for dy in y0..=y1 {
            let layer_slice = &mut self.cells[dy * layer..(dy + 1) * layer];
            for k in 0..half {
                let a = (z0 + k) * w;
                let b = (z1 - k) * w;
                let (low, high) = layer_slice.split_at_mut(b);
                low[a..a + w].swap_with_slice(&mut high[..w]);
            }
        }
    }

    /// Set every cell to `value`.
    pub(super) fn fill(&mut self, value: u32) {
        self.cells.fill(value);
    }

    /// Apply `f` to every stored value, and to the background.
    pub(super) fn map_values(&mut self, mut f: impl FnMut(u32) -> u32) {
        self.background = f(self.background);
        for cell in &mut self.cells {
            *cell = f(*cell);
        }
    }

    /// The cells of `area` (inside the bounds), x fastest, then z, then y.
    pub(super) fn dense(&self, area: &BoundingBox) -> std::borrow::Cow<'_, [u32]> {
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
    pub(super) fn grow(&mut self, new_bounds: BoundingBox) {
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
    pub(super) fn copy_box_from(&mut self, src: &Dense, area: &BoundingBox) {
        if self.bounds == *area && src.bounds == *area {
            self.cells.copy_from_slice(&src.cells);
            return;
        }
        let (min, max) = (area.min, area.max);
        let row_len = (max.0 - min.0 + 1) as usize;
        let dst_w = self.width as usize;
        let src_w = src.width as usize;
        let full_x = row_len == dst_w
            && row_len == src_w
            && min.0 == self.bounds.min.0
            && min.0 == src.bounds.min.0;
        let z_rows = (max.2 - min.2 + 1) as usize;

        if full_x
            && z_rows == self.length as usize
            && z_rows == src.length as usize
            && min.2 == self.bounds.min.2
            && min.2 == src.bounds.min.2
        {
            let total = (max.1 - min.1 + 1) as usize * self.layer as usize;
            let dst_start = self.index(min.0, min.1, min.2);
            let src_start = src.index(min.0, min.1, min.2);
            self.cells[dst_start..dst_start + total]
                .copy_from_slice(&src.cells[src_start..src_start + total]);
            return;
        }

        // Only pay Rayon thread-pool wake/sync overhead for large copies (≥ 1 MB).
        #[cfg(not(target_arch = "wasm32"))]
        if area.volume() >= 262_144 {
            use rayon::prelude::*;
            let layer = self.layer as usize;
            let dst_min_y = self.bounds.min.1;
            let dst_min = self.bounds.min;
            let block_len = z_rows * row_len;
            self.cells
                .par_chunks_mut(layer)
                .enumerate()
                .for_each(|(dy, dst_layer)| {
                    let y = dst_min_y + dy as i32;
                    if y < min.1 || y > max.1 {
                        return;
                    }
                    if full_x {
                        let start = (min.2 - dst_min.2) as usize * dst_w;
                        let src_start = src.index(min.0, y, min.2);
                        dst_layer[start..start + block_len]
                            .copy_from_slice(&src.cells[src_start..src_start + block_len]);
                    } else {
                        let mut start =
                            (min.2 - dst_min.2) as usize * dst_w + (min.0 - dst_min.0) as usize;
                        let mut src_start = src.index(min.0, y, min.2);
                        for _ in 0..z_rows {
                            dst_layer[start..start + row_len]
                                .copy_from_slice(&src.cells[src_start..src_start + row_len]);
                            start += dst_w;
                            src_start += src_w;
                        }
                    }
                });
            return;
        }

        let dst_layer = self.layer as usize;
        let src_layer = src.layer as usize;
        let mut dst_y = self.index(min.0, min.1, min.2);
        let mut src_y = src.index(min.0, min.1, min.2);
        let y_layers = (max.1 - min.1 + 1) as usize;
        if full_x {
            let block_len = z_rows * row_len;
            for _ in 0..y_layers {
                self.cells[dst_y..dst_y + block_len]
                    .copy_from_slice(&src.cells[src_y..src_y + block_len]);
                dst_y += dst_layer;
                src_y += src_layer;
            }
        } else {
            for _ in 0..y_layers {
                let mut dst = dst_y;
                let mut from = src_y;
                for _ in 0..z_rows {
                    self.cells[dst..dst + row_len]
                        .copy_from_slice(&src.cells[from..from + row_len]);
                    dst += dst_w;
                    from += src_w;
                }
                dst_y += dst_layer;
                src_y += src_layer;
            }
        }
    }

    /// Write `values` into row `(y, z)` from `x0`.
    pub(super) fn write_row(&mut self, y: i32, z: i32, x0: i32, values: &[u32]) {
        let start = self.index(x0, y, z);
        self.cells[start..start + values.len()].copy_from_slice(values);
    }

    pub(super) fn into_parts(self) -> (BoundingBox, Vec<u32>, u32) {
        (self.bounds, self.cells, self.background)
    }

    /// Move the box by an offset; cells keep their values.
    pub(super) fn translate(&mut self, dx: i32, dy: i32, dz: i32) {
        let (min, max) = (self.bounds.min, self.bounds.max);
        self.bounds = BoundingBox::new(
            (min.0 + dx, min.1 + dy, min.2 + dz),
            (max.0 + dx, max.1 + dy, max.2 + dz),
        );
    }

    /// Count of cells that are not `air`, and the box they span.
    ///
    /// Scans layers and x-rows as slices with no integer divisions and a
    /// branchless u32 count loop that vectorizes cleanly; once `lo_x` or
    /// `hi_x` reaches the edge of the box, endpoint scans are skipped.
    pub(super) fn content(&self, air: u32) -> (usize, Option<BoundingBox>) {
        let w = self.width as usize;
        let l = self.length as usize;
        let layer = self.layer as usize;
        if w == 0 || l == 0 || self.cells.is_empty() {
            return (0, None);
        }
        let mut count = 0usize;
        let (mut lo_x, mut lo_y, mut lo_z) = (w, usize::MAX, l);
        let (mut hi_x, mut hi_y, mut hi_z) = (0usize, 0usize, 0usize);
        for (dy, layer_cells) in self.cells.chunks_exact(layer).enumerate() {
            // Once X and Z bounds have saturated the full box, subsequent layers
            // only need a single flat vectorized count pass to update `count` and `hi_y`.
            if lo_x == 0 && hi_x + 1 == w && lo_z == 0 && hi_z + 1 == l {
                let mut layer_count = 0u32;
                for &b in layer_cells {
                    layer_count += u32::from(b != air);
                }
                if layer_count != 0 {
                    count += layer_count as usize;
                    hi_y = dy;
                }
                continue;
            }
            let layer_start_count = count;
            for (dz, row) in layer_cells.chunks_exact(w).enumerate() {
                let mut row_count = 0u32;
                for &b in row {
                    row_count += u32::from(b != air);
                }
                if row_count == 0 {
                    continue;
                }
                count += row_count as usize;
                if dz < lo_z {
                    lo_z = dz;
                }
                if dz > hi_z {
                    hi_z = dz;
                }
                if lo_x > 0 {
                    if let Some(first) = row[..lo_x].iter().position(|&b| b != air) {
                        lo_x = first;
                    }
                }
                if hi_x + 1 < w {
                    if let Some(offset) = row[hi_x + 1..].iter().rposition(|&b| b != air) {
                        hi_x = hi_x + 1 + offset;
                    }
                }
            }
            if count != layer_start_count {
                if lo_y == usize::MAX {
                    lo_y = dy;
                }
                hi_y = dy;
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
        let storage = Dense::from_dense(bounds, Vec::new(), 0);
        let (width, length) = (82448i64, 18944i64);
        let layer = width * length;
        assert_eq!(storage.index(-47472, 236, -5216), (300 * layer) as usize);
        assert_eq!(
            storage.index(34975, 319, 13727),
            ((width - 1) + (length - 1) * width + 383 * layer) as usize
        );
    }
}
