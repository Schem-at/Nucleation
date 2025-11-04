// examples/universal_schematic_overhead.rs
use nucleation::{BlockState, Region, UniversalSchematic};
use std::collections::HashMap;
use std::time::Instant;

// Simulate your optimized UniversalSchematic
struct OptimizedUniversalSchematic {
    inner: UniversalSchematic,
    block_cache: HashMap<String, BlockState>,
}

impl OptimizedUniversalSchematic {
    fn new() -> Self {
        Self {
            inner: UniversalSchematic::new("test".to_string()),
            block_cache: HashMap::new(),
        }
    }

    fn set_block_str(&mut self, x: i32, y: i32, z: i32, block_name: &str) -> bool {
        let block_state = match self.block_cache.get(block_name) {
            Some(cached) => cached.clone(),
            None => {
                let new_block = BlockState::new(block_name.to_string());
                self.block_cache
                    .insert(block_name.to_string(), new_block.clone());
                new_block
            }
        };
        self.inner.set_block(x, y, z, block_state)
    }

    fn pre_allocate_for_cube(&mut self, size: i32) {
        let margin = 16;
        self.inner.default_region = Region::new(
            "Main".to_string(),
            (-margin, -margin, -margin),
            (size + 2 * margin, size + 2 * margin, size + 2 * margin),
        );
    }
}

fn main() {
    println!("=== UniversalSchematic Overhead Analysis ===\n");

    for &size in &[10, 20, 30] {
        println!("Testing {}³ = {} blocks", size, size * size * size);

        // 1. Direct Region (baseline)
        let start = Instant::now();
        let mut region = Region::new("Test".to_string(), (0, 0, 0), (1, 1, 1));
        let stone = BlockState::new("minecraft:stone".to_string());
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    region.set_block(x, y, z, stone.clone());
                }
            }
        }
        let region_time = start.elapsed();

        // 2. Direct Region (pre-allocated)
        let start = Instant::now();
        let margin = 16;
        let mut region_prealloc = Region::new(
            "Test".to_string(),
            (-margin, -margin, -margin),
            (size + 2 * margin, size + 2 * margin, size + 2 * margin),
        );
        let stone_prealloc = BlockState::new("minecraft:stone".to_string());
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    region_prealloc.set_block(x, y, z, stone_prealloc.clone());
                }
            }
        }
        let region_prealloc_time = start.elapsed();

        // 3. UniversalSchematic (current - slow)
        let start = Instant::now();
        let mut schematic = UniversalSchematic::new("test".to_string());
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    // This is what your Python binding currently does
                    let stone = BlockState::new("minecraft:stone".to_string());
                    schematic.set_block(x, y, z, stone);
                }
            }
        }
        let universal_slow_time = start.elapsed();

        // 4. UniversalSchematic (cached blocks)
        let start = Instant::now();
        let mut schematic_cached = OptimizedUniversalSchematic::new();
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    schematic_cached.set_block_str(x, y, z, "minecraft:stone");
                }
            }
        }
        let universal_cached_time = start.elapsed();

        // 5. UniversalSchematic (cached + pre-allocated)
        let start = Instant::now();
        let mut schematic_optimized = OptimizedUniversalSchematic::new();
        schematic_optimized.pre_allocate_for_cube(size);
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    schematic_optimized.set_block_str(x, y, z, "minecraft:stone");
                }
            }
        }
        let universal_optimized_time = start.elapsed();

        // 6. Python MCSchematic style (HashMap)
        let start = Instant::now();
        let mut blocks: HashMap<(i32, i32, i32), usize> = HashMap::new();
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    blocks.insert((x, y, z), 1);
                }
            }
        }
        let python_style_time = start.elapsed();

        // Results
        println!(
            "  Direct Region (cached):               {:>8.1}ms",
            region_time.as_secs_f64() * 1000.0
        );
        println!(
            "  Direct Region (pre-allocated):        {:>8.1}ms ({:.1}x)",
            region_prealloc_time.as_secs_f64() * 1000.0,
            region_time.as_secs_f64() / region_prealloc_time.as_secs_f64()
        );
        println!(
            "  UniversalSchematic (current/slow):     {:>8.1}ms ({:.1}x slower than Region)",
            universal_slow_time.as_secs_f64() * 1000.0,
            universal_slow_time.as_secs_f64() / region_time.as_secs_f64()
        );
        println!(
            "  UniversalSchematic (cached):           {:>8.1}ms ({:.1}x)",
            universal_cached_time.as_secs_f64() * 1000.0,
            universal_slow_time.as_secs_f64() / universal_cached_time.as_secs_f64()
        );
        println!(
            "  UniversalSchematic (optimized):        {:>8.1}ms ({:.1}x)",
            universal_optimized_time.as_secs_f64() * 1000.0,
            universal_slow_time.as_secs_f64() / universal_optimized_time.as_secs_f64()
        );
        println!(
            "  Python MCSchematic style:              {:>8.1}ms",
            python_style_time.as_secs_f64() * 1000.0
        );

        // Overhead analysis
        let overhead_current = universal_slow_time.as_secs_f64() / region_time.as_secs_f64();
        let overhead_optimized =
            universal_optimized_time.as_secs_f64() / region_prealloc_time.as_secs_f64();

        println!(
            "  📊 UniversalSchematic overhead (current): {:.1}x",
            overhead_current
        );
        println!(
            "  📊 UniversalSchematic overhead (optimized): {:.1}x",
            overhead_optimized
        );

        if universal_optimized_time < python_style_time {
            println!(
                "  🎉 Optimized beats Python style by {:.1}x!",
                python_style_time.as_secs_f64() / universal_optimized_time.as_secs_f64()
            );
        } else {
            println!(
                "  ⚠️  Python style is {:.1}x faster",
                universal_optimized_time.as_secs_f64() / python_style_time.as_secs_f64()
            );
        }
        println!();
    }

    // Summary with recommendations
    println!("=== Summary & Recommendations ===");
    println!("1. Block State Caching: Essential for reducing string allocation overhead");
    println!("2. Pre-allocation: Critical for avoiding expansion costs");
    println!("3. UniversalSchematic adds minimal overhead when optimized");
    println!("4. For small cubes, Python's HashMap approach has advantages");
    println!("5. For larger cubes, the optimized approach should scale better");

    // Test scaling with a larger cube
    println!("\n=== Scaling Test (50³ = 125,000 blocks) ===");
    let size = 50;

    let start = Instant::now();
    let mut schematic_optimized = OptimizedUniversalSchematic::new();
    schematic_optimized.pre_allocate_for_cube(size);
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                schematic_optimized.set_block_str(x, y, z, "minecraft:stone");
            }
        }
    }
    let optimized_large_time = start.elapsed();

    let start = Instant::now();
    let mut blocks: HashMap<(i32, i32, i32), usize> = HashMap::new();
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                blocks.insert((x, y, z), 1);
            }
        }
    }
    let python_large_time = start.elapsed();

    println!(
        "Optimized Nucleation: {:>8.1}ms",
        optimized_large_time.as_secs_f64() * 1000.0
    );
    println!(
        "Python style:         {:>8.1}ms",
        python_large_time.as_secs_f64() * 1000.0
    );

    if optimized_large_time < python_large_time {
        println!(
            "🎉 Nucleation wins at scale by {:.1}x!",
            python_large_time.as_secs_f64() / optimized_large_time.as_secs_f64()
        );
    } else {
        println!(
            "📈 Python style scales better by {:.1}x",
            optimized_large_time.as_secs_f64() / python_large_time.as_secs_f64()
        );
    }
}
