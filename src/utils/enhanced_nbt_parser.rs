use super::container_spec::get_container_spec;
/// Enhanced NBT parser for block entity strings
///
/// Supports:
/// - `items=[diamond*64,emerald*12]` shorthand for explicit container contents
/// - `record=pigstep` shorthand for a jukebox record
/// - `signal=` shorthand for all containers with `item=` customization
/// - Generic NBT field parsing (Lock, LootTable, Text1-4, integers, etc.)
/// - Backward compatibility with existing `Items:[...]` and `CustomName:` syntax
use crate::nbt::{NbtMap, NbtValue};
use quartz_nbt::NbtTag;
use std::collections::HashMap;

fn unquoted_separator(field: &str, separator: char) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (index, c) in field.char_indices() {
        if quote.is_some() && c == '\\' && !escaped {
            escaped = true;
            continue;
        }
        if matches!(c, '\'' | '"') && !escaped {
            match quote {
                Some(active) if active == c => quote = None,
                None => quote = Some(c),
                _ => {}
            }
        } else if quote.is_none() && c == separator {
            return Some(index);
        }
        escaped = false;
    }
    None
}

fn split_unquoted_once(field: &str, separator: char) -> Option<(&str, &str)> {
    let index = unquoted_separator(field, separator)?;
    Some((&field[..index], &field[index + separator.len_utf8()..]))
}

/// Parse generic NBT fields from a string
/// Format: key:value or key:"value" or key:123
pub fn parse_generic_nbt(nbt_str: &str) -> Result<HashMap<String, NbtValue>, String> {
    let mut raw_fields = Vec::new();
    for field in split_nbt_fields(nbt_str)? {
        let field = field.trim();
        if field.is_empty() {
            continue;
        }
        let colon = unquoted_separator(field, ':');
        if unquoted_separator(field, '=')
            .is_some_and(|equals| colon.is_none_or(|colon| equals < colon))
        {
            continue;
        }
        raw_fields.push(field.to_string());
    }

    if raw_fields.is_empty() {
        return Ok(HashMap::new());
    }

    let wrapped = format!("{{{}}}", raw_fields.join(","));
    let compound =
        quartz_nbt::snbt::parse(&wrapped).map_err(|error| format!("Invalid raw NBT: {error}"))?;
    let mut nbt_map = HashMap::new();
    for (key, tag) in compound.inner() {
        nbt_map.insert(key.clone(), nbt_tag_to_value(tag));
    }
    if let Some(items) = nbt_map.get_mut("Items") {
        normalize_raw_items(items)?;
    }
    Ok(nbt_map)
}

fn nbt_tag_to_value(tag: &NbtTag) -> NbtValue {
    match tag {
        NbtTag::String(value) => NbtValue::String(value.clone()),
        NbtTag::Int(value) => NbtValue::Int(*value),
        NbtTag::Long(value) => NbtValue::Long(*value),
        NbtTag::Float(value) => NbtValue::Float(*value),
        NbtTag::Double(value) => NbtValue::Double(*value),
        NbtTag::Byte(value) => NbtValue::Byte(*value),
        NbtTag::Short(value) => NbtValue::Short(*value),
        NbtTag::IntArray(value) => NbtValue::IntArray(value.clone()),
        NbtTag::LongArray(value) => NbtValue::LongArray(value.clone()),
        NbtTag::ByteArray(value) => NbtValue::ByteArray(value.clone()),
        NbtTag::List(list) => NbtValue::List(list.iter().map(nbt_tag_to_value).collect()),
        NbtTag::Compound(compound) => {
            let mut map = NbtMap::new();
            for (key, value) in compound.inner() {
                map.insert(key.clone(), nbt_tag_to_value(value));
            }
            NbtValue::Compound(map)
        }
    }
}

