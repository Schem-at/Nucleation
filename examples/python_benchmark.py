import time
import nucleation
import mcschematic
from nucleation import Schematic, BuildingTool, Shape, Brush

def benchmark_nucleation_set_block(size):
    print(f"Benchmarking Nucleation set_block ({size}x{size}x{size})...")
    schem = Schematic("benchmark")
    
    # Pre-expand if possible? Nucleation doesn't expose ensure_bounds directly on Schematic Python class yet, 
    # but fill_cuboid does it internally. Here we test raw set_block.
    
    start_time = time.time()
    for x in range(size):
        for y in range(size):
            for z in range(size):
                schem.set_block(x, y, z, "minecraft:stone")
    end_time = time.time()
    
    duration = end_time - start_time
    total_blocks = size * size * size
    mbps = (total_blocks / duration) / 1_000_000
    print(f"  -> {duration:.4f} s")
    print(f"  -> {mbps:.2f} M blocks/sec")
    return mbps

def benchmark_mcschematic_set_block(size):
    print(f"Benchmarking mcschematic setBlock ({size}x{size}x{size})...")
    schem = mcschematic.MCSchematic()
    
    start_time = time.time()
    for x in range(size):
        for y in range(size):
            for z in range(size):
                schem.setBlock((x, y, z), "minecraft:stone")
    end_time = time.time()
    
    duration = end_time - start_time
    total_blocks = size * size * size
    mbps = (total_blocks / duration) / 1_000_000
    print(f"  -> {duration:.4f} s")
    print(f"  -> {mbps:.2f} M blocks/sec")
    return mbps

def benchmark_nucleation_fill(size):
    print(f"Benchmarking Nucleation fill_cuboid ({size}x{size}x{size})...")
    schem = Schematic("benchmark")
    
    start_time = time.time()
    schem.fill_cuboid(0, 0, 0, size-1, size-1, size-1, "minecraft:stone")
    end_time = time.time()
    
    duration = end_time - start_time
    total_blocks = size * size * size
    mbps = (total_blocks / duration) / 1_000_000
    print(f"  -> {duration:.4f} s")
    print(f"  -> {mbps:.2f} M blocks/sec")
    return mbps

def benchmark_mcschematic_fill(size):
    print(f"Benchmarking mcschematic Simulated Fill (loop) ({size}x{size}x{size})...")
    schem = mcschematic.MCSchematic()
    
    start_time = time.time()
    # mcschematic doesn't have native fill, using loop
    for x in range(size):
        for y in range(size):
            for z in range(size):
                schem.setBlock((x, y, z), "minecraft:stone")
    end_time = time.time()
    
    duration = end_time - start_time
    total_blocks = size * size * size
    mbps = (total_blocks / duration) / 1_000_000
    print(f"  -> {duration:.4f} s")
    print(f"  -> {mbps:.2f} M blocks/sec")
    return mbps

def run_benchmarks():
    sizes = [32, 64, 128] # Small, Medium, Large
    
    print("=== Python Benchmark: Nucleation vs mcschematic ===\n")
    
    for size in sizes:
        print(f"\n--- Size: {size}^3 ({size**3:,} blocks) ---")
        
        # Nucleation set_block
        benchmark_nucleation_set_block(size)
        
        # mcschematic setBlock (skip for large sizes if it's too slow)
        if size <= 64:
            benchmark_mcschematic_set_block(size)
        else:
            print(f"Skipping mcschematic setBlock for size {size} (likely too slow)")

        # Nucleation fill
        benchmark_nucleation_fill(size)
        
        # mcschematic fill
        benchmark_mcschematic_fill(size)

if __name__ == "__main__":
    run_benchmarks()
