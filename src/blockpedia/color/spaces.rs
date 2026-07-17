use super::ExtendedColorData;

/// Convert between different color spaces
pub struct ColorSpaceConverter;

impl ColorSpaceConverter {
    /// Convert RGB to all supported color spaces
    pub fn convert_rgb_to_all(r: u8, g: u8, b: u8) -> ExtendedColorData {
        ExtendedColorData::from_rgb(r, g, b)
    }

    /// Parse hex color string to RGB
    pub fn hex_to_rgb(hex: &str) -> Result<[u8; 3], &'static str> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err("Hex color must be 6 characters");
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex digits")?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex digits")?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex digits")?;

        Ok([r, g, b])
    }

    /// Convert HSV to RGB
    pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r_prime, g_prime, b_prime) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let r = ((r_prime + m) * 255.0) as u8;
        let g = ((g_prime + m) * 255.0) as u8;
        let b = ((b_prime + m) * 255.0) as u8;

        [r, g, b]
    }
}
