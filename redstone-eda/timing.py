"""Static timing analysis for PLA-compiled redstone.

Chip flows never tape out without STA; neither should a 150k-block circuit.
Delay model, in redstone ticks (1 redstone tick = 2 game ticks):

  * every torch inverts in 1 tick (tap torches and gate torches alike);
  * every repeater adds its delay (all of ours are delay=1);
  * dust is free.

A column therefore costs 2 ticks (tap torch + gate torch), and a signal's
arrival is the worst arrival over its taps, plus the repeaters the signal
passed through on the way to that tap.  Repeaters are counted per NET from the
built geometry -- every repeater is attributed to the signal whose dust it sits
between -- so the estimate is an upper bound (a tap early on a rail is charged
the whole rail's repeaters), which is the right direction to be wrong in.

validate(ppa, sim, lv, vector) then measures the real settle time of the worst
transition in game ticks for comparison.

BURNOUT (`burnout()` / `min_period()`, added after `probe_torch_burnout.py`)
---------------------------------------------------------------------------
Arrival time is only half of a timing closure.  A redstone torch that turns off
more than MAX_TURNOFFS times inside BURNOUT_WINDOW game ticks goes
stuck-until-update, and the fault is silent: the structure is internally corrupt
while its exit still reads plausibly, settling repairs nothing, and the next
input change emits the wrong value.  So a design can pass arrival-time STA and
still be wrong at its intended clock rate.

Two terms, both per-torch and both in GAME ticks:

  TORCH_MIN_HOLD_GT = BURNOUT_WINDOW / (2 * MAX_TURNOFFS), rounded up = 4 gt.
    A torch toggles twice per full cycle of its input and only the off-going
    half is charged, so this is the shortest a torch's input may be held.
    Measured independently in `probe_torch_burnout.py` B1.

  TORCH_MIN_PERIOD_GT = 2 * TORCH_MIN_HOLD_GT = 8 gt.
    The floor on a CLOCK period for any design with a torch on a toggling path.
    Every NOT gate is a torch and the PLA puts a tap torch and a gate torch in
    every column, so this floor applies to every design this compiler emits.

The binding constraint is therefore max(arrival, torch floor), and which one
binds is a real question rather than a formality: for the datapaths in this
directory arrival wins by an order of magnitude (`probe_rate_limits.py` R2), but
for a single gate or a short chain the torch floor wins, and a design that
pipelines its way to a short critical path walks straight into it.
"""
import collections
import math

# mc-tick's constants (crates/mc-tick .. components.rs: TORCH_BURNOUT_WINDOW,
# MAX_RECENT_TOGGLES), mirrored in `vforms.py` and `ratedrive.py`.
BURNOUT_WINDOW_GT = 60
MAX_TURNOFFS = 8
TORCH_MIN_HOLD_GT = int(math.ceil(BURNOUT_WINDOW_GT / float(2 * MAX_TURNOFFS)))
TORCH_MIN_PERIOD_GT = 2 * TORCH_MIN_HOLD_GT


def net_repeaters(build, labels):
    """signal -> list of (x, y) for every repeater inside that signal's net."""
    reps = collections.defaultdict(list)
    for (x, y, z), blk in build.cells.items():
        if "repeater" not in blk:
            continue
        for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            lab = labels.get((x + dx, y, z + dz))
            if lab and "#" not in lab:
                reps[lab].append((x, y))
                break
    return reps


def reps_to_tap(reps, sig, tap_x):
    """Repeaters a signal passes before a tap at x=tap_x.  Rails run west to
    east at y=1, so rail repeaters east of the tap are not traversed; route
    and collector repeaters (y>1) always are."""
    return sum(1 for (x, y) in reps.get(sig, ()) if y != 1 or x < tap_x)


