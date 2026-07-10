//! Diff + fingerprint. Port of `ffi/diff.rs`.
//!
//! Omitted from port: `diff_free` — destructor is generated.
//! Omitted from port: `diff_free_glb` — buffer memory management is obsolete; the
//! overlay GLB crosses as a base64 string (`to_overlay_glb_b64`, PORTING rule 6).

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Namespace type for the fingerprint free functions (PORTING rule 12).
    #[diplomat::opaque]
    pub struct Fingerprint;

    impl Fingerprint {
        fn spec(preset: &[u8]) -> Result<crate::fingerprint::FingerprintSpec, NucleationError> {
            let preset =
                std::str::from_utf8(preset).map_err(|_| NucleationError::InvalidArgument)?;
            crate::fingerprint::FingerprintSpec::from_preset(preset)
                .ok_or(NucleationError::InvalidArgument)
        }

        /// The fingerprint of a schematic for the given preset, as a hex string.
        /// Errors with `InvalidArgument` on an unknown preset.
        pub fn compute(
            schematic: &Schematic,
            preset: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let spec = Self::spec(preset)?;
            let hex = crate::fingerprint::fingerprint(&schematic.0, &spec).to_hex();
            let _ = write!(out, "{}", hex);
            Ok(())
        }

        /// The structural signature (JSON) of a schematic for the given preset.
        pub fn signature_json(
            schematic: &Schematic,
            preset: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let spec = Self::spec(preset)?;
            let json = crate::fingerprint::signature(&schematic.0, &spec).to_json();
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Translation-invariant fuzzy distance between two builds' footprints.
        pub fn footprint_distance(
            a: &Schematic,
            b: &Schematic,
            preset: &DiplomatStr,
        ) -> Result<f32, NucleationError> {
            let spec = Self::spec(preset)?;
            Ok(crate::fingerprint::footprint_distance(&a.0, &b.0, &spec))
        }

        /// The schematic's translation/scale-invariant FFT shape footprint as a
        /// JSON array of floats.
        pub fn footprint_json(
            schematic: &Schematic,
            preset: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let spec = Self::spec(preset)?;
            let v = crate::fingerprint::footprint(&schematic.0, &spec).0;
            let json = serde_json::to_string(&v).map_err(|_| NucleationError::Serialize)?;
            let _ = write!(out, "{}", json);
            Ok(())
        }

        /// Whether two schematics share the same fingerprint for the given preset.
        pub fn is_duplicate(
            a: &Schematic,
            b: &Schematic,
            preset: &DiplomatStr,
        ) -> Result<bool, NucleationError> {
            let spec = Self::spec(preset)?;
            Ok(crate::fingerprint::is_duplicate(&a.0, &b.0, &spec))
        }
    }

    /// A computed diff between two schematics.
    #[diplomat::opaque]
    pub struct Diff(pub(crate) crate::diff::Diff);

    impl Diff {
        /// Diff two schematics with the given preset (default cost model).
        /// Errors with `InvalidArgument` on an unknown preset.
        pub fn compute(
            a: &Schematic,
            b: &Schematic,
            preset: &DiplomatStr,
        ) -> Result<Box<Diff>, NucleationError> {
            Self::compute_with_opts(a, b, preset, -1, -1, -1, -1, b"")
        }

        /// Diff two schematics with optional cost/symmetry overrides. Negative
        /// cost ints mean "unset" (use the preset default); an empty `symmetry`
        /// string means "unset". Errors with `InvalidArgument` on an unknown
        /// preset or symmetry name.
        #[allow(clippy::too_many_arguments)]
        pub fn compute_with_opts(
            a: &Schematic,
            b: &Schematic,
            preset: &DiplomatStr,
            cost_add: i32,
            cost_delete: i32,
            cost_change: i32,
            cost_swap: i32,
            symmetry: &DiplomatStr,
        ) -> Result<Box<Diff>, NucleationError> {
            let preset =
                std::str::from_utf8(preset).map_err(|_| NucleationError::InvalidArgument)?;
            let symmetry =
                std::str::from_utf8(symmetry).map_err(|_| NucleationError::InvalidArgument)?;

            let mut overrides = crate::diff::SpecOverrides::default();
            if cost_add >= 0 {
                overrides.cost_add = Some(cost_add as u32);
            }
            if cost_delete >= 0 {
                overrides.cost_delete = Some(cost_delete as u32);
            }
            if cost_change >= 0 {
                overrides.cost_change = Some(cost_change as u32);
            }
            if cost_swap >= 0 {
                overrides.cost_swap = Some(cost_swap as u32);
            }
            if !symmetry.is_empty() {
                let sym = crate::fingerprint::symmetry::Symmetry::from_name(symmetry)
                    .ok_or(NucleationError::InvalidArgument)?;
                overrides.symmetry = Some(sym);
            }

            let spec = crate::diff::DiffSpec::resolve(preset, &overrides)
                .ok_or(NucleationError::InvalidArgument)?;
            Ok(Box::new(Diff(crate::diff::diff(&a.0, &b.0, &spec))))
        }

        /// Reconstruct a diff from its JSON representation.
        pub fn from_json(json: &DiplomatStr) -> Result<Box<Diff>, NucleationError> {
            let json = std::str::from_utf8(json).map_err(|_| NucleationError::InvalidArgument)?;
            crate::diff::Diff::from_json(json)
                .map(|d| Box::new(Diff(d)))
                .map_err(|_| NucleationError::Parse)
        }

        /// The edit distance of the diff.
        pub fn distance(&self) -> u64 {
            self.0.distance
        }

        /// The support (alignment confidence) of the diff.
        pub fn support(&self) -> f32 {
            self.0.support
        }

        /// Serialize the diff to its full JSON representation.
        pub fn to_json(&self, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", self.0.to_json());
        }

        /// Serialize the diff to its compact summary JSON.
        pub fn summary_json(&self, out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", self.0.summary_json());
        }

        /// A new schematic containing only the blocks added in this diff.
        pub fn added(&self) -> Box<Schematic> {
            Box::new(Schematic(self.0.added()))
        }

        /// A new schematic containing only the blocks removed in this diff.
        pub fn removed(&self) -> Box<Schematic> {
            Box::new(Schematic(self.0.removed()))
        }

        /// A new schematic containing only the blocks changed in this diff.
        pub fn changed(&self) -> Box<Schematic> {
            Box::new(Schematic(self.0.changed()))
        }

        /// A new schematic containing only the blocks swapped in this diff.
        pub fn swapped(&self) -> Box<Schematic> {
            Box::new(Schematic(self.0.swapped()))
        }

        /// A new schematic with marker blocks summarizing this diff.
        pub fn markers(&self) -> Box<Schematic> {
            Box::new(Schematic(self.0.markers()))
        }

        /// Render a diff overlay on top of an "after" GLB buffer, returning the
        /// new GLB as base64 (PORTING rule 6). Requires the `meshing` feature;
        /// errors with `Mesh` when compiled without it.
        pub fn to_overlay_glb_b64(
            &self,
            after_glb: &[u8],
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            #[cfg(feature = "meshing")]
            {
                use base64::Engine;
                let opts = crate::diff::OverlayOptions::default();
                let data = self
                    .0
                    .to_overlay_glb(after_glb, &opts)
                    .map_err(|_| NucleationError::Mesh)?;
                let _ = write!(
                    out,
                    "{}",
                    base64::engine::general_purpose::STANDARD.encode(&data)
                );
                Ok(())
            }
            #[cfg(not(feature = "meshing"))]
            {
                let _ = (after_glb, out);
                Err(NucleationError::Mesh)
            }
        }
    }
}
