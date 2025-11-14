//! Test to reproduce custom IO adjacent block bug
//!
//! Bug: Custom IO redstone wire directly adjacent to a solid block
//! does not power that block or components on top of it.

#[cfg(feature = "simulation")]
#[test]
fn test_custom_io_adjacent_to_block_bug() {
    use mchprs_blocks::BlockPos;
    use nucleation::simulation::{MchprsWorld, SimulationOptions};
    use nucleation::{BlockState, UniversalSchematic};

    // Create a simple 1x3x1 schematic:
    // Y=2: Redstone torch (should turn OFF when wire is powered)
    // Y=1: Concrete block
    // Y=0: Redstone wire (custom IO)

    let mut schematic = UniversalSchematic::new("custom_io_bug_test".to_string());

    // Y=0: Redstone wire at (0, 0, 0) - will be custom IO
    schematic.set_block(
        0,
        0,
        0,
        BlockState::new("minecraft:redstone_wire".to_string())
            .with_property("power".to_string(), "0".to_string())
            .with_property("north".to_string(), "none".to_string())
            .with_property("south".to_string(), "none".to_string())
            .with_property("east".to_string(), "none".to_string())
            .with_property("west".to_string(), "none".to_string()),
    );

    // Y=1: Concrete block at (0, 1, 0)
    schematic.set_block(
        0,
        1,
        0,
        BlockState::new("minecraft:gray_concrete".to_string()),
    );

    // Y=2: Redstone torch at (0, 2, 0) - starts lit
    schematic.set_block(
        0,
        2,
        0,
        BlockState::new("minecraft:redstone_torch".to_string())
            .with_property("lit".to_string(), "true".to_string()),
    );

    // Create simulation with custom IO at the wire position
    let wire_pos = BlockPos::new(0, 0, 0);
    let torch_pos = BlockPos::new(0, 2, 0);

    let options = SimulationOptions {
        optimize: true,
        io_only: false,
        custom_io: vec![wire_pos],
    };

    let mut world = MchprsWorld::with_options(schematic, options).expect("Failed to create world");

    // Initial state: torch should be lit (wire is unpowered)
    let initial_torch_power = world.get_signal_strength(torch_pos);
    println!(
        "Initial torch signal: {} (should be 0 when lit)",
        initial_torch_power
    );

    // Set custom IO wire to power 15
    world.set_signal_strength(wire_pos, 15);

    // Verify wire is powered
    let wire_power = world.get_signal_strength(wire_pos);
    println!("Wire power after set: {}", wire_power);
    assert_eq!(wire_power, 15, "Wire should be powered to 15");

    // Tick the simulation to propagate the signal
    world.tick(10);

    // Flush and sync to get updated block states
    world.flush();
    world.sync_to_schematic();

    // Check torch state - it should be OFF (signal strength 0 when torch is off)
    let final_torch_power = world.get_signal_strength(torch_pos);
    println!(
        "Final torch signal: {} (should be 0 if torch turned OFF)",
        final_torch_power
    );

    // BUG: This assertion will FAIL because the torch stays lit
    // Expected: torch turns OFF (signal 0) when wire powers the concrete
    // Actual: torch stays lit (signal remains non-zero)
    assert_eq!(
        final_torch_power, 0,
        "BUG REPRODUCED: Torch should turn OFF when custom IO wire powers adjacent concrete block"
    );
}

#[cfg(feature = "simulation")]
#[test]
fn test_custom_io_with_gap_workaround() {
    use mchprs_blocks::BlockPos;
    use nucleation::simulation::{MchprsWorld, SimulationOptions};
    use nucleation::{BlockState, UniversalSchematic};

    // Workaround: Add a gap between custom IO and concrete
    // Y=3: Redstone torch
    // Y=2: Concrete block
    // Y=1: Regular redstone wire (NOT custom IO)
    // Y=0: Custom IO redstone wire

    let mut schematic = UniversalSchematic::new("custom_io_workaround_test".to_string());

    // Y=0: Custom IO wire
    schematic.set_block(
        0,
        0,
        0,
        BlockState::new("minecraft:redstone_wire".to_string())
            .with_property("power".to_string(), "0".to_string()),
    );

    // Y=1: Regular wire (NOT custom IO)
    schematic.set_block(
        0,
        1,
        0,
        BlockState::new("minecraft:redstone_wire".to_string())
            .with_property("power".to_string(), "0".to_string()),
    );

    // Y=2: Concrete
    schematic.set_block(
        0,
        2,
        0,
        BlockState::new("minecraft:gray_concrete".to_string()),
    );

    // Y=3: Torch
    schematic.set_block(
        0,
        3,
        0,
        BlockState::new("minecraft:redstone_torch".to_string())
            .with_property("lit".to_string(), "true".to_string()),
    );

    let custom_io_pos = BlockPos::new(0, 0, 0);
    let torch_pos = BlockPos::new(0, 3, 0);

    let options = SimulationOptions {
        optimize: true,
        io_only: false,
        custom_io: vec![custom_io_pos], // Only bottom wire is custom IO
    };

    let mut world = MchprsWorld::with_options(schematic, options).expect("Failed to create world");

    // Set custom IO wire to power 15
    world.set_signal_strength(custom_io_pos, 15);

    // Tick to propagate through the gap
    world.tick(10);
    world.flush();

    // With the gap, the signal should propagate correctly
    let torch_power = world.get_signal_strength(torch_pos);
    println!("Torch power with gap workaround: {}", torch_power);

    // This should PASS - the workaround works
    assert_eq!(
        torch_power, 0,
        "With gap workaround, torch should turn OFF correctly"
    );
}
