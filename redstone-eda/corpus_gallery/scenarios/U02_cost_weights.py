"""UNSOLVED BY CONSTRUCTION: choose the cost weights.

`src/design.rs` defines `BusCost` with three presets -- `default()` (balanced:
length 1, delay 4, skew 8, coherence 6, footprint 0.5), `compact()` (length 2,
delay 1, skew 4, coherence 6, footprint 2) and a latency preset (length 0.5,
delay 16, skew 24, coherence 3, footprint 0.25) -- and `route_bus` hard-codes
`cost: BusCost::default()` at src/design.rs:2341.  No setter is exposed on
`Design`, and `src/bridge/design.rs` contains no occurrence of the word `cost`.

This entry asks for the latency preset the only way a caller can try: by
putting the weights in the net-class rule.  `NetClassRule` does not deny
unknown fields, so watch whether the request is REFUSED or silently dropped --
and compare the geometry to C01.
"""
import _shared

SCENARIO = _shared.serpentine(
    "U02_cost_weights",
    "Choosing cost weights: length vs delay vs footprint",
    "Ask for the latency-optimised weights (delay 16, skew 24, length 0.5). "
    "Does anything change against C01?",
    rule={"y_band": [0, 20], "cost": {"length": 0.5, "delay": 16.0,
                                      "skew": 24.0, "coherence": 3.0,
                                      "footprint": 0.25}},
    notes="If the cell count, delay and footprint equal C01 exactly, the "
          "weights were ignored -- the router is not tunable from any binding.",
)
SCENARIO["expect"] = "unsolved"
SCENARIO["title"] = "Choosing cost weights: length vs delay vs footprint"
SCENARIO["question"] = SCENARIO["question"]

SCENARIO["config_probe"] = {"baseline": 'C01_cfg_default', "expect_change": True}
