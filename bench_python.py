"""
Benchmark: Nucleation (Rust/PyO3) vs mcschematic (pure Python)
Mandelbulb fractal generation with random colors at multiple sizes.
"""

import math
import random
import time
import os
import tempfile
import shutil


# ── Mandelbulb computation ──────────────────────────────────────────────────

COLORS = [
    "minecraft:red_concrete",
    "minecraft:orange_concrete",
    "minecraft:yellow_concrete",
    "minecraft:lime_concrete",
    "minecraft:green_concrete",
    "minecraft:cyan_concrete",
    "minecraft:light_blue_concrete",
    "minecraft:blue_concrete",
    "minecraft:purple_concrete",
    "minecraft:magenta_concrete",
    "minecraft:pink_concrete",
    "minecraft:white_concrete",
    "minecraft:light_gray_concrete",
    "minecraft:brown_concrete",
    "minecraft:red_glazed_terracotta",
    "minecraft:orange_glazed_terracotta",
]

POWER = 8
MAX_ITER = 12
BAILOUT = 2.0


def mandelbulb_iteration(cx, cy, cz):
    """Run Mandelbulb iteration. Returns (converged, iteration_count)."""
    x, y, z = cx, cy, cz
    for i in range(MAX_ITER):
        r = math.sqrt(x * x + y * y + z * z)
        if r > BAILOUT:
            return False, i
        theta = math.atan2(math.sqrt(x * x + y * y), z)
        phi = math.atan2(y, x)
        rn = r ** POWER
        st = math.sin(theta * POWER)
        x = rn * st * math.cos(phi * POWER) + cx
        y = rn * st * math.sin(phi * POWER) + cy
        z = rn * math.cos(theta * POWER) + cz
    return True, MAX_ITER


def precompute_mandelbulb(size):
    """Pre-compute all Mandelbulb voxels for a given grid size.
    Returns a list of (x, y, z, block_name) for converging points."""
    rng = random.Random(42)  # deterministic seed
    scale = 3.0 / size  # map [0, size) to [-1.5, 1.5)
    offset = -1.5
    blocks = []

    for gy in range(size):
        cy = gy * scale + offset
        for gz in range(size):
            cz = gz * scale + offset
            for gx in range(size):
                cx = gx * scale + offset
                converged, _ = mandelbulb_iteration(cx, cy, cz)
                if converged:
                    color = COLORS[rng.randint(0, len(COLORS) - 1)]
                    blocks.append((gx, gy, gz, color))

    return blocks


# ── Benchmark harness ───────────────────────────────────────────────────────

