//! JSON views over the block table, backing the bridge `Blocks` query API
//! (`src/bridge/blocks.rs`). Every function returns a ready-to-emit JSON
//! string; list results are sorted by block id so output is deterministic
//! across the phf table's arbitrary iteration order.

use serde_json::{json, Map, Value};

use super::{
    all_blocks, blocks_by_tag, get_block, variants_of, BlockFacts, BlockpediaError, Result,
    BLOCKS, BLOCK_TAGS,
};

/// Cap on the number of property-value combinations [`block_states_json`]
/// will enumerate. The current data tops out at 1350 (`minecraft:note_block`),
/// so no vanilla block hits this — it guards against pathological output if a
/// future data refresh adds a combinatorial monster.
pub const MAX_STATE_COMBINATIONS: usize = 4096;

/// Normalize a user-supplied block/kind name to the stored
/// `minecraft:`-prefixed form (`oak_stairs` -> `minecraft:oak_stairs`).
fn normalize_mc_name(name: &str) -> String {
    if name.contains(':') {
        name.to_string()
    } else {
        format!("minecraft:{name}")
    }
}

/// Look up a block accepting both `minecraft:oak_stairs` and short
/// `oak_stairs` forms.
fn get_block_normalized(id: &str) -> Option<&'static BlockFacts> {
    get_block(&normalize_mc_name(id))
}

fn facts_value(f: &BlockFacts) -> Value {
    let mut properties = Map::new();
    for (name, values) in f.properties {
        properties.insert(name.to_string(), json!(values));
    }
    let mut default_state = Map::new();
    for (name, value) in f.default_state {
        default_state.insert(name.to_string(), Value::String(value.to_string()));
    }
    json!({
        "id": f.id,
        "kind": f.kind,
        "base_block": f.base_block,
        "tags": f.tags,
        "full_cube": f.full_cube,
        "transparent": f.transparent,
        "color": f.extras.color.as_ref().map(|c| c.rgb),
        "properties": properties,
        "default_state": default_state,
    })
}

/// Full facts for one block as a JSON object (`None` for unknown ids):
/// `{id, kind, base_block, tags, full_cube, transparent, color,
/// properties, default_state}`. Accepts short and `minecraft:`-prefixed ids.
pub fn block_facts_json(id: &str) -> Option<String> {
    get_block_normalized(id).map(|f| facts_value(f).to_string())
}

/// All known block ids as a sorted JSON array string.
pub fn all_block_ids_json() -> String {
    let mut ids: Vec<&str> = BLOCKS.keys().copied().collect();
    ids.sort_unstable();
    json!(ids).to_string()
}

/// Ids of every block carrying the vanilla tag, as a sorted JSON array
/// string (empty array for unknown tags). Accepts `minecraft:wool` and
/// short `wool` forms, including nested paths like `mineable/pickaxe`.
pub fn block_ids_by_tag_json(tag: &str) -> String {
    let mut ids: Vec<&str> = blocks_by_tag(tag).map(|b| b.id).collect();
    ids.sort_unstable();
    json!(ids).to_string()
}

/// Ids of every block of the given official definition kind
/// (`minecraft:stair`, short `stair`, ...), as a sorted JSON array string
/// (empty array for unknown kinds).
pub fn block_ids_by_kind_json(kind: &str) -> String {
    let key = normalize_mc_name(kind);
    let mut ids: Vec<&str> = all_blocks()
        .filter(|b| b.kind == key)
        .map(|b| b.id)
        .collect();
    ids.sort_unstable();
    json!(ids).to_string()
}

/// The base block followed by all its shape variants (blocks whose
/// `base_block` is `base_id`), as a JSON array string — the base is always
/// first, variants sorted by id after it. `None` for unknown base ids.
pub fn variants_of_ids_json(base_id: &str) -> Option<String> {
    let base = get_block_normalized(base_id)?;
    let mut ids = vec![base.id];
    ids.extend(variants_of(base.id).iter().map(|b| b.id));
    Some(json!(ids).to_string())
}

/// All known vanilla block tag names as a sorted JSON array string
/// (`minecraft:`-prefixed forms, e.g. `minecraft:wool`).
pub fn all_tags_json() -> String {
    let mut tags: Vec<&str> = BLOCK_TAGS.keys().copied().collect();
    tags.sort_unstable();
    json!(tags).to_string()
}

/// Every property-value combination of the block as a JSON array of
/// `{prop: value}` objects (a single `{}` entry for property-less blocks).
/// Errors on unknown ids and when the combination count exceeds `limit`.
pub fn block_states_json_with_limit(id: &str, limit: usize) -> Result<String> {
    let f = get_block_normalized(id).ok_or_else(|| BlockpediaError::block_not_found(id))?;
    let combinations: usize = f
        .properties
        .iter()
        .map(|(_, values)| values.len().max(1))
        .product();
    if combinations > limit {
        return Err(BlockpediaError::custom(format!(
            "{}: {} property-value combinations exceed the limit of {}",
            f.id, combinations, limit
        )));
    }
    let mut states: Vec<Map<String, Value>> = vec![Map::new()];
    for (name, values) in f.properties {
        let mut next = Vec::with_capacity(states.len() * values.len());
        for state in &states {
            for value in *values {
                let mut s = state.clone();
                s.insert(name.to_string(), Value::String(value.to_string()));
                next.push(s);
            }
        }
        states = next;
    }
    Ok(Value::Array(states.into_iter().map(Value::Object).collect()).to_string())
}

/// [`block_states_json_with_limit`] with the default
/// [`MAX_STATE_COMBINATIONS`] guard.
pub fn block_states_json(id: &str) -> Result<String> {
    block_states_json_with_limit(id, MAX_STATE_COMBINATIONS)
}

/// Total number of blocks in the table.
pub fn block_count() -> usize {
    BLOCKS.len()
}
