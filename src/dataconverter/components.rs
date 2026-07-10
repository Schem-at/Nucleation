//! The 1.20.5 item "components squash" — `ConverterItemStackToDataComponents`.
//!
//! Pre-1.20.5 items stored their extra data in a free-form `tag` compound; 1.20.5
//! replaced that with a typed `components` map (and any leftover `tag` becomes
//! `minecraft:custom_data`). This is a faithful port of the Java converter,
//! including the `TransientItemStack` working set (root / tag / components split)
//! and every per-item special case. It is the schematic-critical half of the
//! 1.20.5 transform: items inside containers, item frames, etc. survive the
//! version boundary instead of being dropped.

use crate::nbt::{NbtMap, NbtValue};

use super::helpers::{correct_namespace, create_translatable_component, is_valid_json};
use super::loss::{report_loss, LossKind, Severity};
use super::types::{MapExt, ValueExt};

const TOOLTIP_FLAG_HIDE_ENCHANTMENTS: i32 = 1 << 0;
const TOOLTIP_FLAG_HIDE_MODIFIERS: i32 = 1 << 1;
const TOOLTIP_FLAG_HIDE_UNBREAKABLE: i32 = 1 << 2;
const TOOLTIP_FLAG_HIDE_CAN_DESTROY: i32 = 1 << 3;
const TOOLTIP_FLAG_HIDE_CAN_PLACE: i32 = 1 << 4;
const TOOLTIP_FLAG_HIDE_ADDITIONAL: i32 = 1 << 5;
const TOOLTIP_FLAG_HIDE_DYE: i32 = 1 << 6;
const TOOLTIP_FLAG_HIDE_UPGRADES: i32 = 1 << 7;

const DEFAULT_LEATHER_COLOUR: i32 = (160 << 16) | (101 << 8) | 64;

const BANNER_COLOURS: [&str; 16] = [
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
];

/// `V3818.getBannerColour` — colour index -> name (default white).
pub fn banner_colour(id: i32) -> &'static str {
    if id >= 0 && (id as usize) < BANNER_COLOURS.len() {
        BANNER_COLOURS[id as usize]
    } else {
        BANNER_COLOURS[0]
    }
}

const BUCKETED_MOB_TAGS: [&str; 10] = [
    "NoAI",
    "Silent",
    "NoGravity",
    "Glowing",
    "Invulnerable",
    "Health",
    "Age",
    "Variant",
    "HuntingCooldown",
    "BucketVariantTag",
];

const BOOLEAN_BLOCK_STATE_PROPERTIES: [&str; 47] = [
    "attached",
    "bottom",
    "conditional",
    "disarmed",
    "drag",
    "enabled",
    "extended",
    "eye",
    "falling",
    "hanging",
    "has_bottle_0",
    "has_bottle_1",
    "has_bottle_2",
    "has_record",
    "has_book",
    "inverted",
    "in_wall",
    "lit",
    "locked",
    "occupied",
    "open",
    "persistent",
    "powered",
    "short",
    "signal_fire",
    "snowy",
    "triggered",
    "unstable",
    "waterlogged",
    "berries",
    "bloom",
    "shrieking",
    "can_summon",
    "up",
    "down",
    "north",
    "east",
    "south",
    "west",
    "slot_0_occupied",
    "slot_1_occupied",
    "slot_2_occupied",
    "slot_3_occupied",
    "slot_4_occupied",
    "slot_5_occupied",
    "cracked",
    "crafting",
];

const MAP_DECORATION_CONVERSION_TABLE: [&str; 34] = [
    "player",
    "frame",
    "red_marker",
    "blue_marker",
    "target_x",
    "target_point",
    "player_off_map",
    "player_off_limits",
    "mansion",
    "monument",
    "banner_white",
    "banner_orange",
    "banner_magenta",
    "banner_light_blue",
    "banner_yellow",
    "banner_lime",
    "banner_pink",
    "banner_gray",
    "banner_light_gray",
    "banner_cyan",
    "banner_purple",
    "banner_blue",
    "banner_brown",
    "banner_green",
    "banner_red",
    "banner_black",
    "red_x",
    "village_desert",
    "village_plains",
    "village_savanna",
    "village_snowy",
    "village_taiga",
    "jungle_temple",
    "swamp_hut",
];

fn convert_map_decoration_id(type_id: i32) -> &'static str {
    if type_id >= 0 && (type_id as usize) < MAP_DECORATION_CONVERSION_TABLE.len() {
        MAP_DECORATION_CONVERSION_TABLE[type_id as usize]
    } else {
        MAP_DECORATION_CONVERSION_TABLE[0]
    }
}

fn is_boolean_block_state_property(key: &str) -> bool {
    BOOLEAN_BLOCK_STATE_PROPERTIES.contains(&key)
}

/// `Number.toString()` for the variants block-state property values use.
fn number_to_string(v: &NbtValue) -> Option<String> {
    match v {
        NbtValue::Byte(n) => Some(n.to_string()),
        NbtValue::Short(n) => Some(n.to_string()),
        NbtValue::Int(n) => Some(n.to_string()),
        NbtValue::Long(n) => Some(n.to_string()),
        NbtValue::Float(n) => Some(n.to_string()),
        NbtValue::Double(n) => Some(n.to_string()),
        _ => None,
    }
}

/// `convertBlockStateProperties`: numeric property values become strings;
/// known-boolean keys become `"true"`/`"false"`.
fn convert_block_state_properties(properties: &mut NbtMap) {
    for key in properties.keys() {
        let (num, bval) = match properties.get(&key) {
            Some(v) => (number_to_string(v), v.as_number_i64()),
            None => continue,
        };
        let Some(s) = num else { continue };
        if is_boolean_block_state_property(&key) {
            properties.set_string(&key, ((bval.unwrap_or(0) as i8) != 0).to_string());
        } else {
            properties.set_string(&key, s);
        }
    }
}

/// Copy `src[src_key]` to `dst[dst_key]` if present.
fn copy(src: Option<&NbtMap>, src_key: &str, dst: &mut NbtMap, dst_key: &str) {
    if let Some(s) = src {
        if let Some(v) = s.get(src_key) {
            dst.set_generic(dst_key, v.clone());
        }
    }
}

fn is_valid_player_name(name: &str) -> bool {
    if name.chars().count() > 16 {
        return false;
    }
    name.chars().all(|c| {
        let v = c as u32;
        v > 0x20 && v < 0x7F
    })
}

/// `convertBlockStatePredicate`: parse a `name[props]{nbt}` predicate string.
pub fn convert_block_state_predicate(value: &str) -> NbtMap {
    let property_start = value.find('[');
    let nbt_start = value.find('{');
    let mut block_name_end = value.len();
    if let Some(p) = property_start {
        block_name_end = p;
    }
    if let Some(n) = nbt_start {
        block_name_end = block_name_end.min(n);
    }

    let mut ret = NbtMap::new();
    ret.set_string("blocks", value[..block_name_end].trim());

    if let Some(p) = property_start {
        if let Some(rel) = value[p + 1..].find(']') {
            let prop_end = p + 1 + rel;
            let mut state = NbtMap::new();
            for property in value[p + 1..prop_end].split(',') {
                if let Some(sep) = property.find('=') {
                    let key = property[..sep].trim();
                    let val = &property[sep + 1..];
                    state.set_string(key, val);
                }
            }
            ret.set_map("state", state);
        }
    }

    if let Some(n) = nbt_start {
        if let Some(rel) = value[n + 1..].find('}') {
            let nbt_end = n + 1 + rel;
            ret.set_string("nbt", &value[n..=nbt_end]);
        }
    }

    ret
}

/// The transient working set for one item: the surviving `root` fields, the
/// legacy `tag`, and the new `components` (`ConverterItemStackToDataComponents`'s
/// inner class).
struct TransientItemStack {
    id: String,
    count: i32,
    components: NbtMap,
    tag: NbtMap,
    root: NbtMap,
}

impl TransientItemStack {
    fn new(root: &NbtMap) -> Self {
        let id = root.get_string("id").unwrap_or("").to_string();
        let count = root.get_i32("Count").unwrap_or(0);

        let mut root_copy = root.clone();
        let tag = match root_copy.take("tag") {
            Some(NbtValue::Compound(m)) => m,
            _ => NbtMap::new(),
        };
        root_copy.take("id");
        root_copy.take("Count");

        Self {
            id,
            count,
            components: NbtMap::new(),
            tag,
            root: root_copy,
        }
    }

    fn migrate_tag_to(&mut self, tag_key: &str, dst: &mut NbtMap, dst_key: &str) {
        if let Some(v) = self.tag.take(tag_key) {
            dst.set_generic(dst_key, v);
        }
    }

    fn tag_remove_string(&mut self, key: &str) -> Option<String> {
        let r = self.tag.get_string(key).map(str::to_string);
        self.tag.take(key);
        r
    }

    fn tag_remove_list_unchecked(&mut self, key: &str) -> Option<Vec<NbtValue>> {
        match self.tag.take(key) {
            Some(NbtValue::List(l)) => Some(l),
            _ => None,
        }
    }

    fn tag_remove_list_maps(&mut self, key: &str) -> Option<Vec<NbtValue>> {
        // getList(key, MAP) keeps only list-of-compound; we keep any list and let
        // the caller skip non-compound elements.
        self.tag_remove_list_unchecked(key)
    }

    fn tag_remove_map(&mut self, key: &str) -> Option<NbtMap> {
        match self.tag.take(key) {
            Some(NbtValue::Compound(m)) => Some(m),
            _ => None,
        }
    }

    fn tag_remove_bool(&mut self, key: &str, dfl: bool) -> bool {
        let r = self.tag.get_bool(key).unwrap_or(dfl);
        self.tag.take(key);
        r
    }

    fn tag_remove_int(&mut self, key: &str, dfl: i32) -> i32 {
        let r = self.tag.get_i32(key).unwrap_or(dfl);
        self.tag.take(key);
        r
    }

    fn tag_remove_generic(&mut self, key: &str) -> Option<NbtValue> {
        self.tag.take(key)
    }

    fn tag_migrate_to_component(&mut self, tag_key: &str, component_key: &str) {
        if let Some(v) = self.tag.take(tag_key) {
            self.components.set_generic(component_key, v);
        }
    }

