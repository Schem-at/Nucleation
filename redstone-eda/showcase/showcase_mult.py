"""mult4x4_stacked.schem: the 4x4 multiplier as four stacked planes.

Regenerated via mult4.py: plane 0 computes the 16 partial products, planes
1-3 are Kogge-Stone accumulator rows; inter-plane nets are maze-routed in 3D
(dust, stairs, torch-ladder climbs).  Exhaustively simulated: all 256 A*B
products checked before baking and saving.
"""
import re
import sys
import _wrap

sys.exit(_wrap.regen(
    "mult4.py", [], "mult4x4_stacked.schem",
    r"multiplier: (\d+)/(\d+) correct", 256))
