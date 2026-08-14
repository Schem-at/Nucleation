//! Allocation and decompression limits for untrusted schematic inputs.

use crate::formats::error::{FormatError, Result};
use crate::nbt::io::{read_nbt_with_limits, NbtReadLimits};
use crate::nbt::{Endian, NbtValue};
use crate::UniversalSchematic;
use flate2::read::GzDecoder;
use quartz_nbt::NbtCompound;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeLimits {
    pub max_input_bytes: usize,
    pub max_decompressed_bytes: usize,
    pub max_dimension: usize,
    pub max_volume: usize,
    pub max_regions: usize,
    pub max_palette_entries: usize,
    pub max_entities: usize,
    pub max_block_entities: usize,
    pub max_nbt_depth: usize,
    pub max_nbt_string_bytes: usize,
    pub max_nbt_collection_items: usize,
    pub max_nbt_nodes: usize,
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 256 * 1024 * 1024,
            max_decompressed_bytes: 1024 * 1024 * 1024,
            max_dimension: 16_384,
            max_volume: 536_870_912,
            max_regions: 16_384,
            max_palette_entries: 1_048_576,
            max_entities: 1_000_000,
            max_block_entities: 16_000_000,
            max_nbt_depth: 64,
            max_nbt_string_bytes: 1024 * 1024,
            max_nbt_collection_items: 536_870_912,
            max_nbt_nodes: 64_000_000,
        }
    }
}

impl DecodeLimits {
    pub fn validate(&self) -> Result<()> {
        if self.max_input_bytes == 0
            || self.max_decompressed_bytes == 0
            || self.max_dimension == 0
            || self.max_volume == 0
            || self.max_regions == 0
            || self.max_nbt_depth == 0
            || self.max_nbt_string_bytes == 0
            || self.max_nbt_collection_items == 0
            || self.max_nbt_nodes == 0
        {
            return Err(FormatError::Parse(
                "decode limits must be greater than zero".into(),
            ));
        }
        Ok(())
    }

    pub fn check_input(&self, data: &[u8]) -> Result<()> {
        self.validate()?;
        if data.len() > self.max_input_bytes {
            return Err(FormatError::Parse(format!(
                "input byte limit exceeded: {} > {}",
                data.len(),
                self.max_input_bytes
            )));
        }
        Ok(())
    }

    pub fn check_dimensions(&self, dimensions: (i64, i64, i64)) -> Result<usize> {
        let (x, y, z) = dimensions;
        if x <= 0 || y <= 0 || z <= 0 {
            return Err(FormatError::Parse("dimensions must be positive".into()));
        }
        let dims = [x, y, z].map(|value| {
            usize::try_from(value)
                .map_err(|_| FormatError::Parse("dimension is not addressable".into()))
        });
        let [x, y, z] = dims;
        let (x, y, z) = (x?, y?, z?);
        if x > self.max_dimension || y > self.max_dimension || z > self.max_dimension {
            return Err(FormatError::Parse("dimension limit exceeded".into()));
        }
        let volume = x
            .checked_mul(y)
            .and_then(|value| value.checked_mul(z))
            .ok_or_else(|| FormatError::Parse("volume overflow".into()))?;
        if volume > self.max_volume {
            return Err(FormatError::Parse("volume limit exceeded".into()));
        }
        Ok(volume)
    }

    pub fn nbt(&self) -> NbtReadLimits {
        NbtReadLimits {
            max_depth: self.max_nbt_depth,
            max_string_bytes: self.max_nbt_string_bytes,
            max_collection_items: self.max_nbt_collection_items,
            max_nodes: self.max_nbt_nodes,
        }
    }

