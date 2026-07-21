use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a simplified NBT structure for Block Entities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    List(Vec<NbtValue>),
    Compound(HashMap<String, NbtValue>),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

/// Translator for Block Entity NBT data between Bedrock and Java editions.
///
/// Both directions are best-effort and pass through unknown block-entity
/// types unchanged. The goal is to keep block-entity-bearing blocks
/// functional after a `.mcstructure` round-trip — hoppers keep their items,
/// signs keep their text, pistons retain their arm orientation, etc.
pub struct BlockEntityTranslator;

impl BlockEntityTranslator {
    // ────────────────────────────────────────────────────────────────────
    //  Bedrock → Java
    // ────────────────────────────────────────────────────────────────────

    /// Translate Bedrock block-entity NBT into the Java edition layout.
    pub fn translate_bedrock_to_java(nbt: &HashMap<String, NbtValue>) -> HashMap<String, NbtValue> {
        let mut out = nbt.clone();

        if let Some(NbtValue::String(id)) = nbt.get("id") {
            match id.as_str() {
                "Chest" | "Barrel" | "ShulkerBox" | "TrappedChest" | "Dispenser" | "Dropper"
                | "Hopper" => {
                    Self::container_bedrock_to_java(&mut out, id);
                }
                "Furnace" | "BlastFurnace" | "Smoker" => {
                    Self::furnace_bedrock_to_java(&mut out, id);
                }
                "BrewingStand" => {
                    Self::brewing_stand_bedrock_to_java(&mut out);
                }
                "Sign" | "HangingSign" => {
                    Self::sign_bedrock_to_java(&mut out);
                }
                "Comparator" => {
                    Self::comparator_bedrock_to_java(&mut out);
                }
                "PistonArm" => {
                    Self::piston_arm_bedrock_to_java(&mut out);
                }
                "Beacon" => {
                    Self::beacon_bedrock_to_java(&mut out);
                }
                "CommandBlock" => {
                    Self::command_block_bedrock_to_java(&mut out);
                }
                _ => {}
            }
        }

        // Generic Items[] sweep — runs after type-specific transforms.
        if let Some(NbtValue::List(items)) = out.get_mut("Items") {
            for item in items {
                if let NbtValue::Compound(item_tag) = item {
                    Self::item_bedrock_to_java(item_tag);
                }
            }
        }

        out
    }

    // ────────────────────────────────────────────────────────────────────
    //  Java → Bedrock
    // ────────────────────────────────────────────────────────────────────

    /// Translate Java block-entity NBT into the Bedrock edition layout.
    ///
    /// The input is the standard Java `BlockEntity` payload (with keys
    /// like `id: "minecraft:hopper"`, `Items: [...]`, `Text1`..`Text4`).
    /// The output is what `block_position_data[i].block_entity_data`
    /// expects in an `.mcstructure` file.
    pub fn translate_java_to_bedrock(nbt: &HashMap<String, NbtValue>) -> HashMap<String, NbtValue> {
        let mut out = nbt.clone();

        let java_id = match nbt.get("id") {
            Some(NbtValue::String(s)) => s.clone(),
            _ => String::new(),
        };
        let bedrock_id = java_to_bedrock_be_id(&java_id);
        if !bedrock_id.is_empty() {
            out.insert("id".to_string(), NbtValue::String(bedrock_id.clone()));
        }

        match bedrock_id.as_str() {
            "Chest" | "Barrel" | "ShulkerBox" | "TrappedChest" | "Dispenser" | "Dropper"
            | "Hopper" => {
                Self::container_java_to_bedrock(&mut out, &bedrock_id);
            }
            "Furnace" | "BlastFurnace" | "Smoker" => {
                Self::furnace_java_to_bedrock(&mut out, &bedrock_id);
            }
            "BrewingStand" => {
                Self::brewing_stand_java_to_bedrock(&mut out);
            }
            "Sign" | "HangingSign" => {
                Self::sign_java_to_bedrock(&mut out);
            }
            "Comparator" => {
                Self::comparator_java_to_bedrock(&mut out);
            }
            "PistonArm" => {
                Self::piston_arm_java_to_bedrock(&mut out);
            }
            "Beacon" => {
                Self::beacon_java_to_bedrock(&mut out);
            }
            "CommandBlock" => {
                Self::command_block_java_to_bedrock(&mut out);
            }
            _ => {}
        }

        // Generic Items[] sweep — runs after type-specific transforms.
        if let Some(NbtValue::List(items)) = out.get_mut("Items") {
            for item in items {
                if let NbtValue::Compound(item_tag) = item {
                    Self::item_java_to_bedrock(item_tag);
                }
            }
        }

        out
    }