    fn tag_migrate_non_empty_list(&mut self, tag_key: &str, component_key: &str) {
        if let Some(v) = self.tag.take(tag_key) {
            let empty_list = matches!(&v, NbtValue::List(l) if l.is_empty());
            if !empty_list {
                self.components.set_generic(component_key, v);
            }
        }
    }

    fn tag_migrate_int(&mut self, tag_key: &str, component_key: &str, dfl: i32) {
        let value = self.tag.get_i32(tag_key).unwrap_or(dfl);
        self.tag.take(tag_key);
        if value != dfl {
            self.components.set_i32(component_key, value);
        }
    }

    fn serialize(mut self) -> NbtMap {
        let mut ret = NbtMap::new();
        ret.set_string("id", self.id);
        ret.set_i32("count", self.count);

        if !self.tag.is_empty() {
            self.components.set_map("minecraft:custom_data", self.tag);
        }
        if !self.components.is_empty() {
            ret.set_map("components", self.components);
        }

        // Merge surviving root fields, ret entries take priority.
        for key in self.root.keys() {
            if ret.has_key(&key) {
                continue;
            }
            if let Some(v) = self.root.get(&key) {
                ret.set_generic(&key, v.clone());
            }
        }

        ret
    }
}

fn convert_tile_entity(tile: &mut NbtMap, item: &mut TransientItemStack) {
    if let Some(lock) = tile.take("Lock") {
        item.components.set_generic("minecraft:lock", lock);
    }

    if let Some(loot_table) = tile.take("LootTable") {
        let mut container_loot = NbtMap::new();
        container_loot.set_generic("loot_table", loot_table);
        let seed = tile.get_i64("LootTableSeed").unwrap_or(0);
        if seed != 0 {
            container_loot.set_i64("seed", seed);
        }
        tile.take("LootTableSeed");
        item.components
            .set_map("minecraft:container_loot", container_loot);
    }

    let id = correct_namespace(tile.get_string("id").unwrap_or(""));
    match id.as_str() {
        "minecraft:skull" => {
            if let Some(v) = tile.take("note_block_sound") {
                item.components.set_generic("minecraft:note_block_sound", v);
            }
        }
        "minecraft:decorated_pot" => {
            if let Some(sherds) = tile.take("sherds") {
                item.components
                    .set_generic("minecraft:pot_decorations", sherds);
            }
            if let Some(item_val) = tile.take("item") {
                let mut wrapped = NbtMap::new();
                wrapped.set_i32("slot", 0);
                wrapped.set_generic("item", item_val);
                item.components
                    .set_list("minecraft:container", vec![NbtValue::Compound(wrapped)]);
            }
        }
        "minecraft:banner" => {
            if let Some(patterns) = tile.take("patterns") {
                item.components
                    .set_generic("minecraft:banner_patterns", patterns);
            }
            if let Some(base) = tile.get_i32("Base") {
                tile.take("Base");
                item.components
                    .set_string("minecraft:base_color", banner_colour(base));
            }
        }
        "minecraft:shulker_box"
        | "minecraft:chest"
        | "minecraft:trapped_chest"
        | "minecraft:furnace"
        | "minecraft:ender_chest"
        | "minecraft:dispenser"
        | "minecraft:dropper"
        | "minecraft:brewing_stand"
        | "minecraft:hopper"
        | "minecraft:barrel"
        | "minecraft:smoker"
        | "minecraft:blast_furnace"
        | "minecraft:campfire"
        | "minecraft:chiseled_bookshelf"
        | "minecraft:crafter" => {
            if let Some(NbtValue::List(items)) = tile.take("Items") {
                if !items.is_empty() {
                    let mut wrapped_list = Vec::with_capacity(items.len());
                    for entry in items {
                        if let NbtValue::Compound(mut it) = entry {
                            let slot = it.get_i32("Slot").unwrap_or(0) & 0xFF;
                            it.take("Slot");
                            let mut wrapped = NbtMap::new();
                            wrapped.set_i32("slot", slot);
                            wrapped.set_map("item", it);
                            wrapped_list.push(NbtValue::Compound(wrapped));
                        } else {
                            wrapped_list.push(entry);
                        }
                    }
                    item.components
                        .set_list("minecraft:container", wrapped_list);
                }
            }
        }
        "minecraft:beehive" => {
            if let Some(bees) = tile.take("bees") {
                item.components.set_generic("minecraft:bees", bees);
            }
        }
        _ => {}
    }
}

fn convert_enchantments(
    item: &mut TransientItemStack,
    tag_key: &str,
    component_key: &str,
    hide_tooltip: bool,
) {
    let enchantments = item.tag_remove_list_maps(tag_key);
    let empty_or_none = enchantments.as_ref().is_none_or(|l| l.is_empty());

    if empty_or_none {
        if hide_tooltip {
            let mut new_enchants = NbtMap::new();
            new_enchants.set_map("levels", NbtMap::new());
            new_enchants.set_bool("show_in_tooltip", false);
            item.components.set_map(component_key, new_enchants);
        }
    } else {
        let mut new_levels = NbtMap::new();
        for e in enchantments.as_ref().unwrap() {
            let Some(em) = e.as_compound_ref() else {
                continue;
            };
            let (Some(id), Some(lvl)) = (em.get_string("id"), em.get_i32("lvl")) else {
                continue;
            };
            let clamped = lvl.clamp(0, 0xFF);
            if clamped <= 0 {
                continue;
            }
            new_levels.set_i32(id, clamped);
        }

        if !new_levels.is_empty() || hide_tooltip {
            let mut new_enchants = NbtMap::new();
            new_enchants.set_map("levels", new_levels);
            if hide_tooltip {
                new_enchants.set_bool("show_in_tooltip", false);
            }
            item.components.set_map(component_key, new_enchants);
        }
    }

    if enchantments.as_ref().is_some_and(|l| l.is_empty()) {
        item.components
            .set_bool("minecraft:enchantment_glint_override", true);
    }
}

fn convert_display(item: &mut TransientItemStack, flags: i32) {
    let mut display: Option<NbtMap> = match item.tag.take("display") {
        Some(NbtValue::Compound(m)) => Some(m),
        Some(other) => {
            item.tag.set_generic("display", other);
            None
        }
        None => None,
    };

    if let Some(d) = display.as_mut() {
        let name = d.take("Name");
        if let Some(NbtValue::String(name)) = &name {
            if is_valid_json(name) {
                item.components
                    .set_string("minecraft:custom_name", name.clone());
            }
        }

        let lore = d.take("Lore");
        if let Some(NbtValue::List(lore_list)) = lore {
            let mut valid = Vec::new();
            for entry in &lore_list {
                if let NbtValue::String(s) = entry {
                    if is_valid_json(s) {
                        valid.push(NbtValue::String(s.clone()));
                    }
                }
            }
            item.components.set_list("minecraft:lore", valid);
        }
    }

    let color = display.as_ref().and_then(|d| d.get_i64("color"));
    let hide_dye = (flags & TOOLTIP_FLAG_HIDE_DYE) != 0;

    if hide_dye || color.is_some() {
        if color.is_some() {
            if let Some(d) = display.as_mut() {
                d.take("color");
            }
        }
        let mut dyed = NbtMap::new();
        dyed.set_i32(
            "rgb",
            color.map(|c| c as i32).unwrap_or(DEFAULT_LEATHER_COLOUR),
        );
        if hide_dye {
            dyed.set_bool("show_in_tooltip", false);
        }
        item.components.set_map("minecraft:dyed_color", dyed);
    }

    if let Some(NbtValue::String(loc_name)) = display.as_mut().and_then(|d| d.take("LocName")) {
        item.components.set_string(
            "minecraft:item_name",
            create_translatable_component(&loc_name),
        );
    }

    if item.id == "minecraft:filled_map" {
        if let Some(d) = display.as_mut() {
            if let Some(map_color) = d.take("MapColor") {
                item.components
                    .set_generic("minecraft:map_color", map_color);
            }
        }
    }

    // mirror fixSubTag: keep display only if it still holds anything.
    match display {
        Some(d) if !d.is_empty() => item.tag.set_map("display", d),
        _ => {}
    }
}

fn convert_block_state_predicates(
    item: &mut TransientItemStack,
    tag_key: &str,
    component_key: &str,
    hide_in_tooltip: bool,
) {
    let Some(blocks) = item.tag_remove_list_unchecked(tag_key) else {
        return;
    };

    let mut block_predicates = NbtMap::new();
    if hide_in_tooltip {
        block_predicates.set_bool("show_in_tooltip", false);
    }

    let mut predicates = Vec::with_capacity(blocks.len());
    for block in blocks {
        if let NbtValue::String(s) = &block {
            predicates.push(NbtValue::Compound(convert_block_state_predicate(s)));
        } else {
            predicates.push(block);
        }
    }
    block_predicates.set_list("predicates", predicates);
    item.components.set_map(component_key, block_predicates);
}

fn convert_adventure_mode(item: &mut TransientItemStack, flags: i32) {
    convert_block_state_predicates(
        item,
        "CanDestroy",
        "minecraft:can_break",
        (flags & TOOLTIP_FLAG_HIDE_CAN_DESTROY) != 0,
    );
    convert_block_state_predicates(
        item,
        "CanPlaceOn",
        "minecraft:can_place_on",
        (flags & TOOLTIP_FLAG_HIDE_CAN_PLACE) != 0,
    );
}

fn convert_attribute(input_generic: &NbtValue) -> NbtMap {
    let input = input_generic.as_compound_ref();

    let mut ret = NbtMap::new();
    ret.set_string("name", "");
    ret.set_f64("amount", 0.0);
    ret.set_string("operation", "add_value");

    copy(input, "AttributeName", &mut ret, "type");
    copy(input, "Slot", &mut ret, "slot");
    copy(input, "UUID", &mut ret, "uuid");
    copy(input, "Name", &mut ret, "name");
    copy(input, "Amount", &mut ret, "amount");

    if let Some(inp) = input {
        if inp.has_key("Operation") {
            let operation = match inp.get_i32("Operation").unwrap_or(0) {
                1 => "add_multiplied_base",
                2 => "add_multiplied_total",
                _ => "add_value",
            };
            ret.set_string("operation", operation);
        }
    }

    ret
}

