//! One module per subcommand. Shared discovery/loading helpers live in
//! [`test`], which the other commands import.

#[cfg(any(feature = "mesh", feature = "render"))]
pub(crate) mod animate;
pub(crate) mod convert;
pub(crate) mod diff;
pub(crate) mod info;
pub(crate) mod io;
pub(crate) mod man;
#[cfg(any(feature = "mesh", feature = "render"))]
pub(crate) mod mesh;
#[cfg(any(feature = "mesh", feature = "render"))]
pub(crate) mod pack;
pub(crate) mod port;
pub(crate) mod scene;
pub(crate) mod sdf;
pub(crate) mod worldgen;
#[cfg(feature = "render")]
pub(crate) mod render;
#[cfg(feature = "render")]
pub(crate) mod video;
pub(crate) mod test;

pub(crate) mod geometry;
pub(crate) mod query;

pub(crate) mod door;
