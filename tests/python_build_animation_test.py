"""Parity of the Python build-animation engine with the native fixtures.

Run via tools/verify-build-animation.sh, or directly with the repository's
virtualenv after `pip install ./bindings/python`.
"""

import base64
import json
import math
import os
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation

EPSILON = 1e-4
FIXTURES = Path(__file__).resolve().parent / "fixtures" / "build-animation"


def build_beacon():
    animation = BuildAnimation.create("beacon")
    animation.set_step_ms(140)
    for x in range(-1, 2):
        for z in range(-1, 2):
            animation.set_block(x, 0, z, "minecraft:gold_block")
    animation.with_effect(AnimationEffect.spin_in(680, 1)).set_block(0, 1, 0, "minecraft:beacon")
    animation.add_anchor("beacon", 0.0, 1.5, 0.0)
    camera = AnimationEffect.create(2_400)
    camera.add_tween("rotateY", -4, 4, "inOutSine")
    animation.animate_camera(camera, 0)
    return animation


def build_crafting_nook():
    animation = BuildAnimation.create("crafting_nook")
    animation.set_step_ms(520)
    animation.begin_group()
    for x in range(5):
        for z in range(5):
            animation.set_block(x, 0, z, "minecraft:spruce_planks")
    animation.end_group()
    animation.begin_group()
    for y in (1, 2, 3):
        for x in range(5):
            if x == 2 and y == 2:
                block = "minecraft:light_blue_stained_glass"
            elif x in (0, 4):
                block = "minecraft:stripped_spruce_log[axis=y]"
            else:
                block = "minecraft:oak_planks"
            animation.set_block(x, y, 0, block)
        for z in range(1, 5):
            if z == 2 and y == 2:
                block = "minecraft:light_blue_stained_glass"
            elif z == 4:
                block = "minecraft:stripped_spruce_log[axis=y]"
            else:
                block = "minecraft:oak_planks"
            animation.set_block(0, y, z, block)
    animation.end_group()
    animation.with_effect(AnimationEffect.spin_in(620, 1)).set_block(
        1, 1, 1, "minecraft:crafting_table"
    )
    animation.add_anchor("crafting-table", 1.0, 1.5, 1.0)
    animation.set_block(3, 1, 1, "minecraft:chest[facing=south]")
    animation.begin_group()
    animation.set_block(4, 2, 1, "minecraft:wall_torch[facing=south]")
    animation.set_block(1, 2, 4, "minecraft:wall_torch[facing=east]")
    animation.add_anchor("torches", 4.0, 2.0, 1.0)
    animation.end_group()
    camera = AnimationEffect.create(3_000)
    camera.add_tween("rotateY", -5, 6, "inOutSine")
    animation.animate_camera(camera, 0)
    return animation


def assert_close(actual, expected, path):
    if isinstance(expected, bool) or expected is None or isinstance(expected, str):
        assert actual == expected, f"{path}: {actual!r} != {expected!r}"
    elif isinstance(expected, (int, float)):
        assert isinstance(actual, (int, float)), f"{path}: not a number: {actual!r}"
        assert math.isclose(actual, expected, abs_tol=EPSILON), f"{path}: {actual} != {expected}"
    elif isinstance(expected, list):
        assert isinstance(actual, list) and len(actual) == len(expected), f"{path}: length"
        for i, item in enumerate(expected):
            assert_close(actual[i], item, f"{path}[{i}]")
    else:
        assert isinstance(actual, dict), f"{path}: not an object"
        assert sorted(actual) == sorted(expected), f"{path}: keys {sorted(actual)} != {sorted(expected)}"
        for key, item in expected.items():
            assert_close(actual[key], item, f"{path}.{key}")


def check(name, build):
    expected = json.loads((FIXTURES / f"{name}.json").read_text())
    animation = build()
    assert animation.group_count() == expected["groupCount"], name
    assert_close(animation.duration_ms(), expected["durationMs"], f"{name}.durationMs")
    assert_close(json.loads(animation.anchors_json()), expected["anchors"], f"{name}.anchors")
    for i, t in enumerate(expected["sampleTimesMs"]):
        assert_close(
            json.loads(animation.frame_json(t)), expected["frames"][i], f"{name}.frames[{i}]@{t}"
        )
    first = animation.frame_json(450)
    animation.frame_json(2_000)
    assert animation.frame_json(450) == first, f"{name}: sampling is not pure"


check("beacon", build_beacon)
check("crafting-nook", build_crafting_nook)

# The animated GLB needs the vanilla pack the docs generators use.
pack_path = os.environ.get("NUCLEATION_PACK")
if pack_path and Path(pack_path).exists():
    from nucleation import ResourcePack

    pack = ResourcePack.from_bytes(list(Path(pack_path).read_bytes()))
    glb = base64.b64decode(build_beacon().to_animated_glb_b64(pack, 30))
    assert glb[:4] == b"glTF", "animated GLB magic"
    json_len = int.from_bytes(glb[12:16], "little")
    doc = json.loads(glb[20 : 20 + json_len])
    assert doc["nodes"][0]["name"] == "build:beacon"
    assert sum(1 for node in doc["nodes"] if node.get("name", "").startswith("group:")) == 10
    assert any(node.get("name") == "anchor:beacon" for node in doc["nodes"])
    print("Build-animation Python GLB: OK")

print("Build-animation Python parity: OK")
