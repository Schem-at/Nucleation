//! Named, typed operations over a real mc-tick circuit.
//!
//! A profile attaches the existing `CellContract` to an explicit input order,
//! observation method and fixed game-tick budget. It does not replace the build
//! with an arithmetic implementation. Tests supply their own expected answers.
//! Positions are in the simulation's coordinates. Archive runners use the
//! production importer's bounding-box-relative coordinates and record that fact.
use crate::io_contract::{CellContract, IoType, LayoutFunction, Value};
use mc_tick::{Checkpoint, Pos, Simulation};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::collections::{BTreeMap, BTreeSet};

pub type Values = BTreeMap<String, Json>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterProfile {
    pub schema_version: u32,
    pub id: String,
    pub source_sha256: String,
    pub source_data_version: i32,
    pub description: String,
    pub evidence: Vec<String>,
    pub cell: CellContract,
    /// Every input exactly once; bits within a port follow its positions (LSB first).
    pub input_order: Vec<String>,
    pub outputs: BTreeMap<String, Observation>,
    pub setup_ticks: u64,
    pub response_ticks: u64,
    pub require_quiescent: bool,
    pub rng_seed: i64,
    /// Original-world origin used for order-sensitive Java dust hashing.
    pub hash_origin: [i32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Observation {
    /// Every mapped position must remain a redstone lamp.
    LampLit,
    /// One bit per aperture cell: true only for actual air/cave_air/void_air.
    /// An arbitrary non-solid block is NOT silently counted as an open doorway.
    Air,
}

fn width(ty: &IoType) -> Result<usize, String> {
    match ty {
        IoType::Boolean => Ok(1),
        IoType::UnsignedInt { bits: 1..=64 } => Ok(ty.bit_count()),
        _ => Err("adapter v1 supports Boolean and unsigned integers of 1..=64 bits".into()),
    }
}

fn typed(ty: &IoType, value: &Json) -> Result<Value, String> {
    let bits = width(ty)?;
    match ty {
        IoType::Boolean => value
            .as_bool()
            .map(Value::Bool)
            .ok_or("expected a boolean".into()),
        _ => {
            let n = value.as_u64().ok_or("expected an unsigned integer")?;
            if bits < 64 && n >= (1u64 << bits) {
                return Err(format!("{n} is outside unsigned {bits}-bit range"));
            }
            Ok(Value::U64(n))
        }
    }
}

impl AdapterProfile {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || self.id.is_empty() {
            return Err("expected named adapter schema version 1".into());
        }
        if self.source_sha256.len() != 64
            || !self.source_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("expected pinned source SHA-256".into());
        }
        if self.response_ticks == 0 || self.response_ticks > 100_000 || self.setup_ticks > 100_000 {
            return Err("invalid bounded tick budget".into());
        }
        let input_names: BTreeSet<_> = self.cell.io.inputs.keys().collect();
        if input_names.is_empty()
            || self.input_order.iter().collect::<BTreeSet<_>>() != input_names
            || self.input_order.len() != input_names.len()
        {
            return Err("input_order must name every input exactly once".into());
        }
        if self.outputs.is_empty()
            || self.outputs.keys().collect::<BTreeSet<_>>() != self.cell.io.outputs.keys().collect()
        {
            return Err("every output needs an observation method".into());
        }
        // Validate supported types BEFORE generic layout arithmetic (untrusted JSON).
        for mapping in self
            .cell
            .io
            .inputs
            .values()
            .chain(self.cell.io.outputs.values())
        {
            width(&mapping.io_type)?;
            if mapping.layout != LayoutFunction::OneToOne {
                return Err("adapter v1 requires OneToOne binary ports".into());
            }
            mapping.validate()?;
        }
        let mut occupied = BTreeSet::new();
        for mapping in self.cell.io.inputs.values() {
            for pos in &mapping.positions {
                if !occupied.insert(*pos) {
                    return Err("input positions overlap".into());
                }
            }
        }
        if !self.cell.io.buses.is_empty() {
            return Err(
                "adapter v1 uses explicit bit positions; bus timing is not implemented".into(),
            );
        }
        Ok(())
    }

    pub fn validate_values(&self, values: &Values, output: bool) -> Result<(), String> {
        let mappings = if output {
            &self.cell.io.outputs
        } else {
            &self.cell.io.inputs
        };
        if values.keys().collect::<BTreeSet<_>>() != mappings.keys().collect() {
            return Err("values must name every port exactly once".into());
        }
        for (name, mapping) in mappings {
            typed(&mapping.io_type, &values[name]).map_err(|e| format!("{name}: {e}"))?;
        }
        Ok(())
    }
}

pub struct TickAdapter {
    profile: AdapterProfile,
    sim: Simulation,
    initial: Checkpoint,
}

