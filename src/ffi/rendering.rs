use super::*;

// =============================================================================
// Rendering FFI (feature-gated)
// =============================================================================

#[cfg(feature = "rendering")]
#[allow(unused_imports)]
pub mod rendering_ffi {
    use super::*;
    use crate::rendering::{self, RenderConfig};

    // Re-use the FFI resource pack wrapper from meshing_ffi
    // (rendering implies meshing, so meshing_ffi is always available)
    use crate::ffi::meshing::FFIResourcePack;

    // --- RenderConfig Wrapper ---

    pub struct FFIRenderConfig(RenderConfig);

    #[no_mangle]
    pub extern "C" fn renderconfig_new(width: u32, height: u32) -> *mut FFIRenderConfig {
        Box::into_raw(Box::new(FFIRenderConfig(RenderConfig {
            width,
            height,
            ..RenderConfig::default()
        })))
    }

    #[no_mangle]
    pub extern "C" fn renderconfig_free(ptr: *mut FFIRenderConfig) {
        if !ptr.is_null() {
            unsafe { drop(Box::from_raw(ptr)) };
        }
    }

    #[no_mangle]
    pub extern "C" fn renderconfig_set_yaw(ptr: *mut FFIRenderConfig, yaw: f32) {
        if !ptr.is_null() {
            unsafe { (*ptr).0.yaw = yaw };
        }
    }

    #[no_mangle]
    pub extern "C" fn renderconfig_set_pitch(ptr: *mut FFIRenderConfig, pitch: f32) {
        if !ptr.is_null() {
            unsafe { (*ptr).0.pitch = pitch };
        }
    }

    #[no_mangle]
    pub extern "C" fn renderconfig_set_zoom(ptr: *mut FFIRenderConfig, zoom: f32) {
        if !ptr.is_null() {
            unsafe { (*ptr).0.zoom = zoom };
        }
    }

    #[no_mangle]
    pub extern "C" fn renderconfig_set_fov(ptr: *mut FFIRenderConfig, fov: f32) {
        if !ptr.is_null() {
            unsafe { (*ptr).0.fov = fov };
        }
    }

    /// Set a solid RGBA clear color (linear 0.0–1.0). Alpha < 1.0 yields a
    /// transparent PNG. Ignored when HDRI is enabled.
    #[no_mangle]
    pub extern "C" fn renderconfig_set_background(
        ptr: *mut FFIRenderConfig,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) {
        if !ptr.is_null() {
            unsafe { (*ptr).0.background = Some([r, g, b, a]) };
        }
    }

    /// Clear the custom background — revert to default sky / HDRI.
    #[no_mangle]
    pub extern "C" fn renderconfig_clear_background(ptr: *mut FFIRenderConfig) {
        if !ptr.is_null() {
            unsafe { (*ptr).0.background = None };
        }
    }

    /// Enable (`true`) or disable orthographic projection.
    #[no_mangle]
    pub extern "C" fn renderconfig_set_orthographic(ptr: *mut FFIRenderConfig, orthographic: bool) {
        if !ptr.is_null() {
            unsafe {
                (*ptr).0.projection = if orthographic {
                    rendering::Projection::Orthographic
                } else {
                    rendering::Projection::Perspective
                };
            }
        }
    }

    /// Configure a true isometric view: orthographic at yaw 45° / pitch ≈35.264°
    /// (preserves the current width/height).
    #[no_mangle]
    pub extern "C" fn renderconfig_set_isometric(ptr: *mut FFIRenderConfig) {
        if !ptr.is_null() {
            unsafe {
                let w = (*ptr).0.width;
                let h = (*ptr).0.height;
                let mut iso = rendering::RenderConfig::isometric();
                iso.width = w;
                iso.height = h;
                (*ptr).0 = iso;
            }
        }
    }

    // --- Render Functions ---

