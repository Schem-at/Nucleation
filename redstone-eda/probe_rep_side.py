"""Micro-probe: repeater -> comparator SIDE registration in mc-tick.

Never verified in isolation (probe_comp fed sides via dust).  Configs:

  A: comp facing=west, side repeater approaches from SOUTH (HA C1 config)
  B: comp facing=west, side repeater approaches from NORTH (HA C3 config)
  C: comp facing=east, side repeater from south
  D: comp facing=east, side repeater from north
  E: config B + dust on the comparator's OTHER flank with its own driver --
     vanilla side value = max(left flank, right flank)

Side feed shape mirrors the cells: lever -> dust -> dust -> repeater -> side.
Expected (subtract): out = max(back - side, 0).
"""
import rs

COMP = "minecraft:comparator[facing=%s,mode=%s,powered=false]"
if "comparator" not in rs.EXTRA_STATES:
    rs.EXTRA_STATES += ";" + ";".join(
        "minecraft:comparator[facing=%s,mode=%s,powered=%s]" % (f, m, pw)
        for f in ("north", "south", "east", "west")
        for m in ("compare", "subtract") for pw in ("false", "true"))

b = rs.Build("probe_rep_side")
levers = []


def dust(x, z):
    b.stone(x, 0, z)
    b.put(x, 1, z, rs.DUST)


def lever(x, z):
    b.stone(x, 0, z)
    b.put(x, 1, z, rs.LEVER_OFF)
    levers.append((x, 1, z))


def cfg(z, comp_facing, side_from):
    """comp at (4,1,z).  side_from: 'south' -> repeater at z+1 fed from z+2;
    'north' -> repeater at z-1 fed from z-2.  Returns out probe cell."""
    if comp_facing == "west":                    # input west, output east
        lever(0, z); dust(1, z); dust(2, z); dust(3, z)      # back line
        b.stone(4, 0, z); b.put(4, 1, z, COMP % ("west", "subtract"))
        dust(5, z); dust(6, z)                                # out line
        out = (6, 1, z)
    else:                                        # facing=east: input east
        lever(8, z); dust(7, z); dust(6, z); dust(5, z)
        b.stone(4, 0, z); b.put(4, 1, z, COMP % ("east", "subtract"))
        dust(3, z); dust(2, z)
        out = (2, 1, z)
    zr = z + 1 if side_from == "south" else z - 1
    zd = z + 2 if side_from == "south" else z - 2
    b.stone(4, 0, zr)
    b.put(4, 1, zr, rs.repeater(side_from))      # input side = side_from
    dust(4, zd); dust(5, zd); lever(6, zd)
    return out


outA = cfg(0, "west", "south")
outB = cfg(8, "west", "north")
outC = cfg(14, "east", "south")
outD = cfg(20, "east", "north")
outE = cfg(26, "west", "north")
# E extra: other-flank dust with its own lever
b.stone(4, 0, 27); b.put(4, 1, 27, rs.DUST)
dust(4, 28); dust(5, 28); lever(6, 28)

sim = b.sim()
lv = rs.Levers(sim, levers)  # order: A back, A side, B back, B side, C..., E back, E side, E flank

print("subtract: out should be max(back-side,0)  [15s in, so 15 or 0]")
ok = True
for back, side in ((0, 0), (1, 0), (0, 1), (1, 1)):
    lv.set([back, side, back, side, back, side, back, side, back, side, 0])
    want = 15 if (back and not side) else 0
    got = [sim.power(*outA), sim.power(*outB), sim.power(*outC),
           sim.power(*outD), sim.power(*outE)]
    flag = "" if all(g == want for g in got) else "   <-- MISMATCH want %d" % want
    ok = ok and not flag
    print("  back=%d side=%d -> A=%2d B=%2d C=%2d D=%2d E=%2d%s"
          % (back, side, *got, flag))

print("E two-flank: back=1; side value should be max(rep flank, dust flank)")
for rep_s, dust_s in ((0, 0), (0, 1), (1, 0), (1, 1)):
    lv.set([0, 0, 0, 0, 0, 0, 0, 0, 1, rep_s, dust_s])
    want = 15 if not (rep_s or dust_s) else 0
    got = sim.power(*outE)
    print("  rep=%d dust=%d -> out=%2d (want %2d)%s"
          % (rep_s, dust_s, got, want, "" if got == want else "   <-- MISMATCH"))
