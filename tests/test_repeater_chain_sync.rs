//! Test to verify that wires between repeaters sync correctly with optimize=false

#[cfg(feature = "simulation")]
use nucleation::simulation::{BlockPos, MchprsWorld, SimulationOptions};
#[cfg(feature = "simulation")]
use nucleation::SchematicBuilder;

#[cfg(feature = "simulation")]
#[test]
fn test_repeater_chain_visual_sync() {
    // Circuit: INPUT_WIRE -> REPEATER -> WIRE -> REPEATER -> WIRE -> REPEATER -> OUTPUT_WIRE
    let template = r#"
# Base layer
c
c
c
c
c
c
c
c
c
c
c
# Logic layer
│
↑
│
↑
│
↑
│
↑
│
↑
│
"#;

    let schematic = SchematicBuilder::from_template(template)
        .expect("Failed to parse template")
        .use_standard_palette()
        .build()
        .expect("Failed to build schematic");

    // Custom IO: input at [0,1,10], output at [0,1,0]
    let options = SimulationOptions {
        custom_io: vec![
            BlockPos::new(0, 1, 10), // input
            BlockPos::new(0, 1, 0),  // output
        ],
        optimize: false, // Disable optimization to track all wires
        ..Default::default()
    };

    let mut world = MchprsWorld::with_options(schematic, options).expect("Failed to create world");

    // Set input to 15
    world.set_signal_strength(BlockPos::new(0, 1, 10), 15);
    world.flush();

    // Run simulation
    world.tick(50);

    // Sync state back to schematic
    world.sync_to_schematic();

    // Get the synced schematic
    let synced_schematic = world.get_schematic();

    println!("\n=== REPEATER CHAIN SYNC TEST ===");

    // Check all blocks
    for y in 0..=2 {
        for z in 0..=10 {
            if let Some(block) = synced_schematic.get_block(0, y, z) {
                if y == 1 && block.name.contains("wire") {
                    let power: u8 = block
                        .properties
                        .get("power")
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(0);
                    println!("  [0,1,{}]: {} power={}", z, block.name, power);
                } else if y == 1 && block.name.contains("repeater") {
                    let powered = block
                        .properties
                        .get("powered")
                        .map(|p| p == "true")
                        .unwrap_or(false);
                    println!("  [0,1,{}]: {} powered={}", z, block.name, powered);
                }
            }
        }
    }

    // Check specific intermediate wires
    let wire_2 = synced_schematic
        .get_block(0, 1, 2)
        .expect("Wire at [0,1,2] should exist");
    let wire_4 = synced_schematic
        .get_block(0, 1, 4)
        .expect("Wire at [0,1,4] should exist");
    let wire_6 = synced_schematic
        .get_block(0, 1, 6)
        .expect("Wire at [0,1,6] should exist");
    let wire_8 = synced_schematic
        .get_block(0, 1, 8)
        .expect("Wire at [0,1,8] should exist");

    let power_2: u8 = wire_2
        .properties
        .get("power")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);
    let power_4: u8 = wire_4
        .properties
        .get("power")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);
    let power_6: u8 = wire_6
        .properties
        .get("power")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);
    let power_8: u8 = wire_8
        .properties
        .get("power")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

    println!("\nIntermediate wire powers:");
    println!("  [0,1,2]: {}", power_2);
    println!("  [0,1,4]: {}", power_4);
    println!("  [0,1,6]: {}", power_6);
    println!("  [0,1,8]: {}", power_8);

    // Assertions
    assert!(
        power_2 > 0,
        "BUG: Wire at [0,1,2] between repeaters should have power > 0! Got {}",
        power_2
    );
    assert!(
        power_4 > 0,
        "BUG: Wire at [0,1,4] between repeaters should have power > 0! Got {}",
        power_4
    );
    assert!(
        power_6 > 0,
        "BUG: Wire at [0,1,6] between repeaters should have power > 0! Got {}",
        power_6
    );
    assert!(
        power_8 > 0,
        "BUG: Wire at [0,1,8] between repeaters should have power > 0! Got {}",
        power_8
    );

    println!("✅ All intermediate wires between repeaters synced correctly!");
}
