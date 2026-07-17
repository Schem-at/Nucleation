use super::ExtendedColorData;

/// Generate color palettes from block collections
pub struct PaletteGenerator;

/// Gradient interpolation methods
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientMethod {
    /// Linear interpolation in RGB space
    LinearRgb,
    /// Linear interpolation in HSL space
    LinearHsl,
    /// Linear interpolation in Oklab space (perceptually uniform)
    LinearOklab,
    /// Cubic bezier interpolation
    CubicBezier,
}

/// Represents a color gradient between two or more points
#[derive(Debug, Clone)]
pub struct ColorGradient {
    pub colors: Vec<ExtendedColorData>,
    pub method: GradientMethod,
    pub steps: usize,
}

impl ColorGradient {
    /// Create a new gradient between two colors
    pub fn new_two_color(
        start: ExtendedColorData,
        end: ExtendedColorData,
        steps: usize,
        method: GradientMethod,
    ) -> Self {
        Self {
            colors: vec![start, end],
            method,
            steps,
        }
    }

    /// Create a new gradient from multiple colors
    pub fn new_multi_color(
        colors: Vec<ExtendedColorData>,
        steps: usize,
        method: GradientMethod,
    ) -> Self {
        Self {
            colors,
            method,
            steps,
        }
    }

    /// Generate the gradient palette
    pub fn generate(&self) -> Vec<ExtendedColorData> {
        if self.colors.len() < 2 {
            return self.colors.clone();
        }

        let mut palette = Vec::with_capacity(self.steps);

        // For multi-color gradients, interpolate between segments
        let segments = self.colors.len() - 1;
        let steps_per_segment = self.steps / segments;

        for seg in 0..segments {
            let start = self.colors[seg];
            let end = self.colors[seg + 1];

            let segment_steps = if seg == segments - 1 {
                // Last segment gets any remaining steps
                self.steps - (seg * steps_per_segment)
            } else {
                steps_per_segment
            };

            for i in 0..segment_steps {
                let t = if segment_steps == 1 {
                    0.0
                } else {
                    i as f32 / (segment_steps - 1) as f32
                };
                let color = self.interpolate_colors(start, end, t);
                palette.push(color);
            }
        }

        // Ensure we include the final color
        if palette.len() < self.steps {
            palette.push(*self.colors.last().unwrap());
        }

        palette
    }

    /// Interpolate between two colors using the specified method
    fn interpolate_colors(
        &self,
        start: ExtendedColorData,
        end: ExtendedColorData,
        t: f32,
    ) -> ExtendedColorData {
        match self.method {
            GradientMethod::LinearRgb => self.interpolate_rgb(start, end, t),
            GradientMethod::LinearHsl => self.interpolate_hsl(start, end, t),
            GradientMethod::LinearOklab => self.interpolate_oklab(start, end, t),
            GradientMethod::CubicBezier => self.interpolate_cubic_bezier(start, end, t),
        }
    }

    /// Linear interpolation in RGB space
    fn interpolate_rgb(
        &self,
        start: ExtendedColorData,
        end: ExtendedColorData,
        t: f32,
    ) -> ExtendedColorData {
        let r = (start.rgb[0] as f32 * (1.0 - t) + end.rgb[0] as f32 * t) as u8;
        let g = (start.rgb[1] as f32 * (1.0 - t) + end.rgb[1] as f32 * t) as u8;
        let b = (start.rgb[2] as f32 * (1.0 - t) + end.rgb[2] as f32 * t) as u8;
        ExtendedColorData::from_rgb(r, g, b)
    }

