"""Verify comparator semantics in mc-tick before designing cells on them.

Claimed vanilla behaviour to confirm:
  subtract mode: out = max(back - side, 0)
  compare  mode: out = back if back >= side else 0
  side inputs read from dust (and repeaters) at the two side positions
  delay: 1 redstone tick

Then the three-comparator half adder:
  xor  = (a - b) OR (b - a)          two subtract comparators, merged dust
  and_ = a - (a xor b)               one subtract comparator
"""
import rs
from rs import DUST, LEVER_OFF

COMP = "minecraft:comparator[facing=%s,mode=%s,powered=false]"
rs.EXTRA_STATES += ";" + ";".join(
    COMP % (f, m) for f in ("north", "south", "east", "west")
    for m in ("compare", "subtract"))

b = rs.Build("probe_comp")


def lane(z, x0, x1, y=1, lever=False):
    for x in range(x0, x1 + 1):
        b.stone(x, 0, z)
        b.put(x, y, z, LEVER_OFF if (lever and x == x0) else DUST)


# ---- A: subtract, back=lever line, side=lever line from the south ---------
# comparator at (4,1,0) facing west (input from east? -- facing semantics are
# what we are here to learn: probe BOTH facings and read which one conducts)
lane(0, 0, 3, lever=True)                 # back line, west of comparator
b.stone(4, 0, 0)
b.put(4, 1, 0, COMP % ("east", "subtract"))
lane(0, 5, 7)                             # output line east
lane(2, 4, 6, lever=True)                 # side line approaching from south...
b.stone(4, 0, 1)
b.put(4, 1, 1, DUST)                      # ...turning up to the comparator side

# ---- B: same but facing=west --------------------------------------------
lane(6, 0, 3, lever=True)
b.stone(4, 0, 6)
b.put(4, 1, 6, COMP % ("west", "subtract"))
lane(6, 5, 7)
lane(8, 4, 6, lever=True)
b.stone(4, 0, 7)
b.put(4, 1, 7, DUST)

# ---- C: compare mode, both facings --------------------------------------
lane(12, 0, 3, lever=True)
b.stone(4, 0, 12)
b.put(4, 1, 12, COMP % ("east", "compare"))
lane(12, 5, 7)
lane(14, 4, 6, lever=True)
b.stone(4, 0, 13)
b.put(4, 1, 13, DUST)

sim = b.sim()
LV = [(0, 1, 0), (4, 1, 2), (0, 1, 6), (4, 1, 8), (0, 1, 12), (4, 1, 14)]
lv = rs.Levers(sim, LV)

print("back  side | A(sub,fac=E) B(sub,fac=W) C(cmp,fac=E)")
for back, side in ((0, 0), (1, 0), (0, 1), (1, 1)):
    lv.set([back, side, back, side, back, side])
    print("  %d    %d   |     %2d          %2d          %2d"
          % (back, side, sim.power(6, 1, 0), sim.power(6, 1, 6), sim.power(6, 1, 12)))
