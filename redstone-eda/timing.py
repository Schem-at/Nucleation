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
"""
import collections


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


def report(ppa, sinks):
    arrival, whence = analyze(ppa)
    worst = max(sinks, key=lambda s: arrival.get(s, 0))
    print("predicted arrivals (redstone ticks; 1 rt = 2 game ticks):")
    for s in sinks[:6]:
        print("   %-8s %3d rt  (%d gt)" % (s, arrival[s], 2 * arrival[s]))
    print("critical path -> %s (%d rt = %d gt):" % (worst, arrival[worst], 2 * arrival[worst]))
    chain = critical_path(arrival, whence, worst)
    print("   " + " -> ".join("%s@%d" % (s, t) for s, t in chain))
    return arrival, worst


def measure(sim, lv, bits_a, bits_b):
    """Game ticks for the circuit to settle across a lever transition."""
    lv.set(bits_a)
    t0 = sim.sim.tick_count()
    lv.set(bits_b)
    return sim.sim.tick_count() - t0
