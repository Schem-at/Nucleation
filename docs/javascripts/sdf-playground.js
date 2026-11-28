/**
 * Interactive SDF & VoxelPipeline Studio + 2D CSG / Interval Brick-Pruning Explorer
 * Powered by Nucleation v0.13.0 compiled to WebAssembly.
 */

let wasmPromise;

async function loadNucleation() {
  wasmPromise ??= Promise.all([
    import("./vendor/nucleation/Field.mjs"),
    import("./vendor/nucleation/FieldExprBuilder.mjs"),
    import("./vendor/nucleation/FieldChannel.mjs"),
    import("./vendor/nucleation/FieldDistanceKind.mjs"),
    import("./vendor/nucleation/FieldCellMode.mjs"),
    import("./vendor/nucleation/InterpolationSpace.mjs"),
    import("./vendor/nucleation/Material.mjs"),
    import("./vendor/nucleation/MaterialBuilder.mjs"),
    import("./vendor/nucleation/MaterialCondition.mjs"),
    import("./vendor/nucleation/ScatterFace.mjs"),
    import("./vendor/nucleation/ScatterRule.mjs"),
    import("./vendor/nucleation/VoxelPipeline.mjs"),
    import("./vendor/nucleation/Palette.mjs"),
    import("./vendor/nucleation/MeshConfig.mjs"),
    import("./vendor/nucleation/MeshResult.mjs"),
    import("./vendor/nucleation/ResourcePack.mjs"),
    import("./vendor/nucleation/Schematic.mjs"),
  ]).then((mods) => Object.assign({}, ...mods));
  return wasmPromise;
}

const EMPTY_ZIP = [
  80, 75, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// Authentic 16x16 procedural block textures for the browser mesher.
const BLOCKS = {
  // Natural terrain & stone
  deepslate: { rgb: [66, 70, 78], style: "strata" },
  tuff: { rgb: [108, 109, 102], style: "speckle" },
  stone: { rgb: [130, 132, 134], style: "stone" },
  andesite: { rgb: [152, 154, 152], style: "speckle" },
  calcite: { rgb: [223, 222, 213], style: "vein" },
  dirt: { rgb: [134, 96, 67], style: "speckle" },
  moss_block: { rgb: [98, 136, 56], style: "moss" },
  short_grass: { rgb: [112, 166, 68], style: "cross" },
  water: { rgb: [56, 126, 212], style: "water" },
  // Architectural & jewel blocks
  obsidian: { rgb: [28, 22, 42], style: "speckle" },
  amethyst_block: { rgb: [148, 102, 206], style: "crystal" },
  lapis_block: { rgb: [42, 82, 168], style: "crystal" },
  prismarine_bricks: { rgb: [98, 174, 162], style: "bricks" },
  dark_prismarine: { rgb: [52, 96, 86], style: "bricks" },
  oxidized_copper: { rgb: [82, 164, 146], style: "patina" },
  weathered_copper: { rgb: [108, 154, 118], style: "patina" },
  cut_copper: { rgb: [192, 108, 78], style: "bricks" },
  smooth_sandstone: { rgb: [224, 210, 168], style: "strata" },
  sea_lantern: { rgb: [188, 232, 226], style: "lantern" },
  gold_block: { rgb: [246, 208, 68], style: "crystal" },
  // Concrete spectrum for Oklab gradients
  white_concrete: { rgb: [212, 216, 218], style: "matte" },
  light_gray_concrete: { rgb: [136, 138, 134], style: "matte" },
  gray_concrete: { rgb: [62, 66, 70], style: "matte" },
  black_concrete: { rgb: [22, 24, 30], style: "matte" },
  blue_concrete: { rgb: [44, 52, 148], style: "matte" },
  cyan_concrete: { rgb: [28, 136, 148], style: "matte" },
  light_blue_concrete: { rgb: [52, 164, 208], style: "matte" },
  lime_concrete: { rgb: [108, 178, 42], style: "matte" },
  green_concrete: { rgb: [78, 102, 42], style: "matte" },
  yellow_concrete: { rgb: [244, 194, 48], style: "matte" },
  orange_concrete: { rgb: [226, 114, 32], style: "matte" },
  red_concrete: { rgb: [164, 42, 40], style: "matte" },
  magenta_concrete: { rgb: [178, 62, 168], style: "matte" },
  purple_concrete: { rgb: [112, 44, 166], style: "matte" },
  pink_concrete: { rgb: [218, 114, 152], style: "matte" },
  brown_concrete: { rgb: [102, 66, 44], style: "matte" },
};

const GRADIENT_PALETTE_IDS = JSON.stringify([
  "minecraft:obsidian",
  "minecraft:black_concrete",
  "minecraft:blue_concrete",
  "minecraft:lapis_block",
  "minecraft:purple_concrete",
  "minecraft:amethyst_block",
  "minecraft:dark_prismarine",
  "minecraft:cyan_concrete",
  "minecraft:prismarine_bricks",
  "minecraft:oxidized_copper",
  "minecraft:light_blue_concrete",
  "minecraft:sea_lantern",
  "minecraft:lime_concrete",
  "minecraft:yellow_concrete",
  "minecraft:gold_block",
  "minecraft:orange_concrete",
  "minecraft:red_concrete",
  "minecraft:magenta_concrete",
  "minecraft:white_concrete",
  "minecraft:calcite",
  "minecraft:smooth_sandstone",
]);

const SHADED_PALETTE_IDS = JSON.stringify([
  "minecraft:obsidian",
  "minecraft:deepslate",
  "minecraft:gray_concrete",
  "minecraft:tuff",
  "minecraft:stone",
  "minecraft:light_gray_concrete",
  "minecraft:andesite",
  "minecraft:prismarine_bricks",
  "minecraft:oxidized_copper",
  "minecraft:calcite",
  "minecraft:white_concrete",
  "minecraft:sea_lantern",
]);

function generateTexture(spec) {
  const [r0, g0, b0] = spec.rgb;
  const style = spec.style;
  const pixels = new Uint8Array(16 * 16 * 4);
  for (let y = 0; y < 16; y += 1) {
    for (let x = 0; x < 16; x += 1) {
      const idx = (y * 16 + x) * 4;
      const hash = ((x * 37 + y * 59 + (x ^ y) * 17) & 31) - 16;
      let delta = Math.round(hash * 0.45);
      let alpha = 255;

      if (style === "bricks") {
        const mortar = y % 8 === 0 || (x + (y >= 8 ? 4 : 0)) % 8 === 0;
        delta = mortar ? -26 : Math.round(hash * 0.35);
      } else if (style === "strata") {
        const band = ((y * 3 + (x >> 2)) % 6) === 0 ? -16 : ((y % 4) === 2 ? 10 : 0);
        delta = band + Math.round(hash * 0.3);
      } else if (style === "moss") {
        delta = ((x * 13 + y * 29) % 7 < 2 ? 18 : -10) + Math.round(hash * 0.5);
      } else if (style === "patina") {
        const edge = x === 0 || y === 0 || x === 15 || y === 15;
        delta = edge ? -18 : ((x + y * 2) % 9 < 2 ? 16 : Math.round(hash * 0.35));
      } else if (style === "crystal") {
        const facet = ((x + y) % 7 === 0 || (x - y + 16) % 9 === 0) ? 24 : -8;
        delta = facet + Math.round(hash * 0.35);
      } else if (style === "lantern") {
        const rim = x <= 1 || y <= 1 || x >= 14 || y >= 14;
        delta = rim ? -34 : 18 + Math.round(hash * 0.2);
      } else if (style === "vein") {
        delta = (x - y + 32) % 7 === 0 ? -20 : Math.round(hash * 0.25);
      } else if (style === "water") {
        delta = ((x + y * 2) % 6 < 2 ? 16 : -8);
        alpha = 185;
      } else if (style === "matte") {
        delta = Math.round(hash * 0.18);
      } else if (style === "cross") {
        // Stylized tuft of blades on transparent background
        const blade =
          (Math.abs(x - 4) <= 1 && y >= 6) ||
          (Math.abs(x - 8) <= 1 && y >= 3) ||
          (Math.abs(x - 12) <= 1 && y >= 7) ||
          (y >= 12 && x >= 3 && x <= 13);
        if (!blade) alpha = 0;
        delta = y < 8 ? 22 : -8;
      }

      pixels[idx] = Math.max(0, Math.min(255, r0 + delta));
      pixels[idx + 1] = Math.max(0, Math.min(255, g0 + delta));
      pixels[idx + 2] = Math.max(0, Math.min(255, b0 + delta));
      pixels[idx + 3] = alpha;
    }
  }
  return pixels;
}

let cachedPack;
function getResourcePack(nuc) {
  if (cachedPack) return cachedPack;
  const pack = nuc.ResourcePack.fromBytes(EMPTY_ZIP);
  const cubeFaces = Object.fromEntries(
    ["down", "up", "north", "south", "west", "east"].map((side) => [
      side,
      { texture: "#all", cullface: side },
    ]),
  );
  for (const [name, spec] of Object.entries(BLOCKS)) {
    const block = `minecraft:${name}`;
    const texture = `minecraft:block/${name}`;
    pack.addBlockstateJson(
      block,
      JSON.stringify({ variants: { "": { model: texture } } }),
    );
    if (spec.style === "cross") {
      // Compact tuft cross model
      pack.addModelJson(
        texture,
        JSON.stringify({
          textures: { cross: texture },
          elements: [
            {
              from: [2, 0, 8],
              to: [14, 13, 8],
              rotation: { origin: [8, 8, 8], axis: "y", angle: 45, rescale: true },
              shade: false,
              faces: { north: { texture: "#cross" }, south: { texture: "#cross" } },
            },
            {
              from: [8, 0, 2],
              to: [8, 13, 14],
              rotation: { origin: [8, 8, 8], axis: "y", angle: 45, rescale: true },
              shade: false,
              faces: { east: { texture: "#cross" }, west: { texture: "#cross" } },
            },
          ],
        }),
      );
    } else if (spec.style === "water") {
      pack.addModelJson(
        texture,
        JSON.stringify({
          textures: { all: texture },
          elements: [{ from: [0, 0, 0], to: [16, 14, 16], faces: cubeFaces }],
        }),
      );
    } else {
      pack.addModelJson(
        texture,
        JSON.stringify({
          textures: { all: texture },
          elements: [{ from: [0, 0, 0], to: [16, 16, 16], faces: cubeFaces }],
        }),
      );
    }
    pack.addTexture(texture, 16, 16, generateTexture(spec));
  }
  // Register vanilla fluid texture names used by schematic-mesher for minecraft:water
  const waterTex = generateTexture(BLOCKS.water);
  pack.addTexture("minecraft:block/water_still", 16, 16, waterTex);
  pack.addTexture("minecraft:block/water_flow", 16, 16, waterTex);
  cachedPack = pack;
  return pack;
}

function decodeBase64(value) {
  const binary = globalThis.atob(value);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) out[i] = binary.charCodeAt(i);
  return out;
}

