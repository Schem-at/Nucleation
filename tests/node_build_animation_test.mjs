// Parity of the WASM build-animation engine with the native fixtures under
// tests/fixtures/build-animation. Run via tools/verify-build-animation.sh,
// which packages dist/npm and links it as node_modules/nucleation.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { AnimationEffect, BuildAnimation } from "nucleation";

const EPSILON = 1e-4;
const fixture = (name) =>
  JSON.parse(readFileSync(new URL(`./fixtures/build-animation/${name}.json`, import.meta.url)));

export function buildBeacon() {
  const animation = BuildAnimation.create("beacon");
  animation.setStepMs(140);
  for (let x = -1; x <= 1; x += 1) {
    for (let z = -1; z <= 1; z += 1) animation.setBlock(x, 0, z, "minecraft:gold_block");
  }
  animation.withEffect(AnimationEffect.spinIn(680, 1)).setBlock(0, 1, 0, "minecraft:beacon");
  animation.addAnchor("beacon", 0.0, 1.5, 0.0);
  const camera = AnimationEffect.create(2_400);
  camera.addTween("rotateY", -4, 4, "inOutSine");
  animation.animateCamera(camera, 0);
  return animation;
}

export function buildCraftingNook() {
  const animation = BuildAnimation.create("crafting_nook");
  animation.setStepMs(520);
  animation.beginGroup();
  for (let x = 0; x < 5; x += 1) {
    for (let z = 0; z < 5; z += 1) animation.setBlock(x, 0, z, "minecraft:spruce_planks");
  }
  animation.endGroup();
  animation.beginGroup();
  for (const y of [1, 2, 3]) {
    for (let x = 0; x < 5; x += 1) {
      const block =
        x === 2 && y === 2
          ? "minecraft:light_blue_stained_glass"
          : x === 0 || x === 4
            ? "minecraft:stripped_spruce_log[axis=y]"
            : "minecraft:oak_planks";
      animation.setBlock(x, y, 0, block);
    }
    for (let z = 1; z < 5; z += 1) {
      const block =
        z === 2 && y === 2
          ? "minecraft:light_blue_stained_glass"
          : z === 4
            ? "minecraft:stripped_spruce_log[axis=y]"
            : "minecraft:oak_planks";
      animation.setBlock(0, y, z, block);
    }
  }
  animation.endGroup();
  animation.withEffect(AnimationEffect.spinIn(620, 1)).setBlock(1, 1, 1, "minecraft:crafting_table");
  animation.addAnchor("crafting-table", 1.0, 1.5, 1.0);
  animation.setBlock(3, 1, 1, "minecraft:chest[facing=south]");
  animation.beginGroup();
  animation.setBlock(4, 2, 1, "minecraft:wall_torch[facing=south]");
  animation.setBlock(1, 2, 4, "minecraft:wall_torch[facing=east]");
  animation.addAnchor("torches", 4.0, 2.0, 1.0);
  animation.endGroup();
  const camera = AnimationEffect.create(3_000);
  camera.addTween("rotateY", -5, 6, "inOutSine");
  animation.animateCamera(camera, 0);
  return animation;
}

function assertClose(actual, expected, path) {
  if (typeof expected === "number") {
    assert.ok(typeof actual === "number", `${path}: expected a number, got ${typeof actual}`);
    assert.ok(Math.abs(actual - expected) <= EPSILON, `${path}: ${actual} ≠ ${expected}`);
  } else if (Array.isArray(expected)) {
    assert.ok(Array.isArray(actual), `${path}: expected an array`);
    assert.equal(actual.length, expected.length, `${path}: length`);
    expected.forEach((item, i) => assertClose(actual[i], item, `${path}[${i}]`));
  } else if (expected !== null && typeof expected === "object") {
    assert.ok(actual !== null && typeof actual === "object", `${path}: expected an object`);
    assert.deepEqual(Object.keys(actual).sort(), Object.keys(expected).sort(), `${path}: keys`);
    for (const key of Object.keys(expected)) assertClose(actual[key], expected[key], `${path}.${key}`);
  } else {
    assert.equal(actual, expected, path);
  }
}

const BUILDS = [
  ["beacon", buildBeacon],
  ["crafting-nook", buildCraftingNook],
];

for (const [name, build] of BUILDS) {
  test(`${name}: WASM engine matches the native fixture`, () => {
    const expected = fixture(name);
    const animation = build();
    assert.equal(animation.groupCount(), expected.groupCount);
    assertClose(animation.durationMs(), expected.durationMs, "durationMs");
    expected.sampleTimesMs.forEach((t, i) => {
      const frame = JSON.parse(animation.frameJson(t));
      assertClose(frame, expected.frames[i], `frames[${i}] @${t}ms`);
    });
  });

  test(`${name}: anchors match the native declarations and samples`, () => {
    const expected = fixture(name);
    const animation = build();
    assertClose(JSON.parse(animation.anchorsJson()), expected.anchors, "anchors");
    const last = JSON.parse(animation.frameJson(expected.sampleTimesMs.at(-1)));
    assertClose(last.anchors, expected.frames.at(-1).anchors, "final anchors");
    assert.throws(
      () => animation.addAnchor(expected.anchors[0].name, 0, 0, 0),
      /./,
      "duplicate name rejected",
    );
  });

  test(`${name}: sampling is pure and order-independent`, () => {
    const animation = build();
    const first = animation.frameJson(450);
    animation.frameJson(2_000);
    animation.frameJson(0);
    assert.equal(animation.frameJson(450), first);
  });
}
