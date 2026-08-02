/** The meta structure as three.js geometry.
 *
 * `meta.ts` decides WHAT the overlay says; this file decides how it is drawn.
 * Kept separate for the same reason door-cert splits `lib/xray.ts` from
 * `components/MeshReplay.tsx`: the vocabulary is shared with a flat SVG panel
 * that must never import three.
 *
 * Everything here is OPAQUE and DEPTH-TESTED. Additive glow was tried in the
 * door app and clipped a five-deep build to white; max-blending invents hues at
 * overlaps. A mark that means something must survive being stacked, so the
 * marks are solid geometry and occlusion is solved where it actually lives —
 * by ghosting the BLOCKS (see GLBlocks), not by making the marks shine through.
 *
 * Materials are `MeshBasicMaterial`: unlit, so the validated ink and role hues
 * arrive on screen as the values that were validated, not as whatever the key
 * light leaves of them. */

import * as THREE from "three";

import {
  EDGE_STYLE,
  graphInk,
  graphInkSoft,
  rolePalette,
  type MetaStructure,
  type Role,
} from "./meta";

/** Node marker sizes, in blocks. Small enough that two adjacent cells' markers
 * never touch (0.5 apart at the closest), big enough to hit at a glance. */
const GROUP_R = 0.15;
/** Where a device's facing stub starts and ends, measured from the cell
 * centre. A device gets NO body marker of its own — the role core already
 * occupies that centre, and stacking a second cube there produced a dark blob
 * that hid the very colour it sat on. The stub alone says "device here, facing
 * that way", which is the whole of what the marker was for. */
const DEVICE_R = 0.19;
/** Half-edge of the opaque role core.
 *
 * The role CANNOT ride on the ghost's tint. The palette was validated at full
 * chroma; a 26%-opacity block over a light page composites to a pastel, and the
 * measured separations do not survive it — the first x-ray shot had a 2-cell
 * amber kicker that was plainly visible with the x-ray OFF and simply gone with
 * it on. So every hue-bearing cell also carries an opaque, depth-tested core at
 * the validated colour, and the ghost is left to do the one job it is good at:
 * showing the shape. */
const CORE_R = 0.15;
/** Edge shaft radius. A cylinder, not a line: WebGL ignores `linewidth`, so a
 * line overlay is 1 device pixel at every zoom and vanishes on the evolved
 * blobs exactly when it is needed most.
 *
 * Kept THIN. The graph is hub-and-spoke — every device has a sticks_to edge
 * into its own group's centroid — so four or five edges meet at each hub, and
 * at 0.03 they fused into one dark mass that said nothing. At 0.016 the same
 * edges read as separate strands and the hub stays a hub. */
const SHAFT_R = 0.016;
/** The device stub is a marker, not an edge, so it stays heavier than one. */
const STUB_R = 0.032;
/** The arrowhead is the only thing that distinguishes `pushes` from
 * `sticks_to`, so it is drawn generously — a timid head reads as a rendering
 * artefact rather than as the direction the body is about to go. */
const HEAD_R = 0.062;
const HEAD_LEN = 0.17;

/** Dash geometry per broken mark, in blocks: [dash, gap]. */
const DASH: Record<"dash" | "dot", [number, number]> = {
  dash: [0.2, 0.13],
  dot: [0.06, 0.11],
};

const FACE_VEC: Record<string, [number, number, number]> = {
  east: [1, 0, 0],
  west: [-1, 0, 0],
  up: [0, 1, 0],
  down: [0, -1, 0],
  south: [0, 0, 1],
  north: [0, 0, -1],
};

const UP = new THREE.Vector3(0, 1, 0);

export interface MetaOverlay {
  group: THREE.Group;
  /** Layer visibility, applied immediately. */
  setLayers(opts: { roles: boolean; graph: boolean }): void;
  dispose(): void;
}

/** A segment as an instance transform of a unit cylinder (radius 1, height 1,
 * along +Y). Returns false for a degenerate segment. */
