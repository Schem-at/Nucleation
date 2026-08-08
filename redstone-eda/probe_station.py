"""Probe the block-sandwich repeater station: dust -> block -> repeater -> block -> dust.

Questions this settles (all in mc-tick, the golden model):

  A. entry reach   -- how many dust cells can precede the entry block?  A dust
                      arriving at ss1 weak-powers the block; does the repeater
                      behind the block still fire?  (n=14/15/16 dust runs)
  I. inline max    -- same question for the bare inline repeater (15/16 dust),
                      so the station is measured against the true inline max,
                      not against the old REFRESH=5 safety margin.
  B. exit reach    -- the repeater strongly powers the exit block; does the
                      next dust start at a fresh 15?
  Z. axis check    -- the same station along +Z.
  F. chained       -- two stations at max pitch end to end.
  S. side effects  -- foreign dust beside/atop the entry block (weak-powered),
                      beside the repeater, beside/atop the exit block
                      (STRONGLY powered): which of these leak?
  C. comparator    -- the analog variant: does dust -> block -> comparator ->
                      block -> dust preserve exact signal strength?  Variants
                      isolate the rear-through-block read and the
                      output-through-block re-emit.
  G. tap floor     -- a rail tap (spur dust + torch) needs rail ss >= 2: the
                      tap dust reads rail-1 and a ss0 tap dust never flips the
                      torch.  This is what actually bounds RAIL_REPEAT --
                      NOT the repeater's own ss1 input floor.
"""
import rs
from rs import DUST, STONE, LEVER_OFF, repeater

COMP = "minecraft:comparator[facing=%s,mode=%s,powered=%s]"
rs.EXTRA_STATES += ";" + ";".join(
    COMP % (f, m, p) for f in ("north", "south", "east", "west")
    for m in ("compare", "subtract") for p in ("false", "true"))

b = rs.Build("probe_station")
LEVERS = []


def lane(z, n):
    """Lever + n dust cells along +X; returns x of the last dust cell."""
    b.stone(0, 0, z)
    b.force(0, 1, z, LEVER_OFF)
    LEVERS.append((0, 1, z))
    for x in range(1, n + 1):
        b.stone(x, 0, z)
        b.put(x, 1, z, DUST)
    return n


def station(x, z, dev="repeater"):
    """block / device / block at x, x+1, x+2.  Returns first x after it."""
    b.put(x, 1, z, STONE)
    b.stone(x + 1, 0, z)
    if dev == "repeater":
        b.put(x + 1, 1, z, repeater("west"))
    else:
        b.put(x + 1, 1, z, COMP % ("west", "compare", "false"))
    b.put(x + 2, 1, z, STONE)
    return x + 3


def out(z, x0, n):
    for x in range(x0, x0 + n):
        b.stone(x, 0, z)
        b.put(x, 1, z, DUST)


PROBE = {}

# A: dust run into the entry block: 14 / 15 / 16 cells
for name, nlen, z in (("A14", 14, 0), ("A15", 15, 4), ("A16", 16, 8)):
    e = lane(z, nlen) + 1
    out(z, station(e, z), 2)
    PROBE[name] = (e + 3, 1, z)

# I: bare inline repeater after 15 / 16 dust cells
for name, nlen, z in (("I15", 15, 12), ("I16", 16, 16)):
    e = lane(z, nlen) + 1
    b.stone(e, 0, z)
    b.put(e, 1, z, repeater("west"))
    out(z, e + 1, 2)
    PROBE[name] = (e + 1, 1, z)

# B: exit-side reach: 15 out dust after a max-pitch station
e = lane(20, 15) + 1
out(20, station(e, 20), 15)
PROBE["B_first"] = (e + 3, 1, 20)
PROBE["B_last"] = (e + 17, 1, 20)

# F: two stations chained at max pitch
e = station(lane(24, 15) + 1, 24)
out(24, e, 15)
e2 = station(e + 15, 24)
out(24, e2, 5)
PROBE["F"] = (e2 + 4, 1, 24)

# Z: the same station along +Z
b.stone(40, 0, -1)
b.force(40, 1, -1, LEVER_OFF)
LEVERS.append((40, 1, -1))
for z in range(0, 15):
    b.stone(40, 0, z)
    b.put(40, 1, z, DUST)
b.put(40, 1, 15, STONE)
b.stone(40, 0, 16)
b.put(40, 1, 16, repeater("north"))
b.put(40, 1, 17, STONE)
for z in (18, 19):
    b.stone(40, 0, z)
    b.put(40, 1, z, DUST)
PROBE["Z"] = (40, 1, 18)

# S: side effects on a short-trunk station (station at x=5..7, out 8..9)
SIDE = (("S_entry_side", (5, 1, 1)),    # beside entry block  (weak-powered)
        ("S_entry_top",  (5, 2, 0)),    # atop entry block (dust diagonal!)
        ("S_rep_side",   (6, 1, 1)),    # beside the repeater
        ("S_exit_side",  (7, 1, 1)),    # beside exit block (STRONG-powered)
        ("S_exit_top",   (7, 2, 0)))    # atop exit block
