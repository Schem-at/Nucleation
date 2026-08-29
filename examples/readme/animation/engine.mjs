// --8<-- [start:record]
import { AnimationEffect, BuildAnimation } from "nucleation";


const animation = BuildAnimation.create("engine_walkthrough");
animation.setStepMs(300);

// Calls inside a group share one target, effect, and start time.
animation.beginGroup();
for (let x = 0; x < 5; x += 1) {
  animation.setBlock(x, 0, 0, "minecraft:stone_bricks");
}
animation.endGroup();

// withEffect changes exactly the next recorded target.
animation.withEffect(AnimationEffect.spinIn(700, 1)).setBlock(
  4, 1, 0, "minecraft:diamond_block",
);
animation.setBlock(0, 1, 0, "minecraft:furnace[facing=south]");

// The camera is another target on the same clock.
animation.animateCamera(AnimationEffect.turntable(3_000), 0);
// --8<-- [end:record]


// --8<-- [start:sample]
const frame = JSON.parse(animation.frameJson(450));
console.log(animation.groupCount()); // 3
console.log(animation.durationMs()); // 3000, set by the camera track
console.log(frame.poses.length);      // 3 group poses at t=450 ms
// --8<-- [end:sample]

// --8<-- [start:anchors]
// A named point on the diamond block's group (group 1). Frames report it in
// world space after the group's pose, so a renderer or a docs tool can draw a
// hotspot, a leader line, or a label that lands with the block.
animation.addAnchorToGroup(1, "diamond", 4, 1.5, 0);
const settled = JSON.parse(animation.frameJson(animation.durationMs()));
console.log(settled.anchors[0].world); // [4, 1.5, 0] — the top centre of the block
// --8<-- [end:anchors]

if (animation.groupCount() !== 3) throw new Error("group count changed");
if (settled.anchors.length !== 1 || settled.anchors[0].world.join() !== "4,1.5,0")
  throw new Error("anchor did not settle on the diamond block");
if (animation.durationMs() !== 3_000) throw new Error("duration changed");
if (frame.poses.length !== 3) throw new Error("frame shape changed");
console.log("Animation engine JavaScript example: OK");
