use anyhow::Result;
use std::collections::HashMap;

/// Trait for fetchers that can add extra data to blocks during build time
pub trait ExtraFetcher {
    /// Fetch additional data for blocks
    /// Returns a map of block_id -> extra data blob
    fn fetch(&self) -> Result<HashMap<String, ExtraBlob>>;

    /// Merge the extra data blob into the block's extras field
    fn merge(extras: &mut crate::blockpedia::Extras, blob: &ExtraBlob)
    where
        Self: Sized;

    /// Optional: Register custom query functions using this fetcher's data
    fn register_queries(&self) -> Vec<Box<dyn QueryHelper>> {
        Vec::new()
    }

    /// Name of this fetcher for identification
    fn name(&self) -> &'static str;
}

/// Build-time registry for custom queries
pub trait QueryHelper {
    /// Function name to generate
    fn function_name(&self) -> &'static str;

    /// Generate the function body for code generation
    fn generate_code(&self) -> String;

    /// Dependencies this query helper needs (imports, etc.)
    fn dependencies(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Different types of extra data that fetchers can provide
#[derive(Debug, Clone)]
pub enum ExtraBlob {
    /// Mock data for testing
    Mock { test_value: i32 },

    /// Color data that could be added by a color fetcher
    Color { rgb: [u8; 3], oklab: [f32; 3] },

    /// Texture information
    Texture {
        texture_path: String,
        animated: bool,
    },

    /// Custom string data
    Custom(String),

    /// Raw JSON data for maximum flexibility
    Json(serde_json::Value),
}

/// Example mock fetcher for testing the framework
pub struct MockFetcher;

impl ExtraFetcher for MockFetcher {
    fn fetch(&self) -> Result<HashMap<String, ExtraBlob>> {
        let mut map = HashMap::new();

        // Add mock data to some blocks
        map.insert(
            "minecraft:stone".to_string(),
            ExtraBlob::Mock { test_value: 42 },
        );

        map.insert(
            "minecraft:repeater".to_string(),
            ExtraBlob::Mock { test_value: 123 },
        );

        Ok(map)
    }

    fn merge(extras: &mut crate::blockpedia::Extras, blob: &ExtraBlob) {
        if let ExtraBlob::Mock { test_value } = blob {
            extras.mock_data = Some(*test_value);
        }
    }

    fn register_queries(&self) -> Vec<Box<dyn QueryHelper>> {
        vec![Box::new(MockQueryHelper)]
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}

/// Example query helper for mock data
pub struct MockQueryHelper;

impl QueryHelper for MockQueryHelper {
    fn function_name(&self) -> &'static str {
        "find_blocks_with_mock_data"
    }

    fn generate_code(&self) -> String {
        r#"
/// Find blocks that have mock data with a specific value
pub fn find_blocks_with_mock_data(value: i32) -> impl Iterator<Item = &'static BlockFacts> {
    BLOCKS.values().filter(move |block| {
        block.extras.mock_data == Some(value)
    }).map(|block| *block)
}
"#
        .to_string()
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["use crate::blockpedia::{BLOCKS, BlockFacts};".to_string()]
    }
}

/// Example color fetcher that reads from a CSV file
pub struct ColorFetcher {
    csv_data: String,
}

impl Default for ColorFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorFetcher {
    pub fn new() -> Self {
        // In a real implementation, this would read from a bundled CSV file
        let csv_data = r#"block_id,r,g,b
minecraft:stone,125,125,125
minecraft:dirt,134,96,67
minecraft:grass_block,93,178,75
"#
        .to_string();

        ColorFetcher { csv_data }
    }

    fn parse_csv(&self) -> Result<HashMap<String, [u8; 3]>> {
        let mut colors = HashMap::new();

        for (i, line) in self.csv_data.lines().enumerate() {
            if i == 0 {
                continue;
            } // Skip header

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let block_id = parts[0].to_string();
                let r = parts[1].parse::<u8>().unwrap_or(0);
                let g = parts[2].parse::<u8>().unwrap_or(0);
                let b = parts[3].parse::<u8>().unwrap_or(0);

                colors.insert(block_id, [r, g, b]);
            }
        }

        Ok(colors)
    }
}

impl ExtraFetcher for ColorFetcher {
    fn fetch(&self) -> Result<HashMap<String, ExtraBlob>> {
        let colors = self.parse_csv()?;
        let mut result = HashMap::new();

        for (block_id, rgb) in colors {
            // Convert RGB to Oklab (simplified conversion for demo)
            let oklab = rgb_to_oklab_simplified(rgb);

            result.insert(block_id, ExtraBlob::Color { rgb, oklab });
        }

        Ok(result)
    }

