//! Core pipeline tests, no simulator: the compiled prim graph must agree with
//! an independent evaluation of the raw BLIF truth tables on every input
//! assignment, and the geometry must come out placed and probed.

use std::collections::HashMap;

use nucleation_hdl::blif::{parse_blif, Blif};
use nucleation_hdl::{compile_blif, HdlError};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/data/{name}.blif", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).expect("checked-in BLIF fixture")
}

/// Evaluate the raw BLIF directly — row matching over the file's own covers,
/// sharing no code with the compiler under test.
fn eval_blif(blif: &Blif, inputs: &HashMap<String, u8>) -> HashMap<String, u8> {
    let mut val = inputs.clone();
    let mut progress = true;
    while progress {
        progress = false;
        for (name, node) in &blif.nodes {
            if val.contains_key(name) || !node.inputs.iter().all(|i| val.contains_key(i)) {
                continue;
            }
            let v = if node.rows.is_empty() {
                0
            } else {
                let onset = node.rows[0].1 == "1";
                let hit = node.rows.iter().any(|(pat, _)| {
                    pat.chars()
                        .enumerate()
                        .all(|(i, c)| c == '-' || (c as u8 - b'0') == val[&node.inputs[i]])
                });
                u8::from(hit == onset)
            };
            val.insert(name.clone(), v);
            progress = true;
        }
    }
    val
}

fn check_against_blif(name: &str) {
    let text = fixture(name);
    let blif = parse_blif(&text).unwrap();
    let compiled = compile_blif(&text, name).unwrap();
    assert_eq!(compiled.inputs, blif.inputs);
    let n = blif.inputs.len();
    for case in 0..1u64 << n {
        let bits: Vec<u8> = (0..n).map(|i| ((case >> i) & 1) as u8).collect();
        let pi: HashMap<String, u8> = blif
            .inputs
            .iter()
            .cloned()
            .zip(bits.iter().copied())
            .collect();
        let want = eval_blif(&blif, &pi);
        for (po, got) in compiled.eval_outputs(&bits) {
            assert_eq!(
                got, want[&po],
                "{name}: output {po} disagrees with the BLIF at inputs {case:0n$b}"
            );
        }
    }
}

#[test]
fn seg7_model_matches_blif_exhaustively() {
    check_against_blif("seg7");
}

#[test]
fn popcnt4_model_matches_blif_exhaustively() {
    check_against_blif("popcnt4");
}

#[test]
fn cmp4_model_matches_blif_exhaustively() {
    check_against_blif("cmp4");
}

#[test]
fn geometry_is_placed_probed_and_levered() {
    for name in ["seg7", "popcnt4", "cmp4"] {
        let c = compile_blif(&fixture(name), name).unwrap();
        assert!(c.stats.blocks > 0, "{name}: no cells");
        assert_eq!(c.levers.len(), c.inputs.len(), "{name}: one lever per PI");
        for (po, v) in &c.outputs {
            if let nucleation_hdl::Value::Vid(vid) = v {
                assert!(c.probes.contains_key(vid), "{name}: {po} has no probe");
            }
        }
        // every lever cell really is a lever, every probe cell really is dust
        for (_, (x, y, z)) in &c.levers {
            assert!(c.build.cells[&(*x, *y, *z)].contains("lever"), "{name}");
        }
        for (s, (x, y, z)) in &c.probes {
            assert!(
                c.build.cells[&(*x, *y, *z)].contains("redstone_wire"),
                "{name}: probe {s} is not dust"
            );
        }
        let report = c.report_json();
        assert!(report.contains("\"levers\""), "{name}: report shape");
    }
}

#[test]
fn hierarchy_directives_are_rejected_with_a_clear_error() {
    let text = ".model d\n.inputs a\n.outputs q\n.subckt foo A=a Q=q\n.end\n";
    match compile_blif(text, "d").map(|_| ()) {
        Err(HdlError::Unsupported(m)) => assert!(m.contains(".subckt"), "{m}"),
        other => panic!("expected Unsupported(.subckt), got {other:?}"),
    }
}

#[test]
fn non_rising_edge_latches_are_rejected() {
    let text = ".model d\n.inputs a clk\n.outputs q\n.latch a q fe clk 0\n.end\n";
    match compile_blif(text, "d").map(|_| ()) {
        Err(HdlError::Unsupported(m)) => assert!(m.contains("rising-edge"), "{m}"),
        other => panic!("expected Unsupported(rising-edge), got {other:?}"),
    }
}

#[test]
fn multiple_clock_domains_are_rejected() {
    let text = ".model d\n.inputs a c1 c2\n.outputs q r\n\
                .latch a q re c1 0\n.latch a r re c2 0\n.end\n";
    match compile_blif(text, "d").map(|_| ()) {
        Err(HdlError::Unsupported(m)) => assert!(m.contains("clock domains"), "{m}"),
        other => panic!("expected Unsupported(clock domains), got {other:?}"),
    }
}

