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

/// The compiled design's typed-cell contract, parsed into the shared
/// [`crate::io_contract::CellContract`] type. The compiler derives it
/// (vector-port grouping, lever/probe port mapping, estimated delay table);
/// this parse is also the schema guarantee — the hand-rolled JSON in
/// `nucleation-hdl` must deserialize as a real `CellContract` or compilation
/// of the contract fails loudly here.
pub(crate) fn cell_contract(
    compiled: &nucleation_hdl::Compiled,
) -> Result<crate::io_contract::CellContract, String> {
    crate::io_contract::CellContract::from_json(&compiled.cell_contract_json())
        .map_err(|e| format!("compiler emitted an invalid CellContract: {e}"))
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

        /// Compile `blif` and write its typed-cell contract as JSON — the
        /// `CellContract` file format (name, `io` with typed ports/buses,
        /// `physical` sidecar). Vector ports (`a[0..3]` or `a0..a3`) group
        /// into word buses (LSB = index 0); single bits are boolean. Input
        /// port positions are the drive levers, output positions the dust
        /// probes, in the same schematic coordinates as `compile_blif`.
        ///
        /// The `physical.delays_rt` table is ESTIMATED from levelization
        /// depth (2 redstone ticks per level), not measured; `paste_safe`
        /// is false until proven. Pair with `compile_blif` for the
        /// schematic: schematic + this contract = an executable typed cell.
        pub fn compile_blif_contract(
            blif: &DiplomatStr,
            name: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let (blif, name) = decode(blif, name)?;
            let compiled = nucleation_hdl::compile_blif(blif, name).map_err(|e| {
                crate::bridge::set_last_error_detail(e.to_string());
                NucleationError::InvalidArgument
            })?;
            let contract = super::cell_contract(&compiled).map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::InvalidArgument
            })?;
            let json = contract.to_json().map_err(|e| {
                crate::bridge::set_last_error_detail(e);
                NucleationError::InvalidArgument
            })?;
            let _ = write!(out, "{json}");
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

    const CMP4: &str = include_str!("../../crates/nucleation-hdl/tests/data/cmp4.blif");

    /// The schema guarantee: the compiler's hand-rolled contract JSON
    /// deserializes as a real `CellContract`, typed and canonical.
    #[test]
    fn compiled_contract_parses_as_a_typed_cell_contract() {
        use crate::io_contract::{CellContract, IoType};

        let compiled = nucleation_hdl::compile_blif(CMP4, "cmp4").unwrap();
        let contract = super::cell_contract(&compiled).unwrap();
        assert_eq!(contract.name, "cmp4");
        let a = contract.io.get_input("a").expect("grouped bus port a");
        assert_eq!(a.io_type, IoType::UnsignedInt { bits: 4 });
        assert_eq!(a.positions.len(), 4, "one lever per bit");
        let eq = contract.io.outputs.get("eq").expect("boolean output eq");
        assert_eq!(eq.io_type, IoType::Boolean);
        assert!(!contract.physical.paste_safe, "unknown -> false");
        assert!(
            contract.physical.delay_rt("a", "lt").unwrap_or(0) >= 2,
            "estimated arrival from levelization depth"
        );
        assert!(
            !contract.physical.keepouts.is_empty(),
            "build bounds guard the cell body"
        );
        // canonical serde round trip
        let back = CellContract::from_json(&contract.to_json().unwrap()).unwrap();
        assert_eq!(back, contract);
    }

    /// The typed-cell promise end-to-end: compile cmp4, wrap schematic +
    /// contract in a `BackendCircuitExecutor` over the mc-tick oracle, and
    /// verify 64 sampled cases purely through port NAMES and word VALUES —
    /// no raw coordinates anywhere.
    #[cfg(all(feature = "simulation", feature = "mc-tick"))]
    #[test]
    fn cmp4_executes_as_a_typed_cell_by_port_name() {
        use crate::io_contract::Value;
        use crate::simulation::typed_executor::BackendCircuitExecutor;

        let compiled = nucleation_hdl::compile_blif(CMP4, "cmp4").unwrap();
        let contract = super::cell_contract(&compiled).unwrap();
        // Cells deploy BAKED (settled states saved); the mc-tick backend
        // loads with InWorld settle and trusts them. An unbaked build sits
        // inert until the first lever flip.
        let mut build = compiled.build;
        super::bake_build(&mut build).unwrap();
        let schem = super::to_schematic(&build, "cmp4").unwrap();
        let extra = nucleation_hdl::verify::extra_states();
        let extra: Vec<&str> = extra.iter().map(String::as_str).collect();
        let mut cell = BackendCircuitExecutor::for_cell(schem, &contract, &extra).unwrap();
        cell.settle(4000);

        // 8x8 sample of the 16x16 space; each operand set exercises every bit.
        const A: [u32; 8] = [0, 3, 5, 6, 9, 10, 12, 15];
        const B: [u32; 8] = [0, 2, 7, 8, 11, 13, 14, 15];
        for &a in &A {
            for &b in &B {
                cell.set_input("a", &Value::U32(a)).unwrap();
                cell.set_input("b", &Value::U32(b)).unwrap();
                cell.settle(400);
                assert_eq!(
                    cell.read_output("eq").unwrap(),
                    Value::Bool(a == b),
                    "eq({a},{b})"
                );
                assert_eq!(
                    cell.read_output("lt").unwrap(),
                    Value::Bool(a < b),
                    "lt({a},{b})"
                );
                assert_eq!(
                    cell.read_output("gt").unwrap(),
                    Value::Bool(a > b),
                    "gt({a},{b})"
                );
            }
        }
    }
}
