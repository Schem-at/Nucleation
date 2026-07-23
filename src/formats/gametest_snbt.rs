use crate::block_entity::BlockEntity;
use crate::block_position::BlockPosition;
use crate::entity::Entity;
use crate::formats::error::{FormatError, Result};
use crate::formats::manager::{SchematicExporter, SchematicImporter};
use crate::utils::NbtMap;
use crate::{BlockState, UniversalSchematic};
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use std::collections::BTreeSet;

/// GameTest structures are small, rectangular test fixtures. The vanilla
/// Structure Block limit is 48 blocks per axis; these deliberately generous
/// limits retain headroom for generated suites while bounding allocations from
/// untrusted SNBT and sparse multi-region exports.
pub const MAX_GAMETEST_DIMENSION: i32 = 256;
pub const MAX_GAMETEST_VOLUME: usize = 262_144;

pub struct GameTestSnbtFormat;

impl SchematicImporter for GameTestSnbtFormat {
    fn name(&self) -> String {
        "gametest_snbt".to_string()
    }

    fn detect(&self, data: &[u8]) -> bool {
        is_gametest_snbt(data)
    }

    fn read(&self, data: &[u8]) -> Result<UniversalSchematic> {
        from_gametest_snbt(data)
    }
}

impl SchematicExporter for GameTestSnbtFormat {
    fn name(&self) -> String {
        "gametest_snbt".to_string()
    }

    fn extensions(&self) -> Vec<String> {
        vec!["snbt".to_string()]
    }

    fn available_versions(&self) -> Vec<String> {
        vec!["latest".to_string()]
    }

    fn default_version(&self) -> String {
        "latest".to_string()
    }

    fn write(&self, schematic: &UniversalSchematic, version: Option<&str>) -> Result<Vec<u8>> {
        if !matches!(version, None | Some("latest")) {
            return Err(format!(
                "unsupported GameTest SNBT version {:?}; expected latest",
                version.unwrap_or_default()
            )
            .into());
        }
        to_gametest_snbt(schematic)
    }
}

pub fn to_gametest_snbt(schematic: &UniversalSchematic) -> Result<Vec<u8>> {
    let bounds = schematic.get_bounding_box();
    let min = bounds.min;
    let size = checked_bounds_dimensions(bounds.min, bounds.max)?;
    let volume = checked_structure_volume(size)?;
    let mut palette = BTreeSet::new();
    let mut data_entries = Vec::new();
    data_entries.try_reserve_exact(volume).map_err(|error| {
        FormatError::Parse(format!(
            "cannot allocate GameTest structure with {volume} blocks: {error}"
        ))
    })?;
    for y in min.1..=bounds.max.1 {
        for z in min.2..=bounds.max.2 {
            for x in min.0..=bounds.max.0 {
                let state = schematic
                    .get_block(x, y, z)
                    .cloned()
                    .unwrap_or_else(|| BlockState::new("minecraft:air"));
                let state_string = format_gametest_block_state(&state);
                palette.insert(state_string.clone());

                let mut entry = NbtCompound::new();
                entry.insert("pos", int_list((x - min.0, y - min.1, z - min.2)));
                entry.insert("state", NbtTag::String(state_string));
                if let Some(block_entity) =
                    schematic.get_block_entity_owned(BlockPosition { x, y, z })
                {
                    let mut nbt = block_entity.nbt.to_quartz_nbt();
                    if !nbt.contains_key("id") && !nbt.contains_key("Id") {
                        nbt.insert("id", NbtTag::String(block_entity.id));
                    }
                    entry.insert("nbt", NbtTag::Compound(nbt));
                }
                data_entries.push(NbtTag::Compound(entry));
            }
        }
    }

    let mut entity_entries = Vec::new();
    for entity in schematic.get_entities_as_list() {
        let relative = (
            entity.position.0 - f64::from(min.0),
            entity.position.1 - f64::from(min.1),
            entity.position.2 - f64::from(min.2),
        );
        let mut entry = NbtCompound::new();
        entry.insert(
            "blockPos",
            int_list((
                relative.0.floor() as i32,
                relative.1.floor() as i32,
                relative.2.floor() as i32,
            )),
        );
        entry.insert("pos", double_list(relative));
        entry.insert("nbt", entity.to_nbt());
        entity_entries.push(NbtTag::Compound(entry));
    }

    let data_version = schematic
        .metadata
        .mc_version
        .or(schematic.metadata.source_data_version)
        .unwrap_or(crate::dataconverter::CANONICAL_DATA_VERSION);
    let mut root = NbtCompound::new();
    root.insert("DataVersion", NbtTag::Int(data_version));
    root.insert("size", int_list(size));
    root.insert("data", NbtTag::List(NbtList::from(data_entries)));
    root.insert("entities", NbtTag::List(NbtList::from(entity_entries)));
    root.insert(
        "palette",
        NbtTag::List(NbtList::from(
            palette.into_iter().map(NbtTag::String).collect::<Vec<_>>(),
        )),
    );

    Ok(NbtTag::Compound(root).to_snbt().into_bytes())
}

