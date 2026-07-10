//! SDF (signed distance field) sampling. Port of `ffi/sdf.rs`.

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;

    /// Namespace for the SDF free functions of the old ABI (`schematic_from_sdf`,
    /// `sdf_eval`).
    #[diplomat::opaque]
    pub struct Sdf;

    impl Sdf {
        /// Builds a schematic by sampling an SDF JSON tree with material rules JSON.
        /// When `has_bounds` is false the tree's own AABB is used (fails with
        /// `InvalidArgument` for unbounded trees) and the `min_*`/`max_*` arguments
        /// are ignored.
        #[allow(clippy::too_many_arguments)]
        pub fn schematic_from_sdf(
            sdf_json: &DiplomatStr,
            rules_json: &DiplomatStr,
            has_bounds: bool,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<Schematic>, NucleationError> {
            let sdf_str =
                std::str::from_utf8(sdf_json).map_err(|_| NucleationError::InvalidArgument)?;
            let rules_str =
                std::str::from_utf8(rules_json).map_err(|_| NucleationError::InvalidArgument)?;

            let node =
                crate::sdf::SdfNode::from_json(sdf_str).map_err(|_| NucleationError::Parse)?;
            let rules = crate::sdf::MaterialRules::from_json(rules_str)
                .map_err(|_| NucleationError::Parse)?;
            let bounds = if has_bounds {
                Some(crate::sdf::SampleBounds {
                    min: [min_x, min_y, min_z],
                    max: [max_x, max_y, max_z],
                })
            } else {
                None
            };
            crate::sdf::sample_to_schematic(&node, &rules, bounds, "sdf")
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Evaluates an SDF JSON tree at a point, returning the signed distance.
        pub fn eval(
            sdf_json: &DiplomatStr,
            x: f32,
            y: f32,
            z: f32,
        ) -> Result<f32, NucleationError> {
            let sdf_str =
                std::str::from_utf8(sdf_json).map_err(|_| NucleationError::InvalidArgument)?;
            let node =
                crate::sdf::SdfNode::from_json(sdf_str).map_err(|_| NucleationError::Parse)?;
            Ok(node.eval(x, y, z))
        }
    }
}
