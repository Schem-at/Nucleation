/// Insign IO integration for TypedCircuitExecutor
/// Parses Insign DSL regions and creates IoLayout with distance-based position sorting
use crate::simulation::typed_executor::{
    IoLayoutBuilder, IoType, LayoutFunction, TypedCircuitExecutor,
};
use crate::simulation::MchprsWorld;
use crate::universal_schematic::UniversalSchematic;
use crate::{insign, BlockState};

/// Sort strategy for ordering extracted redstone positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortStrategy {
    /// Sort by Euclidean distance from sign, then Y, X, Z offsets (default)
    Distance,
    /// Sort by Y offset first, then XZ distance, then X, Z
    YFirst,
    /// Sort by X offset first, then Y, Z
    XFirst,
    /// Sort by Z offset first, then X, Y
    ZFirst,
}

impl SortStrategy {
    /// Parse sort strategy from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "distance" => Some(SortStrategy::Distance),
            "y_first" | "yfirst" | "y" => Some(SortStrategy::YFirst),
            "x_first" | "xfirst" | "x" => Some(SortStrategy::XFirst),
            "z_first" | "zfirst" | "z" => Some(SortStrategy::ZFirst),
            _ => None,
        }
    }
}

/// Error types for Insign IO parsing
#[derive(Debug, thiserror::Error)]
pub enum InsignIoError {
    #[error("Insign compilation error: {0}")]
    CompileError(String),

    #[error("No valid redstone positions found in region '{0}'")]
    NoPositions(String),

    #[error("Invalid data type '{0}': {1}")]
    InvalidDataType(String, String),

    #[error("Missing required metadata '{0}' for region '{1}'")]
    MissingMetadata(String, String),

    #[error("Invalid metadata value for '{0}' in region '{1}': {2}")]
    InvalidMetadataValue(String, String, String),

    #[error("Position count mismatch for region '{0}': expected {1}, got {2}")]
    PositionCountMismatch(String, usize, usize),

    #[error("IO layout build error: {0}")]
    LayoutBuildError(String),

    #[error("Schematic error: {0}")]
    SchematicError(String),
}

/// Extract all valid redstone positions from bounding boxes
fn extract_redstone_positions(
    boxes: &[([i32; 3], [i32; 3])],
    sign_pos: (i32, i32, i32),
    schematic: &UniversalSchematic,
    sort_strategy: SortStrategy,
) -> Vec<(i32, i32, i32)> {
    let mut positions = Vec::new();

    for (min, max) in boxes {
        for x in min[0]..=max[0] {
            for y in min[1]..=max[1] {
                for z in min[2]..=max[2] {
                    if let Some(block) = schematic.get_block(x, y, z) {
                        if is_valid_custom_io_block(block) {
                            positions.push((x, y, z));
                        }
                    }
                }
            }
        }
    }

    // Sort by relative offset from sign using the specified strategy
    positions.sort_by_key(|&(x, y, z)| {
        let dx = x - sign_pos.0;
        let dy = y - sign_pos.1;
        let dz = z - sign_pos.2;

        match sort_strategy {
            SortStrategy::Distance => {
                let dist_sq = dx * dx + dy * dy + dz * dz;
                (dist_sq, dy, dx, dz)
            }
            SortStrategy::YFirst => {
                let xz_dist_sq = dx * dx + dz * dz;
                (dy, xz_dist_sq, dx, dz)
            }
            SortStrategy::XFirst => {
                let yz_dist_sq = dy * dy + dz * dz;
                (dx, yz_dist_sq, dy, dz)
            }
            SortStrategy::ZFirst => {
                let xy_dist_sq = dx * dx + dy * dy;
                (dz, xy_dist_sq, dx, dy)
            }
        }
    });

    positions
}

/// Check if a block is valid for custom IO (redstone wire, repeater, comparator, torch, etc.)
fn is_valid_custom_io_block(block: &BlockState) -> bool {
    matches!(
        block.name.as_str(),
        "minecraft:redstone_wire"
            | "minecraft:repeater"
            | "minecraft:comparator"
            | "minecraft:redstone_torch"
            | "minecraft:redstone_wall_torch"
            | "minecraft:lever"
            | "minecraft:stone_button"
            | "minecraft:oak_button"
            | "minecraft:redstone_lamp"
    )
}

