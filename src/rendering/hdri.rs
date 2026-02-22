//! HDRI environment map loading.

use super::RenderError;

/// HDRI environment map data (RGBA f32 pixels).
pub struct HdriData {
    pub width: u32,
    pub height: u32,
    /// RGBA f32 pixel data (4 floats per pixel)
    pub pixels_rgba32f: Vec<f32>,
}

/// Load an HDRI environment map from a file path.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_hdri(path: &str) -> Result<HdriData, RenderError> {
    let data = std::fs::read(path).map_err(RenderError::Io)?;
    load_hdri_from_bytes(&data)
}

/// Load an HDRI environment map from bytes.
pub fn load_hdri_from_bytes(data: &[u8]) -> Result<HdriData, RenderError> {
    let img = image::load_from_memory(data)
        .map_err(|e| RenderError::RenderFailed(format!("Failed to load HDRI: {}", e)))?;
    let rgb32f = img.to_rgb32f();
    let (w, h) = (rgb32f.width(), rgb32f.height());

    // Convert RGB32F → RGBA32F (add alpha=1.0)
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for pixel in rgb32f.pixels() {
        rgba.push(pixel[0]);
        rgba.push(pixel[1]);
        rgba.push(pixel[2]);
        rgba.push(1.0);
    }

    Ok(HdriData {
        width: w,
        height: h,
        pixels_rgba32f: rgba,
    })
}
