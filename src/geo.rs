//! Turning geospatial data into schematics: OSM-style building footprints
//! extruded to their heights, and elevation heightmaps raised into terrain.
//!
//! Pure and network-free by design — the caller fetches tiles / queries
//! Overpass and projects lat/lon to block coordinates; this module only stamps
//! the blocks. That keeps the schematic engine free of HTTP and lets the same
//! logic run against any data source.

use crate::block_state::BlockState;
use crate::building::{PolygonPrism, Shape};
use crate::universal_schematic::UniversalSchematic;

/// One extrudable footprint: a closed polygon in the X/Z plane, a Y span, and
/// the block to fill it with.
pub struct Footprint {
    pub polygon: Vec<(f64, f64)>,
    pub y_min: i32,
    pub y_max: i32,
    pub block: String,
}

/// Stamp building footprints into a massed schematic. Footprints are filled
/// tallest-last, so where they overlap each column keeps its tallest occupant
/// (the way `building:part` refinements sit on top of a base outline). When
/// `base_block` is `Some`, a one-block slab is laid at y=0 across the whole
/// footprint bounding rectangle first — the ground plane.
pub fn extrude_footprints(
    name: &str,
    footprints: &[Footprint],
    base_block: Option<&str>,
) -> UniversalSchematic {
    let mut s = UniversalSchematic::new(name.to_string());

    if let Some(base) = base_block.filter(|b| !b.is_empty()) {
        let (mut lo_x, mut lo_z) = (f64::INFINITY, f64::INFINITY);
        let (mut hi_x, mut hi_z) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
        for f in footprints {
            for &(x, z) in &f.polygon {
                lo_x = lo_x.min(x);
                lo_z = lo_z.min(z);
                hi_x = hi_x.max(x);
                hi_z = hi_z.max(z);
            }
        }
        if lo_x.is_finite() {
            let bs = BlockState::new(base);
            for x in lo_x.floor() as i32..=hi_x.ceil() as i32 {
                for z in lo_z.floor() as i32..=hi_z.ceil() as i32 {
                    s.set_block(x, 0, z, &bs);
                }
            }
        }
    }

    let mut order: Vec<usize> = (0..footprints.len()).collect();
    order.sort_by_key(|&i| footprints[i].y_max);
    for &i in &order {
        let f = &footprints[i];
        if f.polygon.len() < 3 {
            continue;
        }
        let bs = BlockState::new(f.block.as_str());
        let prism = PolygonPrism::new(f.polygon.clone(), f.y_min, f.y_max);
        prism.for_each_point(|x, y, z| {
            s.set_block(x, y, z, &bs);
        });
    }
    s
}

/// Raise terrain columns from a row-major heightmap of per-column heights
/// (in blocks), `width` columns per row. Each column rises to its height; the
/// top `surface_depth` blocks are `surface_block`, everything below is
/// `subsurface_block`.
pub fn heightmap_terrain(
    name: &str,
    heights: &[i32],
    width: usize,
    surface_block: &str,
    subsurface_block: &str,
    surface_depth: i32,
) -> UniversalSchematic {
    let mut s = UniversalSchematic::new(name.to_string());
    if width == 0 {
        return s;
    }
    let surf = BlockState::new(surface_block);
    let sub = BlockState::new(subsurface_block);
    let surface_depth = surface_depth.max(1);
    let depth = heights.len() / width;
    for gz in 0..depth {
        for gx in 0..width {
            let h = heights[gz * width + gx].max(0);
            for y in 0..=h {
                let bs = if y > h - surface_depth { &surf } else { &sub };
                s.set_block(gx as i32, y, gz as i32, bs);
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footprints_extrude_tallest_wins() {
        // A short wide slab and a tall tower that overlap: the tower's column
        // must win where they share ground.
        let fps = vec![
            Footprint {
                polygon: vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)],
                y_min: 1,
                y_max: 3,
                block: "minecraft:bricks".into(),
            },
            Footprint {
                polygon: vec![(4.0, 4.0), (7.0, 4.0), (7.0, 7.0), (4.0, 7.0)],
                y_min: 1,
                y_max: 20,
                block: "minecraft:quartz_block".into(),
            },
        ];
        let s = extrude_footprints("t", &fps, Some("minecraft:gray_concrete"));
        assert_eq!(s.get_block(0, 0, 0).unwrap().name, "minecraft:gray_concrete");
        assert_eq!(s.get_block(1, 2, 1).unwrap().name, "minecraft:bricks");
        // Overlap column: tower filled last, so it owns the shared levels.
        assert_eq!(s.get_block(5, 2, 5).unwrap().name, "minecraft:quartz_block");
        assert_eq!(s.get_block(5, 18, 5).unwrap().name, "minecraft:quartz_block");
    }

    #[test]
    fn heightmap_caps_with_surface() {
        // 2x2 grid, heights [2,2,5,5].
        let s = heightmap_terrain("t", &[2, 2, 5, 5], 2, "minecraft:grass_block", "minecraft:stone", 1);
        assert_eq!(s.get_block(0, 2, 0).unwrap().name, "minecraft:grass_block");
        assert_eq!(s.get_block(0, 1, 0).unwrap().name, "minecraft:stone");
        assert_eq!(s.get_block(0, 5, 1).unwrap().name, "minecraft:grass_block");
        assert!(
            s.get_block(0, 3, 0).map_or(true, |b| b.name == "minecraft:air"),
            "nothing solid above the column top"
        );
    }
}