fn convert_attributes(item: &mut TransientItemStack, flags: i32) {
    let attributes = item.tag_remove_list_unchecked("AttributeModifiers");
    let mut new_attributes = Vec::new();
    if let Some(list) = attributes {
        for a in &list {
            new_attributes.push(NbtValue::Compound(convert_attribute(a)));
        }
    }

    if !new_attributes.is_empty() {
        let mut new_modifiers = NbtMap::new();
        new_modifiers.set_list("modifiers", new_attributes);
        if (flags & TOOLTIP_FLAG_HIDE_MODIFIERS) != 0 {
            new_modifiers.set_bool("show_in_tooltip", false);
        }
        item.components
            .set_map("minecraft:attribute_modifiers", new_modifiers);
    }
}

fn convert_map(item: &mut TransientItemStack) {
    item.tag_migrate_to_component("map", "minecraft:map_id");

    let Some(decorations) = item.tag_remove_list_unchecked("Decorations") else {
        return;
    };

    let mut new_decorations = NbtMap::new();
    for d in &decorations {
        let dm = d.as_compound_ref();
        let id = dm
            .and_then(|m| m.get_string("id"))
            .unwrap_or("")
            .to_string();
        if new_decorations.has_key(&id) {
            continue;
        }
        let type_id = dm.and_then(|m| m.get_i32("type")).unwrap_or(0);
        let x = dm.and_then(|m| m.get_f64("x")).unwrap_or(0.0);
        let z = dm.and_then(|m| m.get_f64("z")).unwrap_or(0.0);
        let rot = dm.and_then(|m| m.get_f64("rot")).unwrap_or(0.0) as f32;

        let mut nd = NbtMap::new();
        nd.set_string("type", convert_map_decoration_id(type_id));
        nd.set_f64("x", x);
        nd.set_f64("z", z);
        nd.set_f32("rotation", rot);
        new_decorations.set_map(&id, nd);
    }

    if !new_decorations.is_empty() {
        item.components
            .set_map("minecraft:map_decorations", new_decorations);
    }
}

fn convert_potion(item: &mut TransientItemStack) {
    let mut potion_contents = NbtMap::new();

    let potion = item.tag_remove_string("Potion");
    if let Some(p) = &potion {
        if p != "minecraft:empty" {
            potion_contents.set_string("potion", p);
        }
    }

    item.migrate_tag_to("CustomPotionColor", &mut potion_contents, "custom_color");
    item.migrate_tag_to(
        "custom_potion_effects",
        &mut potion_contents,
        "custom_effects",
    );

    if !potion_contents.is_empty() {
        item.components
            .set_map("minecraft:potion_contents", potion_contents);
    }
}

fn make_filtered_text(raw: &str, filtered: Option<&str>) -> NbtMap {
    let mut ret = NbtMap::new();
    ret.set_string("raw", raw);
    if let Some(f) = filtered {
        ret.set_string("filtered", f);
    }
    ret
}

fn convert_book_pages(item: &mut TransientItemStack) -> Option<Vec<NbtValue>> {
    let old_pages = item.tag_remove_list_unchecked("pages");
    let filtered_pages = item.tag_remove_map("filtered_pages");

    let old_pages = old_pages?;
    if old_pages.is_empty() {
        return None;
    }

    let mut ret = Vec::with_capacity(old_pages.len());
    for (i, page) in old_pages.iter().enumerate() {
        let page_str = page.as_str().unwrap_or("");
        let key = i.to_string();
        let filtered = filtered_pages.as_ref().and_then(|fp| fp.get_string(&key));
        ret.push(NbtValue::Compound(make_filtered_text(page_str, filtered)));
    }

    Some(ret)
}

fn convert_writable_book(item: &mut TransientItemStack) {
    if let Some(pages) = convert_book_pages(item) {
        let mut book = NbtMap::new();
        book.set_list("pages", pages);
        item.components
            .set_map("minecraft:writable_book_content", book);
    }
}

fn convert_written_book(item: &mut TransientItemStack) {
    let pages = convert_book_pages(item);

    let mut book = NbtMap::new();
    if let Some(p) = pages {
        book.set_list("pages", p);
    }

    let title = item.tag_remove_string("title").unwrap_or_default();
    let filtered_title = item.tag_remove_string("filtered_title");
    book.set_map(
        "title",
        make_filtered_text(&title, filtered_title.as_deref()),
    );

    book.set_string(
        "author",
        item.tag_remove_string("author").unwrap_or_default(),
    );
    item.migrate_tag_to("resolved", &mut book, "resolved");
    item.migrate_tag_to("generation", &mut book, "generation");

    item.components
        .set_map("minecraft:written_book_content", book);
}

fn convert_mob_bucket(item: &mut TransientItemStack) {
    let mut bucket = NbtMap::new();
    for key in BUCKETED_MOB_TAGS {
        item.migrate_tag_to(key, &mut bucket, key);
    }
    if !bucket.is_empty() {
        item.components
            .set_map("minecraft:bucket_entity_data", bucket);
    }
}

fn convert_compass(item: &mut TransientItemStack) {
    let pos = item.tag_remove_generic("LodestonePos");
    let dim = item.tag_remove_generic("LodestoneDimension");

    if pos.is_none() && dim.is_none() {
        return;
    }

    let mut tracker = NbtMap::new();
    if let (Some(pos), Some(dim)) = (pos, dim) {
        let mut target = NbtMap::new();
        target.set_generic("pos", pos);
        target.set_generic("dimension", dim);
        tracker.set_map("target", target);
    }

    let tracked = item.tag_remove_bool("LodestoneTracked", true);
    if !tracked {
        tracker.set_bool("tracked", false);
    }

    item.components
        .set_map("minecraft:lodestone_tracker", tracker);
}

fn convert_firework_explosion(input: &mut NbtValue) {
    let Some(input) = input.as_compound_mut() else {
        return;
    };

    input.rename_key("Colors", "colors");
    input.rename_key("FadeColors", "fade_colors");
    input.rename_key("Trail", "has_trail");
    input.rename_key("Flicker", "has_twinkle");

    let type_id = input.get_i32("Type").unwrap_or(0);
    input.take("Type");

    let shape = match type_id {
        1 => "large_ball",
        2 => "star",
        3 => "creeper",
        4 => "burst",
        _ => "small_ball",
    };
    input.set_string("shape", shape);
}

fn convert_firework_rocket(item: &mut TransientItemStack) {
    let Some(fireworks_generic) = item.tag.take("Fireworks") else {
        return;
    };

    match fireworks_generic {
        NbtValue::Compound(mut fireworks) => {
            let mut new_fireworks = NbtMap::new();
            let flight = fireworks.get_i32("Flight").unwrap_or(0);
            new_fireworks.set_byte("flight_duration", flight as i8);

            let mut explosions = match fireworks.take("Explosions") {
                Some(NbtValue::List(l)) => l,
                _ => Vec::new(),
            };
            for ex in explosions.iter_mut() {
                convert_firework_explosion(ex);
            }
            new_fireworks.set_list("explosions", explosions);
            item.components
                .set_map("minecraft:fireworks", new_fireworks);

            fireworks.take("Flight");
            if !fireworks.is_empty() {
                item.tag.set_map("Fireworks", fireworks);
            }
        }
        other => {
            let mut new_fireworks = NbtMap::new();
            new_fireworks.set_list("explosions", Vec::new());
            new_fireworks.set_byte("flight_duration", 0);
            item.components
                .set_map("minecraft:fireworks", new_fireworks);
            // Java only read Fireworks (never removed); keep it.
            item.tag.set_generic("Fireworks", other);
        }
    }
}

fn convert_firework_star(item: &mut TransientItemStack) {
    let Some(explosion_generic) = item.tag.take("Explosion") else {
        return;
    };

    match explosion_generic {
        NbtValue::Compound(mut explosion) => {
            let mut copy_val = NbtValue::Compound(explosion.clone());
            convert_firework_explosion(&mut copy_val);
            item.components
                .set_generic("minecraft:firework_explosion", copy_val);

            explosion.take("Type");
            explosion.take("Colors");
            explosion.take("FadeColors");
            explosion.take("Trail");
            explosion.take("Flicker");
            if !explosion.is_empty() {
                item.tag.set_map("Explosion", explosion);
            }
        }
        other => {
            item.components
                .set_generic("minecraft:firework_explosion", other.clone());
            item.tag.set_generic("Explosion", other);
        }
    }
}

fn convert_properties(properties: &NbtMap) -> Vec<NbtValue> {
    let mut ret = Vec::new();
    for key in properties.keys() {
        let Some(values) = properties.get_list(&key) else {
            continue;
        };
        for v in values {
            let pm = v.as_compound_ref();
            let value = pm
                .and_then(|m| m.get_string("Value"))
                .unwrap_or("")
                .to_string();
            let signature = pm
                .and_then(|m| m.get_string("Signature"))
                .map(str::to_string);

            let mut np = NbtMap::new();
            np.set_string("name", &key);
            np.set_string("value", value);
            if let Some(sig) = signature {
                np.set_string("signature", sig);
            }
            ret.push(NbtValue::Compound(np));
        }
    }
    ret
}

fn convert_profile(input_generic: &NbtValue) -> NbtMap {
    let mut ret = NbtMap::new();

    if let NbtValue::String(name) = input_generic {
        if is_valid_player_name(name) {
            ret.set_string("name", name);
        }
        return ret;
    }

    let input = input_generic.as_compound_ref();
    let name = input.and_then(|m| m.get_string("Name")).unwrap_or("");
    if is_valid_player_name(name) {
        ret.set_string("name", name);
    }

    if let Some(id) = input.and_then(|m| m.get("Id")) {
        ret.set_generic("id", id.clone());
    }

    if let Some(props) = input.and_then(|m| m.get_map("Properties")) {
        if !props.is_empty() {
            ret.set_list("properties", convert_properties(props));
        }
    }

    ret
}

fn convert_skull(item: &mut TransientItemStack) {
    let Some(skull_owner) = item.tag_remove_generic("SkullOwner") else {
        return;
    };
    let profile = convert_profile(&skull_owner);
    item.components.set_map("minecraft:profile", profile);
}

