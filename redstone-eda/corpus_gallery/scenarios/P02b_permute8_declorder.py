"""CONTROL for P02: the identical permutation, handed over in DECLARATION
order instead of hardest-first.

Same ports, same nets, same volume -- only the order of the eight `route_bus`
calls differs.  Seven nets route and the eighth is refused because an earlier
net already owns the cells its level-shift tile needs.  The router has no
rip-up-and-retry and no global ordering heuristic, so a caller who lists their
nets in the natural order gets a partial route and a refusal.
"""
import copy
import P02_permute8 as good

SCENARIO = copy.deepcopy(good.SCENARIO)
SCENARIO["id"] = "P02b_permute8_declorder"
SCENARIO["title"] = "Control: the same permutation in declaration order"
SCENARIO["question"] = ("P02's nets, reordered.  Does the solver recover from "
                        "a bad net order on its own?")
SCENARIO["buses"] = [
    {"name": "net_a%d" % i, "driver": "a%d" % i, "sinks": ["b%d" % good.P[i]]}
    for i in range(8)]
SCENARIO["expect"] = "unsolved"
SCENARIO["notes"] = ("The blocking assumption: nets are routed greedily in the "
                     "order given, with no rip-up-and-retry.  Ordering is the "
                     "caller's problem, and nothing in the API says so.")
