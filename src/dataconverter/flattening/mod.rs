//! The 1.13 "Flattening" (`HelperBlockFlatteningV1450` + the flatten converters).
//!
//! This is the single largest data transform in the chain: it maps every legacy
//! numeric `(blockId << 4) | data` block, every legacy block-state `{Name,
//! Properties}`, and every legacy `id + Damage` item onto its modern flattened
//! identifier. The data tables ([`data`]) are extracted verbatim from the Java
//! sources; this module reproduces the lookup/default logic of
//! `HelperBlockFlatteningV1450`, `ConverterFlattenItemStack`,
//! `ConverterFlattenSpawnEgg`, and `ConverterFlattenEntity`.
//!
//! The registrations that *use* these tables live in `versions/v1450.rs` and
//! `versions/v1451.rs`.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::nbt::NbtMap;

use super::types::{MapExt, ValueExt};

mod data;

/// Placeholder name DataConverter emits for legacy skull *blocks* — their modern
/// id (`skeleton_skull` / `wither_skeleton_skull` / …) can only be resolved from
/// the per-position block-entity `SkullType`, which the chunk pipeline does and
/// the standalone block-state path cannot. `getNBTForId`/`flatten_nbt` return it
/// verbatim (faithful to the library); the block-state apply boundary maps it to
/// air, exactly as `ConverterFlattenChunk` does when inserting into a palette.
pub const FILTER_ME: &str = "%%FILTER_ME%%";

/// A flattened block state: a name and its (sorted) string properties.
#[derive(Clone, Copy)]
pub struct FlatState {
    pub name: &'static str,
    pub props: &'static [(&'static str, &'static str)],
}

impl FlatState {
    /// Materialize this state as a `{Name, Properties?}` NBT compound.
    pub fn to_nbt(&self) -> NbtMap {
        let mut m = NbtMap::new();
        m.set_string("Name", self.name);
        if !self.props.is_empty() {
            let mut props = NbtMap::new();
            for (k, v) in self.props {
                props.set_string(k, *v);
            }
            m.set_map("Properties", props);
        }
        m
    }
}

/// One `register(id, flattened, preFlattenings…)` row.
pub struct Registration {
    pub id: u16,
    pub flat: FlatState,
    pub pres: &'static [FlatState],
}

/// Canonical, order-independent key for a block state — mirrors NBT compound
/// equality of `{Name, Properties}` (the only keys a pre-flattening state has).
fn canon_key(name: &str, props: &[(&str, &str)]) -> String {
    let mut sorted: Vec<(&str, &str)> = props.to_vec();
    sorted.sort_unstable();
    let mut s = String::with_capacity(name.len() + props.len() * 8);
    s.push_str(name);
    for (k, v) in sorted {
        s.push('\u{1}');
        s.push_str(k);
        s.push('\u{2}');
        s.push_str(v);
    }
    s
}

/// Build the same key from a runtime block-state map. Returns `None` (no match)
/// if the map has any key other than `Name`/`Properties` or any non-string
/// property value — such a compound could not equal a registered pre-flattening.
fn canon_key_from_map(m: &NbtMap) -> Option<String> {
    let mut name: Option<&str> = None;
    let mut props: Vec<(&str, &str)> = Vec::new();
    for (k, v) in m.iter() {
        match k.as_str() {
            "Name" => name = Some(v.as_str()?),
            "Properties" => {
                for (pk, pv) in v.as_compound_ref()?.iter() {
                    props.push((pk.as_str(), pv.as_str()?));
                }
            }
            _ => return None,
        }
    }
    Some(canon_key(name?, &props))
}

struct Tables {
    /// `FLATTENED_BY_ID` after BLOCK_DEFAULTS fill + `finalizeMaps` (length 4096).
    flattened_by_id: Vec<Option<FlatState>>,
    /// `ID_BY_OLD_NBT`: canonical pre-flattening state -> id.
    id_by_old_nbt: HashMap<String, u16>,
    /// `ID_BY_OLD_NAME`: pre-flattening name -> lowest id (putIfAbsent).
    id_by_old_name: HashMap<&'static str, u16>,

