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
fn sequential_blif_is_rejected_with_a_clear_error() {
    let text = ".model d\n.inputs a clk\n.outputs q\n.latch a q re clk 0\n.end\n";
    match compile_blif(text, "d").map(|_| ()) {
        Err(HdlError::Unsupported(m)) => assert!(m.contains(".latch"), "{m}"),
        other => panic!("expected Unsupported(.latch), got {other:?}"),
    }
}
