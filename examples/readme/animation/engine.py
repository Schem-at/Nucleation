"""Verified Python recorder example for docs/features/animation.md."""

import json

from nucleation import AnimationEffect, BuildAnimation


# --8<-- [start:record]
from nucleation import AnimationEffect, BuildAnimation

animation = BuildAnimation.create("engine_walkthrough")
animation.set_step_ms(300)

# Calls inside a group share one target, effect, and start time.
animation.begin_group()
for x in range(5):
    animation.set_block(x, 0, 0, "minecraft:stone_bricks")
animation.end_group()

# with_effect changes exactly the next recorded target.
animation.with_effect(AnimationEffect.spin_in(700, 1)).set_block(
    4, 1, 0, "minecraft:diamond_block"
)
animation.set_block(0, 1, 0, "minecraft:furnace[facing=south]")

# The camera is another target on the same clock.
animation.animate_camera(AnimationEffect.turntable(3_000), 0)
# --8<-- [end:record]


# --8<-- [start:sample]
frame = json.loads(animation.frame_json(450))
print(animation.group_count())  # 3
print(animation.duration_ms())  # 3000.0, set by the camera track
print(len(frame["poses"]))      # 3 group poses at t=450 ms
# --8<-- [end:sample]

# --8<-- [start:anchors]
# A named point on the diamond block's group (group 1). Frames report it in
# world space after the group's pose, so a renderer or a docs tool can draw a
# hotspot, a leader line, or a label that lands with the block.
animation.add_anchor_to_group(1, "diamond", 4.5, 2.0, 0.5)
settled = json.loads(animation.frame_json(animation.duration_ms()))
print(settled["anchors"][0]["world"])  # [4.5, 2.0, 0.5]
# --8<-- [end:anchors]

assert settled["anchors"][0]["world"] == [4.5, 2.0, 0.5], "anchor did not settle"

assert animation.group_count() == 3
assert animation.duration_ms() == 3_000
assert len(frame["poses"]) == 3
print("Animation engine Python example: OK")
