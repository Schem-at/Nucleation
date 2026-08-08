"""adder4_cells.schem: the dense comparator-cell 4-bit ripple-carry adder.

Regenerated via rca_cells.py, which stamps four truth-tabled FA cells at
pitch (carry chain connects by abutment), audits structure, net-checks for
shorts, then EXHAUSTIVELY simulates all 512 input combinations (16x16x2)
before baking the rest state and saving.
"""
import sys
import _wrap

sys.exit(_wrap.regen(
    "rca_cells.py", ["--bits", "4"], "adder4_cells.schem",
    r"exhaustive: (\d+)/(\d+) correct", 512))
