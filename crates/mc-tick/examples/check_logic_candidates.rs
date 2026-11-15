//! Functional checks for the four pinned Logical Redstone Archive candidates.
//!
//! check_logic_candidates --fixtures DIR [--case LFF006|COUNT001|DECODE004|ADD005]
//! Accepts ID.snbt or the original schematic basename with an .snbt extension.
//! These are engine functional checks, not a fresh vanilla capture or benchmark.
use mc_test::{try_build_sim, SettleMode};
use mc_tick::{Pos, Simulation, Structure};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

type Check = Result<(), String>;
const CASES: &[(&str, &str, (i32, i32, i32))] = &[
    ("LFF006", "LFF006_dlatch", (8, 3, 5)),
    ("COUNT001", "COUNT001_8bit_counter_load", (8, 20, 4)),
    ("DECODE004", "DECODE004_decode_h2v", (7, 9, 11)),
    ("ADD005", "ADD005_8bit_cle", (13, 21, 20)),
];

fn require(ok: bool, message: impl FnOnce() -> String) -> Check {
    if ok {
        Ok(())
    } else {
        Err(message())
    }
}

fn property<'a>(descriptor: &'a str, name: &str) -> Option<&'a str> {
    descriptor
        .split_once('[')?
        .1
        .trim_end_matches(']')
        .split(',')
        .find_map(|p| {
            p.split_once('=')
                .filter(|(k, _)| *k == name)
                .map(|(_, v)| v)
        })
}

struct Run {
    sim: Simulation,
    case: String,
    fixture: PathBuf,
    source_hash: String,
    start_tick: u64,
    checks: usize,
    actions: usize,
    stage: String,
    observations: Vec<Value>,
    outputs: Vec<Pos>,
}

impl Run {
    fn new(dir: &Path, case: &str, basename: &str, size: (i32, i32, i32)) -> Result<Self, String> {
        let fixture = [format!("{case}.snbt"), format!("{basename}.snbt")]
            .iter()
            .map(|name| dir.join(name))
            .find(|p| p.is_file())
            .ok_or_else(|| {
                format!(
                    "missing {case}.snbt or {basename}.snbt in {}",
                    dir.display()
                )
            })?;
        let source = std::fs::read_to_string(&fixture).map_err(|e| e.to_string())?;
        let structure = Structure::parse(&source).map_err(|e| format!("parse: {e:?}"))?;
        require(structure.data_version == Some(4189), || {
            format!(
                "pinned source save DataVersion must remain 4189, got {:?}",
                structure.data_version
            )
        })?;
        require(structure.size == size, || {
            format!("fixture dimensions {:?}, expected {size:?}", structure.size)
        })?;
        require(
            structure.entities.is_empty()
                && structure.item_entities.is_empty()
                && structure.inventories.is_empty(),
            || "stationary logic fixture contains entities or inventories".into(),
        )?;
        require(
            !structure.palette.iter().any(|d| d.contains("piston")),
            || "stationary logic fixture contains piston states".into(),
        )?;
        // The source save version is passed through; it is not replaced with the
        // desired vanilla reference version. No extra block is declared inert.
        let mut sim = try_build_sim(
            &structure,
            Pos::default(),
            SettleMode::InWorld,
            &[],
            &[],
            structure.data_version,
            case,
        )?;
        sim.set_rng_seed(0);
        let before = sim.tick_count();
        let stop = sim.run(128);
        require(sim.tick_count() == before + 128, || {
            format!("setup stopped early: {stop:?}")
        })?;
        require(sim.is_quiescent(), || {
            "fixture did not settle after 128 setup ticks".into()
        })?;
        if let Some(report) = sim.unknown_report() {
            return Err(format!("unknown blocks during setup: {report}"));
        }
        sim.record();
        sim.record_updates(true);
        let start_tick = sim.tick_count();
        Ok(Self {
            sim,
            case: case.into(),
            fixture,
            source_hash: blake3::hash(source.as_bytes()).to_hex().to_string(),
            start_tick,
            checks: 0,
            actions: 0,
            stage: "static port mapping".into(),
            observations: Vec::new(),
            outputs: Vec::new(),
        })
    }

