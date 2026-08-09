"""Empirically pin the material table in mc-tick for glass / stained glass /
slab-top / slab-bottom (stone and air as controls).

Questions, per material M (all answered in-sim, mc-tick is the golden model):

  SIT    -- does dust placed on M survive placement settle and conduct flat?
  WEAK   -- does M conduct weak power (dust -> M -> repeater fires)?
  CUT    -- does M above the LOWER dust of a diagonal cut the connection?
            (probed in both flow directions: CUT_UP drives from below,
             CUT_DOWN drives from above -- the cut governs the CONNECTION,
             so both should agree)
  DIODE  -- vertical asymmetry of a 1-y step whose UPPER dust sits on M:
            DIODE_UP drives the lower dust (signal climbing onto M),
            DIODE_DOWN drives the upper dust (signal descending off M).
            Vanilla predicts glass/slab-top are up-only ("glass diode": the
            down-flow pull reads the wire above a side block only when that
            side block -- the upper dust's support -- is a conductor).
  UNDER  -- can a dust line pass under a top-slab that carries dust on top,
            with the two lines isolated? (4 lever combos)

Exit code 0 iff every control behaves and the probed table is self-consistent.
"""
import rs
from rs import DUST, STONE, LEVER_OFF, repeater

GLASS = "minecraft:glass"
SGLASS = "minecraft:white_stained_glass"
SLAB_TOP = "minecraft:smooth_stone_slab[type=top,waterlogged=false]"
SLAB_BOT = "minecraft:smooth_stone_slab[type=bottom,waterlogged=false]"

MATS = [("stone", STONE), ("glass", GLASS), ("sglass", SGLASS),
        ("slab_top", SLAB_TOP), ("slab_bot", SLAB_BOT)]

b = rs.Build("probe_materials")
LEVERS = []
PROBE = {}


def lever(x, y, z):
    """Floor lever on a stone column reaching up to y-1."""
    for yy in range(0, y):
        b.stone(x, yy, z)
    b.force(x, y, z, LEVER_OFF)
    LEVERS.append((x, y, z))
    return len(LEVERS) - 1


# -- SIT: lever, 2 dust on stone, dust on M, 2 dust on stone ----------------
for i, (name, mat) in enumerate(MATS + [("air", None)]):
    z = 0 + 4 * i
    lever(0, 1, z)
    for x in (1, 2):
        b.stone(x, 0, z)
        b.put(x, 1, z, DUST)
    if mat is not None:
        b.put(3, 0, z, mat)
    b.put(3, 1, z, DUST)
    for x in (4, 5):
        b.stone(x, 0, z)
        b.put(x, 1, z, DUST)
    PROBE["SIT_%s_on" % name] = (3, 1, z)
    PROBE["SIT_%s_past" % name] = (5, 1, z)

# -- WEAK: lever, 2 dust, M at dust level, repeater out of M, dust ----------
for i, (name, mat) in enumerate(MATS + [("air", None)]):
    z = 30 + 4 * i
    lever(0, 1, z)
    for x in (1, 2):
        b.stone(x, 0, z)
        b.put(x, 1, z, DUST)
    b.stone(3, 0, z)
    if mat is not None:
        b.put(3, 1, z, mat)
    b.stone(4, 0, z)
    b.put(4, 1, z, repeater("west"))
    b.stone(5, 0, z)
    b.put(5, 1, z, DUST)
    PROBE["WEAK_%s" % name] = (5, 1, z)

# -- CUT_UP: lever low; dust(1,1); M above it at (1,2); dust(2,2) high ------
for i, (name, mat) in enumerate(MATS + [("air", None)]):
    z = 60 + 4 * i
    lever(0, 1, z)
    b.stone(1, 0, z)
    b.put(1, 1, z, DUST)
    if mat is not None:
        b.put(1, 2, z, mat)
    b.stone(2, 1, z)
    b.put(2, 2, z, DUST)
    PROBE["CUTUP_%s" % name] = (2, 2, z)

# -- CUT_DOWN: lever high; dust(1,2) on stone; dust(2,1) low; M at (2,2) ----
for i, (name, mat) in enumerate(MATS + [("air", None)]):
    z = 90 + 4 * i
    lever(0, 2, z)
    b.stone(1, 1, z)
    b.put(1, 2, z, DUST)
    b.stone(2, 0, z)
    b.put(2, 1, z, DUST)
    if mat is not None:
        b.put(2, 2, z, mat)
    PROBE["CUTDN_%s" % name] = (2, 1, z)

# -- DIODE_UP: lever low; dust(1,1); dust(2,2) SUPPORTED BY M at (2,1) ------
for i, (name, mat) in enumerate(MATS):
    z = 120 + 4 * i
    lever(0, 1, z)
    b.stone(1, 0, z)
    b.put(1, 1, z, DUST)
    b.put(2, 1, z, mat)
    b.put(2, 2, z, DUST)
    PROBE["DIOUP_%s" % name] = (2, 2, z)