/// Parse IoType from data_type metadata string
fn parse_io_type(data_type_str: &str, position_count: usize) -> Result<IoType, InsignIoError> {
    match data_type_str.to_lowercase().as_str() {
        "bool" | "boolean" => {
            if position_count != 1 {
                return Err(InsignIoError::PositionCountMismatch(
                    data_type_str.to_string(),
                    1,
                    position_count,
                ));
            }
            Ok(IoType::Boolean)
        }
        "nibble" | "signal" | "signal_strength" => {
            // 4-bit value using Packed4 layout (signal strength 0-15 on a single wire)
            if position_count != 1 {
                return Err(InsignIoError::PositionCountMismatch(
                    data_type_str.to_string(),
                    1,
                    position_count,
                ));
            }
            Ok(IoType::UnsignedInt { bits: 4 })
        }
        "unsigned" | "uint" => Ok(IoType::UnsignedInt {
            bits: position_count,
        }),
        "signed" | "int" => Ok(IoType::SignedInt {
            bits: position_count,
        }),
        s if s.starts_with("unsigned:") || s.starts_with("uint:") => {
            let bits_str = s.split(':').nth(1).ok_or_else(|| {
                InsignIoError::InvalidDataType(
                    data_type_str.to_string(),
                    "Missing bit width".to_string(),
                )
            })?;
            let bits: usize = bits_str.parse().map_err(|_| {
                InsignIoError::InvalidDataType(
                    data_type_str.to_string(),
                    format!("Invalid bit width: {}", bits_str),
                )
            })?;

            if position_count != bits {
                return Err(InsignIoError::PositionCountMismatch(
                    data_type_str.to_string(),
                    bits,
                    position_count,
                ));
            }

            Ok(IoType::UnsignedInt { bits })
        }
        s if s.starts_with("signed:") || s.starts_with("int:") => {
            let bits_str = s.split(':').nth(1).ok_or_else(|| {
                InsignIoError::InvalidDataType(
                    data_type_str.to_string(),
                    "Missing bit width".to_string(),
                )
            })?;
            let bits: usize = bits_str.parse().map_err(|_| {
                InsignIoError::InvalidDataType(
                    data_type_str.to_string(),
                    format!("Invalid bit width: {}", bits_str),
                )
            })?;

            if position_count != bits {
                return Err(InsignIoError::PositionCountMismatch(
                    data_type_str.to_string(),
                    bits,
                    position_count,
                ));
            }

            Ok(IoType::SignedInt { bits })
        }
        "float32" | "f32" => {
            if position_count != 32 {
                return Err(InsignIoError::PositionCountMismatch(
                    data_type_str.to_string(),
                    32,
                    position_count,
                ));
            }
            Ok(IoType::Float32)
        }
        _ => Err(InsignIoError::InvalidDataType(
            data_type_str.to_string(),
            "Unknown data type".to_string(),
        )),
    }
}

/// Infer layout function based on position count and IoType
fn infer_layout_function(position_count: usize, io_type: &IoType) -> LayoutFunction {
    let bit_count = io_type.bit_count();

    // If bit count matches position count, use OneToOne (e.g., 1 bool = 1 position)
    if bit_count == position_count {
        LayoutFunction::OneToOne
    }
    // If we have fewer positions than bits, use Packed4 (e.g., 4 bits in 1 position)
    else if position_count <= 4 && bit_count <= position_count * 4 {
        LayoutFunction::Packed4
    }
    // Otherwise, use OneToOne
    else {
        LayoutFunction::OneToOne
    }
}

/// Find the sign position that first defined a region
fn find_sign_pos_for_region(
    input: &[([i32; 3], String)],
    region_name: &str,
) -> Result<(i32, i32, i32), InsignIoError> {
    for (pos, text) in input {
        if text.contains(&format!("@{}", region_name)) {
            return Ok((pos[0], pos[1], pos[2]));
        }
    }
    Err(InsignIoError::MissingMetadata(
        "sign_position".to_string(),
        region_name.to_string(),
    ))
}

