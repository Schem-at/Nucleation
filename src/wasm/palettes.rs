use blockpedia::{all_blocks, BlockFacts};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PaletteManager;

#[wasm_bindgen]
impl PaletteManager {
    /// Get all wool blocks
    #[wasm_bindgen(js_name = getWoolBlocks)]
    pub fn get_wool_blocks() -> Vec<String> {
        Self::filter_blocks(|f| f.id.contains("wool"))
    }

    /// Get all concrete blocks
    #[wasm_bindgen(js_name = getConcreteBlocks)]
    pub fn get_concrete_blocks() -> Vec<String> {
        Self::filter_blocks(|f| f.id.contains("concrete") && !f.id.contains("powder"))
    }

    /// Get all terracotta blocks
    #[wasm_bindgen(js_name = getTerracottaBlocks)]
    pub fn get_terracotta_blocks() -> Vec<String> {
        Self::filter_blocks(|f| f.id.contains("terracotta") && !f.id.contains("glazed"))
    }

    /// Get a palette containing blocks matching ANY of the provided keywords
    /// Example: `["wool", "obsidian"]` gets all wool blocks AND obsidian
    #[wasm_bindgen(js_name = getPaletteByKeywords)]
    pub fn get_palette_by_keywords(keywords: Vec<String>) -> Vec<String> {
        Self::filter_blocks(|f| {
            // Check if ID contains any keyword OR matches exactly (for things like "minecraft:obsidian")
            keywords
                .iter()
                .any(|k| f.id.contains(k) || f.id == k || f.id == format!("minecraft:{}", k))
        })
    }

    // Helper to reduce code duplication
    fn filter_blocks<F>(filter: F) -> Vec<String>
    where
        F: Fn(&BlockFacts) -> bool,
    {
        let mut blocks = Vec::new();
        for facts in all_blocks() {
            if filter(facts) {
                blocks.push(facts.id.to_string());
            }
        }
        blocks.sort(); // Sorting helps UI consistency
        blocks
    }
}
