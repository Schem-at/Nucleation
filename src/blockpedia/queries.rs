use crate::blockpedia::{errors::*, BlockFacts, Result, BLOCKS};
use std::collections::HashMap;

/// Find all blocks that have a specific property with a specific value
pub fn find_blocks_by_property(
    property: &str,
    value: &str,
) -> impl Iterator<Item = &'static BlockFacts> {
    let property = property.to_string();
    let value = value.to_string();
    BLOCKS
        .values()
        .filter(move |block| {
            block.get_property(&property) == Some(value.as_str())
                || block
                    .get_property_values(&property)
                    .map(|values| values.contains(&value))
                    .unwrap_or(false)
        })
        .copied()
}

/// Find all blocks that match a predicate function
pub fn find_blocks_matching<F>(predicate: F) -> impl Iterator<Item = &'static BlockFacts>
where
    F: Fn(&BlockFacts) -> bool,
{
    BLOCKS
        .values()
        .filter(move |block| predicate(block))
        .copied()
}

/// Search for blocks using a glob-like pattern (supports * wildcard)
pub fn search_blocks(pattern: &str) -> impl Iterator<Item = &'static BlockFacts> {
    let pattern = pattern.to_lowercase();
    BLOCKS
        .values()
        .filter(move |block| {
            let block_id = block.id().to_lowercase();
            if pattern.contains('*') {
                // Simple glob matching - split on * and check each part exists in order
                let parts: Vec<&str> = pattern.split('*').collect();
                if parts.is_empty() {
                    return true;
                }

                let mut search_pos = 0;
                for (i, part) in parts.iter().enumerate() {
                    if part.is_empty() {
                        continue;
                    }

                    if i == 0 {
                        // First part - must be at the beginning
                        if !block_id.starts_with(part) {
                            return false;
                        }
                        search_pos = part.len();
                    } else if i == parts.len() - 1 {
                        // Last part - must be at the end
                        if !block_id.ends_with(part) {
                            return false;
                        }
                    } else {
                        // Middle part - must exist after current position
                        if let Some(pos) = block_id[search_pos..].find(part) {
                            search_pos += pos + part.len();
                        } else {
                            return false;
                        }
                    }
                }
                true
            } else {
                // Exact substring match
                block_id.contains(&pattern)
            }
        })
        .copied()
}

/// Get all possible values for a specific property across all blocks
pub fn get_property_values(property: &str) -> Option<Vec<String>> {
    let mut all_values = std::collections::HashSet::new();
    let mut found_property = false;

    for block in BLOCKS.values() {
        if let Some(values) = block.get_property_values(property) {
            found_property = true;
            for value in values {
                all_values.insert(value);
            }
        }
    }

    if found_property {
        let mut sorted_values: Vec<String> = all_values.into_iter().collect();
        sorted_values.sort();
        Some(sorted_values)
    } else {
        None
    }
}

/// Count blocks that match a predicate
pub fn count_blocks_where<F>(predicate: F) -> usize
where
    F: Fn(&BlockFacts) -> bool,
{
    BLOCKS
        .values()
        .filter(move |block| predicate(block))
        .count()
}

/// Get block families - groups of related blocks
pub fn get_block_families() -> HashMap<String, Vec<String>> {
    let mut families = HashMap::new();

    for block in BLOCKS.values() {
        let id = block.id();

        // Extract family name from block ID
        // minecraft:oak_stairs -> stairs
        // minecraft:red_wool -> wool
        // minecraft:stone_brick_slab -> slab

        if let Some(colon_pos) = id.find(':') {
            let name_part = &id[colon_pos + 1..];

            // Common patterns for families
            let family_name = if name_part.ends_with("_stairs") {
                "stairs"
            } else if name_part.ends_with("_slab") {
                "slab"
            } else if name_part.ends_with("_wool") {
                "wool"
            } else if name_part.ends_with("_log") {
                "log"
            } else if name_part.ends_with("_planks") {
                "planks"
            } else if name_part.ends_with("_leaves") {
                "leaves"
            } else if name_part.ends_with("_door") {
                "door"
            } else if name_part.ends_with("_fence") {
                "fence"
            } else if name_part.ends_with("_wall") {
                "wall"
            } else if name_part.contains("_wood") {
                "wood"
            } else if name_part.contains("stone") && !name_part.contains("redstone") {
                "stone"
            } else {
                // For blocks that don't match patterns, use the full name
                name_part
            };

            families
                .entry(family_name.to_string())
                .or_insert_with(Vec::new)
                .push(id.to_string());
        }
    }

    // Sort each family's blocks
    for blocks in families.values_mut() {
        blocks.sort();
    }

    families
}

