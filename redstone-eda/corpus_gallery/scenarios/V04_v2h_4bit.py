"""The SAME form mismatch as V01, at width 4 instead of 8.

`crates/nucleation-routing/src/pivot.rs` opens with `pub const PIVOT_BITS: u8
= 8;` and every pivot tile is generated over `0..PIVOT_BITS`.  If that constant
is really a constant rather than a parameter, this scenario cannot be solved --
and a router that only changes form at exactly 8 bits wide is not a router you
can use on daily work.  Recorded so the limit is a number, not a suspicion.
"""

N = 4

SCENARIO = {
    "id": "V04_v2h_4bit",
    "title": "Vertical -> flat at width 4 (probing the fixed pivot width)",
    "question": ("V01 at width 4.  Is the v2h pivot parametric in width, or "
                 "hard-wired to PIVOT_BITS = 8?"),
    "ports": [
        {"name": "a_in", "dir": "in", "form": "vertical",
         "anchor": [1, 2, 4], "width": N, "ty": "uint"},
        {"name": "a_out", "dir": "out", "form": "flat_x",
         "anchor": [10, 2, 20], "width": N, "ty": "uint"},
    ],
    "buses": [
        {"name": "bus_pivot4", "driver": "a_in", "sinks": ["a_out"],
         "style": {"bus_block": "minecraft:pink_concrete"}},
    ],
    "verify": {
        "words": [
            {"in": "a_in", "out": "a_out", "patterns": "exhaustive",
             "label": "4-bit stack -> plane, exhaustive"},
        ],
    },
    "render": {"yaw": 145, "pitch": 24, "zoom": 2.0},
    "expect": "solved",
    "notes": ("PREDICTION DISPROVED. `PIVOT_BITS = 8` is the pivot TILE's own "
          "width, not a limit on the bus: a 4-bit bundle pivots and passes "
          "all 16 patterns. Recorded because the constant reads like a "
          "limit and is not one."),
}