    /// Render a schematic to RGBA pixel bytes.
    /// On success, writes pixel data pointer to `out_data` and length to `out_len`.
    /// Returns 0 on success, -1 on error. Caller must free with `render_pixels_free`.
    #[no_mangle]
    pub extern "C" fn schematic_render(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIRenderConfig,
        out_data: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int {
        if schematic.is_null()
            || pack.is_null()
            || config.is_null()
            || out_data.is_null()
            || out_len.is_null()
        {
            return -1;
        }
        let schematic = unsafe { &*(*schematic).0 };
        let pack = unsafe { &(*pack).0 };
        let config = unsafe { &(*config).0 };

        let mesh_config = crate::meshing::MeshConfig::default();
        let mesh = match schematic.to_mesh(pack, &mesh_config) {
            Ok(m) => m,
            Err(e) => {
                set_last_error(format!("Mesh generation failed: {}", e));
                return -1;
            }
        };

        let render_config = config.clone();

        match rendering::render_meshes(&[mesh], &render_config, None) {
            Ok(pixels) => {
                let len = pixels.len();
                let boxed = pixels.into_boxed_slice();
                let ptr = Box::into_raw(boxed) as *mut u8;
                unsafe {
                    *out_data = ptr;
                    *out_len = len;
                }
                0
            }
            Err(e) => {
                set_last_error(format!("Render failed: {}", e));
                -1
            }
        }
    }

    /// Render a schematic to PNG bytes.
    /// On success, writes PNG data pointer to `out_data` and length to `out_len`.
    /// Returns 0 on success, -1 on error. Caller must free with `render_pixels_free`.
    #[no_mangle]
    pub extern "C" fn schematic_render_png(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIRenderConfig,
        out_data: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int {
        if schematic.is_null()
            || pack.is_null()
            || config.is_null()
            || out_data.is_null()
            || out_len.is_null()
        {
            return -1;
        }
        let schematic = unsafe { &*(*schematic).0 };
        let pack = unsafe { &(*pack).0 };
        let config = unsafe { &(*config).0 };

        let mesh_config = crate::meshing::MeshConfig::default();
        let mesh = match schematic.to_mesh(pack, &mesh_config) {
            Ok(m) => m,
            Err(e) => {
                set_last_error(format!("Mesh generation failed: {}", e));
                return -1;
            }
        };

        let render_config = config.clone();

        match rendering::render_meshes_png(&[mesh], &render_config, None) {
            Ok(png) => {
                let len = png.len();
                let boxed = png.into_boxed_slice();
                let ptr = Box::into_raw(boxed) as *mut u8;
                unsafe {
                    *out_data = ptr;
                    *out_len = len;
                }
                0
            }
            Err(e) => {
                set_last_error(format!("Render PNG failed: {}", e));
                -1
            }
        }
    }

    /// Render a schematic to a PNG file.
    /// Returns 0 on success, -1 on error.
    #[no_mangle]
    pub extern "C" fn schematic_render_to_file(
        schematic: *const SchematicWrapper,
        pack: *const FFIResourcePack,
        config: *const FFIRenderConfig,
        path: *const c_char,
    ) -> c_int {
        if schematic.is_null() || pack.is_null() || config.is_null() || path.is_null() {
            return -1;
        }
        let schematic = unsafe { &*(*schematic).0 };
        let pack = unsafe { &(*pack).0 };
        let config = unsafe { &(*config).0 };
        let path = match unsafe { CStr::from_ptr(path) }.to_str() {
            Ok(s) => s,
            Err(e) => {
                set_last_error(format!("Invalid path: {}", e));
                return -1;
            }
        };

        let render_config = config.clone();

        match schematic.render_to_file(pack, path, &render_config) {
            Ok(()) => 0,
            Err(e) => {
                set_last_error(format!("Render to file failed: {}", e));
                -1
            }
        }
    }

    /// Free pixel/PNG data returned by `schematic_render` or `schematic_render_png`.
    #[no_mangle]
    pub extern "C" fn render_pixels_free(data: *mut u8, len: usize) {
        if !data.is_null() && len > 0 {
            unsafe {
                drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(data, len)));
            }
        }
    }
}
