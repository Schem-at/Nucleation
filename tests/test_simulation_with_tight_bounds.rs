#[cfg(feature = "simulation")]
use nucleation::{BlockState, UniversalSchematic};

#[cfg(feature = "simulation")]
use nucleation::simulation::MchprsWorld;

#[cfg(feature = "simulation")]
#[test]
fn test_simulation_with_barrels_and_tight_bounds() {
    println!("\n==============================================");
    println!("Test: Simulation with Barrels and Tight Bounds");
    println!("==============================================");

    let mut schematic = UniversalSchematic::new("test".to_string());

    // This triggers region expansion
    schematic
        .set_block_from_string(0, 1, 0, "minecraft:barrel[facing=north]{signal=1}")
        .expect("Failed to set first barrel");

    let dims1 = schematic.get_dimensions();
    println!("After first barrel at (0, 1, 0): dimensions = {:?}", dims1);

    schematic
        .set_block_from_string(0, -1, 0, "minecraft:barrel[facing=north]{signal=2}")
        .expect("Failed to set second barrel");

    let dims2 = schematic.get_dimensions();
    println!(
        "After second barrel at (0, -1, 0): dimensions = {:?}",
        dims2
    );

    // Check block entities
    let entities = schematic.get_block_entities_as_list();
    println!("\nBlock entities: {}", entities.len());
    for be in &entities {
        println!("  {} at {:?}", be.id, be.position);
    }

    // Check blocks
    let block1 = schematic.get_block(0, 1, 0);
    let block2 = schematic.get_block(0, -1, 0);
    println!("\nBlocks:");
    println!("  At (0, 1, 0): {:?}", block1.map(|b| &b.name));
    println!("  At (0, -1, 0): {:?}", block2.map(|b| &b.name));

    // Check allocated vs tight dimensions
    println!("\nDimension info:");
    println!(
        "  Allocated: {:?}",
        schematic.default_region.get_dimensions()
    );
    println!("  Tight: {:?}", schematic.get_tight_dimensions());

    // Now try to create simulation - this is where it crashes
    println!("\nAttempting to create simulation world...");
    let result = std::panic::catch_unwind(|| MchprsWorld::new(schematic.clone()));

    match result {
        Ok(Ok(_world)) => {
            println!("✓ Simulation created successfully!");
        }
        Ok(Err(e)) => {
            println!("✗ Simulation creation failed with error: {}", e);
            panic!("Simulation creation failed: {}", e);
        }
        Err(panic_info) => {
            println!("✗ Simulation creation panicked!");
            if let Some(s) = panic_info.downcast_ref::<&str>() {
                println!("   Panic message: {}", s);
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                println!("   Panic message: {}", s);
            }
            panic!("Simulation creation panicked - likely block entity position mismatch");
        }
    }
}

#[cfg(feature = "simulation")]
#[test]
fn test_simulation_with_simple_barrel() {
    println!("\n========================================");
    println!("Test: Simulation with Simple Barrel");
    println!("========================================");

    let mut schematic = UniversalSchematic::new("test".to_string());

    // Simple case - no region expansion
    schematic
        .set_block_from_string(0, 0, 0, "minecraft:barrel[facing=north]{signal=5}")
        .expect("Failed to set barrel");

    println!("Dimensions: {:?}", schematic.get_dimensions());

    let entities = schematic.get_block_entities_as_list();
    println!("Block entities: {}", entities.len());
    for be in &entities {
        println!("  {} at {:?}", be.id, be.position);
    }

    println!("\nAttempting to create simulation world...");
    let world = MchprsWorld::new(schematic).expect("Failed to create simulation");
    println!("✓ Simulation created successfully!");
}

#[cfg(feature = "simulation")]
#[test]
fn test_simulation_after_export_reload() {
    println!("\n==============================================");
    println!("Test: Simulation After Export/Reload");
    println!("==============================================");

    use nucleation::schematic;

    let mut schematic = UniversalSchematic::new("test".to_string());

    schematic
        .set_block_from_string(0, 1, 0, "minecraft:barrel[facing=north]{signal=1}")
        .expect("Failed to set first barrel");
    schematic
        .set_block_from_string(0, -1, 0, "minecraft:barrel[facing=north]{signal=2}")
        .expect("Failed to set second barrel");

    println!("Original dimensions: {:?}", schematic.get_dimensions());

    // Export and reload (this uses to_compact)
    let bytes = schematic::to_schematic(&schematic).expect("Failed to export");
    let reloaded = schematic::from_schematic(&bytes).expect("Failed to reload");

    println!("Reloaded dimensions: {:?}", reloaded.get_dimensions());

    let entities = reloaded.get_block_entities_as_list();
    println!("Block entities after reload: {}", entities.len());
    for be in &entities {
        println!("  {} at {:?}", be.id, be.position);
    }

    // Check if blocks exist at expected positions
    let block1 = reloaded.get_block(0, 1, 0);
    let block2 = reloaded.get_block(0, -1, 0);
    println!("\nBlocks after reload:");
    println!("  At (0, 1, 0): {:?}", block1.map(|b| &b.name));
    println!("  At (0, -1, 0): {:?}", block2.map(|b| &b.name));

    // Try to create simulation from reloaded schematic
    println!("\nAttempting to create simulation from reloaded schematic...");
    let result = std::panic::catch_unwind(|| MchprsWorld::new(reloaded.clone()));

    match result {
        Ok(Ok(_world)) => {
            println!("✓ Simulation created successfully from reloaded schematic!");
        }
        Ok(Err(e)) => {
            println!("✗ Simulation creation failed: {}", e);
            panic!("Simulation creation failed: {}", e);
        }
        Err(_) => {
            println!("✗ Simulation creation panicked!");
            panic!("Simulation creation panicked from reloaded schematic");
        }
    }
}
