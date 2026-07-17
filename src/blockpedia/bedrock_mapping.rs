use phf::phf_map;

/// Dynamic blockstate mapping from PrismarineJS blocksJ2B.json
/// Maps Java blockstate strings to Bedrock blockstate strings
/// Format: "minecraft:block_id[prop1=val1,prop2=val2]" -> "minecraft:bedrock_id[prop1=val1,prop2=val2]"
pub struct BedrockBlockStateMapper;

impl BedrockBlockStateMapper {
    /// Convert a Java blockstate string to Bedrock blockstate string using the mapping
    pub fn java_to_bedrock(blockstate_str: &str) -> Option<&'static str> {
        // Parse the blockstate string to normalize property order
        let normalized = Self::normalize_blockstate(blockstate_str);
        BEDROCK_J2B_MAP.get(&normalized).copied()
    }

    /// Convert a Bedrock blockstate string to Java blockstate string using the mapping
    pub fn bedrock_to_java(blockstate_str: &str) -> Option<&'static str> {
        // Parse the blockstate string to normalize property order
        let normalized = Self::normalize_blockstate(blockstate_str);
        BEDROCK_B2J_MAP.get(&normalized).copied()
    }

    /// Normalize a blockstate string by sorting properties alphabetically
    /// This ensures consistent matching regardless of property order
    fn normalize_blockstate(blockstate: &str) -> String {
        if let Some(bracket_pos) = blockstate.find('[') {
            let block_id = &blockstate[..bracket_pos];
            let props_str = &blockstate[bracket_pos + 1..];

            if props_str.ends_with(']') {
                let props_str = &props_str[..props_str.len() - 1];
                if props_str.is_empty() {
                    return format!("{}[]", block_id);
                }

                let mut props: Vec<&str> = props_str.split(',').collect();
                props.sort();
                format!("{}[{}]", block_id, props.join(","))
            } else {
                blockstate.to_string()
            }
        } else {
            format!("{}[]", blockstate)
        }
    }
}

// These maps will be generated at build time from blocksJ2B.json and blocksB2J.json
// For now, we'll use a placeholder that will be replaced during build
include!(concat!(env!("OUT_DIR"), "/bedrock_mappings.rs"));
