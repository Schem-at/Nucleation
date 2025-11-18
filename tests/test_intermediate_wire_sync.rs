//! Test to verify that intermediate wires between components sync correctly
//!
//! This addresses the bug where only custom IO wires were updating visually,
//! but intermediate wires between components were not syncing to the schematic.

#[cfg(feature = "simulation")]
use nucleation::simulation::{BlockPos, MchprsWorld, SimulationOptions};
#[cfg(feature = "simulation")]
use nucleation::SchematicBuilder;

#[cfg(feature = "simulation")]
#[test]
fn test_intermediate_wire_visual_sync() {
    // Circuit: INPUT_WIRE -> COMPARATOR -> MIDDLE_WIRE -> COMPARATOR -> OUTPUT_WIRE
    //
    // Layout (Y=0 base, Y=1 logic):
    // [0,1,4] = input wire (custom IO, power=15)
    // [0,1,3] = comparator facing south
    // [0,1,2] = middle wire (should get power from comparator)
    // [0,1,1] = comparator facing south
    // [0,1,0] = output wire (custom IO)

    let template = r#"
# Base layer
c
c
c
c
c
# Logic layer
│
▲
│
▲
│
"#;

    let schematic = SchematicBuilder::from_template(template)
        .expect("Failed to parse template")
        .use_standard_palette()
        .build()
        .expect("Failed to build schematic");

    // Custom IO: input at [0,1,4], output at [0,1,0]
    let options = SimulationOptions {
        custom_io: vec![
            BlockPos::new(0, 1, 4), // input
            BlockPos::new(0, 1, 0), // output
        ],
        optimize: false, // Disable optimization to track all wires for visual debugging
        ..Default::default()
    };

    let mut world = MchprsWorld::with_options(schematic, options).expect("Failed to create world");

    // Set input to 15
    world.set_signal_strength(BlockPos::new(0, 1, 4), 15);
    world.flush();

    // Run simulation
    world.tick(50);

    // Sync state back to schematic
    world.sync_to_schematic();

    // Get the synced schematic
    let synced_schematic = world.get_schematic();

    // Check that ALL wires and comparators are synced, including the middle wire at [0,1,2]
    let input_wire = synced_schematic
        .get_block(0, 1, 4)
        .expect("Input wire should exist");
    let comp1 = synced_schematic
        .get_block(0, 1, 3)
        .expect("Comparator 1 should exist");
    let middle_wire = synced_schematic
        .get_block(0, 1, 2)
        .expect("Middle wire should exist");
    let comp2 = synced_schematic
        .get_block(0, 1, 1)
        .expect("Comparator 2 should exist");
    let output_wire = synced_schematic
        .get_block(0, 1, 0)
        .expect("Output wire should exist");

    println!("\n=== INTERMEDIATE WIRE SYNC TEST ===");
    println!("Input wire [0,1,4]: {:?}", input_wire);
    println!("Comparator 1 [0,1,3]: {:?}", comp1);
    println!("MIDDLE WIRE [0,1,2]: {:?}", middle_wire);
    println!("Comparator 2 [0,1,1]: {:?}", comp2);
    println!("Output wire [0,1,0]: {:?}", output_wire);

    // Parse power levels
    let input_power: u8 = input_wire
        .properties
        .get("power")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

    let middle_power: u8 = middle_wire
        .properties
        .get("power")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

    let output_power: u8 = output_wire
        .properties
        .get("power")
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

    let comp1_powered = comp1
        .properties
        .get("powered")
        .map(|p| p == "true")
        .unwrap_or(false);

    let comp2_powered = comp2
        .properties
        .get("powered")
        .map(|p| p == "true")
        .unwrap_or(false);

    println!("\nParsed values:");
    println!("  Input power: {}", input_power);
    println!("  Comparator 1 powered: {}", comp1_powered);
    println!("  MIDDLE WIRE power: {}", middle_power);
    println!("  Comparator 2 powered: {}", comp2_powered);
    println!("  Output power: {}", output_power);

    // Assertions
    assert_eq!(input_power, 15, "Input wire should have power=15");
    assert!(comp1_powered, "First comparator should be powered");
    assert!(
        middle_power > 0,
        "BUG: Middle wire should have power > 0 after sync! Got {}",
        middle_power
    );
    assert!(comp2_powered, "Second comparator should be powered");
    assert!(output_power > 0, "Output wire should have power > 0");

    println!("✅ All wires synced correctly, including intermediate wire!");
}
