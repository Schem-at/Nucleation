"""End-to-end test for the generated mc-tick bindings (TickSimulation).

Run via tests/run_python_mc_tick_test.sh, which builds the nanobind extension
against a `--features bridge-full` static lib and puts it on sys.path.

Replays the shulker-pipeline scenario the Rust case
(crates/mc-tick/tests/cases/shulker_pipeline.test.json) pins, and opens the
6x6 in-world door — asserting the same end states through the bindings.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import nucleation

ROOT = Path(__file__).resolve().parent.parent

passed = 0
failed = 0


def expect(cond: bool, what: str) -> None:
    global passed, failed
    if cond:
        passed += 1
        print(f"  ok  {what}")
    else:
        failed += 1
        print(f"  FAIL {what}")


snbt = (ROOT / "crates/mc-tick/tests/corpus/structures/shulker_pipeline.snbt").read_text()
sim = nucleation.TickSimulation.from_snbt(
    snbt, nucleation.TickSettleMode.Placement, 0, 0, 0, ""
)
sim.set_rng_seed(12345)

RB = "minecraft:redstone_block"
AIR = "minecraft:air"
actions = [
    (5, -1, 2, 1, RB),
    (30, 2, 0, 1, RB),
    (34, 2, 0, 1, AIR),
    (38, 2, 0, 1, RB),
    (42, 2, 0, 1, AIR),
    (46, 1, 1, 2, RB),
    (50, 1, 4, 1, RB),
    (54, 1, 4, 1, AIR),
    (60, 1, 1, 2, AIR),
    (82, 2, 0, 1, RB),
    (86, 2, 0, 1, AIR),
]
for t in range(111):
    if t == 12:
        expect(
            sim.get_block(1, 2, 1).startswith("minecraft:white_shulker_box"),
            "shulker placed by the dispenser at tick 12",
        )
    if t == 64:
        expect(sim.get_block(1, 2, 1) == AIR, "shulker broken by tick 64")
    for at, x, y, z, state in actions:
        if at == t:
            sim.place_block(x, y, z, state)
    if t < 110:
        sim.step()

expect(sim.tick_count() == 110, "tick_count reaches 110")

entities = json.loads(sim.item_entities_json())
east = lambda e: 3 <= e["pos"][0] < 6  # noqa: E731
diamonds = sum(
    e["count"] for e in entities["items"] if e["item"] == "minecraft:diamond" and east(e)
)
expect(diamonds == 2, f"two diamonds land east of the dropper (got {diamonds})")
shulkers = [
    e for e in entities["items"] if e["item"] == "minecraft:white_shulker_box" and east(e)
]
expect(len(shulkers) == 1, "one shulker item lands east")
expect(len(shulkers) == 1 and shulkers[0]["contents"] == [], "the shipped shulker is empty")

summary = json.loads(sim.events_summary_json())
expect(len(summary) > 10, "events summary has per-tick rows")
expect(any(r["piston"] > 0 for r in summary), "piston activity shows in the summary")
expect(len(json.loads(sim.changes_json())) > 20, "block changes recorded")
expect(len(json.loads(sim.world_snapshot_json())) > 20, "world snapshot lists blocks")

cp = sim.checkpoint()
sim.place_block(0, 3, 0, RB)
sim.step()
sim.restore(cp)
expect(sim.get_block(0, 3, 0) == AIR, "restore rewinds a write")

door = (ROOT / "crates/mc-tick/tests/corpus/structures/door_6x6_inworld.snbt").read_text()
door_sim = nucleation.TickSimulation.from_snbt(
    door, nucleation.TickSettleMode.InWorld, 15, -64, 0, ""
)
door_sim.use_block(10, 4, 1)
door_sim.run(40)
door_changes = json.loads(door_sim.changes_json())
expect(
    any("moving_piston" in c["to"] for c in door_changes),
    "the 6x6 door's pistons move after the lever click",
)

print(f"\n{passed} passed, {failed} failed")
sys.exit(0 if failed == 0 else 1)