fn int_list(value: (i32, i32, i32)) -> NbtTag {
    NbtTag::List(NbtList::from(vec![
        NbtTag::Int(value.0),
        NbtTag::Int(value.1),
        NbtTag::Int(value.2),
    ]))
}

fn double_list(value: (f64, f64, f64)) -> NbtTag {
    NbtTag::List(NbtList::from(vec![
        NbtTag::Double(value.0),
        NbtTag::Double(value.1),
        NbtTag::Double(value.2),
    ]))
}

fn format_gametest_block_state(state: &BlockState) -> String {
    let mut output = state.name.to_string();
    if !state.properties.is_empty() {
        output.push('{');
        for (index, (key, value)) in state.properties.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push_str(key);
            output.push(':');
            output.push_str(value);
        }
        output.push('}');
    }
    output
}

pub fn is_gametest_snbt(data: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(data) else {
        return false;
    };
    let Ok(root) = quartz_nbt::snbt::parse(text) else {
        return false;
    };
    matches!(root.inner().get("DataVersion"), Some(NbtTag::Int(_)))
        && matches!(root.inner().get("size"), Some(NbtTag::List(_)))
        && matches!(root.inner().get("data"), Some(NbtTag::List(_)))
        && matches!(root.inner().get("entities"), Some(NbtTag::List(_)))
        && matches!(root.inner().get("palette"), Some(NbtTag::List(_)))
}

pub fn from_gametest_snbt(data: &[u8]) -> Result<UniversalSchematic> {
    let text = std::str::from_utf8(data)
        .map_err(|error| FormatError::Parse(format!("GameTest SNBT is not UTF-8: {error}")))?;
    let root = quartz_nbt::snbt::parse(text)
        .map_err(|error| FormatError::Parse(format!("invalid GameTest SNBT: {error}")))?;

    let data_version = root.get::<_, i32>("DataVersion")?;
    let size = int_vec3(root.get::<_, &NbtList>("size")?, "size")?;
    checked_structure_volume(size)?;

    let mut schematic = UniversalSchematic::new("GameTest".to_string());
    schematic.metadata.mc_version = Some(data_version);
    schematic.metadata.source_data_version = Some(data_version);
    schematic.default_region =
        crate::region::Region::new(schematic.default_region_name.clone(), (0, 0, 0), size);

    let entries = root.get::<_, &NbtList>("data")?;
    for (index, entry) in entries.iter().enumerate() {
        let NbtTag::Compound(entry) = entry else {
            return Err(format!("GameTest SNBT data[{index}] must be a compound").into());
        };
        import_block_entry(&mut schematic, entry, index, size)?;
    }

    let entities = root.get::<_, &NbtList>("entities")?;
    for (index, entry) in entities.iter().enumerate() {
        let NbtTag::Compound(entry) = entry else {
            return Err(format!("GameTest SNBT entities[{index}] must be a compound").into());
        };
        import_entity_entry(&mut schematic, entry, index)?;
    }

    Ok(schematic)
}

fn checked_structure_volume(size: (i32, i32, i32)) -> Result<usize> {
    if size.0 <= 0 || size.1 <= 0 || size.2 <= 0 {
        return Err(format!("GameTest SNBT size must be positive, got {size:?}").into());
    }
    if size.0 > MAX_GAMETEST_DIMENSION
        || size.1 > MAX_GAMETEST_DIMENSION
        || size.2 > MAX_GAMETEST_DIMENSION
    {
        return Err(format!(
            "GameTest structure size {size:?} exceeds the {MAX_GAMETEST_DIMENSION}-block axis limit"
        )
        .into());
    }

    let volume = usize::try_from(size.0)
        .ok()
        .and_then(|x| usize::try_from(size.1).ok().and_then(|y| x.checked_mul(y)))
        .and_then(|xy| usize::try_from(size.2).ok().and_then(|z| xy.checked_mul(z)))
        .ok_or_else(|| {
            FormatError::Parse(format!("GameTest structure size is too large: {size:?}"))
        })?;
    if volume > MAX_GAMETEST_VOLUME {
        return Err(format!(
            "GameTest structure volume {volume} exceeds the {MAX_GAMETEST_VOLUME}-block limit"
        )
        .into());
    }
    Ok(volume)
}

