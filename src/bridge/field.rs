//! Geometry-neutral reusable scalar fields.

#[diplomat::bridge]
pub mod ffi {
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::{DiplomatStr, DiplomatWrite};
    use std::fmt::Write;

    /// The closed interval a field's values are analytically proven to lie in.
    #[diplomat::out]
    pub struct FieldRange {
        pub min: f32,
        pub max: f32,
    }

    /// Immutable scalar field evaluated over world-space `(x, y, z)`.
    ///
    /// A `Field3` has scalar semantics only. It may be shared by geometry and
    /// material consumers without being reinterpreted as a signed-distance field.
    #[diplomat::opaque]
    pub struct Field3(pub(crate) crate::field::Field3);

    impl Field3 {
        /// Deterministic value-noise FBM normalized to `[-1, 1]`.
        pub fn value_noise_fbm(
            frequency: f32,
            seed: i32,
            octaves: u32,
        ) -> Result<Box<Field3>, NucleationError> {
            crate::field::Field3::value_noise_fbm(frequency, seed, octaves)
                .map(|field| Box::new(Field3(field)))
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn eval_at(&self, x: f32, y: f32, z: f32) -> f32 {
            self.0.eval(x, y, z)
        }

        /// The field's analytically proven output range.
        ///
        /// Returns `NotFound` when no range can be proven — callers mapping a
        /// field onto a gradient must handle that rather than silently
        /// propagating a sentinel into their `lo`/`hi` bounds.
        pub fn output_range(&self) -> Result<FieldRange, NucleationError> {
            self.0
                .output_range()
                .map(|range| FieldRange {
                    min: range[0],
                    max: range[1],
                })
                .ok_or(NucleationError::NotFound)
        }

        pub fn from_json_string(json: &DiplomatStr) -> Result<Box<Field3>, NucleationError> {
            let json = std::str::from_utf8(json).map_err(|_| NucleationError::InvalidArgument)?;
            crate::field::Field3::from_json(json)
                .map(|field| Box::new(Field3(field)))
                .map_err(|_| NucleationError::Parse)
        }

        pub fn to_json(&self, write: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let json = self.0.to_json().map_err(|_| NucleationError::Serialize)?;
            let _ = write!(write, "{json}");
            Ok(())
        }
    }
}