    // --- reverse (un-flattening) tables, built from the same registrations ---
    /// Exact flattened `{Name, Properties}` (canonical key) -> that
    /// registration's pre-flattening states. First registration wins on
    /// collision; the best-matching pre is chosen at reverse time.
    pre_by_flat: HashMap<String, &'static [FlatState]>,
    /// Flattened block *name* -> every `(flat_props, pres)` registered under it,
    /// so a modern state carrying extra properties (waterlogged, shape, …) the
    /// flattening never set can still be matched by property subset.
    variants_by_flat_name: HashMap<
        &'static str,
        Vec<(
            &'static [(&'static str, &'static str)],
            &'static [FlatState],
        )>,
    >,
    /// Flattened block name -> canonical pre-flattening name (inverse of
    /// `getNewBlockName`). First registration wins.
    old_name_by_new: HashMap<&'static str, &'static str>,
}

static TABLES: LazyLock<Tables> = LazyLock::new(|| {
    let mut flattened_by_id: Vec<Option<FlatState>> = vec![None; 4096];
    // Indexed by block = id >> 4.
    let mut block_defaults: Vec<Option<FlatState>> = vec![None; 256];
    let mut id_by_old_nbt: HashMap<String, u16> = HashMap::new();
    let mut id_by_old_name: HashMap<&'static str, u16> = HashMap::new();

    let mut pre_by_flat: HashMap<String, &'static [FlatState]> = HashMap::new();
    let mut variants_by_flat_name: HashMap<
        &'static str,
        Vec<(
            &'static [(&'static str, &'static str)],
            &'static [FlatState],
        )>,
    > = HashMap::new();
    let mut old_name_by_new: HashMap<&'static str, &'static str> = HashMap::new();

    // Registrations are in ascending-id source order, so the first state seen for
    // a block is its default and the lowest id wins for a name (putIfAbsent).
    for r in data::REGISTRATIONS {
        flattened_by_id[r.id as usize] = Some(r.flat);
        let block = (r.id >> 4) as usize;
        if block_defaults[block].is_none() {
            block_defaults[block] = Some(r.flat);
        }
        for pre in r.pres {
            id_by_old_name.entry(pre.name).or_insert(r.id);
            id_by_old_nbt.insert(canon_key(pre.name, pre.props), r.id);
        }

        // Reverse: keep this registration's full preimage list so the closest
        // pre to a given modern state can be chosen at reverse time.
        if let Some(first) = r.pres.first() {
            pre_by_flat
                .entry(canon_key(r.flat.name, r.flat.props))
                .or_insert(r.pres);
            variants_by_flat_name
                .entry(r.flat.name)
                .or_default()
                .push((r.flat.props, r.pres));
            old_name_by_new.entry(r.flat.name).or_insert(first.name);
        }
    }

    // finalizeMaps: every empty slot falls back to its block default.
    for i in 0..4096 {
        if flattened_by_id[i].is_none() {
            flattened_by_id[i] = block_defaults[i >> 4];
        }
    }

    // Match the most specific variant first (most flat-props), so a stairs state
    // resolves to the registration that pins facing+half rather than a barer one.
    for variants in variants_by_flat_name.values_mut() {
        variants.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    }

    Tables {
        flattened_by_id,
        id_by_old_nbt,
        id_by_old_name,
        pre_by_flat,
        variants_by_flat_name,
        old_name_by_new,
    }
});

/// `getNBTForIdRaw`: `FLATTENED_BY_ID[block]` or `None` if out of range/empty.
fn nbt_for_id_raw(block: i32) -> Option<FlatState> {
    if block >= 0 && (block as usize) < 4096 {
        TABLES.flattened_by_id[block as usize]
    } else {
        None
    }
}

