//! Pilot for a generated-bindings replacement of the hand-written `ffi`/`wasm`/`python`
//! layers, built on [Diplomat](https://github.com/rust-diplomat/diplomat) (forked at
//! `github.com/Nano112/diplomat` to add a PHP backend). See
//! `/Users/harrison/code/stencil` for the codegen pipeline and generated output, and
//! `stencil/docs/nucleation-error.md` for why `NucleationError` looks the way it does.
//!
//! This module is additive and intentionally small: it wraps just enough of
//! [`crate::UniversalSchematic`] to prove the pipeline against the real crate (not a
//! toy), not to replace `ffi::SchematicWrapper`. Nothing here is called by, or affects,
//! the existing `ffi`/`wasm`/`python`/`php` features.

#[diplomat::bridge]
pub mod ffi {
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Every fallible method in this module returns `Result<T, NucleationError>` —
    /// see `stencil/docs/nucleation-error.md` for how these variants were derived from
    /// the three error conventions the hand-written `ffi` module mixes today.
    #[diplomat::attr(auto, error)]
    pub enum NucleationError {
        NullArgument,
        InvalidArgument,
        Parse,
        Serialize,
        Io,
        Lock,
        Store,
        Mesh,
        Render,
        Simulation,
        AlreadyConsumed,
        NotFound,
    }

    #[diplomat::attr(auto, abi_compatible)]
    #[derive(Copy, Clone)]
    pub struct Dimensions {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }

    #[diplomat::opaque_mut]
    pub struct Schematic(crate::UniversalSchematic);

    impl Schematic {
        pub fn create(name: &DiplomatStr) -> Box<Schematic> {
            Box::new(Schematic(crate::UniversalSchematic::new(
                String::from_utf8_lossy(name).into_owned(),
            )))
        }

        pub fn dimensions(&self) -> Dimensions {
            let (x, y, z) = self.0.get_dimensions();
            Dimensions { x, y, z }
        }

        /// Returns `true` if a block was placed (mirrors the plain `UniversalSchematic`
        /// return value; out-of-range coordinates just extend the schematic rather than
        /// erroring, matching today's `set_block` behavior).
        pub fn set_block(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            block_name: &DiplomatStr,
        ) -> Result<bool, NucleationError> {
            let name = std::str::from_utf8(block_name).map_err(|_| NucleationError::InvalidArgument)?;
            Ok(self.0.set_block(x, y, z, &crate::BlockState::new(name)))
        }

        pub fn get_block_name(
            &self,
            x: i32,
            y: i32,
            z: i32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            match self.0.get_block(x, y, z) {
                Some(state) => {
                    let _ = write!(out, "{}", state.name);
                    Ok(())
                }
                None => Err(NucleationError::NotFound),
            }
        }

        pub fn save_to_file(&self, path: &DiplomatStr) -> Result<(), NucleationError> {
            let path = std::str::from_utf8(path).map_err(|_| NucleationError::InvalidArgument)?;
            let bytes = crate::litematic::to_litematic(&self.0)
                .map_err(|_| NucleationError::Serialize)?;
            std::fs::write(path, bytes).map_err(|_| NucleationError::Io)?;
            Ok(())
        }

        pub fn load_from_file(path: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            let path = std::str::from_utf8(path).map_err(|_| NucleationError::InvalidArgument)?;
            let bytes = std::fs::read(path).map_err(|_| NucleationError::Io)?;
            let inner =
                crate::litematic::from_litematic(&bytes).map_err(|_| NucleationError::Parse)?;
            Ok(Box::new(Schematic(inner)))
        }
    }
}
