//! A Rust port of spottedleaf/PaperMC's DataConverter
//! (`DataConverterJava/`), restricted to the types that appear in a schematic
//! file, used to convert block-state / block-entity / entity / item NBT between
//! Minecraft data versions.
//!
//! Forward conversion (old -> new) mirrors the Java engine exactly. Reverse
//! conversion (new -> old), needed to save schematics for older versions, is
//! built on the same engine ([`engine::walk_with_breakpoints_reverse`]) using
//! inverse rename tables and hand-written / best-effort inverses, and reports
//! every approximation or dropped field via [`loss::LossReport`].
//!
//! See `src/dataconverter/README.md` for the design overview and module map,
//! and `REVERSE_CHEATSHEET.md` for authoring reverse converters.

/// The in-memory canonical Minecraft data version — MC "26.1.2". All loaded data
/// is forward-converted to here; "save as current" stamps this. (See
/// `src/dataconverter/README.md`.)
pub const CANONICAL_DATA_VERSION: i32 = 4790;

/// The highest data version with a registered forward converter in this port
/// (the DataConverter handoff seam). Conversion within `(this, CANONICAL]` is a
/// no-op shape-wise — nothing changed between it and the canonical target.
pub const DATACONVERTER_FORWARD_MAX: i32 = 4763;

pub mod apply;
pub mod components;
pub mod engine;
pub mod flattening;
pub mod helpers;
pub mod loss;
pub mod registry;
pub mod types;
pub mod version;
mod versions;
pub mod walker;

pub use apply::{
    convert_block_entity_struct, convert_block_state_struct, convert_palette, convert_region,
    convert_schematic, convert_schematic_reverse,
};
pub use loss::{LossEntry, LossKind, LossReport, Severity};
pub use registry::{
    convert_block_entity, convert_block_entity_reverse, convert_block_state,
    convert_block_state_reverse, convert_entity, convert_entity_reverse, convert_item_stack,
    convert_item_stack_reverse, convert_structure, convert_structure_reverse, registry,
};
pub use version::{encode_versions, get_step, get_version, EncodedVersion};

#[cfg(test)]
mod tests {
    use super::engine::walk_with_breakpoints;
    use super::helpers::invert_pairs;
    use super::types::{MapExt, ValueExt};
    use super::versions;
    use super::*;
    use crate::entity::Entity;
    use crate::nbt::{NbtMap, NbtValue};
    use crate::UniversalSchematic;

    #[test]
    fn breakpoint_walk_splits_into_segments() {
        let bps = vec![10u64, 20u64];
        let mut calls = Vec::new();
        walk_with_breakpoints(&bps, 5, 25, |f, t| calls.push((f, t)));
        assert_eq!(calls, vec![(5, 9), (9, 19), (19, 25)]);
    }

    #[test]
    fn breakpoint_walk_no_op_when_equal() {
        let bps = vec![10u64];
        let mut calls = Vec::new();
        walk_with_breakpoints(&bps, 7, 7, |f, t| calls.push((f, t)));
        // from == to: a single zero-width segment.
        assert_eq!(calls, vec![(7, 7)]);
    }

    #[test]
    fn forward_item_rename() {
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:pottery_shard_archer");
        item.set_byte("Count", 1);
        convert_item_stack(&mut item, 3437, 3438);
        assert_eq!(
            item.get_string("id"),
            Some("minecraft:archer_pottery_shard")
        );
    }

    #[test]
    fn forward_rename_recurses_into_container_items() {
        // A legacy "Chest" tile entity whose Items list holds a renamable shard.
        let mut inner = NbtMap::new();
        inner.set_string("id", "minecraft:pottery_shard_skull");
        inner.set_byte("Count", 1);

        let mut chest = NbtMap::new();
        chest.set_string("id", "Chest");
        chest.set_list("Items", vec![NbtValue::Compound(inner)]);

        convert_block_entity(&mut chest, 3437, 3438);

        let items = chest.get_list("Items").expect("Items list");
        let nested = items[0].as_compound_ref().expect("item compound");
        assert_eq!(
            nested.get_string("id"),
            Some("minecraft:skull_pottery_shard")
        );
    }

    #[test]
    fn namespace_enforcement_on_legacy_ids() {
        // An unnamespaced item id is corrected to minecraft: by the V99 hook.
        let mut item = NbtMap::new();
        item.set_string("id", "stone");
        item.set_byte("Count", 1);
        convert_item_stack(&mut item, 99, 3438);
        assert_eq!(item.get_string("id"), Some("minecraft:stone"));
    }

    #[test]
    fn v135_riding_becomes_passengers() {
        // A pig riding a minecart: the legacy Riding chain inverts so the topmost
        // vehicle becomes the root with the rider in its Passengers list.
        let mut vehicle = NbtMap::new();
        vehicle.set_string("id", "Minecart");
        let mut pig = NbtMap::new();
        pig.set_string("id", "Pig");
        pig.set_map("Riding", vehicle);

        convert_entity(&mut pig, 134, 135);

        assert_eq!(pig.get_string("id"), Some("Minecart"));
        assert!(pig.get_map("Riding").is_none());
        let passengers = pig.get_list("Passengers").expect("Passengers");
        assert_eq!(
            passengers[0].as_compound_ref().unwrap().get_string("id"),
            Some("Pig")
        );
    }

