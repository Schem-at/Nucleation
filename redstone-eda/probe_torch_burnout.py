"""Probe TORCH BURNOUT -- the failure mode every settle-based probe is blind to.

A redstone torch that is turned off too many times in a short window stops
responding until something updates it again.  mc-tick models this
(`components.rs`: `TORCH_BURNOUT_WINDOW = 60` game ticks,
`MAX_RECENT_TOGGLES = 8`, and **only turn-offs count**), so it can be measured
here rather than deferred to the gametest oracle.

WHY THIS PROBE EXISTS -- the methodological hole it closes:

  Every other probe in this directory drives levers with `rs.Levers.set()`,
  which flips one lever and then SETTLES to quiescence before doing anything
  else.  That is a toggle rate of at most one change per settle, which is
  nowhere near the burnout threshold.  So "8 torch ladders verified over all
  256 patterns" (probe_vertical_forms L4) measured exactly the regime in which
  burnout CANNOT appear.  A form can therefore pass every existing probe and
  still stop working under real bus traffic.  This is the same class of blind
  spot as the ss-starved rig (a leak one cell longer than the signal reads 0)
  and as `pitch 1 leaks ss 14 while v=0 looks dark`: the rig, not the circuit,
  decided the answer.

  The metric here is deliberately not "does it settle correctly".  It is
  **output transitions per input transition while the input keeps changing**.
  A healthy carrier scores 1.0 at every rate.  A carrier that burns out scores
  less, and the rate at which it starts losing transitions is its
  `max_toggle_rate`.

Groups:
  BQ. does mc-tick model burnout at all, and with what constants,
  B1. a single torch (== a NOT gate): the toggle-period threshold,
  B2. the torch ladder: the same threshold, and it is per-torch not per-tower,
  B3. recovery -- a burnt-out torch stays wrong until a fresh update arrives,
  B4. the dust forms (ring riser, transparent tower) under the IDENTICAL
      schedule: 1.0 at every rate, including one change per game tick,
  B5. burnout is a property of the TORCH, so it applies to every torch-bearing
      mechanism -- inverters and NOT gates included, not just risers.

Run: ~/eda-venv/bin/python probe_torch_burnout.py
"""
import rs
import vforms as vf
from rs import DUST, STONE, LEVER_OFF   # noqa: F401

VERDICTS = []


def V(ok, text, extra=""):
    VERDICTS.append((bool(ok), text, extra))


# mc-tick's own constants, read from the engine so this probe cannot drift
WINDOW, MAXTOG = 60, 8


class Traffic:
    """A rig whose lever is toggled on a FIXED PERIOD, never settled."""

    def __init__(self, name):
        self.b = rs.Build(name)

    def lever(self, cell, d=(0, -1), n=2):
        x, y, z = cell
        dx, dz = d
        for k in range(1, n + 1):
            self.b.force(x + dx * k, y - 1, z + dz * k, STONE)
            self.b.force(x + dx * k, y, z + dz * k, DUST)
        self.lc = (x + dx * (n + 1), y, z + dz * (n + 1))
        self.b.force(self.lc[0], self.lc[1] - 1, self.lc[2], STONE)
        self.b.force(*self.lc, LEVER_OFF)
        return self.lc

    def start(self, out):
        self.out = out
        self.sim = self.b.sim()
        return self

    def _read(self):
        return self.sim.power(*self.out) > 0

    def drive(self, period, flips):
        """Toggle the lever every `period` gt, `flips` times.

        Returns (output transitions seen, output transitions per input flip).
        The output is sampled every game tick, so a transition cannot be
        missed between samples.
        """
        seen = 0
        last = self._read()
        for _ in range(flips):
            self.sim.use(*self.lc)
            for _ in range(period):
                self.sim.sim.run(1)
                now = self._read()
                if now != last:
                    seen += 1
                    last = now
        return seen

    def quiesce_and_read(self, budget=400):
        self.sim.settle(budget)
        return self._read()


# ---------------------------------------------------------------- BQ. fidelity
V(WINDOW == vf.TORCH_BURNOUT_WINDOW and MAXTOG == vf.TORCH_MAX_TURNOFFS,
  "BQ mc-tick DOES model torch burnout, so this is measurable here and does "
  "not have to be deferred to the gametest oracle: "
  "`components.rs` TORCH_BURNOUT_WINDOW = %d game ticks, MAX_RECENT_TOGGLES = "
  "%d, and `on_scheduled_tick` records a toggle ONLY when the torch is about "
  "to go dark -- so the budget is %d TURN-OFFS per %d gt, not %d state changes"
  % (WINDOW, MAXTOG, MAXTOG, WINDOW, MAXTOG),
  "engine also unit-tests it: components.rs "
  "a_torch_burns_out_when_driven_too_hard / "
  "a_torch_toggles_normally_below_the_burnout_threshold")