fn property<'a>(descriptor: &'a str, key: &str) -> Option<&'a str> {
    descriptor
        .split_once('[')?
        .1
        .trim_end_matches(']')
        .split(',')
        .filter_map(|p| p.split_once('='))
        .find_map(|(k, v)| (k == key).then_some(v))
}

impl TickAdapter {
    /// The caller supplies an honestly imported simulation; no unknown block is
    /// substituted. Source hashing and save-version validation belong to loading.
    pub fn new(mut sim: Simulation, profile: AdapterProfile) -> Result<Self, String> {
        profile.validate()?;
        sim.set_rng_seed(profile.rng_seed);
        // Random block ticking is explicitly disabled in v1. The checkpoint
        // carries entity RNG; it currently does not carry the random-tick RNG.
        sim.set_random_ticks(0);
        sim.discard_recording();
        sim.record_updates(false);
        let initial = sim.checkpoint();
        let mut adapter = Self {
            sim,
            profile,
            initial,
        };
        adapter.advance(adapter.profile.setup_ticks)?;
        adapter.validate_input_blocks()?;
        adapter.read()?;
        adapter.initial = adapter.sim.checkpoint();
        Ok(adapter)
    }

    pub fn profile(&self) -> &AdapterProfile {
        &self.profile
    }
    pub fn simulation(&self) -> &Simulation {
        &self.sim
    }
    pub fn reset(&mut self) {
        self.sim.restore(&self.initial);
    }

    fn descriptor(&self, p: (i32, i32, i32)) -> Result<&str, String> {
        if !self.sim.world().contains(Pos::new(p.0, p.1, p.2)) {
            return Err(format!("observation/input {p:?} is outside the simulated world"));
        }
        self.sim
            .registry()
            .descriptor(self.sim.world().get(Pos::new(p.0, p.1, p.2)))
            .ok_or_else(|| format!("missing state descriptor at {p:?}"))
    }
    fn lever(&self, p: (i32, i32, i32)) -> Result<bool, String> {
        let d = self.descriptor(p)?;
        if d.split('[').next() != Some("minecraft:lever") {
            return Err(format!("input {p:?} requires an actual lever, got {d}"));
        }
        match property(d, "powered") {
            Some("true") => Ok(true),
            Some("false") => Ok(false),
            _ => Err(format!("lever {p:?} has no boolean powered property")),
        }
    }
    fn validate_input_blocks(&self) -> Result<(), String> {
        for mapping in self.profile.cell.io.inputs.values() {
            for &p in &mapping.positions {
                self.lever(p)?;
            }
        }
        Ok(())
    }
    fn advance(&mut self, ticks: u64) -> Result<(), String> {
        let before = self.sim.tick_count();
        let stop = self.sim.run(ticks);
        if self.sim.tick_count() != before + ticks {
            return Err(format!("simulation stopped early: {stop:?}"));
        }
        if let Some(report) = self.sim.unknown_report() {
            return Err(format!("unsupported blocks: {report}"));
        }
        if self.profile.require_quiescent && !self.sim.is_quiescent() {
            return Err(format!("circuit remains active after {ticks} ticks"));
        }
        Ok(())
    }

    /// Toggle differing lever bits in the declared order before the next game
    /// tick, then step the fixed response budget. No settling between bits.
    /// Invalid values or missing input hardware are rejected before any action.
    pub fn drive(&mut self, values: &Values) -> Result<(), String> {
        self.profile.validate_values(values, false)?;
        self.validate_input_blocks()?;
        for name in &self.profile.input_order {
            let m = &self.profile.cell.io.inputs[name];
            let bits = m.encode(&typed(&m.io_type, &values[name])?)?;
            for (&p, signal) in m.positions.iter().zip(bits) {
                if self.lever(p)? != (signal > 0) {
                    self.sim.use_block(Pos::new(p.0, p.1, p.2));
                    if self.lever(p)? != (signal > 0) {
                        return Err(format!("lever {p:?} did not switch"));
                    }
                }
            }
        }
        self.advance(self.profile.response_ticks)
    }

    pub fn read(&self) -> Result<Values, String> {
        let mut values = Values::new();
        for (name, observation) in &self.profile.outputs {
            let m = &self.profile.cell.io.outputs[name];
            let mut signals = Vec::with_capacity(m.positions.len());
            for &p in &m.positions {
                let d = self.descriptor(p)?;
                let block = d.split('[').next().unwrap_or(d);
                let yes = match observation {
                    Observation::Air => matches!(
                        block,
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                    ),
                    Observation::LampLit => {
                        if block != "minecraft:redstone_lamp" {
                            return Err(format!("output {name} at {p:?} requires a lamp, got {d}"));
                        }
                        match property(d, "lit") {
                            Some("true") => true,
                            Some("false") => false,
                            _ => return Err(format!("lamp {p:?} has no boolean lit property")),
                        }
                    }
                };
                signals.push(if yes { 15 } else { 0 });
            }
            let decoded = m.decode(&signals)?;
            values.insert(
                name.clone(),
                match &m.io_type {
                    IoType::Boolean => Json::from(decoded.as_bool()?),
                    _ => Json::from(decoded.as_u64()?),
                },
            );
        }
        Ok(values)
    }

