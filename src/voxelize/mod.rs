//! Mesh voxelization: turn GLB/OBJ triangle meshes into building
//! [`Shape`](crate::building::Shape)s and textured schematics.
//!
//! Pipeline: load a [`MeshModel`] ([`MeshModel::from_glb_bytes`] /
//! [`MeshModel::from_obj_str`]), normalize it into voxel space with
//! [`MeshModel::fit`], index it as a [`MeshShape`], then either fill it with
//! any brush via the building tool or run [`voxelize_textured`] to sample the
//! model's textures into palette blocks.

mod model;
mod shape;

pub use model::{MeshModel, MeshTriangle, TextureImage};
pub use shape::MeshShape;

use crate::building::{BlockPalette, Shape};
use crate::blockpedia::ExtendedColorData;
use crate::{BlockState, UniversalSchematic};

/// Fallback color for voxels with no texture information (mid-gray).
const FALLBACK_RGB: [u8; 3] = [128, 128, 128];

/// Voxelize `model_shape` into a schematic, coloring every solid voxel with
/// the palette block closest to its nearest-surface texture color. Interior
/// voxels inherit the color of the nearest surface point (they are hidden
/// anyway); voxels with no texture info (no UVs / no material) fall back to
/// the palette block closest to mid-gray.
pub fn voxelize_textured(
    model_shape: &MeshShape,
    palette: &BlockPalette,
    schematic_name: &str,
) -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new(schematic_name.to_string());
    model_shape.for_each_point(|x, y, z| {
        let rgb = model_shape.surface_color(x, y, z).unwrap_or(FALLBACK_RGB);
        let target = ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2]);
        if let Some(id) = palette.find_closest(&target) {
            schematic.set_block(x, y, z, &BlockState::new(id));
        }
    });
    schematic
}