    // ────────────────────────────────────────────────────────────────────
    //  Per-type Bedrock → Java transforms
    // ────────────────────────────────────────────────────────────────────

    fn container_bedrock_to_java(nbt: &mut HashMap<String, NbtValue>, bedrock_id: &str) {
        let java_id = match bedrock_id {
            "Chest" => "minecraft:chest",
            "TrappedChest" => "minecraft:trapped_chest",
            "Barrel" => "minecraft:barrel",
            "ShulkerBox" => "minecraft:shulker_box",
            "Dispenser" => "minecraft:dispenser",
            "Dropper" => "minecraft:dropper",
            "Hopper" => "minecraft:hopper",
            _ => return,
        };
        nbt.insert("id".to_string(), NbtValue::String(java_id.to_string()));
    }

    fn furnace_bedrock_to_java(nbt: &mut HashMap<String, NbtValue>, bedrock_id: &str) {
        let java_id = match bedrock_id {
            "Furnace" => "minecraft:furnace",
            "BlastFurnace" => "minecraft:blast_furnace",
            "Smoker" => "minecraft:smoker",
            _ => return,
        };
        nbt.insert("id".to_string(), NbtValue::String(java_id.to_string()));
        if let Some(v) = nbt.remove("BurnDuration") {
            nbt.insert("BurnTime".to_string(), v);
        }
    }

    fn brewing_stand_bedrock_to_java(nbt: &mut HashMap<String, NbtValue>) {
        nbt.insert(
            "id".to_string(),
            NbtValue::String("minecraft:brewing_stand".to_string()),
        );
        if let Some(v) = nbt.remove("FuelAmount") {
            nbt.insert("Fuel".to_string(), v);
        }
        let _ = nbt.remove("FuelTotal");
    }

    fn sign_bedrock_to_java(nbt: &mut HashMap<String, NbtValue>) {
        nbt.insert(
            "id".to_string(),
            NbtValue::String("minecraft:sign".to_string()),
        );
        if let Some(NbtValue::String(text)) = nbt.remove("Text") {
            let mut lines = text.split('\n');
            for i in 1..=4 {
                let line = lines.next().unwrap_or("");
                nbt.insert(
                    format!("Text{}", i),
                    NbtValue::String(format!("{{\"text\":\"{}\"}}", line.replace('"', "\\\""))),
                );
            }
        }
    }

    fn comparator_bedrock_to_java(nbt: &mut HashMap<String, NbtValue>) {
        nbt.insert(
            "id".to_string(),
            NbtValue::String("minecraft:comparator".to_string()),
        );
    }

    fn piston_arm_bedrock_to_java(nbt: &mut HashMap<String, NbtValue>) {
        nbt.insert(
            "id".to_string(),
            NbtValue::String("minecraft:piston".to_string()),
        );
    }

    fn beacon_bedrock_to_java(nbt: &mut HashMap<String, NbtValue>) {
        nbt.insert(
            "id".to_string(),
            NbtValue::String("minecraft:beacon".to_string()),
        );
    }

    fn command_block_bedrock_to_java(nbt: &mut HashMap<String, NbtValue>) {
        nbt.insert(
            "id".to_string(),
            NbtValue::String("minecraft:command_block".to_string()),
        );
    }

    fn item_bedrock_to_java(item: &mut HashMap<String, NbtValue>) {
        if let Some(NbtValue::String(name)) = item.remove("Name") {
            let java_id = if name.contains(':') {
                name
            } else {
                format!("minecraft:{}", name)
            };
            item.insert("id".to_string(), NbtValue::String(java_id));
        }
    }

    // ────────────────────────────────────────────────────────────────────
    //  Per-type Java → Bedrock transforms
    // ────────────────────────────────────────────────────────────────────