def bench(name, fn, iterations=None, warmup=1):
    """Run a benchmark, auto-calibrating iterations if not given."""
    for _ in range(warmup):
        fn()

    if iterations is None:
        single = time.perf_counter()
        fn()
        single = time.perf_counter() - single
        if single < 1e-7:
            single = 1e-7
        iterations = max(3, min(200, int(1.0 / single)))

    times = []
    for _ in range(iterations):
        t0 = time.perf_counter()
        fn()
        times.append(time.perf_counter() - t0)

    times.sort()
    median = times[len(times) // 2]
    best = times[0]
    return {
        "name": name,
        "median_ms": median * 1000,
        "best_ms": best * 1000,
        "iterations": iterations,
    }


def format_result(r, label=""):
    s = f"  {r['name']:<45s} median={r['median_ms']:>10.3f} ms   best={r['best_ms']:>10.3f} ms   (n={r['iterations']})"
    if label:
        s += f"  [{label}]"
    return s


def run_size(size, nucleation, mcschematic):
    """Run all benchmarks for a given Mandelbulb size."""
    print(f"\n{'─'*80}")
    print(f"  Mandelbulb {size}x{size}x{size}  (power={POWER}, max_iter={MAX_ITER})")
    print(f"{'─'*80}")

    print(f"  Pre-computing fractal...", end=" ", flush=True)
    t0 = time.perf_counter()
    voxels = precompute_mandelbulb(size)
    dt = time.perf_counter() - t0
    density = len(voxels) / size**3 * 100
    unique_colors = len(set(v[3] for v in voxels))
    print(f"{len(voxels):,} voxels ({density:.1f}% fill, {unique_colors} colors) in {dt:.2f}s")
    print()

    # Pre-group by color for batch API
    by_color = {}
    for x, y, z, color in voxels:
        by_color.setdefault(color, []).append((x, y, z))

    results = []

    # Adaptive iteration counts based on size
    if size <= 32:
        iters_write = 10
        iters_read = 10
        iters_save = 10
    elif size <= 64:
        iters_write = 5
        iters_read = 5
        iters_save = 5
    else:
        iters_write = 3
        iters_read = 3
        iters_save = 3

    # ── 1. set_block loop ────────────────────────────────────────────────
    tag = f"set_block loop ({size}^3)"
    print(f"  [1] {tag}")

    def nuc_set_block_loop():
        s = nucleation.Schematic("mandelbulb")
        for x, y, z, color in voxels:
            s.set_block(x, y, z, color)
        return s

    def mcs_set_block_loop():
        s = mcschematic.MCSchematic()
        for x, y, z, color in voxels:
            s.setBlock((x, y, z), color)
        return s

    rn = bench(tag, nuc_set_block_loop, iterations=iters_write)
    rm = bench(tag, mcs_set_block_loop, iterations=iters_write)
    results.append((rn, rm))
    print(format_result(rn, "Nucleation"))
    print(format_result(rm, "mcschematic"))
    print()

    # ── 2. set_blocks batch (grouped by color) ───────────────────────────
    tag = f"set_blocks batch ({size}^3)"
    print(f"  [2] {tag}")

    def nuc_set_blocks_batch():
        s = nucleation.Schematic("mandelbulb")
        for color, positions in by_color.items():
            s.set_blocks(positions, color)
        return s

    def mcs_set_blocks_batch():
        s = mcschematic.MCSchematic()
        for color, positions in by_color.items():
            for pos in positions:
                s.setBlock(pos, color)
        return s

    rn = bench(tag, nuc_set_blocks_batch, iterations=iters_write)
    rm = bench(tag, mcs_set_blocks_batch, iterations=iters_write)
    results.append((rn, rm))
    print(format_result(rn, "Nucleation"))
    print(format_result(rm, "mcschematic"))
    print()

    # Build reference schematics for read benchmarks
    nuc_ref = nuc_set_block_loop()
    mcs_ref = mcs_set_block_loop()

    # ── 3. get_block loop (read back all voxel positions) ────────────────
    tag = f"get_block loop ({size}^3)"
    print(f"  [3] {tag}")

    positions_only = [(x, y, z) for x, y, z, _ in voxels]

    def nuc_get_block_loop():
        count = 0
        for x, y, z in positions_only:
            if nuc_ref.get_block(x, y, z) is not None:
                count += 1
        return count

    def mcs_get_block_loop():
        count = 0
        for x, y, z in positions_only:
            if mcs_ref.getBlockStateAt((x, y, z)) != "minecraft:air":
                count += 1
        return count

    rn = bench(tag, nuc_get_block_loop, iterations=iters_read)
    rm = bench(tag, mcs_get_block_loop, iterations=iters_read)
    results.append((rn, rm))
    print(format_result(rn, "Nucleation"))
    print(format_result(rm, "mcschematic"))
    print()

    # ── 4. get_blocks batch ──────────────────────────────────────────────
    tag = f"get_blocks batch ({size}^3)"
    print(f"  [4] {tag}")

    def nuc_get_blocks_batch():
        return nuc_ref.get_blocks(positions_only)

    def mcs_get_blocks_batch():
        return [mcs_ref.getBlockStateAt(p) for p in positions_only]

    rn = bench(tag, nuc_get_blocks_batch, iterations=iters_read)
    rm = bench(tag, mcs_get_blocks_batch, iterations=iters_read)
    results.append((rn, rm))
    print(format_result(rn, "Nucleation"))
    print(format_result(rm, "mcschematic"))
    print()

    # ── 5. block_count ───────────────────────────────────────────────────
    tag = f"block_count ({size}^3)"
    print(f"  [5] {tag}")

    def nuc_block_count():
        return nuc_ref.block_count

    def mcs_block_count():
        count = 0
        for y in range(size):
            for z in range(size):
                for x in range(size):
                    if mcs_ref.getBlockStateAt((x, y, z)) != "minecraft:air":
                        count += 1
        return count

    rn = bench(tag, nuc_block_count)
    rm = bench(tag, mcs_block_count, iterations=iters_read)
    results.append((rn, rm))
    print(format_result(rn, "Nucleation"))
    print(format_result(rm, "mcschematic"))
    print()

    # ── 6. save .schem ───────────────────────────────────────────────────
    tmpdir = tempfile.mkdtemp()
    tag = f"save .schem ({size}^3)"
    print(f"  [6] {tag}")

    def nuc_save():
        data = nuc_ref.to_schematic()
        with open(os.path.join(tmpdir, "nuc.schem"), "wb") as f:
            f.write(data)

    def mcs_save():
        mcs_ref.save(tmpdir, "mcs", mcschematic.Version.JE_1_20)

    rn = bench(tag, nuc_save, iterations=iters_save)
    rm = bench(tag, mcs_save, iterations=iters_save)
    results.append((rn, rm))
    print(format_result(rn, "Nucleation"))
    print(format_result(rm, "mcschematic"))
    print()

    # ── 7. load .schem ───────────────────────────────────────────────────
    tag = f"load .schem ({size}^3)"
    print(f"  [7] {tag}")

    nuc_path = os.path.join(tmpdir, "nuc_load.schem")
    nuc_data = nuc_ref.to_schematic()
    with open(nuc_path, "wb") as f:
        f.write(nuc_data)

    mcs_ref.save(tmpdir, "mcs_load", mcschematic.Version.JE_1_20)
    mcs_path = os.path.join(tmpdir, "mcs_load.schem")

    def nuc_load():
        with open(nuc_path, "rb") as f:
            data = f.read()
        s = nucleation.Schematic("load")
        s.from_schematic(data)
        return s

    def mcs_load():
        return mcschematic.MCSchematic(mcs_path)

    rn = bench(tag, nuc_load, iterations=iters_save)
    rm = bench(tag, mcs_load, iterations=iters_save)
    results.append((rn, rm))
    print(format_result(rn, "Nucleation"))
    print(format_result(rm, "mcschematic"))
    print()

    # ── 8. flip_x ────────────────────────────────────────────────────────
    tag = f"flip_x ({size}^3)"
    print(f"  [8] {tag}")

    def nuc_flip_x():
        s = nuc_set_block_loop()
        s.flip_x()
        return s

    rn = bench(tag, nuc_flip_x, iterations=iters_write)
    rm = {"name": tag, "median_ms": float("nan"), "best_ms": float("nan"), "iterations": 0}
    results.append((rn, rm))
    print(format_result(rn, "Nucleation"))
    print(f"  {'(not available in mcschematic)':<45s}")
    print()

    # ── 9. rotate_y(90) ─────────────────────────────────────────────────
    tag = f"rotate_y 90 ({size}^3)"
    print(f"  [9] {tag}")

    def nuc_rotate():
        s = nuc_set_block_loop()
        s.rotate_y(90)
        return s

    rn = bench(tag, nuc_rotate, iterations=iters_write)
    rm = {"name": tag, "median_ms": float("nan"), "best_ms": float("nan"), "iterations": 0}
    results.append((rn, rm))
    print(format_result(rn, "Nucleation"))
    print(f"  {'(not available in mcschematic)':<45s}")
    print()

    shutil.rmtree(tmpdir, ignore_errors=True)
    return results


def main():
    import nucleation
    import mcschematic

    SIZES = [32, 64, 128]

    print(f"{'='*80}")
    print(f"  Nucleation vs mcschematic  --  Mandelbulb Fractal Benchmark")
    print(f"  Power: {POWER}   Max iterations: {MAX_ITER}   Colors: {len(COLORS)}")
    print(f"  Sizes: {', '.join(f'{s}^3' for s in SIZES)}")
    print(f"{'='*80}")

    all_results = {}  # size -> [(rn, rm), ...]

    for size in SIZES:
        all_results[size] = run_size(size, nucleation, mcschematic)

    # ── Grand Summary ────────────────────────────────────────────────────
    print(f"\n{'='*80}")
    print(f"  GRAND SUMMARY  --  median ms, lower is better")
    print(f"{'='*80}")

    # Column headers
    operations = [
        "set_block loop",
        "set_blocks batch",
        "get_block loop",
        "get_blocks batch",
        "block_count",
        "save .schem",
        "load .schem",
        "flip_x",
        "rotate_y 90",
    ]

    for op_idx, op_name in enumerate(operations):
        print(f"\n  {op_name}:")
        print(f"    {'Size':<10s}  {'Nucleation':>12s}  {'mcschematic':>12s}  {'Speedup':>10s}")
        print(f"    {'─'*10}  {'─'*12}  {'─'*12}  {'─'*10}")

        for size in SIZES:
            results = all_results[size]
            if op_idx >= len(results):
                continue
            rn, rm = results[op_idx]
            nuc_str = f"{rn['median_ms']:.3f}" if rn["iterations"] > 0 else "N/A"
            mcs_str = f"{rm['median_ms']:.3f}" if rm["iterations"] > 0 else "N/A"

            if rn["iterations"] > 0 and rm["iterations"] > 0:
                speedup = rm["median_ms"] / rn["median_ms"]
                speedup_str = f"{speedup:.1f}x"
            else:
                speedup_str = "--"

            print(f"    {size:>3d}^3      {nuc_str:>12s}  {mcs_str:>12s}  {speedup_str:>10s}")

    print()


if __name__ == "__main__":
    main()
