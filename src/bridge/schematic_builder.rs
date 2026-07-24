//! Text-template schematic builder. Port of `ffi/schematic_builder.rs`.
//!
//! Omitted from port: `schematicbuilder_free` — destructor is generated.
//! Omitted from port: `schematicbuilder_build_with_error` — error-string transport
//! variant of `schematicbuilder_build`; obsolete by construction (`build` returns
//! `Result` and error transport is generated).

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Fluent builder for schematics from character-mapped text layers.
    ///
    /// `build` is consuming (PORTING rule 11): the inner builder is held in an
    /// `Option` and taken on `build`; every method afterwards returns
    /// `AlreadyConsumed`.
    #[diplomat::opaque_mut]
    pub struct SchematicBuilder(Option<crate::SchematicBuilder>);

    impl SchematicBuilder {
        /// Create a new builder, pre-loaded with the standard Unicode palette
        /// (override any character via `map` or `palette`).
        pub fn create() -> Box<SchematicBuilder> {
            Box::new(SchematicBuilder(Some(crate::SchematicBuilder::new())))
        }

        /// Parse a builder from the canonical template text format.
        pub fn from_template(
            template: &DiplomatStr,
        ) -> Result<Box<SchematicBuilder>, NucleationError> {
            let template = Self::utf8(template)?;
            crate::SchematicBuilder::from_template(template)
                .map(|b| Box::new(SchematicBuilder(Some(b))))
                .map_err(|_| NucleationError::Parse)
        }

        /// Set the schematic name.
        pub fn name(&mut self, name: &DiplomatStr) -> Result<(), NucleationError> {
            let name = Self::utf8(name)?.to_owned();
            self.apply(|b| b.name(name))
        }

        /// Map a palette character to a block string. `ch` must contain exactly
        /// one character (its first char is used).
        pub fn map(
            &mut self,
            ch: &DiplomatStr,
            block: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let ch = Self::utf8(ch)?
                .chars()
                .next()
                .ok_or(NucleationError::InvalidArgument)?;
            let block = Self::utf8(block)?.to_owned();
            self.apply(|b| b.map(ch, block))
        }

        /// Append layers. `layers_json` is a JSON array of arrays of row strings,
        /// e.g. `[["ab","cd"],["ef","gh"]]`.
        pub fn layers(&mut self, layers_json: &DiplomatStr) -> Result<(), NucleationError> {
            let json = Self::utf8(layers_json)?;
            let layers: Vec<Vec<String>> =
                serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
            self.apply(|b| {
                let layer_refs: Vec<Vec<&str>> = layers
                    .iter()
                    .map(|l| l.iter().map(|s| s.as_str()).collect())
                    .collect();
                let layer_slices: Vec<&[&str]> = layer_refs.iter().map(|v| v.as_slice()).collect();
                b.layers(&layer_slices)
            })
        }

        /// Append a single layer of rows. `rows_json` is a JSON array of strings,
        /// e.g. `["abc", "def"]`. Equivalent to a one-element layers array.
        pub fn layer(&mut self, rows_json: &DiplomatStr) -> Result<(), NucleationError> {
            let json = Self::utf8(rows_json)?;
            let rows: Vec<String> =
                serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
            self.apply(|b| {
                let row_refs: Vec<&str> = rows.iter().map(|s| s.as_str()).collect();
                b.layer(&row_refs)
            })
        }

        /// Bulk-register palette characters. `pairs_json` is a JSON array of
        /// `[char, block]` two-element arrays, e.g.
        /// `[["c", "minecraft:gray_concrete"], [" ", "minecraft:air"]]`.
        pub fn palette(&mut self, pairs_json: &DiplomatStr) -> Result<(), NucleationError> {
            let json = Self::utf8(pairs_json)?;
            let raw: Vec<(String, String)> =
                serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
            self.apply(|b| {
                let pairs: Vec<(char, &str)> = raw
                    .iter()
                    .filter_map(|(k, v)| k.chars().next().map(|c| (c, v.as_str())))
                    .collect();
                b.palette(&pairs)
            })
        }

        /// Set the build offset applied to every placed block.
        pub fn offset(&mut self, x: i32, y: i32, z: i32) -> Result<(), NucleationError> {
            self.apply(|b| b.offset(x, y, z))
        }

        /// Register the standard Unicode palette (redstone wires, repeaters,
        /// comparators, torches, blocks, air...; the same set `create`
        /// pre-loads), overwriting any clashing character mappings.
        pub fn use_standard_palette(&mut self) -> Result<(), NucleationError> {
            self.apply(|b| b.use_standard_palette())
        }

        /// Register the minimal palette (an essentials-only subset of the
        /// standard one), overwriting any clashing character mappings.
        pub fn use_minimal_palette(&mut self) -> Result<(), NucleationError> {
            self.apply(|b| b.use_minimal_palette())
        }

        /// Register the compact ASCII-only palette, overwriting any clashing
        /// character mappings.
        pub fn use_compact_palette(&mut self) -> Result<(), NucleationError> {
            self.apply(|b| b.use_compact_palette())
        }

        /// Run pre-build validation without consuming the builder.
        pub fn validate(&self) -> Result<(), NucleationError> {
            let inner = self.0.as_ref().ok_or(NucleationError::AlreadyConsumed)?;
            inner
                .validate()
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Serialize the builder back into the canonical template format.
        pub fn to_template(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let inner = self.0.as_ref().ok_or(NucleationError::AlreadyConsumed)?;
            let _ = write!(out, "{}", inner.to_template());
            Ok(())
        }

        /// Build the schematic. Consuming: the builder cannot be reused afterwards
        /// (subsequent calls return `AlreadyConsumed`), including after a failed
        /// build.
        pub fn build(&mut self) -> Result<Box<Schematic>, NucleationError> {
            let inner = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            inner
                .build()
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::InvalidArgument)
        }

        // -- private helpers (ignored by Diplomat: not `pub`) --

        fn utf8(s: &[u8]) -> Result<&str, NucleationError> {
            std::str::from_utf8(s).map_err(|_| NucleationError::InvalidArgument)
        }

        /// The underlying builder's fluent methods consume `self`; take the inner
        /// value out of the `Option`, transform it, and put it back.
        fn apply(
            &mut self,
            f: impl FnOnce(crate::SchematicBuilder) -> crate::SchematicBuilder,
        ) -> Result<(), NucleationError> {
            let inner = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(f(inner));
            Ok(())
        }
    }
}
