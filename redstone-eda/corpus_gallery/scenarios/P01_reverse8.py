"""Lane REVERSAL: the sink declares its bits in the opposite order, so bit 0
has to travel to where bit 7 would have landed and every lane crosses every
other.  Nothing in the call says "reverse" -- the port declarations say it, and
the router has to realize the butterfly.
"""

N = 8

SCENARIO = {
    "id": "P01_reverse8",
    "title": "Bit reversal: an 8-lane butterfly from the port declarations",
    "question": ("`a_in` stacks bit 0 at the bottom, `a_out` stacks bit 0 at "
                 "the TOP (step -2y).  Can the router route a full reversal, "
                 "with all eight lanes crossing, and keep them isolated?"),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "vertical",
         "anchor": [1, 2, 4], "width": N, "ty": "uint"},
        {"name": "a_out", "dir": "out", "form": "vertical_desc",
         "anchor": [16, 16, 4], "width": N, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_rev", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:orange_concrete",
                   "transparent_block": "minecraft:orange_stained_glass"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
             "map": {"kind": "identity"},
             "label": "bit i arrives at the sink's bit i (i.e. reversed in y)"},
        ],
    },
    "render": {"yaw": 145, "pitch": 26, "zoom": 1.85},
    "expect": "solved",
    "notes": ("`map` is identity because the check is per DECLARED bit index: "
              "the reversal is geometric.  A walking-ones sweep is what proves "
              "it -- a bus that shorted two lanes would light two lamps."),
}