/// Independent clocked evaluation of the raw BLIF: latch outputs come from
/// `state`, next state reads each latch's input net after the comb settle.
fn step_blif(
    blif: &Blif,
    pi: &HashMap<String, u8>,
    state: &[u8],
) -> (HashMap<String, u8>, Vec<u8>) {
    let mut env = pi.clone();
    for (l, s) in blif.latches.iter().zip(state) {
        env.insert(l.output.clone(), *s);
    }
    let val = eval_blif(blif, &env);
    let next: Vec<u8> = blif.latches.iter().map(|l| val[&l.input]).collect();
    (val, next)
}

/// The compiled sequential model must agree with the independent BLIF
/// stepper on outputs AND next-state at every step of `cases`.
fn check_seq_against_blif(name: &str, cases: &[u64]) {
    let text = fixture(name);
    let blif = parse_blif(&text).unwrap();
    let compiled = compile_blif(&text, name).unwrap();
    assert_eq!(compiled.latches.len(), blif.latches.len(), "{name}");
    assert!(compiled.clock.is_some(), "{name}: no clock");
    let mut state: Vec<u8> = blif.latches.iter().map(|l| l.init_bit()).collect();
    let mut cstate: Vec<u8> = compiled.latches.iter().map(|l| l.init).collect();
    assert_eq!(state, cstate, "{name}: baked init");
    let n = compiled.inputs.len();
    for (si, &case) in cases.iter().enumerate() {
        let bits: Vec<u8> = (0..n).map(|i| ((case >> i) & 1) as u8).collect();
        let pi: HashMap<String, u8> = compiled
            .inputs
            .iter()
            .cloned()
            .zip(bits.iter().copied())
            .collect();
        let (want, next) = step_blif(&blif, &pi, &state);
        let val = compiled.seq_eval(&bits, &cstate);
        for (po, got) in compiled.outputs_from(&val) {
            assert_eq!(got, want[&po], "{name} step {si}: output {po}");
        }
        state = next;
        cstate = compiled.latch_next(&val);
        assert_eq!(cstate, state, "{name} step {si}: next state");
    }
}

#[test]
fn counter4_model_counts_like_the_blif() {
    check_seq_against_blif("counter4", &vec![0u64; 40]);
}

#[test]
fn fsm_model_matches_the_blif_on_a_long_input_tape() {
    // every x pattern of 6 steps — 64 tapes woven into one long run
    let tape: Vec<u64> = (0..64u64)
        .flat_map(|w| (0..6).map(move |b| (w >> b) & 1))
        .collect();
    check_seq_against_blif("fsm", &tape);
}

#[test]
fn toggle1_model_starts_at_one_and_toggles_on_enable() {
    check_seq_against_blif("toggle1", &[1, 1, 1, 0, 1, 1, 0, 1, 0, 0, 1, 1]);
}

#[test]
fn uart_tx_model_matches_the_blif_over_a_frame() {
    // start pulse with data 0xA5, then idle inputs for the whole frame
    let mut cases = vec![1u64 | (0xA5 << 1)];
    cases.extend(std::iter::repeat(0xA5 << 1).take(25));
    check_seq_against_blif("uart_tx", &cases);
}

#[test]
fn seq_geometry_has_a_clock_lever_dff_ports_and_q_rail_probes() {
    for name in ["counter4", "fsm", "toggle1", "uart_tx"] {
        let c = compile_blif(&fixture(name), name).unwrap();
        let clock = c.clock.as_ref().expect("clock");
        let (x, y, z) = clock.lever;
        assert!(
            c.build.cells[&(x, y, z)].contains("lever"),
            "{name}: clock lever"
        );
        assert_eq!(
            c.levers.len(),
            c.inputs.len(),
            "{name}: one lever per real PI"
        );
        for (k, l) in c.latches.iter().enumerate() {
            assert!(
                c.probes.contains_key(&l.q_rail),
                "{name}: latch {k} q rail probed"
            );
            let q = l.q_port;
            assert!(
                c.build.cells[&(q.0, q.1, q.2)].contains("redstone_wire"),
                "{name}: latch {k} Q port is dust"
            );
            // the slave repeater is baked at the declared initial state
            let slave = (q.0 - 12 + 4, 1, q.2);
            let s = &c.build.cells[&(slave.0, slave.1, slave.2)];
            assert!(
                s.contains(&format!("powered={}", l.init == 1)) && s.contains("locked=true"),
                "{name}: latch {k} slave baked at Q={} ({s})",
                l.init
            );
        }
    }
}