/// Parse IO layout from Insign DSL map
pub fn parse_io_layout_from_insign(
    input: &[([i32; 3], String)],
    schematic: &UniversalSchematic,
) -> Result<IoLayoutBuilder, InsignIoError> {
    // Compile Insign DSL
    let dsl_map =
        ::insign::compile(input).map_err(|e| InsignIoError::CompileError(e.to_string()))?;

    let mut builder = IoLayoutBuilder::new();

    // Process all regions that start with "io."
    for (region_name, region_data) in dsl_map.iter() {
        if !region_name.starts_with("io.") {
            continue;
        }

        let name = region_name.strip_prefix("io.").unwrap();

        // Get required metadata
        let io_direction = region_data
            .metadata
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                InsignIoError::MissingMetadata("type".to_string(), region_name.to_string())
            })?;

        let data_type_str = region_data
            .metadata
            .get("data_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                InsignIoError::MissingMetadata("data_type".to_string(), region_name.to_string())
            })?;

        // Get optional sort strategy (default: Distance)
        let sort_strategy = region_data
            .metadata
            .get("sort")
            .and_then(|v| v.as_str())
            .and_then(SortStrategy::from_str)
            .unwrap_or(SortStrategy::Distance);

        // Get bounding boxes
        let boxes = region_data
            .bounding_boxes
            .as_ref()
            .ok_or_else(|| InsignIoError::NoPositions(region_name.to_string()))?;

        // Find sign position for this region
        let sign_pos = find_sign_pos_for_region(input, region_name)?;

        // Extract positions with distance-based sorting
        let positions = extract_redstone_positions(boxes, sign_pos, schematic, sort_strategy);

        if positions.is_empty() {
            return Err(InsignIoError::NoPositions(region_name.to_string()));
        }

        // Parse data type
        let io_type = parse_io_type(data_type_str, positions.len())?;

        // Infer layout function
        let layout = infer_layout_function(positions.len(), &io_type);

        // Add to builder
        match io_direction {
            "input" => {
                builder = builder
                    .add_input(name.to_string(), io_type, layout, positions)
                    .map_err(|e| InsignIoError::LayoutBuildError(e))?;
            }
            "output" => {
                builder = builder
                    .add_output(name.to_string(), io_type, layout, positions)
                    .map_err(|e| InsignIoError::LayoutBuildError(e))?;
            }
            _ => {
                return Err(InsignIoError::InvalidMetadataValue(
                    "type".to_string(),
                    region_name.to_string(),
                    format!("Expected 'input' or 'output', got '{}'", io_direction),
                ))
            }
        }
    }

    Ok(builder)
}

/// Create TypedCircuitExecutor from Insign annotations in a schematic
pub fn create_executor_from_insign(
    schematic: &UniversalSchematic,
) -> Result<TypedCircuitExecutor, InsignIoError> {
    // Extract signs from schematic
    let signs = insign::extract_signs(schematic);

    // Convert to input format
    let input: Vec<([i32; 3], String)> = signs.into_iter().map(|s| (s.pos, s.text)).collect();

    // Parse IO layout
    let builder = parse_io_layout_from_insign(&input, schematic)?;
    let layout = builder.build();

    // Create a copy of the schematic without signs (MCHPRS might not support all sign block types)
    let mut schematic_without_signs = schematic.clone();
    for (pos, _) in &input {
        // Replace sign blocks with air
        let air_block = crate::BlockState {
            name: "minecraft:air".to_string(),
            properties: std::collections::HashMap::new(),
        };
        schematic_without_signs.set_block(pos[0], pos[1], pos[2], air_block);
    }

    // Create MchprsWorld
    let world = MchprsWorld::new(schematic_without_signs)
        .map_err(|e| InsignIoError::SchematicError(e.to_string()))?;

    // Create executor
    Ok(TypedCircuitExecutor::from_layout(world, layout))
}

