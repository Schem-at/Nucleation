//! The standard image/video rendering command.
pub(crate) fn render_main(args: impl Iterator<Item = String>) {
    super::video::render_main(args);
}
