//! Types shared by every bridge module: the unified error enum and small POD structs.

#[diplomat::bridge]
pub mod ffi {
    /// Every fallible method in the bridge returns `Result<T, NucleationError>` —
    /// see `stencil/docs/nucleation-error.md` for how these variants were derived from
    /// the three error conventions the old hand-written `ffi` module mixed.
    #[diplomat::attr(auto, error)]
    #[derive(PartialEq, Eq, Debug)]
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
        /// A world-generation source failed while producing a chunk, even though
        /// the request itself was well-formed (see `world_generation`).
        Generation,
    }

    impl NucleationError {
        /// Why the last failing bridge call on this thread failed, in words.
        ///
        /// The enum cannot carry a message across the FFI, so a caught error
        /// is a bare variant — `InvalidArgument` — while the layer that
        /// refused already knew it was "19.2M cells over the 8M cap". Modules
        /// that know the story record it; this reads it back, so an exception
        /// handler holding the error value can ask it for the words. Empty
        /// when the last detail-carrying call succeeded.
        pub fn detail(self, out: &mut diplomat_runtime::DiplomatWrite) {
            use std::fmt::Write;
            let _ = write!(out, "{}", crate::bridge::last_error_detail());
        }
    }

    #[diplomat::attr(auto, abi_compatible)]
    #[derive(Copy, Clone)]
    pub struct Dimensions {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }

    #[diplomat::attr(auto, abi_compatible)]
    #[derive(Copy, Clone)]
    pub struct BlockPos {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }
}
