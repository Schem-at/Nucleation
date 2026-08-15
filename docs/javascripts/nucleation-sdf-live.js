import { Brush } from "./vendor/nucleation/Brush.mjs";
import { BuildingTool } from "./vendor/nucleation/BuildingTool.mjs";
import { Field3 } from "./vendor/nucleation/Field3.mjs";
import { InterpolationSpace } from "./vendor/nucleation/InterpolationSpace.mjs";
import { MeshConfig } from "./vendor/nucleation/MeshConfig.mjs";
import { MeshResult } from "./vendor/nucleation/MeshResult.mjs";
import { Palette } from "./vendor/nucleation/Palette.mjs";
import { ResourcePack } from "./vendor/nucleation/ResourcePack.mjs";
import { Schematic } from "./vendor/nucleation/Schematic.mjs";
import { Sdf } from "./vendor/nucleation/Sdf.mjs";

const EMPTY_ZIP = [
  80, 75, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const BLOCKS = {
  white_wool: [226, 229, 229],
  orange_wool: [249, 128, 29],
  magenta_wool: [199, 78, 189],
  light_blue_wool: [58, 179, 218],
  yellow_wool: [254, 216, 61],
  lime_wool: [128, 199, 31],
  pink_wool: [243, 139, 170],
  gray_wool: [71, 79, 82],
  light_gray_wool: [157, 157, 151],
  cyan_wool: [22, 156, 156],
  purple_wool: [137, 50, 184],
  blue_wool: [60, 68, 170],
  brown_wool: [131, 84, 50],
  green_wool: [94, 124, 22],
  red_wool: [176, 46, 38],
  black_wool: [29, 29, 33],
  calcite: [221, 220, 211],
  oxidized_copper: [82, 166, 152],
};

const WOOL_NAMES = new Set(Object.keys(BLOCKS).filter((name) => name.endsWith("_wool")));
const meshCache = new Map();
let resourcePack;

function pack() {
  if (resourcePack !== undefined) return resourcePack;
  resourcePack = ResourcePack.fromBytes(EMPTY_ZIP);
  const faces = Object.fromEntries(
    ["down", "up", "north", "south", "west", "east"].map((side) => [
      side,
      { texture: "#all", cullface: side },
    ]),
  );
  for (const [name, color] of Object.entries(BLOCKS)) {
    const block = `minecraft:${name}`;
    const texture = `minecraft:block/${name}`;
    resourcePack.addBlockstateJson(
      block,
      JSON.stringify({ variants: { "": { model: texture } } }),
    );
    resourcePack.addModelJson(
      texture,
      JSON.stringify({
        textures: { all: texture },
        elements: [{ from: [0, 0, 0], to: [16, 16, 16], faces }],
      }),
    );
    resourcePack.addTexture(texture, 16, 16, blockTexture(name, color));
  }
  return resourcePack;
}

function blockTexture(name, base) {
  const pixels = [];
  for (let y = 0; y < 16; y += 1) {
    for (let x = 0; x < 16; x += 1) {
      const hash = (x * 17 + y * 29 + (x ^ y) * 7) % 11;
      const weave = WOOL_NAMES.has(name) ? (x + y) % 4 === 0 : (x * 3 + y * 5) % 9 === 0;
      const mineral = name === "calcite" && (x - y + 32) % 7 === 0;
      const patina = name === "oxidized_copper" && (x + y * 2) % 8 < 2;
      const delta = mineral ? -18 : patina ? 12 : weave ? -7 : hash - 5;
      pixels.push(...base.map((channel) => Math.max(0, Math.min(255, channel + delta))), 255);
    }
  }
  return pixels;
}

function selected(context) {
  return {
    shape: String(context.machineState?.variables.shape ?? "bloom"),
    material: String(context.machineState?.variables.material ?? "field"),
  };
}

function volume(kind, field) {
  if (kind === "rings") {
    const ring = Sdf.torus(4.8, 1.25);
    return ring
      .smoothUnion(Sdf.torus(4.8, 1.25).rotate(90, 0, 0), 0.55)
      .smoothUnion(Sdf.torus(4.8, 1.25).rotate(0, 0, 90), 0.55);
  }
  if (kind === "frame") {
    return Sdf.boxFrame(5.4, 5.4, 5.4, 0.85)
      .rounded(0.35)
      .smoothUnion(Sdf.sphere(2.6), 0.7);
  }
  return Sdf.sphere(5.9)
    .offsetByField(field, 1.65)
    .smoothUnion(Sdf.torus(5.1, 1.15).rotate(90, 0, 0), 0.8);
}

function materialBrush(kind, field) {
  if (kind === "calcite") return Brush.solid("minecraft:calcite");
  if (kind === "copper") return Brush.solid("minecraft:oxidized_copper");
  const brush = Brush.field3(
    field,
    [0, 0.5, 1],
    [29, 29, 33, 58, 179, 218, 254, 216, 61],
    -1,
    1,
    InterpolationSpace.Oklab,
  );
  brush.setPalette(Palette.wool());
  return brush;
}

function decodeBase64(value) {
  const binary = globalThis.atob(value);
  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}

async function generateGlb(context) {
  const choice = selected(context);
  const cacheKey = `${choice.shape}:${choice.material}`;
  context.element.classList.add("nuc-sdf-surface");
  const cached = meshCache.get(cacheKey);
  if (cached !== undefined) {
    context.element.dataset.state = "ready";
    context.element.dataset.vertices = String(cached.vertices);
    context.element.dataset.triangles = String(cached.triangles);
    return cached.bytes;
  }

  context.element.dataset.state = "building";
  await new Promise((resolve) => requestAnimationFrame(resolve));
  if (context.signal.aborted) throw new DOMException("Aborted", "AbortError");

  const field = Field3.valueNoiseFbm(0.12, 2026, 4);
  const schematic = Schematic.create(`sdf-${choice.shape}-${choice.material}`);
  BuildingTool.fill(schematic, volume(choice.shape, field).toShape(), materialBrush(choice.material, field));

  const config = MeshConfig.create();
  config.setAmbientOcclusion(true);
  config.setAoIntensity(0.58);
  config.setGreedyMeshing(true);
  const mesh = MeshResult.create(schematic, pack(), config);
  const bytes = decodeBase64(mesh.glbDataB64());
  const vertices = mesh.vertexCount();
  const triangles = mesh.triangleCount();
  meshCache.set(cacheKey, { bytes, vertices, triangles });
  context.element.dataset.state = "ready";
  context.element.dataset.vertices = String(vertices);
  context.element.dataset.triangles = String(triangles);
  return bytes;
}

export function nucleationSdfSurface(kineglyph) {
  return kineglyph.modelViewerSurface({
    source: generateGlb,
    alt: "Minecraft blocks generated in this page by Nucleation's SDF and meshing APIs",
    attributes: {
      "camera-orbit": "38deg 68deg 118%",
      "field-of-view": "28deg",
      "min-camera-orbit": "auto auto 72%",
      "max-camera-orbit": "auto auto 220%",
      "shadow-intensity": "0.75",
      "shadow-softness": "0.9",
      exposure: "1.05",
      "environment-image": "neutral",
    },
  });
}
