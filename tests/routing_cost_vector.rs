//! The corpus chain's COST VECTOR, as a measured before/after datum.
//!
//! `tests/design_routability.rs` answers "how many buses route". That number
//! alone is gameable: a router that routes more buses by sprawling every one of
//! them across the workspace has not improved anything. The five-term
//! [`nucleation::design::BusCostVector`] of the real corpus chain
//! (`ADD007.sum -> BINTOBCD001.bin`) is the QUALITY half of the pair, and it is
//! the number that proved the mechanism-accurate `interferes()` predicate gave
//! zero gain: routability held at 41/45 and the vector came back
//! byte-identical, which is what re-ranked the roadmap toward negotiated
//! routing.
//!
//! Printed as `RR|cost_vector|...` for `tools/routing_report.sh`. Deliberately
//! NOT asserted against a golden value: the vector is allowed to move when a
//! routing change is a real trade, and a hard-coded number would only teach
//! whoever hits it to update the constant. The report shows the delta; a human
//! decides whether the trade was worth it.

#![cfg(feature = "routing")]

use nucleation::design::{BusState, BusStyle, Design};
use nucleation::UniversalSchematic;

fn load(file: &str) -> Option<UniversalSchematic> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("computational_schematics/enhanced")
        .join(file);
    nucleation::formats::schematic::from_schematic(&std::fs::read(path).ok()?).ok()
}

#[test]
fn corpus_chain_cost_vector() {
    let (Some(add), Some(bcd)) = (
        load("ADD007_8bit_cca_matt_enhanced.schem"),
        load("BINTOBCD001_8bit_comb_binary_to_bcd_enhanced.schem"),
    ) else {
        eprintln!("enhanced corpus unavailable; skipping");
        println!("RR|cost_vector|SKIPPED");
        return;
    };
    // The exact placement the 2026-08-09 report was measured at; changing it
    // would silently invalidate every recorded vector.
    let mut d = Design::new("chain");
    d.add_cell("add", add).unwrap();
    d.add_cell("bcd", bcd).unwrap();
    d.place("u0", "add", (0, 0, 0), 0).unwrap();
    d.place("u1", "bcd", (60, -2, 40), 0).unwrap();
    d.promote_input("u1", "bin").unwrap();
    let st = d
        .route_bus("sum_to_bin", "u0.sum", &["u1.bin"], vec![], BusStyle::default())
        .unwrap();
    assert_eq!(st, BusState::Routed, "{:?}", d.bus_state("sum_to_bin"));
    let layer = d.bus("sum_to_bin").unwrap();
    let v = d.bus_cost(layer);
    println!(
        "RR|cost_vector|{},{},{},{},{}",
        v.length, v.delay_rt, v.skew_rt, v.coherence, v.footprint
    );
    println!("corpus chain cost vector = {}", v.to_json());
    // The route has to be REAL: a zero-length "route" would otherwise report a
    // beautiful cost vector.
    assert!(v.length > 0 && v.footprint > 0, "empty route: {}", v.to_json());
}
