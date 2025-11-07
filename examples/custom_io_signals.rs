//! Example demonstrating custom IO signal injection and monitoring
//!
//! This example shows how to use the custom IO feature to inject and monitor
//! signal strengths at specific positions in a redstone circuit.
//!
//! Run with: cargo run --example custom_io_signals --features simulation

#[cfg(feature = "simulation")]
fn main() {
    use nucleation::simulation::{BlockPos, MchprsWorld, SimulationOptions};
    use nucleation::{BlockState, UniversalSchematic};

    // Create a simple redstone circuit
    let mut schematic = UniversalSchematic::new("Custom IO Demo".to_string());

    // Base layer
    for x in 0..10 {
        schematic.set_block(x, 0, 0, BlockState::new("minecraft:stone".to_string()));
    }

    // Redstone wire
    for x in 1..9 {
        schematic.set_block_str(
            x,
            1,
            0,
            "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]",
        );
    }

    // Lamp at the end
    schematic.set_block_str(9, 1, 0, "minecraft:redstone_lamp[lit=false]");

    println!(
        "Created schematic with dimensions: {:?}",
        schematic.get_dimensions()
    );

    // Configure custom IO nodes at specific positions
    let probe_pos1 = BlockPos::new(3, 1, 0);
    let probe_pos2 = BlockPos::new(6, 1, 0);

    let options = SimulationOptions {
        optimize: true,
        io_only: false,
        custom_io: vec![probe_pos1, probe_pos2],
    };

    // Create simulation world with custom IO
    let mut world =
        MchprsWorld::with_options(schematic, options).expect("Failed to create simulation world");

    println!("\n=== Testing Custom IO Signal Injection ===\n");

    // Inject signal at first probe point
    println!("Injecting signal strength 15 at position (3,1,0)");
    world.set_signal_strength(probe_pos1, 15);
    world.tick(5);
    world.flush();

    // Read signals at both positions
    let strength1 = world.get_signal_strength(probe_pos1);
    let strength2 = world.get_signal_strength(probe_pos2);

    println!("Signal at (3,1,0): {}", strength1);
    println!("Signal at (6,1,0): {}", strength2);

    // Check if lamp is lit
    let lamp_pos = BlockPos::new(9, 1, 0);
    println!("Lamp is lit: {}", world.is_lit(lamp_pos));

    // Try different signal strengths
    println!("\n=== Testing Different Signal Strengths ===\n");

    for strength in [5, 10, 15, 0] {
        println!("Setting signal to {} at (3,1,0)", strength);
        world.set_signal_strength(probe_pos1, strength);
        world.tick(5);
        world.flush();

        let read_strength = world.get_signal_strength(probe_pos1);
        let lamp_lit = world.is_lit(lamp_pos);

        println!("  → Read back: {}", read_strength);
        println!("  → Lamp status: {}", if lamp_lit { "ON" } else { "OFF" });
    }

    // Multiple simultaneous signals
    println!("\n=== Testing Multiple Simultaneous Signals ===\n");

    world.set_signal_strength(probe_pos1, 8);
    world.set_signal_strength(probe_pos2, 12);
    world.tick(5);
    world.flush();

    println!(
        "Signal at (3,1,0): {}",
        world.get_signal_strength(probe_pos1)
    );
    println!(
        "Signal at (6,1,0): {}",
        world.get_signal_strength(probe_pos2)
    );

    // Get redstone power levels (different from signal strength)
    let power1 = world.get_redstone_power(probe_pos1);
    let power2 = world.get_redstone_power(probe_pos2);

    println!("Redstone power at (3,1,0): {}", power1);
    println!("Redstone power at (6,1,0): {}", power2);

    println!("\n=== Custom IO Demo Complete ===");
}

#[cfg(not(feature = "simulation"))]
fn main() {
    eprintln!("This example requires the 'simulation' feature.");
    eprintln!("Run with: cargo run --example custom_io_signals --features simulation");
}
