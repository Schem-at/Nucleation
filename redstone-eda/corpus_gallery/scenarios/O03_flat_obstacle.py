"""The minimal reproducer for the defect X03 hits: a FLAT bundle detouring
around an obstacle comes out broken while the router reports success.

No hex, no analog, no crossing -- one flat 2-pitch 8-bit bundle, one plain wall
across its path.  The router returns state `routed`, `check()` is clean, and in
simulation lanes are dead or shorted to each other.  A router that reports
success for a bus that does not conduct is the single most important thing in
this corpus to fix.
"""

N = 8

SCENARIO = {
    "id": "O03_flat_obstacle",
    "title": "FLAT bundle + obstacle: routed, DRC-clean, does not conduct",
    "question": ("A flat 2-pitch 8-bit bundle has to climb a 1-wide wall. "
                 "Does the reported route actually carry the bits?"),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "flat_z",
         "anchor": [1, 2, 2], "width": N, "ty": "uint", "feed": [-1, 0, 0]},
        {"name": "a_out", "dir": "out", "form": "flat_z",
         "anchor": [20, 2, 2], "width": N, "ty": "uint"},
    ],
    "obstacles": [
        {"min": [11, 0, 0], "max": [11, 1, 17],
         "block": "minecraft:polished_andesite"},
    ],
    "buses": [
        {"name": "bus_flat", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:magenta_concrete",
                   "transparent_block": "minecraft:magenta_stained_glass"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
             "label": "walking ones through the detour"},
        ],
    },
    "render": {"yaw": 140, "pitch": 30, "zoom": 1.8},
    "expect": "unsolved",
    "notes": ("Compare with the same bundle on a CLEAR path, which passes "
              "12/12 -- so the defect is in the detour, not the flat form. "
              "Vertical-form bundles detour correctly (O01, O02)."),
}