    #[test]
    fn v701_skeleton_split_by_type() {
        let mut sk = NbtMap::new();
        sk.set_string("id", "Skeleton");
        sk.set_i32("SkeletonType", 1);
        convert_entity(&mut sk, 700, 701);
        assert_eq!(sk.get_string("id"), Some("WitherSkeleton"));
        assert!(!sk.has_key("SkeletonType"));
    }

    #[test]
    fn v704_namespaces_tile_id_and_copywalkers_preserves_item_recursion() {
        // A legacy "Chest" with an item; convert across V704 (id namespacing +
        // copyWalkers) and V3438 (item rename), spanning the Flattening
        // breakpoint. The namespaced chest must still walk its Items, and the
        // contained item must be renamed.
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:pottery_shard_archer");
        item.set_byte("Count", 1);
        let mut chest = NbtMap::new();
        chest.set_string("id", "Chest");
        chest.set_list("Items", vec![NbtValue::Compound(item)]);

        convert_block_entity(&mut chest, 703, 3438);

        assert_eq!(chest.get_string("id"), Some("minecraft:chest"));
        let items = chest.get_list("Items").expect("Items");
        let nested = items[0].as_compound_ref().expect("item");
        assert_eq!(
            nested.get_string("id"),
            Some("minecraft:archer_pottery_shard")
        );
    }

    #[test]
    fn v1486_entity_rename() {
        let mut e = NbtMap::new();
        e.set_string("id", "minecraft:salmon_mob");
        convert_entity(&mut e, 1485, 1486);
        assert_eq!(e.get_string("id"), Some("minecraft:salmon"));
    }

    #[test]
    fn v1490_block_and_item_renames() {
        let mut block = NbtMap::new();
        block.set_string("Name", "minecraft:melon_block");
        convert_block_state(&mut block, 1489, 1490);
        assert_eq!(block.get_string("Name"), Some("minecraft:melon"));

        // Single-pass lookup: a `melon` item becomes `melon_slice` (not chained).
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:melon");
        item.set_byte("Count", 1);
        convert_item_stack(&mut item, 1489, 1490);
        assert_eq!(item.get_string("id"), Some("minecraft:melon_slice"));
    }

    #[test]
    fn v1456_item_frame_facing_remap() {
        let mut frame = NbtMap::new();
        frame.set_string("id", "minecraft:item_frame");
        frame.set_byte("Facing", 0);
        convert_entity(&mut frame, 1455, 1456);
        assert_eq!(frame.get_i64("Facing"), Some(3)); // 2d 0 -> 3d 3
    }

    #[test]
    fn v1125_legacy_bed_becomes_red() {
        let mut bed = NbtMap::new();
        bed.set_string("id", "minecraft:bed");
        bed.set_short("Damage", 0);
        bed.set_byte("Count", 1);
        convert_item_stack(&mut bed, 1124, 1125);
        assert_eq!(bed.get_i64("Damage"), Some(14)); // red
    }

    #[test]
    fn rename_tables_are_invertible() {
        let inv = invert_pairs(versions::POTTERY_SHARD_RENAMES);
        assert!(inv.contains(&(
            "minecraft:archer_pottery_shard",
            "minecraft:pottery_shard_archer"
        )));
    }

    // --- the 1.13 Flattening, end to end through the real entry points ------

    #[test]
    fn v1450_flattens_block_state() {
        // legacy {Name:stone, Properties:{variant:granite}} -> {Name:granite}
        let mut bs = NbtMap::new();
        bs.set_string("Name", "minecraft:stone");
        let mut props = NbtMap::new();
        props.set_string("variant", "granite");
        bs.set_map("Properties", props);

        convert_block_state(&mut bs, 1449, 1451);

        assert_eq!(bs.get_string("Name"), Some("minecraft:granite"));
        assert!(bs.get_map("Properties").is_none());
    }

    #[test]
    fn v1451_flattens_item_and_migrates_durability() {
        // subtype item: wool damage 14 -> red_wool, Damage dropped
        let mut wool = NbtMap::new();
        wool.set_string("id", "minecraft:wool");
        wool.set_short("Damage", 14);
        wool.set_byte("Count", 1);
        convert_item_stack(&mut wool, 1449, 1451);
        assert_eq!(wool.get_string("id"), Some("minecraft:red_wool"));
        assert!(!wool.has_key("Damage"));

        // durability item: a damaged sword keeps its id, Damage -> tag.Damage
        let mut sword = NbtMap::new();
        sword.set_string("id", "minecraft:diamond_sword");
        sword.set_short("Damage", 42);
        sword.set_byte("Count", 1);
        convert_item_stack(&mut sword, 1449, 1451);
        assert_eq!(sword.get_string("id"), Some("minecraft:diamond_sword"));
        assert!(!sword.has_key("Damage"));
        assert_eq!(sword.get_map("tag").unwrap().get_i64("Damage"), Some(42));
    }

