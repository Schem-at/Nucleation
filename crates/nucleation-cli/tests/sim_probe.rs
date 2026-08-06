//! Diagnostic: what stops an arbitrary build from simulating?
//!
//! ```sh
//! MC_SIM_FILE=path/to/build.litematic \
//!   cargo test -p nucleation-cli --test sim_probe -- --ignored --nocapture
//! ```
use mc_test::mc_tick::{Pos, Structure};
use nucleation::formats::gametest::to_gametest_snbt;

#[test]
#[ignore = "diagnostic, run by hand"]
fn what_refuses() {
    let file = std::env::var("MC_SIM_FILE").expect("set MC_SIM_FILE");
    let bytes = std::fs::read(&file).expect("readable");
    let manager = nucleation::formats::manager::get_manager();
    let schematic = manager.lock().unwrap().read(&bytes).expect("imports");
    eprintln!(
        "dims {:?}  blocks {}",
        schematic.get_dimensions(),
        schematic.total_blocks()
    );
    let snbt = to_gametest_snbt(&schematic);
    let structure = Structure::parse(&snbt).expect("engine parses");
    eprintln!(
        "palette {} states, {} placed",
        structure.palette.len(),
        structure.blocks.len()
    );
    match mc_test::try_build_sim(
        &structure,
        Pos::new(0, 0, 0),
        mc_test::SettleMode::Placement,
        &[],
        &[],
        None,
        "probe",
    ) {
        Ok(_) => eprintln!("SIM STARTS CLEAN"),
        Err(report) => {
            let list = report.rsplit("simulated as nothing: ").next().unwrap_or("");
            let names: std::collections::BTreeSet<&str> = list
                .split(", ")
                .map(|d| d.split('[').next().unwrap_or(d).trim())
                .collect();
            eprintln!("REFUSED — {} distinct block(s):", names.len());
            for n in &names {
                eprintln!("  {n}");
            }
        }
    }
    panic!("probe output above");
}
