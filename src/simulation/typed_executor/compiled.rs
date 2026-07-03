//! Portable compiled-circuit format
//!
//! Serializes a [`super::TypedCircuitExecutor`]'s pre-digested build inputs —
//! the sign-stripped schematic, the resolved IO layout, and the simulation
//! options — into one versioned binary blob. Restoring from the blob skips
//! schematic-format parsing, insign sign extraction/DSL parsing, layout
//! building, and validation.
//!
//! ## Boundary (honest scope)
//! The redpiler compile itself (graph construction + backend init inside
//! [`crate::simulation::MchprsWorld::with_options`]) still runs on restore:
//! mchprs' `CompileGraph` and backend state carry no serde support, so
//! snapshotting them requires MCHPRS-fork surgery. When that lands, it slots
//! in as a new version of this same container.
//!
//! ## Determinism
//! Ports are sorted by name, but `UniversalSchematic` serializes `HashMap`
//! fields in iteration order, so the bytes are NOT stable across processes.
//! Content-hash dedupe should keep using the source schematic bytes.

use super::{IoMapping, StateMode};
use crate::simulation::SimulationOptions;
use crate::UniversalSchematic;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Magic prefix of the compiled-circuit container.
pub const COMPILED_MAGIC: &[u8; 4] = b"NCCK";
/// Current container version. Bump on ANY layout change.
pub const COMPILED_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct PortEntry {
    name: String,
    io_type: super::IoType,
    layout: super::LayoutFunction,
    positions: Vec<(i32, i32, i32)>,
}

#[derive(Serialize, Deserialize)]
struct CompiledCircuit {
    /// Sign-stripped schematic as gzipped .schem bytes — the palette-packed
    /// format is ~400x smaller than a bincode snapshot of the region arrays,
    /// and its parse is fast enough that snapshotting loses (measured in
    /// `bench_compiled_vs_schematic_start`)
    schematic_schem: Vec<u8>,
    inputs: Vec<PortEntry>,
    outputs: Vec<PortEntry>,
    state_mode: u8,
    optimize: bool,
    io_only: bool,
    custom_io: Vec<(i32, i32, i32)>,
}

