//! Sparse-dense block-coordinate set used as the flood's visited marker.
//!
//! Ported from RedstoneTools' `That.kt#BlockSet`: a `HashMap` keyed by chunk
//! coordinate, mapping to a fixed 16×16×16 bitset (4096 bits = 64 u64).
//! Sparse where the world is sparse, dense where the flood actually walks.

use std::collections::HashMap;

const CHUNK_BITS: u32 = 4;
const CHUNK_SIDE: i32 = 1 << CHUNK_BITS; // 16
const CHUNK_MASK: i32 = CHUNK_SIDE - 1; // 15
const CHUNK_VOLUME: usize = 1 << (CHUNK_BITS * 3); // 4096
const WORDS_PER_CHUNK: usize = CHUNK_VOLUME / 64; // 64

type ChunkKey = (i32, i32, i32);

/// A sparse 3D bitset over `i32` block coordinates.
///
/// Memory: one 512-byte chunk per touched 16×16×16 area plus the hashmap entry.
/// A flood that visits N blocks touches at most ⌈N/4096⌉ + boundary chunks.
#[derive(Default)]
pub struct VisitedSet {
    chunks: HashMap<ChunkKey, [u64; WORDS_PER_CHUNK]>,
}

impl VisitedSet {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    /// Approximate live memory footprint, in bytes. Useful for diagnostics.
    pub fn approx_bytes(&self) -> usize {
        // each entry: key (12B) + bitset (512B); ignore hashmap bookkeeping
        self.chunks.len() * (std::mem::size_of::<ChunkKey>() + WORDS_PER_CHUNK * 8)
    }

    /// Number of chunks touched. For diagnostics / tuning.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// True if `(x, y, z)` has been inserted.
    #[inline]
    pub fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        let key = chunk_key(x, y, z);
        match self.chunks.get(&key) {
            Some(bits) => {
                let (word, bit) = index_in_chunk(x, y, z);
                (bits[word] >> bit) & 1 != 0
            }
            None => false,
        }
    }

    /// Insert `(x, y, z)`. Returns `true` if the position was not already set.
    #[inline]
    pub fn insert(&mut self, x: i32, y: i32, z: i32) -> bool {
        let key = chunk_key(x, y, z);
        let chunk = self
            .chunks
            .entry(key)
            .or_insert_with(|| [0u64; WORDS_PER_CHUNK]);
        let (word, bit) = index_in_chunk(x, y, z);
        let mask = 1u64 << bit;
        let was = chunk[word] & mask != 0;
        chunk[word] |= mask;
        !was
    }
}

#[inline]
fn chunk_key(x: i32, y: i32, z: i32) -> ChunkKey {
    // arithmetic shift gives correct floor-div behaviour for negatives in
    // two's complement, which is what we want for chunk coordinates.
    (x >> CHUNK_BITS, y >> CHUNK_BITS, z >> CHUNK_BITS)
}

#[inline]
fn index_in_chunk(x: i32, y: i32, z: i32) -> (usize, u32) {
    let lx = (x & CHUNK_MASK) as u32;
    let ly = (y & CHUNK_MASK) as u32;
    let lz = (z & CHUNK_MASK) as u32;
    // 12-bit linear index inside the 16³ chunk: y high, z mid, x low.
    // Layout doesn't matter for correctness as long as it's a bijection;
    // chosen so that x-major scans stay in the same 64-bit word.
    let linear = (ly << 8) | (lz << 4) | lx;
    let word = (linear >> 6) as usize; // /64
    let bit = linear & 63;
    (word, bit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_returns_true_only_first_time() {
        let mut s = VisitedSet::new();
        assert!(s.insert(0, 0, 0));
        assert!(!s.insert(0, 0, 0));
        assert!(s.insert(1, 0, 0));
    }

    #[test]
    fn contains_matches_insert() {
        let mut s = VisitedSet::new();
        for &p in &[(0, 0, 0), (5, 6, 7), (-1, -2, -3), (1023, 0, -512)] {
            assert!(!s.contains(p.0, p.1, p.2));
            s.insert(p.0, p.1, p.2);
            assert!(s.contains(p.0, p.1, p.2));
        }
        // a neighbour should still be unset
        assert!(!s.contains(1, 0, 0));
    }

    #[test]
    fn negative_coordinates_share_correct_chunk() {
        // (-1, -1, -1) lives in chunk (-1, -1, -1), local (15, 15, 15)
        let mut s = VisitedSet::new();
        s.insert(-1, -1, -1);
        assert!(s.contains(-1, -1, -1));
        // and (-16, -16, -16) is the corner of chunk (-1, -1, -1) too
        s.insert(-16, -16, -16);
        assert!(s.contains(-16, -16, -16));
        // both are in the same chunk
        assert_eq!(s.chunk_count(), 1);
        // (-17, ...) crosses the boundary
        s.insert(-17, -1, -1);
        assert_eq!(s.chunk_count(), 2);
    }

    #[test]
    fn distinct_chunks_dont_alias() {
        // Two positions with the same `lx,ly,lz` but in different chunks must
        // not collide.
        let mut s = VisitedSet::new();
        s.insert(0, 0, 0);
        assert!(!s.contains(16, 0, 0));
        assert!(!s.contains(0, 16, 0));
        assert!(!s.contains(0, 0, 16));
    }

    #[test]
    fn fills_a_chunk_completely() {
        let mut s = VisitedSet::new();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    assert!(s.insert(x, y, z));
                }
            }
        }
        assert_eq!(s.chunk_count(), 1);
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    assert!(s.contains(x, y, z));
                }
            }
        }
        // re-insert is a no-op
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    assert!(!s.insert(x, y, z));
                }
            }
        }
    }
}