function segment(
  m: THREE.Matrix4,
  from: THREE.Vector3,
  to: THREE.Vector3,
  radius: number,
  q: THREE.Quaternion,
  v: THREE.Vector3,
): boolean {
  v.subVectors(to, from);
  const len = v.length();
  if (len < 1e-4) return false;
  q.setFromUnitVectors(UP, v.divideScalar(len));
  m.compose(
    new THREE.Vector3().addVectors(from, to).multiplyScalar(0.5),
    q,
    new THREE.Vector3(radius, len, radius),
  );
  return true;
}

/** Box-edge line segments for one cell, appended to `out` as raw x,y,z triples.
 * Slightly inset so a cage never z-fights the block face it wraps. */
function cageEdges(out: number[], x: number, y: number, z: number): void {
  const p = 0.012;
  const a = [x + p, y + p, z + p];
  const b = [x + 1 - p, y + 1 - p, z + 1 - p];
  const c = (i: number, j: number, k: number) => [
    i ? b[0] : a[0],
    j ? b[1] : a[1],
    k ? b[2] : a[2],
  ];
  const E: Array<[number[], number[]]> = [];
  for (let j = 0; j < 2; j++)
    for (let k = 0; k < 2; k++) E.push([c(0, j, k), c(1, j, k)]);
  for (let i = 0; i < 2; i++)
    for (let k = 0; k < 2; k++) E.push([c(i, 0, k), c(i, 1, k)]);
  for (let i = 0; i < 2; i++)
    for (let j = 0; j < 2; j++) E.push([c(i, j, 0), c(i, j, 1)]);
  for (const [s, e] of E) out.push(...s, ...e);
}

/**
 * Build the overlay for one meta structure.
 *
 * `dark` selects the palette mode — the steps are chosen per surface, not
 * flipped (see the colour note in `meta.ts`).
 */
