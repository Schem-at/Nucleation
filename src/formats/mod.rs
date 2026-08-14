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
