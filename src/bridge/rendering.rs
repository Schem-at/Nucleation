//! GPU rendering of schematics to images. Port of `ffi/rendering.rs`
//! (`rendering_ffi`). The whole module is feature-gated behind `rendering` in
//! `bridge/mod.rs`, mirroring the old gating.
//!
//! The resource pack crosses as raw pack-zip bytes on each render call (the old
//! ABI took a `FFIResourcePack*` from the meshing module; the meshing domain and
//! its `ResourcePack` opaque are ported separately, so these methods parse the
//! pack per call to stay self-contained — they can later grow overloads taking
//! the meshing bridge's `ResourcePack` opaque directly).
//!
//! Omitted from port: `renderconfig_free` — destructor is generated.
//! Omitted from port: `render_pixels_free` — buffer memory management is
//! obsolete; pixel/PNG bytes cross as base64 strings (PORTING rule 6).

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use base64::Engine;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Camera / output configuration for rendering.
    #[diplomat::opaque_mut]
    pub struct RenderConfig(pub(crate) crate::rendering::RenderConfig);

    impl RenderConfig {
        /// Create a config with the given output size in pixels. Camera starts
        /// at the defaults: yaw 45°, pitch 30°, zoom 1.0, fov 45°, perspective
        /// projection, default sky background.
        pub fn create(width: u32, height: u32) -> Box<RenderConfig> {
            Box::new(RenderConfig(crate::rendering::RenderConfig {
                width,
                height,
                ..crate::rendering::RenderConfig::default()
            }))
        }

        /// Set the camera yaw (horizontal orbit angle) in degrees. Default: 45.
        pub fn set_yaw(&mut self, yaw: f32) {
            self.0.yaw = yaw;
        }

        /// Set the camera pitch (downward tilt) in degrees. Default: 30.
        pub fn set_pitch(&mut self, pitch: f32) {
            self.0.pitch = pitch;
        }

        /// Set the zoom factor applied to the auto-fitted framing
        /// (1.0 = frame the whole model; 2.0 = twice as close; 0.5 = twice
        /// as far). Default: 1.0.
        pub fn set_zoom(&mut self, zoom: f32) {
            self.0.zoom = zoom;
        }

        /// Fit the camera to the model's bounding sphere instead of its
        /// yaw-dependent silhouette. The sphere is rotation invariant, so
        /// orbiting cameras (turntables) keep a constant distance instead
        /// of pulsing as the silhouette changes. Frames slightly looser
        /// than the default fit. Default: false.
        pub fn set_sphere_fit(&mut self, sphere_fit: bool) {
            self.0.sphere_fit = sphere_fit;
        }

        /// Set the vertical field of view in degrees (perspective projection
        /// only). Default: 45.
        pub fn set_fov(&mut self, fov: f32) {
            self.0.fov = fov;
        }

        /// Set a solid RGBA clear color (linear 0.0–1.0). Alpha < 1.0 yields a
        /// transparent PNG. Ignored when HDRI is enabled.
        pub fn set_background(&mut self, r: f32, g: f32, b: f32, a: f32) {
            self.0.background = Some([r, g, b, a]);
        }

        /// Clear the custom background — revert to default sky / HDRI.
        pub fn clear_background(&mut self) {
            self.0.background = None;
        }

        /// Configure a one-block world grid. Models are centred on integer
        /// schematic coordinates, so grid lines are placed on half-integer
        /// block boundaries automatically.
        pub fn set_grid(
            &mut self,
            half_extent: i32,
            spacing: i32,
            plane_y: f32,
            show_axes: bool,
            red: f32,
            green: f32,
            blue: f32,
            alpha: f32,
        ) {
            self.0.grid = Some(crate::rendering::GridConfig {
                half_extent,
                fit_to_bounds: false,
                margin: 1,
                spacing,
                plane_y,
                show_axes,
                line_rgba: [red, green, blue, alpha],
            });
        }

        /// Configure a compact grid fitted to half-integer block boundaries.
        pub fn set_fitted_grid(
            &mut self,
            margin: i32,
            spacing: i32,
            plane_y: f32,
            show_axes: bool,
            red: f32,
            green: f32,
            blue: f32,
            alpha: f32,
        ) {
            self.0.grid = Some(crate::rendering::GridConfig {
                half_extent: 1,
                fit_to_bounds: true,
                margin,
                spacing,
                plane_y,
                show_axes,
                line_rgba: [red, green, blue, alpha],
            });
        }

        pub fn clear_grid(&mut self) {
            self.0.grid = None;
        }

        /// Enable (`true`) or disable orthographic projection.
        pub fn set_orthographic(&mut self, orthographic: bool) {
            self.0.projection = if orthographic {
                crate::rendering::Projection::Orthographic
            } else {
                crate::rendering::Projection::Perspective
            };
        }

        /// Configure a true isometric view: orthographic at yaw 45° /
        /// pitch ≈35.264° (preserves the current width/height).
        pub fn set_isometric(&mut self) {
            let w = self.0.width;
            let h = self.0.height;
            let mut iso = crate::rendering::RenderConfig::isometric();
            iso.width = w;
            iso.height = h;
            self.0 = iso;
        }
    }

    /// Namespace type for the render entry points (PORTING rule 12).
    #[diplomat::opaque]
    pub struct Renderer;

    impl Renderer {
        fn mesh(
            schematic: &Schematic,
            pack_zip: &[u8],
        ) -> Result<crate::meshing::MeshOutput, NucleationError> {
            let pack = crate::meshing::ResourcePackSource::from_bytes(pack_zip)
                .map_err(|_| NucleationError::Parse)?;
            let mesh_config = crate::meshing::MeshConfig::default();
            schematic
                .0
                .to_mesh(&pack, &mesh_config)
                .map_err(|_| NucleationError::Mesh)
        }

        /// Render a schematic to raw RGBA pixel bytes, written as base64
        /// (PORTING rule 6). `pack_zip` is a resource-pack zip in memory.
        pub fn render_pixels_b64(
            schematic: &Schematic,
            pack_zip: &[u8],
            config: &RenderConfig,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let mesh = Self::mesh(schematic, pack_zip)?;
            let pixels = crate::rendering::render_meshes(&[mesh], &config.0, None)
                .map_err(|_| NucleationError::Render)?;
            let _ = write!(
                out,
                "{}",
                base64::engine::general_purpose::STANDARD.encode(&pixels)
            );
            Ok(())
        }

        /// Render a schematic to PNG bytes, written as base64 (PORTING rule 6).
        /// `pack_zip` is a resource-pack zip in memory.
        pub fn render_png_b64(
            schematic: &Schematic,
            pack_zip: &[u8],
            config: &RenderConfig,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let mesh = Self::mesh(schematic, pack_zip)?;
            let png = crate::rendering::render_meshes_png(&[mesh], &config.0, None)
                .map_err(|_| NucleationError::Render)?;
            let _ = write!(
                out,
                "{}",
                base64::engine::general_purpose::STANDARD.encode(&png)
            );
            Ok(())
        }

        /// Render a schematic to a PNG file at `path`.
        pub fn render_to_file(
            schematic: &Schematic,
            pack_zip: &[u8],
            config: &RenderConfig,
            path: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let path = std::str::from_utf8(path).map_err(|_| NucleationError::InvalidArgument)?;
            let pack = crate::meshing::ResourcePackSource::from_bytes(pack_zip)
                .map_err(|_| NucleationError::Parse)?;
            schematic
                .0
                .render_to_file(&pack, path, &config.0)
                .map_err(|_| NucleationError::Render)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ffi::RenderConfig;

    #[test]
    fn bridge_render_config_exposes_aligned_world_grid() {
        let mut config = RenderConfig::create(420, 420);
        config.set_grid(10, 1, 0.0, false, 0.42, 0.52, 0.60, 0.38);
        let grid = config.0.grid.unwrap();
        assert_eq!(grid.spacing, 1);
        assert_eq!(grid.plane_y, 0.0);
        assert_eq!(grid.line_rgba[3], 0.38);
    }
}
