//! Mesh voxelization: turn GLB/OBJ triangle meshes into building
//! [`Shape`](crate::building::Shape)s and textured schematics.
//!
//! Pipeline: load a [`MeshModel`] ([`MeshModel::from_glb_bytes`] /
//! [`MeshModel::from_obj_str`]), normalize it into voxel space with
//! [`MeshModel::fit`], index it as a [`MeshShape`], then either fill it with
//! any brush via the building tool or run [`voxelize_textured`] to sample the
//! model's textures into palette blocks.

mod configured;
mod model;
mod shape;
#[doc(hidden)]
pub mod test_meshes;

pub use configured::{VoxelLight, VoxelizeOptions, VoxelizePlan};
pub use model::{MeshModel, MeshTriangle, TextureImage};
pub use shape::MeshShape;

use crate::blockpedia::ExtendedColorData;
use crate::building::{BlockPalette, Shape};
use crate::{BlockState, UniversalSchematic};

/// Fallback color for voxels with no texture information (mid-gray).
const FALLBACK_RGB: [u8; 3] = [128, 128, 128];

/// Forces [`voxelize_textured`] down its sequential path on every target.
/// A test hook, so the sequential and the rayon path can be pinned to the
/// same output in one test binary, and an escape hatch for a host that does
/// not want the voxelizer taking the rayon pool. Not part of the API.
#[doc(hidden)]
pub static VOXELIZE_FORCE_SEQUENTIAL: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

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
    use rayon::prelude::*;
    use std::collections::HashMap;

    let mut schematic = UniversalSchematic::new(schematic_name.to_string());

    // One pass to enumerate the solid voxels, so the colour sampling below
    // can run over a slice. The mask is already built by then.
    let mut points: Vec<(i32, i32, i32)> = Vec::new();
    model_shape.for_each_point(|x, y, z| points.push((x, y, z)));

    // Build the surface field before entering the parallel region. It is a
    // rayon pass of its own behind a OnceLock, so letting the first sample
    // trigger it would nest one rayon pass inside another with every other
    // worker parked on the lock.
    model_shape.warm_surface_field();

    // Sample every voxel's surface colour, packed as 24 bit RGB. O(1) per
    // voxel since the surface field landed, and on native it runs on rayon.
    let sample = |&(x, y, z): &(i32, i32, i32)| -> u32 {
        let rgb = model_shape.surface_color(x, y, z).unwrap_or(FALLBACK_RGB);
        ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | rgb[2] as u32
    };
    let colors: Vec<u32> = if use_parallel() {
        points.par_iter().map(sample).collect()
    } else {
        points.iter().map(sample).collect()
    };

    // Memoise the palette search on the exact 24 bit colour. A texture has
    // far fewer distinct colours than the model has voxels, so this turns a
    // per voxel palette scan into one scan per distinct colour. The key is
    // exact rather than quantised on purpose: quantising would change which
    // block some voxels get, and the golden fixture pins them.
    //
    // Each voxel keeps a u32 slot into `distinct` rather than a palette
    // index, which is the same four bytes and cannot overflow on a palette
    // larger than 65,535 entries. First seen order makes `distinct`
    // deterministic, so the parallel match below is too.
    let mut memo: HashMap<u32, u32> = HashMap::new();
    let mut distinct: Vec<u32> = Vec::new();
    let mut slots: Vec<u32> = Vec::with_capacity(colors.len());
    for &key in &colors {
        let slot = *memo.entry(key).or_insert_with(|| {
            distinct.push(key);
            (distinct.len() - 1) as u32
        });
        slots.push(slot);
    }
    // The slots carry everything the rest of the walk needs, so give the
    // sampled colours back here: 20 bytes per voxel at the peak, 16 after.
    drop(colors);

    // One palette scan per distinct colour, in parallel on native. The scan
    // itself is unchanged, so every voxel still gets the entry the per voxel
    // loop would have given it, ties included.
    let match_color = |&key: &u32| -> Option<usize> {
        let target = ExtendedColorData::from_rgb((key >> 16) as u8, (key >> 8) as u8, key as u8);
        palette.find_closest_index(&target)
    };
    let matched: Vec<Option<usize>> = if use_parallel() {
        distinct.par_iter().map(match_color).collect()
    } else {
        distinct.iter().map(match_color).collect()
    };

    // Resolve each distinct palette index to a BlockState once.
    let mut states: HashMap<usize, BlockState> = HashMap::new();
    for index in matched.iter().flatten() {
        if let std::collections::hash_map::Entry::Vacant(slot) = states.entry(*index) {
            if let Some(id) = palette.block_id(*index) {
                slot.insert(BlockState::new(id));
            }
        }
    }

    for (&(x, y, z), &slot) in points.iter().zip(&slots) {
        if let Some(state) = matched[slot as usize].and_then(|i| states.get(&i)) {
            schematic.set_block(x, y, z, state);
        }
    }
    schematic
}

/// Whether the textured walk may use rayon. Native yes, wasm32 no (it has no
/// thread pool worth the name), and [`VOXELIZE_FORCE_SEQUENTIAL`] forces the
/// sequential path anywhere, which is how the two are tested against each
/// other.
fn use_parallel() -> bool {
    cfg!(not(target_arch = "wasm32"))
        && !VOXELIZE_FORCE_SEQUENTIAL.load(std::sync::atomic::Ordering::Relaxed)
}
