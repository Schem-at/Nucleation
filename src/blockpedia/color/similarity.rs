use super::ExtendedColorData;

/// Calculate color similarity using different metrics
pub struct ColorSimilarity;

impl ColorSimilarity {
    /// Calculate Delta E CIE76 distance
    pub fn delta_e_cie76(color1: &ExtendedColorData, color2: &ExtendedColorData) -> f32 {
        let dl = color1.lab[0] - color2.lab[0];
        let da = color1.lab[1] - color2.lab[1];
        let db = color1.lab[2] - color2.lab[2];
        (dl * dl + da * da + db * db).sqrt()
    }

    /// Calculate Oklab distance (perceptually uniform)
    pub fn oklab_distance(color1: &ExtendedColorData, color2: &ExtendedColorData) -> f32 {
        color1.distance_oklab(color2)
    }

    /// Calculate RGB Euclidean distance
    pub fn rgb_distance(color1: &ExtendedColorData, color2: &ExtendedColorData) -> f32 {
        color1.distance_rgb(color2)
    }

    /// Calculate HSL distance
    pub fn hsl_distance(color1: &ExtendedColorData, color2: &ExtendedColorData) -> f32 {
        let dh = (color1.hsl[0] - color2.hsl[0]).min(360.0 - (color1.hsl[0] - color2.hsl[0]).abs());
        let ds = color1.hsl[1] - color2.hsl[1];
        let dl = color1.hsl[2] - color2.hsl[2];

        // Weight hue less when saturation is low
        let hue_weight = (color1.hsl[1] + color2.hsl[1]) / 2.0;
        let weighted_dh = dh * hue_weight;

        (weighted_dh * weighted_dh + ds * ds + dl * dl).sqrt()
    }

    /// Find the most similar color from a list
    pub fn find_most_similar(
        target: &ExtendedColorData,
        candidates: &[ExtendedColorData],
        metric: SimilarityMetric,
    ) -> Option<(usize, f32)> {
        candidates
            .iter()
            .enumerate()
            .map(|(i, candidate)| {
                let distance = match metric {
                    SimilarityMetric::Oklab => Self::oklab_distance(target, candidate),
                    SimilarityMetric::RGB => Self::rgb_distance(target, candidate),
                    SimilarityMetric::Lab => Self::delta_e_cie76(target, candidate),
                    SimilarityMetric::HSL => Self::hsl_distance(target, candidate),
                };
                (i, distance)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SimilarityMetric {
    Oklab,
    RGB,
    Lab,
    HSL,
}