    fn container_java_to_bedrock(_nbt: &mut HashMap<String, NbtValue>, _bedrock_id: &str) {
        // Bedrock containers keep the Items list; each item is translated
        // by the generic sweep. Containers don't carry direction in their
        // BE — that lives in the block state.
    }

    fn furnace_java_to_bedrock(nbt: &mut HashMap<String, NbtValue>, _bedrock_id: &str) {
        if let Some(v) = nbt.remove("BurnTime") {
            nbt.insert("BurnDuration".to_string(), v);
        }
    }

    fn brewing_stand_java_to_bedrock(nbt: &mut HashMap<String, NbtValue>) {
        if let Some(v) = nbt.remove("Fuel") {
            nbt.insert("FuelAmount".to_string(), v);
            nbt.entry(String::from("FuelTotal"))
                .or_insert(NbtValue::Short(20));
        }
    }

    fn sign_java_to_bedrock(nbt: &mut HashMap<String, NbtValue>) {
        if nbt.contains_key("Text") {
            return;
        }
        let mut lines: Vec<String> = Vec::with_capacity(4);
        for i in 1..=4 {
            if let Some(NbtValue::String(s)) = nbt.remove(&format!("Text{}", i)) {
                lines.push(strip_simple_text_component(&s));
            } else {
                lines.push(String::new());
            }
        }
        if let Some(NbtValue::Compound(front)) = nbt.remove("front_text") {
            if let Some(NbtValue::List(msgs)) = front.get("messages") {
                lines = msgs
                    .iter()
                    .filter_map(|v| match v {
                        NbtValue::String(s) => Some(strip_simple_text_component(s)),
                        _ => None,
                    })
                    .collect();
            }
        }
        while lines.len() < 4 {
            lines.push(String::new());
        }
        lines.truncate(4);
        nbt.insert("Text".to_string(), NbtValue::String(lines.join("\n")));
        nbt.entry(String::from("IgnoreLighting"))
            .or_insert(NbtValue::Byte(0));
        nbt.entry(String::from("SignTextColor"))
            .or_insert(NbtValue::Int(-16777216));
    }

    fn comparator_java_to_bedrock(_nbt: &mut HashMap<String, NbtValue>) {}

    fn piston_arm_java_to_bedrock(nbt: &mut HashMap<String, NbtValue>) {
        if let Some(NbtValue::Float(p)) = nbt.remove("progress") {
            nbt.insert("Progress".to_string(), NbtValue::Float(p));
        }
        if let Some(NbtValue::Byte(extending)) = nbt.remove("extending") {
            let state = if extending != 0 { 1i8 } else { 0i8 };
            nbt.insert("NewState".to_string(), NbtValue::Byte(state));
        }
    }

    fn beacon_java_to_bedrock(_nbt: &mut HashMap<String, NbtValue>) {}

    fn command_block_java_to_bedrock(_nbt: &mut HashMap<String, NbtValue>) {}

    fn item_java_to_bedrock(item: &mut HashMap<String, NbtValue>) {
        if let Some(NbtValue::String(id)) = item.remove("id") {
            let bedrock_name = id.strip_prefix("minecraft:").unwrap_or(&id).to_string();
            item.insert("Name".to_string(), NbtValue::String(bedrock_name));
        }
        if !item.contains_key("Damage") {
            if let Some(NbtValue::Compound(tag)) = item.get("tag") {
                if let Some(NbtValue::Int(d)) = tag.get("Damage") {
                    item.insert("Damage".to_string(), NbtValue::Short(*d as i16));
                }
            }
        }
    }
}

/// Strip a simple JSON text component `{"text":"foo"}` down to `foo`.
/// Returns the input unchanged if it doesn't match the simple pattern.
fn strip_simple_text_component(s: &str) -> String {
    let t = s.trim();
    if t.starts_with("{\"text\":\"") && t.ends_with("\"}") {
        let inner = &t[9..t.len() - 2];
        return inner.replace("\\\"", "\"");
    }
    s.to_string()
}