    #[test]
    fn v1451_piston_block_entity_flattens_blockstate() {
        // blockId 1 (=stone), blockData 0 -> blockState {Name:stone}
        let mut piston = NbtMap::new();
        piston.set_string("id", "minecraft:piston");
        piston.set_i32("blockId", 1);
        piston.set_i32("blockData", 0);
        convert_block_entity(&mut piston, 1449, 1451);
        assert!(!piston.has_key("blockId"));
        assert_eq!(
            piston.get_map("blockState").unwrap().get_string("Name"),
            Some("minecraft:stone")
        );
    }

    #[test]
    fn v1451_falling_block_entity_flattens_named_block() {
        let mut fb = NbtMap::new();
        fb.set_string("id", "minecraft:falling_block");
        fb.set_string("Block", "minecraft:stone");
        fb.set_i32("Data", 0);
        convert_entity(&mut fb, 1449, 1451);
        assert!(!fb.has_key("Block") && !fb.has_key("Data"));
        assert_eq!(
            fb.get_map("BlockState").unwrap().get_string("Name"),
            Some("minecraft:stone")
        );
    }

    #[test]
    fn v1451_jukebox_record_becomes_record_item() {
        let mut jb = NbtMap::new();
        jb.set_string("id", "minecraft:jukebox");
        jb.set_i32("Record", 2256); // numeric id of record_13
        convert_block_entity(&mut jb, 1449, 1451);
        assert!(!jb.has_key("Record"));
        let item = jb.get_map("RecordItem").expect("RecordItem");
        assert_eq!(item.get_string("id"), Some("minecraft:music_disc_13"));
        assert_eq!(item.get_i64("Count"), Some(1));
    }

    #[test]
    fn v3818_components_squash_end_to_end() {
        // A pre-1.20.5 enchanted, damaged sword crosses the 1.20.5 component
        // breakpoint: tag.{Damage,Enchantments} -> components.
        let mut tag = NbtMap::new();
        tag.set_i32("Damage", 5);
        let mut ench = NbtMap::new();
        ench.set_string("id", "minecraft:sharpness");
        ench.set_short("lvl", 3);
        tag.set_list("Enchantments", vec![NbtValue::Compound(ench)]);

        let mut sword = NbtMap::new();
        sword.set_string("id", "minecraft:diamond_sword");
        sword.set_byte("Count", 1);
        sword.set_map("tag", tag);

        convert_item_stack(&mut sword, 3700, 3820);

        assert_eq!(sword.get_string("id"), Some("minecraft:diamond_sword"));
        assert_eq!(sword.get_i64("count"), Some(1));
        assert!(sword.get_map("tag").is_none());
        let comps = sword.get_map("components").expect("components");
        assert_eq!(comps.get_i64("minecraft:damage"), Some(5));
        assert_eq!(
            comps
                .get_map("minecraft:enchantments")
                .unwrap()
                .get_map("levels")
                .unwrap()
                .get_i64("minecraft:sharpness"),
            Some(3)
        );
    }

    #[test]
    fn v3818_container_item_survives_via_block_entity_data() {
        // A pre-1.20.5 chest block entity holding a sword survives load: the
        // block-entity Items walk recurses each item through the 1.20.5 squash.
        let mut sword = NbtMap::new();
        sword.set_string("id", "minecraft:diamond_sword");
        sword.set_byte("Count", 1);
        sword.set_byte("Slot", 0);
        let mut tag = NbtMap::new();
        tag.set_i32("Damage", 7);
        sword.set_map("tag", tag);

        let mut chest = NbtMap::new();
        chest.set_string("id", "minecraft:chest");
        chest.set_list("Items", vec![NbtValue::Compound(sword)]);

        convert_block_entity(&mut chest, 3700, 3820);

        let items = chest.get_list("Items").expect("Items");
        let item = items[0].as_compound_ref().unwrap();
        assert_eq!(item.get_string("id"), Some("minecraft:diamond_sword"));
        // squashed to components form
        assert_eq!(
            item.get_map("components")
                .unwrap()
                .get_i64("minecraft:damage"),
            Some(7)
        );
    }

    #[test]
    fn v3818_banner_block_entity_pattern_and_colour() {
        let mut pattern = NbtMap::new();
        pattern.set_string("Pattern", "bo"); // -> minecraft:border
        pattern.set_i32("Color", 4); // -> yellow
        let mut banner = NbtMap::new();
        banner.set_string("id", "minecraft:banner");
        banner.set_list("Patterns", vec![NbtValue::Compound(pattern)]);

        convert_block_entity(&mut banner, 3700, 3820);

        let patterns = banner.get_list("patterns").expect("renamed to patterns");
        let p = patterns[0].as_compound_ref().unwrap();
        assert_eq!(p.get_string("pattern"), Some("minecraft:border"));
        assert_eq!(p.get_string("color"), Some("yellow"));
    }

    #[test]
    fn v1451_spawn_egg_resolves_entity_id() {
        let mut egg_tag = NbtMap::new();
        let mut entity_tag = NbtMap::new();
        entity_tag.set_string("id", "minecraft:creeper");
        egg_tag.set_map("EntityTag", entity_tag);
        let mut egg = NbtMap::new();
        egg.set_string("id", "minecraft:spawn_egg");
        egg.set_byte("Count", 1);
        egg.set_map("tag", egg_tag);
        convert_item_stack(&mut egg, 1449, 1451);
        assert_eq!(egg.get_string("id"), Some("minecraft:creeper_spawn_egg"));
    }