    fn descriptor(&self, pos: Pos) -> Result<&str, String> {
        self.sim
            .registry()
            .descriptor(self.sim.world().get(pos))
            .ok_or_else(|| format!("{}: missing state descriptor at {pos:?}", self.stage))
    }

    fn block(&self, pos: Pos, name: &str) -> Check {
        let actual = self.descriptor(pos)?;
        require(actual.split('[').next() == Some(name), || {
            format!(
                "{}: static port {pos:?} must be {name}, got {actual}",
                self.stage
            )
        })
    }

    fn block_property(&self, pos: Pos, key: &str, expected: &str) -> Check {
        let actual = self.descriptor(pos)?;
        require(property(actual, key) == Some(expected), || {
            format!(
                "{}: static port {pos:?} expected {key}={expected}, got {actual}",
                self.stage
            )
        })
    }

    fn lever(&mut self, pos: Pos, value: bool) -> Check {
        self.block(pos, "minecraft:lever")?;
        let actual = property(self.descriptor(pos)?, "powered") == Some("true");
        if actual != value {
            self.sim.use_block(pos);
            self.actions += 1;
            require(
                (property(self.descriptor(pos)?, "powered") == Some("true")) == value,
                || format!("{}: lever {pos:?} did not switch to {value}", self.stage),
            )?;
        }
        Ok(())
    }

    fn bus(&mut self, positions: &[Pos], value: u8) -> Check {
        // Positions are fixed from labels/static wiring, never fitted to outputs.
        for (bit, &pos) in positions.iter().enumerate() {
            self.lever(pos, value & (1 << bit) != 0)?;
        }
        Ok(())
    }

    fn advance(&mut self, ticks: u64) -> Check {
        let before = self.sim.tick_count();
        let stop = self.sim.run(ticks);
        require(self.sim.tick_count() == before + ticks, || {
            format!(
                "{}: simulation stopped at tick {}: {stop:?}",
                self.stage,
                self.sim.tick_count()
            )
        })?;
        if let Some(report) = self.sim.unknown_report() {
            return Err(format!("{}: unknown blocks: {report}", self.stage));
        }
        Ok(())
    }

    fn button(&mut self, pos: Pos) -> Check {
        self.block(pos, "minecraft:stone_button")?;
        self.block_property(pos, "powered", "false")?;
        self.sim.use_block(pos);
        self.actions += 1;
        self.block_property(pos, "powered", "true")?;
        // An actual stone-button press/release, followed by a generous fixed
        // settling window. This is functional testing, not a claimed latency.
        self.advance(64)?;
        self.block_property(pos, "powered", "false")
    }

    fn lamps(&self, positions: &[Pos]) -> Result<u8, String> {
        let mut value = 0;
        for (bit, &pos) in positions.iter().enumerate() {
            self.block(pos, "minecraft:redstone_lamp")?;
            match property(self.descriptor(pos)?, "lit") {
                Some("true") => value |= 1 << bit,
                Some("false") => {}
                _ => {
                    return Err(format!(
                        "{}: lamp {pos:?} has no boolean lit state",
                        self.stage
                    ))
                }
            }
        }
        Ok(value)
    }

    fn expect(&mut self, positions: &[Pos], expected: u8, detail: Value) -> Check {
        let actual = self.lamps(positions)?;
        self.observations
            .push(json!({"stage": self.stage, "tick": self.sim.tick_count(),
            "expected": expected, "actual": actual, "input": detail}));
        require(actual == expected, || {
            format!(
                "{} at tick {}: expected {expected:#04x}, got {actual:#04x}; inputs {detail}",
                self.stage,
                self.sim.tick_count()
            )
        })?;
        self.checks += 1;
        require(self.sim.is_quiescent(), || {
            format!("{}: outputs sampled with pending work", self.stage)
        })
    }

