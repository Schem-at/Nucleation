"""alu8.schem: the 8-bit 4-op ALU (ADD / SUB / AND / XOR).

Regenerated via build_alu.py --width 8: the Kogge-Stone adder's internal
g/p terms double as AND/XOR results; a B-select stage and output mux fold
the ops in.  144 cases (per-op corner cases + seeded randoms) simulated
before baking and saving.
"""
import sys
import _wrap

sys.exit(_wrap.regen(
    "build_alu.py", ["--width", "8", "--cases", "124", "--seed", "1"],
    "alu8.schem", r"ALU results correct: (\d+)/(\d+)", 144))
