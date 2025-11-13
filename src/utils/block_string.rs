use crate::utils::{NbtMap, NbtValue};
use crate::BlockState;
use serde_json::Value;
use std::collections::HashMap;

pub fn parse_block_string(block_string: &str) -> Result<(BlockState, Option<NbtMap>), String> {
    let mut parts = block_string.splitn(2, '{');
    let block_state_str = parts.next().unwrap().trim();
    let nbt_str = parts.next().map(|s| s.trim_end_matches('}'));

    // Parse block state
    let block_state = if block_state_str.contains('[') {
        let mut state_parts = block_state_str.splitn(2, '[');
        let block_name = state_parts.next().unwrap();
        let properties_str = state_parts
            .next()
            .ok_or("Missing properties closing bracket")?
            .trim_end_matches(']');

        let mut properties = HashMap::new();
        for prop in properties_str.split(',') {
            let mut kv = prop.split('=');
            let key = kv.next().ok_or("Missing property key")?.trim();
            let value = kv
                .next()
                .ok_or("Missing property value")?
                .trim()
                .trim_matches(|c| c == '\'' || c == '"');
            properties.insert(key.to_string(), value.to_string());
        }

        BlockState::new(block_name.to_string()).with_properties(properties)
    } else {
        BlockState::new(block_state_str.to_string())
    };

    // Parse NBT data if present
    let nbt_data = if let Some(nbt_str) = nbt_str {
        let mut nbt_map = NbtMap::new();

        // Parse Items array if present
        if nbt_str.contains("Items:[") {
            let items = parse_items_array(nbt_str)?;
            nbt_map.insert("Items".to_string(), NbtValue::List(items));
        }

        // Parse CustomName if present
        if nbt_str.contains("CustomName:") {
            let name = parse_custom_name(nbt_str)?;
            nbt_map.insert("CustomName".to_string(), NbtValue::String(name));
        }

        Some(nbt_map)
    } else {
        None
    };

    Ok((block_state, nbt_data))
}

pub fn parse_items_array(nbt_str: &str) -> Result<Vec<NbtValue>, String> {
    // Find the Items array
    let items_start = nbt_str.find("Items:[").ok_or("Missing Items array")?;
    let array_start = items_start + "Items:".len();

    // Extract the array content
    let array_str = extract_balanced_substring(&nbt_str[array_start..], '[', ']')
        .ok_or("Malformed Items array")?;

    // Remove outer brackets
    let items_content = array_str[1..array_str.len() - 1].trim();

    let mut items = Vec::new();
    for item_str in split_items(items_content) {
        let mut item_nbt = NbtMap::new();

        // Parse each property
        let clean_item_str = item_str.trim_matches(|c| c == '{' || c == '}');

        for prop in clean_item_str.split(',') {
            let prop = prop.trim();

            let (key_part, value_part) = prop
                .split_once(':')
                .ok_or_else(|| format!("Invalid property format: '{}'", prop))?;
            // Trim whitespace and quotes from the key
            let key = key_part.trim().trim_matches(|c| c == '"' || c == '\'');
            let value = value_part.trim();

            match key {
                // Accept both "Count" (old) and "count" (new) for backward compat
                "Count" | "count" => {
                    let count_str = value
                        .trim_matches(|c| c == '"' || c == '\'')
                        .trim_end_matches('b')
                        .trim_end_matches('B');
                    let count: i32 = count_str
                        .parse()
                        .map_err(|_| format!("Invalid count value: {}", count_str))?;
                    // Store in modern format: lowercase 'count' as Int
                    item_nbt.insert("count".to_string(), NbtValue::Int(count));
                }
                "Slot" => {
                    let slot_str = value
                        .trim_matches(|c| c == '"' || c == '\'')
                        .trim_end_matches('b')
                        .trim_end_matches('B');
                    let slot: i8 = slot_str
                        .parse()
                        .map_err(|_| format!("Invalid Slot value: {}", slot_str))?;
                    item_nbt.insert("Slot".to_string(), NbtValue::Byte(slot));
                }
                "id" => {
                    let id = value.trim_matches(|c| c == '"' || c == '\'').to_string();
                    item_nbt.insert("id".to_string(), NbtValue::String(id));
                }
                _ => {}
            }
        }

        // Verify required fields (modern format uses lowercase 'count')
        if item_nbt.get("count").is_none()
            || item_nbt.get("Slot").is_none()
            || item_nbt.get("id").is_none()
        {
            return Err("Missing required item properties".to_string());
        }

        items.push(NbtValue::Compound(item_nbt));
    }

    Ok(items)
}