fn ports_to_entries(ports: &HashMap<String, IoMapping>) -> Vec<PortEntry> {
    let mut entries: Vec<PortEntry> = ports
        .iter()
        .map(|(name, m)| PortEntry {
            name: name.clone(),
            io_type: m.io_type.clone(),
            layout: m.layout.clone(),
            positions: m.positions.clone(),
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

fn entries_to_ports(entries: Vec<PortEntry>) -> Result<HashMap<String, IoMapping>, String> {
    let mut out = HashMap::with_capacity(entries.len());
    for e in entries {
        let mapping = IoMapping::new(e.io_type, e.layout, e.positions)
            .map_err(|err| format!("port '{}': {}", e.name, err))?;
        out.insert(e.name, mapping);
    }
    Ok(out)
}

fn state_mode_to_u8(mode: StateMode) -> u8 {
    match mode {
        StateMode::Stateless => 0,
        StateMode::Stateful => 1,
        StateMode::Manual => 2,
    }
}

fn state_mode_from_u8(v: u8) -> Result<StateMode, String> {
    match v {
        0 => Ok(StateMode::Stateless),
        1 => Ok(StateMode::Stateful),
        2 => Ok(StateMode::Manual),
        other => Err(format!("unknown state mode {}", other)),
    }
}

pub(super) fn encode(
    schematic: &UniversalSchematic,
    inputs: &HashMap<String, IoMapping>,
    outputs: &HashMap<String, IoMapping>,
    state_mode: StateMode,
    options: &SimulationOptions,
) -> Result<Vec<u8>, String> {
    let data = CompiledCircuit {
        schematic_schem: crate::formats::schematic::to_schematic(schematic)
            .map_err(|e| format!("schematic encode: {}", e))?,
        inputs: ports_to_entries(inputs),
        outputs: ports_to_entries(outputs),
        state_mode: state_mode_to_u8(state_mode),
        optimize: options.optimize,
        io_only: options.io_only,
        custom_io: options.custom_io.iter().map(|p| (p.x, p.y, p.z)).collect(),
    };
    let body = bincode::serialize(&data).map_err(|e| format!("compiled encode: {}", e))?;
    let mut out = Vec::with_capacity(8 + body.len());
    out.extend_from_slice(COMPILED_MAGIC);
    out.extend_from_slice(&COMPILED_VERSION.to_le_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

#[allow(clippy::type_complexity)]
pub(super) fn decode(
    bytes: &[u8],
) -> Result<
    (
        UniversalSchematic,
        HashMap<String, IoMapping>,
        HashMap<String, IoMapping>,
        StateMode,
        SimulationOptions,
    ),
    String,
> {
    if bytes.len() < 8 || &bytes[0..4] != COMPILED_MAGIC {
        return Err("not a compiled circuit (bad magic)".to_string());
    }
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != COMPILED_VERSION {
        return Err(format!(
            "compiled circuit version {} unsupported (expected {})",
            version, COMPILED_VERSION
        ));
    }
    let data: CompiledCircuit =
        bincode::deserialize(&bytes[8..]).map_err(|e| format!("compiled decode: {}", e))?;
    let options = SimulationOptions {
        optimize: data.optimize,
        io_only: data.io_only,
        custom_io: data
            .custom_io
            .into_iter()
            .map(|(x, y, z)| crate::simulation::BlockPos::new(x, y, z))
            .collect(),
    };
    let schematic = crate::formats::schematic::from_schematic(&data.schematic_schem)
        .map_err(|e| format!("schematic decode: {}", e))?;
    Ok((
        schematic,
        entries_to_ports(data.inputs)?,
        entries_to_ports(data.outputs)?,
        state_mode_from_u8(data.state_mode)?,
        options,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::{create_executor_from_insign, ExecutionMode, TypedCircuitExecutor, Value};
    use super::*;
    use crate::schematic_builder::SchematicBuilder;
    use std::time::Instant;

    /// The insign full adder from the insign_io integration tests — real
    /// logic (torches + wires), 3 bool inputs, 2 bool outputs.
    fn full_adder_with_insign() -> UniversalSchematic {
        let template = r#"# Base layer
·····c····
·····c····
··ccccc···
·ccccccc··
cc··cccccc
·c··c·····
·ccccc····
·cccccc···
···cccc···
···c··c···

# Logic layer
·····│····
·····↑····
··│█←┤█···
·█◀←┬▲▲┐··
──··├┴┴┴←─
·█··↑·····
·▲─←┤█····
·█←┬▲▲┐···
···├┴┴┤···
···│··│···
"#;
        let builder = SchematicBuilder::from_template(template).unwrap();
        let mut schematic = builder.build().unwrap();

        let mut add_sign = |x: i32, y: i32, z: i32, name: &str, dir: &str| {
            let mut nbt = std::collections::HashMap::new();
            nbt.insert(
                "Text1".to_string(),
                format!("{{\"text\":\"@io.{}=rc([0,-1,0],[0,-1,0])\"}}", name),
            );
            nbt.insert(
                "Text2".to_string(),
                format!("{{\"text\":\"#io.{}:type=\\\"{}\\\"\"}}", name, dir),
            );
            nbt.insert(
                "Text3".to_string(),
                format!("{{\"text\":\"#io.{}:data_type=\\\"bool\\\"\"}}", name),
            );
            nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
            schematic
                .set_block_with_nbt(x, y, z, "minecraft:oak_sign[rotation=0]", nbt)
                .unwrap();
        };
        add_sign(3, 2, 9, "a", "input");
        add_sign(6, 2, 9, "b", "input");
        add_sign(9, 2, 4, "carry_in", "input");
        add_sign(5, 2, 0, "sum", "output");
        add_sign(0, 2, 4, "carry_out", "output");
        schematic
    }

    fn run_adder(
        ex: &mut TypedCircuitExecutor,
        a: bool,
        b: bool,
        cin: bool,
    ) -> (Value, Value) {
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("a".to_string(), Value::Bool(a));
        inputs.insert("b".to_string(), Value::Bool(b));
        inputs.insert("carry_in".to_string(), Value::Bool(cin));
        let result = ex
            .execute(inputs, ExecutionMode::FixedTicks { ticks: 40 })
            .unwrap();
        (
            result.outputs.get("sum").unwrap().clone(),
            result.outputs.get("carry_out").unwrap().clone(),
        )
    }

    #[test]
    fn compiled_round_trip_behaves_identically() {
        let schematic = full_adder_with_insign();
        let mut original = create_executor_from_insign(&schematic).unwrap();
        let bytes = original.to_compiled_bytes().unwrap();
        assert_eq!(&bytes[0..4], COMPILED_MAGIC);
        let mut restored = TypedCircuitExecutor::from_compiled_bytes(&bytes).unwrap();

        // identical layout
        let mut orig_in: Vec<_> = original.input_names();
        let mut rest_in: Vec<_> = restored.input_names();
        orig_in.sort();
        rest_in.sort();
        assert_eq!(orig_in, rest_in);

        // identical behavior over the full truth table
        for i in 0..8u8 {
            let (a, b, cin) = (i & 1 != 0, i & 2 != 0, i & 4 != 0);
            let expected = run_adder(&mut original, a, b, cin);
            let actual = run_adder(&mut restored, a, b, cin);
            assert_eq!(expected, actual, "mismatch at a={} b={} cin={}", a, b, cin);
        }
    }

    #[test]
    fn compiled_rejects_bad_magic_and_version() {
        let schematic = full_adder_with_insign();
        let ex = create_executor_from_insign(&schematic).unwrap();
        let bytes = ex.to_compiled_bytes().unwrap();

        let mut bad_magic = bytes.clone();
        bad_magic[0] = b'X';
        match TypedCircuitExecutor::from_compiled_bytes(&bad_magic) {
            Err(e) => assert!(e.contains("magic"), "unexpected error: {}", e),
            Ok(_) => panic!("bad magic accepted"),
        }

        let mut bad_version = bytes.clone();
        bad_version[4] = 99;
        match TypedCircuitExecutor::from_compiled_bytes(&bad_version) {
            Err(e) => assert!(e.contains("version"), "unexpected error: {}", e),
            Ok(_) => panic!("bad version accepted"),
        }

        assert!(TypedCircuitExecutor::from_compiled_bytes(&bytes[0..4]).is_err());
    }

    #[test]
    fn bench_compiled_vs_schematic_start() {
        let schematic = full_adder_with_insign();
        let schem_bytes = crate::formats::schematic::to_schematic(&schematic).unwrap();
        let compiled_bytes = create_executor_from_insign(&schematic)
            .unwrap()
            .to_compiled_bytes()
            .unwrap();

        const N: u32 = 50;
        let t0 = Instant::now();
        for _ in 0..N {
            let parsed =
                crate::formats::schematic::from_schematic(&schem_bytes).unwrap();
            let _ex = create_executor_from_insign(&parsed).unwrap();
        }
        let full_us = t0.elapsed().as_micros() / N as u128;

        let t1 = Instant::now();
        for _ in 0..N {
            let _parsed =
                crate::formats::schematic::from_schematic(&schem_bytes).unwrap();
        }
        let parse_us = t1.elapsed().as_micros() / N as u128;

        let t2 = Instant::now();
        for _ in 0..N {
            let _ex = TypedCircuitExecutor::from_compiled_bytes(&compiled_bytes).unwrap();
        }
        let compiled_us = t2.elapsed().as_micros() / N as u128;

        println!(
            "executor start: full path {} µs (schem parse {} µs, insign+layout+compile {} µs) vs compiled path {} µs — blob {} bytes, schem {} bytes",
            full_us,
            parse_us,
            full_us - parse_us,
            compiled_us,
            compiled_bytes.len(),
            schem_bytes.len()
        );
        // sanity, not a hard perf gate: compiled must not be materially slower
        assert!(compiled_us <= full_us * 2);
    }

    /// Frame-scale data point: a 48x48 repeater/wire field (~2.3k redstone
    /// components) with comparator IO ports — how do the two start paths
    /// scale when redpiler has real work to do?
    #[test]
    fn bench_compiled_vs_schematic_start_large() {
        let mut schematic = UniversalSchematic::new("large".to_string());
        let base = crate::BlockState::new("minecraft:gray_concrete".to_string());
        let wire = crate::BlockState::new("minecraft:redstone_wire".to_string());
        let repeater =
            crate::BlockState::new("minecraft:repeater[facing=west,delay=1]".to_string());
        for x in 0..48 {
            for z in 0..48 {
                schematic.set_block(x, 0, z, &base);
                if x % 8 == 7 {
                    schematic.set_block(x, 1, z, &repeater);
                } else {
                    schematic.set_block(x, 1, z, &wire);
                }
            }
        }
        // wire IO at both ends of row 0 (same pattern as the full-adder test)
        let mut add_sign = |x: i32, z: i32, name: &str, dir: &str| {
            let mut nbt = std::collections::HashMap::new();
            nbt.insert(
                "Text1".to_string(),
                format!("{{\"text\":\"@io.{}=rc([0,-1,0],[0,-1,0])\"}}", name),
            );
            nbt.insert(
                "Text2".to_string(),
                format!("{{\"text\":\"#io.{}:type=\\\"{}\\\"\"}}", name, dir),
            );
            nbt.insert(
                "Text3".to_string(),
                format!("{{\"text\":\"#io.{}:data_type=\\\"bool\\\"\"}}", name),
            );
            nbt.insert("Text4".to_string(), "{\"text\":\"\"}".to_string());
            schematic
                .set_block_with_nbt(x, 2, z, "minecraft:oak_sign[rotation=0]", nbt)
                .unwrap();
        };
        add_sign(0, 0, "in", "input");
        add_sign(46, 0, "out", "output"); // x=47 is a repeater — target the wire

        let schem_bytes = crate::formats::schematic::to_schematic(&schematic).unwrap();
        let compiled_bytes = create_executor_from_insign(&schematic)
            .unwrap()
            .to_compiled_bytes()
            .unwrap();

        const N: u32 = 10;
        let t0 = Instant::now();
        for _ in 0..N {
            let parsed = crate::formats::schematic::from_schematic(&schem_bytes).unwrap();
            let _ex = create_executor_from_insign(&parsed).unwrap();
        }
        let full_us = t0.elapsed().as_micros() / N as u128;

        let t1 = Instant::now();
        for _ in 0..N {
            let _ex = TypedCircuitExecutor::from_compiled_bytes(&compiled_bytes).unwrap();
        }
        let compiled_us = t1.elapsed().as_micros() / N as u128;

        println!(
            "LARGE executor start: full path {} µs vs compiled path {} µs — blob {} bytes, schem {} bytes",
            full_us, compiled_us, compiled_bytes.len(), schem_bytes.len()
        );
    }
}
