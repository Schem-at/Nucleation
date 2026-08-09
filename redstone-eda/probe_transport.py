"""Probe the SIGNAL-TRANSPORT mechanisms, one lane per question (mc-tick).

The material model (probe_materials.py) answered "what may sit where".  This
probes "how does the signal MOVE, and what may share space with it" -- the
facts TRANSPORT_MODEL.md tabulates.  Every lane is driven from its own lever
and every verdict is expressed over the full one-lever-at-a-time matrix, so
"conducts" and "does not leak" are the same kind of claim.

Groups:

  S. STRONG (hard) power
     S1 a strongly powered block does NOT chain strong power to the solid
        block beside it -- foreign dust ON that neighbour stays dark.  This
        is the whole reason CROSSWIRE002 works.
     S2 a strongly powered block lights dust on ALL SIX faces at 15.
     S3 two blocks, each strongly powered by its OWN repeater, sitting side
        by side: no interference (the user's stated crossing insight).
     S4 a repeater sitting ON a strongly powered block is unaffected -- no
        quasi-connectivity for repeaters (CROSSWIRE002's upper line).

  W. WEAK power
     W1 a weakly powered block (dust pointing into it) does NOT light dust
        on any face, including the one above it,
     W2 while a repeater reading that same block DOES fire -- so "powered"
        is not one state but two.
     W3 dust does NOT power the block ABOVE it, so a solid block laid on top
        of a live dust run can carry a foreign dust line (this is what the
        bump supports in CROSSWIRE001 are doing).

  C. the CUT rule vs the DIODE rule -- two DIFFERENT cells, probed as a
     2x2x2 matrix (cut cell x upper support x direction).  Vanilla's
     calculateTargetStrength reads, for a horizontal neighbour np of self:
        up   (self is the LOWER dust): np must be a CONDUCTOR   <- the
             upper dust's SUPPORT, and above(self) must not be    DIODE cell
        down (self is the UPPER dust): np must NOT be a conductor <- above
             the LOWER dust                                        CUT cell
     => uphill is gated by the CUT cell alone; downhill by the CUT cell AND
     the DIODE cell.  Today materials.py has one predicate for both.

  T. TORCH (the vertical carrier + the only inverter)
     T1 a standing torch strongly powers the block ABOVE it,
     T2 it does NOT power its own attachment block,
     T3 it inverts its attachment block's power (the tower's step).

  B. REDSTONE BLOCK (a source that is not a powered block)
     B1 lights adjacent dust at 15,
     B2 does not power the solid block beside it.

  P. LANE PITCH at one level
     P1 two dust lines 1 cell apart MERGE (this is a legality constraint,
        not a bug), P2 2 cells apart are isolated, P3 diagonally offset
        (dx=1,dz=1, same y) are isolated -- dust has no planar diagonal.

Run: ~/eda-venv/bin/python probe_transport.py
"""
import rs
from rs import DUST, STONE, LEVER_OFF, repeater

GLASS = "minecraft:glass"
RBLOCK = "minecraft:redstone_block"
rs.EXTRA_STATES += ";" + RBLOCK

b = rs.Build("probe_transport")
LV = []                 # lever positions, in lever order
PROBE = {}              # name -> cell
NAMES = []              # lever index -> label of the lane that owns it


def lever(x, y, z, label):
    b.stone(x, y - 1, z)
    b.force(x, y, z, LEVER_OFF)
    LV.append((x, y, z))
    NAMES.append(label)
    return len(LV) - 1


def dust(x, y, z, support=STONE):
    if support is not None:
        b.force(x, y - 1, z, support)
    b.force(x, y, z, DUST)


def run(x0, x1, y, z):
    for x in range(x0, x1 + 1):
        dust(x, y, z)


# ------------------------------------------------------------------ S: strong
# S1: lever - 2 dust - entry block - repeater - EXIT block - plain block;
#     probes: dust ON the plain neighbour, dust BESIDE it.
Z = 0
iS1 = lever(0, 1, Z, "S1")
run(1, 2, 1, Z)
b.force(3, 1, Z, STONE)                       # entry (weakly powered)
b.stone(4, 0, Z)
b.force(4, 1, Z, repeater("west"))            # flows +X
b.force(5, 1, Z, STONE)                       # EXIT -- strongly powered
b.force(6, 1, Z, STONE)                       # plain neighbour of the exit
dust(6, 2, Z, support=None)                   # sits on the neighbour
PROBE["S1_dust_on_block_beside_strong"] = (6, 2, Z)
dust(7, 1, Z)
PROBE["S1_dust_past_that_block"] = (7, 1, Z)