/// Map a Java block-entity id (`minecraft:hopper`) to the Bedrock canonical id
/// (`Hopper`). Returns an empty string for unknown ids; callers should treat
/// that as a pass-through.
fn java_to_bedrock_be_id(java_id: &str) -> String {
    let stripped = java_id.strip_prefix("minecraft:").unwrap_or(java_id);
    match stripped {
        "chest" => "Chest",
        "trapped_chest" => "TrappedChest",
        "barrel" => "Barrel",
        "shulker_box" => "ShulkerBox",
        "white_shulker_box"
        | "orange_shulker_box"
        | "magenta_shulker_box"
        | "light_blue_shulker_box"
        | "yellow_shulker_box"
        | "lime_shulker_box"
        | "pink_shulker_box"
        | "gray_shulker_box"
        | "light_gray_shulker_box"
        | "cyan_shulker_box"
        | "purple_shulker_box"
        | "blue_shulker_box"
        | "brown_shulker_box"
        | "green_shulker_box"
        | "red_shulker_box"
        | "black_shulker_box" => "ShulkerBox",
        "dispenser" => "Dispenser",
        "dropper" => "Dropper",
        "hopper" => "Hopper",
        "furnace" => "Furnace",
        "blast_furnace" => "BlastFurnace",
        "smoker" => "Smoker",
        "brewing_stand" => "BrewingStand",
        "sign" | "oak_sign" | "spruce_sign" | "birch_sign" | "jungle_sign" | "acacia_sign"
        | "dark_oak_sign" | "crimson_sign" | "warped_sign" | "mangrove_sign" | "bamboo_sign"
        | "cherry_sign" => "Sign",
        "wall_sign" | "oak_wall_sign" | "spruce_wall_sign" | "birch_wall_sign"
        | "jungle_wall_sign" | "acacia_wall_sign" | "dark_oak_wall_sign" | "crimson_wall_sign"
        | "warped_wall_sign" | "mangrove_wall_sign" | "bamboo_wall_sign" | "cherry_wall_sign" => {
            "Sign"
        }
        "oak_hanging_sign"
        | "spruce_hanging_sign"
        | "birch_hanging_sign"
        | "jungle_hanging_sign"
        | "acacia_hanging_sign"
        | "dark_oak_hanging_sign"
        | "crimson_hanging_sign"
        | "warped_hanging_sign"
        | "mangrove_hanging_sign"
        | "bamboo_hanging_sign"
        | "cherry_hanging_sign" => "HangingSign",
        "comparator" => "Comparator",
        "piston" | "sticky_piston" => "PistonArm",
        "beacon" => "Beacon",
        "command_block" | "chain_command_block" | "repeating_command_block" => "CommandBlock",
        "jukebox" => "Jukebox",
        "lectern" => "Lectern",
        "banner" | "white_banner" | "orange_banner" | "magenta_banner" | "light_blue_banner"
        | "yellow_banner" | "lime_banner" | "pink_banner" | "gray_banner" | "light_gray_banner"
        | "cyan_banner" | "purple_banner" | "blue_banner" | "brown_banner" | "green_banner"
        | "red_banner" | "black_banner" => "Banner",
        "bed" | "white_bed" | "orange_bed" | "magenta_bed" | "light_blue_bed" | "yellow_bed"
        | "lime_bed" | "pink_bed" | "gray_bed" | "light_gray_bed" | "cyan_bed" | "purple_bed"
        | "blue_bed" | "brown_bed" | "green_bed" | "red_bed" | "black_bed" => "Bed",
        "skeleton_skull"
        | "wither_skeleton_skull"
        | "zombie_head"
        | "player_head"
        | "creeper_head"
        | "dragon_head"
        | "piglin_head" => "Skull",
        _ => "",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmp(s: &str) -> NbtValue {
        NbtValue::String(s.to_string())
    }

    #[test]
    fn java_chest_to_bedrock_keeps_items() {
        let mut nbt = HashMap::new();
        nbt.insert("id".to_string(), cmp("minecraft:chest"));
        let mut item = HashMap::new();
        item.insert("id".to_string(), cmp("minecraft:cobblestone"));
        item.insert("Count".to_string(), NbtValue::Byte(64));
        nbt.insert(
            "Items".to_string(),
            NbtValue::List(vec![NbtValue::Compound(item)]),
        );

        let bed = BlockEntityTranslator::translate_java_to_bedrock(&nbt);
        assert_eq!(bed.get("id"), Some(&cmp("Chest")));
        let items = match bed.get("Items").unwrap() {
            NbtValue::List(v) => v,
            _ => panic!("Items must be a list"),
        };
        let first = match &items[0] {
            NbtValue::Compound(c) => c,
            _ => panic!(),
        };
        assert_eq!(first.get("Name"), Some(&cmp("cobblestone")));
        assert_eq!(first.get("Count"), Some(&NbtValue::Byte(64)));

        // Inverse must round-trip.
        let back = BlockEntityTranslator::translate_bedrock_to_java(&bed);
        assert_eq!(back.get("id"), Some(&cmp("minecraft:chest")));
        let items_back = match back.get("Items").unwrap() {
            NbtValue::List(v) => v,
            _ => panic!(),
        };
        let first_back = match &items_back[0] {
            NbtValue::Compound(c) => c,
            _ => panic!(),
        };
        assert_eq!(first_back.get("id"), Some(&cmp("minecraft:cobblestone")));
    }

    #[test]
    fn java_hopper_id_translates() {
        let mut nbt = HashMap::new();
        nbt.insert("id".to_string(), cmp("minecraft:hopper"));
        nbt.insert("TransferCooldown".to_string(), NbtValue::Int(0));
        let bed = BlockEntityTranslator::translate_java_to_bedrock(&nbt);
        assert_eq!(bed.get("id"), Some(&cmp("Hopper")));
        assert_eq!(bed.get("TransferCooldown"), Some(&NbtValue::Int(0)));
    }

    #[test]
    fn java_sign_text_concatenates() {
        let mut nbt = HashMap::new();
        nbt.insert("id".to_string(), cmp("minecraft:oak_sign"));
        nbt.insert("Text1".to_string(), cmp(r#"{"text":"hello"}"#));
        nbt.insert("Text2".to_string(), cmp(r#"{"text":"world"}"#));
        nbt.insert("Text3".to_string(), cmp(r#"{"text":""}"#));
        nbt.insert("Text4".to_string(), cmp(r#"{"text":""}"#));
        let bed = BlockEntityTranslator::translate_java_to_bedrock(&nbt);
        assert_eq!(bed.get("id"), Some(&cmp("Sign")));
        assert_eq!(bed.get("Text"), Some(&cmp("hello\nworld\n\n")));
    }

    #[test]
    fn java_furnace_burn_time_renamed() {
        let mut nbt = HashMap::new();
        nbt.insert("id".to_string(), cmp("minecraft:furnace"));
        nbt.insert("BurnTime".to_string(), NbtValue::Short(200));
        let bed = BlockEntityTranslator::translate_java_to_bedrock(&nbt);
        assert_eq!(bed.get("id"), Some(&cmp("Furnace")));
        assert_eq!(bed.get("BurnDuration"), Some(&NbtValue::Short(200)));
        assert!(bed.get("BurnTime").is_none());
    }

    #[test]
    fn unknown_be_id_passes_through() {
        let mut nbt = HashMap::new();
        nbt.insert("id".to_string(), cmp("minecraft:does_not_exist"));
        nbt.insert("custom".to_string(), NbtValue::Int(42));
        let bed = BlockEntityTranslator::translate_java_to_bedrock(&nbt);
        assert_eq!(bed.get("id"), Some(&cmp("minecraft:does_not_exist")));
        assert_eq!(bed.get("custom"), Some(&NbtValue::Int(42)));
    }

    #[test]
    fn bedrock_chest_to_java_translates_items() {
        let mut nbt = HashMap::new();
        nbt.insert("id".to_string(), cmp("Chest"));
        let mut item = HashMap::new();
        item.insert("Name".to_string(), cmp("cobblestone"));
        item.insert("Count".to_string(), NbtValue::Byte(32));
        nbt.insert(
            "Items".to_string(),
            NbtValue::List(vec![NbtValue::Compound(item)]),
        );

        let java = BlockEntityTranslator::translate_bedrock_to_java(&nbt);
        assert_eq!(java.get("id"), Some(&cmp("minecraft:chest")));
        let items = match java.get("Items").unwrap() {
            NbtValue::List(v) => v,
            _ => panic!(),
        };
        let first = match &items[0] {
            NbtValue::Compound(c) => c,
            _ => panic!(),
        };
        assert_eq!(first.get("id"), Some(&cmp("minecraft:cobblestone")));
    }
}
