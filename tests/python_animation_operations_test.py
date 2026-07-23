import json

import nucleation
import pytest


def test_generated_animation_operations_and_receipts():
    animation = nucleation.BuildAnimation.create("operations")
    animation.create_region("wing", 10, 0, 10, 11, 0, 10)
    animation.set_block_in_region(
        "wing", 10, 0, 10, "minecraft:oak_stairs[facing=east]"
    )

    animation.rotate_region_y("wing", 90, 250.0)
    receipts = json.loads(animation.operations_json())
    assert len(receipts) == 1
    assert receipts[0]["duration_ms"] == 250.0
    assert receipts[0]["cells"][0]["final_block"] == "minecraft:oak_stairs[facing=south]"

    midpoint = receipts[0]["start_ms"] + receipts[0]["duration_ms"] * 0.5
    frame = json.loads(animation.frame_json(midpoint))
    kinds = {line["kind"] for line in frame["gizmos"]}
    assert {"RegionBounds", "Pivot", "RotationArc"} <= kinds
    assert animation.duration_ms() >= receipts[0]["start_ms"] + 250.0

    with pytest.raises(Exception):
        animation.rotate_region_y("wing", 45, 250.0)
    assert len(json.loads(animation.operations_json())) == 1