/// Find blocks that have multiple specific properties
pub fn blocks_with_properties(
    properties: &[(&str, &str)],
) -> impl Iterator<Item = &'static BlockFacts> {
    let properties: Vec<(String, String)> = properties
        .iter()
        .map(|(prop, value)| (prop.to_string(), value.to_string()))
        .collect();
    BLOCKS
        .values()
        .filter(move |block| {
            properties.iter().all(|(prop, value)| {
                if value == "*" {
                    // Wildcard - just check if property exists
                    block.has_property(prop)
                } else {
                    // Check for exact value match
                    block.get_property(prop) == Some(value.as_str())
                        || block
                            .get_property_values(prop)
                            .map(|values| values.contains(value))
                            .unwrap_or(false)
                }
            })
        })
        .copied()
}

/// Find properties that appear in less than a certain percentage of blocks
pub fn find_rare_properties(max_frequency: f64) -> HashMap<String, usize> {
    let total_blocks = BLOCKS.len();
    let mut property_counts = HashMap::new();

    // Count how many blocks have each property
    for block in BLOCKS.values() {
        for (property, _) in block.properties {
            *property_counts.entry(property.to_string()).or_insert(0) += 1;
        }
    }

    // Filter to rare properties
    property_counts
        .into_iter()
        .filter(|(_, count)| (*count as f64 / total_blocks as f64) < max_frequency)
        .collect()
}

/// Statistics about block properties
#[derive(Debug)]
pub struct PropertyStats {
    pub total_unique_properties: usize,
    pub most_common_property: (String, usize),
    pub blocks_with_no_properties: usize,
    pub average_properties_per_block: f64,
}

/// Get comprehensive statistics about block properties
pub fn get_property_stats() -> PropertyStats {
    let mut property_counts = HashMap::new();
    let mut blocks_with_no_properties = 0;
    let mut total_property_instances = 0;

    for block in BLOCKS.values() {
        if block.properties.is_empty() {
            blocks_with_no_properties += 1;
        } else {
            total_property_instances += block.properties.len();
            for (property, _) in block.properties {
                *property_counts.entry(property.to_string()).or_insert(0) += 1;
            }
        }
    }

    let most_common_property = property_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(prop, count)| (prop.clone(), *count))
        .unwrap_or(("none".to_string(), 0));

    PropertyStats {
        total_unique_properties: property_counts.len(),
        most_common_property,
        blocks_with_no_properties,
        average_properties_per_block: total_property_instances as f64 / BLOCKS.len() as f64,
    }
}

/// Enhanced block family detection with better categorization
pub fn get_enhanced_block_families() -> HashMap<String, Vec<String>> {
    let mut families = HashMap::new();

    for block in BLOCKS.values() {
        let id = block.id();

        if let Some(colon_pos) = id.find(':') {
            let name_part = &id[colon_pos + 1..];

            // Enhanced family detection with priority order
            let family_name = detect_block_family(name_part);

            families
                .entry(family_name.to_string())
                .or_insert_with(Vec::new)
                .push(id.to_string());
        }
    }

    // Sort each family's blocks
    for blocks in families.values_mut() {
        blocks.sort();
    }

    families
}

