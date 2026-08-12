"""CONFIG 2/4 -- a LAYER assignment. `NetClassRule.y_band` says which y band
this net class may occupy."""
import _shared

SCENARIO = _shared.serpentine(
    "C02_cfg_yband",
    "Config B: y_band layer assignment",
    "Same problem, plus a net-class rule confining the bus to y 0..20. Does "
    "the route change, and does check() enforce the band?",
    rule={"y_band": [0, 20]},
    notes="If the geometry is byte-identical to C01, the rule was CHECKED "
          "after the fact rather than used to route -- which is what "
          "src/design.rs:4298 does.",
)

SCENARIO["config_probe"] = {"baseline": 'C01_cfg_default', "expect_change": True}