for i, (name, (px, py, pz)) in enumerate(SIDE):
    z = 30 + 4 * i
    e = lane(z, 4) + 1
    out(z, station(e, z), 2)
    p = (px, py, pz + z)
    if py == 1:
        b.stone(px, 0, pz + z)
    b.put(*p, DUST)
    PROBE[name] = p

# C: comparator analog variants
#   C1/C2: dust(->ss12 / ss8) -> block -> comparator -> block -> dust
#   C3:    dust(->ss12) -> comparator direct -> block -> dust
#   C4:    dust(->ss12) -> block -> comparator -> dust direct
for name, nlen, z in (("C1", 4, 52), ("C2", 8, 56)):
    e = lane(z, nlen) + 1
    out(z, station(e, z, dev="comparator"), 3)
    PROBE[name] = (e + 3, 1, z)
e = lane(60, 4) + 1
b.stone(e, 0, 60)
b.put(e, 1, 60, COMP % ("west", "compare", "false"))
b.put(e + 1, 1, 60, STONE)
out(60, e + 2, 3)
PROBE["C3"] = (e + 2, 1, 60)
e = lane(64, 4) + 1
b.put(e, 1, 64, STONE)
b.stone(e + 1, 0, 64)
b.put(e + 1, 1, 64, COMP % ("west", "compare", "false"))
out(64, e + 2, 3)
PROBE["C4"] = (e + 2, 1, 64)


def tap(x, z):
    """A rail tap exactly as build_column lays it: spur dust, block, torch,
    block above, collector dust on top."""
    b.stone(x, 0, z + 1)
    b.put(x, 1, z + 1, DUST)
    b.put(x, 1, z + 2, STONE)
    b.put(x, 2, z + 2, rs.TORCH)
    b.put(x, 3, z + 2, STONE)
    b.put(x, 4, z + 2, DUST)
    return (x, 4, z + 2)


# G: tap driven from a rail cell at ss2 (works) and ss1 (fails)
lane(70, 14)                      # dust x=14 sits at ss2
PROBE["G_ss2_coll"] = tap(14, 70)
lane(76, 15)                      # dust x=15 sits at ss1
PROBE["G_ss1_coll"] = tap(15, 76)

sim = b.sim()
lv = rs.Levers(sim, LEVERS)

print("== all levers OFF ==")
lv.set([0] * len(LEVERS))
off = {k: sim.power(*p) for k, p in sorted(PROBE.items())}
for k, v in sorted(off.items()):
    print("   %-14s %2d" % (k, v))

print("== all levers ON ==")
lv.set([1] * len(LEVERS))
on = {k: sim.power(*p) for k, p in sorted(PROBE.items())}
for k, v in sorted(on.items()):
    print("   %-14s %2d" % (k, v))

print()
verdicts = [
    ("A14 station conducts (ss2 entry)", on["A14"] > 0 and off["A14"] == 0),
    ("A15 station conducts (ss1 entry)", on["A15"] > 0 and off["A15"] == 0),
    ("A16 dead (ss0 never powers the block)", on["A16"] == 0),
    ("I15 inline repeater fires from ss1 dust", on["I15"] > 0 and off["I15"] == 0),
    ("I16 inline dead at ss0", on["I16"] == 0),
    ("B exit block re-emits a fresh 15", on["B_first"] == 15),
    ("B 15th dust after station alive at ss1", on["B_last"] == 1),
    ("F two max-pitch stations chained conduct", on["F"] > 0 and off["F"] == 0),
    ("Z station works along +Z", on["Z"] > 0 and off["Z"] == 0),
    ("S entry-block side dust stays dark (weak power)", on["S_entry_side"] == 0),
    ("S entry-block TOP dust leaks (dust diagonal)", on["S_entry_top"] > 0),
    ("S repeater side dust stays dark", on["S_rep_side"] == 0),
    ("S exit-block side dust LEAKS at 15 (strong power)", on["S_exit_side"] == 15),
    ("S exit-block TOP dust LEAKS", on["S_exit_top"] > 0),
    ("C1 comparator sandwich preserves ss12", on["C1"] == 12),
    ("C2 comparator sandwich preserves ss8", on["C2"] == 8),
    ("C3 direct-rear comparator preserves ss12", on["C3"] == 12),
    ("C4 direct-out comparator preserves ss12", on["C4"] == 12),
    ("G tap on a ss2 rail cell inverts correctly",
     on["G_ss2_coll"] == 0 and off["G_ss2_coll"] == 15),
    ("G tap on a ss1 rail cell is BROKEN (never flips)",
     on["G_ss1_coll"] == 15 and off["G_ss1_coll"] == 15),
]
bad = 0
for text, ok in verdicts:
    print("%s %s" % ("PASS" if ok else "FAIL", text))
    bad += 0 if ok else 1
print("probe_station: %d/%d expectations hold" % (len(verdicts) - bad, len(verdicts)))
raise SystemExit(1 if bad else 0)