fn detect_block_family(name_part: &str) -> &str {
    // Priority-ordered family detection

    // Building materials
    if name_part.ends_with("_stairs") {
        return "stairs";
    }
    if name_part.ends_with("_slab") {
        return "slab";
    }
    if name_part.ends_with("_wall") {
        return "wall";
    }
    if name_part.ends_with("_fence") {
        return "fence";
    }
    if name_part.ends_with("_fence_gate") {
        return "fence_gate";
    }
    if name_part.ends_with("_door") {
        return "door";
    }
    if name_part.ends_with("_trapdoor") {
        return "trapdoor";
    }
    if name_part.ends_with("_button") {
        return "button";
    }
    if name_part.ends_with("_pressure_plate") {
        return "pressure_plate";
    }

    // Natural materials
    if name_part.ends_with("_wood") || name_part.ends_with("_log") {
        return "wood";
    }
    if name_part.ends_with("_planks") {
        return "planks";
    }
    if name_part.ends_with("_leaves") {
        return "leaves";
    }
    if name_part.ends_with("_sapling") {
        return "sapling";
    }

    // Decorative blocks
    if name_part.ends_with("_wool") {
        return "wool";
    }
    if name_part.ends_with("_carpet") {
        return "carpet";
    }
    if name_part.ends_with("_concrete") {
        return "concrete";
    }
    if name_part.ends_with("_concrete_powder") {
        return "concrete_powder";
    }
    if name_part.ends_with("_terracotta") {
        return "terracotta";
    }
    if name_part.ends_with("_glazed_terracotta") {
        return "glazed_terracotta";
    }
    if name_part.ends_with("_glass") {
        return "glass";
    }
    if name_part.ends_with("_glass_pane") {
        return "glass_pane";
    }
    if name_part.ends_with("_stained_glass") {
        return "stained_glass";
    }
    if name_part.ends_with("_stained_glass_pane") {
        return "stained_glass_pane";
    }

    // Stone variants
    if name_part.contains("stone")
        && !name_part.contains("redstone")
        && !name_part.contains("sandstone")
    {
        return "stone";
    }
    if name_part.contains("sandstone") {
        return "sandstone";
    }
    if name_part.contains("granite") {
        return "granite";
    }
    if name_part.contains("diorite") {
        return "diorite";
    }
    if name_part.contains("andesite") {
        return "andesite";
    }

    // Redstone components
    if name_part.contains("redstone") {
        return "redstone";
    }

    // Ores and metals
    if name_part.ends_with("_ore") {
        return "ore";
    }
    if name_part.starts_with("raw_") {
        return "raw_materials";
    }
    if name_part.contains("_ingot") || name_part.contains("_nugget") {
        return "metals";
    }

    // Tools and weapons
    if name_part.ends_with("_sword") {
        return "sword";
    }
    if name_part.ends_with("_pickaxe") {
        return "pickaxe";
    }
    if name_part.ends_with("_axe") && !name_part.ends_with("_pickaxe") {
        return "axe";
    }
    if name_part.ends_with("_shovel") {
        return "shovel";
    }
    if name_part.ends_with("_hoe") {
        return "hoe";
    }

    // Armor
    if name_part.ends_with("_helmet") {
        return "helmet";
    }
    if name_part.ends_with("_chestplate") {
        return "chestplate";
    }
    if name_part.ends_with("_leggings") {
        return "leggings";
    }
    if name_part.ends_with("_boots") {
        return "boots";
    }

    // Food
    if name_part.contains("bread") || name_part.contains("cake") || name_part.contains("cookie") {
        return "food";
    }

    // Use the full name as fallback
    name_part
}

/// Find blocks with complex property combinations
pub fn blocks_with_complex_properties(
    requirements: &[(String, Vec<String>)],
) -> impl Iterator<Item = &'static BlockFacts> {
    let requirements: Vec<(String, Vec<String>)> = requirements.to_vec();
    BLOCKS
        .values()
        .filter(move |block| {
            requirements.iter().all(|(prop, values)| {
                if let Some(block_values) = block.get_property_values(prop) {
                    values
                        .iter()
                        .any(|required_val| block_values.contains(required_val))
                } else {
                    false
                }
            })
        })
        .copied()
}

/// Analyze property correlation - find properties that often appear together
pub fn analyze_property_correlation() -> HashMap<String, Vec<(String, f64)>> {
    let mut correlations = HashMap::new();
    let mut property_pairs = HashMap::new();
    let mut individual_properties = HashMap::new();

    // Count property occurrences and co-occurrences
    for block in BLOCKS.values() {
        let block_properties: Vec<String> = block
            .properties
            .iter()
            .map(|(p, _)| p.to_string())
            .collect();

        // Count individual properties
        for prop in &block_properties {
            *individual_properties.entry(prop.clone()).or_insert(0) += 1;
        }

        // Count property pairs
        for i in 0..block_properties.len() {
            for j in (i + 1)..block_properties.len() {
                let pair = if block_properties[i] < block_properties[j] {
                    (block_properties[i].clone(), block_properties[j].clone())
                } else {
                    (block_properties[j].clone(), block_properties[i].clone())
                };
                *property_pairs.entry(pair).or_insert(0) += 1;
            }
        }
    }

    // Calculate correlations
    for ((prop1, prop2), pair_count) in property_pairs {
        let prop1_count = individual_properties.get(&prop1).unwrap_or(&0);
        let prop2_count = individual_properties.get(&prop2).unwrap_or(&0);

        if *prop1_count > 0 && *prop2_count > 0 {
            let correlation = pair_count as f64 / (*prop1_count as f64).min(*prop2_count as f64);

            correlations
                .entry(prop1.clone())
                .or_insert_with(Vec::new)
                .push((prop2.clone(), correlation));

            correlations
                .entry(prop2)
                .or_insert_with(Vec::new)
                .push((prop1, correlation));
        }
    }

    // Sort correlations by strength
    for correlations_list in correlations.values_mut() {
        correlations_list
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }

    correlations
}

