"""Behavioural (mixed-level) simulation of the accumulator -- E3 PoC.

No voxels, no mc-tick: each cell is its characterized (function, delay
table) pair.  The timing legality check uses ONLY numbers measured on the
real cells; if the schedule is legal, function-level stepping is exact.
Cross-checked against the mc-tick run recorded by accumulator.py.
"""
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))

# characterized numbers (all in game ticks, measured, exact -- see
# seq_README.md and accumulator.py)
DFF = {"setup": 0, "hold": 3, "min_pulse": 3, "clk_to_q": 10, "min_period": 20}
LOOP_MIN_PERIOD = 100      # clk->Q + Q->a corridor + FA ripple + sum->D
B_SETTLE_MIN = 80          # external-B change to D stable (measured sweep)
CLK_SKEW_PER_BIT = 2       # one refresh repeater per chained DFF cell


def legal(high, low, settle_b, bits=4):
    period = high + low
    checks = {
        "pulse width >= DFF min pulse + chain skew":
            high >= DFF["min_pulse"] + CLK_SKEW_PER_BIT * bits,
        "period >= sequential-loop min period": period >= LOOP_MIN_PERIOD,
        "B settle >= measured B-path setup": settle_b >= B_SETTLE_MIN,
        "low phase >= clk->Q (capture visible)": low >= DFF["clk_to_q"],
    }
    return all(checks.values()), checks


def run(b_seq, high, low, settle_b):
    ok, checks = legal(high, low, settle_b)
    if not ok:
        raise ValueError("illegal schedule: %s" % checks)
    q, out = 0, []
    for b in b_seq:                 # one rising edge per step
        q = (q + b) % 16            # register captures adder.sum = Q + B
        out.append(q)
    return out


def main():
    with open(os.path.join(HERE, "accumulator_trace.json")) as f:
        t = json.load(f)
    got = run(t["B"], t["high_gt"], t["low_gt"], t["settle_b_gt"])
    match = sum(1 for a, b in zip(got, t["Q_mc_tick"]) if a == b)
    print("functional sim vs mc-tick: %d/%d clock steps identical"
          % (match, len(got)))
    _, checks = legal(t["high_gt"], t["low_gt"], t["settle_b_gt"])
    for k, v in checks.items():
        print("   %-44s %s" % (k, "OK" if v else "VIOLATED"))
    return match == len(got)


if __name__ == "__main__":
    print("functional_sim:", "ALL PASS" if main() else "FAILURES")