    fn finish_checks(&self) -> Check {
        require(
            self.sim.moving_blocks().is_empty()
                && self.sim.item_entities().is_empty()
                && self.sim.minecarts().is_empty()
                && self.sim.entity_bodies().is_empty(),
            || "unexpected movement/entity activity in stationary circuit".into(),
        )?;
        require(
            self.sim.recorded_entities().is_empty() && self.sim.recorded_inventory().is_empty(),
            || "unexpected entity/inventory records in stationary circuit".into(),
        )?;
        for &pos in &self.outputs {
            require(self.sim.recorded().iter().any(|r| r.pos == pos), || {
                format!("output {pos:?} never changed; input activity alone is insufficient")
            })?;
        }
        require(!self.sim.recorded_updates().is_empty(), || {
            "no update deliveries recorded".into()
        })
    }

    fn report(&self, result: Check) -> Value {
        let fingerprint = blake3::hash(
            format!(
                "{:?}{:?}{:?}{:?}",
                self.sim.recorded(),
                self.sim.recorded_updates(),
                self.sim.recorded_inventory(),
                self.sim.recorded_entities()
            )
            .as_bytes(),
        );
        json!({"case": self.case, "fixture": self.fixture, "snbt_blake3": self.source_hash,
            "source_data_version": 4189, "settling": "InWorld + 128 ticks",
            "status": if result.is_ok() {"functional_pass"} else {"functional_fail"},
            "first_failure": result.err(), "last_stage": self.stage,
            "checks_passed": self.checks, "input_actions": self.actions,
            "ticks": self.sim.tick_count() - self.start_tick, "block_changes": self.sim.recorded().len(),
            "update_deliveries": self.sim.recorded_updates().len(),
            "output_changes": self.outputs.iter().map(|p| json!({"pos":[p.x,p.y,p.z],
                "changes":self.sim.recorded().iter().filter(|r|r.pos==*p).count()})).collect::<Vec<_>>(),
            "ordered_record_fingerprint": fingerprint.to_hex().to_string(),
            "observations": self.observations, "vanilla_parity": "not_newly_tested",
            "scope": "Fixed functional endpoints and ordered engine records; not a benchmark or complete tick-by-tick vanilla trace."})
    }
}

fn latch(run: &mut Run) -> Check {
    let data = Pos::new(0, 1, 4);
    let clock = Pos::new(7, 1, 2);
    let output = [Pos::new(0, 1, 0)];
    run.outputs = output.to_vec();
    // Clock and Data labels are retained in the pinned static inspection catalog.
    for (index, value) in [false, true, false, true, true, false]
        .into_iter()
        .enumerate()
    {
        let held = run.lamps(&output)?;
        run.stage = format!("latch {index}: change data without clock");
        run.lever(data, value)?;
        run.advance(32)?;
        run.expect(&output, held, json!({"data":value,"clock":false}))?;
        run.stage = format!("latch {index}: store data with full clock pulse");
        run.button(clock)?;
        run.expect(
            &output,
            u8::from(value),
            json!({"data":value,"clock":"pressed/released"}),
        )?;
        run.stage = format!("latch {index}: hold stored value while data changes");
        run.lever(data, !value)?;
        run.advance(32)?;
        run.expect(
            &output,
            u8::from(value),
            json!({"data":!value,"clock":false}),
        )?;
    }
    run.finish_checks()
}

