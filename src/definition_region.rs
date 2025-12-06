use crate::bounding_box::BoundingBox;
use crate::BlockState;
use crate::UniversalSchematic;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// A DefinitionRegion represents a logical region defined by multiple bounding boxes.
/// It is used for defining inputs, outputs, and other logical constructs that may be disjoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionRegion {
    pub boxes: Vec<BoundingBox>,
    pub metadata: HashMap<String, String>,
}

impl Default for DefinitionRegion {
    fn default() -> Self {
        Self::new()
    }
}

impl DefinitionRegion {
    /// Create a new empty DefinitionRegion
    pub fn new() -> Self {
        DefinitionRegion {
            boxes: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a DefinitionRegion from a single bounding box
    pub fn from_bounds(min: (i32, i32, i32), max: (i32, i32, i32)) -> Self {
        let mut region = Self::new();
        region.add_bounds(min, max);
        region
    }

    /// Add a bounding box to the region
    pub fn add_bounds(&mut self, min: (i32, i32, i32), max: (i32, i32, i32)) -> &mut Self {
        // Ensure min/max are correctly ordered for the BoundingBox
        let true_min = (min.0.min(max.0), min.1.min(max.1), min.2.min(max.2));
        let true_max = (min.0.max(max.0), min.1.max(max.1), min.2.max(max.2));

        self.boxes.push(BoundingBox::new(true_min, true_max));
        self
    }

    /// Add a single point to the region
    pub fn add_point(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.add_bounds((x, y, z), (x, y, z))
    }

    /// Merge another region into this one
    pub fn merge(&mut self, other: &DefinitionRegion) -> &mut Self {
        self.boxes.extend(other.boxes.clone());
        self.metadata.extend(other.metadata.clone());
        self
    }

    /// Set metadata key-value pair
    pub fn with_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set color of the region (helper for metadata)
    pub fn set_color(&mut self, color: u32) -> &mut Self {
        let hex = format!("#{:06x}", color);
        self.with_metadata("color", hex)
    }

    /// Iterate over all positions in the region
    /// Iterates through boxes in order, and within each box in Y -> X -> Z order.
    ///
    /// **Warning:** This iteration order is NOT globally sorted across boxes.
    /// For deterministic bit ordering in circuits, use `iter_positions_sorted()` instead.
    pub fn iter_positions(&self) -> impl Iterator<Item = (i32, i32, i32)> + '_ {
        self.boxes.iter().flat_map(|bbox| {
            let (min_x, min_y, min_z) = bbox.min;
            let (max_x, max_y, max_z) = bbox.max;

            // Standard redstone order: Y-axis first (layers), then X (rows), then Z (columns)
            (min_y..=max_y).flat_map(move |y| {
                (min_x..=max_x).flat_map(move |x| (min_z..=max_z).map(move |z| (x, y, z)))
            })
        })
    }

    /// Iterate over all positions in the region in a globally sorted order.
    ///
    /// Returns positions sorted by Y (layer), then X (row), then Z (column).
    /// This provides **deterministic bit ordering** for circuits regardless of
    /// how the region was constructed (e.g., via union of disjoint boxes).
    ///
    /// Use this method when position order matters for IO bit assignment.
    pub fn iter_positions_sorted(&self) -> Vec<(i32, i32, i32)> {
        let mut positions: Vec<_> = self.iter_positions().collect();
        // Sort by Y first (layer), then X (row), then Z (column)
        // This matches standard Minecraft redstone conventions
        positions.sort_by(|a, b| {
            a.1.cmp(&b.1) // Y first
                .then(a.0.cmp(&b.0)) // then X
                .then(a.2.cmp(&b.2)) // then Z
        });
        positions.dedup(); // Remove duplicates from overlapping boxes
        positions
    }

    /// Get total volume (number of blocks) covered by all boxes
    pub fn volume(&self) -> u64 {
        self.boxes.iter().map(|b| b.volume()).sum()
    }

    /// Reconstruct the region from a set of points, merging adjacent points into larger bounding boxes.
    /// This effectively simplifies the region representation.
    pub fn from_positions(positions: &[(i32, i32, i32)]) -> Self {
        if positions.is_empty() {
            return Self::new();
        }

        let mut point_set: HashSet<(i32, i32, i32)> = positions.iter().cloned().collect();
        let mut boxes = Vec::new();

        // While there are points left to process
        while !point_set.is_empty() {
            // Pick a starting point (arbitrary, but consistent iteration order helps deterministic results)
            let start = *point_set.iter().next().unwrap();

            // Greedily expand in X, then Z, then Y
            let mut min = start;
            let mut max = start;

            // Expand +X
            while point_set.contains(&(max.0 + 1, min.1, min.2)) {
                max.0 += 1;
            }

            // Expand +Z (for the whole X row)
            let mut can_expand_z = true;
            while can_expand_z {
                let next_z = max.2 + 1;
                // Check if the whole row at next_z exists
                for x in min.0..=max.0 {
                    if !point_set.contains(&(x, min.1, next_z)) {
                        can_expand_z = false;
                        break;
                    }
                }
                if can_expand_z {
                    max.2 += 1;
                }
            }

            // Expand +Y (for the whole X*Z plane)
            let mut can_expand_y = true;
            while can_expand_y {
                let next_y = max.1 + 1;
                // Check if the whole plane at next_y exists
                for x in min.0..=max.0 {
                    for z in min.2..=max.2 {
                        if !point_set.contains(&(x, next_y, z)) {
                            can_expand_y = false;
                            break;
                        }
                    }
                    if !can_expand_y {
                        break;
                    }
                }
                if can_expand_y {
                    max.1 += 1;
                }
            }

            // Remove covered points from set
            for x in min.0..=max.0 {
                for y in min.1..=max.1 {
                    for z in min.2..=max.2 {
                        point_set.remove(&(x, y, z));
                    }
                }
            }

            boxes.push(BoundingBox::new(min, max));
        }

        DefinitionRegion {
            boxes,
            metadata: HashMap::new(),
        }
    }

    /// Simplify the region by merging adjacent/overlapping boxes
    pub fn simplify(&mut self) {
        let positions: Vec<_> = self.iter_positions().collect();
        let simplified = Self::from_positions(&positions);
        self.boxes = simplified.boxes;
    }

    // ========================================================================
    // Boolean Operations (Mutating)
    // ========================================================================

    /// Subtract another region from this one (removes points present in `other`)
    ///
    /// Mutates `self` in place. For an immutable version, use `subtracted()`.
    pub fn subtract(&mut self, other: &DefinitionRegion) -> &mut Self {
        let other_positions: HashSet<_> = other.iter_positions().collect();
        let remaining: Vec<_> = self
            .iter_positions()
            .filter(|pos| !other_positions.contains(pos))
            .collect();

        let simplified = Self::from_positions(&remaining);
        self.boxes = simplified.boxes;
        self
    }

    /// Keep only points present in both regions (intersection)
    ///
    /// Mutates `self` in place. For an immutable version, use `intersected()`.
    pub fn intersect(&mut self, other: &DefinitionRegion) -> &mut Self {
        let other_positions: HashSet<_> = other.iter_positions().collect();
        let intersection: Vec<_> = self
            .iter_positions()
            .filter(|pos| other_positions.contains(pos))
            .collect();

        let simplified = Self::from_positions(&intersection);
        self.boxes = simplified.boxes;
        self
    }

    /// Add all points from another region to this one (union)
    ///
    /// Mutates `self` in place. For an immutable version, use `union()`.
    pub fn union_into(&mut self, other: &DefinitionRegion) -> &mut Self {
        self.merge(other);
        self.simplify();
        self
    }

    // ========================================================================
    // Boolean Operations (Immutable)
    // ========================================================================

    /// Create a new region with points from `other` removed
    ///
    /// Returns a new region without modifying `self`.
    pub fn subtracted(&self, other: &DefinitionRegion) -> DefinitionRegion {
        let mut result = self.clone();
        result.subtract(other);
        result
    }

    /// Create a new region with only points present in both regions
    ///
    /// Returns a new region without modifying `self`.
    pub fn intersected(&self, other: &DefinitionRegion) -> DefinitionRegion {
        let mut result = self.clone();
        result.intersect(other);
        result
    }

    /// Create a new region that is the union of this region and another
    ///
    /// Returns a new region without modifying `self`.
    pub fn union(&self, other: &DefinitionRegion) -> DefinitionRegion {
        let mut result = self.clone();
        result.merge(other);
        result.simplify();
        result
    }

    // ========================================================================
    // Geometric Transformations
    // ========================================================================

    /// Translate all boxes by the given offset
    pub fn shift(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        for bbox in &mut self.boxes {
            bbox.min = (bbox.min.0 + x, bbox.min.1 + y, bbox.min.2 + z);
            bbox.max = (bbox.max.0 + x, bbox.max.1 + y, bbox.max.2 + z);
        }
        self
    }

    /// Expand all boxes by the given amounts in each direction
    /// Positive values expand outward, negative values contract
    pub fn expand(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        for bbox in &mut self.boxes {
            bbox.min = (bbox.min.0 - x, bbox.min.1 - y, bbox.min.2 - z);
            bbox.max = (bbox.max.0 + x, bbox.max.1 + y, bbox.max.2 + z);
        }
        // Remove any boxes that became invalid (min > max)
        self.boxes.retain(|bbox| {
            bbox.min.0 <= bbox.max.0 && bbox.min.1 <= bbox.max.1 && bbox.min.2 <= bbox.max.2
        });
        self
    }

    /// Contract all boxes by the given amount uniformly
    pub fn contract(&mut self, amount: i32) -> &mut Self {
        self.expand(-amount, -amount, -amount)
    }

    /// Get the overall bounding box encompassing all boxes in this region
    pub fn get_bounds(&self) -> Option<BoundingBox> {
        if self.boxes.is_empty() {
            return None;
        }

        let first = &self.boxes[0];
        let mut min = first.min;
        let mut max = first.max;

        for bbox in &self.boxes[1..] {
            min.0 = min.0.min(bbox.min.0);
            min.1 = min.1.min(bbox.min.1);
            min.2 = min.2.min(bbox.min.2);
            max.0 = max.0.max(bbox.max.0);
            max.1 = max.1.max(bbox.max.1);
            max.2 = max.2.max(bbox.max.2);
        }

        Some(BoundingBox::new(min, max))
    }

    // ========================================================================
    // Connectivity Analysis
    // ========================================================================

    /// Check if all points in the region are connected (6-connectivity)
    /// Returns true if empty or all points form a single connected component
    pub fn is_contiguous(&self) -> bool {
        let positions: HashSet<_> = self.iter_positions().collect();
        if positions.len() <= 1 {
            return true;
        }

        // BFS from any starting point
        let start = *positions.iter().next().unwrap();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(pos) = queue.pop_front() {
            // Check 6 neighbors (face-adjacent)
            let neighbors = [
                (pos.0 - 1, pos.1, pos.2),
                (pos.0 + 1, pos.1, pos.2),
                (pos.0, pos.1 - 1, pos.2),
                (pos.0, pos.1 + 1, pos.2),
                (pos.0, pos.1, pos.2 - 1),
                (pos.0, pos.1, pos.2 + 1),
            ];

            for neighbor in neighbors {
                if positions.contains(&neighbor) && !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        visited.len() == positions.len()
    }

    /// Get the number of connected components in this region
    pub fn connected_components(&self) -> usize {
        let positions: HashSet<_> = self.iter_positions().collect();
        if positions.is_empty() {
            return 0;
        }

        let mut remaining = positions.clone();
        let mut components = 0;

        while !remaining.is_empty() {
            let start = *remaining.iter().next().unwrap();
            let mut queue = VecDeque::new();
            queue.push_back(start);
            remaining.remove(&start);

            while let Some(pos) = queue.pop_front() {
                let neighbors = [
                    (pos.0 - 1, pos.1, pos.2),
                    (pos.0 + 1, pos.1, pos.2),
                    (pos.0, pos.1 - 1, pos.2),
                    (pos.0, pos.1 + 1, pos.2),
                    (pos.0, pos.1, pos.2 - 1),
                    (pos.0, pos.1, pos.2 + 1),
                ];

                for neighbor in neighbors {
                    if remaining.contains(&neighbor) {
                        remaining.remove(&neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }

            components += 1;
        }

        components
    }

    // ========================================================================
    // Filtering
    // ========================================================================

    /// Filter positions by block name (substring match)
    /// This modifies the region in-place to keep only blocks matching the name.
    pub fn filter_by_block(
        &mut self,
        schematic: &UniversalSchematic,
        block_name: &str,
    ) -> &mut Self {
        let filtered = self.filter_by_block_immutable(schematic, block_name);
        *self = filtered;
        self
    }

    /// Exclude a block type from the region (subtraction)
    /// This modifies the region in-place to remove blocks matching the name.
    pub fn exclude_block(&mut self, schematic: &UniversalSchematic, block_name: &str) -> &mut Self {
        let to_exclude = self.filter_by_block_immutable(schematic, block_name);
        self.subtract(&to_exclude);
        self
    }

    /// Internal immutable filter helper
    fn filter_by_block_immutable(&self, schematic: &UniversalSchematic, block_name: &str) -> Self {
        let positions: Vec<_> = self
            .iter_positions()
            .filter(|&(x, y, z)| {
                if let Some(block) = schematic.get_block(x, y, z) {
                    block.name.contains(block_name)
                } else {
                    false
                }
            })
            .collect();

        Self::from_positions(&positions)
    }

    /// Filter positions by block state properties
    /// Only keeps positions where the block has ALL specified properties matching
    pub fn filter_by_properties(
        &self,
        schematic: &UniversalSchematic,
        properties: &HashMap<String, String>,
    ) -> Self {
        let positions: Vec<_> = self
            .iter_positions()
            .filter(|&(x, y, z)| {
                if let Some(block) = schematic.get_block(x, y, z) {
                    properties
                        .iter()
                        .all(|(key, value)| block.properties.get(key).map_or(false, |v| v == value))
                } else {
                    false
                }
            })
            .collect();

        Self::from_positions(&positions)
    }

    /// Filter positions where a custom predicate returns true
    pub fn filter_by<F>(&self, schematic: &UniversalSchematic, predicate: F) -> Self
    where
        F: Fn(&BlockState) -> bool,
    {
        let positions: Vec<_> = self
            .iter_positions()
            .filter(|&(x, y, z)| {
                schematic
                    .get_block(x, y, z)
                    .map_or(false, |block| predicate(block))
            })
            .collect();

        Self::from_positions(&positions)
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Check if the region is empty
    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    /// Check if the region contains a specific point
    pub fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        self.boxes.iter().any(|bbox| bbox.contains((x, y, z)))
    }

    /// Get a list of all positions as a Vec
    pub fn positions(&self) -> Vec<(i32, i32, i32)> {
        self.iter_positions().collect()
    }

    // ========================================================================
    // Box Access (for Rendering)
    // ========================================================================

    /// Create a DefinitionRegion from multiple bounding boxes
    ///
    /// Unlike `from_positions()` which takes individual points and merges them,
    /// this takes pre-defined bounding boxes directly.
    pub fn from_bounding_boxes(boxes: Vec<((i32, i32, i32), (i32, i32, i32))>) -> Self {
        let mut region = Self::new();
        for (min, max) in boxes {
            region.add_bounds(min, max);
        }
        region
    }

    /// Get the number of bounding boxes in this region
    pub fn box_count(&self) -> usize {
        self.boxes.len()
    }

    /// Get a specific bounding box by index
    ///
    /// Returns None if index is out of bounds.
    /// The returned tuple is ((min_x, min_y, min_z), (max_x, max_y, max_z))
    pub fn get_box(&self, index: usize) -> Option<((i32, i32, i32), (i32, i32, i32))> {
        self.boxes.get(index).map(|bbox| (bbox.min, bbox.max))
    }

    /// Get all bounding boxes in this region
    ///
    /// Returns a Vec of ((min_x, min_y, min_z), (max_x, max_y, max_z)) tuples.
    /// Useful for rendering each box separately.
    pub fn get_boxes(&self) -> Vec<((i32, i32, i32), (i32, i32, i32))> {
        self.boxes.iter().map(|bbox| (bbox.min, bbox.max)).collect()
    }

    /// Get a reference to the internal boxes (for advanced use)
    pub fn boxes_ref(&self) -> &[BoundingBox] {
        &self.boxes
    }

    // ========================================================================
    // Metadata Access
    // ========================================================================

    /// Get a metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Set a metadata value (mutating version)
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get all metadata as a reference
    pub fn metadata_ref(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Get all metadata keys
    pub fn metadata_keys(&self) -> Vec<&String> {
        self.metadata.keys().collect()
    }

    // ========================================================================
    // Geometry Helpers (for Rendering)
    // ========================================================================

    /// Get the dimensions (width, height, length) of the overall bounding box
    ///
    /// Returns (0, 0, 0) if the region is empty.
    pub fn dimensions(&self) -> (i32, i32, i32) {
        match self.get_bounds() {
            Some(bbox) => bbox.get_dimensions(),
            None => (0, 0, 0),
        }
    }

    /// Get the center point of the region (integer coordinates)
    ///
    /// Returns None if the region is empty.
    pub fn center(&self) -> Option<(i32, i32, i32)> {
        self.get_bounds().map(|bbox| {
            (
                (bbox.min.0 + bbox.max.0) / 2,
                (bbox.min.1 + bbox.max.1) / 2,
                (bbox.min.2 + bbox.max.2) / 2,
            )
        })
    }

    /// Get the center point of the region as f32 (for rendering)
    ///
    /// Returns None if the region is empty.
    pub fn center_f32(&self) -> Option<(f32, f32, f32)> {
        self.get_bounds().map(|bbox| {
            (
                (bbox.min.0 as f32 + bbox.max.0 as f32 + 1.0) / 2.0,
                (bbox.min.1 as f32 + bbox.max.1 as f32 + 1.0) / 2.0,
                (bbox.min.2 as f32 + bbox.max.2 as f32 + 1.0) / 2.0,
            )
        })
    }

    /// Check if this region intersects with a bounding box
    ///
    /// Useful for frustum culling in renderers.
    pub fn intersects_bounds(&self, min: (i32, i32, i32), max: (i32, i32, i32)) -> bool {
        let query = BoundingBox::new(min, max);
        self.boxes.iter().any(|bbox| bbox.intersects(&query))
    }

    // ========================================================================
    // Immutable Geometric Transformations
    // ========================================================================

    /// Create a new region shifted by the given offset (immutable)
    ///
    /// Returns a new region without modifying `self`.
    pub fn shifted(&self, x: i32, y: i32, z: i32) -> Self {
        let mut result = self.clone();
        result.shift(x, y, z);
        result
    }

    /// Create a new region expanded by the given amounts (immutable)
    ///
    /// Returns a new region without modifying `self`.
    pub fn expanded(&self, x: i32, y: i32, z: i32) -> Self {
        let mut result = self.clone();
        result.expand(x, y, z);
        result
    }

    /// Create a new region contracted by the given amount (immutable)
    ///
    /// Returns a new region without modifying `self`.
    pub fn contracted(&self, amount: i32) -> Self {
        let mut result = self.clone();
        result.contract(amount);
        result
    }

    /// Create a deep copy of this region
    ///
    /// Explicit clone method for language bindings that don't have
    /// automatic Clone support.
    pub fn copy(&self) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtract() {
        let mut region_a = DefinitionRegion::from_bounds((0, 0, 0), (2, 0, 0));
        let region_b = DefinitionRegion::from_bounds((1, 0, 0), (1, 0, 0));

        region_a.subtract(&region_b);

        let positions: Vec<_> = region_a.iter_positions().collect();
        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&(0, 0, 0)));
        assert!(positions.contains(&(2, 0, 0)));
        assert!(!positions.contains(&(1, 0, 0)));
    }

    #[test]
    fn test_intersect() {
        let mut region_a = DefinitionRegion::from_bounds((0, 0, 0), (2, 0, 0));
        let region_b = DefinitionRegion::from_bounds((1, 0, 0), (3, 0, 0));

        region_a.intersect(&region_b);

        let positions: Vec<_> = region_a.iter_positions().collect();
        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&(1, 0, 0)));
        assert!(positions.contains(&(2, 0, 0)));
    }

    #[test]
    fn test_shift() {
        let mut region = DefinitionRegion::from_bounds((0, 0, 0), (1, 1, 1));
        region.shift(10, 20, 30);

        let bounds = region.get_bounds().unwrap();
        assert_eq!(bounds.min, (10, 20, 30));
        assert_eq!(bounds.max, (11, 21, 31));
    }

    #[test]
    fn test_expand_contract() {
        let mut region = DefinitionRegion::from_bounds((5, 5, 5), (10, 10, 10));
        region.expand(2, 2, 2);

        let bounds = region.get_bounds().unwrap();
        assert_eq!(bounds.min, (3, 3, 3));
        assert_eq!(bounds.max, (12, 12, 12));

        region.contract(2);
        let bounds = region.get_bounds().unwrap();
        assert_eq!(bounds.min, (5, 5, 5));
        assert_eq!(bounds.max, (10, 10, 10));
    }

    #[test]
    fn test_is_contiguous() {
        // Single line - should be contiguous
        let region1 = DefinitionRegion::from_bounds((0, 0, 0), (5, 0, 0));
        assert!(region1.is_contiguous());

        // Two separate points - not contiguous
        let mut region2 = DefinitionRegion::new();
        region2.add_point(0, 0, 0);
        region2.add_point(5, 5, 5);
        assert!(!region2.is_contiguous());

        // L-shape - contiguous
        let mut region3 = DefinitionRegion::new();
        region3.add_bounds((0, 0, 0), (2, 0, 0));
        region3.add_bounds((2, 0, 0), (2, 2, 0));
        assert!(region3.is_contiguous());
    }

    #[test]
    fn test_connected_components() {
        // Two separate clusters
        let mut region = DefinitionRegion::new();
        region.add_bounds((0, 0, 0), (1, 0, 0));
        region.add_bounds((10, 0, 0), (11, 0, 0));

        assert_eq!(region.connected_components(), 2);
    }

    #[test]
    fn test_contains() {
        let region = DefinitionRegion::from_bounds((0, 0, 0), (10, 10, 10));

        assert!(region.contains(5, 5, 5));
        assert!(region.contains(0, 0, 0));
        assert!(region.contains(10, 10, 10));
        assert!(!region.contains(11, 0, 0));
        assert!(!region.contains(-1, 0, 0));
    }

    // ========================================================================
    // NEW TESTS: Box Access
    // ========================================================================

    #[test]
    fn test_from_bounding_boxes() {
        let boxes = vec![((0, 0, 0), (2, 2, 2)), ((5, 5, 5), (7, 7, 7))];
        let region = DefinitionRegion::from_bounding_boxes(boxes);

        assert_eq!(region.box_count(), 2);
        assert!(region.contains(1, 1, 1));
        assert!(region.contains(6, 6, 6));
        assert!(!region.contains(3, 3, 3));
    }

    #[test]
    fn test_box_count() {
        let mut region = DefinitionRegion::new();
        assert_eq!(region.box_count(), 0);

        region.add_bounds((0, 0, 0), (1, 1, 1));
        assert_eq!(region.box_count(), 1);

        region.add_bounds((5, 5, 5), (6, 6, 6));
        assert_eq!(region.box_count(), 2);
    }

    #[test]
    fn test_get_box() {
        let region = DefinitionRegion::from_bounding_boxes(vec![
            ((0, 0, 0), (2, 2, 2)),
            ((5, 5, 5), (7, 7, 7)),
        ]);

        let box0 = region.get_box(0).unwrap();
        assert_eq!(box0, ((0, 0, 0), (2, 2, 2)));

        let box1 = region.get_box(1).unwrap();
        assert_eq!(box1, ((5, 5, 5), (7, 7, 7)));

        assert!(region.get_box(2).is_none());
    }

    #[test]
    fn test_get_boxes() {
        let region = DefinitionRegion::from_bounding_boxes(vec![
            ((0, 0, 0), (1, 1, 1)),
            ((3, 3, 3), (4, 4, 4)),
        ]);

        let boxes = region.get_boxes();
        assert_eq!(boxes.len(), 2);
        assert_eq!(boxes[0], ((0, 0, 0), (1, 1, 1)));
        assert_eq!(boxes[1], ((3, 3, 3), (4, 4, 4)));
    }

    // ========================================================================
    // NEW TESTS: Metadata Access
    // ========================================================================

    #[test]
    fn test_metadata_access() {
        let mut region = DefinitionRegion::new();

        // Initially no metadata
        assert!(region.get_metadata("color").is_none());

        // Set metadata
        region.set_metadata("color", "red");
        region.set_metadata("label", "Input A");

        // Get metadata
        assert_eq!(region.get_metadata("color"), Some(&"red".to_string()));
        assert_eq!(region.get_metadata("label"), Some(&"Input A".to_string()));
        assert!(region.get_metadata("nonexistent").is_none());

        // Get all keys
        let keys = region.metadata_keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_with_metadata_chaining() {
        let mut region = DefinitionRegion::from_bounds((0, 0, 0), (1, 1, 1));
        region
            .with_metadata("type", "input")
            .with_metadata("index", "0");

        assert_eq!(region.get_metadata("type"), Some(&"input".to_string()));
        assert_eq!(region.get_metadata("index"), Some(&"0".to_string()));
    }

    // ========================================================================
    // NEW TESTS: Geometry Helpers
    // ========================================================================

    #[test]
    fn test_dimensions() {
        let region = DefinitionRegion::from_bounds((0, 0, 0), (9, 4, 2));
        let dims = region.dimensions();
        assert_eq!(dims, (10, 5, 3)); // inclusive bounds

        // Empty region
        let empty = DefinitionRegion::new();
        assert_eq!(empty.dimensions(), (0, 0, 0));
    }

    #[test]
    fn test_center() {
        // Simple box
        let region = DefinitionRegion::from_bounds((0, 0, 0), (10, 10, 10));
        assert_eq!(region.center(), Some((5, 5, 5)));

        // Box not at origin
        let region2 = DefinitionRegion::from_bounds((10, 20, 30), (20, 30, 40));
        assert_eq!(region2.center(), Some((15, 25, 35)));

        // Empty region
        let empty = DefinitionRegion::new();
        assert_eq!(empty.center(), None);
    }

    #[test]
    fn test_center_f32() {
        let region = DefinitionRegion::from_bounds((0, 0, 0), (9, 9, 9));
        let center = region.center_f32().unwrap();
        // Center of 0..9 inclusive is at 5.0 (middle of block coordinates)
        assert!((center.0 - 5.0).abs() < 0.01);
        assert!((center.1 - 5.0).abs() < 0.01);
        assert!((center.2 - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_intersects_bounds() {
        let region = DefinitionRegion::from_bounds((0, 0, 0), (10, 10, 10));

        // Intersecting
        assert!(region.intersects_bounds((5, 5, 5), (15, 15, 15)));
        assert!(region.intersects_bounds((-5, -5, -5), (5, 5, 5)));
        assert!(region.intersects_bounds((5, 5, 5), (6, 6, 6))); // Inside

        // Not intersecting
        assert!(!region.intersects_bounds((20, 20, 20), (30, 30, 30)));
        assert!(!region.intersects_bounds((-10, -10, -10), (-1, -1, -1)));
    }

    // ========================================================================
    // NEW TESTS: Immutable Transformations
    // ========================================================================

    #[test]
    fn test_shifted() {
        let original = DefinitionRegion::from_bounds((0, 0, 0), (5, 5, 5));
        let shifted = original.shifted(10, 20, 30);

        // Original unchanged
        let orig_bounds = original.get_bounds().unwrap();
        assert_eq!(orig_bounds.min, (0, 0, 0));
        assert_eq!(orig_bounds.max, (5, 5, 5));

        // Shifted is different
        let shifted_bounds = shifted.get_bounds().unwrap();
        assert_eq!(shifted_bounds.min, (10, 20, 30));
        assert_eq!(shifted_bounds.max, (15, 25, 35));
    }

    #[test]
    fn test_expanded() {
        let original = DefinitionRegion::from_bounds((5, 5, 5), (10, 10, 10));
        let expanded = original.expanded(2, 2, 2);

        // Original unchanged
        let orig_bounds = original.get_bounds().unwrap();
        assert_eq!(orig_bounds.min, (5, 5, 5));

        // Expanded is different
        let exp_bounds = expanded.get_bounds().unwrap();
        assert_eq!(exp_bounds.min, (3, 3, 3));
        assert_eq!(exp_bounds.max, (12, 12, 12));
    }

    #[test]
    fn test_contracted() {
        let original = DefinitionRegion::from_bounds((0, 0, 0), (10, 10, 10));
        let contracted = original.contracted(2);

        // Original unchanged
        let orig_bounds = original.get_bounds().unwrap();
        assert_eq!(orig_bounds.min, (0, 0, 0));

        // Contracted is different
        let cont_bounds = contracted.get_bounds().unwrap();
        assert_eq!(cont_bounds.min, (2, 2, 2));
        assert_eq!(cont_bounds.max, (8, 8, 8));
    }

    #[test]
    fn test_copy() {
        let mut original = DefinitionRegion::from_bounds((0, 0, 0), (5, 5, 5));
        original.with_metadata("name", "test");

        let copy = original.copy();

        // Both have same content
        assert_eq!(copy.box_count(), 1);
        assert_eq!(copy.get_metadata("name"), Some(&"test".to_string()));

        // They are independent
        let orig_bounds = original.get_bounds().unwrap();
        let copy_bounds = copy.get_bounds().unwrap();
        assert_eq!(orig_bounds.min, copy_bounds.min);
    }

    // ========================================================================
    // NEW TESTS: from_positions vs from_bounding_boxes
    // ========================================================================

    #[test]
    fn test_from_positions_creates_region() {
        // Create points forming a line
        let positions = vec![(0, 0, 0), (1, 0, 0), (2, 0, 0), (3, 0, 0)];

        let region = DefinitionRegion::from_positions(&positions);

        // Volume should be exactly 4 points
        assert_eq!(region.volume(), 4);

        // All positions should be contained
        for pos in &positions {
            assert!(region.contains(pos.0, pos.1, pos.2));
        }

        // Should have some boxes (algorithm may vary)
        assert!(region.box_count() >= 1);
    }

    #[test]
    fn test_from_positions_covers_all_points() {
        // Create 8 points forming a 2x2x2 cube
        // The algorithm may create multiple boxes but should cover all points
        let positions = vec![
            (0, 0, 0),
            (1, 0, 0),
            (0, 1, 0),
            (1, 1, 0),
            (0, 0, 1),
            (1, 0, 1),
            (0, 1, 1),
            (1, 1, 1),
        ];

        let region = DefinitionRegion::from_positions(&positions);

        // Volume should cover all 8 points
        assert_eq!(region.volume(), 8);

        // All original positions should be contained
        for pos in &positions {
            assert!(
                region.contains(pos.0, pos.1, pos.2),
                "Region should contain {:?}",
                pos
            );
        }
    }

    #[test]
    fn test_from_bounding_boxes_keeps_separate() {
        // Two separate boxes
        let boxes = vec![((0, 0, 0), (0, 0, 0)), ((2, 2, 2), (2, 2, 2))];

        let region = DefinitionRegion::from_bounding_boxes(boxes);

        // Kept as 2 boxes (not merged)
        assert_eq!(region.box_count(), 2);
        assert_eq!(region.volume(), 2);
    }
}
