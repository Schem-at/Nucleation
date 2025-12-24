use nucleation::building::{
    BlockPalette, BuildingTool, Cuboid, InterpolationSpace, LinearGradientBrush, SolidBrush, Sphere,
};
use nucleation::{BlockState, UniversalSchematic};
use rand::Rng;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

fn run_benchmark(
    name: &str,
    size_desc: &str,
    block_count: usize,
    mut operation: impl FnMut(),
    csv_file: &mut std::fs::File,
) {
    println!("Running {} ({}) ...", name, size_desc);
    let start = Instant::now();
    operation();
    let duration = start.elapsed();
    let ms = duration.as_millis();
    let secs = duration.as_secs_f64();
    let mbps = (block_count as f64 / 1_000_000.0) / secs;

    println!("  -> {} ms", ms);
    println!("  -> {:.2} M blocks/sec", mbps);

    // Timestamp, Operation, Size, Time(ms), MBPS, Notes
    let timestamp = chrono::Local::now().to_rfc3339();
    writeln!(
        csv_file,
        "{},{},{},{},{:.2},",
        timestamp, name, size_desc, ms, mbps
    )
    .expect("Failed to write to CSV");
}

fn main() {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("building_benchmarks.csv")
        .expect("Failed to open benchmark CSV");

    // Header check (if empty file)
    if file.metadata().unwrap().len() == 0 {
        writeln!(file, "Timestamp,Operation,Size,Time_ms,MBPS,Notes").unwrap();
    }

    let sizes = [
        ("Small", 10),
        ("Medium", 50),
        ("Large", 80),
        ("ExtraLarge", 200),
    ];

    // Pre-create palettes
    let concrete_palette = Arc::new(BlockPalette::new_concrete());
    let solid_palette = Arc::new(BlockPalette::new_solid());
    let builder_palette = Arc::new(
        BlockPalette::builder()
            .full_blocks_only()
            .exclude_transparent()
            .exclude_tile_entities()
            .survival_obtainable_only()
            .build(),
    );

    for (size_name, s) in sizes {
        let total_blocks = s * s * s;
        let total_blocks_usize = total_blocks as usize;

        // 0. Raw set_block Loop (Baseline)
        run_benchmark(
            "Raw set_block",
            size_name,
            total_blocks_usize,
            || {
                let mut schematic = UniversalSchematic::new("bench".to_string());
                // Pre-expand to separate allocation cost from set_block cost
                schematic.ensure_bounds((0, 0, 0), (s, s, s));

                let block = BlockState::new("minecraft:stone".to_string());
                for x in 0..s {
                    for y in 0..s {
                        for z in 0..s {
                            schematic.set_block(x, y, z, &block.clone());
                        }
                    }
                }
            },
            &mut file,
        );

        // 0.1 Random set_block (Simulate noise/sparse filling)
        // We do fewer iterations for random to avoid super long waits, but we still calculate rate based on iterations
        let random_iters = if s > 50 {
            50_000
        } else {
            total_blocks_usize / 2
        };
        run_benchmark(
            "Random set_block",
            size_name,
            random_iters,
            || {
                let mut schematic = UniversalSchematic::new("bench".to_string());
                schematic.ensure_bounds((0, 0, 0), (s, s, s));

                let block = BlockState::new("minecraft:stone".to_string());
                let mut rng = rand::thread_rng();

                for _ in 0..random_iters {
                    let x = rng.gen_range(0..s);
                    let y = rng.gen_range(0..s);
                    let z = rng.gen_range(0..s);
                    schematic.set_block(x, y, z, &block.clone());
                }
            },
            &mut file,
        );

        // 1. Solid Fill (Cuboid)
        run_benchmark(
            "Solid Fill",
            size_name,
            total_blocks_usize,
            || {
                let mut schematic = UniversalSchematic::new("bench".to_string());
                let mut tool = BuildingTool::new(&mut schematic);
                let shape = Cuboid::new((0, 0, 0), (s, s, s));
                let brush = SolidBrush::new(BlockState::new("minecraft:stone".to_string()));
                tool.fill(&shape, &brush);
            },
            &mut file,
        );

        // 2. Linear Gradient Fill (Cuboid)
        run_benchmark(
            "Gradient Fill",
            size_name,
            total_blocks_usize,
            || {
                let mut schematic = UniversalSchematic::new("bench".to_string());
                let mut tool = BuildingTool::new(&mut schematic);
                let shape = Cuboid::new((0, 0, 0), (s, s, s));
                let brush =
                    LinearGradientBrush::new((0, 0, 0), (0, 0, 0), (s, s, s), (255, 255, 255))
                        .with_palette(concrete_palette.clone())
                        .with_space(InterpolationSpace::Oklab);

                tool.fill(&shape, &brush);
            },
            &mut file,
        );

        // 3. Sphere Fill (Solid)
        // Note: Sphere volume is approx 0.52 * s^3, but for simplicity we'll just track operation time vs bounding box volume or calculate approximate blocks.
        // Let's approximate actual filled blocks for MBPS
        let radius = (s as f64) / 2.0;
        let sphere_vol = (4.0 / 3.0 * std::f64::consts::PI * radius.powi(3)) as usize;

        run_benchmark(
            "Sphere Fill",
            size_name,
            sphere_vol,
            || {
                let mut schematic = UniversalSchematic::new("bench".to_string());
                let mut tool = BuildingTool::new(&mut schematic);
                let center = (s / 2, s / 2, s / 2);
                let shape = Sphere::new(center, radius);
                let brush = SolidBrush::new(BlockState::new("minecraft:stone".to_string()));
                tool.fill(&shape, &brush);
            },
            &mut file,
        );

        // 4. Complex Filter Palette Fill (Solid Palette)
        run_benchmark(
            "Solid Palette Gradient",
            size_name,
            total_blocks_usize,
            || {
                let mut schematic = UniversalSchematic::new("bench".to_string());
                let mut tool = BuildingTool::new(&mut schematic);
                let shape = Cuboid::new((0, 0, 0), (s, s, s));
                let brush =
                    LinearGradientBrush::new((0, 0, 0), (0, 0, 0), (s, s, s), (255, 255, 255))
                        .with_palette(solid_palette.clone())
                        .with_space(InterpolationSpace::Rgb);

                tool.fill(&shape, &brush);
            },
            &mut file,
        );

        // 5. Builder Palette Fill
        run_benchmark(
            "Builder Palette Gradient",
            size_name,
            total_blocks_usize,
            || {
                let mut schematic = UniversalSchematic::new("bench".to_string());
                let mut tool = BuildingTool::new(&mut schematic);
                let shape = Cuboid::new((0, 0, 0), (s, s, s));
                let brush =
                    LinearGradientBrush::new((0, 0, 0), (0, 0, 0), (s, s, s), (255, 255, 255))
                        .with_palette(builder_palette.clone())
                        .with_space(InterpolationSpace::Rgb);

                tool.fill(&shape, &brush);
            },
            &mut file,
        );
    }
}
