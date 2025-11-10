/// Insign integration module
/// Provides sign extraction and compilation for region metadata
use crate::block_entity::BlockEntity;
use crate::universal_schematic::UniversalSchematic;
use insign::{compile, Error as CompileError};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Sign input for Insign compilation
#[derive(Debug, Clone)]
pub struct SignInput {
    pub pos: [i32; 3],
    pub text: String,
}

/// Position tuple [x, y, z] for Insign compatibility
pub type InsignPos = [i32; 3];

/// Extract sign text from a block entity
fn extract_sign_text(block_entity: &BlockEntity) -> Option<String> {
    // Signs can have different formats depending on Minecraft version
    // Modern format: front_text.messages array
    // Legacy format: Text1, Text2, Text3, Text4 strings

    let mut lines = Vec::new();

    // Try modern format first (1.20+)
    if let Some(front_text) = block_entity.nbt.get("front_text") {
        if let Some(messages) = front_text.as_compound().and_then(|c| c.get("messages")) {
            if let crate::utils::NbtValue::List(msg_list) = messages {
                for msg in msg_list {
                    if let Some(text) = msg.as_string() {
                        // Parse JSON text component to extract raw text
                        if let Ok(json) = serde_json::from_str::<JsonValue>(text) {
                            if let Some(text_str) = json.get("text").and_then(|t| t.as_str()) {
                                if !text_str.is_empty() {
                                    lines.push(text_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fall back to legacy format (1.7-1.19)
    if lines.is_empty() {
        for i in 1..=4 {
            let key = format!("Text{}", i);
            if let Some(text_value) = block_entity.nbt.get(&key) {
                if let Some(text) = text_value.as_string() {
                    // Parse JSON text component
                    if let Ok(json) = serde_json::from_str::<JsonValue>(text) {
                        if let Some(text_str) = json.get("text").and_then(|t| t.as_str()) {
                            if !text_str.is_empty() {
                                lines.push(text_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Extract all signs from a schematic
pub fn extract_signs(schematic: &UniversalSchematic) -> Vec<SignInput> {
    let mut signs = Vec::new();

    // Get all regions and their block entities
    let all_regions = schematic.get_all_regions();

    for region in all_regions.values() {
        for (pos, block_entity) in &region.block_entities {
            // Check if this is a sign block entity
            if block_entity.id.contains("sign") {
                if let Some(text) = extract_sign_text(block_entity) {
                    signs.push(SignInput {
                        pos: [pos.0, pos.1, pos.2],
                        text,
                    });
                }
            }
        }
    }

    // Sort by position for deterministic order (x, y, z)
    signs.sort_by_key(|sign| (sign.pos[0], sign.pos[1], sign.pos[2]));

    signs
}

/// Compile Insign data from sign inputs
/// Returns raw Insign compilation result as JSON
pub fn compile_insign(signs: Vec<SignInput>) -> Result<JsonValue, CompileError> {
    // Convert SignInput to the tuple format expected by insign::compile
    let sign_tuples: Vec<([i32; 3], String)> = signs.into_iter().map(|s| (s.pos, s.text)).collect();

    let result = compile(&sign_tuples)?;
    Ok(serde_json::to_value(result).expect("Failed to serialize Insign output"))
}

/// Compile Insign from a schematic and return the result
/// This is the main entry point for schematic-level Insign compilation
pub fn compile_schematic_insign(schematic: &UniversalSchematic) -> Result<JsonValue, CompileError> {
    let signs = extract_signs(schematic);
    compile_insign(signs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_entity::BlockEntity;
    use crate::region::Region;
    use crate::utils::NbtValue;

    // ====================================================================================
    // SIGN EXTRACTION TESTS
    // ====================================================================================

    #[test]
    fn test_extract_sign_text_legacy_format() {
        let mut block_entity = BlockEntity::new("minecraft:sign".to_string(), (0, 0, 0));

        // Add legacy sign text (1.7-1.19)
        block_entity.nbt.insert(
            "Text1".to_string(),
            NbtValue::String(r#"{"text":"Hello"}"#.to_string()),
        );
        block_entity.nbt.insert(
            "Text2".to_string(),
            NbtValue::String(r#"{"text":"World"}"#.to_string()),
        );

        let text = extract_sign_text(&block_entity);
        assert_eq!(text, Some("Hello\nWorld".to_string()));
    }

    #[test]
    fn test_extract_sign_text_empty_lines() {
        let mut block_entity = BlockEntity::new("minecraft:sign".to_string(), (5, 10, 15));

        // Sign with empty lines
        block_entity.nbt.insert(
            "Text1".to_string(),
            NbtValue::String(r#"{"text":"Line1"}"#.to_string()),
        );
        block_entity.nbt.insert(
            "Text2".to_string(),
            NbtValue::String(r#"{"text":""}"#.to_string()),
        );
        block_entity.nbt.insert(
            "Text3".to_string(),
            NbtValue::String(r#"{"text":"Line3"}"#.to_string()),
        );

        let text = extract_sign_text(&block_entity);
        assert_eq!(text, Some("Line1\nLine3".to_string()));
    }

    #[test]
    fn test_extract_sign_text_all_empty() {
        let mut block_entity = BlockEntity::new("minecraft:sign".to_string(), (0, 0, 0));

        // All empty lines
        block_entity.nbt.insert(
            "Text1".to_string(),
            NbtValue::String(r#"{"text":""}"#.to_string()),
        );
        block_entity.nbt.insert(
            "Text2".to_string(),
            NbtValue::String(r#"{"text":""}"#.to_string()),
        );

        let text = extract_sign_text(&block_entity);
        assert_eq!(text, None);
    }

    #[test]
    fn test_extract_sign_text_no_nbt() {
        let block_entity = BlockEntity::new("minecraft:sign".to_string(), (0, 0, 0));
        let text = extract_sign_text(&block_entity);
        assert_eq!(text, None);
    }

    #[test]
    fn test_extract_signs_from_schematic() {
        let mut schematic = UniversalSchematic::new("Test".to_string());

        // Add a sign block entity with Insign annotations
        let mut sign = BlockEntity::new("minecraft:sign".to_string(), (10, 64, 10));
        sign.nbt.insert(
            "Text1".to_string(),
            NbtValue::String(r#"{"text":"@rc([0,0,0],"}"#.to_string()),
        );
        sign.nbt.insert(
            "Text2".to_string(),
            NbtValue::String(r#"{"text":"[3,2,1])"}"#.to_string()),
        );
        sign.nbt.insert(
            "Text3".to_string(),
            NbtValue::String("{\"text\":\"#doc.label=\"\"}".to_string()),
        );
        sign.nbt.insert(
            "Text4".to_string(),
            NbtValue::String(r#"{"text":"\"Patch A\""}"#.to_string()),
        );

        let pos = sign.position;
        schematic.default_region.block_entities.insert(pos, sign);

        let signs = extract_signs(&schematic);
        assert_eq!(signs.len(), 1);
        assert_eq!(signs[0].pos, [10, 64, 10]);
        assert!(signs[0].text.contains("@rc"));
        assert!(signs[0].text.contains("Patch A"));
    }

    #[test]
    fn test_extract_signs_multiple_signs() {
        let mut schematic = UniversalSchematic::new("Test".to_string());

        // Add first sign
        let mut sign1 = BlockEntity::new("minecraft:sign".to_string(), (0, 0, 0));
        sign1.nbt.insert(
            "Text1".to_string(),
            NbtValue::String(r#"{"text":"@region1"}"#.to_string()),
        );
        let pos1 = sign1.position;
        schematic.default_region.block_entities.insert(pos1, sign1);

        // Add second sign
        let mut sign2 = BlockEntity::new("minecraft:wall_sign".to_string(), (10, 5, 20));
        sign2.nbt.insert(
            "Text1".to_string(),
            NbtValue::String(r#"{"text":"@region2"}"#.to_string()),
        );
        let pos2 = sign2.position;
        schematic.default_region.block_entities.insert(pos2, sign2);

        let signs = extract_signs(&schematic);
        assert_eq!(signs.len(), 2);
        assert_eq!(signs[0].pos, [0, 0, 0]);
        assert_eq!(signs[1].pos, [10, 5, 20]);
    }

    #[test]
    fn test_extract_signs_no_signs() {
        let schematic = UniversalSchematic::new("Test".to_string());
        let signs = extract_signs(&schematic);
        assert_eq!(signs.len(), 0);
    }

    // ====================================================================================
    // INSIGN COMPILATION TESTS
    // ====================================================================================

    #[test]
    fn test_compile_insign_empty() {
        let signs = vec![];
        let result = compile_insign(signs);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_compile_insign_basic_anonymous_region() {
        // Test: Basic anonymous region with metadata (from fixture a_basic.json)
        let signs = vec![SignInput {
            pos: [10, 64, 10],
            text: "@rc([0,0,0],[3,2,1])\n#doc.label=\"Patch A\"".to_string(),
        }];

        let result = compile_insign(signs);
        assert!(result.is_ok(), "Should compile successfully");

        let json = result.unwrap();
        let regions = json.as_object().unwrap();

        // Should have anonymous region
        assert!(
            regions.contains_key("__anon_0_0"),
            "Should have anonymous region"
        );

        let anon_region = regions.get("__anon_0_0").unwrap();

        // Check bounding boxes
        let boxes = anon_region
            .get("bounding_boxes")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(boxes.len(), 1);
        let bbox = boxes[0].as_array().unwrap();
        assert_eq!(bbox[0].as_array().unwrap(), &vec![10, 64, 10]); // Absolute position
        assert_eq!(bbox[1].as_array().unwrap(), &vec![13, 66, 11]); // Absolute end

        // Check metadata
        let metadata = anon_region.get("metadata").unwrap().as_object().unwrap();
        assert_eq!(
            metadata.get("doc.label").unwrap().as_str().unwrap(),
            "Patch A"
        );
    }

    #[test]
    fn test_compile_insign_named_accumulator() {
        // Test: Named region as accumulator across multiple signs (from b_named_multi.json)
        let signs = vec![
            SignInput {
                pos: [0, 64, 0],
                text: "@cpu.core=rc([100,70,-20],[104,72,-18])\n#cpu.core:logic.clock_hz=4"
                    .to_string(),
            },
            SignInput {
                pos: [10, 64, 10],
                text: "@cpu.core=rc([100,73,-20],[104,75,-18])\n#cpu.core:logic.cache_mb=2"
                    .to_string(),
            },
        ];

        let result = compile_insign(signs);
        assert!(result.is_ok(), "Should compile accumulator regions");

        let json = result.unwrap();
        let regions = json.as_object().unwrap();

        // Should have single cpu.core region with accumulated boxes
        assert!(regions.contains_key("cpu.core"));
        let core = regions.get("cpu.core").unwrap();

        let boxes = core.get("bounding_boxes").unwrap().as_array().unwrap();
        assert_eq!(boxes.len(), 2, "Should have 2 bounding boxes");

        // Check metadata (both keys should be present)
        let metadata = core.get("metadata").unwrap().as_object().unwrap();
        assert_eq!(metadata.get("logic.clock_hz").unwrap().as_i64().unwrap(), 4);
        assert_eq!(metadata.get("logic.cache_mb").unwrap().as_i64().unwrap(), 2);
    }

    #[test]
    fn test_compile_insign_wildcards_and_global() {
        // Test: Wildcards and $global metadata (from c_wildcards_global.json)
        let signs = vec![
            SignInput {
                pos: [0, 64, 0],
                text: "@cpu.unit1=rc([0,0,0],[2,2,2])\n#cpu.*:logic.clock_hz=4".to_string(),
            },
            SignInput {
                pos: [10, 64, 10],
                text: "@cpu.unit2=rc([5,0,0],[7,2,2])\n#$global:io.bus_width=8".to_string(),
            },
        ];

        let result = compile_insign(signs);
        assert!(result.is_ok());

        let json = result.unwrap();
        let regions = json.as_object().unwrap();

        // Check $global
        assert!(regions.contains_key("$global"));
        let global = regions.get("$global").unwrap();
        assert!(
            global.get("bounding_boxes").is_none(),
            "$global should have no boxes"
        );
        let global_meta = global.get("metadata").unwrap().as_object().unwrap();
        assert_eq!(
            global_meta.get("io.bus_width").unwrap().as_i64().unwrap(),
            8
        );

        // Check wildcard
        assert!(regions.contains_key("cpu.*"));
        let wildcard = regions.get("cpu.*").unwrap();
        assert!(
            wildcard.get("bounding_boxes").is_none(),
            "Wildcard should have no boxes"
        );
        let wildcard_meta = wildcard.get("metadata").unwrap().as_object().unwrap();
        assert_eq!(
            wildcard_meta
                .get("logic.clock_hz")
                .unwrap()
                .as_i64()
                .unwrap(),
            4
        );

        // Check that named regions inherit wildcard metadata
        let unit1 = regions.get("cpu.unit1").unwrap();
        let unit1_meta = unit1.get("metadata").unwrap().as_object().unwrap();
        assert_eq!(
            unit1_meta.get("logic.clock_hz").unwrap().as_i64().unwrap(),
            4,
            "unit1 should inherit clock_hz from cpu.*"
        );

        let unit2 = regions.get("cpu.unit2").unwrap();
        let unit2_meta = unit2.get("metadata").unwrap().as_object().unwrap();
        assert_eq!(
            unit2_meta.get("logic.clock_hz").unwrap().as_i64().unwrap(),
            4,
            "unit2 should inherit clock_hz from cpu.*"
        );
    }

    #[test]
    fn test_compile_insign_union_expression() {
        // Test: Boolean union expression (from d_union_expr.json)
        let signs = vec![
            SignInput {
                pos: [0, 64, 0],
                text: "@a=rc([0,0,0],[1,1,1])".to_string(),
            },
            SignInput {
                pos: [0, 64, 0],
                text: "@b=rc([2,0,0],[3,1,1])".to_string(),
            },
            SignInput {
                pos: [0, 64, 0],
                text: "@core=a+b\n#core:desc=\"Combined regions\"".to_string(),
            },
        ];

        let result = compile_insign(signs);
        assert!(result.is_ok());

        let json = result.unwrap();
        let regions = json.as_object().unwrap();

        // Should have all three regions
        assert!(regions.contains_key("a"));
        assert!(regions.contains_key("b"));
        assert!(regions.contains_key("core"));

        // core should have boxes from both a and b
        let core = regions.get("core").unwrap();
        let boxes = core.get("bounding_boxes").unwrap().as_array().unwrap();
        assert_eq!(boxes.len(), 2, "Union should combine both bounding boxes");

        // Check metadata
        let metadata = core.get("metadata").unwrap().as_object().unwrap();
        assert_eq!(
            metadata.get("desc").unwrap().as_str().unwrap(),
            "Combined regions"
        );
    }

    #[test]
    fn test_compile_insign_relative_coordinates() {
        // Test: Relative coordinates (rc) are converted to absolute
        let signs = vec![SignInput {
            pos: [100, 50, 200],
            text: "@test=rc([10,5,15],[20,10,25])".to_string(),
        }];

        let result = compile_insign(signs);
        assert!(result.is_ok());

        let json = result.unwrap();
        let regions = json.as_object().unwrap();
        let test_region = regions.get("test").unwrap();

        let boxes = test_region
            .get("bounding_boxes")
            .unwrap()
            .as_array()
            .unwrap();
        let bbox = boxes[0].as_array().unwrap();

        // rc coordinates should be offset by sign position
        assert_eq!(bbox[0].as_array().unwrap(), &vec![110, 55, 215]); // 100+10, 50+5, 200+15
        assert_eq!(bbox[1].as_array().unwrap(), &vec![120, 60, 225]); // 100+20, 50+10, 200+25
    }

    #[test]
    fn test_compile_insign_absolute_coordinates() {
        // Test: Absolute coordinates (ac) are not affected by sign position
        let signs = vec![SignInput {
            pos: [100, 50, 200],
            text: "@test=ac([10,5,15],[20,10,25])".to_string(),
        }];

        let result = compile_insign(signs);
        assert!(result.is_ok());

        let json = result.unwrap();
        let regions = json.as_object().unwrap();
        let test_region = regions.get("test").unwrap();

        let boxes = test_region
            .get("bounding_boxes")
            .unwrap()
            .as_array()
            .unwrap();
        let bbox = boxes[0].as_array().unwrap();

        // ac coordinates should remain absolute
        assert_eq!(bbox[0].as_array().unwrap(), &vec![10, 5, 15]);
        assert_eq!(bbox[1].as_array().unwrap(), &vec![20, 10, 25]);
    }

    // ====================================================================================
    // INTEGRATION TESTS
    // ====================================================================================

    #[test]
    fn test_full_schematic_to_insign_pipeline() {
        // End-to-end test: Create schematic with signs → extract → compile
        let mut schematic = UniversalSchematic::new("Test Redstone Bot".to_string());

        // Add sign with custom IO annotations
        let mut sign = BlockEntity::new("minecraft:sign".to_string(), (0, 0, 0));
        sign.nbt.insert(
            "Text1".to_string(),
            NbtValue::String(r#"{"text":"@io.input_a"}"#.to_string()),
        );
        sign.nbt.insert(
            "Text2".to_string(),
            NbtValue::String(r#"{"text":"=rc([0,1,0],"}"#.to_string()),
        );
        sign.nbt.insert(
            "Text3".to_string(),
            NbtValue::String(r#"{"text":"[0,1,0])"}"#.to_string()),
        );
        sign.nbt.insert(
            "Text4".to_string(),
            NbtValue::String("{\"text\":\"#io.type=\\\"i\\\"\"}".to_string()),
        );
        let pos = sign.position;
        schematic.default_region.block_entities.insert(pos, sign);

        // Extract and compile
        let result = compile_schematic_insign(&schematic);
        assert!(result.is_ok());

        let json = result.unwrap();
        let regions = json.as_object().unwrap();

        // Should have the io.input_a region
        let input_a_key = "io.input_a";
        assert!(regions.contains_key(input_a_key));
        let input = regions.get(input_a_key).unwrap();

        // Check bounding box
        let boxes = input.get("bounding_boxes").unwrap().as_array().unwrap();
        assert_eq!(boxes.len(), 1);

        // Check metadata
        let metadata = input.get("metadata").unwrap().as_object().unwrap();
        let io_type_key = "io.type";
        assert_eq!(metadata.get(io_type_key).unwrap().as_str().unwrap(), "i");
    }

    #[test]
    fn test_compile_insign_complex_metadata() {
        // Test: Complex JSON metadata values
        let signs = vec![SignInput {
            pos: [0, 0, 0],
            text: r#"@bot=rc([0,0,0],[10,5,10])
#bot:name="AI Bot v1"
#bot:version=2
#bot:active=true
#bot:config={"speed":100,"power":"high"}"#
                .to_string(),
        }];

        let result = compile_insign(signs);
        assert!(result.is_ok());

        let json = result.unwrap();
        let regions = json.as_object().unwrap();
        let bot = regions.get("bot").unwrap();
        let metadata = bot.get("metadata").unwrap().as_object().unwrap();

        // Check different value types
        let expected_name = "AI Bot v1";
        assert_eq!(
            metadata.get("name").unwrap().as_str().unwrap(),
            expected_name
        );
        assert_eq!(metadata.get("version").unwrap().as_i64().unwrap(), 2);
        assert_eq!(metadata.get("active").unwrap().as_bool().unwrap(), true);

        let config = metadata.get("config").unwrap().as_object().unwrap();
        assert_eq!(config.get("speed").unwrap().as_i64().unwrap(), 100);
        assert_eq!(config.get("power").unwrap().as_str().unwrap(), "high");
    }

    // ====================================================================================
    // ERROR HANDLING TESTS
    // ====================================================================================

    #[test]
    fn test_compile_insign_invalid_syntax() {
        // Test: Invalid Insign syntax should return error
        let invalid_text = "@invalid syntax here";
        let signs = vec![SignInput {
            pos: [0, 0, 0],
            text: invalid_text.to_string(),
        }];

        let result = compile_insign(signs);
        let error_msg = "Should fail on invalid syntax";
        assert!(result.is_err(), "{}", error_msg);
    }

    #[test]
    fn test_compile_insign_metadata_without_region() {
        // Test: Metadata without preceding geometry should error
        let orphan_meta = "#orphan.metadata=\"value\"";
        let signs = vec![SignInput {
            pos: [0, 0, 0],
            text: orphan_meta.to_string(),
        }];

        let result = compile_insign(signs);
        let should_fail_msg = "Should fail when metadata has no current region";
        assert!(result.is_err(), "{}", should_fail_msg);

        let error_msg = format!("{}", result.unwrap_err());
        let expected_error = "No current region";
        assert!(
            error_msg.contains(expected_error),
            "Error should mention missing current region"
        );
    }

    #[test]
    fn test_compile_insign_mixed_region_mode() {
        // Test: Region can't be both accumulator and defined
        let signs = vec![
            SignInput {
                pos: [0, 0, 0],
                text: "@cpu=rc([0,0,0],[1,1,1])".to_string(), // Accumulator
            },
            SignInput {
                pos: [0, 0, 0],
                text: "@cpu=a+b".to_string(), // Defined - conflict!
            },
        ];

        let result = compile_insign(signs);
        assert!(result.is_err(), "Should fail on mixed region mode");

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("cannot be both accumulator and defined"));
    }

    #[test]
    fn test_compile_insign_unknown_region_reference() {
        // Test: Expression referencing non-existent region
        let signs = vec![SignInput {
            pos: [0, 0, 0],
            text: "@result=nonexistent+alsomissing".to_string(),
        }];

        let result = compile_insign(signs);
        assert!(result.is_err(), "Should fail on unknown region reference");

        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Unknown region") || error_msg.contains("nonexistent"));
    }
}
