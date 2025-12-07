//! Sort strategies for ordering positions in IO layouts
//!
//! When defining circuit inputs/outputs with multiple positions, the order of
//! positions determines the bit assignment (LSB to MSB). This module provides
//! strategies for controlling that ordering.

/// Sort strategy for ordering positions in IO layouts
///
/// The sort strategy controls how positions are ordered when assigned to bits.
/// Position 0 corresponds to bit 0 (LSB), position 1 to bit 1, etc.
#[derive(Debug, Clone, PartialEq)]
pub enum SortStrategy {
    // ========================================================================
    // Axis-first sorting (ascending)
    // ========================================================================
    /// Sort by Y first (ascending), then X, then Z
    /// Standard Minecraft layer-based ordering.
    YXZ,

    /// Sort by X first (ascending), then Y, then Z
    XYZ,

    /// Sort by Z first (ascending), then Y, then X
    ZYX,

    // ========================================================================
    // Axis-first sorting (descending)
    // ========================================================================
    /// Sort by Y first (descending), then X ascending, then Z ascending
    YDescXZ,

    /// Sort by X first (descending), then Y ascending, then Z ascending
    XDescYZ,

    /// Sort by Z first (descending), then Y ascending, then X ascending
    ZDescYX,

    // ========================================================================
    // Fully descending
    // ========================================================================
    /// Sort by Y descending, then X descending, then Z descending
    YXZDesc,

    // ========================================================================
    // Distance-based sorting
    // ========================================================================
    /// Sort by Euclidean distance from a reference point (ascending)
    /// Closest positions first. Useful for radial layouts.
    DistanceFrom {
        /// Reference point for distance calculation
        reference: (i32, i32, i32),
    },

    /// Sort by Euclidean distance from a reference point (descending)
    /// Farthest positions first.
    DistanceFromDesc {
        /// Reference point for distance calculation
        reference: (i32, i32, i32),
    },

    // ========================================================================
    // Special strategies
    // ========================================================================
    /// Preserve the order positions were added (no sorting)
    /// Useful when you've manually ordered positions or are using `from_bounding_boxes`
    /// where box order matters.
    Preserve,

    /// Reverse of whatever order positions were added
    Reverse,

    /// Custom sort order defined by a list of axes and directions
    /// Example: [ (Y, Descending), (X, Ascending), (Z, Ascending) ]
    Custom(Vec<(Axis, Direction)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Ascending,
    Descending,
}

impl Default for SortStrategy {
    fn default() -> Self {
        SortStrategy::YXZ
    }
}

impl SortStrategy {
    /// Sort positions according to this strategy
    ///
    /// Returns a new Vec with positions in the sorted order.
    pub fn sort(&self, positions: &[(i32, i32, i32)]) -> Vec<(i32, i32, i32)> {
        let mut sorted = positions.to_vec();
        self.sort_in_place(&mut sorted);
        sorted
    }

    /// Sort positions in place according to this strategy
    pub fn sort_in_place(&self, positions: &mut [(i32, i32, i32)]) {
        match self {
            SortStrategy::YXZ => {
                positions.sort_by(|a, b| {
                    a.1.cmp(&b.1)
                        .then_with(|| a.0.cmp(&b.0))
                        .then_with(|| a.2.cmp(&b.2))
                });
            }
            SortStrategy::Custom(orders) => {
                positions.sort_by(|a, b| {
                    for (axis, direction) in orders {
                        let val_a = match axis {
                            Axis::X => a.0,
                            Axis::Y => a.1,
                            Axis::Z => a.2,
                        };
                        let val_b = match axis {
                            Axis::X => b.0,
                            Axis::Y => b.1,
                            Axis::Z => b.2,
                        };

                        let cmp = match direction {
                            Direction::Ascending => val_a.cmp(&val_b),
                            Direction::Descending => val_b.cmp(&val_a),
                        };

                        if cmp != std::cmp::Ordering::Equal {
                            return cmp;
                        }
                    }
                    std::cmp::Ordering::Equal
                });
            }
            SortStrategy::XYZ => {
                positions.sort_by(|a, b| {
                    a.0.cmp(&b.0)
                        .then_with(|| a.1.cmp(&b.1))
                        .then_with(|| a.2.cmp(&b.2))
                });
            }
            SortStrategy::ZYX => {
                positions.sort_by(|a, b| {
                    a.2.cmp(&b.2)
                        .then_with(|| a.1.cmp(&b.1))
                        .then_with(|| a.0.cmp(&b.0))
                });
            }
            SortStrategy::YDescXZ => {
                positions.sort_by(|a, b| {
                    b.1.cmp(&a.1) // Y descending
                        .then_with(|| a.0.cmp(&b.0))
                        .then_with(|| a.2.cmp(&b.2))
                });
            }
            SortStrategy::XDescYZ => {
                positions.sort_by(|a, b| {
                    b.0.cmp(&a.0) // X descending
                        .then_with(|| a.1.cmp(&b.1))
                        .then_with(|| a.2.cmp(&b.2))
                });
            }
            SortStrategy::ZDescYX => {
                positions.sort_by(|a, b| {
                    b.2.cmp(&a.2) // Z descending
                        .then_with(|| a.1.cmp(&b.1))
                        .then_with(|| a.0.cmp(&b.0))
                });
            }
            SortStrategy::YXZDesc => {
                positions.sort_by(|a, b| {
                    b.1.cmp(&a.1)
                        .then_with(|| b.0.cmp(&a.0))
                        .then_with(|| b.2.cmp(&a.2))
                });
            }
            SortStrategy::DistanceFrom { reference } => {
                let (rx, ry, rz) = *reference;
                positions.sort_by(|a, b| {
                    let dist_a = distance_squared(a, &(rx, ry, rz));
                    let dist_b = distance_squared(b, &(rx, ry, rz));
                    dist_a.cmp(&dist_b).then_with(|| {
                        // Tie-breaker: Y, X, Z
                        a.1.cmp(&b.1)
                            .then_with(|| a.0.cmp(&b.0))
                            .then_with(|| a.2.cmp(&b.2))
                    })
                });
            }
            SortStrategy::DistanceFromDesc { reference } => {
                let (rx, ry, rz) = *reference;
                positions.sort_by(|a, b| {
                    let dist_a = distance_squared(a, &(rx, ry, rz));
                    let dist_b = distance_squared(b, &(rx, ry, rz));
                    dist_b.cmp(&dist_a).then_with(|| {
                        // Tie-breaker: Y, X, Z descending
                        b.1.cmp(&a.1)
                            .then_with(|| b.0.cmp(&a.0))
                            .then_with(|| b.2.cmp(&a.2))
                    })
                });
            }
            SortStrategy::Preserve => {
                // Do nothing - keep original order
            }
            SortStrategy::Reverse => {
                positions.reverse();
            }
        }
    }