/// Find blocks that are "similar" based on shared properties
pub fn find_similar_blocks(
    target_block_id: &str,
    min_shared_properties: usize,
) -> Vec<(&'static BlockFacts, usize)> {
    let target_block = match BLOCKS.get(target_block_id) {
        Some(block) => block,
        None => return Vec::new(),
    };

    let target_properties: std::collections::HashSet<&str> =
        target_block.properties.iter().map(|(p, _)| *p).collect();

    let mut similar_blocks = Vec::new();

    for block in BLOCKS.values() {
        if block.id() == target_block_id {
            continue; // Skip the target block itself
        }

        let block_properties: std::collections::HashSet<&str> =
            block.properties.iter().map(|(p, _)| *p).collect();
        let shared_count = target_properties.intersection(&block_properties).count();

        if shared_count >= min_shared_properties {
            similar_blocks.push((*block, shared_count));
        }
    }

    // Sort by number of shared properties (descending)
    similar_blocks.sort_by(|a, b| b.1.cmp(&a.1));
    similar_blocks
}

/// Advanced property statistics with more detailed analysis
#[derive(Debug)]
pub struct AdvancedPropertyStats {
    pub basic_stats: PropertyStats,
    pub property_distribution: HashMap<String, HashMap<String, usize>>, // property -> value -> count
    pub most_diverse_property: (String, usize), // property with most different values
    pub most_correlated_properties: Vec<(String, String, f64)>, // top correlated property pairs
}

pub fn get_advanced_property_stats() -> AdvancedPropertyStats {
    let basic_stats = get_property_stats();
    let mut property_distribution = HashMap::new();

    // Analyze value distribution for each property
    for block in BLOCKS.values() {
        for (property, _) in block.properties {
            if let Some(values) = block.get_property_values(property) {
                let prop_dist = property_distribution
                    .entry(property.to_string())
                    .or_insert_with(HashMap::new);
                for value in values {
                    *prop_dist.entry(value).or_insert(0) += 1;
                }
            }
        }
    }

    // Find most diverse property
    let most_diverse_property = property_distribution
        .iter()
        .max_by_key(|(_, values)| values.len())
        .map(|(prop, values)| (prop.clone(), values.len()))
        .unwrap_or(("none".to_string(), 0));

    // Get top correlated properties
    let correlations = analyze_property_correlation();
    let mut all_correlations = Vec::new();

    for (prop1, correlations_list) in correlations {
        for (prop2, correlation) in correlations_list {
            if prop1 < prop2 {
                // Avoid duplicates
                all_correlations.push((prop1.clone(), prop2, correlation));
            }
        }
    }

    all_correlations.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    let most_correlated_properties = all_correlations.into_iter().take(5).collect();

    AdvancedPropertyStats {
        basic_stats,
        property_distribution,
        most_diverse_property,
        most_correlated_properties,
    }
}

/// Validated query functions with proper error handling
pub mod validated {
    use super::*;

    /// Safely find blocks by property with validation
    pub fn find_blocks_by_property_safe(
        property: &str,
        value: &str,
    ) -> Result<Vec<&'static BlockFacts>> {
        // Validate inputs
        validation::validate_property_name(property)?;
        validation::validate_property_value(value)?;

        let results: Vec<_> = find_blocks_by_property(property, value).collect();

        if results.is_empty() {
            return Err(BlockpediaError::Query(QueryError::NoResults(format!(
                "No blocks found with property '{}' = '{}'",
                property, value
            ))));
        }

