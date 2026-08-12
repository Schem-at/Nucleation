"""FLAGSHIP (1/2): an 8-bit BINARY bundle crosses the hex ANALOG carrier.

The hex trunk is not solver output -- it is the measured mechanism from
`notes-hex-transport.md`, placed as loose blocks.  What the solver has to do is
get eight binary lanes past it without breaking either signal: the trunk's
devices own y=1 across x=6..8, which is exactly where the bundle's bit 0 wants
its support.

Verification is the point: the binary sweep runs with an analog level HELD LIVE
on the trunk, and the analog sweep runs with a binary pattern HELD on the
bundle.  A crossing that corrupts either side fails.
"""

N = 8

SCENARIO = {
    "id": "X02_hex_cross_vertical",
    "title": "8-bit binary bundle crossing a hex analog trunk (vertical form)",
    "question": ("The hex analog stage occupies y=0..1 at x=6..8.  Can the "
                 "router take an 8-bit vertical bundle straight through that "
                 "corridor with the analog value still exact at the tap?"),
    "fixtures": [
        {"kind": "hex_trunk", "name": "hex0", "at": [6, 0, 0],
         "values": [1, 3, 7, 11, 15]},
    ],
    "ports": [
        {"name": "a_in", "dir": "in", "form": "vertical",
         "anchor": [1, 2, 8], "width": N, "ty": "uint"},
        {"name": "a_out", "dir": "out", "form": "vertical",
         "anchor": [14, 2, 8], "width": N, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_bin", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:lime_concrete",
                   "transparent_block": "minecraft:lime_stained_glass"}},
    ],
    "verify": {
        "hold_analog": {"fixture": "hex0", "level": 11},
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
             "label": "binary lanes intact while hex carries 11"},
        ],
        "analog": [
            {"fixture": "hex0", "hold_ports": ["a_in"], "hold_bits": 0xAA,
             "also_read": ["a_out"],
             "label": "hex value exact while the bundle carries 0xAA"},
        ],
    },
    "render": {"yaw": 150, "pitch": 28, "zoom": 1.8},
    "expect": "unsolved",
    "notes": ("The hex trunk is a FIXTURE (probe_hex_transmit.py::Rig, 66/66), "
              "not router output; only the binary bundle is solved.\n\n"
              "MEASURED OUTCOME: the router routes it, `check()` is clean, and "
              "bit 0 comes out stuck HIGH whenever the trunk carries a value. "
              "The mechanism is exact and reproducible: the router detoured "
              "bit 0 to z=-1 and put its dust at (7,2,-1), (8,2,-1), (9,2,-1), "
              "which are DIAGONALLY adjacent to the trunk's live readout dust "
              "at (8,1,0) -- one down, one over, which is a conducting dust "
              "diagonal.  The router avoided the trunk's OCCUPIED cells "
              "correctly and then ran its own wire alongside a live one.\n\n"
              "X02b is the control: with the trunk unpowered the identical "
              "route passes 12/12.  So the defect is clearance, not geometry, "
              "and `NetClassRule.spacing` -- the field that exists for exactly "
              "this -- never reaches the router (see U03)."),
}