    /// Linear interpolation in HSL space
    fn interpolate_hsl(
        &self,
        start: ExtendedColorData,
        end: ExtendedColorData,
        t: f32,
    ) -> ExtendedColorData {
        let h = self.interpolate_hue(start.hsl[0], end.hsl[0], t);
        let s = start.hsl[1] * (1.0 - t) + end.hsl[1] * t;
        let l = start.hsl[2] * (1.0 - t) + end.hsl[2] * t;

        let rgb = hsl_to_rgb(h, s, l);
        ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2])
    }

    /// Linear interpolation in Oklab space (perceptually uniform)
    fn interpolate_oklab(
        &self,
        start: ExtendedColorData,
        end: ExtendedColorData,
        t: f32,
    ) -> ExtendedColorData {
        let l = start.oklab[0] * (1.0 - t) + end.oklab[0] * t;
        let a = start.oklab[1] * (1.0 - t) + end.oklab[1] * t;
        let b = start.oklab[2] * (1.0 - t) + end.oklab[2] * t;

        let rgb = oklab_to_rgb([l, a, b]);
        ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2])
    }

    /// Cubic bezier interpolation for smoother gradients
    fn interpolate_cubic_bezier(
        &self,
        start: ExtendedColorData,
        end: ExtendedColorData,
        t: f32,
    ) -> ExtendedColorData {
        // Use cubic bezier curve for smoother interpolation
        let t_smooth = t * t * (3.0 - 2.0 * t); // Smoothstep function
        self.interpolate_rgb(start, end, t_smooth)
    }

    /// Interpolate hue values, taking the shortest path around the color wheel
    fn interpolate_hue(&self, start_hue: f32, end_hue: f32, t: f32) -> f32 {
        let mut diff = end_hue - start_hue;

        // Take the shortest path around the color wheel
        if diff > 180.0 {
            diff -= 360.0;
        } else if diff < -180.0 {
            diff += 360.0;
        }

        let result = start_hue + diff * t;
        if result < 0.0 {
            result + 360.0
        } else if result >= 360.0 {
            result - 360.0
        } else {
            result
        }
    }
}

/// Helper function to convert HSL to RGB
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
    let h = h / 360.0; // Normalize hue to 0-1

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    [
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    ]
}

/// Helper function to convert Oklab to RGB (simplified)
#[allow(clippy::manual_clamp, clippy::excessive_precision)] // Scientific precision required for color conversion
fn oklab_to_rgb(oklab: [f32; 3]) -> [u8; 3] {
    // Simplified Oklab to RGB conversion
    // In a production system, you'd want to use a proper color space library
    let l = oklab[0].clamp(0.0, 1.0);
    let a = oklab[1];
    let b = oklab[2];

    // Approximate conversion (this is simplified)
    let r = (l + 0.3963377774 * a + 0.2158037573 * b).max(0.0).min(1.0);
    let g = (l - 0.1055613458 * a - 0.0638541728 * b).max(0.0).min(1.0);
    let b_val = (l - 0.0894841775 * a - 1.2914855480 * b).max(0.0).min(1.0);

    [(r * 255.0) as u8, (g * 255.0) as u8, (b_val * 255.0) as u8]
}

impl PaletteGenerator {
    /// Generate a palette of the most distinct colors
    pub fn generate_distinct_palette(
        colors: &[ExtendedColorData],
        max_colors: usize,
    ) -> Vec<ExtendedColorData> {
        if colors.len() <= max_colors {
            return colors.to_vec();
        }

        let mut palette = Vec::with_capacity(max_colors);
        let mut remaining = colors.to_vec();

        // Start with the first color
        if let Some(first) = remaining.pop() {
            palette.push(first);
        }

        // Greedily select colors that are most different from existing palette
        while palette.len() < max_colors && !remaining.is_empty() {
            let mut best_color = remaining[0];
            let mut best_distance = 0.0;
            let mut best_index = 0;

            for (i, candidate) in remaining.iter().enumerate() {
                let min_distance = palette
                    .iter()
                    .map(|p| candidate.distance_oklab(p))
                    .fold(f32::INFINITY, f32::min);

                if min_distance > best_distance {
                    best_distance = min_distance;
                    best_color = *candidate;
                    best_index = i;
                }
            }

            palette.push(best_color);
            remaining.remove(best_index);
        }

        palette
    }