/// `ConverterItemStackToDataComponents.convertItem` — the entry point. Returns a
/// new item compound in the 1.20.5 `{id, count, components}` shape.
pub fn convert_item(input: &NbtMap) -> NbtMap {
    if input.get_string("id").is_none() || input.get_i64("Count").is_none() {
        return input.clone();
    }

    let mut item = TransientItemStack::new(input);

    item.tag_migrate_int("Damage", "minecraft:damage", 0);
    item.tag_migrate_int("RepairCost", "minecraft:repair_cost", 0);
    item.tag_migrate_to_component("CustomModelData", "minecraft:custom_model_data");

    if let Some(mut block_state_props) = item.tag_remove_map("BlockStateTag") {
        convert_block_state_properties(&mut block_state_props);
        item.components
            .set_map("minecraft:block_state", block_state_props);
    }

    item.tag_migrate_to_component("EntityTag", "minecraft:entity_data");

    if let Some(mut tile) = item.tag_remove_map("BlockEntityTag") {
        convert_tile_entity(&mut tile, &mut item);
        if tile.len() > 1 || (tile.len() == 1 && !tile.has_key("id")) {
            item.components.set_map("minecraft:block_entity_data", tile);
        }
    }

    let flags = item.tag_remove_int("HideFlags", 0);

    if item.tag_remove_int("Unbreakable", 0) != 0 {
        let mut unbreakable = NbtMap::new();
        if (flags & TOOLTIP_FLAG_HIDE_UNBREAKABLE) != 0 {
            unbreakable.set_bool("show_in_tooltip", false);
        }
        item.components
            .set_map("minecraft:unbreakable", unbreakable);
    }

    convert_enchantments(
        &mut item,
        "Enchantments",
        "minecraft:enchantments",
        (flags & TOOLTIP_FLAG_HIDE_ENCHANTMENTS) != 0,
    );

    convert_display(&mut item, flags);
    convert_adventure_mode(&mut item, flags);
    convert_attributes(&mut item, flags);

    if let Some(trim) = item.tag_remove_generic("Trim") {
        let trim = if (flags & TOOLTIP_FLAG_HIDE_UPGRADES) != 0 {
            match trim {
                NbtValue::Compound(mut m) => {
                    m.set_bool("show_in_tooltip", false);
                    NbtValue::Compound(m)
                }
                other => other,
            }
        } else {
            trim
        };
        item.components.set_generic("minecraft:trim", trim);
    }

    if (flags & TOOLTIP_FLAG_HIDE_ADDITIONAL) != 0 {
        item.components
            .set_map("minecraft:hide_additional_tooltip", NbtMap::new());
    }

    match item.id.as_str() {
        "minecraft:enchanted_book" => convert_enchantments(
            &mut item,
            "StoredEnchantments",
            "minecraft:stored_enchantments",
            (flags & TOOLTIP_FLAG_HIDE_ADDITIONAL) != 0,
        ),
        "minecraft:crossbow" => {
            item.tag_remove_generic("Charged");
            item.tag_migrate_non_empty_list("ChargedProjectiles", "minecraft:charged_projectiles");
        }
        "minecraft:bundle" => item.tag_migrate_non_empty_list("Items", "minecraft:bundle_contents"),
        "minecraft:filled_map" => convert_map(&mut item),
        "minecraft:potion"
        | "minecraft:splash_potion"
        | "minecraft:lingering_potion"
        | "minecraft:tipped_arrow" => convert_potion(&mut item),
        "minecraft:writable_book" => convert_writable_book(&mut item),
        "minecraft:written_book" => convert_written_book(&mut item),
        "minecraft:suspicious_stew" => {
            item.tag_migrate_to_component("effects", "minecraft:suspicious_stew_effects")
        }
        "minecraft:debug_stick" => {
            item.tag_migrate_to_component("DebugProperty", "minecraft:debug_stick_state")
        }
        "minecraft:pufferfish_bucket"
        | "minecraft:salmon_bucket"
        | "minecraft:cod_bucket"
        | "minecraft:tropical_fish_bucket"
        | "minecraft:axolotl_bucket"
        | "minecraft:tadpole_bucket" => convert_mob_bucket(&mut item),
        "minecraft:goat_horn" => {
            item.tag_migrate_to_component("instrument", "minecraft:instrument")
        }
        "minecraft:knowledge_book" => item.tag_migrate_to_component("Recipes", "minecraft:recipes"),
        "minecraft:compass" => convert_compass(&mut item),
        "minecraft:firework_rocket" => convert_firework_rocket(&mut item),
        "minecraft:firework_star" => convert_firework_star(&mut item),
        "minecraft:player_head" => convert_skull(&mut item),
        _ => {}
    }

    item.serialize()
}

// ===========================================================================
// REVERSE: `unconvert_item` — modern `{id, count, components}` -> legacy
// `{id, Count, Damage?, tag}`. The inverse of `convert_item`.
// ===========================================================================

const V3818: i32 = 3818;

/// Reverse map-decoration name -> numeric `type` id (inverse of
/// `convert_map_decoration_id`). Unknown names default to 0 ("player").
fn unconvert_map_decoration_id(name: &str) -> i32 {
    MAP_DECORATION_CONVERSION_TABLE
        .iter()
        .position(|&n| n == name)
        .map(|p| p as i32)
        .unwrap_or(0)
}

/// Reverse banner colour name -> index (inverse of `banner_colour`). Unknown
/// names default to 0 ("white").
fn unconvert_banner_colour(name: &str) -> i32 {
    BANNER_COLOURS
        .iter()
        .position(|&n| n == name)
        .map(|p| p as i32)
        .unwrap_or(0)
}

/// Reverse firework shape name -> numeric `Type` (inverse of the shape switch
/// in `convert_firework_explosion`).
fn unconvert_firework_shape(shape: &str) -> i32 {
    match shape {
        "large_ball" => 1,
        "star" => 2,
        "creeper" => 3,
        "burst" => 4,
        // "small_ball" and anything unknown
        _ => 0,
    }
}

/// Reverse attribute operation name -> numeric (inverse of the operation switch
/// in `convert_attribute`).
fn unconvert_attribute_operation(op: &str) -> i32 {
    match op {
        "add_multiplied_base" => 1,
        "add_multiplied_total" => 2,
        // "add_value" and anything unknown
        _ => 0,
    }
}