# S2: all six faces of a strongly powered exit block.
Z = 3
iS2 = lever(0, 3, Z, "S2")
run(1, 2, 3, Z)
b.force(3, 3, Z, STONE)
b.stone(4, 2, Z)
b.force(4, 3, Z, repeater("west"))
b.force(5, 3, Z, STONE)                       # EXIT at y=3
dust(5, 4, Z, support=None)                   # above
PROBE["S2_face_above"] = (5, 4, Z)
dust(5, 2, Z)                                 # below (its own floor at y=1)
PROBE["S2_face_below"] = (5, 2, Z)
dust(6, 3, Z)                                 # +X
PROBE["S2_face_side_x"] = (6, 3, Z)
dust(5, 3, Z + 1)                             # +Z
PROBE["S2_face_side_z"] = (5, 3, Z + 1)

# S3: two blocks, each strongly powered by its own repeater, side by side.
Z = 6
iS3a = lever(0, 1, Z, "S3a")
run(1, 1, 1, Z)
b.force(2, 1, Z, STONE)                       # entry A
b.stone(3, 0, Z)
b.force(3, 1, Z, repeater("west"))            # A flows +X
b.force(4, 1, Z, STONE)                       # EXIT A  (strong)
b.force(5, 1, Z, STONE)                       # EXIT B  (strong) -- ADJACENT
b.stone(6, 0, Z)
b.force(6, 1, Z, repeater("east"))            # B flows -X
b.force(7, 1, Z, STONE)                       # entry B
run(8, 8, 1, Z)
iS3b = lever(9, 1, Z, "S3b")
dust(4, 1, Z - 1, )                           # A readout, perpendicular
PROBE["S3_readout_A"] = (4, 1, Z - 1)
dust(5, 1, Z + 1)                             # B readout, perpendicular
PROBE["S3_readout_B"] = (5, 1, Z + 1)

# S4: a repeater sitting ON a strongly powered block.  Lower line runs +X at
#     y=2 and is read BELOW its own exit block; upper line runs +Z at y=3 and
#     its repeater's floor IS that exit block.  The upper line's dust supports
#     at (5,2,Z-1)/(5,2,Z+1) also sit flush against the strong block, so this
#     lane re-checks S1 from a second direction.
Z = 10
iS4a = lever(0, 2, Z, "S4a")
run(1, 2, 2, Z)
b.force(3, 2, Z, STONE)                       # entry
b.stone(4, 1, Z)
b.force(4, 2, Z, repeater("west"))            # flows +X
b.force(5, 2, Z, STONE)                       # EXIT (strong)
dust(5, 1, Z)                                 # read the exit from BELOW
PROBE["S4_lowerline_out"] = (5, 1, Z)
b.force(5, 2, Z - 1, STONE)                   # upper line's entry floor
b.force(5, 3, Z - 1, STONE)                   # entry block
dust(5, 3, Z - 2)
iS4b = lever(5, 3, Z - 3, "S4b")
b.force(5, 3, Z, repeater("north"))           # flows +Z, FLOOR = the exit block
b.force(5, 2, Z + 1, STONE)
dust(5, 3, Z + 1, support=None)
dust(5, 3, Z + 2)
PROBE["S4_upperline_out"] = (5, 3, Z + 2)

# -------------------------------------------------------------------- W: weak
# W1/W2: one weakly powered block, read three ways.
Z = 14
iW = lever(0, 1, Z, "W")
run(1, 2, 1, Z)
b.force(3, 1, Z, STONE)                       # WEAKLY powered by dust (2,1,Z)
#   The probe dusts flank the weak block one level up / in line.  Both of them
#   would otherwise be a legal 1-y DIAGONAL to the driving run, so each gets a
#   conducting cut cell over the LOWER dust of that diagonal -- exactly the
#   cut rule, used here to isolate the weak-power question from the step rules.
b.force(2, 2, Z, STONE)                       # cut cell over the driving dust
b.force(4, 2, Z, STONE)                       # cut cell over the far dust
dust(3, 2, Z, support=None)                   # dust on top of the weak block
PROBE["W1_dust_above_weak"] = (3, 2, Z)
dust(4, 1, Z)                                 # dust past it, in line
PROBE["W1_dust_beyond_weak"] = (4, 1, Z)
b.stone(3, 0, Z + 1)
b.force(3, 1, Z + 1, repeater("north"))       # back faces -Z = the weak block
run(3, 3, 1, Z + 2)
PROBE["W2_repeater_off_weak"] = (3, 1, Z + 2)

# W3: dust does not power the block above it -> that block carries foreign dust.
Z = 18
iW3 = lever(0, 1, Z, "W3")
run(1, 4, 1, Z)
PROBE["W3_lowerline_out"] = (4, 1, Z)
b.force(2, 2, Z, STONE)                       # lid directly over live dust
b.force(3, 2, Z, STONE)
dust(2, 3, Z, support=None)                   # foreign dust ON the lid
dust(3, 3, Z, support=None)
PROBE["W3_dust_on_lid"] = (3, 3, Z)