fn legacy_integer(value: &NbtValue, field: &str) -> Result<i32, String> {
    match value {
        NbtValue::Byte(value) => Ok(i32::from(*value)),
        NbtValue::Short(value) => Ok(i32::from(*value)),
        NbtValue::Int(value) => Ok(*value),
        NbtValue::String(value) => value
            .trim_end_matches(['b', 'B', 's', 'S'])
            .parse::<i32>()
            .map_err(|_| format!("Invalid {field} value: {value}")),
        _ => Err(format!("Invalid {field} value type")),
    }
}

fn normalize_raw_items(items: &mut NbtValue) -> Result<(), String> {
    let NbtValue::List(items) = items else {
        return Err("Items must be an NBT list".to_string());
    };
    for item in items {
        let NbtValue::Compound(item) = item else {
            return Err("Items entries must be NBT compounds".to_string());
        };
        if let Some(count) = item.remove("Count") {
            item.insert(
                "count".to_string(),
                NbtValue::Int(legacy_integer(&count, "count")?),
            );
        }
        if let Some(count) = item.get("count").cloned() {
            item.insert(
                "count".to_string(),
                NbtValue::Int(legacy_integer(&count, "count")?),
            );
        }
        if let Some(slot) = item.get("Slot").cloned() {
            let slot = legacy_integer(&slot, "Slot")?;
            let slot = i8::try_from(slot).map_err(|_| format!("Invalid Slot value: {slot}"))?;
            item.insert("Slot".to_string(), NbtValue::Byte(slot));
        }
    }
    Ok(())
}

