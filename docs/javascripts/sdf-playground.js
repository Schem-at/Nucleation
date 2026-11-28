/**
 * Nucleation v0.13.0 Interactive Browser Studios
 * 1. Progressive 3D SDF & VoxelPipeline Studio (real multi-face Minecraft blocks + custom pack .zip support)
 * 2. 2D SDF, CSG, Dual3 Autodiff Normals & 8x8 Brick-Pruning Lab (evaluated live by Nucleation WASM Field)
 * 3. Infinite WorldGenerator Chunk Streamer (WorldGenerator.pipeline + cellularSdf + composite)
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
    import("./vendor/nucleation/WorldGenerator.mjs"),
    import("./vendor/nucleation/CellularSdfConfig.mjs"),
    import("./vendor/nucleation/GeneratedChunkOverlayMode.mjs"),
    import("./vendor/nucleation/Brush.mjs"),
    import("./vendor/nucleation/TickSimulation.mjs"),
    import("./vendor/nucleation/TickSettleMode.mjs"),
  ]).then((mods) => Object.assign({}, ...mods));
  return wasmPromise;
}

/* ============================================================================
 * BUILT-IN PROCEDURAL MINECRAFT BLOCK PACK (+ OPTIONAL USER .ZIP OVERRIDE)
 * Powered directly by Rust/WASM `ResourcePack.builtinProcedural()`!
 * ============================================================================ */

let activePack = null;
let customPackName = null;
const packListeners = new Set();

function getActivePack(nuc) {
  if (!activePack) activePack = nuc.ResourcePack.builtinProcedural();
  return activePack;
}

async function loadCustomPackFile(nuc, file) {
  const bytes = new Uint8Array(await file.arrayBuffer());
  activePack = nuc.ResourcePack.fromBytes(bytes);
  customPackName = file.name;
  for (const fn of packListeners) fn(customPackName);
}

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
  "minecraft:white_concrete",
  "minecraft:calcite",
  "minecraft:smooth_sandstone",
]);

function decodeBase64(value) {
  const binary = globalThis.atob(value);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) out[i] = binary.charCodeAt(i);
  return out;
}

/* ============================================================================
 * WIDGET 1: PROGRESSIVE 5-STEP 3D MINECRAFT SDF & VOXELPIPELINE STUDIO
 * ============================================================================ */

const CUSTOM_SNIPPETS = [
  {
    label: "Gyroid temple",
    expr: `let q = p * 0.40;\nlet g = abs(sin(q.x)*cos(q.y) + sin(q.y)*cos(q.z) + sin(q.z)*cos(q.x)) - 0.30;\nmax(g, sphere(p, 11.4))`,
  },
  {
    label: "Power-8 Mandelbulb",
    expr: `let z = p / 9.5; let dr = 1.0; let r = 0.0;\nfor i in 0..7 {\n  r = length(z);\n  if r > 3.5 { break; }\n  let th = acos(clamp(z.z / max(r, 1e-5), -1.0, 1.0)) * 8.0;\n  let ph = atan2(z.y, z.x) * 8.0;\n  dr = pow(max(r, 1e-5), 7.0) * 8.0 * dr + 1.0;\n  let zr = pow(max(r, 1e-5), 8.0);\n  z = vec3(sin(th)*cos(ph), sin(ph)*sin(th), cos(th)) * zr + p / 9.5;\n}\nlet rs = max(r, 1e-5);\n0.5 * log(rs) * rs / max(dr, 1e-5) * 9.5`,
  },
  {
    label: "Twisted spire",
    expr: `let a = y * 0.22;\nlet q = vec3(x*cos(a) - z*sin(a), y, x*sin(a) + z*cos(a));\nbox(q, vec3(5.5 - y * 0.28, 10.5, 5.5 - y * 0.28)) - 0.6`,
  },
  {
    label: "Volcanic crater",
    expr: `let r = length(vec3(x, 0.0, z));\nlet cone = y - (10.0 - r * 0.85) + exp(-r * r * 0.05) * 7.0;\nmax(cone, length(vec3(x, 0.0, z)) - 11.5)`,
  },
  {
    label: "Orbital rings",
    expr: `let r1 = torus(p, 9.0, 1.8);\nlet r2 = torus(vec3(x, z, y), 9.0, 1.8);\nlet r3 = torus(vec3(y, x, z), 9.0, 1.8);\nsmin(smin(r1, r2, 1.6), r3, 1.6)`,
  },
];

