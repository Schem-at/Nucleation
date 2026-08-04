//! One module per subcommand. Shared discovery/loading helpers live in
//! [`test`], which the other commands import.

pub(crate) mod convert;
pub(crate) mod diff;
pub(crate) mod info;
pub(crate) mod port;
#[cfg(feature = "render")]
pub(crate) mod render;
pub(crate) mod test;