const CUSTOM_SNIPPETS = [
  {
    label: "Gyroid sphere",
    expr: `let q = p * 0.42;\nlet g = abs(sin(q.x)*cos(q.y) + sin(q.y)*cos(q.z) + sin(q.z)*cos(q.x)) - 0.32;\nmax(g, sphere(p, 11.5))`,
  },
  {
    label: "Twisted torus",
    expr: `let a = y * 0.16;\nlet q = vec3(x*cos(a) - z*sin(a), y, x*sin(a) + z*cos(a));\ntorus(q, 8.0, 2.8) + sin(x*0.7)*cos(z*0.7)*0.55`,
  },
  {
    label: "Schwarz-P lattice",
    expr: `let q = p * 0.48;\nlet s = abs(cos(q.x) + cos(q.y) + cos(q.z)) - 0.42;\nmax(s, box(p, vec3(10.5, 10.5, 10.5)))`,
  },
  {
    label: "Orbital rings",
    expr: `let r1 = torus(p, 9.0, 1.8);\nlet r2 = torus(vec3(x, z, y), 9.0, 1.8);\nlet r3 = torus(vec3(y, x, z), 9.0, 1.8);\nsmin(smin(r1, r2, 1.6), r3, 1.6)`,
  },
];

const PRESETS = {
  island: {
    label: "Mossy island",
    p1: { label: "Noise amplitude", min: 0.5, max: 4.0, step: 0.25, value: 2.2 },
    p2: { label: "Lagoon water level", min: -1, max: 4, step: 1, value: 2 },
    bounds: [-14, -8, -14, 14, 12, 14],
    build(nuc, p1, p2, matMode) {
      const noise = nuc.Field.perlinFbm3(0.09, 42, 4);
      const crater = nuc.Field.sphere(6.2).translate(2.8, 5.5, 2.2);
      const basin = nuc.Field.ellipsoid(13, 6.8, 13)
        .displace(noise, p1)
        .subtract(crater)
        .tag(1);
      const crystal = nuc.Field.octahedron(4.5).translate(-5.2, 5.8, -4.0).tag(2);
      const geom = basin.smoothUnion(crystal, 1.5);

      const mat = buildMaterial(nuc, matMode, noise, () => {
        const layers = nuc.MaterialBuilder.createLayered();
        layers.addLayer(
          nuc.MaterialCondition.exposedFace(4, true).andWith(
            nuc.MaterialCondition.slopeRange(true, 0.0, true, 0.68),
          ),
          nuc.Material.solid("minecraft:moss_block"),
        );
        layers.addLayer(
          nuc.MaterialCondition.depthBelowSurface(true, 1.0, true, 2.0),
          nuc.Material.solid("minecraft:dirt"),
        );
        layers.setDefault(
          nuc.Material.channelRamp(
            nuc.FieldChannel.WorldY,
            -7.0,
            6.0,
            '["minecraft:deepslate","minecraft:tuff","minecraft:stone","minecraft:andesite"]',
            true,
          ),
        );
        const matcher = nuc.MaterialBuilder.createTagMatcher(true, 73);
        matcher.addTagCase(1, layers.build());
        matcher.addTagCase(2, nuc.Material.solid("minecraft:amethyst_block"));
        matcher.setDefault(nuc.Material.solid("minecraft:stone"));
        return matcher.build();
      });

      const pipe = nuc.VoxelPipeline.create(geom, mat);
      if (matMode === "pipeline") {
        // Confine water to the carved lagoon basin below water level p2
        const lagoonPool = crater.intersectionWith(
          nuc.Field.plane(0, 1, 0, -p2),
        );
        pipe.addFluid(lagoonPool, "minecraft:water");
        const grass = nuc.ScatterRule.create(
          nuc.ScatterFace.Top,
          0.36,
          '["minecraft:short_grass"]',
          [1.0],
          19,
        );
        grass.setOnBlock("minecraft:moss_block");
        pipe.addScatter(grass);
      }
      return pipe.toSchematicBounded("mossy_island", ...this.bounds);
    },
  },

  gyroid: {
    label: "Gyroid core",
    p1: { label: "Frequency", min: 0.24, max: 0.56, step: 0.02, value: 0.38 },
    p2: { label: "Wall thickness", min: 0.16, max: 0.52, step: 0.02, value: 0.30 },
    bounds: [-12, -12, -12, 12, 12, 12],
    build(nuc, p1, p2, matMode) {
      const b = nuc.FieldExprBuilder.create();
      b.setBounds(-12, -12, -12, 12, 12, 12);
      b.setDistanceKind(nuc.FieldDistanceKind.Implicit);
      const gyroid = b.compile(
        `let q = p * ${p1.toFixed(3)};\n` +
          `let g = abs(sin(q.x)*cos(q.y) + sin(q.y)*cos(q.z) + sin(q.z)*cos(q.x)) - ${p2.toFixed(3)};\n` +
          `max(g, sphere(p, 11.4))`,
      );
      const radial = nuc.Field.expr("length(p) / 11.5");

      const mat = buildMaterial(nuc, matMode, radial, () => {
        const pal = nuc.Palette.fromBlockIds(GRADIENT_PALETTE_IDS).dithered();
        return nuc.Material.fieldGradient(
          radial,
          [0.0, 0.35, 0.72, 1.0],
          [188, 232, 226, 48, 168, 178, 92, 56, 172, 28, 22, 42],
          0.1,
          1.0,
          pal,
          nuc.InterpolationSpace.Oklab,
        );
      });

      const pipe = nuc.VoxelPipeline.create(gyroid, mat);
      return pipe.toSchematicBounded("gyroid_core", ...this.bounds);
    },
  },

  mandelbulb: {
    label: "Mandelbulb",
    p1: { label: "Power", min: 4, max: 10, step: 1, value: 8 },
    p2: { label: "Iterations", min: 3, max: 9, step: 1, value: 7 },
    bounds: [-13, -13, -13, 13, 13, 13],
    build(nuc, p1, p2, matMode) {
      const power = p1.toFixed(1);
      const powerM1 = (p1 - 1).toFixed(1);
      const iters = Math.round(p2);
      const b = nuc.FieldExprBuilder.create();
      b.setBounds(-1.3, -1.3, -1.3, 1.3, 1.3, 1.3);
      b.setDistanceKind(nuc.FieldDistanceKind.Estimate);
      const bulb = b
        .compile(
          `let z = p; let dr = 1.0; let r = 0.0;\n` +
            `for i in 0..${iters} {\n` +
            `  r = length(z);\n` +
            `  if r > 3.5 { break; }\n` +
            `  let theta = acos(clamp(z.z / max(r, 1e-5), -1.0, 1.0)) * ${power};\n` +
            `  let phi = atan2(z.y, z.x) * ${power};\n` +
            `  dr = pow(max(r, 1e-5), ${powerM1}) * ${power} * dr + 1.0;\n` +
            `  let zr = pow(max(r, 1e-5), ${power});\n` +
            `  z = vec3(sin(theta)*cos(phi), sin(phi)*sin(theta), cos(theta)) * zr + p;\n` +
            `}\n` +
            `let rs = max(r, 1e-5);\n` +
            `0.5 * log(rs) * rs / max(dr, 1e-5)`,
        )
        .scale(10.2);

      const trap = nuc.Field.expr("clamp((length(p) - 4.0) / 8.5, 0.0, 1.0)");
      const mat = buildMaterial(nuc, matMode, trap, () => {
        const pal = nuc.Palette.fromBlockIds(GRADIENT_PALETTE_IDS).dithered();
        return nuc.Material.fieldGradient(
          trap,
          [0.0, 0.35, 0.68, 0.92, 1.0],
          [28, 22, 42, 42, 82, 168, 82, 164, 146, 246, 208, 68, 188, 232, 226],
          0.0,
          1.0,
          pal,
          nuc.InterpolationSpace.Oklab,
        );
      });

      const pipe = nuc.VoxelPipeline.create(bulb, mat);
      return pipe.toSchematicBounded("mandelbulb", ...this.bounds);
    },
  },

  arch: {
    label: "Chamfer colonnade",
    p1: { label: "Columns", min: 4, max: 10, step: 1, value: 6 },
    p2: { label: "Chamfer radius", min: 0.4, max: 3.0, step: 0.2, value: 1.4 },
    bounds: [-13, -9, -13, 13, 11, 13],
    build(nuc, p1, p2, matMode) {
      const base = nuc.Field.cappedCylinder(11.0, 1.5).translate(0, -6.5, 0).tag(1);
      const col = nuc.Field.cappedCylinder(1.5, 7.2)
        .translate(7.8, 0.5, 0)
        .repeatPolar(Math.round(p1))
        .tag(1);
      const ring = nuc.Field.torus(7.8, 1.8).translate(0, 7.5, 0).tag(2);
      const core = nuc.Field.octahedron(4.2).translate(0, 0.5, 0).tag(3);
      const geom = base
        .chamferUnion(col, p2)
        .smoothUnion(ring, p2 * 0.9)
        .unionWith(core);

      const noise = nuc.Field.perlinFbm3(0.14, 17, 3);
      const mat = buildMaterial(nuc, matMode, noise, () => {
        const matcher = nuc.MaterialBuilder.createTagMatcher(true, 11);
        matcher.addTagCase(
          1,
          nuc.Material.weighted(
            '["minecraft:smooth_sandstone","minecraft:calcite","minecraft:andesite"]',
            [0.55, 0.30, 0.15],
            31,
          ),
        );
        matcher.addTagCase(
          2,
          nuc.Material.weighted(
            '["minecraft:oxidized_copper","minecraft:weathered_copper","minecraft:prismarine_bricks"]',
            [0.50, 0.30, 0.20],
            53,
          ),
        );
        matcher.addTagCase(3, nuc.Material.solid("minecraft:sea_lantern"));
        matcher.setDefault(nuc.Material.solid("minecraft:stone"));
        return matcher.build();
      });

      const pipe = nuc.VoxelPipeline.create(geom, mat);
      return pipe.toSchematicBounded("chamfer_colonnade", ...this.bounds);
    },
  },

  custom: {
    label: "Custom Field.expr",
    p1: { label: "Scale", min: 0.7, max: 1.4, step: 0.05, value: 1.0 },
    p2: { label: "Iso offset", min: -1.5, max: 1.5, step: 0.1, value: 0.0 },
    bounds: [-12, -12, -12, 12, 12, 12],
    build(nuc, p1, p2, matMode, customSource) {
      const b = nuc.FieldExprBuilder.create();
      b.setBounds(-12, -12, -12, 12, 12, 12);
      b.setDistanceKind(nuc.FieldDistanceKind.Implicit);
      let field = b.compile(customSource);
      if (Math.abs(p1 - 1.0) > 1e-3) field = field.scale(p1);
      if (Math.abs(p2) > 1e-3) field = field.addConst(p2);
      const colorField = nuc.Field.expr("(y + 12.0) / 24.0");
      const mat = buildMaterial(nuc, matMode, colorField, () => {
        const pal = nuc.Palette.fromBlockIds(GRADIENT_PALETTE_IDS).dithered();
        return nuc.Material.fieldGradient(
          colorField,
          [0.0, 0.5, 1.0],
          [44, 52, 148, 82, 164, 146, 246, 208, 68],
          0.0,
          1.0,
          pal,
          nuc.InterpolationSpace.Oklab,
        );
      });
      const pipe = nuc.VoxelPipeline.create(field, mat);
      return pipe.toSchematicBounded("custom_expr", ...this.bounds);
    },
  },
};

