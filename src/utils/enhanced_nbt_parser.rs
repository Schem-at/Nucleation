/// Enhanced NBT parser for block entity strings
///
/// Supports:
/// - signal= shorthand for all containers with item= customization
/// - Generic NBT field parsing (Lock, LootTable, Text1-4, integers, etc.)
/// - Backward compatibility with existing Items:[] and CustomName: syntax
use crate::utils::{get_container_spec, parse_custom_name, parse_items_array, NbtMap, NbtValue};
use std::collections::HashMap;

/// Parse generic NBT fields from a string
/// Format: key:value or key:"value" or key:123
pub fn parse_generic_nbt(nbt_str: &str) -> Result<HashMap<String, NbtValue>, String> {
    let mut nbt_map = HashMap::new();

    // Split by commas, but respect quotes and brackets
    let fields = split_nbt_fields(nbt_str);

    for field in fields {
        let field = field.trim();
        if field.is_empty() {
            continue;
        }

        // Skip known complex patterns that are handled separately
        if field.starts_with("Items:[") || field.starts_with("CustomName:") {
            continue;
        }

        // Parse key:value
        if let Some((key, value_str)) = field.split_once(':') {
            let key = key.trim();
            let value_str = value_str.trim();

            // Skip special keywords handled separately
            if key == "signal" || key == "item" {
                continue;
            }

            let value = parse_nbt_value(value_str)?;
            nbt_map.insert(key.to_string(), value);
        }
    }

    Ok(nbt_map)
}

/// Parse a single NBT value and infer its type
fn parse_nbt_value(value_str: &str) -> Result<NbtValue, String> {
    let value_str = value_str.trim();

    // String (quoted)
    if value_str.starts_with('"') && value_str.ends_with('"') {
        let s = value_str[1..value_str.len() - 1].to_string();
        return Ok(NbtValue::String(s));
    }

    // Byte (ends with 'b')
    if value_str.ends_with('b') || value_str.ends_with('B') {
        let num_str = &value_str[..value_str.len() - 1];
        if let Ok(byte_val) = num_str.parse::<i8>() {
            return Ok(NbtValue::Byte(byte_val));
        }
    }

    // Short (ends with 's')
    if value_str.ends_with('s') || value_str.ends_with('S') {
        let num_str = &value_str[..value_str.len() - 1];
        if let Ok(short_val) = num_str.parse::<i16>() {
            return Ok(NbtValue::Short(short_val));
        }
    }

    // Long (ends with 'L')
    if value_str.ends_with('L') {
        let num_str = &value_str[..value_str.len() - 1];
        if let Ok(long_val) = num_str.parse::<i64>() {
            return Ok(NbtValue::Long(long_val));
        }
    }

    // Float (ends with 'f')
    if value_str.ends_with('f') || value_str.ends_with('F') {
        let num_str = &value_str[..value_str.len() - 1];
        if let Ok(float_val) = num_str.parse::<f32>() {
            return Ok(NbtValue::Float(float_val));
        }
    }

    // Double (ends with 'd' or contains '.')
    if value_str.ends_with('d') || value_str.ends_with('D') {
        let num_str = &value_str[..value_str.len() - 1];
        if let Ok(double_val) = num_str.parse::<f64>() {
            return Ok(NbtValue::Double(double_val));
        }
    }

    if value_str.contains('.') {
        if let Ok(double_val) = value_str.parse::<f64>() {
            return Ok(NbtValue::Double(double_val));
        }
    }

    // Integer (default for numbers)
    if let Ok(int_val) = value_str.parse::<i32>() {
        return Ok(NbtValue::Int(int_val));
    }

    // Unquoted string (fallback)
    Ok(NbtValue::String(value_str.to_string()))
}

