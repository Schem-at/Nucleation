"""LEVEL CHANGE with no horizontal room to spend.

An 8-bit bundle has to climb 8 levels between ports 5 blocks apart in x.  The
router has exactly one way to change level -- the verified level-shift tile,
which trades 2 cells of straight horizontal run per y -- so it needs 23 cells
of run it does not have, and refuses with a very good error message.

Z02 is the same climb done in a 1x1 column per bit by a form that is already
measured and verified.  The two cards together are the argument: the capability
exists, the solver just cannot reach for it.
"""

N = 8

SCENARIO = {
    "id": "Z01_level_change_tight",
    "title": "Climb 8 levels with 5 blocks of horizontal room",
    "question": ("The ports are 5 apart in x and 8 apart in y.  What does the "
                 "solver do when a level change has nowhere to spread out?"),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "vertical",
         "anchor": [1, 2, 4], "width": N, "ty": "uint"},
        {"name": "a_out", "dir": "out", "form": "vertical",
         "anchor": [6, 10, 4], "width": N, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_climb", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:light_gray_concrete"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
             "label": "bundle arrives 8 levels up"},
        ],
    },
    "render": {"yaw": 145, "pitch": 24, "zoom": 2.0},
    "expect": "unsolved",
    "notes": ("The blocking assumption: ONE level-change mechanism, priced at "
              "2 horizontal cells per y.  The router's error is exemplary -- it "
              "names the tile, the run it needs, the run available, and two "
              "ways out -- but there is no denser form for it to fall back to."),
}