// Helper function to split items, handling nested braces
fn split_items(items_str: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current_item = String::new();
    let mut brace_count = 0;

    for c in items_str.chars() {
        match c {
            '{' => {
                brace_count += 1;
                current_item.push(c);
            }
            '}' => {
                brace_count -= 1;
                current_item.push(c);
                if brace_count == 0 {
                    let item = current_item.trim();
                    if !item.is_empty() {
                        items.push(item.to_string());
                    }
                    current_item.clear();
                }
            }
            ',' if brace_count == 0 => {}
            _ => {
                if brace_count > 0 {
                    current_item.push(c);
                }
            }
        }
    }

    items
}

pub fn parse_custom_name(nbt_str: &str) -> Result<String, String> {
    let name_start = nbt_str
        .find("CustomName:")
        .ok_or("No CustomName field found")?
        + "CustomName:".len();

    // Find the end of the CustomName value (either at a comma or end of string)
    let name_content = nbt_str[name_start..].trim_start();

    let name_content = name_content.trim_start_matches('\'');

    let name_content = name_content
        .split_once(',')
        .map(|(name, _)| name)
        .unwrap_or_else(|| name_content);

    let name_content = name_content.trim_end_matches('\'').trim();

    // If it's a JSON object
    if name_content.starts_with('{') {
        // Parse the JSON string
        match serde_json::from_str::<Value>(name_content) {
            Ok(json) => json
                .get("text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| "Missing or invalid 'text' field in CustomName JSON".to_string()),
            Err(e) => Err(format!("Invalid JSON in CustomName: {}", e)),
        }
    } else {
        // If it's a plain string, return it as is
        Ok(name_content.to_string())
    }
}

fn extract_balanced_substring(s: &str, open: char, close: char) -> Option<&str> {
    let mut depth = 0;
    let mut start = None;

    for (i, c) in s.chars().enumerate() {
        match c {
            c if c == open => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            c if c == close => {
                depth -= 1;
                if depth == 0 && start.is_some() {
                    return Some(&s[start.unwrap()..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_block_string() {
        let block_str = r#"minecraft:barrel[facing=up]{CustomName:'{"text":"Storage"}',Items:[{"Count":"64b","Slot":"0b","id":"minecraft:redstone"}]}"#;
        let (block_state, nbt_data) = parse_block_string(block_str).unwrap();

        // Check block state
        assert_eq!(block_state.get_name(), "minecraft:barrel");
        assert_eq!(block_state.get_property("facing"), Some(&"up".to_string()));

        // Check NBT data
        let nbt_data = nbt_data.expect("NBT data should be present");

        // Check Items
        if let Some(NbtValue::List(items)) = nbt_data.get("Items") {
            assert_eq!(items.len(), 1);
            if let NbtValue::Compound(item) = &items[0] {
                assert_eq!(
                    item.get("id"),
                    Some(&NbtValue::String("minecraft:redstone".to_string()))
                );
                // Modern format uses lowercase 'count' as Int
                assert_eq!(item.get("count"), Some(&NbtValue::Int(64)));
                assert_eq!(item.get("Slot"), Some(&NbtValue::Byte(0)));
            } else {
                panic!("Expected compound NBT value for item");
            }
        } else {
            panic!("Expected list of items");
        }

        // Check CustomName
        if let Some(NbtValue::String(name)) = nbt_data.get("CustomName") {
            assert_eq!(name, "Storage");
        } else {
            panic!("Expected CustomName to be a string with value 'Storage'");
        }
    }

    #[test]
    fn test_parse_custom_name() {
        let test_cases = [
            (
                r#"CustomName:'{"text":"Test Name"}'"#,
                "Test Name",
                "Basic JSON case",
            ),
            ("CustomName:'Plain Text'", "Plain Text", "Plain text case"),
            (
                r#"CustomName:'{"text":"Test Name"}',OtherField:123"#,
                "Test Name",
                "JSON with trailing comma",
            ),
            (
                "CustomName:'Simple Name',OtherField:123",
                "Simple Name",
                "Plain text with trailing comma",
            ),
        ];

        for (input, expected, description) in test_cases {
            let result = parse_custom_name(input).unwrap();
            assert_eq!(result, expected, "Failed on case: {}", description);
        }
    }

    #[test]
    fn test_parse_items_array() {
        let nbt_str = r#"Items:[{"Count":"64b","Slot":"0b","id":"minecraft:stone"}]"#;
        let items = parse_items_array(nbt_str).unwrap();

        assert_eq!(items.len(), 1);
        if let NbtValue::Compound(item) = &items[0] {
            assert_eq!(
                item.get("id"),
                Some(&NbtValue::String("minecraft:stone".to_string()))
            );
            // Modern format uses lowercase 'count' as Int
            assert_eq!(item.get("count"), Some(&NbtValue::Int(64)));
            assert_eq!(item.get("Slot"), Some(&NbtValue::Byte(0)));
        } else {
            panic!("Expected compound NBT value");
        }
    }
}