        Ok(results)
    }

    /// Safely search blocks with pattern validation
    pub fn search_blocks_safe(pattern: &str) -> Result<Vec<&'static BlockFacts>> {
        if pattern.is_empty() {
            return Err(BlockpediaError::invalid_format(
                pattern,
                "non-empty search pattern",
            ));
        }

        if pattern.len() > 128 {
            return Err(BlockpediaError::Validation(
                ValidationError::InvalidLength {
                    input: pattern.to_string(),
                    min_length: 1,
                    max_length: 128,
                },
            ));
        }

        // Check for invalid pattern characters
        let invalid_chars: Vec<char> = pattern
            .chars()
            .filter(|c| {
                !c.is_ascii_alphanumeric() && *c != '_' && *c != '-' && *c != ':' && *c != '*'
            })
            .collect();

        if !invalid_chars.is_empty() {
            return Err(BlockpediaError::Validation(
                ValidationError::InvalidCharacters {
                    input: pattern.to_string(),
                    invalid_chars,
                },
            ));
        }

        let results: Vec<_> = search_blocks(pattern).collect();

        if results.is_empty() {
            // Try to suggest alternatives
            let suggestions = recovery::suggest_similar_blocks(pattern);
            let suggestion_text = if suggestions.is_empty() {
                "No suggestions available".to_string()
            } else {
                format!("Suggestions: {}", suggestions.join(", "))
            };

            return Err(BlockpediaError::Query(QueryError::NoResults(format!(
                "No blocks match pattern '{}'. {}",
                pattern, suggestion_text
            ))));
        }

        Ok(results)
    }

    /// Safely get property values with validation
    pub fn get_property_values_safe(property: &str) -> Result<Vec<String>> {
        validation::validate_property_name(property)?;

        get_property_values(property).ok_or_else(|| {
            BlockpediaError::Property(PropertyError::NotFound {
                block_id: "any".to_string(),
                property: property.to_string(),
            })
        })
    }

    /// Safely validate block properties with detailed error reporting
    pub fn validate_block_properties_safe(
        block_id: &str,
        properties: &[(String, String)],
    ) -> Result<()> {
        validation::validate_block_id(block_id)?;

        let block_facts = BLOCKS
            .get(block_id)
            .ok_or_else(|| BlockpediaError::block_not_found(block_id))?;

        let mut errors = Vec::new();

        for (property, value) in properties {
            // Validate property name format
            if let Err(e) = validation::validate_property_name(property) {
                errors.push(format!("Property '{}': {}", property, e));
                continue;
            }

            // Validate property value format
            if let Err(e) = validation::validate_property_value(value) {
                errors.push(format!("Value '{}': {}", value, e));
                continue;
            }

            // Check if property exists on block
            if !block_facts.has_property(property) {
                errors.push(format!(
                    "Property '{}' does not exist on block '{}'",
                    property, block_id
                ));
                continue;
            }

            // Check if value is valid for property
            if let Some(valid_values) = block_facts.get_property_values(property) {
                if !valid_values.contains(value) {
                    errors.push(format!(
                        "Invalid value '{}' for property '{}'. Valid values: {:?}",
                        value, property, valid_values
                    ));
                }
            }
        }

        if !errors.is_empty() {
            return Err(BlockpediaError::State(StateError::ValidationFailed {
                state: format!("{}[properties]", block_id),
                errors,
            }));
        }

        Ok(())
    }

    /// Safely create a BlockState with comprehensive validation and helpful error recovery
    pub fn create_block_state_safe(
        block_id: &str,
        properties: &[(String, String)],
    ) -> Result<crate::blockpedia::BlockState> {
        // First validate all properties before creating the state
        validate_block_properties_safe(block_id, properties)?;

        // If validation passed, create the state step by step
        let mut state = crate::blockpedia::BlockState::new(block_id)?;

        for (property, value) in properties {
            // This should not fail since we already validated, but handle errors gracefully
            state = state.with(property, value)?;
        }

        Ok(state)
    }

    /// Query with timeout simulation (for future async support)
    pub fn query_with_timeout<F, R>(query_name: &str, query_fn: F) -> Result<R>
    where
        F: FnOnce() -> R,
    {
        // For now, just execute the query
        // In a real async implementation, this would have actual timeout logic

        if query_name.len() > 64 {
            return Err(BlockpediaError::Query(QueryError::InvalidSyntax(
                "Query name too long".to_string(),
            )));
        }

        Ok(query_fn())
    }
}