/// `HelperBlockFlatteningV1450.flattenNBT`: flatten a legacy block-state map.
/// Returns the new `{Name, Properties?}` compound, or `None` when the state is
/// not a known pre-flattening (caller keeps the original, like Java returning
/// `old`). May carry the [`FILTER_ME`] placeholder name for skulls.
pub fn flatten_nbt(old: &NbtMap) -> Option<NbtMap> {
    let key = canon_key_from_map(old)?;
    let id = *TABLES.id_by_old_nbt.get(&key)?;
    nbt_for_id_raw(id as i32).map(|fs| fs.to_nbt())
}

/// `HelperBlockFlatteningV1450.getNewBlockName`: legacy block *name* -> new name
/// (returns the input unchanged when unknown).
pub fn get_new_block_name(old: &str) -> String {
    match TABLES.id_by_old_name.get(old) {
        Some(&id) => nbt_for_id_raw(id as i32)
            .map(|fs| fs.name.to_string())
            .unwrap_or_else(|| old.to_string()),
        None => old.to_string(),
    }
}

/// `HelperBlockFlatteningV1450.getNameForId`: numeric block -> new name
/// (default `minecraft:air`).
pub fn get_name_for_id(block: i32) -> String {
    match nbt_for_id_raw(block) {
        Some(fs) => fs.name.to_string(),
        None => "minecraft:air".to_string(),
    }
}

/// `HelperBlockFlatteningV1450.getNBTForId`: numeric block -> flattened state
/// (default = id 0 = air; never null).
pub fn get_nbt_for_id(block: i32) -> NbtMap {
    match nbt_for_id_raw(block) {
        Some(fs) => fs.to_nbt(),
        // FLATTENED_BY_ID[0] is air (registered at id 0).
        None => nbt_for_id_raw(0)
            .expect("id 0 (air) is always registered")
            .to_nbt(),
    }
}

// --- reverse: un-flattening (new -> old block state) ------------------------

/// Outcome of reversing a flattened block state to its pre-1.13 form.
pub enum Unflatten {
    /// The modern state is exactly a known flattening output; reversed losslessly.
    Exact(NbtMap),
    /// Matched the flattened name plus a subset of its properties; extra
    /// modern-only properties (e.g. `waterlogged`, `shape`) were dropped because
    /// the pre-1.13 block could not represent them.
    Approximated(NbtMap),
    /// The flattened block name has no pre-1.13 representation (a block added in
    /// 1.13 or later). The caller should keep the modern name and report a loss.
    Unknown,
}

/// Inverse of [`flatten_nbt`]: map a modern flattened `{Name, Properties}` back
/// to its canonical pre-1.13 `{Name, Properties}` form.
///
/// Tries an exact match first (covers every block with no modern-only
/// properties — wool, concrete, the stone/dirt/sandstone variants, …), then
/// falls back to matching the flattened name against the most specific
/// registered property subset (covers stairs/slabs/logs/etc. whose modern state
/// carries extra properties the Flattening never set).
pub fn unflatten_nbt(modern: &NbtMap) -> Unflatten {
    let modern_props: Vec<(&str, &str)> = match modern.get_map("Properties") {
        Some(p) => p
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.as_str(), s)))
            .collect(),
        None => Vec::new(),
    };

    if let Some(key) = canon_key_from_map(modern) {
        if let Some(pres) = TABLES.pre_by_flat.get(&key) {
            return Unflatten::Exact(best_pre(pres, &modern_props).to_nbt());
        }
    }

    let name = match modern.get_string("Name") {
        Some(n) => n,
        None => return Unflatten::Unknown,
    };

    if let Some(variants) = TABLES.variants_by_flat_name.get(name) {
        // `variants` is sorted most-specific-first, so the first subset match is
        // the registration that pins the most properties.
        for (flat_props, pres) in variants {
            let is_subset = flat_props
                .iter()
                .all(|(k, v)| modern_props.iter().any(|(mk, mv)| mk == k && mv == v));
            if is_subset {
                return Unflatten::Approximated(best_pre(pres, &modern_props).to_nbt());
            }
        }
    }

    Unflatten::Unknown
}