fn checked_bounds_dimensions(
    min: (i32, i32, i32),
    max: (i32, i32, i32),
) -> Result<(i32, i32, i32)> {
    fn axis(min: i32, max: i32, name: &str) -> Result<i32> {
        let dimension = i64::from(max) - i64::from(min) + 1;
        i32::try_from(dimension).map_err(|_| {
            FormatError::Parse(format!(
                "GameTest {name} dimension {dimension} exceeds the supported coordinate range"
            ))
        })
    }

    Ok((
        axis(min.0, max.0, "X")?,
        axis(min.1, max.1, "Y")?,
        axis(min.2, max.2, "Z")?,
    ))
}

fn import_entity_entry(
    schematic: &mut UniversalSchematic,
    entry: &NbtCompound,
    index: usize,
) -> Result<()> {
    let pos = double_vec3(entry.get::<_, &NbtList>("pos")?, "entity pos")?;
    let mut nbt = entry.get::<_, &NbtCompound>("nbt")?.clone();
    // Vanilla's structure loader ignores entity records whose payload has no
    // type id. GameTest files can contain these empty placeholder records.
    if !nbt.contains_key("id") && !nbt.contains_key("Id") {
        return Ok(());
    }
    nbt.insert(
        "Pos",
        NbtTag::List(NbtList::from(vec![
            NbtTag::Double(pos.0),
            NbtTag::Double(pos.1),
            NbtTag::Double(pos.2),
        ])),
    );
    let entity = Entity::from_nbt(&nbt).map_err(|error| {
        FormatError::Parse(format!(
            "invalid entity NBT in GameTest SNBT entities[{index}]: {error}"
        ))
    })?;
    schematic.add_entity(entity);
    Ok(())
}

fn import_block_entry(
    schematic: &mut UniversalSchematic,
    entry: &NbtCompound,
    index: usize,
    size: (i32, i32, i32),
) -> Result<()> {
    let pos = int_vec3(entry.get::<_, &NbtList>("pos")?, "block pos")?;
    if pos.0 < 0 || pos.1 < 0 || pos.2 < 0 || pos.0 >= size.0 || pos.1 >= size.1 || pos.2 >= size.2
    {
        return Err(
            format!("GameTest SNBT data[{index}] position {pos:?} is outside {size:?}").into(),
        );
    }
    let state = entry.get::<_, &str>("state")?;
    let state = parse_gametest_block_state(state).map_err(|error| {
        FormatError::Parse(format!(
            "invalid state in GameTest SNBT data[{index}]: {error}"
        ))
    })?;
    schematic.set_block(pos.0, pos.1, pos.2, &state);

    if let Ok(nbt) = entry.get::<_, &NbtCompound>("nbt") {
        let nbt = NbtMap::from_quartz_nbt(nbt);
        let id = nbt
            .get("id")
            .or_else(|| nbt.get("Id"))
            .and_then(|value| value.as_string())
            .cloned()
            .unwrap_or_else(|| state.get_name().to_string());
        let mut block_entity = BlockEntity::new(id, pos);
        block_entity.set_nbt(nbt);
        schematic.set_block_entity(
            BlockPosition {
                x: pos.0,
                y: pos.1,
                z: pos.2,
            },
            block_entity,
        );
    }
    Ok(())
}

fn int_vec3(list: &NbtList, field: &str) -> Result<(i32, i32, i32)> {
    if list.len() != 3 {
        return Err(format!("GameTest SNBT {field} must contain exactly 3 integers").into());
    }
    Ok((
        list.get::<i32>(0)?,
        list.get::<i32>(1)?,
        list.get::<i32>(2)?,
    ))
}

fn double_vec3(list: &NbtList, field: &str) -> Result<(f64, f64, f64)> {
    if list.len() != 3 {
        return Err(format!("GameTest SNBT {field} must contain exactly 3 numbers").into());
    }
    Ok((
        list.get::<f64>(0)?,
        list.get::<f64>(1)?,
        list.get::<f64>(2)?,
    ))
}

fn parse_gametest_block_state(value: &str) -> std::result::Result<BlockState, String> {
    let Some(open) = value.find('{') else {
        return BlockState::from_block_string(value);
    };
    if !value.ends_with('}') {
        return Err(format!("unterminated properties in {value:?}"));
    }
    let name = value[..open].trim();
    if name.is_empty() {
        return Err("empty block name".to_string());
    }
    let mut state = BlockState::new(name);
    let properties = &value[open + 1..value.len() - 1];
    for property in properties.split(',').filter(|part| !part.is_empty()) {
        let (key, value) = property
            .split_once(':')
            .ok_or_else(|| format!("property {property:?} missing ':'"))?;
        if key.is_empty() || value.is_empty() {
            return Err(format!("invalid property {property:?}"));
        }
        state.set_property(key, value);
    }
    Ok(state)
}
