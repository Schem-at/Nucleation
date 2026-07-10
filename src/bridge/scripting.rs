//! Embedded scripting (Lua / JS) entry points. Port of `ffi/scripting.rs`.
//!
//! The module as a whole is gated in `bridge/mod.rs` behind
//! `any(feature = "scripting-lua", feature = "scripting-js")`, mirroring the old
//! per-function gating; within the module, the engine-specific methods keep a stable
//! signature and fail with `InvalidArgument` when their engine is compiled out
//! (feature-dependent behavior lives in the body, per the porting rules).

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;

    /// Namespace for the script-runner free functions of the old ABI
    /// (`run_lua_script`, `run_js_script`, `run_script`).
    #[diplomat::opaque]
    pub struct Scripting;

    impl Scripting {
        /// Run a Lua script file. Returns the schematic the script assigns to
        /// `result`; `NotFound` if it produced none, `Parse` if it failed, and
        /// `InvalidArgument` when built without the `scripting-lua` feature.
        pub fn run_lua_script(path: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            let path = std::str::from_utf8(path).map_err(|_| NucleationError::InvalidArgument)?;
            #[cfg(feature = "scripting-lua")]
            {
                match crate::scripting::lua_engine::run_lua_script(path) {
                    Ok(Some(ss)) => Ok(Box::new(Schematic(ss.inner))),
                    Ok(None) => Err(NucleationError::NotFound),
                    Err(_) => Err(NucleationError::Parse),
                }
            }
            #[cfg(not(feature = "scripting-lua"))]
            {
                let _ = path;
                Err(NucleationError::InvalidArgument)
            }
        }

        /// Run a JS script file. Returns the schematic the script assigns to
        /// `result`; `NotFound` if it produced none, `Parse` if it failed, and
        /// `InvalidArgument` when built without the `scripting-js` feature.
        pub fn run_js_script(path: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            let path = std::str::from_utf8(path).map_err(|_| NucleationError::InvalidArgument)?;
            #[cfg(feature = "scripting-js")]
            {
                match crate::scripting::js_engine::run_js_script(path) {
                    Ok(Some(ss)) => Ok(Box::new(Schematic(ss.inner))),
                    Ok(None) => Err(NucleationError::NotFound),
                    Err(_) => Err(NucleationError::Parse),
                }
            }
            #[cfg(not(feature = "scripting-js"))]
            {
                let _ = path;
                Err(NucleationError::InvalidArgument)
            }
        }

        /// Run a script file, auto-detecting the engine by extension (`.lua` or
        /// `.js`). Returns the schematic the script assigns to `result`; `NotFound`
        /// if it produced none, `Parse` if it failed (including unsupported
        /// extensions).
        pub fn run_script(path: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            let path = std::str::from_utf8(path).map_err(|_| NucleationError::InvalidArgument)?;
            match crate::scripting::run_script(path) {
                Ok(Some(ss)) => Ok(Box::new(Schematic(ss.inner))),
                Ok(None) => Err(NucleationError::NotFound),
                Err(_) => Err(NucleationError::Parse),
            }
        }
    }
}
