"""
Benchmark: nucleation (Python bindings) vs mcschematic
Run from project root:
    python benches/bench_python.py
"""

import time
import statistics
import sys

# ── helpers ──────────────────────────────────────────────────────────

def bench(fn, label, warmup=2, iters=10):
    """Run fn() iters times after warmup, print median/min/max."""
    for _ in range(warmup):
        fn()
    times = []
    for _ in range(iters):
        t0 = time.perf_counter()
        fn()
        times.append(time.perf_counter() - t0)
    med = statistics.median(times)
    lo, hi = min(times), max(times)
    print(f"  {label:40s}  median {fmt(med)}   [{fmt(lo)} .. {fmt(hi)}]")
    return med

def fmt(sec):
    if sec < 1e-3:
        return f"{sec*1e6:8.1f} µs"
    elif sec < 1:
        return f"{sec*1e3:8.2f} ms"
    else:
        return f"{sec:8.3f} s "

# ── nucleation benchmarks ───────────────────────────────────────────

def nucleation_available():
    try:
        import nucleation
        return True
    except ImportError:
        return False

def bench_nucleation_set_blocks(n):
    import nucleation
    s = nucleation.Schematic("bench")
    for i in range(n):
        s.set_block(i, 0, 0, "minecraft:stone")

def bench_nucleation_fill_cuboid(n):
    import nucleation
    s = nucleation.Schematic("bench")
    s.fill_cuboid(0, 0, 0, n - 1, n - 1, n - 1, "minecraft:stone")

def bench_nucleation_fill_and_export(n):
    import nucleation
    s = nucleation.Schematic("bench")
    s.fill_cuboid(0, 0, 0, n - 1, n - 1, n - 1, "minecraft:stone")
    _ = s.save_as("schematic")

# ── mcschematic benchmarks ──────────────────────────────────────────

def mcschematic_available():
    try:
        import mcschematic
        return True
    except ImportError:
        return False

def bench_mcschematic_set_blocks(n):
    import mcschematic
    s = mcschematic.MCSchematic()
    for i in range(n):
        s.setBlock((i, 0, 0), "minecraft:stone")

def bench_mcschematic_fill_cuboid(n):
    import mcschematic
    s = mcschematic.MCSchematic()
    # cuboidFilled is on the internal MCStructure, not MCSchematic
    s.getStructure().cuboidFilled("minecraft:stone", (0, 0, 0), (n - 1, n - 1, n - 1))

def bench_mcschematic_fill_and_export(n):
    import mcschematic, tempfile, os
    s = mcschematic.MCSchematic()
    s.getStructure().cuboidFilled("minecraft:stone", (0, 0, 0), (n - 1, n - 1, n - 1))
    tmp = tempfile.mkdtemp()
    s.save(tmp, "bench", mcschematic.Version.JE_1_18_2)
    # clean up
    p = os.path.join(tmp, "bench.schem")
    if os.path.exists(p):
        os.remove(p)
    os.rmdir(tmp)

# ── main ────────────────────────────────────────────────────────────

if __name__ == "__main__":
    print("=" * 72)
    print("Python Schematic Library Benchmark")
    print("=" * 72)

    has_nuc = nucleation_available()
    has_mc  = mcschematic_available()

    if not has_nuc and not has_mc:
        print("Neither nucleation nor mcschematic installed. Nothing to benchmark.")
        sys.exit(1)

    # --- set_blocks ---
    for n in [100, 1000, 10000]:
        print(f"\n── set_blocks  n={n} ──")
        if has_nuc:
            bench(lambda: bench_nucleation_set_blocks(n), f"nucleation  (n={n})")
        if has_mc:
            bench(lambda: bench_mcschematic_set_blocks(n), f"mcschematic (n={n})")

    # --- fill_cuboid ---
    for n in [10, 32, 64]:
        print(f"\n── fill_cuboid  {n}x{n}x{n} ──")
        if has_nuc:
            bench(lambda: bench_nucleation_fill_cuboid(n), f"nucleation  ({n}³)")
        if has_mc:
            bench(lambda: bench_mcschematic_fill_cuboid(n), f"mcschematic ({n}³)")

    # --- fill + export ---
    for n in [10, 32]:
        print(f"\n── fill_and_export  {n}x{n}x{n} ──")
        if has_nuc:
            bench(lambda: bench_nucleation_fill_and_export(n), f"nucleation  ({n}³)")
        if has_mc:
            bench(lambda: bench_mcschematic_fill_and_export(n), f"mcschematic ({n}³)")

    print("\n" + "=" * 72)
    print("Done.")