    /// Parse sort strategy from string
    ///
    /// Accepts various formats:
    /// - "yxz", "y_x_z", "y" -> YXZ
    /// - "xyz", "x_y_z", "x" -> XYZ
    /// - "zyx", "z_y_x", "z" -> ZYX
    /// - "y_desc", "ydesc" -> YDescXZ
    /// - "preserve", "none" -> Preserve
    /// - "reverse" -> Reverse
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('_', "").as_str() {
            "yxz" | "yfirst" | "y" => Some(SortStrategy::YXZ),
            "xyz" | "xfirst" | "x" => Some(SortStrategy::XYZ),
            "zyx" | "zfirst" | "z" => Some(SortStrategy::ZYX),
            "ydescxz" | "ydesc" => Some(SortStrategy::YDescXZ),
            "xdescyz" | "xdesc" => Some(SortStrategy::XDescYZ),
            "zdescyx" | "zdesc" => Some(SortStrategy::ZDescYX),
            "yxzdesc" | "desc" | "descending" => Some(SortStrategy::YXZDesc),
            "preserve" | "none" | "boxorder" => Some(SortStrategy::Preserve),
            "reverse" => Some(SortStrategy::Reverse),
            // Distance-based requires coordinates, can't parse from simple string
            _ => None,
        }
    }

    /// Create a distance-based sort strategy from a reference point
    pub fn distance_from(x: i32, y: i32, z: i32) -> Self {
        SortStrategy::DistanceFrom {
            reference: (x, y, z),
        }
    }

    /// Create a descending distance-based sort strategy from a reference point
    pub fn distance_from_desc(x: i32, y: i32, z: i32) -> Self {
        SortStrategy::DistanceFromDesc {
            reference: (x, y, z),
        }
    }

    /// Get a human-readable name for this strategy
    pub fn name(&self) -> &'static str {
        match self {
            SortStrategy::YXZ => "y_x_z",
            SortStrategy::XYZ => "x_y_z",
            SortStrategy::ZYX => "z_y_x",
            SortStrategy::YDescXZ => "y_desc_x_z",
            SortStrategy::XDescYZ => "x_desc_y_z",
            SortStrategy::ZDescYX => "z_desc_y_x",
            SortStrategy::YXZDesc => "y_x_z_desc",
            SortStrategy::DistanceFrom { .. } => "distance_from",
            SortStrategy::DistanceFromDesc { .. } => "distance_from_desc",
            SortStrategy::Preserve => "preserve",
            SortStrategy::Reverse => "reverse",
            SortStrategy::Custom(_) => "custom",
        }
    }
}

/// Calculate squared Euclidean distance between two points
fn distance_squared(a: &(i32, i32, i32), b: &(i32, i32, i32)) -> i64 {
    let dx = (a.0 - b.0) as i64;
    let dy = (a.1 - b.1) as i64;
    let dz = (a.2 - b.2) as i64;
    dx * dx + dy * dy + dz * dz
}

// ============================================================================
// Conversion from old SortStrategy (for backward compatibility)
// ============================================================================

