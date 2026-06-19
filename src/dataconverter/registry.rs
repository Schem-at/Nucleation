//! The type registry: holds every schematic-relevant data type, the breakpoint
//! list, and the public top-level conversion entry points.
//!
//! Ported from `datatypes/MCTypeRegistry.java` and `MCVersionRegistry.java`,
//! restricted to the types that actually appear in a schematic file. Non-
//! schematic types (LEVEL, CHUNK, PLAYER, POI, STATS, ADVANCEMENTS, …) are
//! intentionally omitted — they never occur in a schematic, so version-file
//! registrations targeting them are skipped during porting.

use std::sync::LazyLock;

use crate::nbt::NbtMap;

use super::engine::{
    encode_endpoints, walk_with_breakpoints, walk_with_breakpoints_reverse, DataType, MCValueType,
};
use super::loss::{self, LossReport};
use super::version::{encode_versions, EncodedVersion, MAX_STEP};
use super::versions;

/// All schematic-relevant types plus the breakpoint list. Built once (mutably,
/// during registration), then immutable at convert time.
pub struct Registry {
    // Compound types (MCDataType / IDDataType).
    pub block_state: DataType,
    pub tile_entity: DataType,
    pub item_stack: DataType,
    pub entity: DataType,
    pub data_components: DataType,
    pub villager_trade: DataType,
    pub structure: DataType,
    pub untagged_spawner: DataType,
    pub entity_equipment: DataType,
    pub text_component: DataType,
    pub particle: DataType,

    // Leaf value types (MCValueType).
    pub block_name: MCValueType,
    pub flat_block_state: MCValueType,
    pub item_name: MCValueType,
    pub entity_name: MCValueType,
    pub biome: MCValueType,
    pub game_event_name: MCValueType,

    pub breakpoints: Vec<EncodedVersion>,
}

/// Alias used by registration code (the registry is mutated only at build time).
pub type RegistryBuilder = Registry;

impl Registry {
    fn empty() -> Self {
        Self {
            block_state: DataType::new("BlockState"),
            tile_entity: DataType::new("TileEntity"),
            item_stack: DataType::new("ItemStack"),
            entity: DataType::new("Entity"),
            data_components: DataType::new("DataComponents"),
            villager_trade: DataType::new("VillagerTrade"),
            structure: DataType::new("Structure"),
            untagged_spawner: DataType::new("Spawner"),
            entity_equipment: DataType::new("EntityEquipment"),
            text_component: DataType::new("TextComponent"),
            particle: DataType::new("Particle"),
            block_name: MCValueType::new("BlockName"),
            flat_block_state: MCValueType::new("FlatBlockState"),
            item_name: MCValueType::new("ItemName"),
            entity_name: MCValueType::new("EntityName"),
            biome: MCValueType::new("Biome"),
            game_event_name: MCValueType::new("GameEventName"),
            breakpoints: Vec::new(),
        }
    }

    fn finalize(&mut self) {
        self.block_state.finalize();
        self.tile_entity.finalize();
        self.item_stack.finalize();
        self.entity.finalize();
        self.data_components.finalize();
        self.villager_trade.finalize();
        self.structure.finalize();
        self.untagged_spawner.finalize();
        self.entity_equipment.finalize();
        self.text_component.finalize();
        self.particle.finalize();
        self.block_name.finalize();
        self.flat_block_state.finalize();
        self.item_name.finalize();
        self.entity_name.finalize();
        self.biome.finalize();
        self.game_event_name.finalize();
        self.breakpoints.sort_unstable();
        self.breakpoints.dedup();
    }
}

/// The breakpoint list (MCVersionRegistry.java:330-355). Partitions the chain
/// into atomically-applied segments.
fn register_breakpoints(reg: &mut Registry) {
    let after = |v: i32, step: i32| encode_versions(v, step) + 1;
    let at = |v: i32, step: i32| encode_versions(v, step);
    reg.breakpoints.extend([
        at(1451, 0),          // V17W47A — the Flattening
        after(1451, MAX_STEP),
        after(2730, MAX_STEP), // 1.17.1
        after(2975, MAX_STEP), // 1.18.2
        after(3337, MAX_STEP), // 1.19.4
        at(3818, 5),           // V24W07A+1 step 5 — the 1.20.5 component split
        after(3818, MAX_STEP),
        after(3839, MAX_STEP), // 1.20.6
        after(4290, 0),        // V4290 text-component sub-reads
        after(4671, MAX_STEP), // 1.21.11
    ]);
}

