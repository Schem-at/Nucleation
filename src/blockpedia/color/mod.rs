#[cfg(feature = "mc-data-refresh")]
use anyhow::{Context, Result};
#[cfg(feature = "mc-data-refresh")]
use image::{DynamicImage, Rgba};
use palette::{IntoColor, Lab, Oklch, Srgb};
#[cfg(feature = "mc-data-refresh")]
use std::path::Path;

pub mod block_palettes;
// Texture color extraction needs the `image` crate; only the data-refresh
// tooling reads textures, so keep it (and `image`) out of normal builds.
#[cfg(feature = "mc-data-refresh")]
pub mod extraction;
pub mod palettes;
pub mod similarity;
pub mod spaces;
pub mod texture_mapping;

/// Extended color data structure supporting multiple color spaces
#[derive(Debug, Clone, Copy)]
pub struct ExtendedColorData {
    pub rgb: [u8; 3],
    pub oklab: [f32; 3],
    pub hsl: [f32; 3],
    pub lab: [f32; 3],
    pub oklch: [f32; 3],
    pub hex: u32,
}

impl ExtendedColorData {
    /// Create ExtendedColorData from RGB values
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let rgb = [r, g, b];

        // Convert to normalized RGB for palette crate
        let srgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);

        // Convert to various color spaces
        let lab: Lab = srgb.into_color();
        let oklch: Oklch = srgb.into_color();

        // Simple HSL conversion
        let hsl = rgb_to_hsl(r, g, b);

        // Simple Oklab conversion (matching existing code)
        let oklab = rgb_to_oklab_simple(rgb);

        // Convert to Lab values
        let lab_values = [lab.l, lab.a, lab.b];

        // Convert to Oklch values
        let oklch_values = [oklch.l, oklch.chroma, oklch.hue.into_positive_degrees()];

        // Hex representation
        let hex = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);

        ExtendedColorData {
            rgb,
            oklab,
            hsl,
            lab: lab_values,
            oklch: oklch_values,
            hex,
        }
    }

    /// Get hex string representation
    pub fn hex_string(&self) -> String {
        format!("#{:06X}", self.hex)
    }

    /// Calculate distance between two colors in Oklab space
    pub fn distance_oklab(&self, other: &ExtendedColorData) -> f32 {
        let dl = self.oklab[0] - other.oklab[0];
        let da = self.oklab[1] - other.oklab[1];
        let db = self.oklab[2] - other.oklab[2];
        (dl * dl + da * da + db * db).sqrt()
    }

    /// Calculate distance between two colors in RGB space
    pub fn distance_rgb(&self, other: &ExtendedColorData) -> f32 {
        let dr = (self.rgb[0] as f32) - (other.rgb[0] as f32);
        let dg = (self.rgb[1] as f32) - (other.rgb[1] as f32);
        let db = (self.rgb[2] as f32) - (other.rgb[2] as f32);
        (dr * dr + dg * dg + db * db).sqrt()
    }
}

/// Extract dominant color from an image
#[cfg(feature = "mc-data-refresh")]
pub fn extract_dominant_color(image_path: &Path) -> Result<ExtendedColorData> {
    let img = image::open(image_path)
        .with_context(|| format!("Failed to open image: {:?}", image_path))?;

    extract_dominant_color_from_image(&img)
}

/// Extract dominant color from an image buffer
#[cfg(feature = "mc-data-refresh")]
pub fn extract_dominant_color_from_image(img: &DynamicImage) -> Result<ExtendedColorData> {
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();

    // Simple average color extraction (can be improved with clustering)
    let mut r_sum = 0u64;
    let mut g_sum = 0u64;
    let mut b_sum = 0u64;
    let mut pixel_count = 0u64;

    for y in 0..height {
        for x in 0..width {
            let pixel = rgba_img.get_pixel(x, y);
            let Rgba([r, g, b, a]) = *pixel;

            // Skip transparent pixels
            if a > 128 {
                r_sum += r as u64;
                g_sum += g as u64;
                b_sum += b as u64;
                pixel_count += 1;
            }
        }
    }

    if pixel_count == 0 {
        anyhow::bail!("No opaque pixels found in image");
    }

    let avg_r = (r_sum / pixel_count) as u8;
    let avg_g = (g_sum / pixel_count) as u8;
    let avg_b = (b_sum / pixel_count) as u8;

    Ok(ExtendedColorData::from_rgb(avg_r, avg_g, avg_b))
}

/// Simple RGB to HSL conversion
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> [f32; 3] {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    // Lightness
    let l = (max + min) / 2.0;

    if delta == 0.0 {
        // Achromatic
        return [0.0, 0.0, l];
    }

    // Saturation
    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    // Hue
    let h = if max == r {
        ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };

    [h * 360.0, s, l]
}

/// Simple RGB to Oklab conversion (matching existing build script)
fn rgb_to_oklab_simple(rgb: [u8; 3]) -> [f32; 3] {
    let r = rgb[0] as f32 / 255.0;
    let g = rgb[1] as f32 / 255.0;
    let b = rgb[2] as f32 / 255.0;
    let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let a = (r - g) * 0.5;
    let b_val = (r + g - 2.0 * b) * 0.25;
    [l, a, b_val]
}