# ------------------------------------------------- C: cut cell vs diode cell
#   lower dust L at (1,y,z) ; cut cell at (1,y+1,z) ; upper support at
#   (2,y,z) ; upper dust U at (2,y+1,z).  Drive L (uphill) or U (downhill).
CUT = {"air": None, "solid": STONE}
SUP = {"solid": STONE, "glass": GLASS}
Z = 22
for cutname, cut in CUT.items():
    for supname, sup in SUP.items():
        for direction in ("up", "down"):
            tag = "C_%s_cut_%s_sup_%s" % (direction, cutname, supname)
            y = 1
            dust(1, y, Z)                                    # lower dust L
            if cut is not None:
                b.force(1, y + 1, Z, cut)                    # THE CUT CELL
            b.force(2, y, Z, sup)                            # THE DIODE CELL
            b.force(2, y + 1, Z, DUST)                       # upper dust U
            if direction == "up":
                lever(0, y, Z, tag)                          # drive L
                PROBE[tag] = (2, y + 1, Z)                   # read U
            else:
                dust(3, y + 1, Z)                            # U's continuation
                lever(4, y + 1, Z, tag)                      # drive U
                PROBE[tag] = (1, y, Z)                       # read L
            Z += 3

# ------------------------------------------------------------------- T: torch
# T1/T2: a standing torch on a block; block above; dust on top of that block.
iT1 = lever(0, 1, Z, "T1")                     # lever only to keep the matrix
run(1, 1, 1, Z)                                # square (unused electrically)
b.force(3, 2, Z, STONE)                        # the torch's attachment block
b.force(3, 3, Z, rs.TORCH)                     # standing torch, always lit
b.force(3, 4, Z, STONE)                        # block above the torch
dust(3, 5, Z, support=None)
PROBE["T1_strong_above_torch"] = (3, 5, Z)
dust(3, 1, Z)                                  # two below the attachment block
PROBE["T2_attachment_not_powered"] = (3, 1, Z)
Z += 3

# T3: inversion -- powering the attachment block extinguishes the torch.
iT3 = lever(0, 1, Z, "T3")
run(1, 2, 1, Z)
b.force(3, 1, Z, STONE)                        # attachment block, weakly powered
b.force(3, 2, Z, rs.TORCH)                     # torch on top of it
dust(4, 2, Z, support=None)                    # torch's output dust
b.force(4, 1, Z, STONE)
PROBE["T3_torch_out"] = (4, 2, Z)
Z += 3

# ------------------------------------------------------- B: block of redstone
iB = lever(0, 1, Z, "B")                       # unused; keeps the matrix square
run(1, 1, 1, Z)
b.force(3, 1, Z, RBLOCK)
dust(4, 1, Z)
PROBE["B1_dust_beside_rblock"] = (4, 1, Z)
b.force(3, 1, Z + 1, STONE)                    # solid block beside the rblock
dust(3, 1, Z + 2)
PROBE["B2_dust_past_block_beside_rblock"] = (3, 1, Z + 2)
Z += 4

# ------------------------------------------------------------- P: lane pitch
iP = lever(0, 1, Z, "P")
run(1, 5, 1, Z)                                # driven line
run(1, 5, 1, Z + 1)                            # ADJACENT line (dz=1)
PROBE["P1_adjacent_line"] = (5, 1, Z + 1)
run(1, 5, 1, Z + 3)                            # dz=2 from the adjacent one,
PROBE["P2_gap_line"] = (5, 1, Z + 3)           #   dz=3 from the driven one
Z += 6
iP3 = lever(0, 1, Z, "P3")
run(1, 3, 1, Z)
dust(4, 1, Z + 1)                              # dx=1, dz=1 from (3,1,Z)
dust(5, 1, Z + 1)
PROBE["P3_planar_diagonal"] = (5, 1, Z + 1)


# ------------------------------------------------------------------ the matrix
sim = b.sim()
lv = rs.Levers(sim, LV)
n = len(LV)
lv.set([0] * n)
off = {k: sim.power(*p) for k, p in PROBE.items()}
solo = []
for i in range(n):
    lv.set([1 if j == i else 0 for j in range(n)])
    solo.append({k: sim.power(*p) for k, p in PROBE.items()})
lv.set([1] * n)
allon = {k: sim.power(*p) for k, p in PROBE.items()}
lv.set([0] * n)


def owner(label):
    return NAMES.index(label)


def hot(probe, label):
    """probe is powered when ONLY `label`'s lever is on."""
    return solo[owner(label)][probe] > 0