    fn merge(extras: &mut crate::blockpedia::Extras, blob: &ExtraBlob) {
        if let ExtraBlob::Color { rgb, oklab } = blob {
            extras.color = Some(crate::blockpedia::ColorData {
                rgb: *rgb,
                oklab: *oklab,
            });
        }
    }

    fn register_queries(&self) -> Vec<Box<dyn QueryHelper>> {
        vec![Box::new(ClosestColorQuery), Box::new(ColorRangeQuery)]
    }

    fn name(&self) -> &'static str {
        "color"
    }
}

/// Simplified RGB to Oklab conversion for demo purposes
fn rgb_to_oklab_simplified(rgb: [u8; 3]) -> [f32; 3] {
    // This is a very simplified conversion - in reality you'd want a proper color space conversion
    let r = rgb[0] as f32 / 255.0;
    let g = rgb[1] as f32 / 255.0;
    let b = rgb[2] as f32 / 255.0;

    // Fake Oklab values for demonstration
    let l = (r * 0.299 + g * 0.587 + b * 0.114) * 100.0; // Simplified lightness
    let a = (r - g) * 50.0; // Simplified a channel
    let b_channel = (g - b) * 50.0; // Simplified b channel

    [l, a, b_channel]
}

/// Query helper for finding closest color
pub struct ClosestColorQuery;

impl QueryHelper for ClosestColorQuery {
    fn function_name(&self) -> &'static str {
        "closest_block_by_color"
    }

    fn generate_code(&self) -> String {
        r#"
/// Find the block with color closest to the target RGB values
pub fn closest_block_by_color(target_rgb: [u8; 3]) -> Option<&'static BlockFacts> {
    let mut closest_block = None;
    let mut closest_distance = f32::INFINITY;
    
    for block in BLOCKS.values() {
        if let Some(color_data) = &block.extras.color {
            let distance = color_distance(target_rgb, color_data.rgb);
            if distance < closest_distance {
                closest_distance = distance;
                closest_block = Some(*block);
            }
        }
    }
    
    closest_block
}

fn color_distance(a: [u8; 3], b: [u8; 3]) -> f32 {
    let dr = a[0] as f32 - b[0] as f32;
    let dg = a[1] as f32 - b[1] as f32;
    let db = a[2] as f32 - b[2] as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}
"#
        .to_string()
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["use crate::blockpedia::{BLOCKS, BlockFacts};".to_string()]
    }
}

/// Query helper for color range searches
pub struct ColorRangeQuery;

impl QueryHelper for ColorRangeQuery {
    fn function_name(&self) -> &'static str {
        "blocks_in_color_range"
    }

    fn generate_code(&self) -> String {
        r#"
/// Find blocks within a color range
pub fn blocks_in_color_range(center_rgb: [u8; 3], tolerance: f32) -> impl Iterator<Item = &'static BlockFacts> {
    BLOCKS.values().filter(move |block| {
        if let Some(color_data) = &block.extras.color {
            let distance = color_distance(center_rgb, color_data.rgb);
            distance <= tolerance
        } else {
            false
        }
    }).map(|block| *block)
}
"#.to_string()
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["use crate::blockpedia::{BLOCKS, BlockFacts};".to_string()]
    }
}

/// Registry for managing fetchers during build time
pub struct FetcherRegistry {
    fetchers: Vec<Box<dyn ExtraFetcher>>,
}

impl FetcherRegistry {
    pub fn new() -> Self {
        FetcherRegistry {
            fetchers: Vec::new(),
        }
    }

    pub fn register<F: ExtraFetcher + 'static>(mut self, fetcher: F) -> Self {
        self.fetchers.push(Box::new(fetcher));
        self
    }

    pub fn fetch_all(&self) -> Result<HashMap<String, Vec<ExtraBlob>>> {
        let mut all_data: HashMap<String, Vec<ExtraBlob>> = HashMap::new();

        for fetcher in &self.fetchers {
            let fetcher_data = fetcher.fetch()?;
            for (block_id, blob) in fetcher_data {
                all_data.entry(block_id).or_default().push(blob);
            }
        }

        Ok(all_data)
    }

    pub fn get_query_helpers(&self) -> Vec<Box<dyn QueryHelper>> {
        let mut all_helpers = Vec::new();

        for fetcher in &self.fetchers {
            let mut helpers = fetcher.register_queries();
            all_helpers.append(&mut helpers);
        }

        all_helpers
    }

    pub fn get_fetchers(&self) -> &[Box<dyn ExtraFetcher>] {
        &self.fetchers
    }
}

impl Default for FetcherRegistry {
    fn default() -> Self {
        Self::new()
    }
}
