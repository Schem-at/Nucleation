//! `DistanceField`: depth-from-surface and a surface normal over a build's
//! occupancy, so materials can key on depth and slope over arbitrary geometry
//! (an imported schematic, a voxelized model, map data) the way an SDF shape
//! already exposes for free.

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use diplomat_runtime::DiplomatWrite;
    use std::collections::HashSet;
    use std::fmt::Write as _;

    #[diplomat::opaque]
    pub struct DistanceField(pub(crate) crate::building::DistanceField);

    impl DistanceField {
        /// Distance transform of a build's occupied voxels: every solid block
        /// learns how many blocks it sits below the surface, and the gradient of
        /// that depth gives the outward normal. Computed once over the
        /// schematic's bounding box.
        pub fn from_schematic(schematic: &Schematic) -> Box<DistanceField> {
            let bb = schematic.0.get_bounding_box();
            let mut occ: HashSet<(i32, i32, i32)> = HashSet::new();
            for (pos, block) in schematic.0.iter_blocks() {
                if block.name.as_str() != "minecraft:air" {
                    occ.insert((pos.x, pos.y, pos.z));
                }
            }
            let field =
                crate::building::DistanceField::from_occupancy(bb.min, bb.max, |x, y, z| {
                    occ.contains(&(x, y, z))
                });
            Box::new(DistanceField(field))
        }

        /// Blocks below the surface at a voxel: 0 for empty/outside, 1 at the
        /// surface, increasing inward.
        pub fn depth(&self, x: i32, y: i32, z: i32) -> i32 {
            self.0.depth_at(x, y, z)
        }

        /// The upward component of the outward surface normal: 1 on flat ground,
        /// 0 on a vertical face, negative under an overhang. The scalar to key
        /// slope-based landscaping on (grass on the flats, stone on the steeps).
        pub fn slope(&self, x: i32, y: i32, z: i32) -> f32 {
            self.0.normal_at(x, y, z).1 as f32
        }

        /// The full outward surface normal as JSON `[nx, ny, nz]`.
        pub fn normal_json(&self, x: i32, y: i32, z: i32, out: &mut DiplomatWrite) {
            let (nx, ny, nz) = self.0.normal_at(x, y, z);
            let _ = write!(out, "[{nx},{ny},{nz}]");
        }
    }
}
