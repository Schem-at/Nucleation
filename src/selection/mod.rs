//! Connected-component flood selection over block coordinates.
//!
//! This module exists so callers can answer two questions efficiently:
//!
//! 1. *"Given a seed block, what build is it part of?"* — see [`flood`].
//! 2. *"Walk a volume of blocks and emit every build I find."* — see
//!    [`connected_components`] / [`connected_components_collect`].
//!
//! Both are mask-driven: callers supply a [`Mask`] (the predicate) and a
//! [`Connectivity`] choice (which neighbours count as connected). Adapter
//! masks for the common cases — non-air over a [`UniversalSchematic`],
//! allow-lists, boolean combinators — live in [`mask`].
//!
//! The design follows RedstoneTools' `/that` selector but generalises it
//! from a single-target selector to a streaming whole-volume scanner with
//! a shared visited set, so every block in the input is touched at most
//! once across the entire pass.

pub mod connectivity;
pub mod flood;
pub mod mask;
pub mod visited;

pub use connectivity::Connectivity;
pub use flood::{
    connected_components, connected_components_collect, flood, flood_with_visited, iter_bounds,
    Component, Continue, Limits, StopReason,
};
pub use mask::{AndMask, BlocklistMask, Mask, NotAirMask, NotMask, OrMask};
pub use visited::VisitedSet;

#[cfg(test)]
mod integration_tests {
    //! End-to-end tests that exercise the public surface against a real
    //! [`UniversalSchematic`] — the same path the build-extractor uses.

    use super::*;
    use crate::block_position::BlockPosition;
    use crate::block_state::BlockState;
    use crate::bounding_box::BoundingBox;
    use crate::universal_schematic::UniversalSchematic;

    /// Place a filled axis-aligned box of `block` inside `schem`.
    fn fill_box(
        schem: &mut UniversalSchematic,
        block: &BlockState,
        (mnx, mny, mnz): (i32, i32, i32),
        (mxx, mxy, mxz): (i32, i32, i32),
    ) {
        for x in mnx..=mxx {
            for y in mny..=mxy {
                for z in mnz..=mxz {
                    schem.set_block(x, y, z, block);
                }
            }
        }
    }

    #[test]
    fn two_separated_builds_in_a_universal_schematic() {
        let mut schem = UniversalSchematic::new("test".to_string());
        let stone = BlockState::new("minecraft:stone");
        let glass = BlockState::new("minecraft:glass");

        // Build A: 3x3x3 stone cube at origin (27 blocks).
        fill_box(&mut schem, &stone, (0, 0, 0), (2, 2, 2));
        // Build B: 2x2x2 glass cube far away (8 blocks).
        fill_box(&mut schem, &glass, (50, 0, 50), (51, 1, 51));

        // Sanity: schematic actually contains what we placed.
        assert_eq!(
            schem.get_block(0, 0, 0).map(|b| b.get_name()),
            Some("minecraft:stone")
        );
        assert_eq!(
            schem.get_block(50, 0, 50).map(|b| b.get_name()),
            Some("minecraft:glass")
        );

        let mask = NotAirMask::new(&schem);
        let bounds = BoundingBox::new((0, 0, 0), (51, 2, 51));
        let comps = connected_components_collect(
            iter_bounds(&bounds),
            &mask,
            Connectivity::Face,
            &Limits::unbounded(),
        );

        assert_eq!(comps.len(), 2, "should find exactly two builds");

        let mut sizes: Vec<usize> = comps.iter().map(|c| c.blocks.len()).collect();
        sizes.sort();
        assert_eq!(sizes, vec![8, 27]);

        // Bounds are tight to placed blocks (not the search volume).
        let cube_a = comps.iter().find(|c| c.blocks.len() == 27).unwrap();
        assert_eq!(cube_a.bounds.min, (0, 0, 0));
        assert_eq!(cube_a.bounds.max, (2, 2, 2));
        let cube_b = comps.iter().find(|c| c.blocks.len() == 8).unwrap();
        assert_eq!(cube_b.bounds.min, (50, 0, 50));
        assert_eq!(cube_b.bounds.max, (51, 1, 51));
    }

    #[test]
    fn corner_touching_builds_face_vs_corner() {
        // Two 2x2x2 cubes touching only at a single corner.
        // Cube A spans 0..=1, cube B spans 2..=3 with their nearest corners
        // at (1,1,1) and (2,2,2) — face-disconnected, corner-connected.
        let mut schem = UniversalSchematic::new("test".to_string());
        let stone = BlockState::new("minecraft:stone");
        fill_box(&mut schem, &stone, (0, 0, 0), (1, 1, 1));
        fill_box(&mut schem, &stone, (2, 2, 2), (3, 3, 3));

        let mask = NotAirMask::new(&schem);
        let bounds = BoundingBox::new((0, 0, 0), (3, 3, 3));

        let face = connected_components_collect(
            iter_bounds(&bounds),
            &mask,
            Connectivity::Face,
            &Limits::unbounded(),
        );
        assert_eq!(face.len(), 2);

        let corner = connected_components_collect(
            iter_bounds(&bounds),
            &mask,
            Connectivity::Corner,
            &Limits::unbounded(),
        );
        assert_eq!(corner.len(), 1);
        assert_eq!(corner[0].blocks.len(), 16);
    }

    #[test]
    fn blocklist_mask_filters_by_block_name() {
        // Two cubes of different materials; allow-list only one.
        let mut schem = UniversalSchematic::new("test".to_string());
        let stone = BlockState::new("minecraft:stone");
        let glass = BlockState::new("minecraft:glass");
        fill_box(&mut schem, &stone, (0, 0, 0), (1, 1, 1)); // 8 stone
        fill_box(&mut schem, &glass, (10, 0, 0), (11, 1, 1)); // 8 glass

        let mask = BlocklistMask::allow(&schem, ["minecraft:stone"]);
        let bounds = BoundingBox::new((0, 0, 0), (11, 1, 1));
        let comps = connected_components_collect(
            iter_bounds(&bounds),
            &mask,
            Connectivity::Face,
            &Limits::unbounded(),
        );
        assert_eq!(comps.len(), 1, "only the stone cube should pass the mask");
        assert_eq!(comps[0].blocks.len(), 8);
        assert_eq!(comps[0].bounds.min, (0, 0, 0));
        assert_eq!(comps[0].bounds.max, (1, 1, 1));
    }

    #[test]
    fn flood_from_arbitrary_seed_picks_one_build() {
        // Three builds; flood from a known seed should return only that one.
        let mut schem = UniversalSchematic::new("test".to_string());
        let stone = BlockState::new("minecraft:stone");
        fill_box(&mut schem, &stone, (0, 0, 0), (2, 2, 2)); // 27
        fill_box(&mut schem, &stone, (20, 0, 0), (21, 0, 0)); // 2
        fill_box(&mut schem, &stone, (40, 0, 0), (40, 4, 0)); // 5

        let mask = NotAirMask::new(&schem);

        let from_a = flood(
            BlockPosition::new(1, 1, 1),
            &mask,
            Connectivity::Face,
            &Limits::unbounded(),
        );
        assert_eq!(from_a.blocks.len(), 27);

        let from_b = flood(
            BlockPosition::new(20, 0, 0),
            &mask,
            Connectivity::Face,
            &Limits::unbounded(),
        );
        assert_eq!(from_b.blocks.len(), 2);

        let from_c = flood(
            BlockPosition::new(40, 2, 0),
            &mask,
            Connectivity::Face,
            &Limits::unbounded(),
        );
        assert_eq!(from_c.blocks.len(), 5);
    }
}
