"""Verify the vertical transport primitive: the torch ladder.

Facts already verified: a dead-end dust weakly powers the block it points
into, a torch on top of a powered block turns off, and a lit torch strongly
powers the block directly above it.  Chain those and a signal climbs a 1x1
column at 2 blocks per torch, inverting at each torch -- so an EVEN torch
count is non-inverting.

Also probed here, because the router must know them:
  * exit clearance -- does the top dust leak diagonally into neighbours?
  * shaft spacing  -- can two ladders stand 2 apart without shorting?
  * delay          -- measured ticks per torch.
"""
import rs
from rs import STONE, DUST, LEVER_OFF

b = rs.Build("probe_vert")


def ladder(x, z, y0, torches):
    """Input: dust at (x-1, y0, z) dead-ending into the base block.
    Output: dust on top, at (x, y0 + 2*torches + 1, z).  Inverts if odd."""
    b.stone(x - 2, y0 - 1, z)
    b.stone(x - 1, y0 - 1, z)
    b.put(x - 1, y0, z, DUST)                  # entry dust, dead-ends into base
    y = y0
    for _ in range(torches):
        b.stone(x, y, z)                       # block
        b.put(x, y + 1, z, rs.TORCH)           # torch on top: inverts
        y += 2
    b.put(x, y, z, STONE)                      # cap block (strongly powered)
    b.put(x, y + 1, z, DUST)                   # output dust
    return (x, y + 1, z)


# A: 2 torches (rise 4) -- non-inverting
b.stone(0, 0, 0); b.put(0, 1, 0, LEVER_OFF)
b.stone(1, 0, 0); b.put(1, 1, 0, DUST)
outA = ladder(3, 0, 1, 2)

# B: 4 torches (rise 8) -- non-inverting
b.stone(0, 0, 6); b.put(0, 1, 6, LEVER_OFF)
b.stone(1, 0, 6); b.put(1, 1, 6, DUST)
outB = ladder(3, 6, 1, 4)

# C: two ladders 2 apart in x -- do they interfere?
b.stone(0, 0, 12); b.put(0, 1, 12, LEVER_OFF)
b.stone(1, 0, 12); b.put(1, 1, 12, DUST)
outC1 = ladder(3, 12, 1, 2)
b.stone(0, 0, 14); b.put(0, 1, 14, LEVER_OFF)   # separate driver, kept OFF
b.stone(1, 0, 14); b.put(1, 1, 14, DUST)
outC2 = ladder(5, 14, 1, 2)                      # x=5: 2 from x=3

# D: exit into a horizontal run on the cap level (what a plane rail sees)
b.stone(0, 0, 20); b.put(0, 1, 20, LEVER_OFF)
b.stone(1, 0, 20); b.put(1, 1, 20, DUST)
outD = ladder(3, 20, 1, 2)
x, y, z = outD
for k in range(1, 4):
    b.stone(x + k, y - 1, z)
    b.put(x + k, y, z, DUST)

sim = b.sim()
lv = rs.Levers(sim, [(0, 1, 0), (0, 1, 6), (0, 1, 12), (0, 1, 14), (0, 1, 20)])

for bits in ([0, 0, 0, 0, 0], [1, 1, 1, 0, 1]):
    lv.set(bits)
    print("levers", bits)
    print("   A out(2 torches)", sim.power(*outA),
          "  B out(4 torches)", sim.power(*outB))
    print("   C1", sim.power(*outC1), " C2 (driver off)", sim.power(*outC2))
    print("   D end of top run", sim.power(outD[0] + 3, outD[1], outD[2]))

# delay: measured ticks for the 4-torch ladder to settle after a toggle
t0 = sim.sim.tick_count()
lv.set([1, 0, 1, 0, 1])
print("B ladder toggle settle:", sim.sim.tick_count() - t0, "game ticks (4 torches)")