/// Split NBT string by commas, respecting quotes and brackets
fn split_nbt_fields(s: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut bracket_depth = 0;
    let mut brace_depth = 0;

    for c in s.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                current.push(c);
            }
            '[' if !in_quotes => {
                bracket_depth += 1;
                current.push(c);
            }
            ']' if !in_quotes => {
                bracket_depth -= 1;
                current.push(c);
            }
            '{' if !in_quotes => {
                brace_depth += 1;
                current.push(c);
            }
            '}' if !in_quotes => {
                brace_depth -= 1;
                current.push(c);
            }
            ',' if !in_quotes && bracket_depth == 0 && brace_depth == 0 => {
                if !current.trim().is_empty() {
                    fields.push(current.clone());
                }
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.trim().is_empty() {
        fields.push(current);
    }

    fields
}

/// Parse signal shorthand with optional item customization
/// Returns (signal_strength, item_id)
pub fn parse_signal_params(nbt_str: &str) -> Option<(u8, Option<String>)> {
    let fields = split_nbt_fields(nbt_str);

    let mut signal = None;
    let mut item = None;

    for field in fields {
        let field = field.trim();
        if let Some((key, value)) = field.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "signal" => {
                    signal = value.parse::<u8>().ok();
                }
                "item" => {
                    item = Some(value.to_string());
                }
                _ => {}
            }
        }
    }

    signal.map(|s| (s, item))
}

/// Generate items for signal strength with custom item support
pub fn create_container_items_nbt(
    container_slots: u32,
    signal_strength: u8,
    item_id: Option<&str>,
) -> Vec<NbtValue> {
    if signal_strength == 0 {
        return Vec::new();
    }

    // Ensure item_id has minecraft: namespace
    let item_id = if let Some(id) = item_id {
        if id.starts_with("minecraft:") {
            id.to_string()
        } else {
            format!("minecraft:{}", id)
        }
    } else {
        "minecraft:redstone_block".to_string()
    };

    const MAX_STACK: u32 = 64;
    const MAX_SIGNAL: u32 = 14; // Comparator max signal for non-zero

    // Calculate items needed based on container size
    let total_capacity = container_slots * MAX_STACK;
    let calculated = (total_capacity as f64 / MAX_SIGNAL as f64) * (signal_strength as f64 - 1.0);
    let items_needed = calculated.ceil() as u32;

    // Ensure minimum signal
    let total_items = std::cmp::max(signal_strength as u32, items_needed);
    let total_items = std::cmp::min(total_items, total_capacity); // Don't exceed capacity

    let mut items = Vec::new();
    let mut remaining_items = total_items;
    let mut slot: u8 = 0;

    while remaining_items > 0 && (slot as u32) < container_slots {
        let stack_size = std::cmp::min(remaining_items, MAX_STACK);
        let mut item_nbt = NbtMap::new();
        // Use modern format (1.20.5+): lowercase 'count' as Int
        item_nbt.insert("count".to_string(), NbtValue::Int(stack_size as i32));
        item_nbt.insert("Slot".to_string(), NbtValue::Byte(slot as i8));
        item_nbt.insert("id".to_string(), NbtValue::String(item_id.clone()));

        items.push(NbtValue::Compound(item_nbt));

        remaining_items -= stack_size as u32;
        slot += 1;
    }

    items
}

/// Get the music disc for a given signal strength (1-15)
/// Signal 0 = no disc, 1-15 = specific discs
fn get_jukebox_disc(signal: u8) -> Option<&'static str> {
    match signal {
        0 => None,
        1 => Some("minecraft:music_disc_13"),
        2 => Some("minecraft:music_disc_cat"),
        3 => Some("minecraft:music_disc_blocks"),
        4 => Some("minecraft:music_disc_chirp"),
        5 => Some("minecraft:music_disc_far"),
        6 => Some("minecraft:music_disc_mall"),
        7 => Some("minecraft:music_disc_mellohi"),
        8 => Some("minecraft:music_disc_stal"),
        9 => Some("minecraft:music_disc_strad"),
        10 => Some("minecraft:music_disc_ward"),
        11 => Some("minecraft:music_disc_11"),
        12 => Some("minecraft:music_disc_wait"),
        13 => Some("minecraft:music_disc_pigstep"),
        14 => Some("minecraft:music_disc_otherside"),
        15 => Some("minecraft:music_disc_5"),
        _ => None,
    }
}

