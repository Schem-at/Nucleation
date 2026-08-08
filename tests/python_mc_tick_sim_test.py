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

# What the engine currently has in flight — the renderer's answer to "which
# block is sliding, from where, arriving when". Reconstructing it from
# `changes_json` is a reimplementation of piston mechanics in the host, which
# is how an animation ends up on a different clock from the simulation that
# decides the landing.
flight_sim = nucleation.TickSimulation.from_snbt(
    door, nucleation.TickSettleMode.InWorld, 15, -64, 0, ""
)
flight_sim.use_block(10, 4, 1)
air = []
for _ in range(40):
    flight_sim.step()
    air = json.loads(flight_sim.moving_blocks_json())
    if air:
        break
expect(len(air) > 0, f"the door reports blocks in flight (got {len(air)})")
STEP = {
    "east": (1, 0, 0), "west": (-1, 0, 0), "up": (0, 1, 0),
    "down": (0, -1, 0), "south": (0, 0, 1), "north": (0, 0, -1),
}
if air:
    m = air[0]
    expect(
        not m["state"].startswith("minecraft:moving_piston"),
        f"a flight names the block being carried, not the placeholder ({m['state']})",
    )
    d = STEP[m["dir"]]
    expect(
        [f + s for f, s in zip(m["from"], d)] == m["to"],
        f"it travels one cell {m['dir']}: {m['from']} -> {m['to']}",
    )
    expect(
        all(f["started"] == m["started"] and f["lands"] == m["lands"] for f in air),
        "blocks dispatched together share one flight window",
    )
    expect(
        m["lands"] > m["started"] and m["started"] == flight_sim.tick_count() - 1,
        f"the window runs from dispatch to landing ({m['started']}..{m['lands']})",
    )

# A retracting piston is the one move where what lands is not what travels:
# vanilla's PistonHeadRenderer draws a `piston_head` on the interpolated slot
# and the body, EXTENDED=true, parked outside it.
retractions = 0
split = True
for _ in range(120):
    flight_sim.step()
    for f in json.loads(flight_sim.moving_blocks_json()):
        if f["source_piston"] and not f["extending"]:
            retractions += 1
            if (
                not f["carried"].startswith("minecraft:piston_head")
                or "extended=true" not in (f["remains"] or "")
                or "extended=false" not in f["state"]
            ):
                split = False
        elif f["carried"] != f["state"] or f["remains"] is not None:
            split = False
        # A moving arm comes in two lengths — the game shortens it while the
        # head is beside its body, or the shaft passes through the piston.
        if f["carried"].startswith("minecraft:piston_head"):
            short = f["carried_short"] or ""
            if "short=true" not in short or short.replace("short=true", "short=false") != f["carried"]:
                split = False
        elif f["carried_short"] is not None:
            split = False
expect(retractions > 0, f"the door's pistons retract ({retractions} flight-ticks seen)")
expect(
    split,
    "a retraction carries a piston_head and parks an extended body; "
    "every other move carries itself and parks nothing",
)

# --- the event timeline ---
tl = nucleation.TickSimulation.from_snbt(
    door, nucleation.TickSettleMode.InWorld, 15, -64, 0, ""
)
tl.record_timeline()
tl.use_block(10, 4, 1)
tl.run(40)
tl.stop_timeline()
# A stopped span stays readable — that is what makes it exportable.
activity = json.loads(tl.timeline_activity_json())
expect(
    len(activity["ticks"]) > 0,
    f"the door records activity ({len(activity['ticks'])} active ticks)",
)
expect(
    all(t["changes"] > 0 or t["inputs"] > 0 or t["pistons"] > 0 for t in activity["ticks"]),
    "every listed tick did something — idle ticks are absent, so a still build does not advance the strip",
)
expect(
    all(
        activity["ticks"][i]["tick"] > activity["ticks"][i - 1]["tick"]
        for i in range(1, len(activity["ticks"]))
    ),
    "ticks are strictly ascending",
)
expect(
    len(activity["ticks"]) < activity["end"] - activity["start"] + 1,
    "the door has idle ticks that were skipped "
    f"({len(activity['ticks'])} of {activity['end'] - activity['start'] + 1})",
)

