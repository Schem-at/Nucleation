/// Version of the root-level `NucleationTest {Format, Spec}` compound that
/// carries a build's embedded test. Written identically by every format whose
/// writer preserves the tag (`litematic`, `schematic`).
pub(crate) const NUCLEATION_TEST_FORMAT: i32 = 1;

pub mod anvil;
pub mod classic_schematic;
pub mod error;
pub mod gametest;
pub mod limits;
pub mod litematic;
pub mod manager;
pub mod mcstructure;
pub mod schematic;
pub mod snapshot;
pub mod structure_snbt;
pub mod world;
#[cfg(not(target_arch = "wasm32"))]
pub mod world_pack;
pub mod world_stream;

/// Write `root` as uncompressed NBT through a gzip encoder.
///
/// The NBT writer issues one small write per tag; buffering them in front of
/// the encoder hands deflate large blocks instead of per-tag calls.
pub(crate) fn gzip_nbt(
    root: &quartz_nbt::NbtCompound,
    compression: flate2::Compression,
) -> error::Result<Vec<u8>> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), compression);
    let mut writer = std::io::BufWriter::with_capacity(1 << 16, encoder);
    quartz_nbt::io::write_nbt(&mut writer, None, root, quartz_nbt::io::Flavor::Uncompressed)?;
    // `into_inner` writes out the buffer without flushing the encoder (a
    // gzip flush would emit an extra sync block into the stream).
    let encoder = writer
        .into_inner()
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    Ok(encoder.finish()?)
}
