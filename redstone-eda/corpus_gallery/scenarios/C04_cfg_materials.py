"""CONFIG 4/4 -- a MATERIAL policy. `Style` picks the structural block and the
transparent block the router uses for cut/insulate cells."""
import _shared

SCENARIO = _shared.serpentine(
    "C04_cfg_materials",
    "Config D: material policy (structural + transparent block)",
    "Same problem, different materials. Does the router honour a per-bus "
    "material policy without changing the topology it found?",
    style={"bus_block": "minecraft:blue_concrete",
           "transparent_block": "minecraft:blue_stained_glass"},
    notes="Cell counts identical to C01 with a different palette is the "
          "RIGHT answer here: materials are a policy, not a topology.",
)

SCENARIO["config_probe"] = {"baseline": 'C01_cfg_default', "expect_change": False}