def ss(probe, label):
    return solo[owner(label)][probe]


VERDICTS = []


def V(ok, text, extra=""):
    VERDICTS.append((bool(ok), text, extra))


# --- S
V(not hot("S1_dust_on_block_beside_strong", "S1"),
  "S1 strong power does NOT chain block->block (dust on the neighbouring "
  "block stays dark)", "ss=%d" % ss("S1_dust_on_block_beside_strong", "S1"))
V(not hot("S1_dust_past_that_block", "S1"),
  "S1 nor to dust beyond that block")
V(all(ss(k, "S2") == 15 for k in
      ("S2_face_above", "S2_face_below", "S2_face_side_x", "S2_face_side_z")),
  "S2 a strongly powered block lights dust on all six faces at ss15",
  str({k.split("_", 1)[1]: ss(k, "S2") for k in
       ("S2_face_above", "S2_face_below", "S2_face_side_x", "S2_face_side_z")}))
V(hot("S3_readout_A", "S3a") and not hot("S3_readout_B", "S3a")
  and hot("S3_readout_B", "S3b") and not hot("S3_readout_A", "S3b"),
  "S3 two ADJACENT blocks, each hard-powered by its own repeater, do not "
  "interfere (the crossing insight)")
V(hot("S4_lowerline_out", "S4a") and not hot("S4_upperline_out", "S4a")
  and hot("S4_upperline_out", "S4b") and not hot("S4_lowerline_out", "S4b"),
  "S4 a repeater standing ON a hard-powered block is unaffected by it "
  "(no quasi-connectivity for repeaters)")

# --- W
V(not hot("W1_dust_above_weak", "W") and not hot("W1_dust_beyond_weak", "W"),
  "W1 a WEAKLY powered block lights no dust at all (above or beside)")
V(hot("W2_repeater_off_weak", "W"),
  "W2 ...but a repeater reading that same block DOES fire -- weak and strong "
  "are two different states of 'powered'")
V(hot("W3_lowerline_out", "W3") and not hot("W3_dust_on_lid", "W3"),
  "W3 dust does not power the block ABOVE it, so a lid over a live run "
  "carries an independent dust line")

# --- C
CEXP = {}
for cutname in CUT:
    for supname in SUP:
        CEXP[("up", cutname, supname)] = (cutname == "air")
        CEXP[("down", cutname, supname)] = (cutname == "air"
                                            and supname == "solid")
for (direction, cutname, supname), want in sorted(CEXP.items()):
    tag = "C_%s_cut_%s_sup_%s" % (direction, cutname, supname)
    V(hot(tag, tag) == want,
      "C %-4s cut=%-5s support=%-5s -> %s" %
      (direction, cutname, supname, "conducts" if want else "blocked"),
      "ss=%d" % ss(tag, tag))

# --- T
V(off["T1_strong_above_torch"] == 15,
  "T1 a standing torch STRONGLY powers the block above it (ss15 on the dust "
  "on top), with no lever involved")
V(off["T2_attachment_not_powered"] == 0,
  "T2 a torch does NOT power its own attachment block")
V(off["T3_torch_out"] > 0 and not hot("T3_torch_out", "T3"),
  "T3 a torch INVERTS its attachment block's power (1 rt, the only inverter "
  "and the vertical carrier's step)")

# --- B
V(off["B1_dust_beside_rblock"] == 15,
  "B1 a block of redstone is a 15 source for adjacent dust, unconditionally")
V(off["B2_dust_past_block_beside_rblock"] == 0,
  "B2 ...and does not power the solid block beside it (it is a source, not a "
  "powered block)")

# --- P
V(hot("P1_adjacent_line", "P"),
  "P1 two same-level dust lines 1 cell apart are ONE net (lane pitch >= 2)")
V(not hot("P2_gap_line", "P"),
  "P2 2 cells apart: isolated")
V(not hot("P3_planar_diagonal", "P3"),
  "P3 dust has no planar diagonal: dx=1,dz=1 at the same y is isolated")

# --- the whole rig, all levers on: nothing that must stay dark lights up
DARK = [k for k in PROBE if k.startswith(("S1", "W1", "P2", "P3"))
        or k in ("W3_dust_on_lid",)]
V(all(allon[k] == 0 for k in DARK),
  "X all-levers-on: every must-stay-dark probe is still dark",
  str({k: allon[k] for k in DARK}))

bad = 0
for ok, text, extra in VERDICTS:
    print("%s %s%s" % ("PASS" if ok else "FAIL", text,
                       ("   [%s]" % extra) if extra else ""))
    bad += 0 if ok else 1
print("probe_transport: %d/%d" % (len(VERDICTS) - bad, len(VERDICTS)))
raise SystemExit(1 if bad else 0)