# --- cycles, projection, and a selection's schematic ---
cycles = json.loads(tl.timeline_cycles_json())
expect("exact" in cycles and "translated" in cycles, "cycles report has both kinds")

# A flying machine repeats itself displaced; an absent cycle is null, not an error.
fly_snbt = (ROOT / "crates/mc-tick/tests/corpus/structures/flying_machine_east.snbt").read_text()
fly = nucleation.TickSimulation.from_snbt(
    fly_snbt, nucleation.TickSettleMode.InWorld, 0, 0, 0, ""
)
# Kick timing matches the CLI's verified `--place 2:...=redstone_block
# --place 4:...=air` (see GLOBAL CONSTRAINTS): 2 quiet ticks, a 2-tick pulse,
# then run to 60. A kick held longer (e.g. placed at tick 0) does not launch
# this machine at all — placement timing is load-bearing here, not cosmetic.
fly.record_timeline()
fly.run(2)
fly.place_block(2, 1, 1, RB)
fly.run(2)
fly.place_block(2, 1, 1, AIR)
fly.run(56)
fly_cycles = json.loads(fly.timeline_cycles_json())
expect(fly_cycles["translated"] is not None, "the flying machine repeats itself, displaced")
expect(
    fly_cycles["translated"]["drift"][0] != 0,
    f"it travels along x (drift {fly_cycles['translated']['drift']})",
)

projected = json.loads(fly.animation_timeline_json(0, 20, 50.0))
expect(
    isinstance(projected["events"], list) and len(projected["events"]) > 0,
    "the projection has events",
)
expect(
    all(e["kind"] in ("piston", "set_block") for e in projected["events"]),
    "every event is one the mesher understands",
)
expect(
    isinstance(projected["tick_ms"], (int, float)) and len(projected["origin"]) == 3,
    "origin and tick_ms are present",
)

seed = fly.selection_schematic_b64(0, 20)
expect(len(seed) > 0, "the selection's initial schematic comes back as bytes")

# --- pinned bridge-vs-CLI agreement ---
#
# `nucleation-cli animate crates/mc-tick/tests/corpus/structures/flying_machine_east.snbt
# --ticks 60 --place 2:2,1,1=minecraft:redstone_block --place 4:2,1,1=minecraft:air`
# prints these exact numbers (verified by hand against this run — see GLOBAL
# CONSTRAINTS in the task brief):
#   exact:      start=0 end=7  period=7  drift=(0,0,0)
#   translated: start=5 end=15 period=10 drift=(-1,0,0)
# `Placement` settle reproduces the CLI's own numbers exactly; `InWorld` settle
# (the case above) legitimately gives different numbers because it simulates
# more of the world around the kick. Pinning this here catches a sign or
# period regression in either the bridge or the CLI path.
pinned = nucleation.TickSimulation.from_snbt(
    fly_snbt, nucleation.TickSettleMode.Placement, 0, 0, 0, ""
)
pinned.record_timeline()
pinned.run(2)
pinned.place_block(2, 1, 1, RB)
pinned.run(2)
pinned.place_block(2, 1, 1, AIR)
pinned.run(56)
pinned_cycles = json.loads(pinned.timeline_cycles_json())
exact = pinned_cycles["exact"]
translated = pinned_cycles["translated"]
expect(
    exact is not None
    and exact["start"] == 0
    and exact["end"] == 7
    and exact["period"] == 7
    and exact["drift"] == [0, 0, 0],
    f"exact cycle matches the CLI's pinned numbers (got {exact})",
)
expect(
    translated is not None
    and translated["start"] == 5
    and translated["end"] == 15
    and translated["period"] == 10
    and translated["drift"] == [-1, 0, 0],
    f"translated cycle matches the CLI's pinned numbers (got {translated})",
)