    #[test]
    fn v3209_reverse_typed_spawn_egg_restores_entity_tag() {
        let mut egg = NbtMap::new();
        egg.set_string("id", "minecraft:cow_spawn_egg");
        egg.set_i32("count", 1);

        let report = convert_item_stack_reverse(&mut egg, 3209, 3208);

        assert!(report.is_empty());
        assert_eq!(egg.get_string("id"), Some("minecraft:pig_spawn_egg"));
        assert_eq!(
            egg.get_map("tag")
                .and_then(|tag| tag.get_map("EntityTag"))
                .and_then(|entity_tag| entity_tag.get_string("id")),
            Some("minecraft:cow")
        );
    }

    #[test]
    fn v3209_reverse_spawn_egg_reports_conflicting_entity_tag() {
        let mut entity_tag = NbtMap::new();
        entity_tag.set_string("id", "minecraft:pig");
        let mut tag = NbtMap::new();
        tag.set_map("EntityTag", entity_tag);
        let mut egg = NbtMap::new();
        egg.set_string("id", "minecraft:cow_spawn_egg");
        egg.set_map("tag", tag);

        let report = convert_item_stack_reverse(&mut egg, 3209, 3208);

        assert_eq!(report.entries.len(), 1);
        assert_eq!(
            egg.get_map("tag")
                .and_then(|tag| tag.get_map("EntityTag"))
                .and_then(|entity_tag| entity_tag.get_string("id")),
            Some("minecraft:cow")
        );
    }

    #[test]
    fn v3209_reverse_newer_spawn_egg_reports_unsupported() {
        let mut egg = NbtMap::new();
        egg.set_string("id", "minecraft:sniffer_spawn_egg");

        let report = convert_item_stack_reverse(&mut egg, 3209, 3208);

        assert_eq!(report.loss_count(), 1);
        assert_eq!(egg.get_string("id"), Some("minecraft:pig_spawn_egg"));
        assert_eq!(
            egg.get_map("tag")
                .and_then(|tag| tag.get_map("EntityTag"))
                .and_then(|entity_tag| entity_tag.get_string("id")),
            Some("minecraft:sniffer")
        );
    }

    #[test]
    fn v3093_reverse_goat_missing_horn_reports_loss() {
        let mut goat = NbtMap::new();
        goat.set_string("id", "minecraft:goat");
        goat.set_bool("HasLeftHorn", false);
        goat.set_bool("HasRightHorn", true);

        let report = convert_entity_reverse(&mut goat, 3093, 3092);

        assert_eq!(report.loss_count(), 1);
        assert!(!goat.has_key("HasLeftHorn"));
        assert!(!goat.has_key("HasRightHorn"));
    }

    #[test]
    fn v3327_decorated_pot_walks_legacy_shards_and_item() {
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:pottery_shard_prize");
        let mut pot = NbtMap::new();
        pot.set_string("id", "minecraft:decorated_pot");
        pot.set_list(
            "shards",
            vec![NbtValue::String(
                "minecraft:pottery_shard_archer".to_string(),
            )],
        );
        pot.set_map("item", item);

        convert_block_entity(&mut pot, 3326, 3438);

        assert_eq!(
            pot.get_list("shards").unwrap()[0].as_str(),
            Some("minecraft:archer_pottery_shard")
        );
        assert_eq!(
            pot.get_map("item").unwrap().get_string("id"),
            Some("minecraft:prize_pottery_shard")
        );
    }

