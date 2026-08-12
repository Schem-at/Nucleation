"""The same 90-degree corner as V03, with the sink 14 blocks closer.

V03 routes.  This does not: the form adapter hits an internal plan conflict and
the bus is refused.  Nothing in the API tells a caller how much room a corner
needs, and the error names a cell rather than a clearance, so the only way to
find the limit is to try -- which is exactly the "gets in the way" failure mode
this corpus is meant to catch.
"""
import copy
import V03_flat90 as roomy

SCENARIO = copy.deepcopy(roomy.SCENARIO)
SCENARIO["id"] = "V03b_flat90_tight"
SCENARIO["title"] = "The same corner, 14 blocks tighter: refused"
SCENARIO["question"] = ("How much room does the flat 90-degree corner need, "
                        "and does the router say so before it fails?")
for p in SCENARIO["ports"]:
    if p["name"] == "a_out":
        p["anchor"] = [26, 2, 6]
SCENARIO["render"] = {"yaw": 140, "pitch": 30, "zoom": 1.9}
SCENARIO["expect"] = "unsolved"
SCENARIO["notes"] = ("The blocking assumption: the pivot tiles are stamped "
                     "from a fixed plan with no clearance query, so tight "
                     "geometry fails late, inside the adapter.")