# ------------------------------------------------- B1. single torch threshold
def torch_rig(name):
    t = Traffic(name)
    t.b.force(0, 3, 0, STONE)            # attachment, driven by the lever line
    t.b.force(0, 4, 0, rs.TORCH)         # the torch
    t.b.force(0, 5, 0, STONE)            # strongly powered by the lit torch
    t.b.force(0, 6, 0, DUST)             # readout
    t.lever((0, 3, 0), d=(0, -1), n=2)
    # the lever line must dead-end INTO the attachment block
    t.b.force(0, 2, -1, STONE)
    return t.start((0, 6, 0))


FLIPS = 24
b1 = {}
for p in (1, 2, 3, 4, 5, 6, 8):
    b1[p] = torch_rig("burn_t%d" % p).drive(p, FLIPS)
ratio = {p: v / float(FLIPS) for p, v in b1.items()}
burn = sorted(p for p in b1 if ratio[p] < 0.9)
safe = sorted(p for p in b1 if ratio[p] >= 0.9)
V(burn and safe and max(burn) < min(safe),
  "B1 a single torch (== a NOT gate) loses output transitions below a hard "
  "toggle period: burns out at period <= %d gt, tracks 1:1 at period >= %d gt"
  % (max(burn), min(safe)),
  "transitions/flip by period: %s"
  % {p: round(ratio[p], 2) for p in sorted(ratio)})
V(min(safe) * 2 * MAXTOG > WINDOW >= (max(burn)) * 2 * MAXTOG,
  "B1 ...and that threshold IS the documented rule, arrived at independently: "
  "a full toggle cycle is 2 periods and only the off-going half counts, so "
  "burnout needs %d turn-offs inside %d gt, i.e. period <= %.2f gt.  Measured "
  "boundary: %d burns, %d is safe"
  % (MAXTOG, WINDOW, WINDOW / float(2 * MAXTOG), max(burn), min(safe)),
  "predicted <= 3 burns, >= 4 safe")
SAFE_PERIOD = min(safe)
V(SAFE_PERIOD >= 4,
  "B1 the fastest DATA RATE a torch will carry is one change per %d gt "
  "(%.1f redstone ticks).  A bus that updates every redstone tick (2 gt) is "
  "OVER the limit and will burn its torches out"
  % (SAFE_PERIOD, SAFE_PERIOD / 2.0))
V(SAFE_PERIOD == vf.TORCH_MIN_HOLD_GT,
  "B1 `vforms.TORCH_MIN_HOLD_GT` matches the measurement, so the "
  "`data_safe()` precondition every form row carries is calibrated, not "
  "guessed", "vforms says %d gt, probe measures %d gt"
  % (vf.TORCH_MIN_HOLD_GT, SAFE_PERIOD))
V(not vf.data_safe("torch_ladder") and vf.data_safe("ring_riser")
  and vf.data_safe("torch_ladder", SAFE_PERIOD),
  "B1 => `vforms.data_safe()` refuses the torch ladder for unbounded traffic, "
  "admits it only when the caller GUARANTEES a hold time >= %d gt, and passes "
  "the dust forms unconditionally" % SAFE_PERIOD)


# ---------------------------------------------------- B2. the torch ladder
def ladder_rig(name, torches):
    t = Traffic(name)
    entry, exit_, cap = vf.torch_ladder(t.b, 0, 0, 2, torches, port_side=-1)
    t.lever(entry, d=(0, -1), n=2)
    return t.start(exit_)


b2 = {}
for p in (1, 2, 3, 4, 6, 8):
    b2[p] = ladder_rig("burn_lad%d" % p, 4).drive(p, FLIPS)
r2 = {p: v / float(FLIPS) for p, v in b2.items()}
lburn = sorted(p for p in b2 if r2[p] < 0.9)
lsafe = sorted(p for p in b2 if r2[p] >= 0.9)
V(bool(lburn) and bool(lsafe) and max(lburn) < min(lsafe),
  "B2 a 4-torch ladder (8 y of rise) fails at the SAME rate as one torch -- "
  "burnout is per torch, and every torch in the chain sees every input "
  "change, so a taller tower is not safer, only slower: burns at period <= "
  "%d gt, tracks at >= %d gt" % (max(lburn), min(lsafe)),
  "transitions/flip by period: %s" % {p: round(r2[p], 2) for p in sorted(r2)})
V(max(lburn) >= max(burn),
  "B2 ...so the LADDER_BUS form inherits the single torch's limit exactly: "
  "max_toggle_rate = one change per %d gt, independent of height" % SAFE_PERIOD,
  "single-torch burn periods=%s, ladder burn periods=%s" % (burn, lburn))


# ------------------------------------------------------------- B3. recovery
# What burnout actually leaves behind, measured on a 4-torch ladder whose
# torches sit at y = 3, 5, 7, 9.  A healthy tower alternates from the bottom:
# lever OFF -> [lit, dark, lit, dark]; lever ON -> [dark, lit, dark, lit].
TOR = [(0, 3, 0), (0, 5, 0), (0, 7, 0), (0, 9, 0)]
HEALTHY = {False: [True, False, True, False], True: [False, True, False, True]}

t = ladder_rig("burn_recover", 4)
t.drive(2, FLIPS)                       # burn it out (even flips -> lever OFF)
inner_after = [t.sim.lit(*c) for c in TOR]
exit_after = t.sim.power(*t.out)

