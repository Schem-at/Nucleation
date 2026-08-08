//! HDL -> redstone: the `hdl` feature's bridge surface.
//!
//! Thin wrapper over `nucleation-hdl` (crates/nucleation-hdl), the Rust port
//! of the verified `redstone-eda/hdl` Python pipeline: a combinational BLIF
//! (yosys `synth -lut 4; write_blif` output, or hand-written) compiles to a
//! placed, probed, lever-driven dual-rail PLA build.
//!
//! Same one-way rule as mc-tick and routing: `nucleation-hdl` never sees
//! nucleation; this module converts its cell map into a [`crate::UniversalSchematic`].
//! Structured results cross as JSON strings (PORTING.md rule 9).
//!
//! Bindings are regenerated (`tools/gen-bindings.sh`); the module compiles
//! under `--features bridge,hdl` (bake additionally wants `mc-tick`).

/// Author the compiled cells into a fresh schematic.
fn to_schematic(
    build: &nucleation_hdl::Build,
    name: &str,
) -> Result<crate::UniversalSchematic, String> {
    let mut schem = crate::UniversalSchematic::new(name.to_string());
    for (&(x, y, z), block) in &build.cells {
        schem.set_block_from_string(x, y, z, block)?;
    }
    Ok(schem)
}

/// Settle the build in the tick engine (levers at rest) and write every
/// settled state back — the compiled circuit saved "at rest".
#[cfg(feature = "mc-tick")]
fn bake_build(build: &mut nucleation_hdl::Build) -> Result<usize, String> {
    let sim = nucleation_hdl::verify::simulate(build, 4000)?;
    Ok(nucleation_hdl::verify::bake(build, &sim))
}

#[cfg(not(feature = "mc-tick"))]
fn bake_build(_build: &mut nucleation_hdl::Build) -> Result<usize, String> {
    Err("bake=true needs a simulator: rebuild with the `mc-tick` feature".to_string())
}

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Namespacing opaque for the HDL compiler entry points (static methods,
    /// like `Routing`).
    #[diplomat::opaque]
    pub struct Hdl;

    impl Hdl {
        /// Compile combinational BLIF text into a redstone PLA schematic.
        ///
        /// `blif` is yosys `write_blif` output (`.latch`/`.subckt` are
        /// rejected — combinational only). One floor lever per `.inputs` net
        /// drives the build; every signal has a dust probe. `bake=true`
        /// settles the build in the tick engine first and saves it at rest
        /// (needs the `mc-tick` feature, else errors).
        ///
        /// Probe/lever coordinates and stats come from `compile_blif_report`.
        pub fn compile_blif(
            blif: &DiplomatStr,
            name: &DiplomatStr,
            bake: bool,
        ) -> Result<Box<Schematic>, NucleationError> {
            let (blif, name) = decode(blif, name)?;
            let compiled = nucleation_hdl::compile_blif(blif, name).map_err(|e| {
                crate::bridge::set_last_error_detail(e.to_string());
                NucleationError::InvalidArgument
            })?;
            let mut build = compiled.build;
            if bake {
                super::bake_build(&mut build).map_err(|e| {
                    crate::bridge::set_last_error_detail(e);
                    NucleationError::Simulation
                })?;
            }
            let schem = super::to_schematic(&build, name).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::InvalidArgument
            })?;
            Ok(Box::new(Schematic(schem)))
        }

        /// Compile `blif` and write the JSON report: stats (`prims`,
        /// `levels`, `peephole_removed`, `blocks`, `bounds`), `inputs` (=
        /// lever drive order), `outputs` (each `{name, probe}` or `{name,
        /// const}`), `levers` (`{signal, pos}`), and `probes`
        /// (signal -> `[x, y, z]` dust cell, in the schematic's own
        /// coordinates).
        pub fn compile_blif_report(
            blif: &DiplomatStr,
            name: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let (blif, name) = decode(blif, name)?;
            let compiled = nucleation_hdl::compile_blif(blif, name).map_err(|e| {
                crate::bridge::set_last_error_detail(e.to_string());
                NucleationError::InvalidArgument
            })?;
            let _ = write!(out, "{}", compiled.report_json());
            Ok(())
        }
    }

    fn decode<'a>(
        blif: &'a DiplomatStr,
        name: &'a DiplomatStr,
    ) -> Result<(&'a str, &'a str), NucleationError> {
        let blif = core::str::from_utf8(blif).map_err(|_| NucleationError::InvalidArgument)?;
        let name = core::str::from_utf8(name).map_err(|_| NucleationError::InvalidArgument)?;
        Ok((blif, name))
    }
}

#[cfg(test)]
mod tests {
    /// The bridge path end-to-end: a 2-input BLIF compiles to a schematic
    /// whose cells match the compiler's build, and the report carries the
    /// lever/probe metadata a driver needs.
    #[test]
    fn a_blif_compiles_to_a_schematic_with_report() {
        let blif = ".model xor2\n.inputs a b\n.outputs y\n.names a b y\n01 1\n10 1\n.end\n";
        let compiled = nucleation_hdl::compile_blif(blif, "xor2").unwrap();
        let schem = super::to_schematic(&compiled.build, "xor2").unwrap();
        let placed = schem
            .iter_blocks()
            .filter(|(_, bs)| bs.name.as_str() != "minecraft:air")
            .count();
        assert_eq!(placed, compiled.build.cells.len());
        let report = compiled.report_json();
        assert!(report.contains("\"levers\""), "{report}");
        assert!(report.contains("\"probe\""), "{report}");
    }
}
