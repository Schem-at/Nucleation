"""kogge_stone_32bit.schem: the flagship 32-bit Kogge-Stone prefix adder.

Regenerated via build_ppa.py --width 32: PLA-compiled prefix stages,
channel-routed rails, structural audit + net-short check, then 54 32-bit
addition cases (47 seeded randoms + 7 corner cases) simulated on the lever
bank before baking and saving.
"""
import sys
import _wrap

sys.exit(_wrap.regen(
    "build_ppa.py", ["--width", "32", "--cases", "47", "--seed", "1"],
    "kogge_stone_32bit.schem", r"sums correct: (\d+)/(\d+)", 54))
