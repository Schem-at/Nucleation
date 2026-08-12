"""UNSOLVED BY CONSTRUCTION: an EXCLUSION ZONE the router respects.

`src/io_contract/routing.rs` defines `RoutingRegion` (a union of include boxes
minus exclude boxes) with a legality predicate `contains()`, a metadata
round-trip, and Insign authoring (`#route_zone="name exclude"`).  Grep for its
consumers: `insign_ext.rs` parses them and `routing.rs` tests them.  `design.rs`
never mentions `RoutingRegion` at all.

So a designer can author a keepout in-world or by API and the router will drive
straight through it.  The only thing that actually keeps a route out of a
volume today is a SOLID BLOCK in it (which O01/O02 prove works).

This entry asks for a region by name via the one field that exists for it,
`NetClassRule.region`, over the C01 geometry.
"""
import _shared

SCENARIO = _shared.serpentine(
    "U03_exclusion_zone",
    "A declared exclusion zone (RoutingRegion) the router must respect",
    "Name a routing region on the net class and ask the router to stay in it. "
    "Is the constraint honoured, checked, or dropped?",
    rule={"region": "bus_north", "spacing": 2,
          "direction_bias": "X"},
    notes="Compare geometry with C01. Identical geometry plus a silent check "
          "means region/spacing/direction_bias are dead fields on the "
          "routing path.",
)
SCENARIO["expect"] = "unsolved"

SCENARIO["config_probe"] = {"baseline": 'C01_cfg_default', "expect_change": True}