function buildMaterial(nuc, mode, scalarField, pipelineFactory) {
  if (mode === "pipeline") return pipelineFactory();
  if (mode === "shaded") {
    const pal = nuc.Palette.fromBlockIds(SHADED_PALETTE_IDS).dithered();
    return nuc.Material.shaded(210, 232, 226, 0.55, 0.75, 0.35, 0.18, pal);
  }
  if (mode === "strata") {
    return nuc.Material.channelRamp(
      nuc.FieldChannel.WorldY,
      -10.0,
      10.0,
      '["minecraft:deepslate","minecraft:tuff","minecraft:stone","minecraft:andesite","minecraft:calcite","minecraft:smooth_sandstone"]',
      true,
    );
  }
  // "oklab"
  const pal = nuc.Palette.fromBlockIds(GRADIENT_PALETTE_IDS).dithered();
  return nuc.Material.fieldGradient(
    scalarField,
    [0.0, 0.5, 1.0],
    [42, 62, 158, 66, 178, 166, 246, 200, 62],
    -1.0,
    1.0,
    pal,
    nuc.InterpolationSpace.Oklab,
  );
}

/* ============================================================================
 * WIDGET 1: 3D VOXELPIPELINE & FIELD.EXPR LIVE WORKBENCH
 * ============================================================================ */

