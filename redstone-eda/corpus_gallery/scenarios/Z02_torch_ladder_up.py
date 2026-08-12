"""The dense vertical-UP form, verified here and UNREACHABLE from the solver.

`vforms.ladder_bus` builds a torch ladder per bit: a 1x1 column, 2 y per torch,
1 gt per y, output refreshed to 15 so there is no reach limit and no repeater is
ever needed.  Eight bits at x-pitch 1 climb 8 levels in an 8x1 footprint -- one
xz cell per bit, against the level-shift tile's 2 horizontal cells per y that
Z01 could not afford.

This entry is NOT solved.  The geometry comes from `vforms.py`'s own verified
constructor, not from `route_bus`, and the corpus rule is that only solver
output counts as solved.  What it does establish, by driving the real thing in
mc-tick, is that the form works -- so the gap is selection, not physics.

Trap avoided (per the vertical-transport notes): these forms fail DEAD at least
as often as they fail leaky, so a quiet neighbour proves nothing. The walking
-ones sweep here checks that every bit ARRIVES with the right value, which a
severed tower fails.
"""

N, TORCHES = 8, 4          # 4 torches = 8 y of climb, even count = non-inverting

SCENARIO = {
    "id": "Z02_torch_ladder_up",
    "title": "Dense vertical UP: 8 bits, 8 levels, one xz cell per bit",
    "question": ("The climb Z01 refused, in the form that is already measured. "
                 "Does it carry all eight bits, and can the solver ask for it?"),
    "fixtures": [
        {"kind": "torch_ladder_bus", "name": "ladder", "at": [2, 1, 6],
         "nbits": N, "torches": TORCHES, "axis": "x"},
    ],
    "ports": [],
    "buses": [],
    "solver_produced": False,
    "blocked_by": ("The form is verified in simulation on this very build, but "
                   "no `route_bus` call can produce it: the router has one "
                   "level-change mechanism (the 2-cells-per-y level-shift "
                   "tile) and no way to select the torch ladder."),
    "verify": {
        "words": [
            {"in": "ladder_in", "out": "ladder_out", "patterns": "exhaustive",
             "label": "all 256 patterns climb 8 levels, no bit lost or shorted"},
        ],
    },
    "render": {"yaw": 140, "pitch": 22, "zoom": 2.2},
    "expect": "unsolved",
    "notes": ("Density: 1.000 xz cell per bit per level, vs the level-shift "
              "tile's 2 horizontal cells per y for the whole bundle. The "
              "constraint that makes it work is at the PORTS, not the towers -- "
              "entries must alternate +z/-z sides or the pitch-1 dusts merge "
              "into a T that stops pointing into the torch bases and the whole "
              "array reads a constant."),
}
