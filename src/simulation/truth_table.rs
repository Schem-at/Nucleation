use super::MchprsWorld;
use crate::UniversalSchematic;
use mchprs_blocks::BlockPos;
use mchprs_world::World;
use std::collections::HashMap;

/// Generates a truth table for a redstone circuit
///
/// Automatically finds all levers (inputs) and redstone lamps (outputs)
/// and tests all possible input combinations.
///
/// # Example
///
/// ```ignore
/// let schematic = /* your redstone circuit */;
/// let truth_table = generate_truth_table(&schematic);
///
/// for row in truth_table {
///     println!("Input 0: {}, Output 0: {}",
///         row.get("Input 0").unwrap(),
///         row.get("Output 0").unwrap()
///     );
/// }
/// ```
pub fn generate_truth_table(schematic: &UniversalSchematic) -> Vec<HashMap<String, bool>> {
    let mut world = match MchprsWorld::new(schematic.clone()) {
        Ok(w) => w,
        Err(e) => {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::error_1(&format!("Failed to create world: {}", e).into());
            #[cfg(not(target_arch = "wasm32"))]
            eprintln!("Failed to create world: {}", e);
            return Vec::new();
        }
    };

    // Find all levers and lamps
    let (inputs, outputs) = find_inputs_and_outputs(&world);

    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("Inputs: {:?}", inputs).into());
        web_sys::console::log_1(&format!("Outputs: {:?}", outputs).into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("Inputs: {:?}", inputs);
        println!("Outputs: {:?}", outputs);
    }

    let mut truth_table = Vec::new();
    let input_combinations = generate_input_combinations(inputs.len());

    for combination in input_combinations {
        // Set lever states for this combination
        for (i, &input_pos) in inputs.iter().enumerate() {
            if combination[i] {
                world.on_use_block(input_pos);
            }
        }

        // Run simulation
        world.tick(20);
        world.flush();

        // Record results
        let mut result = HashMap::new();
        for (i, &input_pos) in inputs.iter().enumerate() {
            result.insert(format!("Input {}", i), world.get_lever_power(input_pos));
        }
        for (i, &output_pos) in outputs.iter().enumerate() {
            result.insert(format!("Output {}", i), world.is_lit(output_pos));
        }

        truth_table.push(result);

        // Reset world for next combination
        world = match MchprsWorld::new(schematic.clone()) {
            Ok(w) => w,
            Err(_) => return truth_table, // Return what we have so far
        };
    }

    truth_table
}

/// Finds all levers (inputs) and redstone lamps (outputs) in the circuit
fn find_inputs_and_outputs(world: &MchprsWorld) -> (Vec<BlockPos>, Vec<BlockPos>) {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    let dimensions = world.schematic.get_dimensions();
    for x in 0..dimensions.0 {
        for y in 0..dimensions.1 {
            for z in 0..dimensions.2 {
                let pos = BlockPos::new(x, y, z);
                let block = world.get_block(pos);
                let block_name = block.get_name();
                match block_name {
                    "lever" => inputs.push(pos),
                    "redstone_lamp" => outputs.push(pos),
                    _ => {}
                }
            }
        }
    }

    (inputs, outputs)
}

/// Generates all possible boolean combinations for n inputs
fn generate_input_combinations(num_inputs: usize) -> Vec<Vec<bool>> {
    (0..2usize.pow(num_inputs as u32))
        .map(|i| (0..num_inputs).map(|j| (i & (1 << j)) != 0).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_combinations() {
        let combos = generate_input_combinations(2);
        assert_eq!(combos.len(), 4);
        assert_eq!(combos[0], vec![false, false]);
        assert_eq!(combos[1], vec![true, false]);
        assert_eq!(combos[2], vec![false, true]);
        assert_eq!(combos[3], vec![true, true]);
    }
}
