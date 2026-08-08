//! Minimal oracle sanity: a lever driving a dust line, toggled and read back.
//! Run: cargo run -p nucleation-hdl --features mc-tick --example lever_probe

fn main() {
    let mut b = nucleation_hdl::Build::default();
    for x in 0..4 {
        b.force(x, 0, 0, "minecraft:stone");
    }
    b.force(0, 1, 0, nucleation_hdl::pla::LEVER_OFF);
    for x in 1..4 {
        b.force(x, 1, 0, nucleation_hdl::pla::DUST);
    }
    let mut sim = nucleation_hdl::verify::simulate(&b, 400).expect("wire");
    println!("lever: {}", sim.block(0, 1, 0));
    println!("dust3: {}", sim.block(3, 1, 0));
    sim.use_block(0, 1, 0);
    let quiet = sim.settle(400);
    println!("after use (quiet={quiet}):");
    println!("lever: {}", sim.block(0, 1, 0));
    println!("dust3: {} on={}", sim.block(3, 1, 0), sim.on(3, 1, 0));
}
