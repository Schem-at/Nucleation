//! Auto-stack: periodicity detection + resize. Port of `ffi/autostack.rs`.

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Free functions in the old ABI hung directly off `schematic_*`; here they get a
    /// namespacing opaque-less home via static methods on a zero-size type is not
    /// supported, so they live as static methods taking `&Schematic` explicitly.
    #[diplomat::opaque]
    pub struct Autostack;

    impl Autostack {
        /// Detect repeating structures (region coverage). Writes a JSON array string;
        /// each element has `mode`, `vectors`, `coverage`, `region_min`/`region_max`,
        /// `cell_min`/`cell_max`, `label`.
        pub fn detect_structures(schematic: &Schematic, out: &mut DiplomatWrite) {
            let _ = write!(
                out,
                "{}",
                crate::autostack::detect_structures_json(&schematic.0)
            );
        }

        /// Graph-based detection: recovers diagonal lattice periods via the redstone
        /// logic graph. Writes `"[]"` for non-redstone builds. Requires the
        /// `simulation` feature; writes `"[]"` when compiled without it.
        pub fn detect_structures_graph(schematic: &Schematic, out: &mut DiplomatWrite) {
            #[cfg(feature = "simulation")]
            {
                let _ = write!(
                    out,
                    "{}",
                    crate::autostack::detect_structures_graph_json(&schematic.0)
                );
            }
            #[cfg(not(feature = "simulation"))]
            {
                let _ = write!(out, "[]");
            }
        }

        /// Resize a 1D / diagonal structure along its period vector.
        pub fn resize_1d(
            schematic: &Schematic,
            vx: i32,
            vy: i32,
            vz: i32,
            units: u32,
        ) -> Result<Box<Schematic>, NucleationError> {
            crate::autostack::resize_1d(&schematic.0, [vx, vy, vz], units as usize)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Resize a 2D structure to `n1`×`n2` cells along the two period vectors.
        #[allow(clippy::too_many_arguments)]
        pub fn resize_2d(
            schematic: &Schematic,
            v1x: i32,
            v1y: i32,
            v1z: i32,
            v2x: i32,
            v2y: i32,
            v2z: i32,
            n1: u32,
            n2: u32,
        ) -> Result<Box<Schematic>, NucleationError> {
            crate::autostack::resize_2d(
                &schematic.0,
                [v1x, v1y, v1z],
                [v2x, v2y, v2z],
                n1 as usize,
                n2 as usize,
            )
            .map(|s| Box::new(Schematic(s)))
            .map_err(|_| NucleationError::InvalidArgument)
        }
    }
}