/// Among a registration's preimages, pick the one closest to the modern state:
/// the pre whose properties match the most of `modern_props`. This is what
/// recovers e.g. a stair's real `shape` (the Flattening collapses all shapes to
/// `straight` but lists every shape as a preimage). Ties prefer the earliest
/// (lowest-id, putIfAbsent-style) preimage.
fn best_pre(pres: &'static [FlatState], modern_props: &[(&str, &str)]) -> &'static FlatState {
    let score = |pre: &FlatState| -> usize {
        pre.props
            .iter()
            .filter(|(k, v)| modern_props.iter().any(|(mk, mv)| mk == k && mv == v))
            .count()
    };
    // Prefer the earliest preimage on a tie (so the loop only replaces on a
    // strictly higher score).
    let mut best = &pres[0];
    let mut best_score = score(best);
    for pre in &pres[1..] {
        let s = score(pre);
        if s > best_score {
            best = pre;
            best_score = s;
        }
    }
    best
}

/// Inverse of [`get_new_block_name`]: modern block name -> canonical pre-1.13
/// name (returns the input unchanged when there is no known older name).
pub fn get_old_block_name(new: &str) -> String {
    TABLES
        .old_name_by_new
        .get(new)
        .map(|s| s.to_string())
        .unwrap_or_else(|| new.to_string())
}

/// Inverse of `getNBTForId((id << 4) | data)`: a modern flattened `{Name,
/// Properties}` block state -> the legacy numeric `(blockId << 4) | blockData`
/// index, or `None` if it has no pre-1.13 form. Used by the piston / falling
/// block / display-tile reverses that wrote numeric block ids. Reuses
/// [`unflatten_nbt`] to get the canonical pre, then maps it through
/// `ID_BY_OLD_NBT`.
pub fn flat_to_numeric(modern: &NbtMap) -> Option<i32> {
    let old = match unflatten_nbt(modern) {
        Unflatten::Exact(o) | Unflatten::Approximated(o) => o,
        Unflatten::Unknown => return None,
    };
    let key = canon_key_from_map(&old)?;
    TABLES.id_by_old_nbt.get(&key).map(|&id| id as i32)
}

// --- item flattening (ConverterFlattenItemStack) ----------------------------

static FLATTEN_ITEM: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| data::FLATTEN_ITEM_MAP.iter().copied().collect());

/// `IDS_REQUIRING_FLATTENING`: the id prefixes (before `.`) of every FLATTEN_MAP
/// key.
static IDS_REQUIRING_FLATTENING: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    data::FLATTEN_ITEM_MAP
        .iter()
        .map(|(k, _)| &k[..k.find('.').expect("FLATTEN_MAP key has a '.'")])
        .collect()
});

static ITEMS_WITH_DAMAGE: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| data::ITEMS_WITH_DAMAGE.iter().copied().collect());

/// `ConverterFlattenItemStack.flattenItem`: legacy item id + damage -> new id, or
/// `None` if the id is not a subtype-bearing (flattening) id.
pub fn flatten_item(old_name: &str, data: i32) -> Option<&'static str> {
    if !IDS_REQUIRING_FLATTENING.contains(old_name) {
        return None;
    }
    let exact = FLATTEN_ITEM
        .get(format!("{old_name}.{data}").as_str())
        .copied();
    exact.or_else(|| FLATTEN_ITEM.get(format!("{old_name}.0").as_str()).copied())
}