fn counter(run: &mut Run) -> Check {
    let data: Vec<_> = (0..8).map(|i| Pos::new(0, 5 + 2 * i, 2)).collect();
    let output: Vec<_> = (0..8).map(|i| Pos::new(5, 5 + 2 * i, 3)).collect();
    run.outputs = output.clone();
    let load = Pos::new(0, 1, 2);
    let count = Pos::new(0, 3, 2);
    // Fixed bottom-to-top order follows the carry staircase rising above each
    // slice. The highest slice omits the torch feeding the next carry stage.
    // If this wiring interpretation is wrong, fail; never reverse bits to fit.
    for y in (5..19).step_by(2) {
        run.block(Pos::new(5, y, 1), "minecraft:redstone_wall_torch")?;
        run.block_property(Pos::new(5, y, 1), "facing", "north")?;
    }
    run.block(Pos::new(5, 19, 1), "minecraft:air")?;
    for value in [0u8, 1, 2, 4, 8, 16, 32, 64, 128, 127, 254, 255, 0x55, 0xaa] {
        let held = run.lamps(&output)?;
        run.stage = format!("counter: set load bus {value:#04x} without Load");
        run.bus(&data, value)?;
        run.advance(32)?;
        run.expect(&output, held, json!({"load_bus":value,"load":false}))?;
        run.stage = format!("counter: load {value:#04x}");
        run.button(load)?;
        run.expect(&output, value, json!({"load_bus":value,"load":true}))?;
        for step in 1u8..=3 {
            run.stage = format!("counter: count {step} after loading {value:#04x}");
            run.button(count)?;
            run.expect(
                &output,
                value.wrapping_add(step),
                json!({"loaded":value,"count_pulses":step}),
            )?;
        }
    }
    run.stage = "counter: initialize full modulo-256 cycle".into();
    run.bus(&data, 0)?;
    run.advance(32)?;
    run.button(load)?;
    run.expect(&output, 0, json!({"load_bus":0}))?;
    for pulse in 1..=256 {
        run.stage = format!("counter: full cycle pulse {pulse}");
        run.button(count)?;
        run.expect(&output, pulse as u8, json!({"count_pulses":pulse}))?;
    }
    run.finish_checks()
}

fn decoder(run: &mut Run) -> Check {
    let data: Vec<_> = [4, 6, 8, 10]
        .into_iter()
        .map(|z| Pos::new(6, 1, z))
        .collect();
    let output: Vec<_> = [2, 4, 6, 8]
        .into_iter()
        .map(|y| Pos::new(0, y, 0))
        .collect();
    run.outputs = output.clone();
    // The downloaded original has identical programmed rows, not a complete
    // 4-to-16 decoder. Each row ORs input0,!input1,input2,!input3, then inverts.
    // This truth model comes from components/facing, not observed engine output.
    for y in [2, 4, 6, 8] {
        for z in [4, 8] {
            run.block(Pos::new(1, y, z), "minecraft:repeater")?;
            run.block_property(Pos::new(1, y, z), "facing", "east")?;
        }
        for z in [6, 10] {
            run.block(Pos::new(1, y, z), "minecraft:redstone_wall_torch")?;
            run.block_property(Pos::new(1, y, z), "facing", "west")?;
        }
        run.block(Pos::new(0, y, 1), "minecraft:redstone_wall_torch")?;
        run.block_property(Pos::new(0, y, 1), "facing", "north")?;
    }
    // Ascending and descending orders include both turn-on/off and multi-bit
    // changes. Four outputs must agree; one-hot would be incorrect here.
    for word in (0u8..16).chain((0u8..16).rev()) {
        run.stage = format!("decoder: original identical rows, input {word:04b}");
        run.bus(&data, word)?;
        run.advance(32)?;
        run.expect(
            &output,
            if word == 0b1010 { 0b1111 } else { 0 },
            json!({"bits_in_increasing_z_order":(0..4).map(|i| (word>>i)&1).collect::<Vec<_>>(),
                "expected_programmed_pattern":[0,1,0,1]}),
        )?;
    }
    run.finish_checks()
}

