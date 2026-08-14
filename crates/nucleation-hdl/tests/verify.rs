//! In-sim verification against mc-tick (`--features mc-tick`): the same
//! checked-in BLIFs the verified Python pipeline ships, driven through the
//! levers with toggle-to-target discipline and compared probe-by-probe
//! against the prim-graph model.
//!
//! Python baseline (hdl2redstone.py): seg7 16/16, popcnt4 16/16, cmp4 256/256.

use nucleation_hdl::{compile_blif, verify::verify, verify::verify_clocked};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/data/{name}.blif", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).expect("checked-in BLIF fixture")
}

fn run(name: &str, cases: Vec<u64>) {
    let c = compile_blif(&fixture(name), name).unwrap();
    let n = cases.len();
    let report = verify(&c, &cases, 400).unwrap();
    assert!(
        report.pass(),
        "{name}: outputs correct {}/{n}, disagreeing signals: {:?}",
        report.outputs_ok,
        report.sig_bad
    );
}

#[test]
fn seg7_is_exhaustively_correct_in_sim() {
    run("seg7", (0..16).collect());
}

#[test]
fn popcnt4_is_exhaustively_correct_in_sim() {
    run("popcnt4", (0..16).collect());
}

#[test]
fn cmp4_is_exhaustively_correct_in_sim() {
    run("cmp4", (0..256).collect());
}

#[test]
fn cmp4_yosys_0_33_is_exhaustively_correct_in_sim() {
    run("cmp4_yosys_0_33", (0..256).collect());
}

// ---- sequential: fixed-tick clocked protocol vs the stepped model --------

fn run_clocked(name: &str, cases: Vec<u64>) -> nucleation_hdl::verify::ClockedReport {
    let c = compile_blif(&fixture(name), name).unwrap();
    let report = verify_clocked(&c, &cases, 40, 800).unwrap();
    assert!(
        report.pass(),
        "{name}: init_ok={} steps {}/{} setup {}gt edge {}gt: {:?}",
        report.init_ok,
        report.steps_ok,
        report.steps,
        report.measured_setup_gt,
        report.measured_edge_gt,
        report.mismatches
    );
    println!(
        "{name}: {} steps, measured setup {} gt, edge->settled {} gt, min period {} gt",
        report.steps,
        report.measured_setup_gt,
        report.measured_edge_gt,
        report.measured_min_period_gt()
    );
    report
}

/// counter4: no data inputs — 24 rising edges must count 1..24 mod 16,
/// checked step-by-step against the model (exact, deterministic).
#[test]
fn counter4_counts_24_steps_in_sim() {
    run_clocked("counter4", vec![0; 24]);
}

/// fsm ("11" detector): 30 steps over a tape that exercises every
/// transition, incl. runs of 1s, lone 1s and back-to-back resets.
#[test]
fn fsm_detects_sequences_over_30_steps_in_sim() {
    let tape: Vec<u64> = [
        1, 1, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1,
    ]
    .to_vec();
    assert_eq!(tape.len(), 30);
    run_clocked("fsm", tape);
}

/// toggle1 starts at the BAKED Q=1 (init-by-construction) and toggles only
/// when en is high — the init check inside verify_clocked is the proof the
/// non-zero initial state deployed correctly.
#[test]
fn toggle1_boots_at_one_and_toggles_in_sim() {
    run_clocked("toggle1", vec![1, 1, 1, 0, 1, 1, 0, 1, 0, 0, 1, 1]);
}

/// STRETCH — uart_tx: one 8N1 frame of 0xA5 at divider 2, bit-by-bit.
/// The in-sim run is compared against the model every step; the model's tx
/// tape is then asserted to spell the frame, so the sim provably emitted
/// start, LSB-first data and stop bits, each held for 2 clocks.
#[test]
fn uart_tx_serializes_a_frame_bit_by_bit_in_sim() {
    let name = "uart_tx";
    let c = compile_blif(&fixture(name), name).unwrap();
    let mut cases = vec![1u64 | (0xA5 << 1)];
    cases.extend(std::iter::repeat(0xA5 << 1).take(21));

    // the model's own tx/busy tape across the driven steps
    let n = c.inputs.len();
    let mut state: Vec<u8> = c.latches.iter().map(|l| l.init).collect();
    let mut tx_tape = Vec::new();
    for &case in &cases {
        let bits: Vec<u8> = (0..n).map(|i| ((case >> i) & 1) as u8).collect();
        let val = c.seq_eval(&bits, &state);
        state = c.latch_next(&val);
        let after = c.seq_eval(&bits, &state);
        let po: std::collections::HashMap<String, u8> =
            c.outputs_from(&after).into_iter().collect();
        tx_tape.push((po["tx"], po["busy"]));
    }
    // frame shape: start 0,0 then A5 LSB-first (1,0,1,0,0,1,0,1) x2 each,
    // then stop 1 — busy high throughout, idle after
    let bits_expected: Vec<u8> = [0u8, 1, 0, 1, 0, 0, 1, 0, 1, 1]
        .iter()
        .flat_map(|&b| [b, b])
        .collect();
    for (i, &want) in bits_expected.iter().enumerate() {
        assert_eq!(tx_tape[i].0, want, "model tx bit {i}");
        assert_eq!(tx_tape[i].1, 1, "model busy during frame at {i}");
    }
    assert_eq!(tx_tape[20].1, 0, "idle after the frame");
    assert_eq!(tx_tape[20].0, 1, "tx idles high");

    // and the sim matches that model on every step
    let report = verify_clocked(&c, &cases, 40, 800).unwrap();
    assert!(
        report.pass(),
        "{name}: init_ok={} steps {}/{}: {:?}",
        report.init_ok,
        report.steps_ok,
        report.steps,
        report.mismatches
    );
    println!(
        "{name}: {} steps, setup {} gt, edge {} gt",
        report.steps, report.measured_setup_gt, report.measured_edge_gt
    );
}
