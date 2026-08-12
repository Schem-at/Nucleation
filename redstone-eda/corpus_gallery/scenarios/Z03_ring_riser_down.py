"""The descending form, verified here and UNREACHABLE from the solver.

There is no active descending carrier in redstone at all: a torch powers only
the block above it, and a dust's downward weak power is invisible to dust.  The
only active descent is 1 y per repeater station.  `vforms.ring_bus` instead
uses the passive ring riser: a chordless perimeter where y equals the path
index, so 1 y per cell at 0 gt and -1 ss per y.

A 3x3 ring holds floor(8/3) = 2 bits at phases 0 and 4 -- the 180-degree
offset -- and the drop is kept inside dust's reach so no station is needed.

Same verdict as Z02 and for the same reason: verified, not solver-reachable.
"""

SCENARIO = {
    "id": "Z03_ring_riser_down",
    "title": "Descending: a 180-degree tiled ring riser, 2 bits on one 3x3",
    "question": ("Descent has no active carrier.  Does the passive ring riser "
                 "carry two bits down 8 levels with the 180-degree phase "
                 "offset, and can the solver ask for it?"),
    "fixtures": [
        {"kind": "ring_riser_bus", "name": "ring", "at": [2, 2, 2],
         "size": [3, 3], "levels": 9, "sep": 3},
    ],
    "ports": [],
    "buses": [],
    "solver_produced": False,
    "blocked_by": ("Verified in simulation here, but unreachable from the "
                   "solver: `route_bus` has no descending form other than the "
                   "level-shift tile, and no way to express 'two bits sharing "
                   "one ring at 180 degrees'."),
    "verify": {
        "words": [
            {"in": "ring_in", "out": "ring_out", "patterns": "exhaustive",
             "label": "both bits descend 8 levels, neither dead nor shorted"},
        ],
    },
    "render": {"yaw": 135, "pitch": 26, "zoom": 2.3},
    "expect": "unsolved",
    "notes": ("Two bits on one 3x3 ring is 4.5 xz cells per bit; an 11x3 ring "
              "carries a byte at 4.125. Separation is the whole legality "
              "argument: 1 IS the step so the nets merge, 2 puts a foreign "
              "support directly above a dust and SEVERS both, 3 is the first "
              "legal value. A severed bit is simply quiet, so this entry "
              "checks arrival, not silence."),
}
