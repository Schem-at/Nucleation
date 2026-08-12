"""HORIZONTAL -> VERTICAL: the return trip.  Same mismatch, opposite sign, so
the router must pick the h2v pivot rather than the v2h one."""

N = 8

SCENARIO = {
    "id": "V02_h2v",
    "title": "Flat plane -> vertical stack (the pivot, reversed)",
    "question": ("A flat 2-pitch row along +X drives a dense 2y stack.  Is "
                 "the reverse pivot available, or is only one direction "
                 "implemented?"),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "flat_x",
         "anchor": [2, 2, 2], "width": N, "ty": "uint", "feed": [0, 0, -1]},
        {"name": "a_out", "dir": "out", "form": "vertical",
         "anchor": [22, 2, 20], "width": N, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_pivot", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:purple_concrete",
                   "transparent_block": "minecraft:purple_stained_glass"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
             "label": "plane -> stack, bit order preserved"},
        ],
    },
    "render": {"yaw": 150, "pitch": 26, "zoom": 1.75},
    "expect": "solved",
    "notes": "",
}
