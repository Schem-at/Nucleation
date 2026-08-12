"""CONTROL for X02: the identical geometry with the hex trunk IDLE.

X02 routes and passes DRC and then fails in simulation.  This entry exists to
say precisely why: with the analog carrier switched off, the very same route
conducts all twelve patterns.  The defect is therefore SIGNAL COUPLING between
the routed bundle and a live neighbour -- not geometry, not the router's
occupancy handling.
"""
import copy
import X02_hex_cross_vertical as live

SCENARIO = copy.deepcopy(live.SCENARIO)
SCENARIO["id"] = "X02b_hex_idle_vertical"
SCENARIO["title"] = "Control: the same crossing with the hex trunk idle"
SCENARIO["question"] = ("X02's route, with every hex injector off.  Does the "
                        "bundle conduct when its neighbour is quiet?")
# no hold_analog, no analog sweep: only the binary lanes, trunk unpowered
SCENARIO["verify"] = {"words": [
    {"in": "a_in", "out": "a_out", "patterns": "walking+extremes",
     "label": "binary lanes intact, hex trunk unpowered"}]}
SCENARIO["expect"] = "solved"
SCENARIO["notes"] = ("Read together with X02: same cells, same DRC, and the "
                     "only difference is whether the analog carrier is live.")