/// Parse a single NBT value and infer its type
#[cfg(test)]
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
    if let Some(num_str) = value_str.strip_suffix('L') {
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
fn split_nbt_fields(s: &str) -> Result<Vec<String>, String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut bracket_depth: i32 = 0;
    let mut brace_depth: i32 = 0;

    for c in s.chars() {
        if quote.is_some() && c == '\\' && !escaped {
            escaped = true;
            current.push(c);
            continue;
        }
        match c {
            '\'' | '"' if !escaped && quote.is_none_or(|active| active == c) => {
                quote = if quote.is_some() { None } else { Some(c) };
                current.push(c);
            }
            '[' if quote.is_none() => {
                bracket_depth += 1;
                current.push(c);
            }
            ']' if quote.is_none() => {
                bracket_depth -= 1;
                if bracket_depth < 0 {
                    return Err("Unmatched closing ']' in block entity data".to_string());
                }
                current.push(c);
            }
            '{' if quote.is_none() => {
                brace_depth += 1;
                current.push(c);
            }
            '}' if quote.is_none() => {
                brace_depth -= 1;
                if brace_depth < 0 {
                    return Err("Unmatched closing '}' in block entity data".to_string());
                }
                current.push(c);
            }
            ',' if quote.is_none() && bracket_depth == 0 && brace_depth == 0 => {
                if !current.trim().is_empty() {
                    fields.push(current.clone());
                }
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
        escaped = false;
    }

    if quote.is_some() {
        return Err("Unterminated quoted string in block entity data".to_string());
    }
    if bracket_depth != 0 || brace_depth != 0 {
        return Err("Unbalanced brackets or braces in block entity data".to_string());
    }
    if !current.trim().is_empty() {
        fields.push(current);
    }

    Ok(fields)
}

/// Parse signal shorthand with optional item customization
/// Returns (signal_strength, item_id)
pub fn parse_signal_params(nbt_str: &str) -> Option<(u8, Option<String>)> {
    let fields = split_nbt_fields(nbt_str).ok()?;

    let mut signal = None;
    let mut item = None;

    for field in fields {
        let field = field.trim();
        if let Some((key, value)) = split_unquoted_once(field, '=') {
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

fn assignment_value(nbt_str: &str, wanted: &str) -> Result<Option<String>, String> {
    let mut found = None;
    for field in split_nbt_fields(nbt_str)? {
        let Some((key, value)) = split_unquoted_once(&field, '=') else {
            continue;
        };
        if key.trim() != wanted {
            continue;
        }
        if found.is_some() {
            return Err(format!("Duplicate {wanted}= shorthand"));
        }
        found = Some(value.trim().to_string());
    }
    Ok(found)
}

fn validate_shorthand_assignments(nbt_str: &str) -> Result<(), String> {
    for field in split_nbt_fields(nbt_str)? {
        let Some(equals) = unquoted_separator(&field, '=') else {
            continue;
        };
        if unquoted_separator(&field, ':').is_some_and(|colon| colon < equals) {
            continue;
        }
        let key = field[..equals].trim();
        if !matches!(key, "items" | "record" | "signal" | "item") {
            return Err(format!("Unknown block entity shorthand '{key}='"));
        }
    }
    Ok(())
}

fn validated_signal_params(nbt_str: &str) -> Result<Option<(u8, Option<String>)>, String> {
    let raw_signal = assignment_value(nbt_str, "signal")?;
    let raw_item = assignment_value(nbt_str, "item")?;

    let Some(raw_signal) = raw_signal else {
        if raw_item.is_some() {
            return Err("item= shorthand requires signal=".to_string());
        }
        return Ok(None);
    };
    let signal = raw_signal
        .parse::<u8>()
        .map_err(|_| format!("Invalid signal strength '{raw_signal}'"))?;
    if signal > 15 {
        return Err("Signal strength must be between 0 and 15".to_string());
    }
    let item = raw_item.map(|item| normalize_item_id(&item)).transpose()?;
    Ok(Some((signal, item)))
}

fn normalize_item_id(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() || raw.chars().any(char::is_whitespace) {
        return Err("Item id must not be empty or contain whitespace".to_string());
    }

    let normalized = if raw.contains(':') {
        raw.to_string()
    } else {
        format!("minecraft:{raw}")
    };
    let Some((namespace, path)) = normalized.split_once(':') else {
        unreachable!("a namespace was added above");
    };
    if namespace.is_empty()
        || path.is_empty()
        || normalized.matches(':').count() != 1
        || !namespace
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.'))
        || !path.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.' | '/')
        })
    {
        return Err(format!("Invalid item id '{raw}'"));
    }
    Ok(normalized)
}

fn parse_explicit_items(nbt_str: &str) -> Result<Option<Vec<NbtValue>>, String> {
    let Some(value) = assignment_value(nbt_str, "items")? else {
        return Ok(None);
    };
    if !(value.starts_with('[') && value.ends_with(']')) {
        return Err("items= shorthand must use [item*count,...]".to_string());
    }

    let inner = value[1..value.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut items = Vec::new();
    for (slot, entry) in inner.split(',').enumerate() {
        let entry = entry.trim();
        let (raw_id, count) = match entry.rsplit_once('*') {
            Some((id, count)) => {
                let count = count
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| format!("Invalid item count in '{entry}'"))?;
                (id, count)
            }
            None => (entry, 1),
        };
        if !(1..=64).contains(&count) {
            return Err(format!("Item count must be between 1 and 64 in '{entry}'"));
        }
        let slot = u8::try_from(slot).map_err(|_| "Too many item entries".to_string())?;
        let mut item = NbtMap::new();
        item.insert(
            "id".to_string(),
            NbtValue::String(normalize_item_id(raw_id)?),
        );
        item.insert("count".to_string(), NbtValue::Int(i32::from(count)));
        item.insert("Slot".to_string(), NbtValue::Byte(slot as i8));
        items.push(NbtValue::Compound(item));
    }
    Ok(Some(items))
}

fn jukebox_record_item(nbt_str: &str) -> Result<Option<NbtValue>, String> {
    let Some(raw_record) = assignment_value(nbt_str, "record")? else {
        return Ok(None);
    };
    let normalized = normalize_item_id(&raw_record)?;
    let (namespace, path) = normalized
        .split_once(':')
        .expect("normalize_item_id always returns a namespace");
    let id = if namespace == "minecraft" && !path.starts_with("music_disc_") {
        format!("minecraft:music_disc_{path}")
    } else {
        normalized
    };

    let mut record = NbtMap::new();
    record.insert("count".to_string(), NbtValue::Int(1));
    record.insert("id".to_string(), NbtValue::String(id));
    Ok(Some(NbtValue::Compound(record)))
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

    // Preserve explicit namespaces; bare ids use minecraft:.
    let item_id = if let Some(id) = item_id {
        if id.contains(':') {
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

        remaining_items -= stack_size;
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

    validate_shorthand_assignments(nbt_str)?;
    let is_jukebox = matches!(block_name, "jukebox" | "minecraft:jukebox");
    let signal_params = validated_signal_params(nbt_str)?;
    let explicit_items = parse_explicit_items(nbt_str)?;
    let explicit_record = jukebox_record_item(nbt_str)?;
    // Parse raw SNBT before conflict checks so quoted keys are compared by
    // their decoded names (for example "Items" and 'RecordItem').
    let generic_nbt = parse_generic_nbt(nbt_str)?;
    let has_raw_items = generic_nbt.contains_key("Items");
    let has_raw_record = generic_nbt.contains_key("RecordItem");

    if explicit_items.is_some() && get_container_spec(block_name).is_none() {
        return Err("items= shorthand requires a supported container block".to_string());
    }
    if explicit_record.is_some() && !is_jukebox {
        return Err("record= shorthand requires a jukebox".to_string());
    }
    if explicit_items.is_some() && has_raw_items {
        return Err("Use either items= or raw Items: NBT, not both".to_string());
    }
    if explicit_record.is_some() && has_raw_record {
        return Err("Use either record= or raw RecordItem: NBT, not both".to_string());
    }
    if signal_params.is_some() && !is_jukebox && get_container_spec(block_name).is_none() {
        return Err("signal= shorthand requires a supported container or jukebox".to_string());
    }

    // 1. Check for jukebox content (special case)
    if is_jukebox {
        if signal_params
            .as_ref()
            .is_some_and(|(_, item)| item.is_some())
        {
            return Err("item= shorthand is not supported for jukeboxes".to_string());
        }
        if explicit_record.is_some() && signal_params.is_some() {
            return Err("Use either record= or signal= for a jukebox, not both".to_string());
        }
        if let Some(record) = explicit_record {
            nbt_map.insert("RecordItem".to_string(), record);
        } else if let Some((signal, _)) = signal_params {
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
        if explicit_items.is_some() && signal_params.is_some() {
            return Err("Use either items= or signal= for a container, not both".to_string());
        }
        if let Some(items) = explicit_items {
            if items.len() > spec.slots as usize {
                return Err(format!(
                    "{} has {} slots, but items= contains {} entries",
                    spec.description,
                    spec.slots,
                    items.len()
                ));
            }
            nbt_map.insert("Items".to_string(), NbtValue::List(items));
        } else if let Some((signal, custom_item)) = signal_params {
            // Only generate items if Items aren't explicitly provided
            if !has_raw_items {
                let items = create_container_items_nbt(spec.slots, signal, custom_item.as_deref());
                if !items.is_empty() {
                    nbt_map.insert("Items".to_string(), NbtValue::List(items));
                }
            }
        }
    }

    // 3. Parse generic NBT fields, including quote-aware CustomName SNBT.
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
        assert!(
            !items.is_empty(),
            "Hopper should generate items for signal=10"
        );
    }

    #[test]
    fn explicit_item_shorthand_builds_sequential_container_slots() {
        let nbt = parse_enhanced_nbt(
            "minecraft:chest",
            "items=[diamond*64,minecraft:emerald*12,mod:token]",
        )
        .unwrap();
        let NbtValue::List(items) = nbt.get("Items").unwrap() else {
            panic!("Items must be a list");
        };
        assert_eq!(nbt.len(), 1, "items= must not leak generic NBT fields");
        assert_eq!(items.len(), 3);

        let expected = [
            ("minecraft:diamond", 64, 0),
            ("minecraft:emerald", 12, 1),
            ("mod:token", 1, 2),
        ];
        for (item, (id, count, slot)) in items.iter().zip(expected) {
            let NbtValue::Compound(item) = item else {
                panic!("Each item must be a compound");
            };
            assert_eq!(item.get("id"), Some(&NbtValue::String(id.to_string())));
            assert_eq!(item.get("count"), Some(&NbtValue::Int(count)));
            assert_eq!(item.get("Slot"), Some(&NbtValue::Byte(slot)));
        }

        let empty = parse_enhanced_nbt("minecraft:chest", "items=[]").unwrap();
        assert_eq!(empty.get("Items"), Some(&NbtValue::List(Vec::new())));

        let signalled = parse_enhanced_nbt("minecraft:chest", "signal=2,item=mod:token").unwrap();
        let Some(NbtValue::List(items)) = signalled.get("Items") else {
            panic!("signal= must create an Items list");
        };
        let Some(NbtValue::Compound(first)) = items.first() else {
            panic!("signal= must create item compounds");
        };
        assert_eq!(
            first.get("id"),
            Some(&NbtValue::String("mod:token".to_string()))
        );
    }

    #[test]
    fn jukebox_record_shorthand_builds_record_item() {
        let nbt = parse_enhanced_nbt("minecraft:jukebox", "record=pigstep").unwrap();
        let NbtValue::Compound(record) = nbt.get("RecordItem").unwrap() else {
            panic!("RecordItem must be a compound");
        };
        assert_eq!(nbt.len(), 1, "record= must not leak generic NBT fields");
        assert_eq!(
            record.get("id"),
            Some(&NbtValue::String(
                "minecraft:music_disc_pigstep".to_string()
            ))
        );
        assert_eq!(record.get("count"), Some(&NbtValue::Int(1)));
    }

    #[test]
    fn content_shorthands_reject_invalid_or_ambiguous_input() {
        for (block, shorthand) in [
            ("minecraft:chest", "items=[diamond*0]"),
            ("minecraft:chest", "items=[diamond*65]"),
            ("minecraft:chest", "items=diamond"),
            ("minecraft:chest", "items=[Diamond]"),
            ("minecraft:chest", "items=[diamond],items=[emerald]"),
            ("minecraft:chest", "items=[diamond],signal=bogus"),
            ("minecraft:chest", "items=[diamond],item=emerald"),
            (
                "minecraft:chest",
                "items=[diamond],Items:[{Slot:0b,id:\"minecraft:emerald\",count:1}]",
            ),
            (
                "minecraft:chest",
                "items=[diamond],\"Items\":[{Slot:0b,id:\"minecraft:emerald\",count:1}]",
            ),
            (
                "minecraft:chest",
                "items=[diamond],'Items':[{Slot:0b,id:\"minecraft:emerald\",count:1}]",
            ),
            ("minecraft:hopper", "items=[a,b,c,d,e,f]"),
            ("minecraft:stone", "items=[diamond]"),
            ("minecraft:stone", "signal=10"),
            ("minecraft:stone", "unknown=value"),
            ("minecraft:chest", "signal=bogus"),
            ("minecraft:chest", "signal=1,signal=2"),
            ("minecraft:chest", "item=diamond"),
            ("minecraft:chest", "signal=1,item=diamond,item=emerald"),
            ("minecraft:stone", "record=pigstep"),
            ("minecraft:jukebox", "record=pigstep,signal=13"),
            ("minecraft:jukebox", "record=pigstep,signal=bogus"),
            (
                "minecraft:jukebox",
                "record=pigstep,RecordItem:{id:\"minecraft:music_disc_cat\",count:1}",
            ),
            (
                "minecraft:jukebox",
                "record=pigstep,\"RecordItem\":{id:\"minecraft:music_disc_cat\",count:1}",
            ),
            (
                "minecraft:jukebox",
                "record=pigstep,'RecordItem':{id:\"minecraft:music_disc_cat\",count:1}",
            ),
            ("minecraft:jukebox", "record=cat,record=pigstep"),
            ("minecraft:jukebox", "record="),
            ("minecraft:chest", "items=[diamond"),
            ("minecraft:chest", "items=[diamond]]"),
            ("minecraft:chest", "items=[\"diamond]"),
        ] {
            assert!(
                parse_enhanced_nbt(block, shorthand).is_err(),
                "{block}{{{shorthand}}} should fail"
            );
        }
    }

    #[test]
    fn generic_nbt_strings_can_still_contain_equals_signs() {
        let nbt = parse_enhanced_nbt("minecraft:chest", "Lock:\"left=right\"").unwrap();
        assert_eq!(
            nbt.get("Lock"),
            Some(&NbtValue::String("left=right".to_string()))
        );
    }

    #[test]
    fn raw_nested_nbt_preserves_compounds_and_extra_item_fields() {
        let chest = parse_enhanced_nbt(
            "minecraft:chest",
            r#"Items:[{Slot:0b,id:"minecraft:diamond",count:1,components:{"minecraft:custom_name":'[{"text":"Gem"}]'}}]"#,
        )
        .unwrap();
        let Some(NbtValue::List(items)) = chest.get("Items") else {
            panic!("raw Items must remain a list");
        };
        let Some(NbtValue::Compound(item)) = items.first() else {
            panic!("raw Items entries must remain compounds");
        };
        let Some(NbtValue::Compound(components)) = item.get("components") else {
            panic!("additional nested item fields must be preserved");
        };
        assert_eq!(
            components.get("minecraft:custom_name"),
            Some(&NbtValue::String(r#"[{"text":"Gem"}]"#.to_string()))
        );

        let jukebox = parse_enhanced_nbt(
            "minecraft:jukebox",
            r#"RecordItem:{id:"minecraft:music_disc_pigstep",count:1,components:{"minecraft:custom_data":{source:"raw"}}}"#,
        )
        .unwrap();
        let Some(NbtValue::Compound(record)) = jukebox.get("RecordItem") else {
            panic!("raw RecordItem must remain a compound");
        };
        assert_eq!(
            record.get("id"),
            Some(&NbtValue::String(
                "minecraft:music_disc_pigstep".to_string()
            ))
        );
        assert!(matches!(
            record.get("components"),
            Some(NbtValue::Compound(_))
        ));

        let quoted_key =
            parse_enhanced_nbt("minecraft:chest", r#""foo=bar":{nested:[1,2,3]}"#).unwrap();
        let Some(NbtValue::Compound(value)) = quoted_key.get("foo=bar") else {
            panic!("quoted raw SNBT keys containing '=' must be preserved");
        };
        assert!(matches!(value.get("nested"), Some(NbtValue::List(values)) if values.len() == 3));

        let custom_name =
            parse_enhanced_nbt("minecraft:chest", "CustomName:'Plain, Name'").unwrap();
        assert_eq!(
            custom_name.get("CustomName"),
            Some(&NbtValue::String("Plain, Name".to_string()))
        );
        let spaced_custom_name =
            parse_enhanced_nbt("minecraft:chest", "CustomName : 'Legacy Name'").unwrap();
        assert_eq!(
            spaced_custom_name.get("CustomName"),
            Some(&NbtValue::String("Legacy Name".to_string()))
        );

        let quoted_raw_items = parse_enhanced_nbt(
            "minecraft:chest",
            r#"signal=15,"Items":[{Slot:3b,id:"mod:raw",count:7}]"#,
        )
        .unwrap();
        let Some(NbtValue::List(items)) = quoted_raw_items.get("Items") else {
            panic!("quoted raw Items must take precedence over signal=");
        };
        assert_eq!(items.len(), 1);
        let Some(NbtValue::Compound(item)) = items.first() else {
            panic!("raw item must remain a compound");
        };
        assert_eq!(item.get("Slot"), Some(&NbtValue::Byte(3)));
        assert_eq!(
            item.get("id"),
            Some(&NbtValue::String("mod:raw".to_string()))
        );
        assert_eq!(item.get("count"), Some(&NbtValue::Int(7)));
    }
}
