//! Greedy interval colouring: assign the minimum number of "tracks" to a set
//! of 1-D intervals so that overlapping intervals never share a track.
//! Classic channel/track assignment; optimal for interval graphs.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// A closed interval `[lo, hi]` with a caller id.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Interval {
    /// Caller-assigned id, echoed in the result.
    pub id: usize,
    /// Inclusive lower end.
    pub lo: i64,
    /// Inclusive upper end.
    pub hi: i64,
}

/// Colour intervals greedily by left endpoint. Returns `colors[i]` for
/// `items[i]`; the number of distinct colours equals the maximum overlap
/// depth (optimal). Deterministic: ties break on `(lo, id)`.
pub fn color_intervals(items: &[Interval]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by_key(|&i| (items[i].lo, items[i].id));

    let mut colors = vec![0usize; items.len()];
    // Active intervals: min-heap of (hi, color) — reuse the colour of the
    // earliest-ending interval that no longer overlaps.
    let mut active: BinaryHeap<Reverse<(i64, usize)>> = BinaryHeap::new();
    // Free colours (smallest first) recycled from expired intervals.
    let mut free: BinaryHeap<Reverse<usize>> = BinaryHeap::new();
    let mut next_color = 0usize;

    for &i in &order {
        let iv = items[i];
        while let Some(Reverse((hi, c))) = active.peek().copied() {
            if hi < iv.lo {
                active.pop();
                free.push(Reverse(c));
            } else {
                break;
            }
        }
        let c = match free.pop() {
            Some(Reverse(c)) => c,
            None => {
                let c = next_color;
                next_color += 1;
                c
            }
        };
        colors[i] = c;
        active.push(Reverse((iv.hi, c)));
    }
    colors
}

/// Number of colours a colouring uses.
pub fn color_count(colors: &[usize]) -> usize {
    colors.iter().max().map_or(0, |m| m + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iv(id: usize, lo: i64, hi: i64) -> Interval {
        Interval { id, lo, hi }
    }

    #[test]
    fn overlapping_get_distinct_colors() {
        let items = vec![iv(0, 0, 10), iv(1, 5, 15), iv(2, 12, 20)];
        let c = color_intervals(&items);
        assert_ne!(c[0], c[1]);
        assert_ne!(c[1], c[2]);
        // 0 and 2 do not overlap: greedy reuses the track.
        assert_eq!(c[0], c[2]);
        assert_eq!(color_count(&c), 2);
    }

    #[test]
    fn colour_count_equals_max_depth() {
        // Depth 3 at t=5.
        let items = vec![
            iv(0, 0, 9),
            iv(1, 1, 8),
            iv(2, 2, 7),
            iv(3, 10, 12),
            iv(4, 11, 13),
        ];
        let c = color_intervals(&items);
        assert_eq!(color_count(&c), 3);
        for i in 0..items.len() {
            for j in i + 1..items.len() {
                let (a, b) = (items[i], items[j]);
                if a.lo <= b.hi && b.lo <= a.hi {
                    assert_ne!(c[i], c[j], "overlapping {i} and {j} share a track");
                }
            }
        }
    }

    #[test]
    fn deterministic() {
        let items = vec![iv(0, 0, 4), iv(1, 0, 4), iv(2, 0, 4), iv(3, 5, 9)];
        assert_eq!(color_intervals(&items), color_intervals(&items));
    }
}
