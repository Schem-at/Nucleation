"""CONFIG 3/4 -- a DELAY BUDGET. `NetClassRule.max_len_rt` is the maximum
route length in redstone ticks."""
import _shared

SCENARIO = _shared.serpentine(
    "C03_cfg_budget",
    "Config C: max_len_rt delay budget of 2 rt",
    "Same problem with a deliberately tight 2 rt budget (the default route needs 3). Does the router "
    "route SHORTER to fit the budget, or route as before and report the "
    "violation?",
    rule={"max_len_rt": 2},
    notes="MEASURED ANSWER: it routes exactly as C01 -- byte-identical "
          "geometry, 3 rt -- and then REPORTS the violation: check() comes "
          "back clean=False with `bus `bus_a`: max bit delay 3rt exceeds "
          "max_len_rt 2rt`. So the budget is a post-hoc assertion, not a "
          "routing objective. That is a defensible design (fail loudly rather "
          "than silently), but it means a caller cannot ask for a faster "
          "route -- only be told the one they got is too slow.",
)

SCENARIO["config_probe"] = {"baseline": 'C01_cfg_default', "expect_change": True}