export function buildMetaOverlay(
  meta: MetaStructure,
  dark: boolean,
): MetaOverlay {
  const root = new THREE.Group();
  root.name = "meta-overlay";
  const owned: Array<THREE.BufferGeometry | THREE.Material> = [];
  const keep = <T extends THREE.BufferGeometry | THREE.Material>(t: T): T => {
    owned.push(t);
    return t;
  };

  const palette = rolePalette(dark);
  const ink = graphInk(dark);

  const m = new THREE.Matrix4();
  const q = new THREE.Quaternion();
  const v = new THREE.Vector3();

  /* ----------------------------------------------------- role layer --- */
  // Two marks, and the split is the colour budget from meta.ts made physical:
  // the three hue-bearing roles get an opaque CORE at the validated colour,
  // dead weight gets a hueless CAGE. GLBlocks separately tints the block itself
  // so the role reads on a solid build too; the core is what survives the
  // x-ray.
  const roleLayer = new THREE.Group();
  roleLayer.name = "meta-roles";
  root.add(roleLayer);

  const cagePts: number[] = [];
  /** Cores per role, so each role needs one material and one draw call. */
  const cores = new Map<Role, Array<[number, number, number]>>();
  for (const [key, role] of meta.roles) {
    const [x, y, z] = key.split(",").map(Number);
    const r = role as Role;
    if (palette[r].mark === "cage") {
      cageEdges(cagePts, x, y, z);
      continue;
    }
    // Every hue-bearing cell gets a core, including cells with no block at all
    // — a kick source is classified but has nothing to tint, and tinting alone
    // would drop it from the 3-D view while the flat panel still drew it.
    const list = cores.get(r) ?? [];
    list.push([x + 0.5, y + 0.5, z + 0.5]);
    cores.set(r, list);
  }
  if (cores.size) {
    const geo = keep(new THREE.BoxGeometry(CORE_R * 2, CORE_R * 2, CORE_R * 2));
    for (const [role, list] of cores) {
      const mat = keep(
        new THREE.MeshBasicMaterial({
          color: palette[role].hex,
          toneMapped: false,
        }),
      );
      const mesh = new THREE.InstancedMesh(geo, mat, list.length);
      list.forEach((p, i) => {
        m.compose(
          new THREE.Vector3(...p),
          new THREE.Quaternion(),
          new THREE.Vector3(1, 1, 1),
        );
        mesh.setMatrixAt(i, m);
      });
      mesh.renderOrder = 2;
      roleLayer.add(mesh);
    }
  }
  if (cagePts.length) {
    const geo = keep(new THREE.BufferGeometry());
    geo.setAttribute("position", new THREE.Float32BufferAttribute(cagePts, 3));
    const mat = keep(
      new THREE.LineBasicMaterial({ color: palette.dead.hex, toneMapped: false }),
    );
    const cages = new THREE.LineSegments(geo, mat);
    cages.renderOrder = 2;
    roleLayer.add(cages);
  }

  /* ---------------------------------------------------- graph layer --- */
  const graphLayer = new THREE.Group();
  graphLayer.name = "meta-graph";
  root.add(graphLayer);

  const inkMat = keep(
    new THREE.MeshBasicMaterial({ color: ink, toneMapped: false }),
  );

  // --- nodes. Group vs device separate by SHAPE, never by colour: the graph
  // layer is achromatic on purpose so it cannot be mistaken for a fifth role.
  const groups = meta.nodes.filter((n) => n.kind === "group");
  const devices = meta.nodes.filter((n) => n.kind === "device");

  if (groups.length) {
    const geo = keep(new THREE.OctahedronGeometry(1, 0));
    const mesh = new THREE.InstancedMesh(geo, inkMat, groups.length);
    groups.forEach((n, i) => {
      // Bigger body, bigger marker — a log scale so a 40-cell group is
      // noticeably larger than a 2-cell one without swallowing it.
      const r = GROUP_R * (1 + 0.22 * Math.log2(Math.max(n.size, 1)));
      m.compose(
        new THREE.Vector3(...n.pos),
        new THREE.Quaternion(),
        new THREE.Vector3(r, r, r),
      );
      mesh.setMatrixAt(i, m);
    });
    mesh.renderOrder = 2;
    graphLayer.add(mesh);
  }

  if (devices.length) {
    // Facing stub, and nothing else: which way a device points is half of what
    // a device IS, and the flat panel could only ever say it in a tooltip.
    const stubGeo = keep(new THREE.CylinderGeometry(1, 1, 1, 6));
    const stubs = new THREE.InstancedMesh(stubGeo, inkMat, devices.length);
    let si = 0;
    for (const n of devices) {
      const f = FACE_VEC[n.facing ?? ""];
      if (!f) continue;
      const a = new THREE.Vector3(...n.pos).addScaledVector(
        new THREE.Vector3(...f),
        DEVICE_R,
      );
      const b = new THREE.Vector3(...n.pos).addScaledVector(
        new THREE.Vector3(...f),
        0.45,
      );
      if (segment(m, a, b, STUB_R, q, v)) stubs.setMatrixAt(si++, m);
    }
    stubs.count = si;
    stubs.renderOrder = 2;
    graphLayer.add(stubs);
  }

  // --- edges. Collected into two instanced meshes (shafts, arrowheads); the
  // four kinds are told apart by how the shaft is broken up and whether it
  // ends in a head. See EDGE_STYLE in meta.ts for why not by colour.
  // Matter/force at full ink, signal recessive — see graphInkSoft in meta.ts.
  const softMat = keep(
    new THREE.MeshBasicMaterial({ color: graphInkSoft(), toneMapped: false }),
  );
  const shafts: THREE.Matrix4[] = [];
  const softShafts: THREE.Matrix4[] = [];
  const heads: THREE.Matrix4[] = [];

  for (const e of meta.edges) {
    const style = EDGE_STYLE[e.kind];
    const a = new THREE.Vector3(...e.from.pos);
    const b = new THREE.Vector3(...e.to.pos);
    const dir = new THREE.Vector3().subVectors(b, a);
    const len = dir.length();
    // A device that sticks to a one-cell group sits ON its own node. There is
    // nothing to draw and nothing lost: the marker already says both.
    if (len < GROUP_R + CORE_R + 0.1) continue;
    dir.divideScalar(len);
    // Start and end clear of the node markers so the edge reads as joining
    // them rather than skewering them.
    const start = a.clone().addScaledVector(dir, CORE_R + 0.06);
    let end = b.clone().addScaledVector(dir, -(GROUP_R + 0.05));

    if (style.mark === "arrow") {
      const tip = end.clone();
      end = end.clone().addScaledVector(dir, -HEAD_LEN);
      const hm = new THREE.Matrix4();
      if (segment(hm, end, tip, 1, q, v)) {
        // The cone is authored radius 1 / height 1, so the segment scale
        // (r, len, r) has to be re-imposed with the head's own proportions.
        hm.compose(
          new THREE.Vector3().addVectors(end, tip).multiplyScalar(0.5),
          new THREE.Quaternion().setFromUnitVectors(UP, dir),
          new THREE.Vector3(HEAD_R, HEAD_LEN, HEAD_R),
        );
        heads.push(hm);
      }
    }

    if (style.mark === "bond" || style.mark === "arrow") {
      const sm = new THREE.Matrix4();
      if (segment(sm, start, end, SHAFT_R, q, v)) shafts.push(sm);
    } else {
      const [dl, gap] = DASH[style.mark];
      const span = start.distanceTo(end);
      const step = dl + gap;
      const n = Math.max(1, Math.floor((span + gap) / step));
      // Centre the dash run so both ends look deliberate.
      const pad = (span - (n * step - gap)) / 2;
      for (let i = 0; i < n; i++) {
        const s = start.clone().addScaledVector(dir, pad + i * step);
        const t = s.clone().addScaledVector(dir, dl);
        const sm = new THREE.Matrix4();
        if (segment(sm, s, t, SHAFT_R * 0.75, q, v)) softShafts.push(sm);
      }
    }
  }

  const shaftGeo = keep(new THREE.CylinderGeometry(1, 1, 1, 6));
  for (const [list, mat] of [
    [shafts, inkMat],
    [softShafts, softMat],
  ] as const) {
    if (!list.length) continue;
    const mesh = new THREE.InstancedMesh(shaftGeo, mat, list.length);
    list.forEach((mm, i) => mesh.setMatrixAt(i, mm));
    mesh.renderOrder = 2;
    graphLayer.add(mesh);
  }
  if (heads.length) {
    const geo = keep(new THREE.ConeGeometry(1, 1, 8));
    const mesh = new THREE.InstancedMesh(geo, inkMat, heads.length);
    heads.forEach((mm, i) => mesh.setMatrixAt(i, mm));
    mesh.renderOrder = 2;
    graphLayer.add(mesh);
  }

  return {
    group: root,
    setLayers({ roles, graph }) {
      roleLayer.visible = roles;
      graphLayer.visible = graph;
    },
    dispose() {
      root.traverse((o) => {
        const im = o as THREE.InstancedMesh;
        if (im.isInstancedMesh) im.dispose();
      });
      for (const t of owned) t.dispose();
      root.clear();
    },
  };
}

/** Paint one block's cloned materials with its role hue.
 *
 * A multiply on the material colour, so the block keeps its texture and its
 * baked AO and merely wears the role — which is what "tint" has to mean if the
 * overlay is to stay legible on a 40-block blob where an inset marker would be
 * swallowed. Dead weight is deliberately NOT tinted: it has no hue (see
 * meta.ts) and is marked by its cage instead. */
export function tintForRole(role: Role | undefined, dark: boolean): number | null {
  if (!role) return null;
  const style = rolePalette(dark)[role];
  return style.mark === "tint" ? style.hex : null;
}