    pub fn execute(&mut self, values: &Values) -> Result<Values, String> {
        self.drive(values)?;
        self.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io_contract::IoLayoutBuilder;
    fn adapter() -> TickAdapter {
        let s = mc_tick::Structure::parse(r#"{size:[3,2,1],palette:[{Name:"minecraft:stone"},{Name:"minecraft:lever",Properties:{face:"floor",facing:"north",powered:"false"}},{Name:"minecraft:redstone_lamp",Properties:{lit:"false"}}],blocks:[{pos:[0,0,0],state:0},{pos:[0,1,0],state:1},{pos:[1,1,0],state:2}],entities:[]}"#).unwrap();
        let sim = mc_test::try_build_sim(
            &s,
            Pos::default(),
            mc_test::SettleMode::InWorld,
            &[],
            &[],
            None,
            "adapter",
        )
        .unwrap();
        let io = IoLayoutBuilder::new()
            .add_input(
                "switch",
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(0, 1, 0)],
            )
            .unwrap()
            .add_output(
                "lit",
                IoType::Boolean,
                LayoutFunction::OneToOne,
                vec![(1, 1, 0)],
            )
            .unwrap()
            .build();
        TickAdapter::new(
            sim,
            AdapterProfile {
                schema_version: 1,
                id: "test".into(),
                source_sha256: "0".repeat(64),
                source_data_version: 4671,
                description: "test".into(),
                evidence: vec![],
                cell: CellContract::new("test", io),
                input_order: vec!["switch".into()],
                outputs: BTreeMap::from([("lit".into(), Observation::LampLit)]),
                setup_ticks: 4,
                response_ticks: 8,
                require_quiescent: true,
                rng_seed: 0,
                hash_origin: [0, 0, 0],
            },
        )
        .unwrap()
    }
    #[test]
    fn real_lever_lamp_and_checkpoint_repeat() {
        let mut a = adapter();
        let start = a.simulation().checkpoint();
        let on = BTreeMap::from([("switch".into(), Json::from(true))]);
        assert_eq!(a.execute(&on).unwrap()["lit"], true);
        let end = a.simulation().checkpoint();
        a.reset();
        assert_eq!(a.simulation().checkpoint(), start);
        assert_eq!(a.execute(&on).unwrap()["lit"], true);
        assert_eq!(a.simulation().checkpoint(), end);
        assert_eq!(
            a.execute(&BTreeMap::from([("switch".into(), Json::from(false))]))
                .unwrap()["lit"],
            false
        );
    }
    #[test]
    fn bad_values_and_ports_do_not_mutate_simulation() {
        let mut a = adapter();
        let start = a.simulation().checkpoint();
        for values in [
            BTreeMap::new(),
            BTreeMap::from([("switch".into(), Json::from(1))]),
            BTreeMap::from([("typo".into(), Json::from(true))]),
        ] {
            assert!(a.execute(&values).is_err());
            assert_eq!(a.simulation().checkpoint(), start);
        }
        assert!(typed(&IoType::UnsignedInt { bits: 8 }, &Json::from(256)).is_err());
        assert!(typed(&IoType::UnsignedInt { bits: 8 }, &Json::from(-1)).is_err());
        assert!(typed(&IoType::UnsignedInt { bits: 8 }, &Json::from(1.5)).is_err());
        let mut p = a.profile().clone();
        p.input_order.push("switch".into());
        assert!(p.validate().is_err());
        p = a.profile().clone();
        p.cell.io.inputs.get_mut("switch").unwrap().io_type =
            IoType::UnsignedInt { bits: usize::MAX };
        assert!(p.validate().is_err());
    }

    #[test]
    fn air_observations_cannot_report_outside_world_as_open() {
        let mut a = adapter();
        a.profile.outputs.insert("lit".into(), Observation::Air);
        a.profile.cell.io.outputs.get_mut("lit").unwrap().positions = vec![(i32::MAX, 0, 0)];
        assert!(a.read().unwrap_err().contains("outside the simulated world"));
        a.profile.cell.io.outputs.get_mut("lit").unwrap().positions = vec![(0, 1, 1)];
        assert_eq!(a.read().unwrap()["lit"], true);
    }
}
