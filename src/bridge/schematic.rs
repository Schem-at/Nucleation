//! The core `Schematic` opaque, wrapping [`crate::UniversalSchematic`].

#[diplomat::bridge]
pub mod ffi {
    use super::super::shared::ffi::{Dimensions, NucleationError};
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    #[diplomat::opaque_mut]
    pub struct Schematic(pub(crate) crate::UniversalSchematic);

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

        /// Returns `true` if a block was placed (out-of-range coordinates extend the
        /// schematic rather than erroring, matching `UniversalSchematic::set_block`).
        pub fn set_block(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            block_name: &DiplomatStr,
        ) -> Result<bool, NucleationError> {
            let name =
                std::str::from_utf8(block_name).map_err(|_| NucleationError::InvalidArgument)?;
            Ok(self.0.set_block_str(x, y, z, name))
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
            let bytes =
                crate::litematic::to_litematic(&self.0).map_err(|_| NucleationError::Serialize)?;
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