/// Inverse of [`flatten_item`]: flattened item id -> the legacy `(id, Damage)`
/// preimage. The forward `FLATTEN_MAP` is unique-valued, so this is exact for
/// every subtype item (e.g. `minecraft:red_wool` -> (`minecraft:wool`, 14)).
/// `None` when the modern id was not produced by item flattening.
static UNFLATTEN_ITEM: LazyLock<HashMap<&'static str, (&'static str, i32)>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, (&'static str, i32)> = HashMap::new();
    for (key, new_id) in data::FLATTEN_ITEM_MAP {
        // key is "<old_id>.<data>"; split on the final '.'.
        if let Some((old_id, data_str)) = key.rsplit_once('.') {
            if let Ok(data) = data_str.parse::<i32>() {
                m.entry(new_id).or_insert((old_id, data));
            }
        }
    }
    m
});

/// Flattened item id -> legacy `(old_id, Damage)`; see [`UNFLATTEN_ITEM`].
pub fn unflatten_item(new_id: &str) -> Option<(&'static str, i32)> {
    UNFLATTEN_ITEM.get(new_id).copied()
}

/// Whether this id is in `IDS_REQUIRING_FLATTENING` (its Damage is a subtype).
pub fn id_requires_flattening(id: &str) -> bool {
    IDS_REQUIRING_FLATTENING.contains(id)
}

/// `ITEMS_WITH_DAMAGE`: ids whose Damage is real durability (migrated to tag).
pub fn item_has_damage(id: &str) -> bool {
    ITEMS_WITH_DAMAGE.contains(id)
}

// --- spawn-egg flattening (ConverterFlattenSpawnEgg) ------------------------

static SPAWN_EGG: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| data::SPAWN_EGG_MAP.iter().copied().collect());

/// `ENTITY_ID_TO_NEW_EGG_ID.getOrDefault(id, "minecraft:pig_spawn_egg")`.
pub fn spawn_egg_for_entity(id: &str) -> &'static str {
    SPAWN_EGG
        .get(id)
        .copied()
        .unwrap_or("minecraft:pig_spawn_egg")
}

/// Inverse of [`spawn_egg_for_entity`]: a flattened spawn-egg id -> the entity id
/// it spawns (first-wins on the rare collision). `None` for an unknown egg id.
static ENTITY_BY_SPAWN_EGG: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, &'static str> = HashMap::new();
    for (entity, egg) in data::SPAWN_EGG_MAP {
        m.entry(*egg).or_insert(*entity);
    }
    m
});

/// Flattened spawn-egg id -> entity id; see [`ENTITY_BY_SPAWN_EGG`].
pub fn entity_for_spawn_egg(egg_id: &str) -> Option<&'static str> {
    ENTITY_BY_SPAWN_EGG.get(egg_id).copied()
}

// --- entity block-id table (ConverterFlattenEntity) -------------------------

static ENTITY_BLOCK_NAME_TO_ID: LazyLock<HashMap<&'static str, i32>> =
    LazyLock::new(|| data::ENTITY_BLOCK_NAME_TO_ID.iter().copied().collect());

/// `ConverterFlattenEntity.getBlockId`: legacy block name -> numeric id
/// (default 0).
pub fn entity_block_id(name: &str) -> i32 {
    ENTITY_BLOCK_NAME_TO_ID.get(name).copied().unwrap_or(0)
}

// --- numeric item id -> name (HelperItemNameV102, jukebox Record) -----------

static ITEM_NAMES_BY_ID: LazyLock<HashMap<i32, &'static str>> =
    LazyLock::new(|| data::ITEM_NAMES_BY_ID.iter().copied().collect());

/// `HelperItemNameV102.getNameFromId`: numeric item id -> name (nullable).
pub fn get_name_from_id(id: i32) -> Option<&'static str> {
    ITEM_NAMES_BY_ID.get(&id).copied()
}