def analyze(ppa):
    """Per-signal arrival time in redstone ticks + the critical path."""
    reps = net_repeaters(ppa.b, ppa.labels)
    arrival, whence = {}, {}
    for sig, _pos in ppa.levers:
        arrival[sig] = 0

    # walk stages in order; within a stage: inverters first, then nodes
    import build_ppa as bp
    for st in ppa.stages:
        for sl, src, dst in st["inverters"]:
            tap_x = bp.sx(sl) + bp.INV_X
            arrival[dst] = arrival.get(src, 0) + reps_to_tap(reps, src, tap_x) + 1
            whence[dst] = src
        for sl, groups in st["nodes"]:
            for gi, (out, terms) in enumerate(groups):
                cols = ((bp.COL_A + bp.COL_B + bp.EXTRA_COLS) if gi == 0
                        else bp.COL_B)[:len(terms)]
                best, why = 0, None
                for term, col in zip(terms, cols):
                    tap_x = bp.sx(sl) + col
                    for sig in term:
                        t = arrival.get(sig, 0) + reps_to_tap(reps, sig, tap_x)
                        if t > best:
                            best, why = t, sig
                # collector + route repeaters ride on the OUT net (y > 1),
                # charged when a consumer taps it
                arrival[out] = best + 2
                whence[out] = why
    return arrival, whence


def critical_path(arrival, whence, sink):
    path, s = [], sink
    while s is not None:
        path.append((s, arrival.get(s, 0)))
        s = whence.get(s)
    return list(reversed(path))


# ------------------------------------------------------------------- burnout

def torch_count(build):
    """Redstone torches in a build -- wall torches included.

    Every one of them is a component with a toggle budget, and every NOT gate is
    one of them.  A build with none (the `cells.py` genlib, which is repeaters
    and comparators) has no burnout term at all.
    """
    return sum(1 for blk in build.cells.values() if "redstone_torch" in blk
               or "redstone_wall_torch" in blk)


def torch_positions(build):
    """Every torch cell, so the burnout term can be measured and not assumed."""
    return [p for p, blk in sorted(build.cells.items())
            if "redstone_torch" in blk or "redstone_wall_torch" in blk]


def peak_turnoffs(sim, positions, apply_change, window_gt=BURNOUT_WINDOW_GT):
    """Worst per-torch TURN-OFF count in the `window_gt` ticks after one change.

    This calibrates `toggles_per_change` instead of guessing it, and it answers a
    question worth asking directly: can a SINGLE input change, all by itself,
    glitch one torch past its budget?  If it can, a circuit burns out in the
    quasi-static regime too and the whole settle-based corpus is in trouble; if
    it cannot, the exhaustive results stand and only the rate claims need
    narrowing.

    `apply_change()` performs one input change and returns immediately (it must
    NOT settle -- the point is to watch the transient).
    """
    was = {p: sim.lit(*p) for p in positions}
    off = dict.fromkeys(positions, 0)
    apply_change()
    for _ in range(window_gt):
        sim.sim.run(1)
        for p in positions:
            now = sim.lit(*p)
            if was[p] is True and now is False:
                off[p] += 1
            was[p] = now
    worst = max(off, key=off.get) if off else None
    return (off[worst] if worst else 0), worst, off


def min_hold_gt(toggles_per_change=0.5):
    """Shortest safe input hold, in game ticks, for a torch whose input makes
    `toggles_per_change` TURN-OFFS per input change.

    A clean alternating signal turns a torch off once every two changes, hence
    the 0.5 default and the 4 gt result.  A net that glitches -- a ripple carry
    settling through several intermediate values -- charges its torches more
    than that per change, and the floor rises in proportion.  Pass a measured
    figure when you have one; the default is the optimistic end.
    """
    return int(math.ceil(BURNOUT_WINDOW_GT * toggles_per_change / MAX_TURNOFFS))


def burnout(build, toggles_per_change=0.5):
    """The burnout term for a build: (torches, min hold gt, min period gt).

    `min period` is the clocked-design floor: a clock is the fastest-toggling net
    in any design, and a torch anywhere on a toggling path caps it.
    """
    n = torch_count(build)
    if not n:
        return 0, 0, 0
    hold = min_hold_gt(toggles_per_change)
    return n, hold, 2 * hold