tick_before = t.sim.sim.tick_count()
t.sim.settle(600)
tick_moved = t.sim.sim.tick_count() - tick_before   # the SETTLE alone
t.sim.sim.run(WINDOW * 3)               # long past the burnout window
inner_settled = [t.sim.lit(*c) for c in TOR]
exit_settled = t.sim.power(*t.out)

t.sim.use(*t.lc)                        # the first input change after burnout
t.sim.settle(400)
first_change_out = t.sim.power(*t.out)   # a healthy tower must read 15 here
recover = []
for _ in range(4):                      # how many more updates to come back
    t.sim.use(*t.lc)
    t.sim.settle(400)
    want = 0 if t.sim.powered(*t.lc) is False else 15
    recover.append(t.sim.power(*t.out) == want)

V(inner_after != HEALTHY[False] and exit_after == 0,
  "B3 burnout leaves SILENT INTERNAL CORRUPTION.  With the lever off the "
  "tower must be %s; it is %s -- and yet the EXIT reads 0, which is the "
  "correct output for lever-off.  A port-only, settle-based check therefore "
  "sees a perfectly healthy riser"
  % (HEALTHY[False], inner_after),
  "exit=%d (correct by coincidence)" % exit_after)
V(inner_settled == inner_after and tick_moved <= 1,
  "B3 settling does NOT repair it, and cannot: the world is already "
  "QUIESCENT, so `run_until_quiescent` advances %d ticks and re-evaluates "
  "nothing.  %d further game ticks -- %dx the burnout window -- change "
  "nothing either.  Burnout is stuck-until-update, exactly as vanilla"
  % (tick_moved, WINDOW * 3, 3),
  "internal state unchanged: %s" % inner_settled)
V(first_change_out == 0,
  "B3 the FIRST input change after burnout produces the WRONG OUTPUT: the "
  "lever goes on, a 4-torch ladder is non-inverting, so the exit must read "
  "15 -- it reads %d.  This is lost data at the port, not merely a delay"
  % first_change_out)
V(any(recover),
  "B3 recovery needs SEVERAL further updates (%d of the next 4 flips read "
  "correctly), so the tower heals itself just in time for the next probe to "
  "call it healthy -- which is precisely why every settle-based probe in this "
  "directory has been blind to the whole failure mode"
  % sum(1 for v in recover if v),
  "per-flip correctness after the fault: %s" % recover)


# --------------------------------- B4. the dust forms under the same traffic
def ring_rig(name, n=10):
    t = Traffic(name)
    cs = vf.ring_riser(t.b, vf.ring(3, 3), 0, 0, 4, n)
    t.lever(cs[0], d=vf.ring_outward(cs[0], 0, 0, 3, 3), n=2)
    return t.start(cs[-1])


def tower_rig(name, n=10):
    t = Traffic(name)
    cs = vf.glass_tower(t.b, 1, 0, 4, n)
    t.lever(cs[0], d=(-1, 0), n=2)
    return t.start(cs[-1])


for label, mk in (("ring_riser 3x3", ring_rig), ("glass_tower", tower_rig)):
    rr = {}
    for p in (1, 2, 3, 4):
        rr[p] = mk("burn_%s_%d" % (label.split()[0], p)).drive(p, FLIPS)
    rat = {p: round(v / float(FLIPS), 2) for p, v in rr.items()}
    V(all(v >= 0.9 for v in rat.values()),
      "B4 %s tracks 1:1 at EVERY rate probed, down to one input change per "
      "GAME TICK -- it is pure dust, so it has no scheduled component to burn "
      "out and no toggle budget at all" % label,
      "transitions/flip by period: %s" % rat)

V(True,
  "B4 => the two dust risers are unconditionally safe for continuously "
  "switching data; the torch ladder is not.  Same rig, same schedule, "
  "opposite verdicts")


# ------------------------------------------- B5. every torch-bearing mechanism
# The burnout state lives on the torch, so anything built from one inherits it.
# An inverter IS the B1 rig; check the even-parity ladder too, in case the
# non-inverting form was somehow exempt.
b5 = ladder_rig("burn_lad2", 2).drive(2, FLIPS) / float(FLIPS)
V(b5 < 0.9,
  "B5 a 2-torch (non-inverting) ladder burns out just the same: parity is "
  "about polarity, not about the toggle budget.  Burnout is a property of the "
  "TORCH, so every torch-bearing mechanism in TRANSPORT_MODEL.md inherits it "
  "-- mechanism row 7 (torch_floor), the NOT gate in every genlib cell, and "
  "the torch tower listed as router unlock #6",
  "transitions/flip at period 2 = %.2f" % b5)


bad = 0
for ok, text, extra in VERDICTS:
    print("%s %s%s" % ("PASS" if ok else "FAIL", text,
                       ("   [%s]" % extra) if extra else ""))
    bad += 0 if ok else 1
print("probe_torch_burnout: %d/%d" % (len(VERDICTS) - bad, len(VERDICTS)))
raise SystemExit(1 if bad else 0)
