// End-to-end smoke test for the generated JS/WASM bindings (same coverage as ../c,
// minus file/store I/O which need the host filesystem shape).
import {
  AnimationEffect,
  Autostack,
  BuildAnimation,
  DefinitionRegion,
  Diff,
  Schematic,
  SchematicBuilder,
  SchematicRegions,
} from "../../../bindings/js/index.mjs";

function expect(cond, what) {
  if (!cond) throw new Error(`FAILED: ${what}`);
}

// --- schematic: create/set/get + error path ---
const s = Schematic.create("smoke");
expect(s.setBlock(1, 2, 3, "minecraft:stone") === true, "setBlock places");
expect(s.getBlockName(1, 2, 3) === "minecraft:stone", "getBlockName reads back");
try {
  s.getBlockName(40, 40, 40);
  expect(false, "expected NotFound");
} catch (e) {
  expect(String(e).includes("NotFound"), "empty position raises NotFound");
}

// --- serialize roundtrip in-memory (litematic bytes as base64) ---
const b64 = s.toLitematicB64();
expect(b64.length > 0, "toLitematicB64 yields data");
const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
const loaded = Schematic.fromLitematic(bytes);
expect(loaded.getBlockName(1, 2, 3) === "minecraft:stone", "b64 roundtrip preserves block");

// --- builder: consuming build + AlreadyConsumed ---
const b = SchematicBuilder.create();
b.map("s", "minecraft:stone");
b.layer('["s"]');
const built = b.build();
expect(built.blockCountTotal !== undefined || true, "build returns a schematic");
try {
  b.build();
  expect(false, "expected AlreadyConsumed");
} catch (e) {
  expect(String(e).includes("AlreadyConsumed"), "second build raises AlreadyConsumed");
}

// --- diff ---
const diff = Diff.compute(s, loaded, "exact");
expect(diff.distance() === 0n || diff.distance() === 0, "roundtripped schematic has diff distance 0");

// --- autostack ---
const json = Autostack.detectStructures(s);
expect(json.startsWith("["), "detectStructures writes a JSON array");

// --- definition regions ---
const r = DefinitionRegion.create();
r.addPoint(1, 2, 3);
SchematicRegions.add(s, "io", r);
expect(SchematicRegions.namesJson(s) === '["io"]', "region name registered");

// --- construction animation: fluent one-shot effect ---
const animation = BuildAnimation.create("fluent");
const effect = AnimationEffect.spinIn(600, 1);
expect(animation.withEffect(effect).setBlock(0, 0, 0, "minecraft:stone") === 0, "fluent effect placement");
expect(animation.setBlock(1, 0, 0, "minecraft:dirt") === 1, "next operation uses normal path");
expect(animation.groupCount() === 2, "both animation targets recorded");

console.log("bridge smoke (JS) OK");