/// Build and finalize the full registry.
pub fn build() -> Registry {
    let mut reg = Registry::empty();
    register_breakpoints(&mut reg);
    versions::register_all(&mut reg);
    reg.finalize();
    reg
}

/// The process-wide registry, built once on first use.
pub fn registry() -> &'static Registry {
    static REGISTRY: LazyLock<Registry> = LazyLock::new(build);
    &REGISTRY
}

// --- public top-level conversion entry points -------------------------------

macro_rules! define_convert {
    ($name:ident, $field:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(data: &mut NbtMap, from_data_version: i32, to_data_version: i32) {
            convert_with(|reg| &reg.$field, data, from_data_version, to_data_version)
        }
    };
}

fn convert_with(
    pick: impl Fn(&'static Registry) -> &'static DataType,
    data: &mut NbtMap,
    from_dv: i32,
    to_dv: i32,
) {
    let reg = registry();
    let ty = pick(reg);
    let (from, to) = encode_endpoints(from_dv, to_dv);
    walk_with_breakpoints(&reg.breakpoints, from, to, |seg_from, seg_to| {
        ty.convert(reg, data, seg_from, seg_to);
    });
}

define_convert!(convert_block_state, block_state, "Convert one palette BLOCK_STATE map.");
define_convert!(convert_block_entity, tile_entity, "Convert one block-entity (TILE_ENTITY) map.");
define_convert!(convert_item_stack, item_stack, "Convert one ITEM_STACK map.");
define_convert!(convert_entity, entity, "Convert one ENTITY map.");
define_convert!(convert_structure, structure, "Convert a whole STRUCTURE root (entities/blocks/palette).");

// --- reverse (new -> old) entry points --------------------------------------

/// Reverse-convert `data` of `ty` from the newer `from_dv` down to the older
/// `to_dv`, returning the accumulated [`LossReport`]. Runs the *same* walker
/// topology in reverse: walker descent first, inverse converters in descending
/// version order, breakpoint segments traversed high -> low and applied
/// atomically. The thread-local direction (set by [`loss::run_reverse`]) makes
/// every nested `convert` call take the reverse path.
pub(crate) fn convert_reverse_under_session(
    ty: &'static DataType,
    data: &mut NbtMap,
    from_dv: i32,
    to_dv: i32,
) {
    let reg = registry();
    let (from, to) = encode_endpoints(from_dv, to_dv);
    walk_with_breakpoints_reverse(&reg.breakpoints, from, to, |seg_from, seg_to| {
        ty.convert(reg, data, seg_from, seg_to);
    });
}

macro_rules! define_convert_reverse {
    ($name:ident, $field:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(data: &mut NbtMap, from_data_version: i32, to_data_version: i32) -> LossReport {
            let reg = registry();
            let (_, report) = loss::run_reverse(|| {
                convert_reverse_under_session(&reg.$field, data, from_data_version, to_data_version)
            });
            report
        }
    };
}

define_convert_reverse!(convert_block_state_reverse, block_state, "Reverse-convert one palette BLOCK_STATE map (new -> old).");
define_convert_reverse!(convert_block_entity_reverse, tile_entity, "Reverse-convert one block-entity (TILE_ENTITY) map (new -> old).");
define_convert_reverse!(convert_item_stack_reverse, item_stack, "Reverse-convert one ITEM_STACK map (new -> old).");
define_convert_reverse!(convert_entity_reverse, entity, "Reverse-convert one ENTITY map (new -> old).");
define_convert_reverse!(convert_structure_reverse, structure, "Reverse-convert a whole STRUCTURE root (new -> old).");
