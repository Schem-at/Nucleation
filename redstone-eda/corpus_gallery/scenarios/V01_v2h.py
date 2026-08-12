"""VERTICAL -> HORIZONTAL.  The driver presents the dense 2y stack, the sink
presents the flat 2-pitch form.  The forms differ, so the router has to notice
and stamp a pivot; the scenario never asks for one.
"""

N = 8

SCENARIO = {
    "id": "V01_v2h",
    "title": "Vertical stack -> flat plane (the router stamps the pivot)",
    "question": ("`a_in` is a dense 2y stack, `a_out` is a flat 2-pitch row "
                 "along +X.  Does the router detect the form mismatch and "
                 "insert a v2h pivot on its own?"),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "vertical",
         "anchor": [1, 2, 4], "width": N, "ty": "uint"},
        {"name": "a_out", "dir": "out", "form": "flat_x",
         "anchor": [10, 2, 22], "width": N, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_pivot", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:pink_concrete",
                   "transparent_block": "minecraft:pink_stained_glass"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
             "label": "stack -> plane, bit order preserved"},
        ],
    },
    "render": {"yaw": 145, "pitch": 24, "zoom": 1.75},
    "expect": "solved",
    "notes": "`pivot_for()` in crates/nucleation-routing/src/pivot.rs picks V2H.",
}