/// Inverse of [`get_name_from_id`]: item name -> numeric id (first-wins). Used by
/// the V102 numeric-id and the jukebox `Record` reverses.
static ID_BY_ITEM_NAME: LazyLock<HashMap<&'static str, i32>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, i32> = HashMap::new();
    for (id, name) in data::ITEM_NAMES_BY_ID {
        m.entry(*name).or_insert(*id);
    }
    m
});

/// Item name -> numeric id; see [`ID_BY_ITEM_NAME`].
pub fn id_from_item_name(name: &str) -> Option<i32> {
    ID_BY_ITEM_NAME.get(name).copied()
}

static POTION_NAMES_BY_ID: LazyLock<HashMap<i32, &'static str>> =
    LazyLock::new(|| data::POTION_NAMES_BY_ID.iter().copied().collect());

/// `HelperItemNameV102.getPotionNameFromId`: numeric potion id -> name. Mirrors
/// Java's `POTION_NAMES[id & 127]` (nullable).
pub fn get_potion_name_from_id(id: i16) -> Option<&'static str> {
    POTION_NAMES_BY_ID.get(&((id & 127) as i32)).copied()
}

static SPAWN_EGG_NAME_BY_ID: LazyLock<HashMap<i32, &'static str>> =
    LazyLock::new(|| data::SPAWN_EGG_NAME_BY_ID.iter().copied().collect());

