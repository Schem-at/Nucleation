"""The material model: glass appears only where a diagonal must survive."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))))       # redstone-eda/ for materials.py

import materials as m

# A 1-y step passes power UP always, but DOWN only if the upper dust's support
# conducts -- so a descending line needs a CONDUCTOR under its upper cell.
assert m.step_conducts("minecraft:stone", downhill=True)
assert not m.step_conducts(m.GLASS, downhill=True)

# A block above dust only matters when that dust is the lower end of a diagonal
# in use; on a straight run a solid cap is harmless (so straight runs use SOLID).
assert m.cap_is_harmful("minecraft:stone", dust_uses_diagonal_here=True)
assert not m.cap_is_harmful("minecraft:stone", dust_uses_diagonal_here=False)

# pick_support() derives the material from the constraints and refuses geometry
# that is over-constrained (must conduct AND must not) -- the build is wrong.
print("insulating support ->", m.pick_support(need_insulator=True))
print("conducting support ->", m.pick_support(need_conductor=True))
print("OK 10_materials")