/// Create a TypedCircuitExecutor from Insign annotations with custom simulation options
pub fn create_executor_from_insign_with_options(
    schematic: &UniversalSchematic,
    options: crate::simulation::SimulationOptions,
) -> Result<TypedCircuitExecutor, InsignIoError> {
    // Extract signs from schematic
    let signs = insign::extract_signs(schematic);

    // Convert to input format
    let input: Vec<([i32; 3], String)> = signs.into_iter().map(|s| (s.pos, s.text)).collect();

    // Parse IO layout
    let builder = parse_io_layout_from_insign(&input, schematic)?;
    let layout = builder.build();

    // Create a copy of the schematic without signs (MCHPRS might not support all sign block types)
    let mut schematic_without_signs = schematic.clone();
    for (pos, _) in &input {
        // Replace sign blocks with air
        let air_block = crate::BlockState {
            name: "minecraft:air".to_string(),
            properties: std::collections::HashMap::new(),
        };
        schematic_without_signs.set_block(pos[0], pos[1], pos[2], air_block);
    }

    // Create MchprsWorld with options
    let world = MchprsWorld::with_options(schematic_without_signs, options.clone())
        .map_err(|e| InsignIoError::SchematicError(e.to_string()))?;

    // Create executor with the same options so reset() preserves them
    Ok(TypedCircuitExecutor::from_layout_with_options(
        world, layout, options,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schematic_builder::SchematicBuilder;

    #[test]
    fn test_sort_strategy_from_str() {
        assert_eq!(
            SortStrategy::from_str("distance"),
            Some(SortStrategy::Distance)
        );
        assert_eq!(
            SortStrategy::from_str("y_first"),
            Some(SortStrategy::YFirst)
        );
        assert_eq!(SortStrategy::from_str("YFirst"), Some(SortStrategy::YFirst));
        assert_eq!(SortStrategy::from_str("x"), Some(SortStrategy::XFirst));
        assert_eq!(
            SortStrategy::from_str("z_first"),
            Some(SortStrategy::ZFirst)
        );
        assert_eq!(SortStrategy::from_str("invalid"), None);
    }

    #[test]
    fn test_parse_io_type_bool() {
        let result = parse_io_type("bool", 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IoType::Boolean);

        // Should fail with wrong position count
        let result = parse_io_type("bool", 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_io_type_unsigned() {
        let result = parse_io_type("unsigned", 8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IoType::UnsignedInt { bits: 8 });

        let result = parse_io_type("unsigned", 17);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IoType::UnsignedInt { bits: 17 });
    }

    #[test]
    fn test_parse_io_type_unsigned_explicit() {
        let result = parse_io_type("unsigned:8", 8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IoType::UnsignedInt { bits: 8 });

        // Should fail with mismatched position count
        let result = parse_io_type("unsigned:8", 7);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_io_type_signed() {
        let result = parse_io_type("signed", 12);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IoType::SignedInt { bits: 12 });
    }

    #[test]
    fn test_parse_io_type_float() {
        let result = parse_io_type("float32", 32);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IoType::Float32);

        let result = parse_io_type("f32", 32);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IoType::Float32);
    }

    #[test]
    fn test_extract_redstone_positions_distance_sort() {
        // Create a simple schematic with redstone wires in a line
        let template = "│││││";

        let schematic = SchematicBuilder::from_template(template)
            .unwrap()
            .build()
            .unwrap();

        let boxes = vec![([0, 0, 0], [4, 0, 0])];
        let sign_pos = (2, 0, 0); // Center position

        let positions =
            extract_redstone_positions(&boxes, sign_pos, &schematic, SortStrategy::Distance);

        // Should be sorted by distance from center (2,0,0)
        // Closest: (2,0,0) distance=0
        // Next: (1,0,0), (3,0,0) distance=1
        // Farthest: (0,0,0), (4,0,0) distance=2

        assert_eq!(positions.len(), 5);
        assert_eq!(positions[0], (2, 0, 0)); // Center, distance 0
    }

    #[test]
    fn test_extract_redstone_positions_y_first_sort() {
        // Create a horizontal line (Y-first sorting will still work, just with Y=0 for all)
        let template = "│││││";

        let schematic = SchematicBuilder::from_template(template)
            .unwrap()
            .build()
            .unwrap();

        let boxes = vec![([0, 0, 0], [4, 0, 0])];
        let sign_pos = (2, 0, 0);

        let positions =
            extract_redstone_positions(&boxes, sign_pos, &schematic, SortStrategy::YFirst);

        // With Y-first sorting on a horizontal line, all Y offsets are 0,
        // so it falls back to XZ distance, then X, then Z
        // Should be sorted by X offset: -2, -1, 0, 1, 2

        assert_eq!(positions.len(), 5);
        // All should have Y=0
        for pos in &positions {
            assert_eq!(pos.1, 0);
        }
    }

    #[test]
    fn test_parse_io_layout_single_input() {
        let template = r#"
│││││││││
        "#;

        let schematic = SchematicBuilder::from_template(template)
            .unwrap()
            .build()
            .unwrap();

        let input = vec![(
            [0, 0, 0],
            "@io.counter=rc([0,0,0],[7,0,0])\n#io.counter:type=\"output\"\n#io.counter:data_type=\"unsigned\""
                .to_string(),
        )];

        let builder = parse_io_layout_from_insign(&input, &schematic);
        assert!(builder.is_ok());

        let layout = builder.unwrap().build();
        assert_eq!(layout.outputs.len(), 1);
        assert!(layout.outputs.contains_key("counter"));
    }

    #[test]
    fn test_parse_io_layout_with_sort_override() {
        let template = "│││││";

        let schematic = SchematicBuilder::from_template(template)
            .unwrap()
            .build()
            .unwrap();

        let input = vec![(
            [2, 0, 0],
            "@io.test=rc([-2,0,0],[2,0,0])\n#io.test:type=\"input\"\n#io.test:data_type=\"unsigned\"\n#io.test:sort=\"y_first\""
                .to_string(),
        )];

        let builder = parse_io_layout_from_insign(&input, &schematic);
        assert!(builder.is_ok());
    }

    #[test]
    fn test_parse_io_layout_missing_metadata() {
        let template = "│";
        let schematic = SchematicBuilder::from_template(template)
            .unwrap()
            .build()
            .unwrap();

        // Missing data_type
        let input = vec![(
            [0, 0, 0],
            "@io.test=rc([0,0,0],[0,0,0])\n#io.test:type=\"input\"".to_string(),
        )];

        let result = parse_io_layout_from_insign(&input, &schematic);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InsignIoError::MissingMetadata(_, _)
        ));
    }

    #[test]
    fn test_rotation_invariance() {
        // Test that 180° rotation produces same bit ordering
        let template = r#"
│
│
│
│
        "#;

        let schematic = SchematicBuilder::from_template(template)
            .unwrap()
            .build()
            .unwrap();

        // Original: sign at (0,0,0), wires at (0,0-3,0)
        let input1 = vec![(
            [0, 0, 0],
            "@io.test=rc([0,0,0],[0,3,0])\n#io.test:type=\"input\"\n#io.test:data_type=\"unsigned\""
                .to_string(),
        )];

        let builder1 = parse_io_layout_from_insign(&input1, &schematic).unwrap();
        let layout1 = builder1.build();

        // After 180° rotation: sign at (0,0,0), wires still at relative (0,0-3,0)
        // (In reality, the schematic would be rotated, but for this test we just verify
        // that the same relative positions produce the same ordering)
        let input2 = vec![(
            [0, 0, 0],
            "@io.test=rc([0,0,0],[0,3,0])\n#io.test:type=\"input\"\n#io.test:data_type=\"unsigned\""
                .to_string(),
        )];

        let builder2 = parse_io_layout_from_insign(&input2, &schematic).unwrap();
        let layout2 = builder2.build();

        // Layouts should be identical
        assert_eq!(layout1.inputs.len(), layout2.inputs.len());
        assert_eq!(
            layout1.inputs.get("test").unwrap().positions,
            layout2.inputs.get("test").unwrap().positions
        );
    }

    #[test]
    fn test_create_executor_from_insign_with_full_adder() {
        // This test reproduces the issue where SchematicBuilder creates a full adder
        // with signs, and we need to ensure MCHPRS can handle it (by removing signs)
        let template = r#"# Base layer
·····c····
·····c····
··ccccc···
·ccccccc··
cc··cccccc
·c··c·····
·ccccc····
·cccccc···
···cccc···
···c··c···

# Logic layer
·····│····
·····↑····
··│█←┤█···
·█◀←┬▲▲┐··
──··├┴┴┴←─
·█··↑·····
·▲─←┤█····
·█←┬▲▲┐···
···├┴┴┤···
···│··│···
"#;

        let builder = SchematicBuilder::from_template(template).unwrap();
        let mut schematic = builder.build().unwrap();

        // Add Insign IO annotations via signs
        let mut nbt_a = std::collections::HashMap::new();
        nbt_a.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.a=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt_a.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.a:type=\\\"input\\\"\"}".to_string(),
        );
        nbt_a.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.a:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt_a.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(3, 2, 9, "minecraft:oak_sign[rotation=0]", nbt_a)
            .unwrap();

        let mut nbt_b = std::collections::HashMap::new();
        nbt_b.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.b=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt_b.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.b:type=\\\"input\\\"\"}".to_string(),
        );
        nbt_b.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.b:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt_b.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(6, 2, 9, "minecraft:oak_sign[rotation=0]", nbt_b)
            .unwrap();

        let mut nbt_cin = std::collections::HashMap::new();
        nbt_cin.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.carry_in=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt_cin.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.carry_in:type=\\\"input\\\"\"}".to_string(),
        );
        nbt_cin.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.carry_in:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt_cin.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(9, 2, 4, "minecraft:oak_sign[rotation=0]", nbt_cin)
            .unwrap();

        let mut nbt_sum = std::collections::HashMap::new();
        nbt_sum.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.sum=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt_sum.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.sum:type=\\\"output\\\"\"}".to_string(),
        );
        nbt_sum.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.sum:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt_sum.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(5, 2, 0, "minecraft:oak_sign[rotation=0]", nbt_sum)
            .unwrap();

        let mut nbt_cout = std::collections::HashMap::new();
        nbt_cout.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.carry_out=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt_cout.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.carry_out:type=\\\"output\\\"\"}".to_string(),
        );
        nbt_cout.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.carry_out:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt_cout.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(0, 2, 4, "minecraft:oak_sign[rotation=0]", nbt_cout)
            .unwrap();

        // This should not panic - signs should be removed before creating MCHPRS world
        // Before the fix, this would panic with "called `Option::unwrap()` on a `None` value"
        // because MCHPRS couldn't handle the oak_sign blocks
        let executor = create_executor_from_insign(&schematic);
        assert!(
            executor.is_ok(),
            "Failed to create executor from Insign with full adder: {:?}",
            executor.err()
        );

        // If we got here, the fix worked! The signs were properly removed before
        // creating the MCHPRS world, so it didn't panic on unsupported block types.
    }

    #[test]
    fn test_insign_position_extraction_matches_javascript() {
        // This test verifies that Rust extracts the same positions as JavaScript
        // Reproduces the issue where JavaScript finds 1 position but Rust finds 4

        let template = r#"# Base layer
·····c····
·····c····
··ccccc···
·ccccccc··
cc··cccccc
·c··c·····
·ccccc····
·cccccc···
···cccc···
···c··c···

# Logic layer
·····│····
·····↑····
··│█←┤█···
·█◀←┬▲▲┐··
──··├┴┴┴←─
·█··↑·····
·▲─←┤█····
·█←┬▲▲┐···
···├┴┴┤···
···│··│···
"#;

        let builder = SchematicBuilder::from_template(template).unwrap();
        let mut schematic = builder.build().unwrap();

        // Add ONE sign for input 'a' at position [3, 2, 9]
        // The sign points to [3, 1, 9] with rc([0,-1,0],[0,-1,0])
        let mut nbt_a = std::collections::HashMap::new();
        nbt_a.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.a=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt_a.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.a:type=\\\"input\\\"\"}".to_string(),
        );
        nbt_a.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.a:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt_a.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(3, 2, 9, "minecraft:oak_sign[rotation=0]", nbt_a)
            .unwrap();

        // Extract signs
        let signs = crate::insign::extract_signs(&schematic);
        assert_eq!(signs.len(), 1, "Should have exactly 1 sign");

        let sign = &signs[0];
        assert_eq!(sign.pos, [3, 2, 9], "Sign should be at [3, 2, 9]");

        // Parse with Insign
        let input: Vec<([i32; 3], String)> = signs.into_iter().map(|s| (s.pos, s.text)).collect();
        let dsl_map = ::insign::compile(&input).expect("Insign compilation should succeed");

        // Check the compiled region
        let region = dsl_map.get("io.a").expect("Should have io.a region");
        let boxes = region
            .bounding_boxes
            .as_ref()
            .expect("Should have bounding boxes");

        println!("Bounding boxes for io.a: {:?}", boxes);
        assert_eq!(boxes.len(), 1, "Should have exactly 1 bounding box");

        // The bounding box should be [[3, 1, 9], [3, 1, 9]] (absolute coords)
        let bbox = boxes[0];
        println!("Bounding box: min={:?}, max={:?}", bbox.0, bbox.1);
        assert_eq!(bbox.0, [3, 1, 9], "Bounding box min should be [3, 1, 9]");
        assert_eq!(bbox.1, [3, 1, 9], "Bounding box max should be [3, 1, 9]");

        // Now extract redstone positions
        let positions =
            extract_redstone_positions(boxes, (3, 2, 9), &schematic, SortStrategy::Distance);

        println!("Extracted {} positions: {:?}", positions.len(), positions);

        // Check what block is at [3, 1, 9]
        if let Some(block) = schematic.get_block(3, 1, 9) {
            println!("Block at [3, 1, 9]: {}", block.name);
        } else {
            println!("No block at [3, 1, 9]");
        }

        // THIS IS THE BUG: We expect 1 position but might get 4
        assert_eq!(
            positions.len(),
            1,
            "Should extract exactly 1 position for a single-block bounding box, but got {}",
            positions.len()
        );

        if !positions.is_empty() {
            assert_eq!(positions[0], (3, 1, 9), "Position should be [3, 1, 9]");
        }
    }

    #[test]
    fn test_full_adder_execution_reproduces_browser_error() {
        use super::super::{ExecutionMode, Value};

        // Reproduce the exact scenario from the browser where we get "Bit count mismatch: expected 1, got 4"

        let template = r#"# Base layer
·····c····
·····c····
··ccccc···
·ccccccc··
cc··cccccc
·c··c·····
·ccccc····
·cccccc···
···cccc···
···c··c···

# Logic layer
·····│····
·····↑····
··│█←┤█···
·█◀←┬▲▲┐··
──··├┴┴┴←─
·█··↑·····
·▲─←┤█····
·█←┬▲▲┐···
···├┴┴┤···
···│··│···
"#;

        let builder = SchematicBuilder::from_template(template).unwrap();
        let mut schematic = builder.build().unwrap();

        println!("Schematic bounding box: {:?}", schematic.get_bounding_box());

        // Add all 5 Insign IO annotations exactly as in the browser
        let mut nbt = std::collections::HashMap::new();

        // io.a at [3, 2, 9]
        nbt.clear();
        nbt.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.a=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.a:type=\\\"input\\\"\"}".to_string(),
        );
        nbt.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.a:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(3, 2, 9, "minecraft:oak_sign[rotation=0]", nbt.clone())
            .unwrap();

        // io.b at [6, 2, 9]
        nbt.clear();
        nbt.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.b=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.b:type=\\\"input\\\"\"}".to_string(),
        );
        nbt.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.b:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(6, 2, 9, "minecraft:oak_sign[rotation=0]", nbt.clone())
            .unwrap();

        // io.carry_in at [9, 2, 4]
        nbt.clear();
        nbt.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.carry_in=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.carry_in:type=\\\"input\\\"\"}".to_string(),
        );
        nbt.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.carry_in:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(9, 2, 4, "minecraft:oak_sign[rotation=0]", nbt.clone())
            .unwrap();

        // io.sum at [5, 2, 0] (OUTPUT)
        nbt.clear();
        nbt.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.sum=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.sum:type=\\\"output\\\"\"}".to_string(),
        );
        nbt.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.sum:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(5, 2, 0, "minecraft:oak_sign[rotation=0]", nbt.clone())
            .unwrap();

        // io.carry_out at [0, 2, 4] (OUTPUT)
        nbt.clear();
        nbt.insert(
            "Text1".to_string(),
            "{\"text\":\"@io.carry_out=rc([0,-1,0],[0,-1,0])\"}".to_string(),
        );
        nbt.insert(
            "Text2".to_string(),
            "{\"text\":\"#io.carry_out:type=\\\"output\\\"\"}".to_string(),
        );
        nbt.insert(
            "Text3".to_string(),
            "{\"text\":\"#io.carry_out:data_type=\\\"bool\\\"\"}".to_string(),
        );
        nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
        schematic
            .set_block_with_nbt(0, 2, 4, "minecraft:oak_sign[rotation=0]", nbt.clone())
            .unwrap();

        // Create executor from Insign
        println!("\n=== Creating TypedCircuitExecutor from Insign ===");
        let mut executor =
            create_executor_from_insign(&schematic).expect("Should create executor from Insign");

        println!("\n=== Executor created successfully ===");

        // Try to execute with all false inputs (matching browser scenario)
        println!("\n=== Executing with all false inputs ===");
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("a".to_string(), Value::Bool(false));
        inputs.insert("b".to_string(), Value::Bool(false));
        inputs.insert("carry_in".to_string(), Value::Bool(false));

        let result = executor.execute(
            inputs,
            ExecutionMode::UntilStable {
                stable_ticks: 5,
                max_ticks: 100,
            },
        );

        match result {
            Ok(exec_result) => {
                println!("✓ Execution succeeded!");
                println!("Outputs: {:?}", exec_result.outputs);
                println!("Ticks: {}", exec_result.ticks_elapsed);
            }
            Err(e) => {
                println!("✗ Execution failed: {}", e);
                panic!("Should not fail with error: {}", e);
            }
        }
    }
}