impl SortStrategy {
    /// Convert from legacy insign SortStrategy
    pub fn from_legacy(legacy: &str, reference: Option<(i32, i32, i32)>) -> Option<Self> {
        match legacy.to_lowercase().as_str() {
            "distance" => {
                let r = reference.unwrap_or((0, 0, 0));
                Some(SortStrategy::DistanceFrom { reference: r })
            }
            "y_first" | "yfirst" => Some(SortStrategy::YXZ),
            "x_first" | "xfirst" => Some(SortStrategy::XYZ),
            "z_first" | "zfirst" => Some(SortStrategy::ZYX),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yxz_sort() {
        let positions = vec![(1, 2, 0), (0, 1, 0), (1, 1, 0), (0, 2, 0)];
        let sorted = SortStrategy::YXZ.sort(&positions);
        // Y=1 first, then Y=2
        assert_eq!(sorted, vec![(0, 1, 0), (1, 1, 0), (0, 2, 0), (1, 2, 0)]);
    }

    #[test]
    fn test_xyz_sort() {
        let positions = vec![(1, 0, 0), (0, 1, 0), (1, 1, 0), (0, 0, 0)];
        let sorted = SortStrategy::XYZ.sort(&positions);
        // X=0 first, then X=1
        assert_eq!(sorted, vec![(0, 0, 0), (0, 1, 0), (1, 0, 0), (1, 1, 0)]);
    }

    #[test]
    fn test_y_desc_sort() {
        let positions = vec![(0, 1, 0), (0, 2, 0), (1, 1, 0), (1, 2, 0)];
        let sorted = SortStrategy::YDescXZ.sort(&positions);
        // Y=2 first (descending), then Y=1
        assert_eq!(sorted, vec![(0, 2, 0), (1, 2, 0), (0, 1, 0), (1, 1, 0)]);
    }

    #[test]
    fn test_distance_from_sort() {
        let positions = vec![(0, 0, 0), (1, 0, 0), (2, 0, 0), (3, 0, 0)];
        let sorted = SortStrategy::distance_from(2, 0, 0).sort(&positions);
        // Distance from (2,0,0): (2,0,0)=0, (1,0,0)=1, (3,0,0)=1, (0,0,0)=4
        assert_eq!(sorted[0], (2, 0, 0)); // Distance 0
                                          // (1,0,0) and (3,0,0) both distance 1, tie-breaker by X
        assert_eq!(sorted[1], (1, 0, 0));
        assert_eq!(sorted[2], (3, 0, 0));
        assert_eq!(sorted[3], (0, 0, 0)); // Distance 4
    }

    #[test]
    fn test_preserve_order() {
        let positions = vec![(5, 5, 5), (0, 0, 0), (3, 3, 3), (1, 1, 1)];
        let sorted = SortStrategy::Preserve.sort(&positions);
        assert_eq!(sorted, positions);
    }

    #[test]
    fn test_reverse_order() {
        let positions = vec![(0, 0, 0), (1, 1, 1), (2, 2, 2)];
        let sorted = SortStrategy::Reverse.sort(&positions);
        assert_eq!(sorted, vec![(2, 2, 2), (1, 1, 1), (0, 0, 0)]);
    }

    #[test]
    fn test_from_str() {
        assert_eq!(SortStrategy::from_str("yxz"), Some(SortStrategy::YXZ));
        assert_eq!(SortStrategy::from_str("y_x_z"), Some(SortStrategy::YXZ));
        assert_eq!(SortStrategy::from_str("Y"), Some(SortStrategy::YXZ));
        assert_eq!(SortStrategy::from_str("xyz"), Some(SortStrategy::XYZ));
        assert_eq!(
            SortStrategy::from_str("y_desc"),
            Some(SortStrategy::YDescXZ)
        );
        assert_eq!(
            SortStrategy::from_str("preserve"),
            Some(SortStrategy::Preserve)
        );
        assert_eq!(
            SortStrategy::from_str("boxOrder"),
            Some(SortStrategy::Preserve)
        );
        assert_eq!(SortStrategy::from_str("invalid"), None);
    }

    #[test]
    fn test_multibox_scenario() {
        // Simulate two separate boxes that would be in a DefinitionRegion
        let box1_positions: Vec<(i32, i32, i32)> = (0..4).map(|x| (x, 0, 0)).collect();
        let box2_positions: Vec<(i32, i32, i32)> = (0..4).map(|x| (x, 0, 2)).collect();

        // Concatenate as they would be from iter_positions()
        let mut all_positions = box1_positions.clone();
        all_positions.extend(box2_positions.clone());

        // With Preserve, box order is maintained
        let preserved = SortStrategy::Preserve.sort(&all_positions);
        assert_eq!(&preserved[..4], &box1_positions[..]);
        assert_eq!(&preserved[4..], &box2_positions[..]);

        // With YXZ, everything gets globally sorted
        let sorted = SortStrategy::YXZ.sort(&all_positions);
        // All Y=0, so sorted by X, then Z
        // (0,0,0), (0,0,2), (1,0,0), (1,0,2), (2,0,0), (2,0,2), (3,0,0), (3,0,2)
        assert_eq!(
            sorted,
            vec![
                (0, 0, 0),
                (0, 0, 2),
                (1, 0, 0),
                (1, 0, 2),
                (2, 0, 0),
                (2, 0, 2),
                (3, 0, 0),
                (3, 0, 2)
            ]
        );
    }
}
