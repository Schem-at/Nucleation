/// Container block specifications for inventory-based block entities
///
/// This module defines the inventory slot counts for various container blocks
/// used in signal strength calculations and item generation.
use std::collections::HashMap;

/// Information about a container block type
#[derive(Debug, Clone)]
pub struct ContainerSpec {
    /// Number of inventory slots
    pub slots: u32,
    /// Block type identifier (e.g., "barrel", "chest")
    pub block_type: &'static str,
    /// Description of the container
    pub description: &'static str,
}

/// Build the container specifications map
fn build_container_specs() -> HashMap<String, ContainerSpec> {
    let mut specs = HashMap::new();

    // Standard containers (27 slots)
    specs.insert(
        "barrel".to_string(),
        ContainerSpec {
            slots: 27,
            block_type: "barrel",
            description: "Barrel (3 rows of 9 slots)",
        },
    );

    specs.insert(
        "chest".to_string(),
        ContainerSpec {
            slots: 27,
            block_type: "chest",
            description: "Single Chest (3 rows of 9 slots)",
        },
    );

    specs.insert(
        "trapped_chest".to_string(),
        ContainerSpec {
            slots: 27,
            block_type: "trapped_chest",
            description: "Trapped Chest (3 rows of 9 slots)",
        },
    );

    specs.insert(
        "shulker_box".to_string(),
        ContainerSpec {
            slots: 27,
            block_type: "shulker_box",
            description: "Shulker Box (3 rows of 9 slots)",
        },
    );

    // All colored shulker boxes
    for color in &[
        "white",
        "orange",
        "magenta",
        "light_blue",
        "yellow",
        "lime",
        "pink",
        "gray",
        "light_gray",
        "cyan",
        "purple",
        "blue",
        "brown",
        "green",
        "red",
        "black",
    ] {
        let key = format!("{}_shulker_box", color);
        specs.insert(
            key.clone(),
            ContainerSpec {
                slots: 27,
                block_type: "shulker_box", // Generic type
                description: "Shulker Box (3 rows of 9 slots)",
            },
        );
    }

    // Hoppers (5 slots)
    specs.insert(
        "hopper".to_string(),
        ContainerSpec {
            slots: 5,
            block_type: "hopper",
            description: "Hopper (1 row of 5 slots)",
        },
    );

    // Dispensers and Droppers (9 slots)
    specs.insert(
        "dispenser".to_string(),
        ContainerSpec {
            slots: 9,
            block_type: "dispenser",
            description: "Dispenser (3x3 grid)",
        },
    );

    specs.insert(
        "dropper".to_string(),
        ContainerSpec {
            slots: 9,
            block_type: "dropper",
            description: "Dropper (3x3 grid)",
        },
    );

    // Furnaces (3 slots: input, fuel, output)
    specs.insert(
        "furnace".to_string(),
        ContainerSpec {
            slots: 3,
            block_type: "furnace",
            description: "Furnace (input, fuel, output)",
        },
    );

    specs.insert(
        "blast_furnace".to_string(),
        ContainerSpec {
            slots: 3,
            block_type: "blast_furnace",
            description: "Blast Furnace (input, fuel, output)",
        },
    );

    specs.insert(
        "smoker".to_string(),
        ContainerSpec {
            slots: 3,
            block_type: "smoker",
            description: "Smoker (input, fuel, output)",
        },
    );

    // Brewing Stand (5 slots: 3 potions, 1 ingredient, 1 fuel)
    specs.insert(
        "brewing_stand".to_string(),
        ContainerSpec {
            slots: 5,
            block_type: "brewing_stand",
            description: "Brewing Stand (3 potions, ingredient, fuel)",
        },
    );

    specs
}

/// Get container specification by block name
///
/// Strips "minecraft:" prefix if present and looks up the container spec
pub fn get_container_spec(block_name: &str) -> Option<ContainerSpec> {
    let name = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    build_container_specs().get(name).cloned()
}

/// Check if a block is a container
pub fn is_container(block_name: &str) -> bool {
    get_container_spec(block_name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_specs() {
        // Test standard containers
        assert_eq!(get_container_spec("barrel").unwrap().slots, 27);
        assert_eq!(get_container_spec("chest").unwrap().slots, 27);
        assert_eq!(get_container_spec("minecraft:chest").unwrap().slots, 27);

        // Test hoppers
        assert_eq!(get_container_spec("hopper").unwrap().slots, 5);

        // Test dispensers
        assert_eq!(get_container_spec("dispenser").unwrap().slots, 9);
        assert_eq!(get_container_spec("dropper").unwrap().slots, 9);

        // Test furnaces
        assert_eq!(get_container_spec("furnace").unwrap().slots, 3);

        // Test colored shulker boxes
        assert_eq!(get_container_spec("red_shulker_box").unwrap().slots, 27);
        assert_eq!(
            get_container_spec("minecraft:blue_shulker_box")
                .unwrap()
                .slots,
            27
        );
    }

    #[test]
    fn test_is_container() {
        assert!(is_container("barrel"));
        assert!(is_container("minecraft:chest"));
        assert!(is_container("hopper"));
        assert!(!is_container("stone"));
        assert!(!is_container("minecraft:dirt"));
    }
}