/// `HelperSpawnEggNameV105.getSpawnNameFromId`: legacy spawn-egg damage -> entity
/// name. Mirrors Java's `ID_TO_STRING[id & 255]` (nullable).
pub fn get_spawn_name_from_id(id: i16) -> Option<&'static str> {
    SPAWN_EGG_NAME_BY_ID.get(&((id & 255) as i32)).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_state_flatten_and_default() {
        // exact pre-flattening
        let mut stone = NbtMap::new();
        stone.set_string("Name", "minecraft:stone");
        let mut props = NbtMap::new();
        props.set_string("variant", "granite");
        stone.set_map("Properties", props);
        let out = flatten_nbt(&stone).expect("granite flattens");
        assert_eq!(out.get_string("Name"), Some("minecraft:granite"));
        assert!(out.get_map("Properties").is_none());

        // unknown state -> None (kept as-is)
        let mut weird = NbtMap::new();
        weird.set_string("Name", "minecraft:not_a_block");
        assert!(flatten_nbt(&weird).is_none());
    }

    #[test]
    fn numeric_id_lookups() {
        // id 16 = stone, id 17 = granite
        assert_eq!(get_name_for_id(16), "minecraft:stone");
        assert_eq!(get_name_for_id(17), "minecraft:granite");
        // out of range -> air
        assert_eq!(get_name_for_id(99999), "minecraft:air");
        assert_eq!(get_name_for_id(-1), "minecraft:air");
        // getNBTForId default is air
        assert_eq!(get_nbt_for_id(-1).get_string("Name"), Some("minecraft:air"));
        // id 128 = water level 0 (block 8 = flowing_water defaults), id 144*... not used
        assert_eq!(
            get_nbt_for_id(128).get_string("Name"),
            Some("minecraft:water")
        );
    }

    #[test]
    fn block_default_fill() {
        // id 16 (stone) registered; id 16+15=31 unregistered -> block default (16).
        assert_eq!(get_name_for_id(31), "minecraft:stone");
    }

    #[test]
    fn skull_uses_filter_me_placeholder() {
        // id 2304 = skull facing down -> FILTER_ME (resolved to air at apply layer)
        assert_eq!(get_name_for_id(2304), FILTER_ME);
    }

    #[test]
    fn name_lookup() {
        // legacy name that was renamed by the Flattening: grass -> grass_block.
        assert_eq!(
            get_new_block_name("minecraft:grass"),
            "minecraft:grass_block"
        );
        // unknown name is returned unchanged.
        assert_eq!(
            get_new_block_name("minecraft:unknown_zzz"),
            "minecraft:unknown_zzz"
        );
    }

    #[test]
    fn item_flatten() {
        assert_eq!(
            flatten_item("minecraft:wool", 14),
            Some("minecraft:red_wool")
        );
        assert_eq!(
            flatten_item("minecraft:wool", 99),
            Some("minecraft:white_wool")
        ); // .0 fallback
        assert_eq!(flatten_item("minecraft:diamond_sword", 0), None); // not a subtype id
        assert!(item_has_damage("minecraft:diamond_sword"));
        assert!(!item_has_damage("minecraft:wool"));
    }

    #[test]
    fn spawn_egg_and_entity_block_id() {
        assert_eq!(
            spawn_egg_for_entity("minecraft:creeper"),
            "minecraft:creeper_spawn_egg"
        );
        assert_eq!(
            spawn_egg_for_entity("minecraft:unknown"),
            "minecraft:pig_spawn_egg"
        );
        assert_eq!(entity_block_id("minecraft:chest"), 54);
        assert_eq!(entity_block_id("minecraft:unknown"), 0);
    }

    #[test]
    fn unflatten_exact_round_trips_forward() {
        // granite: stone{variant=granite} -> granite (no props) and back.
        let mut stone = NbtMap::new();
        stone.set_string("Name", "minecraft:stone");
        let mut props = NbtMap::new();
        props.set_string("variant", "granite");
        stone.set_map("Properties", props);
        let flat = flatten_nbt(&stone).expect("granite flattens");

        match unflatten_nbt(&flat) {
            Unflatten::Exact(old) => {
                assert_eq!(old.get_string("Name"), Some("minecraft:stone"));
                assert_eq!(
                    old.get_map("Properties").unwrap().get_string("variant"),
                    Some("granite")
                );
            }
            _ => panic!("granite should reverse exactly"),
        }
    }

    #[test]
    fn unflatten_drops_modern_only_props_via_subset() {
        // A modern oak_stairs state carrying shape/waterlogged the Flattening
        // never set must still match by property subset.
        let mut stairs = NbtMap::new();
        stairs.set_string("Name", "minecraft:oak_stairs");
        let mut props = NbtMap::new();
        props.set_string("facing", "east");
        props.set_string("half", "bottom");
        props.set_string("shape", "straight");
        props.set_string("waterlogged", "false");
        stairs.set_map("Properties", props);

        // `shape` is a real 1.12 stairs property (kept); `waterlogged` is the
        // 1.13 addition the Flattening never set (dropped on downgrade).
        match unflatten_nbt(&stairs) {
            Unflatten::Exact(old) | Unflatten::Approximated(old) => {
                assert_eq!(old.get_string("Name"), Some("minecraft:oak_stairs"));
                let p = old
                    .get_map("Properties")
                    .expect("stairs keep facing/half/shape");
                assert_eq!(p.get_string("facing"), Some("east"));
                assert_eq!(p.get_string("half"), Some("bottom"));
                assert_eq!(p.get_string("shape"), Some("straight"));
                assert!(
                    p.get_string("waterlogged").is_none(),
                    "modern-only waterlogged dropped"
                );
            }
            Unflatten::Unknown => panic!("oak_stairs should reverse"),
        }
    }

    #[test]
    fn unflatten_unknown_modern_block() {
        // A block introduced after 1.13 cannot be represented pre-Flattening.
        let mut blackstone = NbtMap::new();
        blackstone.set_string("Name", "minecraft:blackstone");
        assert!(matches!(unflatten_nbt(&blackstone), Unflatten::Unknown));
    }

    #[test]
    fn unflatten_item_inverts_subtype() {
        assert_eq!(
            flatten_item("minecraft:wool", 14),
            Some("minecraft:red_wool")
        );
        assert_eq!(
            unflatten_item("minecraft:red_wool"),
            Some(("minecraft:wool", 14))
        );
        assert_eq!(unflatten_item("minecraft:diamond_sword"), None);
    }
}