const WORKBENCH_STEPS = {
  sphere: {
    label: "1. Sphere",
    p1: { label: "Radius", min: 4, max: 11, step: 0.5, value: 8.5 },
    p2: { label: "Hollow shell (0 = solid)", min: 0, max: 2.5, step: 0.5, value: 0 },
    p3: null,
    bounds: [-12, -12, -12, 12, 12, 12],
    code(lang, p1, p2) {
      if (lang === "js") {
        const shape =
          p2 > 0
            ? `const geom = Field.sphere(${p1})\n  .shell(${p2})\n  .subtract(Field.boxShape(8, 8, 8, 0).translate(7, 5, 7));`
            : `const geom = Field.sphere(${p1});`;
        return `${shape}
const mat = Material.weighted(
  '["minecraft:stone_bricks","minecraft:mossy_stone_bricks","minecraft:cobblestone"]',
  [0.62, 0.24, 0.14], 17,
);
const schem = VoxelPipeline.create(geom, mat).toSchematic("first_sphere");`;
      }
      const shape =
        p2 > 0
          ? `geom = (
    Field.sphere(${p1})
    .shell(${p2})
    .subtract(Field.box_shape(8, 8, 8, 0).translate(7, 5, 7))
)`
          : `geom = Field.sphere(${p1})`;
      return `${shape}
mat = Material.weighted(
    '["minecraft:stone_bricks","minecraft:mossy_stone_bricks","minecraft:cobblestone"]',
    [0.62, 0.24, 0.14], 17,
)
schem = VoxelPipeline.create(geom, mat).to_schematic("first_sphere")`;
    },
    build(nuc, p1, p2) {
      let geom = nuc.Field.sphere(p1);
      if (p2 > 0) {
        geom = geom.shell(p2).subtract(nuc.Field.boxShape(8, 8, 8, 0).translate(7, 5, 7));
      }
      const mat = nuc.Material.weighted(
        '["minecraft:stone_bricks","minecraft:mossy_stone_bricks","minecraft:cobblestone"]',
        [0.62, 0.24, 0.14],
        17,
      );
      return nuc.VoxelPipeline.create(geom, mat).toSchematicBounded("first_sphere", ...this.bounds);
    },
  },

  arch: {
    label: "2. Carve & blend",
    p1: { label: "Move sphere (X)", min: -8.0, max: 8.0, step: 0.5, value: 3.5 },
    p2: { label: "Smooth blend (k)", min: 0.0, max: 4.0, step: 0.2, value: 2.2 },
    p3: { label: "Archway carve", min: 0.0, max: 5.0, step: 0.5, value: 3.5 },
    bounds: [-14, -8, -12, 14, 13, 12],
    code(lang, p1, p2, p3) {
      const k = Number(p2).toFixed(1);
      const sx = Number(p1).toFixed(1);
      if (lang === "js") {
        const combine = p2 > 0.05 ? `tower.smoothUnion(sphere, ${k})` : `tower.unionWith(sphere)`;
        const carve =
          p3 > 0.1
            ? `\nconst arch = Field.cappedCylinder(${p3}, 10).rotate(90, 0, 0).translate(0, -2.5, 0);\nconst geom = ${combine}.subtract(arch);`
            : `\nconst geom = ${combine};`;
        return `const tower  = Field.boxShape(6.5, 6.5, 6.5, 0.8).translate(0, -1, 0).tag(1);
const sphere = Field.sphere(6.0).translate(${sx}, 5.0, 0).tag(2);${carve}

// Smooth seam dithering across tag 1 (stone_bricks) & tag 2 (oxidized_copper)
const m = MaterialBuilder.createTagMatcher(true, 41);
m.addTagCase(1, stoneBricksMat);
m.addTagCase(2, oxidizedCopperMat);
const schem = VoxelPipeline.create(geom, m.build()).toSchematic("carve_and_blend");`;
      }
      const combine = p2 > 0.05 ? `tower.smooth_union(sphere, ${k})` : `tower.union_with(sphere)`;
      const carve =
        p3 > 0.1
          ? `\narch = Field.capped_cylinder(${p3}, 10).rotate(90, 0, 0).translate(0, -2.5, 0)\ngeom = ${combine}.subtract(arch)`
          : `\ngeom = ${combine}`;
      return `tower  = Field.box_shape(6.5, 6.5, 6.5, 0.8).translate(0, -1, 0).tag(1)
sphere = Field.sphere(6.0).translate(${sx}, 5.0, 0).tag(2)${carve}

# Smooth seam dithering across tag 1 (stone_bricks) & tag 2 (oxidized_copper)
m = MaterialBuilder.create_tag_matcher(True, 41)
m.add_tag_case(1, stone_bricks_mat)
m.add_tag_case(2, oxidized_copper_mat)
schem = VoxelPipeline.create(geom, m.build()).to_schematic("carve_and_blend")`;
    },
    build(nuc, p1, p2, p3) {
      const tower = nuc.Field.boxShape(6.5, 6.5, 6.5, 0.8).translate(0, -1, 0).tag(1);
      const sphere = nuc.Field.sphere(6.0).translate(p1, 5.0, 0).tag(2);
      let geom = p2 > 0.05 ? tower.smoothUnion(sphere, p2) : tower.unionWith(sphere);
      if (p3 > 0.1) {
        const doorwayZ = nuc.Field.cappedCylinder(p3, 10.0).rotate(90, 0, 0).translate(0, -2.5, 0);
        const doorwayX = nuc.Field.cappedCylinder(p3, 10.0).rotate(0, 0, 90).translate(0, -2.5, 0);
        geom = geom.subtract(doorwayZ).subtract(doorwayX);
      }

      const m = nuc.MaterialBuilder.createTagMatcher(true, 41);
      m.addTagCase(
        1,
        nuc.Material.weighted(
          '["minecraft:stone_bricks","minecraft:mossy_stone_bricks","minecraft:stone"]',
          [0.58, 0.24, 0.18],
          11,
        ),
      );
      m.addTagCase(
        2,
        nuc.Material.weighted(
          '["minecraft:oxidized_copper","minecraft:weathered_copper","minecraft:prismarine_bricks"]',
          [0.52, 0.28, 0.20],
          29,
        ),
      );
      m.setDefault(nuc.Material.solid("minecraft:stone_bricks"));

      return nuc.VoxelPipeline.create(geom, m.build()).toSchematicBounded("carve_and_blend", ...this.bounds);
    },
  },

  island: {
    label: "3. Island",
    p1: { label: "Terrain roughness", min: 0.5, max: 3.8, step: 0.25, value: 2.0 },
    p2: { label: "Lagoon water level", min: -1, max: 4, step: 1, value: 2 },
    p3: null,
    bounds: [-14, -8, -14, 14, 14, 14],
    code(lang, p1, p2) {
      if (lang === "js") {
        return `const noise   = Field.perlinFbm3(0.09, 42, 4);
const lagoon  = Field.sphere(6.2).translate(3.2, 5.8, 2.5);
const terrain = Field.ellipsoid(13, 6.6, 13)
  .displace(noise, ${p1}).subtract(lagoon).tag(1);
const tree = trunk.tag(2).unionWith(crown.tag(3));

// Exposed +Y flat tops -> grass_block; depth 1..2 -> dirt; stone below
layers.addLayer(
  MaterialCondition.exposedTop().andWith(MaterialCondition.slopeBelow(0.72)),
  Material.solid("minecraft:grass_block"),
);
layers.addLayer(
  MaterialCondition.depthBetween(1, 2),
  Material.solid("minecraft:dirt"),
);

const pipe = VoxelPipeline.create(terrain.unionWith(tree), matcher.build());
pipe.addFluid(lagoon.intersectionWith(Field.plane(0, 1, 0, ${-p2})), "minecraft:water");
pipe.addScatter(flora);  // short_grass, poppy, dandelion on grass_block`;
      }
      return `noise   = Field.perlin_fbm3(0.09, 42, 4)
lagoon  = Field.sphere(6.2).translate(3.2, 5.8, 2.5)
terrain = (
    Field.ellipsoid(13, 6.6, 13)
    .displace(noise, ${p1}).subtract(lagoon).tag(1)
)
tree = trunk.tag(2).union_with(crown.tag(3))

# Exposed +Y flat tops -> grass_block; depth 1..2 -> dirt; stone below
layers.add_layer(
    MaterialCondition.exposed_top().and_with(MaterialCondition.slope_below(0.72)),
    Material.solid("minecraft:grass_block"),
)
layers.add_layer(
    MaterialCondition.depth_between(1, 2),
    Material.solid("minecraft:dirt"),
)

pipe = VoxelPipeline.create(terrain.union_with(tree), matcher.build())
pipe.add_fluid(lagoon.intersection_with(Field.plane(0, 1, 0, ${-p2})), "minecraft:water")
pipe.add_scatter(flora)  # short_grass, poppy, dandelion on grass_block`;
    },
    build(nuc, p1, p2) {
      const noise = nuc.Field.perlinFbm3(0.09, 42, 4);
      const crater = nuc.Field.sphere(6.2).translate(3.2, 5.8, 2.5);
      const terrain = nuc.Field.ellipsoid(13, 6.6, 13)
        .displace(noise, p1)
        .subtract(crater)
        .tag(1);
      const trunk = nuc.Field.cappedCylinder(0.95, 3.5).translate(-5.5, 7.2, -3.5).tag(2);
      const crown = nuc.Field.sphere(3.6)
        .displace(nuc.Field.perlinFbm3(0.35, 9, 2), 0.7)
        .translate(-5.5, 11.2, -3.5)
        .tag(3);
      const geom = terrain.unionWith(trunk).unionWith(crown);

      const layers = nuc.MaterialBuilder.createLayered();
      layers.addLayer(
        nuc.MaterialCondition.exposedTop().andWith(nuc.MaterialCondition.slopeBelow(0.72)),
        nuc.Material.solid("minecraft:grass_block"),
      );
      layers.addLayer(
        nuc.MaterialCondition.depthBetween(1, 2),
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
      const matcher = nuc.MaterialBuilder.createTagMatcher(false, 73);
      matcher.addTagCase(1, layers.build());
      matcher.addTagCase(2, nuc.Material.solid("minecraft:oak_log"));
      matcher.addTagCase(3, nuc.Material.solid("minecraft:oak_leaves"));
      matcher.setDefault(nuc.Material.solid("minecraft:stone"));

      const pipe = nuc.VoxelPipeline.create(geom, matcher.build());
      const lagoonPool = crater.intersectionWith(nuc.Field.plane(0, 1, 0, -p2));
      pipe.addFluid(lagoonPool, "minecraft:water");
      const flora = nuc.ScatterRule.create(
        nuc.ScatterFace.Top,
        0.32,
        '["minecraft:short_grass","minecraft:poppy","minecraft:dandelion"]',
        [0.65, 0.18, 0.17],
        19,
      );
      flora.setOnBlock("minecraft:grass_block");
      pipe.addScatter(flora);
      return pipe.toSchematicBounded("minecraft_island", ...this.bounds);
    },
  },

  temple: {
    label: "4. Temple",
    p1: { label: "Columns (repeat_polar)", min: 4, max: 10, step: 1, value: 6 },
    p2: { label: "Chamfer bevel (r)", min: 0.4, max: 2.8, step: 0.2, value: 1.4 },
    p3: null,
    bounds: [-13, -9, -13, 13, 12, 13],
    code(lang, p1, p2) {
      const cols = Math.round(p1);
      if (lang === "js") {
        return `const plinth = Field.cappedCylinder(11.2, 1.5).translate(0, -6.5, 0).tag(1);
const col    = Field.cappedCylinder(1.4, 7.0)
  .translate(7.8, 0.5, 0).repeatPolar(${cols}).tag(1);
const ring   = Field.torus(7.8, 1.8).translate(0, 7.5, 0).tag(2);
const altar  = Field.octahedron(3.8).translate(0, 0.5, 0).tag(3);

const geom = plinth
  .chamferUnion(col, ${p2}).smoothUnion(ring, 1.2).unionWith(altar);
// Tag 1 = sandstone, Tag 2 = oxidized/cut copper, Tag 3 = sea_lantern
const schem = VoxelPipeline.create(geom, matcher.build()).toSchematic("temple");`;
      }
      return `plinth = Field.capped_cylinder(11.2, 1.5).translate(0, -6.5, 0).tag(1)
col    = (
    Field.capped_cylinder(1.4, 7.0)
    .translate(7.8, 0.5, 0).repeat_polar(${cols}).tag(1)
)
ring   = Field.torus(7.8, 1.8).translate(0, 7.5, 0).tag(2)
altar  = Field.octahedron(3.8).translate(0, 0.5, 0).tag(3)

geom = plinth.chamfer_union(col, ${p2}).smooth_union(ring, 1.2).union_with(altar)
# Tag 1 = sandstone, Tag 2 = oxidized/cut copper, Tag 3 = sea_lantern
schem = VoxelPipeline.create(geom, matcher.build()).to_schematic("temple")`;
    },
    build(nuc, p1, p2) {
      const plinth = nuc.Field.cappedCylinder(11.2, 1.5).translate(0, -6.5, 0).tag(1);
      const col = nuc.Field.cappedCylinder(1.4, 7.0)
        .translate(7.8, 0.5, 0)
        .repeatPolar(Math.round(p1))
        .tag(1);
      const ring = nuc.Field.torus(7.8, 1.8).translate(0, 7.5, 0).tag(2);
      const altar = nuc.Field.octahedron(3.8).translate(0, 0.5, 0).tag(3);
      const geom = plinth
        .chamferUnion(col, p2)
        .smoothUnion(ring, 1.2)
        .unionWith(altar);

      const m = nuc.MaterialBuilder.createTagMatcher(true, 11);
      m.addTagCase(
        1,
        nuc.Material.weighted(
          '["minecraft:sandstone","minecraft:smooth_sandstone","minecraft:stone_bricks"]',
          [0.50, 0.30, 0.20],
          31,
        ),
      );
      m.addTagCase(
        2,
        nuc.Material.weighted(
          '["minecraft:oxidized_copper","minecraft:weathered_copper","minecraft:cut_copper"]',
          [0.54, 0.28, 0.18],
          53,
        ),
      );
      m.addTagCase(3, nuc.Material.solid("minecraft:sea_lantern"));
      m.setDefault(nuc.Material.solid("minecraft:sandstone"));

      return nuc.VoxelPipeline.create(geom, m.build()).toSchematicBounded("temple_colonnade", ...this.bounds);
    },
  },

  custom: {
    label: "5. Field.expr",
    p1: { label: "Scale", min: 0.7, max: 1.4, step: 0.05, value: 1.0 },
    p2: { label: "Surface offset", min: -0.8, max: 0.8, step: 0.05, value: 0.0 },
    p3: null,
    bounds: [-12, -12, -12, 12, 12, 12],
    code(lang, p1, p2) {
      const mods =
        (Math.abs(p1 - 1.0) > 1e-3 ? `.scale(${p1})` : "") +
        (Math.abs(p2) > 1e-3 ? (lang === "js" ? `.addConst(${p2})` : `.add_const(${p2})`) : "");
      if (lang === "js") {
        return `const b = FieldExprBuilder.create();
b.setBounds(-12, -12, -12, 12, 12, 12);
b.setDistanceKind(FieldDistanceKind.Implicit);
const field = b.compile(source)${mods};
const schem = VoxelPipeline.create(field, oklabCrystalMat).toSchematic("custom_expr");`;
      }
      return `b = FieldExprBuilder.create()
b.set_bounds(-12, -12, -12, 12, 12, 12)
b.set_distance_kind(FieldDistanceKind.Implicit)
field = b.compile(source)${mods}
schem = VoxelPipeline.create(field, oklab_crystal_mat).to_schematic("custom_expr")`;
    },
    build(nuc, p1, p2, _p3, customSource) {
      const b = nuc.FieldExprBuilder.create();
      b.setBounds(-12, -12, -12, 12, 12, 12);
      b.setDistanceKind(nuc.FieldDistanceKind.Implicit);
      let field = b.compile(customSource);
      if (Math.abs(p1 - 1.0) > 1e-3) field = field.scale(p1);
      if (Math.abs(p2) > 1e-3) field = field.addConst(p2);
      const colorField = nuc.Field.expr("clamp((length(p) - 2.0) / 10.0, 0.0, 1.0)");
      const pal = nuc.Palette.fromBlockIds(GRADIENT_PALETTE_IDS).dithered();
      const mat = nuc.Material.fieldGradient(
        colorField,
        [0.0, 0.35, 0.72, 1.0],
        [206, 240, 234, 82, 164, 146, 148, 102, 206, 28, 22, 42],
        0.0,
        1.0,
        pal,
        nuc.InterpolationSpace.Oklab,
      );
      return nuc.VoxelPipeline.create(field, mat).toSchematicBounded("custom_expr", ...this.bounds);
    },
  },
};

async function mountWorkbench(root, ensureModelViewer) {
  if (root.dataset.mounted === "true") return () => {};
  root.dataset.mounted = "true";

  let presetKey = "island";
  let codeLang = "py";
  let p1Val = WORKBENCH_STEPS.island.p1.value;
  let p2Val = WORKBENCH_STEPS.island.p2.value;
  let p3Val = WORKBENCH_STEPS.island.p3 ? WORKBENCH_STEPS.island.p3.value : 0;
  let customExpr = CUSTOM_SNIPPETS[0].expr;
  let currentUrl = null;
  let lastSchematic = null;
  let buildSeq = 0;
  let debounceTimer = null;
  let nuc = null;

  root.innerHTML = `
    <div class="sdf-studio__bar">
      <div class="sdf-seg" role="radiogroup" aria-label="Progressive SDF step" data-role="presets">
        ${Object.entries(WORKBENCH_STEPS)
          .map(
            ([k, p]) =>
              `<button type="button" class="sdf-seg__btn" role="radio" aria-checked="${
                k === presetKey
              }" data-preset="${k}">${p.label}</button>`,
          )
          .join("")}
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
          rotation-per-second="13deg"
          camera-orbit="38deg 64deg 118%"
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
        <div class="sdf-sliders" data-role="sliders">
          <label class="sdf-slider">
            <span class="sdf-slider__head">
              <span data-role="p1-label">${WORKBENCH_STEPS.island.p1.label}</span>
              <output data-role="p1-val">${p1Val}</output>
            </span>
            <input type="range" data-role="p1-input" min="${WORKBENCH_STEPS.island.p1.min}" max="${WORKBENCH_STEPS.island.p1.max}" step="${WORKBENCH_STEPS.island.p1.step}" value="${p1Val}">
          </label>
          <label class="sdf-slider">
            <span class="sdf-slider__head">
              <span data-role="p2-label">${WORKBENCH_STEPS.island.p2.label}</span>
              <output data-role="p2-val">${p2Val}</output>
            </span>
            <input type="range" data-role="p2-input" min="${WORKBENCH_STEPS.island.p2.min}" max="${WORKBENCH_STEPS.island.p2.max}" step="${WORKBENCH_STEPS.island.p2.step}" value="${p2Val}">
          </label>
          <label class="sdf-slider" data-role="p3-wrap" hidden>
            <span class="sdf-slider__head">
              <span data-role="p3-label"></span>
              <output data-role="p3-val">0</output>
            </span>
            <input type="range" data-role="p3-input" min="0" max="5" step="0.5" value="0">
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

      <div class="sdf-code-panel">
        <div class="sdf-code-panel__head">
          <span class="sdf-code-panel__title">Underlying Nucleation code</span>
          <div class="sdf-code-panel__actions">
            <div class="sdf-seg" role="radiogroup" aria-label="Code language" data-role="code-lang">
              <button type="button" class="sdf-seg__btn" role="radio" aria-checked="true" data-lang="py">Python</button>
              <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-lang="js">JavaScript</button>
            </div>
            <button type="button" class="sdf-copy-btn" data-role="code-copy">Copy</button>
          </div>
        </div>
        <pre class="sdf-code-panel__pre"><code data-role="live-code"></code></pre>
      </div>
    </div>

    <div class="sdf-studio__footer">
      <div class="sdf-telemetry" data-role="telemetry">Initializing Nucleation WebAssembly...</div>
      <div class="sdf-footer-actions">
        <label class="sdf-pack-btn" title="Drop any Minecraft resource pack .zip or client.jar to render with its exact block textures (stays 100% local in your browser)">
          <span data-role="pack-label">Load pack .zip</span>
          <input type="file" accept=".zip,.jar" data-role="pack-input" hidden>
        </label>
        <button type="button" class="sdf-download-btn" data-role="download" disabled>Download .schem</button>
      </div>
    </div>
  `;

  const viewer = root.querySelector(".sdf-studio__viewer");
  const statusEl = root.querySelector('[data-role="status"]');
  const liveCodeEl = root.querySelector('[data-role="live-code"]');
  const codeCopyBtn = root.querySelector('[data-role="code-copy"]');
  const telemetryEl = root.querySelector('[data-role="telemetry"]');
  const downloadBtn = root.querySelector('[data-role="download"]');
  const packInput = root.querySelector('[data-role="pack-input"]');
  const packLabel = root.querySelector('[data-role="pack-label"]');
  const slidersEl = root.querySelector('[data-role="sliders"]');
  const p1Label = root.querySelector('[data-role="p1-label"]');
  const p1ValEl = root.querySelector('[data-role="p1-val"]');
  const p1Input = root.querySelector('[data-role="p1-input"]');
  const p2Label = root.querySelector('[data-role="p2-label"]');
  const p2ValEl = root.querySelector('[data-role="p2-val"]');
  const p2Input = root.querySelector('[data-role="p2-input"]');
  const p3Wrap = root.querySelector('[data-role="p3-wrap"]');
  const p3Label = root.querySelector('[data-role="p3-label"]');
  const p3ValEl = root.querySelector('[data-role="p3-val"]');
  const p3Input = root.querySelector('[data-role="p3-input"]');
  const exprPane = root.querySelector('[data-role="expr-pane"]');
  const exprInput = root.querySelector('[data-role="expr-input"]');
  const exprError = root.querySelector('[data-role="expr-error"]');
  const exprRun = root.querySelector('[data-role="expr-run"]');

  function updateCode() {
    liveCodeEl.textContent = WORKBENCH_STEPS[presetKey].code(codeLang, p1Val, p2Val, p3Val);
  }

  function syncSliderConfig() {
    const preset = WORKBENCH_STEPS[presetKey];
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

    if (preset.p3) {
      p3Wrap.hidden = false;
      slidersEl.classList.add("sdf-sliders--3");
      p3Label.textContent = preset.p3.label;
      p3Input.min = String(preset.p3.min);
      p3Input.max = String(preset.p3.max);
      p3Input.step = String(preset.p3.step);
      p3Input.value = String(p3Val);
      p3ValEl.textContent = String(p3Val);
    } else {
      p3Wrap.hidden = true;
      slidersEl.classList.remove("sdf-sliders--3");
    }

    exprPane.hidden = presetKey !== "custom";
    updateCode();
  }

  async function rebuild() {
    const seq = ++buildSeq;
    statusEl.textContent = "Voxelizing in WASM...";
    statusEl.hidden = false;
    exprError.hidden = true;
    updateCode();

    try {
      if (!nuc) {
        await ensureModelViewer();
        nuc = await loadNucleation();
      }
      if (seq !== buildSeq) return;

      const t0 = performance.now();
      const schem = WORKBENCH_STEPS[presetKey].build(nuc, p1Val, p2Val, p3Val, customExpr);
      const tVox = performance.now();

      if (seq !== buildSeq) return;

      const blocks = schem.blockCount();
      if (blocks === 0) {
        lastSchematic = null;
        if (currentUrl) {
          URL.revokeObjectURL(currentUrl);
          currentUrl = null;
        }
        viewer.removeAttribute("src");
        downloadBtn.disabled = true;
        statusEl.textContent =
          "0 blocks inside bounds (d > 0 everywhere) — lower Surface offset or adjust the formula.";
        statusEl.hidden = false;
        telemetryEl.textContent = `0 blocks · 0 triangles · ${Math.round(tVox - t0)} ms voxelize`;
        return;
      }

      const cfg = nuc.MeshConfig.create();
      cfg.setAmbientOcclusion(true);
      cfg.setAoIntensity(0.55);
      cfg.setGreedyMeshing(true);
      const mesh = nuc.MeshResult.create(schem, getActivePack(nuc), cfg);
      const glbBytes = decodeBase64(mesh.glbDataB64());
      const tEnd = performance.now();

      if (seq !== buildSeq) return;

      lastSchematic = schem;
      const nextUrl = URL.createObjectURL(new Blob([glbBytes], { type: "model/gltf-binary" }));
      if (currentUrl) URL.revokeObjectURL(currentUrl);
      currentUrl = nextUrl;
      viewer.src = nextUrl;

      statusEl.hidden = true;
      downloadBtn.disabled = false;
      telemetryEl.textContent =
        `${blocks.toLocaleString()} blocks · ${mesh.triangleCount().toLocaleString()} triangles · ` +
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

  function scheduleRebuild(delay = 85) {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => void rebuild(), delay);
  }

  const onPackUpdated = (name) => {
    packLabel.textContent = `Pack: ${name.slice(0, 18)}`;
    scheduleRebuild(0);
  };
  packListeners.add(onPackUpdated);

  packInput.addEventListener("change", async () => {
    const file = packInput.files?.[0];
    if (!file) return;
    try {
      if (!nuc) nuc = await loadNucleation();
      statusEl.textContent = "Loading resource pack...";
      statusEl.hidden = false;
      await loadCustomPackFile(nuc, file);
    } catch (err) {
      statusEl.hidden = true;
      telemetryEl.textContent = `Pack error: ${err instanceof Error ? err.message : String(err)}`;
    }
  });

  root.querySelector('[data-role="presets"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-preset]");
    if (!btn) return;
    presetKey = btn.dataset.preset;
    for (const el of root.querySelectorAll("[data-preset]")) {
      el.setAttribute("aria-checked", String(el === btn));
    }
    const step = WORKBENCH_STEPS[presetKey];
    p1Val = step.p1.value;
    p2Val = step.p2.value;
    p3Val = step.p3 ? step.p3.value : 0;
    syncSliderConfig();
    scheduleRebuild(0);
  });

  root.querySelector('[data-role="code-lang"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-lang]");
    if (!btn) return;
    codeLang = btn.dataset.lang;
    for (const el of root.querySelectorAll("[data-lang]")) {
      el.setAttribute("aria-checked", String(el === btn));
    }
    updateCode();
  });

  codeCopyBtn.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(liveCodeEl.textContent || "");
      codeCopyBtn.textContent = "Copied";
      setTimeout(() => {
        codeCopyBtn.textContent = "Copy";
      }, 1400);
    } catch {
      codeCopyBtn.textContent = "Copy";
    }
  });

  p1Input.addEventListener("input", () => {
    p1Val = Number(p1Input.value);
    p1ValEl.textContent = String(p1Val);
    updateCode();
    scheduleRebuild(65);
  });

  p2Input.addEventListener("input", () => {
    p2Val = Number(p2Input.value);
    p2ValEl.textContent = String(p2Val);
    updateCode();
    scheduleRebuild(65);
  });

  p3Input.addEventListener("input", () => {
    p3Val = Number(p3Input.value);
    p3ValEl.textContent = String(p3Val);
    updateCode();
    scheduleRebuild(65);
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
    packListeners.delete(onPackUpdated);
    if (currentUrl) URL.revokeObjectURL(currentUrl);
    root.dataset.mounted = "false";
  };
}

/* ============================================================================
 * WIDGET 2: INTERACTIVE 2D SDF, CSG, DUAL NORMALS & 8x8 BRICK-PRUNING LAB
 * Powered directly by Nucleation WASM `Field.evalAt` + `Field.normal` (Dual3)!
 * ============================================================================ */

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
    nuc: null,
    wasmField: null,
    fieldKey: "",
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
      <canvas class="sdf-csg__canvas" width="900" height="400" aria-label="Interactive 2D signed distance field powered by Nucleation WebAssembly"></canvas>
    </div>

    <div class="sdf-code-preview" data-role="csg-code"></div>

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
        <label class="sdf-check"><input type="checkbox" data-toggle="normals" checked><span>Dual3 normals</span></label>
        <label class="sdf-check"><input type="checkbox" data-toggle="bricks" checked><span>8×8 interval bricks</span></label>
      </div>
    </div>

    <div class="sdf-studio__footer">
      <div class="sdf-telemetry" data-role="csg-telemetry">Loading Nucleation WASM Field...</div>
      <div class="sdf-telemetry sdf-telemetry--right" data-role="brick-stats"></div>
    </div>
  `;

  const canvas = root.querySelector(".sdf-csg__canvas");
  const ctx = canvas.getContext("2d");
  const kInput = root.querySelector('[data-role="k-input"]');
  const kVal = root.querySelector('[data-role="k-val"]');
  const kLabel = root.querySelector('[data-role="k-label"]');
  const codeEl = root.querySelector('[data-role="csg-code"]');
  const telemetry = root.querySelector('[data-role="csg-telemetry"]');
  const brickStats = root.querySelector('[data-role="brick-stats"]');

  const WORLD_W = 48;
  const WORLD_H = 24;

  function formatCode() {
    const k = state.k.toFixed(1);
    const tail = `  // a=sphere(${state.ar})@(${state.ax.toFixed(1)},${state.ay.toFixed(1)})  b=box(${state.bhx},${state.bhy})@(${state.bx.toFixed(1)},${state.by.toFixed(1)})`;
    if (state.mode === "union") return `a.unionWith(b)${tail}`;
    if (state.mode === "smooth_union") return `a.smoothUnion(b, ${k})${tail}`;
    if (state.mode === "chamfer") return `a.chamferUnion(b, ${k})${tail}`;
    if (state.mode === "steps") return `a.stepsUnion(b, ${k}, ${state.steps})${tail}`;
    if (state.mode === "smooth_sub") return `a.smoothSubtract(b, ${k})${tail}`;
    return `a.smoothIntersection(b, ${k})${tail}`;
  }

  function ensureWasmField() {
    if (!state.nuc) return null;
    const key = `${state.mode}|${state.k.toFixed(2)}|${state.ax.toFixed(2)}|${state.ay.toFixed(2)}|${state.bx.toFixed(2)}|${state.by.toFixed(2)}`;
    if (state.fieldKey === key && state.wasmField) return state.wasmField;
    const F = state.nuc.Field;
    const a = F.sphere(state.ar).translate(state.ax, state.ay, 0.0);
    const b = F.boxShape(state.bhx, state.bhy, 5.4, state.br).translate(state.bx, state.by, 0.0);
    let out;
    if (state.mode === "union") out = a.unionWith(b);
    else if (state.mode === "smooth_union") out = a.smoothUnion(b, state.k);
    else if (state.mode === "chamfer") out = a.chamferUnion(b, state.k);
    else if (state.mode === "steps") out = a.stepsUnion(b, state.k, state.steps);
    else if (state.mode === "smooth_sub") out = a.smoothSubtract(b, state.k);
    else out = a.smoothIntersection(b, state.k);
    state.wasmField = out;
    state.fieldKey = key;
    return out;
  }

  function evalField(wx, wy) {
    const f = ensureWasmField();
    if (f) return f.evalAt(wx, wy, 0.0);
    return Math.hypot(wx - state.ax, wy - state.ay) - state.ar;
  }

  function evalNormal(wx, wy) {
    const f = ensureWasmField();
    if (f) {
      try {
        const n = f.normal(wx, wy, 0.0, 0.05);
        return [n.x, n.y];
      } catch {
        // fall through to finite difference at exact medial-axis singularity
      }
    }
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
    codeEl.textContent = formatCode();
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

    const wasmReady = ensureWasmField() !== null;
    if (!wasmReady) return;

    // 1. Evaluate the REAL Nucleation WASM Field across a crisp grid
    const step = 6;
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

    // 2. Crisp isolines & d = 0 contour + anchor points for Dual3 normals
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
                contourAnchors.push([(pts[0][0] + pts[1][0]) * 0.5, (pts[0][1] + pts[1][1]) * 0.5]);
              }
            }
          }
        }
        ctx.stroke();
      }
    }

    // 3. 8x8 Conservative Interval Brick Pruning
    let totalBricks = 0;
    let prunedBricks = 0;
    let surfaceBricks = 0;
    const brickSize = 8;
    const halfDiag = Math.hypot(brickSize * 0.5, brickSize * 0.5);
    const lipBoost = state.mode === "steps" ? 1.35 : 1.05;

    for (let by = -WORLD_H / 2; by < WORLD_H / 2; by += brickSize) {
      for (let bx = -WORLD_W / 2; bx < WORLD_W / 2; bx += brickSize) {
        totalBricks += 1;
        const dc = evalField(bx + brickSize * 0.5, by + brickSize * 0.5);
        const bound = halfDiag * lipBoost;
        let kind = "surface";
        if (dc - bound > 0) {
          kind = "pruned";
          prunedBricks += 1;
        } else if (dc + bound <= 0) {
          kind = "solid";
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
      `Nucleation WASM Field · ${prunedBricks}/${totalBricks} bricks pruned (${prunedPct}% skipped)`;

    // 4. Contour-anchored Dual3 autodiff normal vectors from WASM Field.normal()
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
        const ex = px + nx * 15;
        const ey = py - ny * 15;
        ctx.beginPath();
        ctx.moveTo(px, py);
        ctx.lineTo(ex, ey);
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(ex, ey, 2.1, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    // 5. Draggable shape center handles
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
      ctx.beginPath();
      ctx.arc(px, py, Math.max(2, (Math.abs(d) / WORLD_W) * cssW), 0, Math.PI * 2);
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
        `p = (${wx.toFixed(1)}, ${wy.toFixed(1)}, 0) · evalAt = ${d >= 0 ? "+" : ""}${d.toFixed(2)} · ` +
        `normal = (${nx.toFixed(2)}, ${ny.toFixed(2)})`;
    } else {
      telemetry.textContent = "Drag shape handle A or B, or hover anywhere to query Field.evalAt() and Dual3 normal().";
    }
  }

  function pointerToWorld(e) {
    const rect = canvas.getBoundingClientRect();
    return pxToWorld(e.clientX - rect.left, e.clientY - rect.top, rect.width, rect.height);
  }

  canvas.addEventListener("pointerdown", (e) => {
    const [wx, wy] = pointerToWorld(e);
    const da = Math.hypot(wx - state.ax, wy - state.ay);
    const db = Math.hypot(wx - state.bx, wy - state.by);
    if (da < 3.0 || da <= db) state.dragging = da < 4.5 ? "a" : db < 4.5 ? "b" : null;
    else if (db < 4.5) state.dragging = "b";
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
      const near =
        Math.hypot(wx - state.ax, wy - state.ay) < 2.5 ||
        Math.hypot(wx - state.bx, wy - state.by) < 2.5;
      canvas.style.cursor = near ? "grab" : "crosshair";
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

  void loadNucleation().then((nuc) => {
    state.nuc = nuc;
    render();
  });

  return {
    rerender: render,
    destroy: () => {
      ro.disconnect();
      root.dataset.mounted = "false";
    },
  };
}

/* ============================================================================
 * WIDGET 3: INTERACTIVE INFINITE SDF CHUNK-STREAMING WORLD EXPLORER
 * ============================================================================ */

const WORLD_PRESETS = {
  archipelago: {
    label: "Alpine archipelago",
    minY: -8,
    maxY: 22,
    buildGenerator(nuc) {
      const hills = nuc.Field.perlinFbm2(0.038, 42, 4).mulConst(10.0);
      const peaks = nuc.Field.ridgedFbm3(0.025, 99, 3).mulConst(7.0);
      const terrain = nuc.Field.heightmap(1.0, hills.add(peaks));

      const layers = nuc.MaterialBuilder.createLayered();
      layers.addLayer(
        nuc.MaterialCondition.exposedFace(1, true)
          .andWith(nuc.MaterialCondition.yRange(true, 11.0, false, 0.0))
          .andWith(nuc.MaterialCondition.slopeRange(true, 0.0, true, 0.75)),
        nuc.Material.solid("minecraft:snow_block"),
      );
      layers.addLayer(
        nuc.MaterialCondition.exposedFace(1, true)
          .andWith(nuc.MaterialCondition.yRange(true, 4.5, true, 12.0))
          .andWith(nuc.MaterialCondition.slopeRange(true, 0.0, true, 0.72)),
        nuc.Material.solid("minecraft:grass_block"),
      );
      layers.addLayer(
        nuc.MaterialCondition.depthBelowSurface(true, 1.0, true, 2.0)
          .andWith(nuc.MaterialCondition.yRange(true, 3.0, true, 11.0)),
        nuc.Material.solid("minecraft:dirt"),
      );
      layers.setDefault(
        nuc.Material.channelRamp(
          nuc.FieldChannel.WorldY,
          -8.0,
          14.0,
          '["minecraft:deepslate","minecraft:tuff","minecraft:stone","minecraft:andesite"]',
          true,
        ),
      );

      const pipe = nuc.VoxelPipeline.create(terrain, layers.build());
      pipe.addFluidLevel(5.0, "minecraft:water");
      const flora = nuc.ScatterRule.create(
        nuc.ScatterFace.Top,
        0.26,
        '["minecraft:short_grass","minecraft:poppy","minecraft:dandelion"]',
        [0.70, 0.15, 0.15],
        13,
      );
      flora.setOnBlock("minecraft:grass_block");
      pipe.addScatter(flora);

      return nuc.WorldGenerator.pipeline(pipe, this.minY, this.maxY, "docs:alpine-archipelago", "v1");
    },
  },

  karst: {
    label: "3D gyroid caves",
    minY: -10,
    maxY: 18,
    buildGenerator(nuc) {
      const hills = nuc.Field.perlinFbm2(0.036, 17, 3).mulConst(6.5);
      const surface = nuc.Field.heightmap(7.5, hills);
      const caves = nuc.Field.expr(
        "let q = p * 0.20; abs(sin(q.x)*cos(q.y) + sin(q.y)*cos(q.z) + sin(q.z)*cos(q.x)) - 0.36",
      );
      const carved = surface.subtract(caves);

      const layers = nuc.MaterialBuilder.createLayered();
      layers.addLayer(
        nuc.MaterialCondition.exposedFace(1, true)
          .andWith(nuc.MaterialCondition.yRange(true, 4.0, false, 0.0))
          .andWith(nuc.MaterialCondition.slopeRange(true, 0.0, true, 0.62)),
        nuc.Material.solid("minecraft:grass_block"),
      );
      layers.addLayer(
        nuc.MaterialCondition.exposedFace(2, true).andWith(
          nuc.MaterialCondition.chance(0.08, 71),
        ),
        nuc.Material.solid("minecraft:glowstone"),
      );
      layers.setDefault(
        nuc.Material.channelRamp(
          nuc.FieldChannel.WorldY,
          -10.0,
          14.0,
          '["minecraft:obsidian","minecraft:deepslate","minecraft:tuff","minecraft:stone","minecraft:sandstone"]',
          true,
        ),
      );

      const pipe = nuc.VoxelPipeline.create(carved, layers.build());
      pipe.addFluidLevel(-4.0, "minecraft:water");
      return nuc.WorldGenerator.pipeline(pipe, this.minY, this.maxY, "docs:gyroid-karst", "v1");
    },
  },

  spires: {
    label: "Composite + cellular spires",
    minY: -8,
    maxY: 22,
    buildGenerator(nuc) {
      const lowland = nuc.Field.heightmap(
        1.5,
        nuc.Field.perlinFbm2(0.035, 31, 3).mulConst(4.5),
      );
      const groundMat = nuc.MaterialBuilder.createLayered();
      groundMat.addLayer(
        nuc.MaterialCondition.exposedFace(1, true).andWith(
          nuc.MaterialCondition.yRange(true, 1.0, false, 0.0),
        ),
        nuc.Material.solid("minecraft:grass_block"),
      );
      groundMat.addLayer(
        nuc.MaterialCondition.depthBelowSurface(true, 1.0, true, 2.0),
        nuc.Material.solid("minecraft:dirt"),
      );
      groundMat.setDefault(
        nuc.Material.channelRamp(
          nuc.FieldChannel.WorldY,
          -8.0,
          8.0,
          '["minecraft:deepslate","minecraft:tuff","minecraft:stone"]',
          true,
        ),
      );
      const groundPipe = nuc.VoxelPipeline.create(lowland, groundMat.build());
      groundPipe.addFluidLevel(0.0, "minecraft:water");

      const base = nuc.WorldGenerator.pipeline(groundPipe, this.minY, this.maxY, "docs:spires-ground", "v1");

      const spireShape = nuc.Field.roundCone(0, -1, 0, 0, 14, 0, 4.2, 0.6).smoothUnion(
        nuc.Field.octahedron(3.2).translate(0, 14.5, 0),
        1.2,
      );
      const cells = nuc.CellularSdfConfig.create(
        26,
        26,
        0x6a09e667n,
        5.0,
        5.0,
        45.0,
        0.78,
        1.22,
        -1,
        3,
        4,
        5,
        0n,
      );
      const spireLayer = nuc.WorldGenerator.cellularSdf(
        spireShape,
        nuc.Brush.solid("minecraft:amethyst_block"),
        this.minY,
        this.maxY,
        cells,
        "docs:spires-motifs",
        "v1",
      );

      const comp = nuc.WorldGenerator.composite("docs:spires-world", "v1");
      comp.addLayer(base, nuc.GeneratedChunkOverlayMode.Replace);
      comp.addLayer(spireLayer, nuc.GeneratedChunkOverlayMode.Replace);
      return comp;
    },
  },
};

async function mountWorldgenLab(root, ensureModelViewer) {
  if (root.dataset.mounted === "true") return () => {};
  root.dataset.mounted = "true";

  let presetKey = "archipelago";
  let originCx = 0;
  let originCz = 0;
  let selected = new Set();
  let currentUrl = null;
  let lastSchematic = null;
  let runSeq = 0;
  let nuc = null;
  let generator = null;

  function selectAll3x3() {
    selected = new Set();
    for (let dz = -1; dz <= 1; dz += 1) {
      for (let dx = -1; dx <= 1; dx += 1) {
        selected.add(`${originCx + dx},${originCz + dz}`);
      }
    }
  }
  selectAll3x3();

  root.innerHTML = `
    <div class="sdf-studio__bar">
      <div class="sdf-seg" role="radiogroup" aria-label="Infinite world preset" data-role="wg-presets">
        ${Object.entries(WORLD_PRESETS)
          .map(
            ([k, p]) =>
              `<button type="button" class="sdf-seg__btn" role="radio" aria-checked="${
                k === presetKey
              }" data-wg-preset="${k}">${p.label}</button>`,
          )
          .join("")}
      </div>
      <button type="button" class="sdf-run-btn" data-role="wg-wave">Stream 3×3 wave</button>
    </div>

    <div class="wg-layout">
      <div class="wg-sidebar">
        <div class="wg-sidebar__head">
          <span>Chunk window (16×16)</span>
          <span data-role="wg-origin">(0, 0)</span>
        </div>

        <div class="wg-grid" data-role="wg-grid" aria-label="Selectable 3x3 chunk window"></div>

        <div class="wg-pan" role="group" aria-label="Pan chunk window">
          <button type="button" class="wg-pan__btn" data-pan="0,-1">North (−Z)</button>
          <button type="button" class="wg-pan__btn" data-pan="reset">Origin (0,0)</button>
          <button type="button" class="wg-pan__btn" data-pan="0,1">South (+Z)</button>
          <button type="button" class="wg-pan__btn" data-pan="-1,0">West (−X)</button>
          <button type="button" class="wg-pan__btn" data-pan="neg">Jump (−12, 8)</button>
          <button type="button" class="wg-pan__btn" data-pan="1,0">East (+X)</button>
        </div>

        <p class="wg-note">Click any chunk to toggle it, or pan anywhere in the infinite coordinate space. Every chunk is evaluated independently by <code>WorldGenerator</code> in Rust/WASM and stitched with zero seams.</p>
      </div>

      <div class="wg-stage">
        <model-viewer
          class="sdf-studio__viewer"
          alt="Interactive 3D multi-chunk Minecraft terrain streamed in the browser by Nucleation WorldGenerator"
          camera-controls
          auto-rotate
          auto-rotate-delay="2200"
          rotation-per-second="10deg"
          camera-orbit="42deg 58deg 128%"
          field-of-view="30deg"
          min-camera-orbit="auto auto 68%"
          max-camera-orbit="auto auto 240%"
          shadow-intensity="0.7"
          shadow-softness="0.85"
          exposure="1.05"
          interaction-prompt="none"
        ></model-viewer>
        <div class="sdf-studio__overlay" data-role="wg-status">Loading WorldGenerator WASM...</div>
      </div>
    </div>

    <div class="sdf-studio__footer">
      <div class="sdf-telemetry" data-role="wg-telemetry">Initializing infinite WorldGenerator...</div>
      <div class="sdf-footer-actions">
        <label class="sdf-pack-btn" title="Drop any Minecraft resource pack .zip or client.jar to render with its exact block textures">
          <span data-role="wg-pack-label">Load pack .zip</span>
          <input type="file" accept=".zip,.jar" data-role="wg-pack-input" hidden>
        </label>
        <button type="button" class="sdf-download-btn" data-role="wg-download" disabled>Download .schem</button>
      </div>
    </div>
  `;

  const gridEl = root.querySelector('[data-role="wg-grid"]');
  const originEl = root.querySelector('[data-role="wg-origin"]');
  const viewer = root.querySelector(".sdf-studio__viewer");
  const statusEl = root.querySelector('[data-role="wg-status"]');
  const telemetryEl = root.querySelector('[data-role="wg-telemetry"]');
  const downloadBtn = root.querySelector('[data-role="wg-download"]');
  const packInput = root.querySelector('[data-role="wg-pack-input"]');
  const packLabel = root.querySelector('[data-role="wg-pack-label"]');
  const waveBtn = root.querySelector('[data-role="wg-wave"]');

  function renderGrid(streamingKey = null) {
    originEl.textContent = `(${originCx}, ${originCz})`;
    const cells = [];
    for (let dz = -1; dz <= 1; dz += 1) {
      for (let dx = -1; dx <= 1; dx += 1) {
        const cx = originCx + dx;
        const cz = originCz + dz;
        const key = `${cx},${cz}`;
        cells.push(
          `<button type="button" class="wg-cell" data-cx="${cx}" data-cz="${cz}" data-active="${selected.has(key)}" data-streaming="${streamingKey === key}">${cx}, ${cz}</button>`,
        );
      }
    }
    gridEl.innerHTML = cells.join("");
  }

  async function ensureGenerator() {
    if (!nuc) {
      await ensureModelViewer();
      nuc = await loadNucleation();
    }
    if (!generator) generator = WORLD_PRESETS[presetKey].buildGenerator(nuc);
    return generator;
  }

  async function rebuildPatch(animateWave = false) {
    const seq = ++runSeq;
    statusEl.textContent = animateWave ? "Streaming chunks..." : "Generating chunks...";
    statusEl.hidden = false;
    waveBtn.disabled = true;

    try {
      const gen = await ensureGenerator();
      if (seq !== runSeq) return;

      const coords = [];
      for (let dz = -1; dz <= 1; dz += 1) {
        for (let dx = -1; dx <= 1; dx += 1) {
          const cx = originCx + dx;
          const cz = originCz + dz;
          if (selected.has(`${cx},${cz}`)) coords.push([cx, cz]);
        }
      }
      if (coords.length === 0) {
        statusEl.textContent = "Select at least one chunk on the left.";
        waveBtn.disabled = false;
        return;
      }

      const combined = nuc.Schematic.create(`worldgen_${presetKey}`);
      let genMsTotal = 0;

      for (const [cx, cz] of coords) {
        if (seq !== runSeq) return;
        if (animateWave) {
          renderGrid(`${cx},${cz}`);
          await new Promise((r) => setTimeout(r, 45));
        }
        const tc0 = performance.now();
        const chunk = gen.generate(cx, cz);
        const view = chunk.takeView();
        const chunkSchem = view.toSchematic();
        genMsTotal += performance.now() - tc0;

        if (chunkSchem.blockCount() > 0) {
          const lo = chunkSchem.tightBoundsMin();
          const hi = chunkSchem.tightBoundsMax();
          combined.stampBox(
            chunkSchem,
            lo.x,
            lo.y,
            lo.z,
            hi.x,
            hi.y,
            hi.z,
            lo.x,
            lo.y,
            lo.z,
            '["minecraft:air"]',
          );
        }
      }

      renderGrid(null);
      if (seq !== runSeq) return;

      const totalBlocks = combined.blockCount();
      if (totalBlocks === 0) {
        lastSchematic = null;
        if (currentUrl) {
          URL.revokeObjectURL(currentUrl);
          currentUrl = null;
        }
        viewer.removeAttribute("src");
        statusEl.textContent = "0 solid blocks in selected chunks — pan to another region.";
        statusEl.hidden = false;
        downloadBtn.disabled = true;
        waveBtn.disabled = false;
        telemetryEl.textContent =
          `${coords.length} chunk${coords.length === 1 ? "" : "s"} · 0 blocks · ${(genMsTotal / coords.length).toFixed(1)} ms/chunk`;
        return;
      }

      const tMesh0 = performance.now();
      const cfg = nuc.MeshConfig.create();
      cfg.setAmbientOcclusion(true);
      cfg.setAoIntensity(0.5);
      cfg.setGreedyMeshing(true);
      const mesh = nuc.MeshResult.create(combined, getActivePack(nuc), cfg);
      const glbBytes = decodeBase64(mesh.glbDataB64());
      const meshMs = performance.now() - tMesh0;

      if (seq !== runSeq) return;

      const nextUrl = URL.createObjectURL(new Blob([glbBytes], { type: "model/gltf-binary" }));
      if (currentUrl) URL.revokeObjectURL(currentUrl);
      currentUrl = nextUrl;
      viewer.src = nextUrl;

      lastSchematic = combined;
      statusEl.hidden = true;
      downloadBtn.disabled = false;
      waveBtn.disabled = false;
      telemetryEl.textContent =
        `${coords.length} chunk${coords.length === 1 ? "" : "s"} · ` +
        `${combined.blockCount().toLocaleString()} blocks · ` +
        `${mesh.triangleCount().toLocaleString()} tris · ` +
        `${(genMsTotal / coords.length).toFixed(1)} ms/chunk · ${Math.round(meshMs)} ms mesh`;
    } catch (err) {
      if (seq !== runSeq) return;
      statusEl.hidden = true;
      waveBtn.disabled = false;
      telemetryEl.textContent = `WorldGenerator error: ${err instanceof Error ? err.message : String(err)}`;
    }
  }

  const onPackUpdated = (name) => {
    packLabel.textContent = `Pack: ${name.slice(0, 18)}`;
    void rebuildPatch(false);
  };
  packListeners.add(onPackUpdated);

  packInput.addEventListener("change", async () => {
    const file = packInput.files?.[0];
    if (!file) return;
    try {
      if (!nuc) nuc = await loadNucleation();
      statusEl.textContent = "Loading resource pack...";
      statusEl.hidden = false;
      await loadCustomPackFile(nuc, file);
    } catch (err) {
      statusEl.hidden = true;
      telemetryEl.textContent = `Pack error: ${err instanceof Error ? err.message : String(err)}`;
    }
  });

  root.querySelector('[data-role="wg-presets"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-wg-preset]");
    if (!btn) return;
    presetKey = btn.dataset.wgPreset;
    generator = null;
    for (const el of root.querySelectorAll("[data-wg-preset]")) {
      el.setAttribute("aria-checked", String(el === btn));
    }
    void rebuildPatch(false);
  });

  gridEl.addEventListener("click", (e) => {
    const cell = e.target.closest(".wg-cell");
    if (!cell) return;
    const key = `${cell.dataset.cx},${cell.dataset.cz}`;
    if (selected.has(key)) {
      if (selected.size > 1) selected.delete(key);
    } else {
      selected.add(key);
    }
    renderGrid();
    void rebuildPatch(false);
  });

  root.querySelector(".wg-pan").addEventListener("click", (e) => {
    const btn = e.target.closest("[data-pan]");
    if (!btn) return;
    const action = btn.dataset.pan;
    if (action === "reset") {
      originCx = 0;
      originCz = 0;
    } else if (action === "neg") {
      originCx = -12;
      originCz = 8;
    } else {
      const [dx, dz] = action.split(",").map(Number);
      originCx += dx;
      originCz += dz;
    }
    selectAll3x3();
    renderGrid();
    void rebuildPatch(false);
  });

  waveBtn.addEventListener("click", () => {
    selectAll3x3();
    renderGrid();
    void rebuildPatch(true);
  });

  downloadBtn.addEventListener("click", () => {
    if (!lastSchematic) return;
    const bytes = decodeBase64(lastSchematic.toSchematicB64());
    const blobUrl = URL.createObjectURL(new Blob([bytes], { type: "application/octet-stream" }));
    const a = document.createElement("a");
    a.href = blobUrl;
    a.download = `nucleation-world-${presetKey}-${originCx}_${originCz}.schem`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(blobUrl), 2000);
  });

  renderGrid();
  void rebuildPatch(false);

  return () => {
    packListeners.delete(onPackUpdated);
    if (currentUrl) URL.revokeObjectURL(currentUrl);
    root.dataset.mounted = "false";
  };
}

/* ============================================================================
 * WIDGET 4: INTERACTIVE OKLAB PALETTE & DITHERING STUDIO (palettes-and-color)
 * ============================================================================ */

function hexToRgb(hex) {
  const v = hex.replace("#", "");
  return [
    parseInt(v.slice(0, 2), 16) || 0,
    parseInt(v.slice(2, 4), 16) || 0,
    parseInt(v.slice(4, 6), 16) || 0,
  ];
}

const PALETTE_PRESETS = [
  { label: "Sunset", a: "#2B1E70", b: "#F6BE38" },
  { label: "Ocean", a: "#183B8C", b: "#58E0D4" },
  { label: "Nether", a: "#23142C", b: "#F47820" },
  { label: "Forest", a: "#1E4A2C", b: "#B6E244" },
  { label: "Monochrome", a: "#181A20", b: "#EEF2F6" },
];

async function mountPaletteStudio(root, ensureModelViewer) {
  if (root.dataset.mounted === "true") return () => {};
  root.dataset.mounted = "true";

  let colorA = PALETTE_PRESETS[0].a;
  let colorB = PALETTE_PRESETS[0].b;
  let family = "concrete";
  let space = "Oklab";
  let dither = true;
  let currentUrl = null;
  let lastSchematic = null;
  let seq = 0;
  let nuc = null;

  root.innerHTML = `
    <div class="sdf-studio__bar">
      <div class="sdf-seg" role="radiogroup" aria-label="Palette family" data-role="pal-family">
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="true" data-family="concrete">Concrete</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-family="wool">Wool</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-family="terracotta">Terracotta</button>
      </div>
      <div class="sdf-seg" role="radiogroup" aria-label="Interpolation color space" data-role="pal-space">
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="true" data-space="Oklab">Oklab</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-space="Lab">CIE Lab</button>
        <button type="button" class="sdf-seg__btn" role="radio" aria-checked="false" data-space="Rgb">sRGB</button>
      </div>
    </div>

    <div class="sdf-studio__stage">
      <model-viewer
        class="sdf-studio__viewer"
        alt="Interactive 3D Minecraft block gradient wall generated by Nucleation Oklab palette mapping"
        camera-controls
        camera-orbit="12deg 78deg 112%"
        field-of-view="26deg"
        shadow-intensity="0.6"
        exposure="1.05"
        interaction-prompt="none"
      ></model-viewer>
      <div class="sdf-studio__overlay" data-role="pal-status">Loading Nucleation WASM...</div>
    </div>

    <div class="pal-controls">
      <div class="pal-pickers">
        <label class="pal-color">Start <input type="color" data-role="pal-a" value="${colorA}"></label>
        <label class="pal-color">End <input type="color" data-role="pal-b" value="${colorB}"></label>
        <label class="sdf-check"><input type="checkbox" data-role="pal-dither" checked><span>Ordered Bayer dithering</span></label>
      </div>
      <div class="sdf-expr-presets" data-role="pal-presets">
        ${PALETTE_PRESETS.map(
          (p, i) => `<button type="button" class="sdf-expr-snippet" data-pal-preset="${i}">${p.label}</button>`,
        ).join("")}
      </div>
    </div>

    <div class="pal-blocks" data-role="pal-blocks"></div>

    <div class="sdf-studio__footer">
      <div class="sdf-telemetry" data-role="pal-telemetry">Initializing Oklab palette engine...</div>
      <button type="button" class="sdf-download-btn" data-role="pal-download" disabled>Download .schem</button>
    </div>
  `;

  const viewer = root.querySelector(".sdf-studio__viewer");
  const statusEl = root.querySelector('[data-role="pal-status"]');
  const inputA = root.querySelector('[data-role="pal-a"]');
  const inputB = root.querySelector('[data-role="pal-b"]');
  const ditherEl = root.querySelector('[data-role="pal-dither"]');
  const blocksEl = root.querySelector('[data-role="pal-blocks"]');
  const telemetryEl = root.querySelector('[data-role="pal-telemetry"]');
  const downloadBtn = root.querySelector('[data-role="pal-download"]');

  async function rebuild() {
    const mySeq = ++seq;
    statusEl.textContent = "Mapping gradient in WASM...";
    statusEl.hidden = false;
    try {
      if (!nuc) {
        await ensureModelViewer();
        nuc = await loadNucleation();
      }
      if (mySeq !== seq) return;

      const t0 = performance.now();
      let pal =
        family === "wool"
          ? nuc.Palette.wool()
          : family === "terracotta"
            ? nuc.Palette.terracotta()
            : nuc.Palette.concrete();
      if (dither) pal = pal.dithered();

      const rgbA = hexToRgb(colorA);
      const rgbB = hexToRgb(colorB);
      const interp =
        space === "Lab"
          ? nuc.InterpolationSpace.Lab
          : space === "Rgb"
            ? nuc.InterpolationSpace.Rgb
            : nuc.InterpolationSpace.Oklab;

      const wall = nuc.Field.boxShape(12.0, 5.0, 0.9, 0.0);
      const mat = nuc.Material.channelGradient(
        nuc.FieldChannel.WorldX,
        [0.0, 1.0],
        [...rgbA, ...rgbB],
        -11.5,
        11.5,
        pal,
        interp,
      );
      const schem = nuc.VoxelPipeline.create(wall, mat).toSchematicBounded(
        "palette_gradient",
        -12,
        -5,
        -1,
        12,
        5,
        1,
      );

      const cfg = nuc.MeshConfig.create();
      cfg.setAmbientOcclusion(true);
      cfg.setAoIntensity(0.45);
      cfg.setGreedyMeshing(true);
      const mesh = nuc.MeshResult.create(schem, getActivePack(nuc), cfg);
      const glbBytes = decodeBase64(mesh.glbDataB64());
      const ms = Math.round(performance.now() - t0);
      if (mySeq !== seq) return;

      lastSchematic = schem;
      const nextUrl = URL.createObjectURL(new Blob([glbBytes], { type: "model/gltf-binary" }));
      if (currentUrl) URL.revokeObjectURL(currentUrl);
      currentUrl = nextUrl;
      viewer.src = nextUrl;

      const paletteIds = JSON.parse(schem.countBlocksJson());
      const names = Object.entries(paletteIds)
        .sort((a, b) => b[1] - a[1])
        .map(([id, n]) => `${id.replace("minecraft:", "")} (${n})`);
      blocksEl.textContent = `Selected blocks (${names.length}): ${names.join(" · ")}`;
      telemetryEl.textContent = `${schem.blockCount()} blocks · ${space} · ${dither ? "Bayer dithered" : "snapped"} · ${ms} ms`;
      statusEl.hidden = true;
      downloadBtn.disabled = false;
    } catch (err) {
      if (mySeq !== seq) return;
      statusEl.hidden = true;
      telemetryEl.textContent = `Palette error: ${err instanceof Error ? err.message : String(err)}`;
    }
  }

  root.querySelector('[data-role="pal-family"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-family]");
    if (!btn) return;
    family = btn.dataset.family;
    for (const el of root.querySelectorAll("[data-family]")) {
      el.setAttribute("aria-checked", String(el === btn));
    }
    void rebuild();
  });

  root.querySelector('[data-role="pal-space"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-space]");
    if (!btn) return;
    space = btn.dataset.space;
    for (const el of root.querySelectorAll("[data-space]")) {
      el.setAttribute("aria-checked", String(el === btn));
    }
    void rebuild();
  });

  root.querySelector('[data-role="pal-presets"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-pal-preset]");
    if (!btn) return;
    const preset = PALETTE_PRESETS[Number(btn.dataset.palPreset)];
    colorA = preset.a;
    colorB = preset.b;
    inputA.value = colorA;
    inputB.value = colorB;
    void rebuild();
  });

  inputA.addEventListener("input", () => {
    colorA = inputA.value;
    void rebuild();
  });
  inputB.addEventListener("input", () => {
    colorB = inputB.value;
    void rebuild();
  });
  ditherEl.addEventListener("change", () => {
    dither = ditherEl.checked;
    void rebuild();
  });
  downloadBtn.addEventListener("click", () => {
    if (!lastSchematic) return;
    const bytes = decodeBase64(lastSchematic.toSchematicB64());
    const blobUrl = URL.createObjectURL(new Blob([bytes], { type: "application/octet-stream" }));
    const a = document.createElement("a");
    a.href = blobUrl;
    a.download = `nucleation-gradient-${family}-${space.toLowerCase()}.schem`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(blobUrl), 2000);
  });

  void rebuild();
  return () => {
    if (currentUrl) URL.revokeObjectURL(currentUrl);
    root.dataset.mounted = "false";
  };
}

/* ============================================================================
 * WIDGET 5: FULL INTERACTIVE 3D MC-TICK + NUCLEATION MESHER LAB
 * ============================================================================ */

const TICK_CIRCUITS = {
  delay: {
    label: "1. Repeater delay",
    inputPos: [0, 1, 1],
    actionLabel: "Toggle lever",
    build(nuc) {
      const s = nuc.Schematic.create("repeater_delay");
      for (let x = 0; x <= 7; x += 1) {
        for (let z = 0; z <= 2; z += 1) {
          s.setBlock(x, 0, z, "minecraft:smooth_stone");
        }
      }
      s.setBlockFromString(0, 1, 1, "minecraft:lever[face=floor,facing=east,powered=false]");
      s.setBlockFromString(1, 1, 1, "minecraft:redstone_wire[east=side,west=side,north=none,south=none,power=0]");
      s.setBlockFromString(2, 1, 1, "minecraft:repeater[facing=west,delay=2,locked=false,powered=false]");
      s.setBlockFromString(3, 1, 1, "minecraft:redstone_wire[east=side,west=side,north=none,south=none,power=0]");
      s.setBlockFromString(4, 1, 1, "minecraft:repeater[facing=west,delay=2,locked=false,powered=false]");
      s.setBlockFromString(5, 1, 1, "minecraft:redstone_wire[east=side,west=side,north=none,south=none,power=0]");
      s.setBlockFromString(6, 1, 1, "minecraft:repeater[facing=west,delay=2,locked=false,powered=false]");
      s.setBlockFromString(7, 1, 1, "minecraft:redstone_lamp[lit=false]");
      return s;
    },
  },
  inverter: {
    label: "2. Torch inverter",
    inputPos: [0, 1, 1],
    actionLabel: "Toggle lever",
    build(nuc) {
      const s = nuc.Schematic.create("torch_inverter");
      for (let x = 0; x <= 5; x += 1) {
        for (let z = 0; z <= 2; z += 1) {
          s.setBlock(x, 0, z, "minecraft:smooth_stone");
        }
      }
      s.setBlockFromString(0, 1, 1, "minecraft:lever[face=floor,facing=east,powered=false]");
      s.setBlockFromString(1, 1, 1, "minecraft:redstone_wire[east=side,west=side,north=none,south=none,power=0]");
      s.setBlock(2, 1, 1, "minecraft:stone_bricks");
      s.setBlockFromString(3, 1, 1, "minecraft:redstone_wall_torch[facing=east,lit=true]");
      s.setBlockFromString(4, 1, 1, "minecraft:redstone_wire[east=side,west=side,north=none,south=none,power=15]");
      s.setBlockFromString(5, 1, 1, "minecraft:redstone_lamp[lit=true]");
      return s;
    },
  },
  button: {
    label: "3. Sticky piston",
    inputPos: [0, 1, 1],
    actionLabel: "Press button",
    build(nuc) {
      const s = nuc.Schematic.create("sticky_piston_demo");
      for (let x = 0; x <= 6; x += 1) {
        for (let z = 0; z <= 2; z += 1) {
          s.setBlock(x, 0, z, "minecraft:smooth_stone");
        }
      }
      s.setBlockFromString(0, 1, 1, "minecraft:stone_button[face=floor,facing=east,powered=false]");
      s.setBlockFromString(1, 1, 1, "minecraft:redstone_wire[east=side,west=side,north=none,south=none,power=0]");
      s.setBlockFromString(2, 1, 1, "minecraft:redstone_wire[east=side,west=side,north=none,south=none,power=0]");
      s.setBlockFromString(3, 1, 1, "minecraft:sticky_piston[facing=east,extended=false]");
      s.setBlock(4, 1, 1, "minecraft:gold_block");
      return s;
    },
  },
  sandbox: {
    label: "4. Observer piston + sandbox",
    inputPos: [1, 1, 3],
    actionLabel: "Toggle lever",
    build(nuc) {
      const s = nuc.Schematic.create("redstone_sandbox");
      for (let x = 0; x <= 7; x += 1) {
        for (let z = 0; z <= 5; z += 1) {
          s.setBlock(x, 0, z, "minecraft:smooth_stone");
        }
      }
      // Obsidian glide rail under slime block so piston slides cleanly without dragging the floor!
      s.setBlock(5, 0, 3, "minecraft:obsidian");
      s.setBlock(6, 0, 3, "minecraft:obsidian");
      s.setBlockFromString(1, 1, 3, "minecraft:lever[face=floor,facing=east,powered=false]");
      s.setBlockFromString(2, 1, 3, "minecraft:observer[facing=west,powered=false]");
      s.setBlockFromString(3, 1, 3, "minecraft:redstone_wire[east=side,west=side,north=none,south=none,power=0]");
      s.setBlockFromString(4, 1, 3, "minecraft:sticky_piston[facing=east,extended=false]");
      s.setBlock(5, 1, 3, "minecraft:slime_block");
      s.setBlock(5, 2, 3, "minecraft:gold_block");
      s.setBlockFromString(3, 1, 1, "minecraft:redstone_lamp[lit=false]");
      return s;
    },
  },
};

const OPPOSITE_FACING = {
  north: "south",
  south: "north",
  east: "west",
  west: "east",
};

function faceToCardinal(face, fallback) {
  if (face[0] === 1) return "east";
  if (face[0] === -1) return "west";
  if (face[2] === 1) return "south";
  if (face[2] === -1) return "north";
  return fallback;
}

const TICK_TOOLS = [
  { id: "interact", label: "Interact", swatch: "#5fd38d", title: "Click levers, buttons, or repeaters in 3D" },
  {
    id: "dust",
    label: "Dust",
    swatch: "#e02b20",
    state: () => "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]",
  },
  {
    id: "repeater",
    label: "Repeater",
    swatch: "#b2b6bc",
    // Repeater's `facing` points opposite the signal travel direction (vanilla convention)
    state: (_face, facing) =>
      `minecraft:repeater[delay=1,facing=${OPPOSITE_FACING[facing] || "west"},locked=false,powered=false]`,
  },
  {
    id: "torch",
    label: "Torch",
    swatch: "#ff5533",
    state: (face) => {
      if (face[1] === 0 && (face[0] !== 0 || face[2] !== 0)) {
        return `minecraft:redstone_wall_torch[facing=${faceToCardinal(face, "east")},lit=true]`;
      }
      return "minecraft:redstone_torch[lit=true]";
    },
  },
  {
    id: "lever",
    label: "Lever",
    swatch: "#9c7548",
    state: (face, facing) => {
      if (face[1] === 0 && (face[0] !== 0 || face[2] !== 0)) {
        return `minecraft:lever[face=wall,facing=${faceToCardinal(face, facing)},powered=false]`;
      }
      return `minecraft:lever[face=floor,facing=${facing},powered=false]`;
    },
  },
  {
    id: "button",
    label: "Button",
    swatch: "#8e9296",
    state: (face, facing) => {
      if (face[1] === 0 && (face[0] !== 0 || face[2] !== 0)) {
        return `minecraft:stone_button[face=wall,facing=${faceToCardinal(face, facing)},powered=false]`;
      }
      return `minecraft:stone_button[face=floor,facing=${facing},powered=false]`;
    },
  },
  {
    id: "piston",
    label: "Sticky piston",
    swatch: "#78c858",
    state: (face, facing) => {
      let dir = OPPOSITE_FACING[facing] || "east";
      if (face[1] === 1) dir = facing;
      return `minecraft:sticky_piston[extended=false,facing=${dir}]`;
    },
  },
  {
    id: "observer",
    label: "Observer",
    swatch: "#6a6e76",
    state: (_face, facing) => `minecraft:observer[facing=${facing},powered=false]`,
  },
  {
    id: "rsblock",
    label: "Redstone block",
    swatch: "#c81e16",
    state: () => "minecraft:redstone_block",
  },
  {
    id: "lamp",
    label: "Lamp",
    swatch: "#d69240",
    state: () => "minecraft:redstone_lamp[lit=false]",
  },
  {
    id: "slime",
    label: "Slime",
    swatch: "#70c45c",
    state: () => "minecraft:slime_block",
  },
  {
    id: "stone",
    label: "Stone",
    swatch: "#a8acb2",
    state: () => "minecraft:smooth_stone",
  },
  { id: "break", label: "Break", swatch: "#e05252", danger: true, title: "Click any block to mine it (or Shift+Click)" },
];

const TICK_EXTRA_STATES = (() => {
  const dirs4 = ["north", "south", "east", "west"];
  const dirs6 = ["north", "south", "east", "west", "up", "down"];
  const states = [
    "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]",
    "minecraft:redstone_torch[lit=true]",
    "minecraft:redstone_block",
    "minecraft:redstone_lamp[lit=false]",
    "minecraft:slime_block",
    "minecraft:smooth_stone",
    "minecraft:gold_block",
    "minecraft:obsidian",
  ];
  for (const d of dirs4) {
    states.push(`minecraft:repeater[delay=1,facing=${d},locked=false,powered=false]`);
    states.push(`minecraft:redstone_wall_torch[facing=${d},lit=true]`);
    states.push(`minecraft:lever[face=floor,facing=${d},powered=false]`);
    states.push(`minecraft:lever[face=wall,facing=${d},powered=false]`);
    states.push(`minecraft:stone_button[face=floor,facing=${d},powered=false]`);
    states.push(`minecraft:stone_button[face=wall,facing=${d},powered=false]`);
  }
  for (const d of dirs6) {
    states.push(`minecraft:sticky_piston[extended=false,facing=${d}]`);
    states.push(`minecraft:observer[facing=${d},powered=false]`);
  }
  return states.join(";");
})();

async function mountTickStudio(root) {
  if (root.dataset.mounted === "true") return { destroy: () => {} };
  root.dataset.mounted = "true";

  let circuitKey = "delay";
  let activeTool = "interact";
  let nuc = null;
  let sim = null;
  let liveSchem = null;
  let viewport = null;
  let customFileBytes = null;
  let customFileName = null;
  let playing = false;
  let tps = 10;
  let tickCarry = 0;
  let remeshing = false;
  let needsRemesh = false;
  let refitNext = false;
  const solidSet = new Set();
  const vKey = (x, y, z) => `${x},${y},${z}`;

  root.innerHTML = `
    <div class="sdf-studio__bar">
      <div class="sdf-seg" role="radiogroup" aria-label="Redstone circuit preset" data-role="tick-presets">
        ${Object.entries(TICK_CIRCUITS)
          .map(
            ([k, c]) =>
              `<button type="button" class="sdf-seg__btn" role="radio" aria-checked="${
                k === circuitKey
              }" data-circuit="${k}">${c.label}</button>`,
          )
          .join("")}
      </div>
    </div>

    <div class="tick-stage" data-role="tick-stage">
      <div class="tick-hud">
        <div class="tick-hud__pill" data-role="tick-hover">Hover a block · Left-click interacts · Drag orbits · Shift+Click breaks</div>
        <div class="tick-hud__pill tick-hud__pill--muted" data-role="tick-clock">tick 0 · paused</div>
      </div>
      <div class="tick-hud__bottom">
        <span class="tick-hud__pill tick-hud__pill--muted" data-role="tick-hint">Drop .schem / .litematic / .nbt here</span>
        <button type="button" class="sdf-pack-btn" data-role="tick-cam">Reset view</button>
      </div>
      <div class="sdf-studio__overlay" data-role="tick-status">Loading Nucleation WASM mesher + Three.js...</div>
    </div>

    <div class="tick-toolbar" role="radiogroup" aria-label="3D build & interact tool" data-role="tick-tools">
      <span class="tick-toolbar__label">Tool</span>
      ${TICK_TOOLS.map(
        (t) =>
          `<button type="button" class="tick-tool${t.danger ? " tick-tool--danger" : ""}" role="radio" aria-checked="${
            t.id === activeTool
          }" data-tool="${t.id}" title="${t.title || `Place ${t.label} on clicked block face`}"><span class="tick-tool__swatch" style="background:${t.swatch}"></span>${t.label}</button>`,
      ).join("")}
    </div>

    <div class="tick-actions">
      <div class="tick-actions__group">
        <button type="button" class="sdf-run-btn" data-role="tick-play">Play</button>
        <label class="tick-tps">
          <span>Speed</span>
          <select data-role="tick-tps" aria-label="Simulation ticks per second">
            <option value="2">2 TPS</option>
            <option value="5">5 TPS</option>
            <option value="10" selected>10 TPS</option>
            <option value="20">20 TPS</option>
          </select>
        </label>
        <button type="button" class="sdf-pack-btn" data-role="tick-use">${TICK_CIRCUITS[circuitKey].actionLabel}</button>
        <button type="button" class="sdf-pack-btn" data-role="tick-step1">+1 tick</button>
        <button type="button" class="sdf-pack-btn" data-role="tick-step2">+2 ticks</button>
        <button type="button" class="sdf-pack-btn" data-role="tick-settle">Settle</button>
        <button type="button" class="sdf-pack-btn" data-role="tick-reset">Reset</button>
      </div>
      <div class="tick-actions__group">
        <label class="sdf-pack-btn" title="Open any .schem, .litematic, .schematic, or .nbt file">
          <span>Open schematic</span>
          <input type="file" accept=".schem,.litematic,.schematic,.nbt,.snbt,.mcstructure" data-role="tick-schem-input" hidden>
        </label>
        <button type="button" class="sdf-pack-btn" data-role="tick-export">Download .schem</button>
      </div>
    </div>

    <pre class="tick-log" data-role="tick-log">Click directly on the lever in 3D (or click Toggle lever), then press Play or +1 tick.</pre>

    <div class="sdf-studio__footer">
      <div class="sdf-telemetry" data-role="tick-telemetry">Initializing Nucleation WASM mesher & mc-tick...</div>
      <div class="sdf-footer-actions">
        <label class="sdf-pack-btn" title="Drop any Minecraft resource pack .zip or client.jar to render with vanilla textures">
          <span data-role="tick-pack-label">Load pack .zip</span>
          <input type="file" accept=".zip,.jar" data-role="tick-pack-input" hidden>
        </label>
      </div>
    </div>
  `;

  const stageEl = root.querySelector('[data-role="tick-stage"]');
  const statusEl = root.querySelector('[data-role="tick-status"]');
  const hoverEl = root.querySelector('[data-role="tick-hover"]');
  const clockEl = root.querySelector('[data-role="tick-clock"]');
  const hintEl = root.querySelector('[data-role="tick-hint"]');
  const playBtn = root.querySelector('[data-role="tick-play"]');
  const tpsSel = root.querySelector('[data-role="tick-tps"]');
  const useBtn = root.querySelector('[data-role="tick-use"]');
  const logEl = root.querySelector('[data-role="tick-log"]');
  const telemetryEl = root.querySelector('[data-role="tick-telemetry"]');
  const packInput = root.querySelector('[data-role="tick-pack-input"]');
  const packLabel = root.querySelector('[data-role="tick-pack-label"]');
  const schemInput = root.querySelector('[data-role="tick-schem-input"]');

  function syncSolidAndSchemFromSim() {
    if (!sim || !nuc) return;
    const snap = JSON.parse(sim.worldSnapshotJson());
    solidSet.clear();
    const fresh = nuc.Schematic.create(customFileName || circuitKey);
    for (const cell of snap) {
      const [x, y, z] = cell.pos;
      const st = cell.state;
      if (!st || st === "minecraft:air") continue;
      solidSet.add(vKey(x, y, z));
      if (!st.startsWith("minecraft:moving_piston")) {
        try {
          fresh.setBlockFromString(x, y, z, st);
        } catch {
          /* blockstate not expressible in schematic */
        }
      }
    }
    liveSchem = fresh;
  }

  function updateLogAndTelemetry(actionNote = null) {
    if (!sim) return;
    const tick = Number(sim.tickCount());
    const total = Number(sim.changesCount());
    clockEl.textContent = `tick ${tick} · ${playing ? `${tps} TPS` : sim.isQuiescent() ? "quiescent" : "paused"}`;

    const changes = JSON.parse(sim.changesJson());
    const recent = changes.slice(-7).reverse();
    const lines = [];
    if (actionNote) lines.push(actionNote);
    if (recent.length === 0 && !actionNote) {
      lines.push(`Tick ${tick} · click any 3D lever/button/repeater, pick a block to build, or press Play.`);
    } else {
      for (const c of recent) {
        lines.push(
          `[t=${String(c.tick).padStart(2, " ")}] (${c.pos.join(",")})  ${c.from.replace("minecraft:", "")}  ->  ${c.to.replace("minecraft:", "")}`,
        );
      }
    }
    logEl.textContent = lines.join("\n");

    const stats = viewport?.lastMeshStats ?? { tris: 0, verts: 0, ms: 0 };
    const packDesc = customPackName ? customPackName.slice(0, 20) : "built-in 3D pack";
    telemetryEl.textContent = `Nucleation WASM MeshResult · ${stats.tris.toLocaleString()} tris · ${stats.ms.toFixed(1)} ms · ${packDesc} · ${total} transitions`;
  }

  async function flushRemesh() {
    if (remeshing || !needsRemesh || !viewport || !nuc || !sim) return;
    remeshing = true;
    const doRefit = refitNext;
    refitNext = false;
    needsRemesh = false;
    try {
      syncSolidAndSchemFromSim();
      const pack = getActivePack(nuc);
      await viewport.updateSchematicMesh(nuc, liveSchem, pack, doRefit);
      let moving = [];
      try {
        moving = JSON.parse(sim.movingBlocksJson() || "[]");
      } catch {
        moving = [];
      }
      viewport.syncFlights(nuc, pack, moving);
      updateLogAndTelemetry();
      statusEl.hidden = true;
    } catch (err) {
      statusEl.hidden = true;
      telemetryEl.textContent = `Mesh error: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      remeshing = false;
      if (needsRemesh) void flushRemesh();
    }
  }

  function scheduleRemesh(refit = false) {
    needsRemesh = true;
    if (refit) refitNext = true;
    void flushRemesh();
  }

  function setPlaying(next) {
    playing = Boolean(next);
    tickCarry = 0;
    playBtn.textContent = playing ? "Pause" : "Play";
    updateLogAndTelemetry();
  }

  function handleVoxelClick(hit, meta) {
    if (!sim || !nuc) return;
    const [x, y, z] = hit.pos;
    const isBreak = activeTool === "break" || meta.shiftKey;
    const isInteract = activeTool === "interact" && !meta.shiftKey;
    const short = (s) => String(s || "air").replace("minecraft:", "");

    try {
      if (isBreak) {
        const before = sim.getBlock(x, y, z);
        sim.placeBlock(x, y, z, "minecraft:air");
        viewport?.flashVoxel(x, y, z, 0xff6b6b);
        scheduleRemesh(false);
        updateLogAndTelemetry(`Broke ${short(before)} @ (${x},${y},${z})`);
        return;
      }

      if (isInteract) {
        const before = sim.getBlock(x, y, z);
        sim.useBlock(x, y, z);
        const after = sim.getBlock(x, y, z);
        const changed = after !== before;
        viewport?.flashVoxel(x, y, z, changed ? 0x5fd38d : 0xe8c14a);
        scheduleRemesh(false);
        updateLogAndTelemetry(
          changed
            ? `Used (${x},${y},${z}): ${short(before)} -> ${short(after)}`
            : `Clicked ${short(before)} @ (${x},${y},${z}) — not interactive (switch Tool to build or Shift+Click to break)`,
        );
        return;
      }

      const toolDef = TICK_TOOLS.find((t) => t.id === activeTool);
      if (!toolDef?.state) return;
      const nx = x + hit.face[0];
      const ny = y + hit.face[1];
      const nz = z + hit.face[2];
      const state = toolDef.state(hit.face, meta.facing);
      sim.placeBlock(nx, ny, nz, state);
      viewport?.flashVoxel(nx, ny, nz, 0x53c7b4);
      scheduleRemesh(false);
      updateLogAndTelemetry(`Placed ${short(sim.getBlock(nx, ny, nz))} @ (${nx},${ny},${nz})`);
    } catch (err) {
      updateLogAndTelemetry(`Action error: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  async function ensureEngineAndViewport() {
    if (!nuc || !viewport) {
      const [{ loadThree, TickSimViewport }, loadedNuc] = await Promise.all([
        import("./tick-sim-viewport.js"),
        loadNucleation(),
      ]);
      nuc = loadedNuc;
      if (!viewport) {
        const threeKit = await loadThree();
        viewport = new TickSimViewport(stageEl, threeKit, {
          isSolid: (x, y, z) => solidSet.has(vKey(x, y, z)),
          getBlock: (x, y, z) => {
            try {
              return sim ? sim.getBlock(x, y, z) : "minecraft:air";
            } catch {
              return "minecraft:air";
            }
          },
          onHover: (info) => {
            if (!info) {
              hoverEl.textContent =
                activeTool === "interact"
                  ? "Hover a block · Click lever/button/repeater · Drag orbits · Shift+Click breaks"
                  : activeTool === "break"
                    ? "Break mode · Click any block to mine it"
                    : `Build (${TICK_TOOLS.find((t) => t.id === activeTool)?.label}) · Click a face to place`;
              return;
            }
            const clean = String(info.state || "minecraft:air").replace("minecraft:", "");
            hoverEl.textContent = `(${info.pos.join(", ")})  ${clean}`;
          },
          onVoxelClick: handleVoxelClick,
          onFrame: (dt) => {
            if (!sim || !nuc) return;
            let subTick = 0;
            if (playing) {
              tickCarry += dt * tps;
              const steps = Math.min(Math.floor(tickCarry), 8);
              if (steps > 0) {
                tickCarry -= steps;
                const beforeCount = Number(sim.changesCount());
                for (let i = 0; i < steps; i += 1) sim.step();
                const afterCount = Number(sim.changesCount());
                if (afterCount !== beforeCount || viewport.flights.length > 0) {
                  scheduleRemesh(false);
                } else {
                  clockEl.textContent = `tick ${sim.tickCount()} · ${tps} TPS`;
                }
              }
              subTick = Math.min(tickCarry, 1);
            }
            if (viewport.flights.length > 0) {
              viewport.animateFlights(nuc, getActivePack(nuc), Number(sim.tickCount()) + subTick);
            }
          },
        });
        viewport.setTool(activeTool);
      }
    }
  }

  async function initCircuit(bytesOverride = null, fileLabel = null) {
    setPlaying(false);
    statusEl.textContent = fileLabel ? `Meshing ${fileLabel}...` : "Meshing circuit via Nucleation WASM...";
    statusEl.hidden = false;
    try {
      await ensureEngineAndViewport();
      let baseSchem;
      if (bytesOverride) {
        customFileBytes = bytesOverride;
        customFileName = fileLabel || "custom.schem";
        baseSchem = nuc.Schematic.fromData(bytesOverride);
        useBtn.hidden = true;
        hintEl.textContent = `Loaded ${customFileName}`;
      } else {
        customFileBytes = null;
        customFileName = null;
        baseSchem = TICK_CIRCUITS[circuitKey].build(nuc);
        useBtn.hidden = false;
        useBtn.textContent = TICK_CIRCUITS[circuitKey].actionLabel;
        hintEl.textContent = "Drop .schem / .litematic / .nbt here";
      }

      sim = nuc.TickSimulation.fromSchematic(
        baseSchem,
        nuc.TickSettleMode.InWorld,
        0,
        0,
        0,
        TICK_EXTRA_STATES,
      );
      sim.recordUpdates(true);
      scheduleRemesh(true);

      globalThis.__nucleationTickLab = {
        get sim() { return sim; },
        get viewport() { return viewport; },
        get activeTool() { return activeTool; },
        projectVoxel: (x, y, z) => viewport?.projectVoxelToClient(x, y, z),
      };
    } catch (err) {
      statusEl.hidden = true;
      telemetryEl.textContent = `mc-tick error: ${err instanceof Error ? err.message : String(err)}`;
    }
  }

  root.querySelector('[data-role="tick-presets"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-circuit]");
    if (!btn) return;
    circuitKey = btn.dataset.circuit;
    for (const el of root.querySelectorAll("[data-circuit]")) {
      el.setAttribute("aria-checked", String(el === btn));
    }
    void initCircuit(null, null);
  });

  root.querySelector('[data-role="tick-tools"]').addEventListener("click", (e) => {
    const btn = e.target.closest("[data-tool]");
    if (!btn) return;
    activeTool = btn.dataset.tool;
    for (const el of root.querySelectorAll("[data-tool]")) {
      el.setAttribute("aria-checked", String(el === btn));
    }
    viewport?.setTool(activeTool);
  });

  playBtn.addEventListener("click", () => setPlaying(!playing));
  tpsSel.addEventListener("change", () => {
    tps = Number(tpsSel.value) || 10;
    if (playing) updateLogAndTelemetry();
  });

  useBtn.addEventListener("click", () => {
    if (!sim) return;
    const [x, y, z] = TICK_CIRCUITS[circuitKey].inputPos;
    sim.useBlock(x, y, z);
    viewport?.flashVoxel(x, y, z, 0x5fd38d);
    scheduleRemesh(false);
  });
  root.querySelector('[data-role="tick-step1"]').addEventListener("click", () => {
    if (!sim) return;
    setPlaying(false);
    sim.step();
    scheduleRemesh(false);
  });
  root.querySelector('[data-role="tick-step2"]').addEventListener("click", () => {
    if (!sim) return;
    setPlaying(false);
    sim.run(2);
    scheduleRemesh(false);
  });
  root.querySelector('[data-role="tick-settle"]').addEventListener("click", () => {
    if (!sim) return;
    setPlaying(false);
    sim.runUntilQuiescent(120);
    scheduleRemesh(false);
  });
  root.querySelector('[data-role="tick-reset"]').addEventListener("click", () => {
    void initCircuit(customFileBytes, customFileName);
  });
  root.querySelector('[data-role="tick-cam"]').addEventListener("click", () => {
    viewport?.resetCamera();
  });

  root.querySelector('[data-role="tick-export"]').addEventListener("click", () => {
    if (!liveSchem) return;
    const bytes = decodeBase64(liveSchem.toSchematicB64());
    const url = URL.createObjectURL(new Blob([bytes], { type: "application/octet-stream" }));
    const a = document.createElement("a");
    a.href = url;
    a.download = `${(customFileName || circuitKey).replace(/\.[^.]+$/, "")}-tick${sim?.tickCount?.() ?? 0}.schem`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 2000);
  });

  async function openSchematicFile(file) {
    if (!file) return;
    const bytes = new Uint8Array(await file.arrayBuffer());
    await initCircuit(bytes, file.name);
  }

  schemInput.addEventListener("change", () => {
    const file = schemInput.files?.[0];
    schemInput.value = "";
    if (file) void openSchematicFile(file);
  });

  stageEl.addEventListener("dragover", (e) => {
    e.preventDefault();
    stageEl.dataset.dragover = "true";
  });
  stageEl.addEventListener("dragleave", () => {
    stageEl.dataset.dragover = "false";
  });
  stageEl.addEventListener("drop", (e) => {
    e.preventDefault();
    stageEl.dataset.dragover = "false";
    const file = e.dataTransfer?.files?.[0];
    if (!file) return;
    if (/\.(zip|jar)$/i.test(file.name)) {
      void (async () => {
        if (!nuc) nuc = await loadNucleation();
        await loadCustomPackFile(nuc, file);
      })();
    } else {
      void openSchematicFile(file);
    }
  });

  const onPackUpdated = (name) => {
    packLabel.textContent = `Pack: ${name.slice(0, 18)}`;
    viewport?.clearCaches();
    scheduleRemesh(false);
  };
  packListeners.add(onPackUpdated);

  packInput.addEventListener("change", async () => {
    const file = packInput.files?.[0];
    packInput.value = "";
    if (!file) return;
    try {
      if (!nuc) nuc = await loadNucleation();
      statusEl.textContent = "Loading resource pack...";
      statusEl.hidden = false;
      await loadCustomPackFile(nuc, file);
    } catch (err) {
      statusEl.hidden = true;
      telemetryEl.textContent = `Pack error: ${err instanceof Error ? err.message : String(err)}`;
    }
  });

  void initCircuit();
  return {
    onThemeChange: () => {
      viewport?.setTheme(document.body.getAttribute("data-md-color-scheme") === "vellum");
    },
    destroy: () => {
      setPlaying(false);
      packListeners.delete(onPackUpdated);
      viewport?.destroy();
      viewport = null;
      if (globalThis.__nucleationTickLab) delete globalThis.__nucleationTickLab;
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

  for (const el of document.querySelectorAll("[data-worldgen-lab]")) {
    const destroy = await mountWorldgenLab(el, ensureModelViewer);
    cleanups.push(destroy);
  }

  for (const el of document.querySelectorAll("[data-palette-studio]")) {
    const destroy = await mountPaletteStudio(el, ensureModelViewer);
    cleanups.push(destroy);
  }

  for (const el of document.querySelectorAll("[data-tick-studio]")) {
    const tick = await mountTickStudio(el);
    if (tick?.onThemeChange) rerenders.push(tick.onThemeChange);
    if (tick?.destroy) cleanups.push(tick.destroy);
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