    #[test]
    fn v3439_reverse_sign_reports_back_text_metadata() {
        let mut back_text = NbtMap::new();
        back_text.set_list(
            "messages",
            vec![
                NbtValue::String(r#"{"text":""}"#.to_string()),
                NbtValue::String(r#"{"text":""}"#.to_string()),
                NbtValue::String(r#"{"text":""}"#.to_string()),
                NbtValue::String(r#"{"text":""}"#.to_string()),
            ],
        );
        back_text.set_string("color", "red");
        back_text.set_bool("has_glowing_text", true);
        let mut sign = NbtMap::new();
        sign.set_string("id", "minecraft:sign");
        sign.set_map("back_text", back_text);

        let report = convert_block_entity_reverse(&mut sign, 3439, 3438);

        assert_eq!(report.loss_count(), 2);
        assert!(!sign.has_key("back_text"));
    }

    #[test]
    fn v3564_removes_filtered_text_zero_not_four() {
        let mut sign = NbtMap::new();
        sign.set_string("id", "minecraft:sign");
        sign.set_string("FilteredText0", "zero");
        sign.set_string("FilteredText4", "four");

        convert_block_entity(&mut sign, 3563, 3564);

        assert!(!sign.has_key("FilteredText0"));
        assert_eq!(sign.get_string("FilteredText4"), Some("four"));
    }

    #[test]
    fn v3568_reverse_mooshroom_reports_extra_stew_effects() {
        let mut first = NbtMap::new();
        first.set_string("id", "minecraft:speed");
        first.set_i32("duration", 120);
        first.set_string("hidden", "extra");
        let mut second = NbtMap::new();
        second.set_string("id", "minecraft:slowness");
        let mut mooshroom = NbtMap::new();
        mooshroom.set_string("id", "minecraft:mooshroom");
        mooshroom.set_list(
            "stew_effects",
            vec![NbtValue::Compound(first), NbtValue::Compound(second)],
        );

        let report = convert_entity_reverse(&mut mooshroom, 3568, 3567);

        assert_eq!(report.loss_count(), 2);
        assert_eq!(mooshroom.get_i32("EffectId"), Some(1));
        assert_eq!(mooshroom.get_i32("EffectDuration"), Some(120));
    }

    #[test]
    fn v3683_reverse_accepts_legacy_tnt_default_block_state() {
        let mut block_state = NbtMap::new();
        block_state.set_string("Name", "TNT");
        let mut tnt = NbtMap::new();
        tnt.set_string("id", "minecraft:tnt");
        tnt.set_map("block_state", block_state);

        let report = convert_entity_reverse(&mut tnt, 3683, 3682);

        assert!(report.is_empty());
        assert!(!tnt.has_key("block_state"));
    }

    #[test]
    fn v3808_reverse_reports_nondefault_body_armor_drop_chance() {
        let mut armor = NbtMap::new();
        armor.set_string("id", "minecraft:leather_horse_armor");
        let mut horse = NbtMap::new();
        horse.set_string("id", "minecraft:horse");
        horse.set_map("body_armor_item", armor);
        horse.set_f32("body_armor_drop_chance", 0.5);

        let report = convert_entity_reverse(&mut horse, 3808, 3807);

        assert_eq!(report.loss_count(), 1);
        assert_eq!(
            horse.get_map("ArmorItem").unwrap().get_string("id"),
            Some("minecraft:leather_horse_armor")
        );
        assert!(!horse.has_key("body_armor_drop_chance"));
    }

    #[test]
    fn v3812_reverse_wolf_health_collapse_reports_approximation() {
        let mut attr = NbtMap::new();
        attr.set_string("Name", "generic.max_health");
        attr.set_f64("Base", 40.0);
        let mut wolf = NbtMap::new();
        wolf.set_string("id", "minecraft:wolf");
        wolf.set_f32("Health", 30.0);
        wolf.set_list("Attributes", vec![NbtValue::Compound(attr)]);

        let report = convert_entity_reverse(&mut wolf, 3812, 3811);

        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.loss_count(), 0);
        assert_eq!(wolf.get_f64("Health"), Some(15.0));
        let attrs = wolf.get_list("Attributes").unwrap();
        assert_eq!(
            attrs[0].as_compound_ref().unwrap().get_f64("Base"),
            Some(20.0)
        );
    }

    #[test]
    fn v3818_reverse_banner_unknown_color_reports() {
        let mut pattern = NbtMap::new();
        pattern.set_string("pattern", "minecraft:border");
        pattern.set_string("color", "chartreuse");
        let mut banner = NbtMap::new();
        banner.set_string("id", "minecraft:banner");
        banner.set_list("patterns", vec![NbtValue::Compound(pattern)]);

        let report = convert_block_entity_reverse(&mut banner, 3818, 3817);

        assert_eq!(report.entries.len(), 1);
        let pattern = banner.get_list("Patterns").unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(pattern.get_i32("Color"), Some(0));
    }

    #[test]
    fn v3818_reverse_area_effect_cloud_preserves_malformed_potion_contents() {
        let mut cloud = NbtMap::new();
        cloud.set_string("id", "minecraft:area_effect_cloud");
        cloud.set_string("potion_contents", "bad");

        let report = convert_entity_reverse(&mut cloud, 3818, 3817);

        assert!(report.is_empty());
        assert_eq!(cloud.get_string("potion_contents"), Some("bad"));
    }

    #[test]
    fn v3818_reverse_area_effect_cloud_reports_extra_potion_contents() {
        let mut contents = NbtMap::new();
        contents.set_string("potion", "minecraft:water");
        contents.set_string("custom_name", "extra");
        let mut cloud = NbtMap::new();
        cloud.set_string("id", "minecraft:area_effect_cloud");
        cloud.set_map("potion_contents", contents);

        let report = convert_entity_reverse(&mut cloud, 3818, 3817);

        assert_eq!(report.loss_count(), 1);
        assert_eq!(cloud.get_string("Potion"), Some("minecraft:water"));
        assert!(!cloud.has_key("potion_contents"));
    }

    #[test]
    fn reverse_arrow_reports_non_synthesized_item_drop() {
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:tipped_arrow");
        item.set_i32("Count", 1);
        let mut tag = NbtMap::new();
        tag.set_string("CustomName", "{\"text\":\"kept\"}");
        item.set_map("tag", tag);

        let mut arrow = NbtMap::new();
        arrow.set_string("id", "minecraft:arrow");
        arrow.set_string("Potion", "minecraft:healing");
        arrow.set_map("item", item);

        let report = convert_entity_reverse(&mut arrow, 3685, 3684);

        assert!(arrow.get_map("item").is_none());
        assert_eq!(report.loss_count(), 1);
        assert!(report.summary().contains("arrow item field"));
    }

    #[test]
    fn schematic_conversion_reaches_mobile_entities() {
        let mut schematic = UniversalSchematic::new("entities".to_string());
        schematic.add_entity(Entity::new(
            "minecraft:salmon_mob".to_string(),
            (0.5, 0.0, 0.5),
        ));

        convert_schematic(&mut schematic, 1485, 1486);

        assert_eq!(schematic.default_region.entities[0].id, "minecraft:salmon");
    }

    #[test]
    fn schematic_reverse_reaches_mobile_entities() {
        let mut schematic = UniversalSchematic::new("entities".to_string());
        schematic.add_entity(Entity::new("minecraft:salmon".to_string(), (0.5, 0.0, 0.5)));

        let report = convert_schematic_reverse(&mut schematic, 1486, 1485);

        assert!(report.is_empty());
        assert_eq!(
            schematic.default_region.entities[0].id,
            "minecraft:salmon_mob"
        );
    }

    // --- gap-remediation regression tests (audit fixes) ---------------------

    #[test]
    fn v99_villager_trade_items_convert() {
        // A villager's trade buy item must convert (VILLAGER_TRADE walker, the
        // type that was entirely missing). Cross the 3438 pottery-shard rename.
        let mut buy = NbtMap::new();
        buy.set_string("id", "minecraft:pottery_shard_archer");
        buy.set_byte("Count", 1);
        let mut recipe = NbtMap::new();
        recipe.set_map("buy", buy);
        let mut offers = NbtMap::new();
        offers.set_list("Recipes", vec![NbtValue::Compound(recipe)]);
        let mut villager = NbtMap::new();
        villager.set_string("id", "minecraft:villager");
        villager.set_map("Offers", offers);

        convert_entity(&mut villager, 3437, 3438);

        let recipes = villager
            .get_map("Offers")
            .unwrap()
            .get_list("Recipes")
            .unwrap();
        let buy = recipes[0]
            .as_compound_ref()
            .unwrap()
            .get_map("buy")
            .unwrap();
        assert_eq!(buy.get_string("id"), Some("minecraft:archer_pottery_shard"));
    }

    #[test]
    fn v100_equipment_list_splits() {
        let mut sword = NbtMap::new();
        sword.set_string("id", "minecraft:diamond_sword");
        sword.set_byte("Count", 1);
        let mut boots = NbtMap::new();
        boots.set_string("id", "minecraft:diamond_boots");
        boots.set_byte("Count", 1);
        let mut mob = NbtMap::new();
        mob.set_string("id", "minecraft:zombie");
        mob.set_list(
            "Equipment",
            vec![NbtValue::Compound(sword), NbtValue::Compound(boots)],
        );

        convert_entity(&mut mob, 99, 100);

        assert!(!mob.has_key("Equipment"));
        let hand = mob.get_list("HandItems").expect("HandItems");
        assert_eq!(
            hand[0].as_compound_ref().unwrap().get_string("id"),
            Some("minecraft:diamond_sword")
        );
        let armor = mob.get_list("ArmorItems").expect("ArmorItems");
        assert_eq!(
            armor[0].as_compound_ref().unwrap().get_string("id"),
            Some("minecraft:diamond_boots")
        );
    }

    #[test]
    fn v102_numeric_item_id_becomes_name() {
        let mut item = NbtMap::new();
        item.set_i32("id", 1); // numeric stone
        item.set_byte("Count", 1);
        convert_item_stack(&mut item, 101, 102);
        assert_eq!(item.get_string("id"), Some("minecraft:stone"));
    }

    #[test]
    fn v107_minecart_type_splits_id() {
        let mut cart = NbtMap::new();
        cart.set_string("id", "Minecart");
        cart.set_i32("Type", 1); // chest
        convert_entity(&mut cart, 106, 107);
        assert_eq!(cart.get_string("id"), Some("MinecartChest"));
        assert!(!cart.has_key("Type"));
    }

    #[test]
    fn v108_uuid_string_becomes_long_pair() {
        let mut e = NbtMap::new();
        e.set_string("id", "minecraft:pig");
        e.set_string("UUID", "12345678-1234-5678-9abc-def012345678");
        convert_entity(&mut e, 107, 108);
        assert!(!e.has_key("UUID"));
        // correct JDK UUID.fromString bit layout (the fixed parse_uuid)
        assert_eq!(e.get_i64("UUIDMost"), Some(0x1234_5678_1234_5678u64 as i64));
        assert_eq!(
            e.get_i64("UUIDLeast"),
            Some(0x9abc_def0_1234_5678u64 as i64)
        );
    }

    #[test]
    fn v110_horse_saddle_flag_becomes_item() {
        let mut horse = NbtMap::new();
        horse.set_string("id", "EntityHorse");
        horse.set_bool("Saddle", true);
        convert_entity(&mut horse, 109, 110);
        assert!(!horse.has_key("Saddle"));
        assert_eq!(
            horse.get_map("SaddleItem").unwrap().get_string("id"),
            Some("minecraft:saddle")
        );
    }

    #[test]
    fn v101_sign_text_becomes_json_component() {
        let mut sign = NbtMap::new();
        sign.set_string("id", "Sign");
        sign.set_string("Text1", "hello");
        convert_block_entity(&mut sign, 100, 101);
        assert_eq!(sign.get_string("Text1"), Some(r#"{"text":"hello"}"#));
    }

    #[test]
    fn v1458_custom_name_becomes_json_component() {
        let mut e = NbtMap::new();
        e.set_string("id", "minecraft:pig");
        e.set_string("CustomName", "Bob");
        convert_entity(&mut e, 1457, 1458);
        assert_eq!(e.get_string("CustomName"), Some(r#"{"text":"Bob"}"#));
    }

    #[test]
    fn v3438_suspicious_sand_renames_to_brushable_block() {
        let mut be = NbtMap::new();
        be.set_string("id", "minecraft:suspicious_sand");
        convert_block_entity(&mut be, 3437, 3438);
        assert_eq!(be.get_string("id"), Some("minecraft:brushable_block"));
    }

    // --- reverse engine smoke tests ----------------------------------------

    #[test]
    fn reverse_block_rename_undoes_forward() {
        // V1490 renames melon_block -> melon. Reverse must restore melon_block.
        let mut map = NbtMap::new();
        map.set_string("Name", "minecraft:melon_block");
        convert_block_state(&mut map, 1489, 1490);
        assert_eq!(map.get_string("Name"), Some("minecraft:melon"));

        let report = convert_block_state_reverse(&mut map, 1490, 1489);
        assert_eq!(map.get_string("Name"), Some("minecraft:melon_block"));
        assert!(report.is_empty(), "a pure rename inverse loses nothing");
    }

    #[test]
    fn reverse_rename_round_trips_across_a_breakpoint() {
        // melon_block -> melon happens at V1490; the Flattening breakpoint sits
        // below it. Forward far past it, then reverse all the way back.
        let mut map = NbtMap::new();
        map.set_string("Name", "minecraft:melon_block");
        convert_block_state(&mut map, 1450, 3438);
        assert_eq!(map.get_string("Name"), Some("minecraft:melon"));

        let report = convert_block_state_reverse(&mut map, 3438, 1450);
        assert_eq!(map.get_string("Name"), Some("minecraft:melon_block"));
        assert!(report.is_empty());
    }

    #[test]
    fn reverse_walker_descends_and_unrenames_nested_item() {
        // A chest holding an item whose id was renamed at V3438. The reverse
        // walker must descend `Items` (via the copied minecraft:chest walker)
        // and invert the item rename, even though the chest-id inverse (V704)
        // is not yet ported.
        let mut be = NbtMap::new();
        be.set_string("id", "minecraft:chest");
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:archer_pottery_shard"); // post-V3438 name
        item.set_byte("Count", 1);
        be.set_list("Items", vec![NbtValue::Compound(item)]);

        let report = convert_block_entity_reverse(&mut be, 3438, 3437);
        let items = be.get_list("Items").expect("Items");
        assert_eq!(
            items[0].as_compound_ref().unwrap().get_string("id"),
            Some("minecraft:pottery_shard_archer") // V3438 rename inverted
        );
        assert!(report.is_empty());
    }

    #[test]
    fn loss_report_records_path_and_detail() {
        use super::loss::{self, LossKind, Severity};
        use super::walker::convert_list;

        // Drive a reverse session manually and emit a loss from within a nested
        // descent to prove the path stack + collector are wired.
        let reg = registry();
        let mut root = NbtMap::new();
        let mut child = NbtMap::new();
        child.set_string("id", "x");
        root.set_list("Items", vec![NbtValue::Compound(child)]);

        let (_, report) = loss::run_reverse(|| {
            let _scope = loss::path_scope("block_entity test");
            // Walk into Items[0] so path_scope("Items[0]") is pushed by the
            // descent helper, then report a loss as a converter would.
            let from = encode_versions(3438, get_step_max());
            let to = encode_versions(3437, get_step_max());
            convert_list(reg, &reg.item_stack, &mut root, "Items", from, to);
            loss::report_loss(1234, LossKind::Other, Severity::Loss, "demo loss");
        });

        assert_eq!(report.len(), 1);
        let e = &report.entries[0];
        assert_eq!(e.version, 1234);
        assert_eq!(e.severity, Severity::Loss);
        assert!(e.path.contains("block_entity test"), "path was: {}", e.path);
        assert_eq!(e.detail, "demo loss");
        assert_eq!(report.loss_count(), 1);
    }

    #[test]
    fn v3945_reverse_reports_invalid_attribute_operation() {
        let mut modifier = NbtMap::new();
        modifier.set_string("id", "minecraft:unknown");
        modifier.set_string("operation", "multiply_sideways");
        modifier.set_f64("amount", 1.0);
        let mut attribute = NbtMap::new();
        attribute.set_string("id", "minecraft:generic.attack_damage");
        attribute.set_list("modifiers", vec![NbtValue::Compound(modifier)]);
        let mut entity = NbtMap::new();
        entity.set_list("attributes", vec![NbtValue::Compound(attribute)]);

        let report = convert_entity_reverse(&mut entity, 3945, 3944);

        let modifier = entity.get_list("Attributes").unwrap()[0]
            .as_compound_ref()
            .unwrap()
            .get_list("Modifiers")
            .unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(modifier.get_i32("Operation"), Some(0));
        assert!(report
            .summary()
            .contains("unknown attribute modifier operation"));
    }

    #[test]
    fn v4068_reverse_reports_extra_lock_predicate_fields() {
        let mut components = NbtMap::new();
        components.set_string("minecraft:custom_name", "key");
        components.set_bool("minecraft:extra", true);
        let mut lock = NbtMap::new();
        lock.set_map("components", components);
        lock.set_string("extra", "lost");
        let mut block_entity = NbtMap::new();
        block_entity.set_map("lock", lock);

        let report = convert_block_entity_reverse(&mut block_entity, 4068, 4067);

        assert_eq!(block_entity.get_string("Lock"), Some("key"));
        assert_eq!(report.loss_count(), 1);
        assert!(report.summary().contains("fields beyond"));
    }

    #[test]
    fn v4175_reverse_reports_custom_model_data_extra_values() {
        use super::loss;
        use super::registry::convert_reverse_under_session;

        let mut floats = vec![NbtValue::Float(12.0), NbtValue::Float(34.0)];
        let mut cmd = NbtMap::new();
        cmd.set_list("floats", std::mem::take(&mut floats));
        cmd.set_string("strings", "unsupported");
        let mut components = NbtMap::new();
        components.set_map("minecraft:custom_model_data", cmd);

        let reg = registry();
        let (_, report) = loss::run_reverse(|| {
            convert_reverse_under_session(&reg.data_components, &mut components, 4175, 4174)
        });

        assert_eq!(
            components.get("minecraft:custom_model_data"),
            Some(&NbtValue::Float(12.0))
        );
        assert_eq!(report.loss_count(), 1);
    }

    #[test]
    fn v4181_reverse_reports_mismatched_furnace_lit_total_time() {
        let mut furnace = NbtMap::new();
        furnace.set_string("id", "minecraft:furnace");
        furnace.set_i32("lit_time_remaining", 20);
        furnace.set_i32("lit_total_time", 40);

        let report = convert_block_entity_reverse(&mut furnace, 4181, 4180);

        assert_eq!(furnace.get_i32("BurnTime"), Some(20));
        assert_eq!(report.loss_count(), 1);
        assert!(report.summary().contains("lit_total_time differed"));
    }

    #[test]
    fn v4187_reverse_reports_follow_range_default_ambiguity() {
        let mut attribute = NbtMap::new();
        attribute.set_string("id", "minecraft:follow_range");
        attribute.set_f64("base", 16.0);
        let mut villager = NbtMap::new();
        villager.set_string("id", "minecraft:villager");
        villager.set_list("attributes", vec![NbtValue::Compound(attribute)]);

        let report = convert_entity_reverse(&mut villager, 4187, 4186);

        let attribute = villager.get_list("attributes").unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(attribute.get_f64("base"), Some(48.0));
        assert_eq!(report.len(), 1);
        assert!(report.summary().contains("V4187 default rewrite"));
    }

    #[test]
    fn v4059_reverse_reports_noncompound_consume_effect() {
        use super::loss;
        use super::registry::convert_reverse_under_session;

        let mut consumable = NbtMap::new();
        consumable.set_list(
            "on_consume_effects",
            vec![NbtValue::String("unsupported".to_string())],
        );
        let mut components = NbtMap::new();
        components.set_map("minecraft:consumable", consumable);

        let reg = registry();
        let (_, report) = loss::run_reverse(|| {
            convert_reverse_under_session(&reg.data_components, &mut components, 4059, 4058)
        });

        assert_eq!(report.loss_count(), 1);
        assert!(report.summary().contains("non-compound"));
    }

    fn get_step_max() -> i32 {
        i32::MAX
    }

    #[test]
    fn v135_riding_chain_round_trips_depth_3() {
        // A rides B rides C. Forward collapses to nested Passengers (C root);
        // reverse must restore the exact 3-deep Riding chain (the case the pilot
        // verifier caught as broken).
        fn ent(id: &str) -> NbtMap {
            let mut m = NbtMap::new();
            m.set_string("id", id);
            m
        }
        let mut c = ent("C");
        let mut b = ent("B");
        b.set_map("Riding", c);
        let mut a = ent("A");
        a.set_map("Riding", b);

        convert_entity(&mut a, 134, 135);
        // Now `a` is the topmost vehicle C, with A nested two levels down.
        assert_eq!(a.get_string("id"), Some("C"));
        assert!(
            a.get_map("Riding").is_none(),
            "Riding replaced by Passengers"
        );
        let b2 = a.get_list("Passengers").unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(b2.get_string("id"), Some("B"));
        let a2 = b2.get_list("Passengers").unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(a2.get_string("id"), Some("A"));

        let report = convert_entity_reverse(&mut a, 135, 134);
        // Back to A{Riding: B{Riding: C}}.
        assert_eq!(a.get_string("id"), Some("A"));
        assert!(a.get_list("Passengers").is_none(), "Passengers gone");
        let b3 = a.get_map("Riding").expect("A rides B");
        assert_eq!(b3.get_string("id"), Some("B"));
        let c3 = b3.get_map("Riding").expect("B rides C");
        assert_eq!(c3.get_string("id"), Some("C"));
        assert!(c3.get_map("Riding").is_none());
        assert!(report.is_empty(), "single-rider chain is lossless");
    }
}
