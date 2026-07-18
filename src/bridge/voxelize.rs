//! Mesh voxelization: GLB/OBJ models → building Shapes and textured
//! schematics. New surface (no old `ffi` counterpart).

#[diplomat::bridge]
pub mod ffi {
    use super::super::building::ffi::{Palette, Shape};
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;

    /// Namespace for the mesh-voxelization entry points (GLB/OBJ → shapes
    /// and textured schematics).
    #[diplomat::opaque]
    pub struct Voxelizer;

    impl Voxelizer {
        /// Load a binary glTF (`.glb`, embedded buffers/images) and voxelize
        /// it into a fillable Shape: the model is uniformly scaled so its
        /// largest dimension equals `target_size` voxels, centered on x/z
        /// with its base resting at y = 0. Solidity is a parity test at each
        /// voxel center (robust on closed meshes; open meshes are
        /// best-effort). Errors with `Parse` on malformed/triangle-less GLB
        /// and `InvalidArgument` on a non-positive `target_size`.
        pub fn shape_from_glb(
            data: &[u8],
            target_size: f32,
        ) -> Result<Box<Shape>, NucleationError> {
            if !(target_size > 0.0) {
                return Err(NucleationError::InvalidArgument);
            }
            let mut model = crate::voxelize::MeshModel::from_glb_bytes(data)
                .map_err(|_| NucleationError::Parse)?;
            model.fit(target_size);
            Ok(Box::new(Shape(crate::building::ShapeEnum::Mesh(
                crate::voxelize::MeshShape::new(model),
            ))))
        }

        /// Load a Wavefront OBJ (`v`/`vt`/`f` lines; polygon faces are
        /// fan-triangulated, negative indices supported, materials ignored)
        /// and voxelize it into a fillable Shape, fitted exactly like
        /// `shape_from_glb`. Errors with `Parse` on malformed/triangle-less
        /// OBJ and `InvalidArgument` on invalid UTF-8 or a non-positive
        /// `target_size`.
        pub fn shape_from_obj(
            text: &DiplomatStr,
            target_size: f32,
        ) -> Result<Box<Shape>, NucleationError> {
            let text =
                std::str::from_utf8(text).map_err(|_| NucleationError::InvalidArgument)?;
            if !(target_size > 0.0) {
                return Err(NucleationError::InvalidArgument);
            }
            let mut model = crate::voxelize::MeshModel::from_obj_str(text)
                .map_err(|_| NucleationError::Parse)?;
            model.fit(target_size);
            Ok(Box::new(Shape(crate::building::ShapeEnum::Mesh(
                crate::voxelize::MeshShape::new(model),
            ))))
        }

        /// Load a binary glTF and voxelize it directly into a textured
        /// schematic named `name`: every solid voxel becomes the `palette`
        /// block closest to its nearest-surface texture color (interior
        /// voxels inherit the nearest surface color; voxels without texture
        /// info snap to mid-gray). Errors with `Parse` on malformed GLB and
        /// `InvalidArgument` on invalid UTF-8 or a non-positive
        /// `target_size`.
        pub fn schematic_from_glb_textured(
            data: &[u8],
            target_size: f32,
            palette: &Palette,
            name: &DiplomatStr,
        ) -> Result<Box<Schematic>, NucleationError> {
            let name =
                std::str::from_utf8(name).map_err(|_| NucleationError::InvalidArgument)?;
            if !(target_size > 0.0) {
                return Err(NucleationError::InvalidArgument);
            }
            let mut model = crate::voxelize::MeshModel::from_glb_bytes(data)
                .map_err(|_| NucleationError::Parse)?;
            model.fit(target_size);
            let shape = crate::voxelize::MeshShape::new(model);
            let schematic = crate::voxelize::voxelize_textured(&shape, &palette.0, name);
            Ok(Box::new(Schematic(schematic)))
        }
    }
}
