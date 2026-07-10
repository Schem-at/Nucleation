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
