use anyhow::{Context, Result};
use image::DynamicImage;
use std::collections::HashMap;
use std::path::Path;

use super::{extract_dominant_color_from_image, ExtendedColorData};

/// Extract colors from multiple images using different methods
pub struct ColorExtractor {
    extraction_method: ExtractionMethod,
}

/// Different methods for color extraction
#[derive(Debug, Clone)]
pub enum ExtractionMethod {
    /// Simple average of all pixels
    Average,
    /// Most frequent color (quantized)
    MostFrequent { bins: u8 },
    /// K-means clustering dominant color
    Clustering { k: u8 },
    /// Edge-weighted average (ignores edges)
    EdgeWeighted,
    /// Alpha-weighted average: every pixel contributes proportionally to its
    /// alpha value instead of using a hard opacity cutoff. This is the
    /// estimator used to build the shipped color cache.
    AlphaWeighted,
}

impl ColorExtractor {
    pub fn new(method: ExtractionMethod) -> Self {
        Self {
            extraction_method: method,
        }
    }

    /// Extract color from an image using the configured method
    pub fn extract_color(&self, img: &DynamicImage) -> Result<ExtendedColorData> {
        match &self.extraction_method {
            ExtractionMethod::Average => self.extract_average_color(img),
            ExtractionMethod::MostFrequent { bins } => self.extract_most_frequent_color(img, *bins),
            ExtractionMethod::Clustering { k } => self.extract_clustered_color(img, *k),
            ExtractionMethod::EdgeWeighted => self.extract_edge_weighted_color(img),
            ExtractionMethod::AlphaWeighted => self.extract_alpha_weighted_color(img),
        }
    }

    /// Alpha-weighted average color extraction.
    ///
    /// Pixels are weighted by their alpha channel (a/255), so anti-aliased
    /// and semi-transparent texels (leaves, glass, plants) contribute
    /// proportionally instead of being kept/dropped by a hard threshold.
    fn extract_alpha_weighted_color(&self, img: &DynamicImage) -> Result<ExtendedColorData> {
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        // Animated textures are vertical strips of square frames; only use
        // the first frame so every animation phase doesn't skew the average.
        let frame_height = if height > width && height % width == 0 {
            width
        } else {
            height
        };

        let mut r_sum = 0.0f64;
        let mut g_sum = 0.0f64;
        let mut b_sum = 0.0f64;
        let mut weight_sum = 0.0f64;

        for y in 0..frame_height {
            for x in 0..width {
                let [r, g, b, a] = rgba_img.get_pixel(x, y).0;
                let w = a as f64 / 255.0;
                r_sum += r as f64 * w;
                g_sum += g as f64 * w;
                b_sum += b as f64 * w;
                weight_sum += w;
            }
        }

        if weight_sum <= 0.0 {
            anyhow::bail!("Texture is fully transparent");
        }

        Ok(ExtendedColorData::from_rgb(
            (r_sum / weight_sum).round() as u8,
            (g_sum / weight_sum).round() as u8,
            (b_sum / weight_sum).round() as u8,
        ))
    }

    /// Simple average color extraction
    fn extract_average_color(&self, img: &DynamicImage) -> Result<ExtendedColorData> {
        extract_dominant_color_from_image(img)
    }

    /// Extract most frequent color by quantizing the color space
    fn extract_most_frequent_color(
        &self,
        img: &DynamicImage,
        bins: u8,
    ) -> Result<ExtendedColorData> {
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        let bin_size = 256 / bins as u32;

        let mut color_counts: HashMap<(u8, u8, u8), u32> = HashMap::new();

        for y in 0..height {
            for x in 0..width {
                let pixel = rgba_img.get_pixel(x, y);
                let [r, g, b, a] = pixel.0;

                // Skip transparent pixels
                if a > 128 {
                    // Quantize colors
                    let r_bin = ((r as u32 / bin_size) * bin_size).min(255) as u8;
                    let g_bin = ((g as u32 / bin_size) * bin_size).min(255) as u8;
                    let b_bin = ((b as u32 / bin_size) * bin_size).min(255) as u8;

                    *color_counts.entry((r_bin, g_bin, b_bin)).or_insert(0) += 1;
                }
            }
        }

        // Find most frequent color
        let most_frequent = color_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .context("No colors found in image")?;

        let (r, g, b) = most_frequent.0;
        Ok(ExtendedColorData::from_rgb(*r, *g, *b))
    }