# -- DIODE_DOWN: lever high; dust(1,2) on M at (1,1); dust(2,1) low ---------
for i, (name, mat) in enumerate(MATS):
    z = 150 + 4 * i
    lever(0, 2, z)
    b.put(1, 1, z, mat)
    b.put(1, 2, z, DUST)
    b.stone(2, 0, z)
    b.put(2, 1, z, DUST)
    PROBE["DIODN_%s" % name] = (2, 1, z)

# -- UNDER: dust line at y=1 passing under a top-slab carrying dust at y=3 --
ZU = 180
i_under = lever(0, 1, ZU)                 # under-line driver
for x in range(1, 6):
    b.stone(x, 0, ZU)
    b.put(x, 1, ZU, DUST)
PROBE["UNDER_lo"] = (5, 1, ZU)
for dz in (-1, 0, 1):                     # slab bridge crossing at x=3
    b.put(3, 2, ZU + dz, SLAB_TOP)
    b.put(3, 3, ZU + dz, DUST)
b.put(3, 2, ZU - 2, STONE)                # over-line driver mount + lever
i_over = len(LEVERS)
b.force(3, 3, ZU - 2, LEVER_OFF)
LEVERS.append((3, 3, ZU - 2))
PROBE["UNDER_hi"] = (3, 3, ZU + 1)

sim = b.sim()
lv = rs.Levers(sim, LEVERS)

lv.set([0] * len(LEVERS))
off = {k: sim.power(*p) for k, p in PROBE.items()}
blk_off = {k: sim.block(*PROBE[k]) for k in PROBE if k.startswith("SIT")}
lv.set([1] * len(LEVERS))
on = {k: sim.power(*p) for k, p in PROBE.items()}

# UNDER isolation: drive each lever alone
bits = [1] * len(LEVERS)
bits[i_over] = 0
lv.set(bits)
under_only = (sim.power(*PROBE["UNDER_lo"]), sim.power(*PROBE["UNDER_hi"]))
bits = [0] * len(LEVERS)
bits[i_over] = 1
lv.set(bits)
over_only = (sim.power(*PROBE["UNDER_lo"]), sim.power(*PROBE["UNDER_hi"]))

names = [n for n, _ in MATS]
print("== probed material table (mc-tick) ==")
hdr = "%-9s %-9s %-11s %-14s %-12s %-12s" % (
    "material", "dust sits", "weak-power", "cuts diagonal", "diode: up", "diode: down")
print(hdr)
print("-" * len(hdr))
ROWS = {}
for name in names + ["air"]:
    sits = ("redstone_wire" in blk_off.get("SIT_%s_on" % name, "")
            and on["SIT_%s_past" % name] > 0)
    weak = on["WEAK_%s" % name] > 0
    cuts = on["CUTUP_%s" % name] == 0 and on["CUTDN_%s" % name] == 0
    cut_mixed = (on["CUTUP_%s" % name] == 0) != (on["CUTDN_%s" % name] == 0)
    if name != "air":
        d_up = on["DIOUP_%s" % name] > 0
        d_dn = on["DIODN_%s" % name] > 0
    else:
        d_up = d_dn = None
    ROWS[name] = (sits, weak, cuts, d_up, d_dn)
    print("%-9s %-9s %-11s %-14s %-12s %-12s" % (
        name, "yes" if sits else "NO", "yes" if weak else "NO",
        ("MIXED" if cut_mixed else ("yes" if cuts else "NO")),
        {True: "conducts", False: "BLOCKED", None: "-"}[d_up],
        {True: "conducts", False: "BLOCKED", None: "-"}[d_dn]))
print()
print("UNDER (dust under a top-slab carrying dust): lo=%d hi=%d (both on), "
      "under-only lo=%d hi=%d, over-only lo=%d hi=%d"
      % (on["UNDER_lo"], on["UNDER_hi"], under_only[0], under_only[1],
         over_only[0], over_only[1]))

verdicts = [
    ("stone control: sits/conducts/weak/cuts",
     ROWS["stone"][0] and ROWS["stone"][1] and ROWS["stone"][2]),
    ("stone step conducts both vertical directions",
     ROWS["stone"][3] and ROWS["stone"][4]),
    ("air control: no weak power, no cut",
     not ROWS["air"][1] and not ROWS["air"][2]
     and on["CUTUP_air"] > 0 and on["CUTDN_air"] > 0),
    ("all probes dark with levers off", all(v <= 0 for v in off.values())),
    ("UNDER: both lines conduct", on["UNDER_lo"] > 0 and on["UNDER_hi"] > 0),
    ("UNDER: isolated (under-only)",
     under_only[0] > 0 and under_only[1] == 0),
    ("UNDER: isolated (over-only)",
     over_only[0] == 0 and over_only[1] > 0),
]
bad = 0
for text, ok in verdicts:
    print("%s %s" % ("PASS" if ok else "FAIL", text))
    bad += 0 if ok else 1
print("probe_materials: %d/%d control expectations hold"
      % (len(verdicts) - bad, len(verdicts)))
raise SystemExit(1 if bad else 0)
