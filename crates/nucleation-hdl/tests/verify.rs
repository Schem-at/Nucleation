//! In-sim verification against mc-tick (`--features mc-tick`): the same
//! checked-in BLIFs the verified Python pipeline ships, driven through the
//! levers with toggle-to-target discipline and compared probe-by-probe
//! against the prim-graph model.
//!
//! Python baseline (hdl2redstone.py): seg7 16/16, popcnt4 16/16, cmp4 256/256.

use nucleation_hdl::{compile_blif, verify::verify};

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