    /// Generate a palette sorted by hue
    pub fn generate_hue_sorted_palette(colors: &[ExtendedColorData]) -> Vec<ExtendedColorData> {
        let mut sorted_colors = colors.to_vec();
        sorted_colors.sort_by(|a, b| {
            a.hsl[0]
                .partial_cmp(&b.hsl[0])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_colors
    }

    /// Generate a palette sorted by lightness
    pub fn generate_lightness_sorted_palette(
        colors: &[ExtendedColorData],
    ) -> Vec<ExtendedColorData> {
        let mut sorted_colors = colors.to_vec();
        sorted_colors.sort_by(|a, b| {
            a.hsl[2]
                .partial_cmp(&b.hsl[2])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_colors
    }

    /// Generate a palette of complementary colors for a given color
    pub fn generate_complementary_palette(
        base_color: &ExtendedColorData,
    ) -> Vec<ExtendedColorData> {
        let mut palette = vec![*base_color];

        // Complementary (180° hue shift)
        let comp_hue = (base_color.hsl[0] + 180.0) % 360.0;
        let comp_rgb = hsl_to_rgb(comp_hue, base_color.hsl[1], base_color.hsl[2]);
        let complementary = ExtendedColorData::from_rgb(comp_rgb[0], comp_rgb[1], comp_rgb[2]);
        palette.push(complementary);

        // Triadic (120° shifts)
        for offset in [120.0, 240.0] {
            let triadic_hue = (base_color.hsl[0] + offset) % 360.0;
            let rgb = hsl_to_rgb(triadic_hue, base_color.hsl[1], base_color.hsl[2]);
            palette.push(ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2]));
        }

        palette
    }

    /// Export palette to various formats
    pub fn export_palette_css(palette: &[ExtendedColorData]) -> String {
        let mut css = String::from(":root {\n");
        for (i, color) in palette.iter().enumerate() {
            css.push_str(&format!("  --color-{}: {};\n", i + 1, color.hex_string()));
        }
        css.push_str("}\n");
        css
    }

    /// Generate a gradient palette between two colors
    pub fn generate_gradient_palette(
        start: ExtendedColorData,
        end: ExtendedColorData,
        steps: usize,
        method: GradientMethod,
    ) -> Vec<ExtendedColorData> {
        let gradient = ColorGradient::new_two_color(start, end, steps, method);
        gradient.generate()
    }

    /// Generate a gradient palette between multiple colors
    pub fn generate_multi_gradient_palette(
        colors: Vec<ExtendedColorData>,
        steps: usize,
        method: GradientMethod,
    ) -> Vec<ExtendedColorData> {
        let gradient = ColorGradient::new_multi_color(colors, steps, method);
        gradient.generate()
    }

    /// Generate a gradient palette between two blocks
    pub fn generate_block_gradient_palette(
        block1_color: ExtendedColorData,
        block2_color: ExtendedColorData,
        steps: usize,
        method: GradientMethod,
    ) -> Vec<ExtendedColorData> {
        Self::generate_gradient_palette(block1_color, block2_color, steps, method)
    }

    /// Generate a sunset/sunrise gradient palette
    pub fn generate_sunset_palette(steps: usize) -> Vec<ExtendedColorData> {
        let colors = vec![
            ExtendedColorData::from_rgb(255, 94, 77),  // Red
            ExtendedColorData::from_rgb(255, 154, 0),  // Orange
            ExtendedColorData::from_rgb(255, 206, 84), // Yellow
            ExtendedColorData::from_rgb(163, 94, 195), // Purple
            ExtendedColorData::from_rgb(25, 25, 112),  // Midnight blue
        ];
        Self::generate_multi_gradient_palette(colors, steps, GradientMethod::LinearOklab)
    }

    /// Generate a ocean depth gradient palette
    pub fn generate_ocean_palette(steps: usize) -> Vec<ExtendedColorData> {
        let colors = vec![
            ExtendedColorData::from_rgb(135, 206, 235), // Light blue
            ExtendedColorData::from_rgb(0, 119, 190),   // Medium blue
            ExtendedColorData::from_rgb(0, 82, 164),    // Dark blue
            ExtendedColorData::from_rgb(0, 39, 77),     // Deep blue
            ExtendedColorData::from_rgb(0, 20, 40),     // Abyss
        ];
        Self::generate_multi_gradient_palette(colors, steps, GradientMethod::LinearOklab)
    }

    /// Generate a forest gradient palette
    pub fn generate_forest_palette(steps: usize) -> Vec<ExtendedColorData> {
        let colors = vec![
            ExtendedColorData::from_rgb(173, 255, 47), // Light green
            ExtendedColorData::from_rgb(50, 205, 50),  // Lime green
            ExtendedColorData::from_rgb(34, 139, 34),  // Forest green
            ExtendedColorData::from_rgb(0, 100, 0),    // Dark green
            ExtendedColorData::from_rgb(25, 25, 25),   // Almost black
        ];
        Self::generate_multi_gradient_palette(colors, steps, GradientMethod::LinearOklab)
    }

    /// Generate a fire gradient palette
    pub fn generate_fire_palette(steps: usize) -> Vec<ExtendedColorData> {
        let colors = vec![
            ExtendedColorData::from_rgb(255, 255, 0), // Yellow
            ExtendedColorData::from_rgb(255, 165, 0), // Orange
            ExtendedColorData::from_rgb(255, 69, 0),  // Red-orange
            ExtendedColorData::from_rgb(220, 20, 60), // Crimson
            ExtendedColorData::from_rgb(139, 0, 0),   // Dark red
        ];
        Self::generate_multi_gradient_palette(colors, steps, GradientMethod::LinearRgb)
    }

    /// Generate a monochrome gradient palette
    pub fn generate_monochrome_palette(
        base_color: ExtendedColorData,
        steps: usize,
    ) -> Vec<ExtendedColorData> {
        let black = ExtendedColorData::from_rgb(0, 0, 0);
        let white = ExtendedColorData::from_rgb(255, 255, 255);

        let half_steps = steps / 2;
        let mut palette = Vec::new();

        // Dark to base color
        let dark_gradient = Self::generate_gradient_palette(
            black,
            base_color,
            half_steps,
            GradientMethod::LinearOklab,
        );
        palette.extend(dark_gradient);

        // Base color to light
        let light_gradient = Self::generate_gradient_palette(
            base_color,
            white,
            steps - half_steps,
            GradientMethod::LinearOklab,
        );
        palette.extend(light_gradient.into_iter().skip(1)); // Skip duplicate base color

        palette
    }

    /// Export palette to Photoshop ACO format (simplified)
    pub fn export_palette_aco_data(palette: &[ExtendedColorData]) -> Vec<u8> {
        let mut data = Vec::new();

        // ACO header (version 1)
        data.extend_from_slice(&1u16.to_be_bytes()); // version
        data.extend_from_slice(&(palette.len() as u16).to_be_bytes()); // color count

        for color in palette {
            data.extend_from_slice(&0u16.to_be_bytes()); // RGB color space
            data.extend_from_slice(&((color.rgb[0] as u16) << 8).to_be_bytes()); // R
            data.extend_from_slice(&((color.rgb[1] as u16) << 8).to_be_bytes()); // G
            data.extend_from_slice(&((color.rgb[2] as u16) << 8).to_be_bytes()); // B
            data.extend_from_slice(&0u16.to_be_bytes()); // Unused
        }

        data
    }

    /// Export palette to GIMP GPL format
    pub fn export_palette_gpl(palette: &[ExtendedColorData], name: &str) -> String {
        let mut gpl = String::new();
        gpl.push_str("GIMP Palette\n");
        gpl.push_str(&format!("Name: {}\n", name));
        gpl.push_str("Columns: 0\n");
        gpl.push_str("#\n");

        for (i, color) in palette.iter().enumerate() {
            gpl.push_str(&format!(
                "{:3} {:3} {:3} Color {}\n",
                color.rgb[0],
                color.rgb[1],
                color.rgb[2],
                i + 1
            ));
        }

        gpl
    }
}