    pub fn validate_schematic(&self, schematic: &UniversalSchematic) -> Result<()> {
        let regions = 1usize.saturating_add(schematic.other_regions.len());
        if regions > self.max_regions {
            return Err(FormatError::Parse("region limit exceeded".into()));
        }
        let mut total_volume = 0usize;
        let mut entities = 0usize;
        let mut block_entities = 0usize;
        for region in
            std::iter::once(&schematic.default_region).chain(schematic.other_regions.values())
        {
            let volume = self.check_dimensions((
                i64::from(region.size.0),
                i64::from(region.size.1),
                i64::from(region.size.2),
            ))?;
            total_volume = total_volume
                .checked_add(volume)
                .ok_or_else(|| FormatError::Parse("total volume overflow".into()))?;
            if total_volume > self.max_volume {
                return Err(FormatError::Parse("total volume limit exceeded".into()));
            }
            if region.palette_len() > self.max_palette_entries {
                return Err(FormatError::Parse("palette limit exceeded".into()));
            }
            entities = entities.saturating_add(region.entities.len());
            block_entities = block_entities.saturating_add(region.block_entities.len());
        }
        if entities > self.max_entities {
            return Err(FormatError::Parse("entity limit exceeded".into()));
        }
        if block_entities > self.max_block_entities {
            return Err(FormatError::Parse("block-entity limit exceeded".into()));
        }
        Ok(())
    }
}

pub(crate) fn parse_gzip_nbt(data: &[u8], limits: &DecodeLimits) -> Result<NbtCompound> {
    limits.check_input(data)?;
    let mut decoder = GzDecoder::new(data);
    let mut raw = Vec::new();
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let read = decoder.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        let next = raw
            .len()
            .checked_add(read)
            .ok_or_else(|| FormatError::Parse("decompressed size overflow".into()))?;
        if next > limits.max_decompressed_bytes {
            return Err(FormatError::Parse(
                "decompressed byte limit exceeded".into(),
            ));
        }
        raw.try_reserve(read)
            .map_err(|error| FormatError::Parse(error.to_string()))?;
        raw.extend_from_slice(&chunk[..read]);
    }
    parse_raw_nbt(&raw, Endian::Big, limits)
}

pub(crate) fn parse_raw_nbt(
    data: &[u8],
    endian: Endian,
    limits: &DecodeLimits,
) -> Result<NbtCompound> {
    limits.check_input(data)?;
    let root = read_nbt_with_limits(&mut Cursor::new(data), endian, limits.nbt())?;
    match root {
        NbtValue::Compound(value) => Ok(value.to_quartz_nbt()),
        _ => Err(FormatError::Parse("root NBT is not a compound".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::manager::get_manager;
    use crate::nbt::{NbtMap, NbtValue};
    use crate::BlockState;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    fn gzip_nbt(map: &NbtMap) -> Vec<u8> {
        let mut raw = Vec::new();
        crate::nbt::io::write_nbt(&mut raw, map, "", Endian::Big).unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&raw).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn gzip_and_string_limits_fail_before_unbounded_growth() {
        let mut root = NbtMap::new();
        root.insert("message".into(), NbtValue::String("hello".into()));
        let data = gzip_nbt(&root);

        let mut limits = DecodeLimits::default();
        limits.max_decompressed_bytes = 8;
        assert!(parse_gzip_nbt(&data, &limits).is_err());

        limits.max_decompressed_bytes = 1024;
        limits.max_nbt_string_bytes = 4;
        assert!(parse_gzip_nbt(&data, &limits).is_err());
    }

    #[test]
    fn manager_rejects_volume_before_litematic_region_allocation() {
        let mut schematic = UniversalSchematic::new("bounded".into());
        schematic.set_block(0, 0, 0, &BlockState::new("minecraft:stone"));
        schematic.set_block(1, 0, 0, &BlockState::new("minecraft:stone"));
        let data = crate::formats::litematic::to_litematic(&schematic).unwrap();
        let mut limits = DecodeLimits::default();
        limits.max_volume = 1;
        let manager = get_manager();
        assert!(manager
            .lock()
            .unwrap()
            .read_bounded(&data, &limits)
            .is_err());
    }

    #[test]
    fn dimensions_use_checked_arithmetic() {
        let limits = DecodeLimits::default();
        assert!(limits.check_dimensions((-1, 1, 1)).is_err());
        assert!(limits.check_dimensions((i64::MAX, 2, 2)).is_err());
    }
}