# --- record, project, export ---
pack_path = ROOT / "apps/sim-lab-wasm/public/pack/mesher-pack.zip"
if pack_path.exists():
    import base64
    import struct

    rp = nucleation.ResourcePack.from_bytes(pack_path.read_bytes())
    seed_schem = nucleation.Schematic.from_data(base64.b64decode(seed))
    glb_b64 = nucleation.MeshResult.animated_glb_b64(
        seed_schem, rp, fly.animation_timeline_json(0, 20, 50.0)
    )
    glb = base64.b64decode(glb_b64)
    expect(len(glb) > 1000, f"an animated GLB comes back ({len(glb)} bytes)")
    expect(glb[0:4] == b"glTF", "and it is really a GLB")

    # Parse the GLB's JSON chunk and check for animation channels — the
    # property that distinguishes an animated export from a static mesh.
    json_chunk_length = struct.unpack_from("<I", glb, 12)[0]
    gltf = json.loads(glb[20 : 20 + json_chunk_length].decode("utf8"))
    animations = gltf.get("animations") or []
    expect(
        isinstance(gltf.get("animations"), list) and len(animations) > 0,
        f"the GLB has animations ({len(animations)})",
    )
    channel_count = sum(len(a.get("channels") or []) for a in animations)
    expect(channel_count > 0, f"the animation(s) have channels ({channel_count})")
    nodes = gltf.get("nodes") or []
    expect(
        isinstance(gltf.get("nodes"), list) and len(nodes) > 0,
        f"the GLB has nodes ({len(nodes)})",
    )
else:
    print(f"  skip animated GLB export — {pack_path} is absent (build artifact, not source)")

# --- dropping a consumed change log ---
#
# The log grows for as long as the simulation runs and nothing empties it, so
# a long-running host accumulates every block change forever. A host that has
# already consumed the changes needs to say so.
clr = nucleation.TickSimulation.from_snbt(
    door, nucleation.TickSettleMode.InWorld, 15, -64, 0, ""
)
clr.use_block(10, 4, 1)
clr.run(20)
before = clr.changes_count()
expect(before > 0, f"the door recorded changes ({before})")
clr.clear_changes()
expect(clr.changes_count() == 0, "clear_changes empties the log")
expect(len(json.loads(clr.changes_json())) == 0, "and changes_json agrees")
# This door has already fully opened and settled by tick 20 (verified: it
# never produces another change no matter how much longer it runs), so a
# second lever click is needed to give the still-recording sim something new
# to see — otherwise the door would stay quiescent and the assertion below
# would fail regardless of whether clear_changes had secretly turned
# recording off, telling us nothing about which one happened.
clr.use_block(10, 4, 1)
clr.run(20)
expect(
    clr.changes_count() > 0,
    "recording continues after a clear — it drops what happened, it does not stop recording",
)

# --- reading only what is new ---
#
# While a run timeline is recording the engine refuses to drop the change log,
# so a host must be able to read the tail without paying for the whole
# backlog on every frame.
inc = nucleation.TickSimulation.from_snbt(
    door, nucleation.TickSettleMode.InWorld, 15, -64, 0, ""
)
inc.use_block(10, 4, 1)
inc.run(20)
whole = json.loads(inc.changes_json())
expect(len(whole) > 10, f"the door recorded a backlog ({len(whole)})")
tail = json.loads(inc.changes_json_from(len(whole) - 5))
expect(len(tail) == 5, f"asking from n returns exactly the tail ({len(tail)})")
expect(
    tail == whole[-5:],
    "and it is the same entries changes_json would have given",
)
expect(len(json.loads(inc.changes_json_from(0))) == len(whole), "from 0 is the whole log")
expect(len(json.loads(inc.changes_json_from(len(whole)))) == 0, "from the end is empty")
expect(
    len(json.loads(inc.changes_json_from(len(whole) + 1000))) == 0,
    "and past the end is empty, not an error",
)

print(f"\n{passed} passed, {failed} failed")
sys.exit(0 if failed == 0 else 1)