def min_period(ppa, sinks, toggles_per_change=0.5):
    """Combine both terms.  Returns (min period gt, binding, detail).

    `binding` is "arrival" or "burnout" -- which term actually sets the period.
    This is the number a design has to respect, and until now STA reported only
    the first half of it.
    """
    arrival, _whence = analyze(ppa)
    worst = max(sinks, key=lambda s: arrival.get(s, 0))
    arrival_gt = 2 * arrival[worst]
    n, hold, floor = burnout(ppa.b, toggles_per_change)
    period = max(arrival_gt, floor)
    binding = "arrival" if arrival_gt >= floor else "burnout"
    return period, binding, dict(arrival_gt=arrival_gt, worst=worst,
                                 torches=n, torch_hold_gt=hold,
                                 torch_period_gt=floor)


def report(ppa, sinks, toggles_per_change=0.5):
    arrival, whence = analyze(ppa)
    worst = max(sinks, key=lambda s: arrival.get(s, 0))
    print("predicted arrivals (redstone ticks; 1 rt = 2 game ticks):")
    for s in sinks[:6]:
        print("   %-8s %3d rt  (%d gt)" % (s, arrival[s], 2 * arrival[s]))
    print("critical path -> %s (%d rt = %d gt):" % (worst, arrival[worst], 2 * arrival[worst]))
    chain = critical_path(arrival, whence, worst)
    print("   " + " -> ".join("%s@%d" % (s, t) for s, t in chain))

    period, binding, d = min_period(ppa, sinks, toggles_per_change)
    print("burnout term (%d torches, %.2f turn-offs per input change):"
          % (d["torches"], toggles_per_change))
    if d["torches"]:
        print("   torch min hold   %3d gt   (one input change per torch)"
              % d["torch_hold_gt"])
        print("   torch min period %3d gt   (clocked designs)"
              % d["torch_period_gt"])
    else:
        print("   none -- this build has no torch, so no toggle budget")
    print("MIN PERIOD %d gt, binding constraint: %s "
          "(arrival %d gt vs torch floor %d gt)"
          % (period, binding.upper(), d["arrival_gt"], d["torch_period_gt"]))
    return arrival, worst


def measure(sim, lv, bits_a, bits_b):
    """Game ticks for the circuit to settle across a lever transition."""
    lv.set(bits_a)
    t0 = sim.sim.tick_count()
    lv.set(bits_b)
    return sim.sim.tick_count() - t0


def _demo():
    """Report the binding constraint for each PLA circuit in this directory.

    Run: ~/eda-venv/bin/python timing.py
    """
    import build_alu
    import build_ppa as bp

    designs = [("kogge_stone_4bit", bp.PPA(4, name="k4"),
                ["s%d" % i for i in range(4)] + ["cout"]),
               ("kogge_stone_8bit", bp.PPA(8, name="k8"),
                ["s%d" % i for i in range(8)] + ["cout"]),
               ("kogge_stone_32bit", bp.PPA(32, name="k32"),
                ["s%d" % i for i in range(32)] + ["cout"]),
               ("alu_4bit", bp.PPA(4, make=build_alu.alu_netlist, name="a4"),
                ["out%d" % i for i in range(4)]),
               ("alu_8bit", bp.PPA(8, make=build_alu.alu_netlist, name="a8"),
                ["out%d" % i for i in range(8)])]
    print("STA with the burnout term (game ticks).  torch floor = %d gt hold / "
          "%d gt period" % (TORCH_MIN_HOLD_GT, TORCH_MIN_PERIOD_GT))
    print("%-20s %8s %8s %10s %10s  %s"
          % ("design", "torches", "arrival", "torch min", "min period",
             "binding"))
    for name, ppa, sinks in designs:
        ppa.build()
        period, binding, d = min_period(ppa, sinks)
        print("%-20s %8d %6d gt %7d gt %7d gt  %s"
              % (name, d["torches"], d["arrival_gt"], d["torch_period_gt"],
                 period, binding.upper()))
    print()
    print("Arrival binds everywhere in this flow, because a PLA column already "
          "costs two torches (4 gt) and no compiled design has a critical path "
          "anywhere near %d gt.  The term still has to be in the tool: it is "
          "the constraint that would bind a hand-built fast gadget, a pipelined "
          "design whose stage delay is short, or any clocked design that tries "
          "to run its clock faster than %d gt -- and nothing warned about that "
          "before." % (TORCH_MIN_PERIOD_GT, TORCH_MIN_PERIOD_GT))


if __name__ == "__main__":
    _demo()
