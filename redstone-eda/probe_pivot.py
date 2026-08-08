"""Probe the facts the pivot tiles depend on (mc-tick, golden model):

THE POINTING LAW (result): dust weak-powers only the blocks it POINTS
into -- its connection axes (dust, levers, repeaters count; one connection
extends to the opposite side; a true dot powers all sides).  Probes:

  A. dot drive      -- a lever-adjacent dust with no dust connections next
                       to a station entry block: fires (the lever gives it
                       the axis; the dead end extends it forward).
  B. corner drive   -- dust arriving along +Z, entry block along +X (the
                       dust's one connection is Z): does NOT fire -- the
                       law.  Station entries must be entered IN-LINE.
  C. straight drive -- control: dead-end run in line with the block (known
                       good, crossing_parity).
  D. corner exit    -- a station EXIT block at a corner: strongly powered,
                       lights dust on ANY side (a corner after an exit
                       block is free).
  E. descent        -- station exit -> 14-step 1y/1x descending staircase
                       (all solid supports) -> flat run: conducts (the
                       diode law downhill; h2v's climbs prove uphill).
"""
import rs
from rs import DUST

b = rs.Build("probe_pivot")
LV = []


def lever(x, y, z):
    b.stone(x, y - 1, z)
    b.force(x, y, z, rs.LEVER_OFF)
    LV.append((x, y, z))
    return len(LV) - 1


def station_x(x, z, y=1):
    """entry/repeater/exit along +X at x..x+2."""
    b.put(x, y, z, rs.PALETTE["route"])
    b.stone(x + 1, y - 1, z)
    b.put(x + 1, y, z, rs.repeater("west"))
    b.put(x + 2, y, z, rs.PALETTE["route"])


PROBE = {}

# A: lever - dot dust - entry block - repeater - exit - out dust  (z=0)
iA = lever(0, 1, 0)
b.stone(1, 0, 0)
b.put(1, 1, 0, DUST)                       # dot: no dust connections
station_x(2, 0)
b.stone(5, 0, 0)
b.put(5, 1, 0, DUST)
PROBE["A_dot"] = (5, 1, 0)

# B: dust run along +Z turning into an entry block along +X  (x=10)
iB = lever(10, 1, 4)
for z in (5, 6, 7):
    b.stone(10, 0, z)
    b.put(10, 1, z, DUST)                  # corner dust at z=7, connection -Z
station_x(11, 7)
b.stone(14, 0, 7)
b.put(14, 1, 7, DUST)
PROBE["B_corner"] = (14, 1, 7)

# C: straight dead-end control  (z=12)
iC = lever(0, 1, 12)
for x in (1, 2, 3):
    b.stone(x, 0, 12)
    b.put(x, 1, 12, DUST)
station_x(4, 12)
b.stone(7, 0, 12)
b.put(7, 1, 12, DUST)
PROBE["C_straight"] = (7, 1, 12)

# D: station along +Z whose EXIT block is a corner; continuation along +X
iD = lever(20, 1, 0)
for z in (1, 2):
    b.stone(20, 0, z)
    b.put(20, 1, z, DUST)                  # straight run +Z into entry
b.put(20, 1, 3, rs.PALETTE["route"])       # entry
b.stone(20, 0, 4)
b.put(20, 1, 4, rs.repeater("north"))      # flows +Z
b.put(20, 1, 5, rs.PALETTE["route"])       # exit block AT the corner
b.stone(21, 0, 5)
b.put(21, 1, 5, DUST)                      # continuation +X off the corner
b.stone(22, 0, 5)
b.put(22, 1, 5, DUST)
PROBE["D_corner_exit"] = (22, 1, 5)

# E: descent: lever - dust - station - 14-step staircase down - flat - probe
Y0 = 15
iE = lever(30, Y0, 0)
b.stone(31, Y0 - 1, 0)
b.put(31, Y0, 0, DUST)
station_x(32, 0, Y0)
x0 = 35                                    # first dust after exit, then down
for i in range(15):                        # y = Y0 - i, floor at y-1
    y = max(1, Y0 - i)
    b.stone(x0 + i, y - 1, 0, "gate")
    b.put(x0 + i, y, 0, DUST)
PROBE["E_descent"] = (x0 + 14, 1, 0)

sim = b.sim()
lv = rs.Levers(sim, LV)
lv.set([1] * len(LV))
on = {k: sim.power(*p) for k, p in PROBE.items()}
lv.set([0] * len(LV))
off = {k: sim.power(*p) for k, p in PROBE.items()}

verdicts = [
    ("A_dot", True, "A lever-axis dead-end dust weak-powers the entry block"),
    ("B_corner", False, "B corner dust does NOT fire a perpendicular entry "
                        "block (the pointing law)"),
    ("C_straight", True, "C straight dead-end fires the entry block (control)"),
    ("D_corner_exit", True, "D corner EXIT block lights the +X continuation"),
    ("E_descent", True, "E station exit -> 14-step descent -> flat conducts"),
]
bad = 0
for key, conducts, text in verdicts:
    ok = (on[key] > 0 and off[key] == 0) if conducts else on[key] == 0
    print("%s %s (on=%d off=%d)" % ("PASS" if ok else "FAIL", text,
                                    on[key], off[key]))
    bad += 0 if ok else 1
print("probe_pivot: %d/%d" % (len(verdicts) - bad, len(verdicts)))
raise SystemExit(1 if bad else 0)