fn adder(run: &mut Run) -> Check {
    // Sign labels give both input and output weights 1,2,...128. The named A/B
    // assignment is arbitrary because addition is commutative; bit weights are
    // not. CIN/COUT are explicitly labeled in the original artifact.
    let a: Vec<_> = (0..8).map(|i| Pos::new(0, 4 + 2 * i, 17 - 2 * i)).collect();
    let b: Vec<_> = (0..8).map(|i| Pos::new(0, 4 + 2 * i, 19 - 2 * i)).collect();
    let sum: Vec<_> = (0..8)
        .map(|i| Pos::new(11, 1 + 2 * i, 15 - 2 * i))
        .collect();
    let cin = Pos::new(5, 1, 18);
    let cout = [Pos::new(10, 18, 7)];
    run.outputs = sum.iter().copied().chain(cout).collect();
    let mut pairs = vec![
        (0u8, 0u8),
        (0, 255),
        (255, 0),
        (255, 255),
        (127, 1),
        (128, 128),
        (0x55, 0xaa),
        (0xaa, 0x55),
        (0x55, 0x55),
        (0xaa, 0xaa),
        (15, 1),
        (240, 16),
    ];
    for bit in 0..8 {
        pairs.push((1 << bit, 0));
        pairs.push((0, 1 << bit));
        pairs.push((((1u16 << (bit + 1)) - 1) as u8, 1));
    }
    let mut rng = 0x4c52_4135u32;
    for _ in 0..128 {
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let left = (rng >> 24) as u8;
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
        pairs.push((left, (rng >> 24) as u8));
    }
    for (index, (left, right)) in pairs.into_iter().enumerate() {
        for carry in [false, true] {
            run.stage = format!(
                "CLE vector {index}: {left:#04x}+{right:#04x}+{}",
                u8::from(carry)
            );
            run.bus(&a, left)?;
            run.bus(&b, right)?;
            run.lever(cin, carry)?;
            run.advance(32)?;
            let expected = u16::from(left) + u16::from(right) + u16::from(carry);
            let detail = json!({"a":left,"b":right,"cin":carry});
            run.expect(&sum, expected as u8, detail.clone())?;
            run.expect(&cout, (expected >> 8) as u8, detail)?;
        }
    }
    run.finish_checks()
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut fixtures = None;
    let mut selected = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--fixtures" => {
                i += 1;
                fixtures = args.get(i).map(PathBuf::from);
            }
            "--case" => {
                i += 1;
                selected.push(args.get(i).expect("--case requires an ID").as_str());
            }
            "--help" | "-h" => {
                println!("check_logic_candidates --fixtures DIR [--case LFF006|COUNT001|DECODE004|ADD005]");
                return;
            }
            other => panic!("unknown argument {other}"),
        }
        i += 1;
    }
    let fixtures = fixtures.expect("--fixtures DIR is required");
    assert!(
        selected
            .iter()
            .all(|id| CASES.iter().any(|(case, _, _)| id == case)),
        "unknown case ID"
    );
    let mut failed = false;
    for &(case, basename, size) in CASES
        .iter()
        .filter(|(case, _, _)| selected.is_empty() || selected.contains(case))
    {
        let report = match Run::new(&fixtures, case, basename, size) {
            Ok(mut run) => {
                let result = match case {
                    "LFF006" => latch(&mut run),
                    "COUNT001" => counter(&mut run),
                    "DECODE004" => decoder(&mut run),
                    "ADD005" => adder(&mut run),
                    _ => unreachable!(),
                };
                if let Err(error) = &result {
                    eprintln!("{case}: first failure: {error}");
                }
                failed |= result.is_err();
                run.report(result)
            }
            Err(error) => {
                eprintln!("{case}: load/setup failure: {error}");
                failed = true;
                json!({"case":case,"status":"load_failed","first_failure":error,
                    "vanilla_parity":"not_newly_tested"})
            }
        };
        println!("{report}");
    }
    if failed {
        std::process::exit(1);
    }
}