/// Main parsing function that combines all features
pub fn parse_enhanced_nbt(
    block_name: &str,
    nbt_str: &str,
) -> Result<HashMap<String, NbtValue>, String> {
    let mut nbt_map = HashMap::new();

    let block_name_stripped = block_name.strip_prefix("minecraft:").unwrap_or(block_name);

    // 1. Check for jukebox signal (special case)
    if block_name_stripped == "jukebox" {
        if let Some((signal, _)) = parse_signal_params(nbt_str) {
            if signal > 15 {
                return Err("Signal strength must be between 0 and 15".to_string());
            }

            if let Some(disc) = get_jukebox_disc(signal) {
                // Create RecordItem NBT for jukebox
                let mut record_item = NbtMap::new();
                record_item.insert("count".to_string(), NbtValue::Int(1));
                record_item.insert("id".to_string(), NbtValue::String(disc.to_string()));
                nbt_map.insert("RecordItem".to_string(), NbtValue::Compound(record_item));
            }
            // Signal 0 means no disc, so we don't add RecordItem
        }
    }
    // 2. Check for signal shorthand (for containers)
    else if let Some(spec) = get_container_spec(block_name) {
        if let Some((signal, custom_item)) = parse_signal_params(nbt_str) {
            if signal > 15 {
                return Err("Signal strength must be between 0 and 15".to_string());
            }

            // Only generate items if Items aren't explicitly provided
            if !nbt_str.contains("Items:[") {
                let items = create_container_items_nbt(spec.slots, signal, custom_item.as_deref());
                if !items.is_empty() {
                    nbt_map.insert("Items".to_string(), NbtValue::List(items));
                }
            }
        }
    }

    // 2. Parse explicit Items array (overrides signal)
    if nbt_str.contains("Items:[") {
        let items = parse_items_array(nbt_str)?;
        nbt_map.insert("Items".to_string(), NbtValue::List(items));
    }

    // 3. Parse CustomName
    if nbt_str.contains("CustomName:") {
        let name = parse_custom_name(nbt_str)?;
        nbt_map.insert("CustomName".to_string(), NbtValue::String(name));
    }

    // 4. Parse generic NBT fields
    let generic_nbt = parse_generic_nbt(nbt_str)?;
    for (key, value) in generic_nbt {
        // Don't override Items if already set
        if key != "Items" || !nbt_map.contains_key("Items") {
            nbt_map.insert(key, value);
        }
    }

    Ok(nbt_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nbt_value_types() {
        assert!(matches!(
            parse_nbt_value("\"hello\""),
            Ok(NbtValue::String(_))
        ));
        assert!(matches!(parse_nbt_value("123"), Ok(NbtValue::Int(123))));
        assert!(matches!(parse_nbt_value("1b"), Ok(NbtValue::Byte(1))));
        assert!(matches!(parse_nbt_value("3.14"), Ok(NbtValue::Double(_))));
    }

    #[test]
    fn test_parse_signal_params() {
        let (signal, item) = parse_signal_params("signal=14").unwrap();
        assert_eq!(signal, 14);
        assert_eq!(item, None);

        let (signal, item) = parse_signal_params("signal=10,item=diamond").unwrap();
        assert_eq!(signal, 10);
        assert_eq!(item, Some("diamond".to_string()));
    }

    #[test]
    fn test_create_container_items_hopper() {
        let items = create_container_items_nbt(5, 10, None);
        assert!(items.len() <= 5, "Hopper should not exceed 5 slots");
        assert!(items.len() > 0, "Should generate items for signal=10");
    }
}