/// `{"translate": key}` JSON -> the bare `key` (inverse of
/// `create_translatable_component`). Returns `None` if `input` is not a single
/// `translate` object.
fn extract_translate_key(input: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(input).ok()?;
    let obj = v.as_object()?;
    match obj.get("translate") {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

/// The working set for the reverse: pull `components` and rebuild `tag`.
struct ReverseItemStack {
    id: String,
    count: i32,
    /// the modern components, drained as they are inverted
    components: NbtMap,
    /// the legacy tag under construction (seeded from `minecraft:custom_data`)
    tag: NbtMap,
    /// reconstructed HideFlags bitfield (0 if no bits set)
    hide_flags: i32,
    /// surviving root fields (everything except id/count/components)
    root: NbtMap,
}

impl ReverseItemStack {
    fn new(root: &NbtMap) -> Self {
        let id = root.get_string("id").unwrap_or("").to_string();
        let count = root.get_i32("count").unwrap_or(0);

        let mut root_copy = root.clone();
        let components = match root_copy.take("components") {
            Some(NbtValue::Compound(m)) => m,
            _ => NbtMap::new(),
        };
        root_copy.take("id");
        root_copy.take("count");

        // The custom_data component is the escape hatch: its contents are the
        // base tag, restoring all leftover/unknown keys losslessly.
        let mut components = components;
        let tag = match components.take("minecraft:custom_data") {
            Some(NbtValue::Compound(m)) => m,
            _ => NbtMap::new(),
        };

        Self {
            id,
            count,
            components,
            tag,
            hide_flags: 0,
            root: root_copy,
        }
    }

    /// Take a component, regardless of type.
    fn comp_take(&mut self, key: &str) -> Option<NbtValue> {
        self.components.take(key)
    }

    fn comp_take_map(&mut self, key: &str) -> Option<NbtMap> {
        match self.components.take(key) {
            Some(NbtValue::Compound(m)) => Some(m),
            other => {
                // put it back if it was a non-map, so it can be reported as
                // dropped later
                if let Some(v) = other {
                    self.components.set_generic(key, v);
                }
                None
            }
        }
    }

    fn comp_take_list(&mut self, key: &str) -> Option<Vec<NbtValue>> {
        match self.components.take(key) {
            Some(NbtValue::List(l)) => Some(l),
            other => {
                if let Some(v) = other {
                    self.components.set_generic(key, v);
                }
                None
            }
        }
    }

    fn set_hide_flag(&mut self, bit: i32) {
        self.hide_flags |= bit;
    }

    /// Move a component value directly into a legacy tag key.
    fn comp_migrate_to_tag(&mut self, component_key: &str, tag_key: &str) {
        if let Some(v) = self.components.take(component_key) {
            self.tag.set_generic(tag_key, v);
        }
    }

    fn serialize(mut self) -> NbtMap {
        // Anything left in components has no legacy representation: report+drop.
        for key in self.components.keys() {
            report_loss(
                V3818,
                LossKind::ComponentDropped,
                Severity::Loss,
                format!("{key}: no legacy tag representation"),
            );
        }

        let mut ret = NbtMap::new();
        ret.set_string("id", self.id);
        ret.set_byte("Count", self.count as i8);

        // Re-emit HideFlags if any bit was reconstructed.
        if self.hide_flags != 0 {
            self.tag.set_i32("HideFlags", self.hide_flags);
        }

        if !self.tag.is_empty() {
            ret.set_map("tag", self.tag);
        }

        // Merge surviving root fields, ret entries take priority.
        for key in self.root.keys() {
            if ret.has_key(&key) {
                continue;
            }
            if let Some(v) = self.root.get(&key) {
                ret.set_generic(&key, v.clone());
            }
        }

        ret
    }
}

fn report_unconverted_component_fields(component_key: &str, component: &NbtMap, handled: &[&str]) {
    for key in component.keys() {
        if handled.iter().any(|h| *h == key) {
            continue;
        }
        report_loss(
            V3818,
            LossKind::ComponentDropped,
            Severity::Loss,
            format!("{component_key}.{key}: no legacy tag representation"),
        );
    }
}

/// Inverse of `convert_enchantments`: a `{levels:{id->lvl}, show_in_tooltip?}`
/// component back to a legacy list-of-`{id,lvl}` under `tag_key`, plus the hide
/// flag and the empty-glint override (handled by the caller).
fn unconvert_enchantments(
    item: &mut ReverseItemStack,
    component_key: &str,
    tag_key: &str,
    hide_flag: i32,
) {
    let Some(comp) = item.comp_take_map(component_key) else {
        return;
    };

    if comp.get_bool("show_in_tooltip") == Some(false) {
        item.set_hide_flag(hide_flag);
    }

    if let Some(levels) = comp.get_map("levels") {
        let mut list = Vec::new();
        for key in levels.keys() {
            let lvl = levels.get_i32(&key).unwrap_or(0);
            let mut e = NbtMap::new();
            e.set_string("id", &key);
            // Forward stored Enchantments lvl as int; legacy used short. Use
            // short to match vanilla pre-1.20.5 storage.
            e.set_short("lvl", lvl.clamp(i16::MIN as i32, i16::MAX as i32) as i16);
            list.push(NbtValue::Compound(e));
        }
        if !list.is_empty() {
            item.tag.set_list(tag_key, list);
        }
    }
    report_unconverted_component_fields(component_key, &comp, &["levels", "show_in_tooltip"]);
}

/// Inverse of `convert_display`: rebuild the legacy `display` compound from the
/// custom_name / item_name / lore / dyed_color components and the map color.
fn unconvert_display(item: &mut ReverseItemStack) {
    // Start from whatever `display` may already be in the tag (rare — forward
    // re-stores a display only if it still had leftover keys).
    let mut display = match item.tag.take("display") {
        Some(NbtValue::Compound(m)) => m,
        Some(other) => {
            item.tag.set_generic("display", other);
            NbtMap::new()
        }
        None => NbtMap::new(),
    };

    // custom_name -> display.Name (forward stored the raw JSON string).
    if let Some(NbtValue::String(name)) = item.comp_take("minecraft:custom_name") {
        display.set_string("Name", name);
    }

    // lore -> display.Lore (list of JSON strings).
    if let Some(lore) = item.comp_take_list("minecraft:lore") {
        display.set_list("Lore", lore);
    }

    // item_name -> display.LocName (forward applied createTranslatableComponent;
    // invert by extracting the translate key). If it is not a translate object,
    // it cannot be represented as a legacy LocName -> drop+report.
    if let Some(NbtValue::String(item_name)) = item.comp_take("minecraft:item_name") {
        if let Some(key) = extract_translate_key(&item_name) {
            display.set_string("LocName", key);
        } else {
            report_loss(
                V3818,
                LossKind::ComponentDropped,
                Severity::Loss,
                "minecraft:item_name: not a translatable component, no legacy LocName form",
            );
        }
    }

    // dyed_color -> display.color (+ hide-dye flag).
    if let Some(dyed) = item.comp_take_map("minecraft:dyed_color") {
        if dyed.get_bool("show_in_tooltip") == Some(false) {
            item.set_hide_flag(TOOLTIP_FLAG_HIDE_DYE);
        }
        if let Some(rgb) = dyed.get_i32("rgb") {
            display.set_i32("color", rgb);
        }
        report_unconverted_component_fields(
            "minecraft:dyed_color",
            &dyed,
            &["rgb", "show_in_tooltip"],
        );
    }

    // map_color -> display.MapColor (filled_map only on the forward path).
    if let Some(map_color) = item.comp_take("minecraft:map_color") {
        display.set_generic("MapColor", map_color);
    }

    if !display.is_empty() {
        item.tag.set_map("display", display);
    }
}

/// Inverse of `convert_block_state_predicate`: a `{blocks, state?, nbt?}` map
/// back to the `name[props]{nbt}` predicate string.
fn unconvert_block_state_predicate(predicate: &NbtMap) -> String {
    let mut out = String::new();
    if let Some(blocks) = predicate.get_string("blocks") {
        out.push_str(blocks);
    } else if let Some(list) = predicate.get_list("blocks") {
        // forward stored a string under "blocks"; a list form has no single
        // predicate-string representation — take the first if present.
        if let Some(NbtValue::String(s)) = list.first() {
            out.push_str(s);
        }
    }

    if let Some(state) = predicate.get_map("state") {
        let mut parts = Vec::new();
        for key in state.keys() {
            if let Some(v) = state.get_string(&key) {
                parts.push(format!("{key}={v}"));
            }
        }
        if !parts.is_empty() {
            out.push('[');
            out.push_str(&parts.join(","));
            out.push(']');
        }
    }

    if let Some(nbt) = predicate.get_string("nbt") {
        out.push_str(nbt);
    }

    out
}

/// Inverse of `convert_block_state_predicates`: a `{predicates:[...],
/// show_in_tooltip?}` component back to the legacy list-of-strings under
/// `tag_key`, plus the hide flag.
fn unconvert_block_state_predicates(
    item: &mut ReverseItemStack,
    component_key: &str,
    tag_key: &str,
    hide_flag: i32,
) {
    let Some(comp) = item.comp_take_map(component_key) else {
        return;
    };

    if comp.get_bool("show_in_tooltip") == Some(false) {
        item.set_hide_flag(hide_flag);
    }

    if let Some(predicates) = comp.get_list("predicates") {
        let mut list = Vec::with_capacity(predicates.len());
        for p in predicates {
            if let NbtValue::Compound(pm) = p {
                list.push(NbtValue::String(unconvert_block_state_predicate(pm)));
            } else {
                list.push(p.clone());
            }
        }
        item.tag.set_list(tag_key, list);
    }
    report_unconverted_component_fields(component_key, &comp, &["predicates", "show_in_tooltip"]);
}

/// Inverse of `convert_attribute`: a modern modifier back to the legacy form.
fn unconvert_attribute(input: &NbtValue) -> NbtMap {
    let inp = input.as_compound_ref();
    let mut ret = NbtMap::new();

    if let Some(m) = inp {
        if let Some(v) = m.get("type") {
            ret.set_generic("AttributeName", v.clone());
        }
        if let Some(v) = m.get("slot") {
            ret.set_generic("Slot", v.clone());
        }
        if let Some(v) = m.get("uuid") {
            ret.set_generic("UUID", v.clone());
        }
        if let Some(v) = m.get("name") {
            ret.set_generic("Name", v.clone());
        }
        if let Some(v) = m.get("amount") {
            ret.set_generic("Amount", v.clone());
        }
        if let Some(op) = m.get_string("operation") {
            ret.set_i32("Operation", unconvert_attribute_operation(op));
        }
    }

    ret
}

/// Inverse of `convert_attributes`.
fn unconvert_attributes(item: &mut ReverseItemStack) {
    let Some(comp) = item.comp_take_map("minecraft:attribute_modifiers") else {
        return;
    };

    if comp.get_bool("show_in_tooltip") == Some(false) {
        item.set_hide_flag(TOOLTIP_FLAG_HIDE_MODIFIERS);
    }

    if let Some(modifiers) = comp.get_list("modifiers") {
        let mut list = Vec::with_capacity(modifiers.len());
        for m in modifiers {
            list.push(NbtValue::Compound(unconvert_attribute(m)));
        }
        item.tag.set_list("AttributeModifiers", list);
    }
    report_unconverted_component_fields(
        "minecraft:attribute_modifiers",
        &comp,
        &["modifiers", "show_in_tooltip"],
    );
}

/// Inverse of `convert_tile_entity` sub-components -> rebuild a `BlockEntityTag`.
/// `tile` is the base tag (from `minecraft:block_entity_data` if present, else
/// a fresh compound). Returns whether anything tile-entity-related was found.
fn unconvert_tile_entity(item: &mut ReverseItemStack, tile: &mut NbtMap) -> bool {
    let mut found = false;

    if let Some(lock) = item.comp_take("minecraft:lock") {
        tile.set_generic("Lock", lock);
        found = true;
    }

    if let Some(container_loot) = item.comp_take_map("minecraft:container_loot") {
        if let Some(loot_table) = container_loot.get("loot_table") {
            tile.set_generic("LootTable", loot_table.clone());
        }
        if let Some(seed) = container_loot.get_i64("seed") {
            if seed != 0 {
                tile.set_i64("LootTableSeed", seed);
            }
        }
        found = true;
        report_unconverted_component_fields(
            "minecraft:container_loot",
            &container_loot,
            &["loot_table", "seed"],
        );
    }

    // note_block_sound (skull), pot_decorations/item (decorated_pot),
    // banner_patterns/base_color (banner), bees (beehive), container (chests &
    // decorated_pot). These are unambiguous keys, so we invert each regardless
    // of the recorded id.
    if let Some(v) = item.comp_take("minecraft:note_block_sound") {
        tile.set_generic("note_block_sound", v);
        found = true;
    }

    if let Some(v) = item.comp_take("minecraft:pot_decorations") {
        tile.set_generic("sherds", v);
        found = true;
    }

    if let Some(v) = item.comp_take("minecraft:banner_patterns") {
        tile.set_generic("patterns", v);
        found = true;
    }

    if let Some(NbtValue::String(base_color)) = item.comp_take("minecraft:base_color") {
        tile.set_i32("Base", unconvert_banner_colour(&base_color));
        found = true;
    }

    if let Some(v) = item.comp_take("minecraft:bees") {
        tile.set_generic("bees", v);
        found = true;
    }

    // container: list of {slot, item}. For a decorated_pot the forward used a
    // single {slot:0, item} -> tile.item; for chests/shulkers etc. -> tile.Items
    // with Slot restored. We disambiguate by id when present, else default to
    // the Items form (the common container case).
    if let Some(container) = item.comp_take_list("minecraft:container") {
        let id = correct_namespace(tile.get_string("id").unwrap_or(""));
        if id == "minecraft:decorated_pot" {
            // single item back to tile.item
            if let Some(first) = container.into_iter().next() {
                if let NbtValue::Compound(mut wrapped) = first {
                    if let Some(it) = wrapped.take("item") {
                        tile.set_generic("item", it);
                    }
                }
            }
        } else {
            let mut items = Vec::with_capacity(container.len());
            for entry in container {
                if let NbtValue::Compound(mut wrapped) = entry {
                    let slot = wrapped.get_i32("slot").unwrap_or(0) & 0xFF;
                    match wrapped.take("item") {
                        Some(NbtValue::Compound(mut it)) => {
                            it.set_byte("Slot", slot as i8);
                            items.push(NbtValue::Compound(it));
                        }
                        Some(other) => items.push(other),
                        None => {}
                    }
                } else {
                    items.push(entry);
                }
            }
            if !items.is_empty() {
                tile.set_list("Items", items);
            }
        }
        found = true;
    }

    found
}

/// Inverse of `convert_map`: map_id + map_decorations -> tag.map + Decorations.
fn unconvert_map(item: &mut ReverseItemStack) {
    item.comp_migrate_to_tag("minecraft:map_id", "map");

    if let Some(decorations) = item.comp_take_map("minecraft:map_decorations") {
        let mut list = Vec::new();
        for id in decorations.keys() {
            let Some(d) = decorations.get_map(&id) else {
                continue;
            };
            let mut nd = NbtMap::new();
            nd.set_string("id", &id);
            let type_name = d.get_string("type").unwrap_or("player");
            nd.set_i32("type", unconvert_map_decoration_id(type_name));
            nd.set_f64("x", d.get_f64("x").unwrap_or(0.0));
            nd.set_f64("z", d.get_f64("z").unwrap_or(0.0));
            // forward stored rotation as float; legacy used double "rot".
            nd.set_f64("rot", d.get_f64("rotation").unwrap_or(0.0));
            list.push(NbtValue::Compound(nd));
        }
        if !list.is_empty() {
            item.tag.set_list("Decorations", list);
        }
        for id in decorations.keys() {
            if let Some(d) = decorations.get_map(&id) {
                report_unconverted_component_fields(
                    &format!("minecraft:map_decorations.{id}"),
                    d,
                    &["type", "x", "z", "rotation"],
                );
            }
        }
    }
}

/// Inverse of `convert_potion`.
fn unconvert_potion(item: &mut ReverseItemStack) {
    let Some(contents) = item.comp_take_map("minecraft:potion_contents") else {
        return;
    };

    if let Some(potion) = contents.get_string("potion") {
        item.tag.set_string("Potion", potion);
    }
    if let Some(color) = contents.get("custom_color") {
        item.tag.set_generic("CustomPotionColor", color.clone());
    }
    if let Some(effects) = contents.get("custom_effects") {
        item.tag
            .set_generic("custom_potion_effects", effects.clone());
    }
    report_unconverted_component_fields(
        "minecraft:potion_contents",
        &contents,
        &["potion", "custom_color", "custom_effects"],
    );
}

/// Inverse of `make_filtered_text` -> (raw, filtered).
fn unmake_filtered_text(m: &NbtMap) -> (String, Option<String>) {
    let raw = m.get_string("raw").unwrap_or("").to_string();
    let filtered = m.get_string("filtered").map(str::to_string);
    (raw, filtered)
}

/// Inverse of `convert_book_pages`: a list of `{raw, filtered?}` back to legacy
/// `pages` (list of strings) and `filtered_pages` (index->string map).
fn unconvert_book_pages(pages: &[NbtValue]) -> (Vec<NbtValue>, NbtMap) {
    let mut old_pages = Vec::with_capacity(pages.len());
    let mut filtered_pages = NbtMap::new();
    for (i, p) in pages.iter().enumerate() {
        if let NbtValue::Compound(m) = p {
            let (raw, filtered) = unmake_filtered_text(m);
            old_pages.push(NbtValue::String(raw));
            if let Some(f) = filtered {
                filtered_pages.set_string(&i.to_string(), f);
            }
        } else {
            old_pages.push(p.clone());
        }
    }
    (old_pages, filtered_pages)
}

/// Inverse of `convert_writable_book`.
fn unconvert_writable_book(item: &mut ReverseItemStack) {
    let Some(book) = item.comp_take_map("minecraft:writable_book_content") else {
        return;
    };
    if let Some(pages) = book.get_list("pages") {
        let (old_pages, filtered) = unconvert_book_pages(pages);
        if !old_pages.is_empty() {
            item.tag.set_list("pages", old_pages);
        }
        if !filtered.is_empty() {
            item.tag.set_map("filtered_pages", filtered);
        }
    }
    report_unconverted_component_fields("minecraft:writable_book_content", &book, &["pages"]);
}

/// Inverse of `convert_written_book`.
fn unconvert_written_book(item: &mut ReverseItemStack) {
    let Some(book) = item.comp_take_map("minecraft:written_book_content") else {
        return;
    };

    if let Some(pages) = book.get_list("pages") {
        let (old_pages, filtered) = unconvert_book_pages(pages);
        if !old_pages.is_empty() {
            item.tag.set_list("pages", old_pages);
        }
        if !filtered.is_empty() {
            item.tag.set_map("filtered_pages", filtered);
        }
    }

    if let Some(title) = book.get_map("title") {
        let (raw, filtered) = unmake_filtered_text(title);
        item.tag.set_string("title", raw);
        if let Some(f) = filtered {
            item.tag.set_string("filtered_title", f);
        }
    }

    if let Some(author) = book.get_string("author") {
        item.tag.set_string("author", author);
    }
    if let Some(resolved) = book.get("resolved") {
        item.tag.set_generic("resolved", resolved.clone());
    }
    if let Some(generation) = book.get("generation") {
        item.tag.set_generic("generation", generation.clone());
    }
    report_unconverted_component_fields(
        "minecraft:written_book_content",
        &book,
        &["pages", "title", "author", "resolved", "generation"],
    );
}

/// Inverse of `convert_mob_bucket`.
fn unconvert_mob_bucket(item: &mut ReverseItemStack) {
    let Some(bucket) = item.comp_take_map("minecraft:bucket_entity_data") else {
        return;
    };
    for key in BUCKETED_MOB_TAGS {
        if let Some(v) = bucket.get(key) {
            item.tag.set_generic(key, v.clone());
        }
    }
    report_unconverted_component_fields(
        "minecraft:bucket_entity_data",
        &bucket,
        &BUCKETED_MOB_TAGS,
    );
}

/// Inverse of `convert_compass`.
fn unconvert_compass(item: &mut ReverseItemStack) {
    let Some(tracker) = item.comp_take_map("minecraft:lodestone_tracker") else {
        return;
    };

    if let Some(target) = tracker.get_map("target") {
        if let Some(pos) = target.get("pos") {
            item.tag.set_generic("LodestonePos", pos.clone());
        }
        if let Some(dim) = target.get("dimension") {
            item.tag.set_generic("LodestoneDimension", dim.clone());
        }
    }

    // forward only wrote tracked=false; the legacy default is true. So write
    // LodestoneTracked only if explicitly false.
    if tracker.get_bool("tracked") == Some(false) {
        item.tag.set_bool("LodestoneTracked", false);
    }
    report_unconverted_component_fields(
        "minecraft:lodestone_tracker",
        &tracker,
        &["target", "tracked"],
    );
}

/// Inverse of `convert_firework_explosion`: rename keys + shape->Type, in place.
fn unconvert_firework_explosion(input: &mut NbtValue) {
    let Some(input) = input.as_compound_mut() else {
        return;
    };

    input.rename_key("colors", "Colors");
    input.rename_key("fade_colors", "FadeColors");
    input.rename_key("has_trail", "Trail");
    input.rename_key("has_twinkle", "Flicker");

    if let Some(shape) = input.get_string("shape").map(str::to_string) {
        input.take("shape");
        input.set_i32("Type", unconvert_firework_shape(&shape));
    }
}

/// Inverse of `convert_firework_rocket`.
fn unconvert_firework_rocket(item: &mut ReverseItemStack) {
    let Some(fw) = item.comp_take_map("minecraft:fireworks") else {
        return;
    };

    // Merge into an existing Fireworks tag if the forward left one behind.
    let mut fireworks = match item.tag.take("Fireworks") {
        Some(NbtValue::Compound(m)) => m,
        Some(other) => {
            item.tag.set_generic("Fireworks", other);
            NbtMap::new()
        }
        None => NbtMap::new(),
    };

    if let Some(flight) = fw.get_i32("flight_duration") {
        fireworks.set_i32("Flight", flight);
    }

    if let Some(explosions) = fw.get_list("explosions") {
        let mut list = Vec::with_capacity(explosions.len());
        for ex in explosions {
            let mut ex = ex.clone();
            unconvert_firework_explosion(&mut ex);
            list.push(ex);
        }
        fireworks.set_list("Explosions", list);
    }
    report_unconverted_component_fields(
        "minecraft:fireworks",
        &fw,
        &["flight_duration", "explosions"],
    );

    if !fireworks.is_empty() {
        item.tag.set_map("Fireworks", fireworks);
    }
}

/// Inverse of `convert_firework_star`.
fn unconvert_firework_star(item: &mut ReverseItemStack) {
    let Some(explosion) = item.comp_take("minecraft:firework_explosion") else {
        return;
    };

    // Merge into an existing Explosion tag if the forward left one behind.
    let existing = match item.tag.take("Explosion") {
        Some(NbtValue::Compound(m)) => Some(m),
        Some(other) => {
            item.tag.set_generic("Explosion", other);
            None
        }
        None => None,
    };

    match explosion {
        NbtValue::Compound(m) => {
            let mut copy = NbtValue::Compound(m);
            unconvert_firework_explosion(&mut copy);
            if let NbtValue::Compound(mut new_explosion) = copy {
                if let Some(existing) = existing {
                    // keep any leftover keys the forward re-stored
                    for key in existing.keys() {
                        if !new_explosion.has_key(&key) {
                            if let Some(v) = existing.get(&key) {
                                new_explosion.set_generic(&key, v.clone());
                            }
                        }
                    }
                }
                item.tag.set_map("Explosion", new_explosion);
            }
        }
        other => {
            item.tag.set_generic("Explosion", other);
        }
    }
}

/// Inverse of `convert_properties`: list-of-`{name, value, signature?}` back to
/// a `{name -> [{Value, Signature?}]}` map.
fn unconvert_properties(properties: &[NbtValue]) -> NbtMap {
    let mut ret = NbtMap::new();
    for p in properties {
        let Some(pm) = p.as_compound_ref() else {
            continue;
        };
        let name = pm.get_string("name").unwrap_or("").to_string();
        let value = pm.get_string("value").unwrap_or("").to_string();
        let signature = pm.get_string("signature").map(str::to_string);

        let mut entry = NbtMap::new();
        entry.set_string("Value", value);
        if let Some(sig) = signature {
            entry.set_string("Signature", sig);
        }

        match ret.get_list_mut(&name) {
            Some(list) => list.push(NbtValue::Compound(entry)),
            None => ret.set_list(&name, vec![NbtValue::Compound(entry)]),
        }
    }
    ret
}

/// Inverse of `convert_profile` -> a legacy `SkullOwner` value (string or
/// compound).
fn unconvert_profile(profile: &NbtMap) -> NbtValue {
    let has_id = profile.has_key("id");
    let has_props = profile.get_map("properties").is_some()
        || matches!(profile.get("properties"), Some(NbtValue::List(_)));

    // forward emitted a bare {name} when the source was a string; restore the
    // string form when only a name is present.
    if !has_id && !has_props {
        if let Some(name) = profile.get_string("name") {
            return NbtValue::String(name.to_string());
        }
        // empty profile -> empty compound
        return NbtValue::Compound(NbtMap::new());
    }

    let mut ret = NbtMap::new();
    if let Some(name) = profile.get_string("name") {
        ret.set_string("Name", name);
    }
    if let Some(id) = profile.get("id") {
        ret.set_generic("Id", id.clone());
    }
    if let Some(props) = profile.get_list("properties") {
        ret.set_map("Properties", unconvert_properties(props));
    }
    NbtValue::Compound(ret)
}

/// Inverse of `convert_skull`.
fn unconvert_skull(item: &mut ReverseItemStack) {
    let Some(profile) = item.comp_take_map("minecraft:profile") else {
        return;
    };
    item.tag
        .set_generic("SkullOwner", unconvert_profile(&profile));
    report_unconverted_component_fields(
        "minecraft:profile",
        &profile,
        &["name", "id", "properties"],
    );
}

/// Inverse of `convert_item`: modern `{id, count, components}` -> legacy
/// `{id, Count, Damage?, tag}`. The `minecraft:custom_data` component is the
/// escape hatch — its contents become the base `tag`, then each known component
/// is un-squashed back to its legacy `tag` key. Components with no legacy
/// representation are dropped and reported via report_loss.
pub fn unconvert_item(input: &NbtMap) -> NbtMap {
    // Mirror the forward no-op: if neither count nor components are present this
    // is not a squashed item; return it unchanged.
    if input.get_i64("count").is_none() && !input.has_key("components") {
        return input.clone();
    }

    let mut item = ReverseItemStack::new(input);

    // --- scalars that came straight from a tag key ---
    // damage -> tag.Damage (int) + base Damage (short).
    let damage = match item.comp_take("minecraft:damage") {
        Some(v) => v.as_number_i64(),
        None => None,
    };

    item.comp_migrate_to_tag("minecraft:repair_cost", "RepairCost");
    item.comp_migrate_to_tag("minecraft:custom_model_data", "CustomModelData");

    // block_state -> BlockStateTag. The forward stringified numeric/boolean
    // values; we cannot recover their original numeric types, so the round-trip
    // is exact only for already-string values. Report as approximated.
    if let Some(bs) = item.comp_take_map("minecraft:block_state") {
        report_loss(
            V3818,
            LossKind::ComponentDropped,
            Severity::Approximated,
            "minecraft:block_state: restored BlockStateTag with stringified property values",
        );
        item.tag.set_map("BlockStateTag", bs);
    }

    item.comp_migrate_to_tag("minecraft:entity_data", "EntityTag");

    // --- block entity data + its sub-components -> BlockEntityTag ---
    let mut tile = item
        .comp_take_map("minecraft:block_entity_data")
        .unwrap_or_default();
    let tile_found = unconvert_tile_entity(&mut item, &mut tile);
    if !tile.is_empty() && (tile.len() > 1 || tile_found || !tile.has_key("id")) {
        item.tag.set_map("BlockEntityTag", tile);
    }

    // --- unbreakable ---
    if let Some(unbreakable) = item.comp_take_map("minecraft:unbreakable") {
        item.tag.set_i32("Unbreakable", 1);
        if unbreakable.get_bool("show_in_tooltip") == Some(false) {
            item.set_hide_flag(TOOLTIP_FLAG_HIDE_UNBREAKABLE);
        }
    }

    // --- enchantments / glint ---
    unconvert_enchantments(
        &mut item,
        "minecraft:enchantments",
        "Enchantments",
        TOOLTIP_FLAG_HIDE_ENCHANTMENTS,
    );
    // enchantment_glint_override(true) was the forward's signal for an empty
    // Enchantments list; there is no legacy tag for it. Drop+report if present.
    if item
        .comp_take("minecraft:enchantment_glint_override")
        .is_some()
    {
        report_loss(
            V3818,
            LossKind::ComponentDropped,
            Severity::Loss,
            "minecraft:enchantment_glint_override: no legacy tag representation",
        );
    }

    unconvert_display(&mut item);
    unconvert_block_state_predicates(
        &mut item,
        "minecraft:can_break",
        "CanDestroy",
        TOOLTIP_FLAG_HIDE_CAN_DESTROY,
    );
    unconvert_block_state_predicates(
        &mut item,
        "minecraft:can_place_on",
        "CanPlaceOn",
        TOOLTIP_FLAG_HIDE_CAN_PLACE,
    );
    unconvert_attributes(&mut item);

    // trim -> tag.Trim (+ hide-upgrades flag if show_in_tooltip=false).
    if let Some(mut trim) = item.comp_take("minecraft:trim") {
        if let NbtValue::Compound(m) = &mut trim {
            if m.get_bool("show_in_tooltip") == Some(false) {
                item.set_hide_flag(TOOLTIP_FLAG_HIDE_UPGRADES);
                m.take("show_in_tooltip");
            }
        }
        item.tag.set_generic("Trim", trim);
    }

    // hide_additional_tooltip -> HideFlags bit (no other legacy form).
    if item
        .comp_take("minecraft:hide_additional_tooltip")
        .is_some()
    {
        item.set_hide_flag(TOOLTIP_FLAG_HIDE_ADDITIONAL);
    }

    // --- per-item handlers, mirroring the forward switch ---
    match item.id.as_str() {
        "minecraft:enchanted_book" => unconvert_enchantments(
            &mut item,
            "minecraft:stored_enchantments",
            "StoredEnchantments",
            TOOLTIP_FLAG_HIDE_ADDITIONAL,
        ),
        "minecraft:crossbow" => {
            // forward dropped Charged and moved ChargedProjectiles.
            item.comp_migrate_to_tag("minecraft:charged_projectiles", "ChargedProjectiles");
        }
        "minecraft:bundle" => {
            item.comp_migrate_to_tag("minecraft:bundle_contents", "Items");
        }
        "minecraft:filled_map" => unconvert_map(&mut item),
        "minecraft:potion"
        | "minecraft:splash_potion"
        | "minecraft:lingering_potion"
        | "minecraft:tipped_arrow" => unconvert_potion(&mut item),
        "minecraft:writable_book" => unconvert_writable_book(&mut item),
        "minecraft:written_book" => unconvert_written_book(&mut item),
        "minecraft:suspicious_stew" => {
            item.comp_migrate_to_tag("minecraft:suspicious_stew_effects", "effects");
        }
        "minecraft:debug_stick" => {
            item.comp_migrate_to_tag("minecraft:debug_stick_state", "DebugProperty");
        }
        "minecraft:pufferfish_bucket"
        | "minecraft:salmon_bucket"
        | "minecraft:cod_bucket"
        | "minecraft:tropical_fish_bucket"
        | "minecraft:axolotl_bucket"
        | "minecraft:tadpole_bucket" => unconvert_mob_bucket(&mut item),
        "minecraft:goat_horn" => {
            item.comp_migrate_to_tag("minecraft:instrument", "instrument");
        }
        "minecraft:knowledge_book" => {
            item.comp_migrate_to_tag("minecraft:recipes", "Recipes");
        }
        "minecraft:compass" => unconvert_compass(&mut item),
        "minecraft:firework_rocket" => unconvert_firework_rocket(&mut item),
        "minecraft:firework_star" => unconvert_firework_star(&mut item),
        "minecraft:player_head" => unconvert_skull(&mut item),
        _ => {}
    }

    // Apply damage now (after per-item handlers) so tag.Damage lands in the tag.
    let mut result = item.serialize();
    if let Some(d) = damage {
        // base Damage (short) on the root.
        result.set_short("Damage", d as i16);
        // tag.Damage (int) inside the tag.
        match result.get_map_mut("tag") {
            Some(tag) => tag.set_i32("Damage", d as i32),
            None => {
                let mut tag = NbtMap::new();
                tag.set_i32("Damage", d as i32);
                result.set_map("tag", tag);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, count: i8) -> NbtMap {
        let mut m = NbtMap::new();
        m.set_string("id", id);
        m.set_byte("Count", count);
        m
    }

    #[test]
    fn damage_and_repair_migrate_to_components() {
        let mut it = item("minecraft:diamond_sword", 1);
        let mut tag = NbtMap::new();
        tag.set_i32("Damage", 5);
        tag.set_i32("RepairCost", 3);
        it.set_map("tag", tag);

        let out = convert_item(&it);
        assert_eq!(out.get_string("id"), Some("minecraft:diamond_sword"));
        assert_eq!(out.get_i64("count"), Some(1));
        let comps = out.get_map("components").expect("components");
        assert_eq!(comps.get_i64("minecraft:damage"), Some(5));
        assert_eq!(comps.get_i64("minecraft:repair_cost"), Some(3));
        assert!(out.get_map("tag").is_none());
    }

    #[test]
    fn enchantments_become_levels_map() {
        let mut it = item("minecraft:diamond_sword", 1);
        let mut tag = NbtMap::new();
        let mut ench = NbtMap::new();
        ench.set_string("id", "minecraft:sharpness");
        ench.set_short("lvl", 5);
        tag.set_list("Enchantments", vec![NbtValue::Compound(ench)]);
        it.set_map("tag", tag);

        let out = convert_item(&it);
        let comps = out.get_map("components").unwrap();
        let levels = comps
            .get_map("minecraft:enchantments")
            .unwrap()
            .get_map("levels")
            .unwrap();
        assert_eq!(levels.get_i64("minecraft:sharpness"), Some(5));
    }

    #[test]
    fn block_entity_chest_items_become_container() {
        // a shulker box item carrying a chest's worth of items in BlockEntityTag
        let mut inner_item = NbtMap::new();
        inner_item.set_string("id", "minecraft:stone");
        inner_item.set_byte("Count", 1);
        inner_item.set_byte("Slot", 2);

        let mut be = NbtMap::new();
        be.set_string("id", "minecraft:shulker_box");
        be.set_list("Items", vec![NbtValue::Compound(inner_item)]);

        let mut it = item("minecraft:shulker_box", 1);
        let mut tag = NbtMap::new();
        tag.set_map("BlockEntityTag", be);
        it.set_map("tag", tag);

        let out = convert_item(&it);
        let comps = out.get_map("components").unwrap();
        let container = comps.get_list("minecraft:container").unwrap();
        let slot0 = container[0].as_compound_ref().unwrap();
        assert_eq!(slot0.get_i64("slot"), Some(2));
        assert_eq!(
            slot0.get_map("item").unwrap().get_string("id"),
            Some("minecraft:stone")
        );
        // Slot key stripped from the inner item
        assert!(slot0.get_map("item").unwrap().get("Slot").is_none());
    }

    #[test]
    fn leftover_tag_becomes_custom_data() {
        let mut it = item("minecraft:stone", 1);
        let mut tag = NbtMap::new();
        tag.set_string("MyCustomKey", "hello");
        it.set_map("tag", tag);

        let out = convert_item(&it);
        let comps = out.get_map("components").unwrap();
        let custom = comps.get_map("minecraft:custom_data").unwrap();
        assert_eq!(custom.get_string("MyCustomKey"), Some("hello"));
    }

    #[test]
    fn no_tag_no_components() {
        let it = item("minecraft:stone", 1);
        let out = convert_item(&it);
        assert!(out.get_map("components").is_none());
        assert_eq!(out.get_i64("count"), Some(1));
    }

    // ===================================================================
    // REVERSE round-trip tests: unconvert_item(convert_item(x)) == x
    // ===================================================================

    /// Convenience: forward then reverse.
    fn round_trip(it: &NbtMap) -> NbtMap {
        unconvert_item(&convert_item(it))
    }

    #[test]
    fn rt_damage_and_repair() {
        let mut it = item("minecraft:diamond_sword", 1);
        let mut tag = NbtMap::new();
        tag.set_i32("Damage", 5);
        tag.set_i32("RepairCost", 3);
        it.set_map("tag", tag);

        let back = round_trip(&it);
        assert_eq!(back.get_string("id"), Some("minecraft:diamond_sword"));
        assert_eq!(back.get_i64("Count"), Some(1));
        let tag = back.get_map("tag").expect("tag");
        assert_eq!(tag.get_i64("Damage"), Some(5));
        assert_eq!(tag.get_i64("RepairCost"), Some(3));
        // spec: minecraft:damage restores the base root Damage too.
        assert_eq!(back.get_i64("Damage"), Some(5));
        // no leftover components since both were inverted.
        assert!(back.get_map("components").is_none());
    }

    #[test]
    fn rt_enchantments_levels_map() {
        let mut it = item("minecraft:diamond_sword", 1);
        let mut tag = NbtMap::new();
        let mut ench = NbtMap::new();
        ench.set_string("id", "minecraft:sharpness");
        ench.set_short("lvl", 5);
        tag.set_list("Enchantments", vec![NbtValue::Compound(ench)]);
        it.set_map("tag", tag);

        let back = round_trip(&it);
        let tag = back.get_map("tag").expect("tag");
        let ench_list = tag.get_list("Enchantments").expect("Enchantments");
        assert_eq!(ench_list.len(), 1);
        let e = ench_list[0].as_compound_ref().unwrap();
        assert_eq!(e.get_string("id"), Some("minecraft:sharpness"));
        assert_eq!(e.get_i64("lvl"), Some(5));
    }

    #[test]
    fn rt_display_name_and_lore() {
        let mut it = item("minecraft:stone", 1);
        let mut tag = NbtMap::new();
        let mut display = NbtMap::new();
        // forward only keeps valid-JSON Name/Lore entries.
        display.set_string("Name", "{\"text\":\"Cool Rock\"}");
        display.set_list(
            "Lore",
            vec![
                NbtValue::String("{\"text\":\"line one\"}".to_string()),
                NbtValue::String("{\"text\":\"line two\"}".to_string()),
            ],
        );
        tag.set_map("display", display);
        it.set_map("tag", tag);

        let back = round_trip(&it);
        let display = back
            .get_map("tag")
            .expect("tag")
            .get_map("display")
            .expect("display");
        assert_eq!(display.get_string("Name"), Some("{\"text\":\"Cool Rock\"}"));
        let lore = display.get_list("Lore").expect("Lore");
        assert_eq!(lore.len(), 2);
        assert_eq!(lore[0].as_str(), Some("{\"text\":\"line one\"}"));
        assert_eq!(lore[1].as_str(), Some("{\"text\":\"line two\"}"));
    }

    #[test]
    fn rt_chest_container_items() {
        let mut inner_item = NbtMap::new();
        inner_item.set_string("id", "minecraft:stone");
        inner_item.set_byte("Count", 1);
        inner_item.set_byte("Slot", 2);

        let mut be = NbtMap::new();
        be.set_string("id", "minecraft:shulker_box");
        be.set_list("Items", vec![NbtValue::Compound(inner_item)]);

        let mut it = item("minecraft:shulker_box", 1);
        let mut tag = NbtMap::new();
        tag.set_map("BlockEntityTag", be);
        it.set_map("tag", tag);

        let back = round_trip(&it);
        // Note: the BlockEntityTag `id` is NOT recoverable here — the forward
        // converter, after extracting Items into `minecraft:container`, drops a
        // tile that is left with only `{id}` (it stores block_entity_data only
        // when more than the id remains). So the inverse reconstructs Items via
        // the container component but cannot restore the original `id`.
        let be = back
            .get_map("tag")
            .expect("tag")
            .get_map("BlockEntityTag")
            .expect("BlockEntityTag");
        let items = be.get_list("Items").expect("Items");
        assert_eq!(items.len(), 1);
        let it0 = items[0].as_compound_ref().unwrap();
        assert_eq!(it0.get_string("id"), Some("minecraft:stone"));
        assert_eq!(it0.get_i64("Slot"), Some(2));
    }

    #[test]
    fn rt_leftover_unknown_tag_via_custom_data() {
        let mut it = item("minecraft:stone", 1);
        let mut tag = NbtMap::new();
        tag.set_string("MyCustomKey", "hello");
        tag.set_i32("AnotherKey", 42);
        it.set_map("tag", tag);

        let back = round_trip(&it);
        let tag = back.get_map("tag").expect("tag");
        assert_eq!(tag.get_string("MyCustomKey"), Some("hello"));
        assert_eq!(tag.get_i64("AnotherKey"), Some(42));
        assert!(back.get_map("components").is_none());
    }

    #[test]
    fn rt_no_tag_no_components() {
        let it = item("minecraft:stone", 1);
        let back = round_trip(&it);
        assert_eq!(back.get_string("id"), Some("minecraft:stone"));
        assert_eq!(back.get_i64("Count"), Some(1));
        assert!(back.get_map("tag").is_none());
        assert!(back.get_map("components").is_none());
    }

    #[test]
    fn unconvert_drops_modern_only_component_with_report() {
        // an item with a modern-only component and nothing legacy.
        let mut comps = NbtMap::new();
        comps.set_string("minecraft:rarity", "epic");
        let mut modern = NbtMap::new();
        modern.set_string("id", "minecraft:stone");
        modern.set_i32("count", 1);
        modern.set_map("components", comps);

        let (back, report) = super::super::loss::run_reverse(|| unconvert_item(&modern));
        // rarity has no legacy form -> dropped, tag empty.
        assert!(back.get_map("tag").is_none());
        assert_eq!(report.loss_count(), 1);
        assert!(report.summary().contains("minecraft:rarity"));
    }

    #[test]
    fn unconvert_reports_unknown_fields_inside_known_component() {
        let mut potion_contents = NbtMap::new();
        potion_contents.set_string("potion", "minecraft:strong_healing");
        potion_contents.set_string("future_key", "future_value");

        let mut comps = NbtMap::new();
        comps.set_map("minecraft:potion_contents", potion_contents);

        let mut modern = NbtMap::new();
        modern.set_string("id", "minecraft:potion");
        modern.set_i32("count", 1);
        modern.set_map("components", comps);

        let (back, report) = super::super::loss::run_reverse(|| unconvert_item(&modern));
        assert_eq!(
            back.get_map("tag").and_then(|tag| tag.get_string("Potion")),
            Some("minecraft:strong_healing")
        );
        assert_eq!(report.loss_count(), 1);
        assert!(report
            .summary()
            .contains("minecraft:potion_contents.future_key"));
    }

    #[test]
    fn unconvert_block_state_reports_stringified_property_approximation() {
        let mut block_state = NbtMap::new();
        block_state.set_string("powered", "true");

        let mut comps = NbtMap::new();
        comps.set_map("minecraft:block_state", block_state);

        let mut modern = NbtMap::new();
        modern.set_string("id", "minecraft:stone");
        modern.set_i32("count", 1);
        modern.set_map("components", comps);

        let (back, report) = super::super::loss::run_reverse(|| unconvert_item(&modern));
        assert!(back
            .get_map("tag")
            .and_then(|tag| tag.get_map("BlockStateTag"))
            .is_some());
        assert_eq!(report.loss_count(), 0);
        assert_eq!(report.len(), 1);
        assert!(report.summary().contains("minecraft:block_state"));
    }

    #[test]
    fn unconvert_input_passthrough_when_not_squashed() {
        // No `count` and no `components` -> not a squashed item, returned as-is.
        let it = item("minecraft:stone", 1); // has Count(byte) but no count(int)
        let back = unconvert_item(&it);
        assert_eq!(back, it);
    }
}
