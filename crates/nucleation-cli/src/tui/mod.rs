//! The ratatui shell: Browser → Inspector, plus the live Tests screen.
//!
//! Every screen draws from a plain state struct through a pure `draw`
//! function, so screens are unit-tested against `TestBackend` without a
//! terminal. The event loop, terminal setup/teardown and screen switching
//! live in [`app`].

pub(crate) mod app;
pub(crate) mod browser;
pub(crate) mod inspector;
#[cfg(test)]
mod screen_tests;
pub(crate) mod tests_screen;
pub(crate) mod voxel;

pub(crate) use app::{run, Screen};