async function mountWorkbench(root, ensureModelViewer) {
  if (root.dataset.mounted === "true") return () => {};
  root.dataset.mounted = "true";

  let presetKey = "island";
  let matMode = "pipeline";
  let p1Val = PRESETS.island.p1.value;
  let p2Val = PRESETS.island.p2.value;
  let customExpr = CUSTOM_SNIPPETS[0].expr;
  let currentUrl = null;
  let lastSchematic = null;
  let buildSeq = 0;
  let debounceTimer = null;

  root.innerHTML = `
    <div class="sdf-studio__bar">
      <div class="sdf-seg" role="radiogroup" aria-label="Scene preset" data-role="presets">
        ${Object.entries(PRESETS)
          .map(
            ([k, p]) =>
              `<button type="button" class="sdf-seg__btn" role="radio" aria-checked="${
                k === presetKey
              }" data-preset="${k}">${p.label}</button>`,
          )
          .join("")}
      </div>
      <div class="sdf-seg sdf-seg--compact" role="radiogroup" aria-label="Material shading" data-role="materials">
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="true" data-mat="pipeline">Pipeline</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-mat="oklab">Oklab</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-mat="shaded">Shaded</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-mat="strata">Strata</button>
      </div>
    </div>

    <div class="sdf-studio__body">
      <div class="sdf-studio__stage">
        <model-viewer
          class="sdf-studio__viewer"
          alt="Interactive 3D Minecraft schematic generated in the browser by Nucleation WebAssembly"
          camera-controls
          auto-rotate
          auto-rotate-delay="1800"
          rotation-per-second="14deg"
          camera-orbit="38deg 66deg 118%"
          field-of-view="28deg"
          min-camera-orbit="auto auto 68%"
          max-camera-orbit="auto auto 220%"
          shadow-intensity="0.7"
          shadow-softness="0.85"
          exposure="1.05"
          interaction-prompt="none"
        ></model-viewer>
        <div class="sdf-studio__overlay" data-role="status">Loading Nucleation WASM...</div>
      </div>

      <div class="sdf-studio__controls">
        <div class="sdf-sliders">
          <label class="sdf-slider">
            <span class="sdf-slider__head">
              <span data-role="p1-label">${PRESETS.island.p1.label}</span>
              <output data-role="p1-val">${p1Val}</output>
            </span>
            <input type="range" data-role="p1-input" min="${PRESETS.island.p1.min}" max="${PRESETS.island.p1.max}" step="${PRESETS.island.p1.step}" value="${p1Val}">
          </label>
          <label class="sdf-slider">
            <span class="sdf-slider__head">
              <span data-role="p2-label">${PRESETS.island.p2.label}</span>
              <output data-role="p2-val">${p2Val}</output>
            </span>
            <input type="range" data-role="p2-input" min="${PRESETS.island.p2.min}" max="${PRESETS.island.p2.max}" step="${PRESETS.island.p2.step}" value="${p2Val}">
          </label>
        </div>

        <div class="sdf-expr-pane" data-role="expr-pane" hidden>
          <div class="sdf-expr-pane__head">
            <div class="sdf-expr-presets" role="group" aria-label="Expression examples">
              ${CUSTOM_SNIPPETS.map(
                (s, i) =>
                  `<button type="button" class="sdf-expr-snippet" data-snippet="${i}">${s.label}</button>`,
              ).join("")}
            </div>
            <button type="button" class="sdf-run-btn" data-role="expr-run">Run</button>
          </div>
          <textarea class="sdf-expr-input" data-role="expr-input" rows="4" spellcheck="false" aria-label="Field.expr source code">${customExpr}</textarea>
          <div class="sdf-expr-error" data-role="expr-error" hidden></div>
        </div>
      </div>
    </div>

    <div class="sdf-studio__footer">
      <div class="sdf-telemetry" data-role="telemetry">Initializing WebAssembly engine...</div>
      <button type="button" class="sdf-download-btn" data-role="download" disabled>Download .schem</button>
    </div>
  `;

  const viewer = root.querySelector(".sdf-studio__viewer");
  const statusEl = root.querySelector('[data-role="status"]');
  const telemetryEl = root.querySelector('[data-role="telemetry"]');
  const downloadBtn = root.querySelector('[data-role="download"]');
  const p1Label = root.querySelector('[data-role="p1-label"]');
  const p1ValEl = root.querySelector('[data-role="p1-val"]');
  const p1Input = root.querySelector('[data-role="p1-input"]');
  const p2Label = root.querySelector('[data-role="p2-label"]');
  const p2ValEl = root.querySelector('[data-role="p2-val"]');
  const p2Input = root.querySelector('[data-role="p2-input"]');
  const exprPane = root.querySelector('[data-role="expr-pane"]');
  const exprInput = root.querySelector('[data-role="expr-input"]');
  const exprError = root.querySelector('[data-role="expr-error"]');
  const exprRun = root.querySelector('[data-role="expr-run"]');

  function syncSliderConfig() {
    const preset = PRESETS[presetKey];
    p1Label.textContent = preset.p1.label;
    p1Input.min = String(preset.p1.min);
    p1Input.max = String(preset.p1.max);
    p1Input.step = String(preset.p1.step);
    p1Input.value = String(p1Val);
    p1ValEl.textContent = String(p1Val);

    p2Label.textContent = preset.p2.label;
    p2Input.min = String(preset.p2.min);
    p2Input.max = String(preset.p2.max);
    p2Input.step = String(preset.p2.step);
    p2Input.value = String(p2Val);
    p2ValEl.textContent = String(p2Val);

    exprPane.hidden = presetKey !== "custom";
  }

  let nuc = null;

  async function rebuild() {
    const seq = ++buildSeq;
    statusEl.textContent = "Voxelizing...";
    statusEl.hidden = false;
    exprError.hidden = true;

    try {
      if (!nuc) {
        await ensureModelViewer();
        nuc = await loadNucleation();
      }
      if (seq !== buildSeq) return;

      const t0 = performance.now();
      const preset = PRESETS[presetKey];
      const schem = preset.build(nuc, p1Val, p2Val, matMode, customExpr);
      const tVox = performance.now();

      const cfg = nuc.MeshConfig.create();
      cfg.setAmbientOcclusion(true);
      cfg.setAoIntensity(0.55);
      cfg.setGreedyMeshing(true);
      const mesh = nuc.MeshResult.create(schem, getResourcePack(nuc), cfg);
      const glbBytes = decodeBase64(mesh.glbDataB64());
      const tEnd = performance.now();

      if (seq !== buildSeq) return;

      const blocks = schem.blockCount();
      const tris = mesh.triangleCount();
      lastSchematic = schem;

      const nextUrl = URL.createObjectURL(new Blob([glbBytes], { type: "model/gltf-binary" }));
      if (currentUrl) URL.revokeObjectURL(currentUrl);
      currentUrl = nextUrl;
      viewer.src = nextUrl;

      statusEl.hidden = true;
      downloadBtn.disabled = false;
      telemetryEl.textContent =
        `${blocks.toLocaleString()} blocks · ${tris.toLocaleString()} triangles · ` +
        `${Math.round(tVox - t0)} ms voxelize · ${Math.round(tEnd - tVox)} ms mesh`;
    } catch (err) {
      if (seq !== buildSeq) return;
      statusEl.hidden = true;
      const msg = err instanceof Error ? err.message : String(err);
      if (presetKey === "custom") {
        exprError.textContent = `Expression error: ${msg}`;
        exprError.hidden = false;
      } else {
        telemetryEl.textContent = `Build error: ${msg}`;
      }
    }
  }

  function scheduleRebuild(delay = 90) {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => void rebuild(), delay);
  }

  root.querySelector('[data-role="presets"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-preset]");
    if (!btn) return;
    presetKey = btn.dataset.preset;
    for (const el of root.querySelectorAll("[data-preset]")) {
      el.setAttribute("aria-checked", String(el === btn));
    }
    p1Val = PRESETS[presetKey].p1.value;
    p2Val = PRESETS[presetKey].p2.value;
    syncSliderConfig();
    scheduleRebuild(0);
  });

  root.querySelector('[data-role="materials"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-mat]");
    if (!btn) return;
    matMode = btn.dataset.mat;
    for (const el of root.querySelectorAll("[data-mat]")) {
      el.setAttribute("aria-checked", String(el === btn));
    }
    scheduleRebuild(0);
  });

  p1Input.addEventListener("input", () => {
    p1Val = Number(p1Input.value);
    p1ValEl.textContent = String(p1Val);
    scheduleRebuild(75);
  });

  p2Input.addEventListener("input", () => {
    p2Val = Number(p2Input.value);
    p2ValEl.textContent = String(p2Val);
    scheduleRebuild(75);
  });

  exprPane.addEventListener("click", (e) => {
    const btn = e.target.closest("[data-snippet]");
    if (!btn) return;
    const idx = Number(btn.dataset.snippet);
    customExpr = CUSTOM_SNIPPETS[idx].expr;
    exprInput.value = customExpr;
    scheduleRebuild(0);
  });

  exprRun.addEventListener("click", () => {
    customExpr = exprInput.value;
    scheduleRebuild(0);
  });

  exprInput.addEventListener("keydown", (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      customExpr = exprInput.value;
      scheduleRebuild(0);
    }
  });

  downloadBtn.addEventListener("click", () => {
    if (!lastSchematic) return;
    const bytes = decodeBase64(lastSchematic.toSchematicB64());
    const blobUrl = URL.createObjectURL(new Blob([bytes], { type: "application/octet-stream" }));
    const a = document.createElement("a");
    a.href = blobUrl;
    a.download = `nucleation-${presetKey}.schem`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(blobUrl), 2000);
  });

  syncSliderConfig();
  void rebuild();

  return () => {
    clearTimeout(debounceTimer);
    if (currentUrl) URL.revokeObjectURL(currentUrl);
    root.dataset.mounted = "false";
  };
}

/* ============================================================================
 * WIDGET 2: INTERACTIVE 2D SDF, CSG, DUAL NORMALS & 8x8 BRICK-PRUNING LAB
 * ============================================================================ */

function sdCircle(px, py, cx, cy, r) {
  return Math.hypot(px - cx, py - cy) - r;
}

function sdRoundBox(px, py, cx, cy, hx, hy, r) {
  const qx = Math.abs(px - cx) - hx + r;
  const qy = Math.abs(py - cy) - hy + r;
  return Math.hypot(Math.max(qx, 0), Math.max(qy, 0)) + Math.min(Math.max(qx, qy), 0) - r;
}

function opCSG(a, b, mode, k, steps) {
  if (mode === "union") return Math.min(a, b);
  if (mode === "smooth_union") {
    const h = Math.max(k - Math.abs(a - b), 0) / k;
    return Math.min(a, b) - h * h * k * 0.25;
  }
  if (mode === "chamfer") {
    return Math.min(Math.min(a, b), (a - k + b) * Math.SQRT1_2);
  }
  if (mode === "steps") {
    const s = k / Math.max(steps, 1);
    const u = b - k;
    return Math.min(Math.min(a, b), 0.5 * (u + a + Math.abs(((u - a + s) % (2 * s) + 2 * s) % (2 * s) - s)));
  }
  if (mode === "smooth_sub") {
    const h = Math.max(k - Math.abs(-b - a), 0) / k;
    return Math.max(a, -b) + h * h * k * 0.25;
  }
  // smooth_intersect
  const h = Math.max(k - Math.abs(a - b), 0) / k;
  return Math.max(a, b) + h * h * k * 0.25;
}

function mountCsgLab(root) {
  if (root.dataset.mounted === "true") return () => {};
  root.dataset.mounted = "true";

  const state = {
    mode: "smooth_union",
    k: 3.2,
    steps: 4,
    showIsolines: true,
    showNormals: true,
    showBricks: true,
    ax: -6.5,
    ay: 0.0,
    ar: 7.2,
    bx: 5.5,
    by: 1.0,
    bhx: 6.2,
    bhy: 5.4,
    br: 1.0,
    dragging: null,
    hover: null,
  };

  root.innerHTML = `
    <div class="sdf-studio__bar">
      <div class="sdf-seg" role="radiogroup" aria-label="CSG operator" data-role="csg-ops">
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-op="union">Union</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="true" data-op="smooth_union">Smooth union</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-op="chamfer">Chamfer</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-op="steps">Steps</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-op="smooth_sub">Smooth subtract</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-op="smooth_int">Smooth intersect</button>
      </div>
    </div>

    <div class="sdf-csg__stage">
      <canvas class="sdf-csg__canvas" width="900" height="400" aria-label="Interactive 2D signed distance field and 8x8 brick pruning canvas"></canvas>
    </div>

    <div class="sdf-csg__controls">
      <label class="sdf-slider sdf-slider--inline">
        <span class="sdf-slider__head">
          <span data-role="k-label">Blend radius (k)</span>
          <output data-role="k-val">${state.k.toFixed(1)}</output>
        </span>
        <input type="range" data-role="k-input" min="0.2" max="6.0" step="0.2" value="${state.k}">
      </label>

      <div class="sdf-toggles">
        <label class="sdf-check"><input type="checkbox" data-toggle="isolines" checked><span>Isolines</span></label>
        <label class="sdf-check"><input type="checkbox" data-toggle="normals" checked><span>Dual normals</span></label>
        <label class="sdf-check"><input type="checkbox" data-toggle="bricks" checked><span>8×8 interval bricks</span></label>
      </div>
    </div>

    <div class="sdf-studio__footer">
      <div class="sdf-telemetry" data-role="csg-telemetry">Drag either shape handle directly on the field.</div>
      <div class="sdf-telemetry sdf-telemetry--right" data-role="brick-stats"></div>
    </div>
  `;

  const canvas = root.querySelector(".sdf-csg__canvas");
  const ctx = canvas.getContext("2d");
  const kInput = root.querySelector('[data-role="k-input"]');
  const kVal = root.querySelector('[data-role="k-val"]');
  const kLabel = root.querySelector('[data-role="k-label"]');
  const telemetry = root.querySelector('[data-role="csg-telemetry"]');
  const brickStats = root.querySelector('[data-role="brick-stats"]');

  // World coordinate extent: X in [-24, 24] (48 blocks = 6 bricks of 8), Y in [-12, 12] (24 blocks = 3 bricks of 8)
  const WORLD_W = 48;
  const WORLD_H = 24;

  function evalField(wx, wy) {
    const a = sdCircle(wx, wy, state.ax, state.ay, state.ar);
    const b = sdRoundBox(wx, wy, state.bx, state.by, state.bhx, state.bhy, state.br);
    return opCSG(a, b, state.mode, state.k, state.steps);
  }

  function evalNormal(wx, wy) {
    const e = 0.05;
    const dx = evalField(wx + e, wy) - evalField(wx - e, wy);
    const dy = evalField(wx, wy + e) - evalField(wx, wy - e);
    const len = Math.hypot(dx, dy);
    return len > 1e-6 ? [dx / len, dy / len] : [0, 1];
  }

  function themePalette() {
    const light = document.body.getAttribute("data-md-color-scheme") === "vellum";
    return light
      ? {
          bg: "#FCFBF9",
          insideTint: [26, 117, 100],
          outsideTint: [110, 116, 128],
          contour: "#146458",
          isolineIn: "rgba(26, 117, 100, 0.28)",
          isolineOut: "rgba(42, 46, 54, 0.14)",
          brickPruned: "rgba(179, 66, 66, 0.08)",
          brickPrunedStroke: "rgba(179, 66, 66, 0.28)",
          brickSolid: "rgba(26, 117, 100, 0.10)",
          brickSolidStroke: "rgba(26, 117, 100, 0.35)",
          brickSurfaceStroke: "rgba(154, 101, 8, 0.55)",
          normal: "#9A6508",
          handleA: "#1A7564",
          handleB: "#275BA0",
          handleText: "#FFFFFF",
          cursor: "#12151A",
        }
      : {
          bg: "#12161C",
          insideTint: [95, 209, 188],
          outsideTint: [138, 147, 163],
          contour: "#7FF0DC",
          isolineIn: "rgba(95, 209, 188, 0.30)",
          isolineOut: "rgba(228, 231, 236, 0.11)",
          brickPruned: "rgba(242, 153, 153, 0.07)",
          brickPrunedStroke: "rgba(242, 153, 153, 0.24)",
          brickSolid: "rgba(95, 209, 188, 0.10)",
          brickSolidStroke: "rgba(95, 209, 188, 0.36)",
          brickSurfaceStroke: "rgba(242, 205, 135, 0.58)",
          normal: "#F2CD87",
          handleA: "#5FD1BC",
          handleB: "#8AB8FF",
          handleText: "#0E1014",
          cursor: "#F6F8FA",
        };
  }

  function worldToPx(wx, wy, w, h) {
    return [((wx + WORLD_W / 2) / WORLD_W) * w, ((WORLD_H / 2 - wy) / WORLD_H) * h];
  }

  function pxToWorld(px, py, w, h) {
    return [(px / w) * WORLD_W - WORLD_W / 2, WORLD_H / 2 - (py / h) * WORLD_H];
  }

  function render() {
    const rect = canvas.getBoundingClientRect();
    const cssW = Math.max(300, Math.round(rect.width));
    const cssH = Math.round(cssW * (WORLD_H / WORLD_W));
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    if (canvas.width !== Math.round(cssW * dpr) || canvas.height !== Math.round(cssH * dpr)) {
      canvas.width = Math.round(cssW * dpr);
      canvas.height = Math.round(cssH * dpr);
    }

    const pal = themePalette();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.fillStyle = pal.bg;
    ctx.fillRect(0, 0, cssW, cssH);

    // 1. Sample distance grid (coarse 2px resolution for fast 60fps contours)
    const step = 3;
    const cols = Math.ceil(cssW / step) + 1;
    const rows = Math.ceil(cssH / step) + 1;
    const grid = new Float32Array(cols * rows);

    for (let j = 0; j < rows; j += 1) {
      const py = j * step;
      const wy = WORLD_H / 2 - (py / cssH) * WORLD_H;
      for (let i = 0; i < cols; i += 1) {
        const px = i * step;
        const wx = (px / cssW) * WORLD_W - WORLD_W / 2;
        const d = evalField(wx, wy);
        grid[j * cols + i] = d;

        if (d <= 0) {
          const depth = Math.min(1, -d / 8.0);
          const alpha = 0.14 + depth * 0.22;
          ctx.fillStyle = `rgba(${pal.insideTint[0]},${pal.insideTint[1]},${pal.insideTint[2]},${alpha.toFixed(3)})`;
          ctx.fillRect(px, py, step, step);
        }
      }
    }

    // 2. Crisp isolines & d=0 surface contour (+ collect exact contour anchors for normals)
    const contourAnchors = [];
    if (state.showIsolines || state.showNormals) {
      const levels = state.showIsolines ? [-6, -4, -2, 0, 2, 4, 6, 8, 10] : [0];
      for (const lv of levels) {
        ctx.beginPath();
        ctx.lineWidth = lv === 0 ? 2.2 : 1.0;
        ctx.strokeStyle = lv === 0 ? pal.contour : lv < 0 ? pal.isolineIn : pal.isolineOut;
        for (let j = 0; j < rows - 1; j += 1) {
          for (let i = 0; i < cols - 1; i += 1) {
            const idx = j * cols + i;
            const v00 = grid[idx] - lv;
            const v10 = grid[idx + 1] - lv;
            const v01 = grid[idx + cols] - lv;
            const v11 = grid[idx + cols + 1] - lv;
            const mask = (v00 > 0 ? 1 : 0) | (v10 > 0 ? 2 : 0) | (v11 > 0 ? 4 : 0) | (v01 > 0 ? 8 : 0);
            if (mask === 0 || mask === 15) continue;
            const x0 = i * step;
            const y0 = j * step;
            const pts = [];
            if ((v00 > 0) !== (v10 > 0)) pts.push([x0 + (step * v00) / (v00 - v10), y0]);
            if ((v10 > 0) !== (v11 > 0)) pts.push([x0 + step, y0 + (step * v10) / (v10 - v11)]);
            if ((v01 > 0) !== (v11 > 0)) pts.push([x0 + (step * v01) / (v01 - v11), y0 + step]);
            if ((v00 > 0) !== (v01 > 0)) pts.push([x0, y0 + (step * v00) / (v00 - v01)]);
            if (pts.length >= 2) {
              if (state.showIsolines || lv === 0) {
                ctx.moveTo(pts[0][0], pts[0][1]);
                ctx.lineTo(pts[1][0], pts[1][1]);
              }
              if (lv === 0 && state.showNormals) {
                contourAnchors.push([
                  (pts[0][0] + pts[1][0]) * 0.5,
                  (pts[0][1] + pts[1][1]) * 0.5,
                ]);
              }
            }
          }
        }
        ctx.stroke();
      }
    }

    // 3. 8x8 Conservative Interval Brick Pruning Overlay
    let totalBricks = 0;
    let prunedBricks = 0;
    let solidBricks = 0;
    let surfaceBricks = 0;
    const brickSize = 8; // 8x8 world units
    const halfDiag = Math.hypot(brickSize * 0.5, brickSize * 0.5);
    const lipBoost = state.mode === "steps" ? 1.35 : 1.05;

    for (let by = -WORLD_H / 2; by < WORLD_H / 2; by += brickSize) {
      for (let bx = -WORLD_W / 2; bx < WORLD_W / 2; bx += brickSize) {
        totalBricks += 1;
        const cx = bx + brickSize * 0.5;
        const cy = by + brickSize * 0.5;
        const dc = evalField(cx, cy);
        const bound = halfDiag * lipBoost;
        const lo = dc - bound;
        const hi = dc + bound;

        let kind = "surface";
        if (lo > 0) {
          kind = "pruned";
          prunedBricks += 1;
        } else if (hi <= 0) {
          kind = "solid";
          solidBricks += 1;
        } else {
          surfaceBricks += 1;
        }

        if (state.showBricks) {
          const [px0, py0] = worldToPx(bx, by + brickSize, cssW, cssH);
          const [px1, py1] = worldToPx(bx + brickSize, by, cssW, cssH);
          const bw = px1 - px0;
          const bh = py1 - py0;

          if (kind === "pruned") {
            ctx.fillStyle = pal.brickPruned;
            ctx.fillRect(px0 + 1, py0 + 1, bw - 2, bh - 2);
            ctx.strokeStyle = pal.brickPrunedStroke;
            ctx.setLineDash([3, 3]);
            ctx.lineWidth = 1;
            ctx.strokeRect(px0 + 0.5, py0 + 0.5, bw - 1, bh - 1);
            ctx.setLineDash([]);
          } else if (kind === "solid") {
            ctx.fillStyle = pal.brickSolid;
            ctx.fillRect(px0 + 1, py0 + 1, bw - 2, bh - 2);
            ctx.strokeStyle = pal.brickSolidStroke;
            ctx.lineWidth = 1;
            ctx.strokeRect(px0 + 0.5, py0 + 0.5, bw - 1, bh - 1);
          } else {
            ctx.strokeStyle = pal.brickSurfaceStroke;
            ctx.lineWidth = 1.25;
            ctx.strokeRect(px0 + 0.5, py0 + 0.5, bw - 1, bh - 1);
          }
        }
      }
    }

    const prunedPct = Math.round((prunedBricks / totalBricks) * 100);
    brickStats.textContent =
      `${prunedBricks}/${totalBricks} bricks pruned (${prunedPct}% skipped) · ${surfaceBricks} surface`;

    // 4. Dual-number gradient / normal vectors anchored directly on the d = 0 contour
    if (state.showNormals && contourAnchors.length > 0) {
      ctx.strokeStyle = pal.normal;
      ctx.fillStyle = pal.normal;
      ctx.lineWidth = 1.3;
      const drawn = [];
      const minSpacingSq = 30 * 30;
      for (const [px, py] of contourAnchors) {
        let tooClose = false;
        for (const [dx, dy] of drawn) {
          if ((px - dx) * (px - dx) + (py - dy) * (py - dy) < minSpacingSq) {
            tooClose = true;
            break;
          }
        }
        if (tooClose) continue;
        drawn.push([px, py]);
        const [wx, wy] = pxToWorld(px, py, cssW, cssH);
        const [nx, ny] = evalNormal(wx, wy);
        const len = 15;
        const ex = px + nx * len;
        const ey = py - ny * len;
        ctx.beginPath();
        ctx.moveTo(px, py);
        ctx.lineTo(ex, ey);
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(ex, ey, 2.1, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    // 5. Draggable shape center handles (A and B)
    for (const [label, wx, wy, color] of [
      ["A", state.ax, state.ay, pal.handleA],
      ["B", state.bx, state.by, pal.handleB],
    ]) {
      const [px, py] = worldToPx(wx, wy, cssW, cssH);
      ctx.beginPath();
      ctx.arc(px, py, 10, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();
      ctx.font = '600 11px "Geist Mono", ui-monospace, monospace';
      ctx.fillStyle = pal.handleText;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(label, px, py + 0.5);
    }

    // 6. Live cursor distance probe
    if (state.hover) {
      const [wx, wy] = state.hover;
      const d = evalField(wx, wy);
      const [nx, ny] = evalNormal(wx, wy);
      const [px, py] = worldToPx(wx, wy, cssW, cssH);
      const pxRadius = (Math.abs(d) / WORLD_W) * cssW;
      ctx.beginPath();
      ctx.arc(px, py, Math.max(2, pxRadius), 0, Math.PI * 2);
      ctx.strokeStyle = pal.cursor;
      ctx.globalAlpha = 0.45;
      ctx.lineWidth = 1;
      ctx.setLineDash([4, 3]);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.globalAlpha = 1;
      ctx.beginPath();
      ctx.arc(px, py, 3, 0, Math.PI * 2);
      ctx.fillStyle = pal.cursor;
      ctx.fill();
      telemetry.textContent =
        `p = (${wx.toFixed(1)}, ${wy.toFixed(1)}) · d = ${d >= 0 ? "+" : ""}${d.toFixed(2)} · ` +
        `n = (${nx.toFixed(2)}, ${ny.toFixed(2)})`;
    }
  }

  function pointerToWorld(e) {
    const rect = canvas.getBoundingClientRect();
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    return pxToWorld(px, py, rect.width, rect.height);
  }

  canvas.addEventListener("pointerdown", (e) => {
    const [wx, wy] = pointerToWorld(e);
    const da = Math.hypot(wx - state.ax, wy - state.ay);
    const db = Math.hypot(wx - state.bx, wy - state.by);
    if (da < 3.0 || da <= db) {
      state.dragging = da < 4.5 ? "a" : db < 4.5 ? "b" : null;
    } else if (db < 4.5) {
      state.dragging = "b";
    }
    if (state.dragging) {
      canvas.setPointerCapture(e.pointerId);
      canvas.style.cursor = "grabbing";
    }
  });

  canvas.addEventListener("pointermove", (e) => {
    const [wx, wy] = pointerToWorld(e);
    state.hover = [wx, wy];
    if (state.dragging === "a") {
      state.ax = Math.max(-20, Math.min(20, wx));
      state.ay = Math.max(-9, Math.min(9, wy));
    } else if (state.dragging === "b") {
      state.bx = Math.max(-20, Math.min(20, wx));
      state.by = Math.max(-9, Math.min(9, wy));
    } else {
      const nearHandle =
        Math.hypot(wx - state.ax, wy - state.ay) < 2.5 ||
        Math.hypot(wx - state.bx, wy - state.by) < 2.5;
      canvas.style.cursor = nearHandle ? "grab" : "crosshair";
    }
    render();
  });

  const endDrag = () => {
    state.dragging = null;
    canvas.style.cursor = "crosshair";
  };
  canvas.addEventListener("pointerup", endDrag);
  canvas.addEventListener("pointercancel", endDrag);
  canvas.addEventListener("pointerleave", () => {
    if (!state.dragging) {
      state.hover = null;
      telemetry.textContent = "Drag shape handle A or B, or hover anywhere to inspect d(p) and ∇f.";
      render();
    }
  });

  root.querySelector('[data-role="csg-ops"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-op]");
    if (!btn) return;
    state.mode = btn.dataset.op;
    for (const el of root.querySelectorAll("[data-op]")) {
      el.setAttribute("aria-checked", String(el === btn));
    }
    kInput.disabled = state.mode === "union";
    kLabel.textContent =
      state.mode === "chamfer"
        ? "Chamfer radius (r)"
        : state.mode === "steps"
        ? "Terrace span (r)"
        : "Blend radius (k)";
    render();
  });

  kInput.addEventListener("input", () => {
    state.k = Number(kInput.value);
    kVal.textContent = state.k.toFixed(1);
    render();
  });

  for (const input of root.querySelectorAll("[data-toggle]")) {
    input.addEventListener("change", () => {
      const key = input.dataset.toggle;
      if (key === "isolines") state.showIsolines = input.checked;
      if (key === "normals") state.showNormals = input.checked;
      if (key === "bricks") state.showBricks = input.checked;
      render();
    });
  }

  const ro = new ResizeObserver(() => render());
  ro.observe(canvas);
  render();

  return {
    rerender: render,
    destroy: () => {
      ro.disconnect();
      root.dataset.mounted = "false";
    },
  };
}

export async function mountSdfPlaygrounds(ensureModelViewer) {
  const cleanups = [];
  const rerenders = [];

  for (const el of document.querySelectorAll("[data-sdf-workbench]")) {
    const destroy = await mountWorkbench(el, ensureModelViewer);
    cleanups.push(destroy);
  }

  for (const el of document.querySelectorAll("[data-sdf-csg-lab]")) {
    const lab = mountCsgLab(el);
    if (lab?.rerender) rerenders.push(lab.rerender);
    if (lab?.destroy) cleanups.push(lab.destroy);
  }

  return {
    onThemeChange: () => {
      for (const fn of rerenders) fn();
    },
    destroy: () => {
      for (const fn of cleanups) fn();
    },
  };
}