    /// Simple k-means clustering (simplified version)
    fn extract_clustered_color(&self, img: &DynamicImage, _k: u8) -> Result<ExtendedColorData> {
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        // Collect all opaque pixels
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let pixel = rgba_img.get_pixel(x, y);
                let [r, g, b, a] = pixel.0;
                if a > 128 {
                    pixels.push([r as f32, g as f32, b as f32]);
                }
            }
        }

        if pixels.is_empty() {
            anyhow::bail!("No opaque pixels found in image");
        }

        // Simple k-means (just return average for now, can be improved)
        let mut r_sum = 0.0;
        let mut g_sum = 0.0;
        let mut b_sum = 0.0;

        for pixel in &pixels {
            r_sum += pixel[0];
            g_sum += pixel[1];
            b_sum += pixel[2];
        }

        let count = pixels.len() as f32;
        let avg_r = (r_sum / count) as u8;
        let avg_g = (g_sum / count) as u8;
        let avg_b = (b_sum / count) as u8;

        Ok(ExtendedColorData::from_rgb(avg_r, avg_g, avg_b))
    }

    /// Edge-weighted color extraction (avoids edges)
    fn extract_edge_weighted_color(&self, img: &DynamicImage) -> Result<ExtendedColorData> {
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        // Skip edge pixels
        let margin = (width.min(height) / 8).max(1);

        let mut r_sum = 0u64;
        let mut g_sum = 0u64;
        let mut b_sum = 0u64;
        let mut pixel_count = 0u64;

        for y in margin..(height - margin) {
            for x in margin..(width - margin) {
                let pixel = rgba_img.get_pixel(x, y);
                let [r, g, b, a] = pixel.0;

                if a > 128 {
                    r_sum += r as u64;
                    g_sum += g as u64;
                    b_sum += b as u64;
                    pixel_count += 1;
                }
            }
        }

        if pixel_count == 0 {
            // Fallback to full image if margin is too large
            return self.extract_average_color(img);
        }

        let avg_r = (r_sum / pixel_count) as u8;
        let avg_g = (g_sum / pixel_count) as u8;
        let avg_b = (b_sum / pixel_count) as u8;

        Ok(ExtendedColorData::from_rgb(avg_r, avg_g, avg_b))
    }
}

/// Extract colors from multiple texture variants of the same block
pub fn extract_block_color_variants(
    block_name: &str,
    texture_paths: &[&Path],
) -> Result<ExtendedColorData> {
    if texture_paths.is_empty() {
        anyhow::bail!("No texture paths provided for block: {}", block_name);
    }

    let extractor = ColorExtractor::new(ExtractionMethod::Average);
    let mut colors = Vec::new();

    for path in texture_paths {
        match image::open(path) {
            Ok(img) => {
                if let Ok(color) = extractor.extract_color(&img) {
                    colors.push(color);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to process texture {:?}: {}", path, e);
            }
        }
    }

    if colors.is_empty() {
        anyhow::bail!("No valid textures found for block: {}", block_name);
    }

    // Average all the colors
    let r_avg = colors.iter().map(|c| c.rgb[0] as u32).sum::<u32>() / colors.len() as u32;
    let g_avg = colors.iter().map(|c| c.rgb[1] as u32).sum::<u32>() / colors.len() as u32;
    let b_avg = colors.iter().map(|c| c.rgb[2] as u32).sum::<u32>() / colors.len() as u32;

    Ok(ExtendedColorData::from_rgb(
        r_avg as u8,
        g_avg as u8,
        b_avg as u8,
    ))
}
