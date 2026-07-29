/** Exhibit A, native-quality: the recorded change log replayed through
 * nucleation's REAL mesher (wasm) into a three.js WebGL scene. Every unique
 * blockstate in the cast is meshed once as a 1-block schematic (levers, dust,
 * repeaters, torches, slabs get their real models, not cubes) and instanced
 * per cast member; poses come from the same cast/castAt animation brain as the
 * classic view (g4mespeed pause-at-end easing, fractional wall-clock ticks).
 *
 * X-RAY MODE turns the same scene into a propagation view. The build drops to
 * a ghost and the recorded redstone UPDATE stream is drawn on top: a cell
 * flares when an update lands on it, coloured by tick phase or by update kind,
 * decaying over ~an eighth of a second so a wave reads as it travels rather
 * than strobing. Every other tool can show you which blocks CHANGED; this
 * shows which blocks were TOLD — including the ones that were told and did
 * nothing, which is the whole point (an update crossing a leaf block is
 * invisible in a change log and visible here).
 *
 * It is only meaningful because the engine's update ORDER is verified against
 * real Minecraft: `(tick, seq)` is the order the game itself would deliver in,
 * which is what makes the sub-tick scrubber below more than an animation.
 *
 * Smoothness contract: the rAF loop never goes through React — time lives in
 * a ref, member groups are posed in place (zero per-frame allocations), and
 * the scrubber/readout are poked imperatively. React only re-renders on
 * play/pause/speed/mode changes. */

import { useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { buildCast, headClipHalfSpace, type CastMember } from "../lib/cast";
import {
  CARRIER_ALPHA,
  CARRIER_ALPHA_XRAY,
  doorwayFacts,
  doorwayStyles,
  doorwaySummary,
  idleStyle,
  IDLE_DOT,
} from "../lib/doorway";
import { instanceFor, addDefaultLights, fitCamera, meshStats } from "../lib/mesher";
import type { ApertureGeometry, Replay, Vec3 } from "../lib/types";
import {
  channelStyles,
  hexCss,
  XRAY_SURFACE,
  type Channel,
  type XrayData,
  type XrayHeatTick,
} from "../lib/xray";
import { VoxelReplay } from "./VoxelReplay";

const SPEEDS = [0.5, 1, 2, 4];
const STAGE_H = 380;

/* --------------------------------------------------------------- x-ray --- */

/** Seconds for a flare to fade to 1/e. Short enough that a wave has a visible
 * front, long enough that a single tick does not strobe. */
const FLARE_TAU = 0.13;
/** Seconds to sweep one tick's whole update sequence at 1x. Constant in TIME,
 * not in updates: a 19,834-update tick and a 12-update tick both take this
 * long, so the wavefront moves at a readable speed either way. */
const SWEEP_SEC = 2.6;
const SUB_SPEEDS = [0.25, 1, 4];
/** Ghost opacity for a block that never receives an update, and for one that
 * does. The first is deliberately near-nothing: the inert half of a door
 * should recede so the working half reads through it. */
const GHOST_INERT = 0.03;
const GHOST_LIVE = 0.13;
/** …and how far its colour is pulled down. Opacity alone is not enough: a
 * five-deep build stacks twenty translucent layers, and twenty layers of a
 * pale block accumulate into a bright haze that beats the flares in front of
 * it. A ghost has to be DARK as well as thin.
 *
 * Taken from 0.34 to 0.20 after the ghost was judged too heavy. The two
 * failure modes the first pass documented were re-checked at the new value on
 * the busiest tick of both test doors: the flares still out-brighten the ghost
 * everywhere (0.20 of a mid-grey block behind a stack of layers stays below
 * the dimmest legal flare step, which is 0.3 of its palette hue), and the
 * flare colours are untouched, so the legend still reads true. It stops at
 * 0.20 rather than going lower because the piston tape and the frame are the
 * only cues to where you are in the build once the models go translucent —
 * under about 0.15 the silhouette breaks up and the flares float in the
 * dark with nothing to place them against. */
const GHOST_TINT = 0.2;

/* ------------------------------------------------------------- doorway --- */

/** How far a door-block pane is inset inside its passage cell. The gap is
 * doing two jobs, both about the nesting: it keeps the violet shell visible
 * around the pattern, and it keeps the panes from fusing into one magenta
 * mass, so a doorway whose pattern only fills SOME of its passage (a 4 × 4
 * vault door fills 32 of 48 cells) reads as blocks-in-a-hole rather than as a
 * filled block. */
const PANE = 0.7;
/** …and how big a CARRIER's core is: a small solid marker rather than a
 * panel, so "the panel" and "the slime pushing it" separate by silhouette
 * without spending a hue on the difference. See the three-channel note in
 * `lib/doorway.ts`. */
const CORE = 0.4;
/** Pane opacity, plain replay and x-ray. Lower under the x-ray so the flares
 * keep the stage — the overlay is an annotation, not a fourth channel. */
const PANE_ALPHA = 0.62;
const PANE_ALPHA_XRAY = 0.42;

/** A 45° hatch, drawn once into a canvas and repeated over the door-block
 * panes. This is the overlay's escape from the flare palette: `lib/doorway.ts`
 * records that no fourth hue clears the all-pairs gate beside the x-ray's
 * three, so the door blocks are separated by TEXTURE instead — a striped pane
 * is not a flare at any hue, and the stripes let the flares behind it through
 * rather than masking them. */
function hatchTexture(): THREE.Texture {
  const s = 64;
  const c = document.createElement("canvas");
  c.width = s;
  c.height = s;
  const g = c.getContext("2d")!;
  g.clearRect(0, 0, s, s);
  g.strokeStyle = "#ffffff";
  g.lineWidth = 8;
  for (let i = -s; i < s * 2; i += 21) {
    g.beginPath();
    g.moveTo(i, 0);
    g.lineTo(i + s, s);
    g.stroke();
  }
  const t = new THREE.CanvasTexture(c);
  t.wrapS = THREE.RepeatWrapping;
  t.wrapT = THREE.RepeatWrapping;
  return t;
}

/** The passage as a wireframe SHELL: the grid on the faces where the passage
 * meets solid wall, and nothing inside.
 *
 * Wireframing every cell was built first and rejected on sight — a 6 × 6 × 3
 * passage is 108 boxes, and 108 boxes of interior lattice is a solid violet
 * brick, the opposite of "here is the hole". Culling every face that has
 * another passage cell behind it leaves exactly the surface you would walk
 * through: the opening's grid at each end, and the four strips that show how
 * thick the wall is. Same cells, same counts, legible shape.
 *
 * Shared edges are emitted once. Static — built once, never touched by the
 * frame loop. */
function passageEdges(cells: Vec3[]): THREE.BufferGeometry {
  const key = (x: number, y: number, z: number) => `${x},${y},${z}`;
  const set = new Set(cells.map((p) => key(p[0], p[1], p[2])));
  // The four corners of each face, as ± half-offsets from the cell centre.
  const FACES: { d: Vec3; quad: Vec3[] }[] = [];
  for (let axis = 0; axis < 3; axis++)
    for (const s of [-1, 1]) {
      const d: Vec3 = [0, 0, 0];
      d[axis] = s;
      const [u, v] = [0, 1, 2].filter((a) => a !== axis);
      const quad: Vec3[] = [
        [-1, -1],
        [1, -1],
        [1, 1],
        [-1, 1],
      ].map(([a, b]) => {
        const c: Vec3 = [0, 0, 0];
        c[axis] = s;
        c[u] = a;
        c[v] = b;
        return c;
      });
      FACES.push({ d, quad });
    }

  const pts: number[] = [];
  const seen = new Set<string>();
  for (const p of cells)
    for (const f of FACES) {
      if (set.has(key(p[0] + f.d[0], p[1] + f.d[1], p[2] + f.d[2]))) continue;
      for (let i = 0; i < 4; i++) {
        const a = f.quad[i];
        const b = f.quad[(i + 1) % 4];
        const ax = p[0] + 0.5 + a[0] / 2;
        const ay = p[1] + 0.5 + a[1] / 2;
        const az = p[2] + 0.5 + a[2] / 2;
        const bx = p[0] + 0.5 + b[0] / 2;
        const by = p[1] + 0.5 + b[1] / 2;
        const bz = p[2] + 0.5 + b[2] / 2;
        // Two coplanar faces of neighbouring cells share this edge; order the
        // endpoints so both spellings collapse to one line.
        const k =
          ax < bx || (ax === bx && (ay < by || (ay === by && az <= bz)))
            ? `${ax},${ay},${az}|${bx},${by},${bz}`
            : `${bx},${by},${bz}|${ax},${ay},${az}`;
        if (seen.has(k)) continue;
        seen.add(k);
        pts.push(ax, ay, az, bx, by, bz);
      }
    }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.BufferAttribute(Float32Array.from(pts), 3));
  return geo;
}

/** True when the page is on its dark plate. The overlay's violet steps down
 * for the white page and up for the two dark surfaces (dark theme, and the
 * x-ray darkroom, which is dark in both themes). */
function pageIsDark(): boolean {
  if (typeof document === "undefined") return false;
  const t = document.documentElement.dataset.theme;
  if (t === "dark") return true;
  if (t === "light") return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

/** A hollow box drawn as twelve thin bars. This is a MARK, not a hue — it is
 * how the `boundary` phase stays distinguishable without a fourth colour (see
 * lib/xray.ts for why a fourth colour is not available). */
function cageGeometry(t = 0.1): THREE.BufferGeometry {
  const parts: Float32Array[] = [];
  const bar = (sx: number, sy: number, sz: number, px: number, py: number, pz: number) => {
    const g = new THREE.BoxGeometry(sx, sy, sz).translate(px, py, pz).toNonIndexed();
    parts.push(g.getAttribute("position").array as Float32Array);
    g.dispose();
  };
  const h = 0.5;
  for (const y of [-h, h]) for (const z of [-h, h]) bar(1, t, t, 0, y, z);
  for (const x of [-h, h]) for (const z of [-h, h]) bar(t, 1, t, x, 0, z);
  for (const x of [-h, h]) for (const y of [-h, h]) bar(t, t, 1, x, y, 0);
  const out = new Float32Array(parts.reduce((a, p) => a + p.length, 0));
  let o = 0;
  for (const p of parts) {
    out.set(p, o);
    o += p.length;
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.BufferAttribute(out, 3));
  return geo;
}

/** Linear-space colour table + mark flags for a channel, indexed by category.
 * The extra trailing slot catches a phase the engine reports but never fires
 * a counted update in. */
function colourTable(data: XrayData, channel: Channel) {
  const styles = channelStyles(data, channel);
  const cols = new Float32Array((styles.length + 1) * 3);
  const cage = new Uint8Array(styles.length + 1);
  const c = new THREE.Color();
  styles.forEach((s, i) => {
    c.setHex(s.hex);
    cols[i * 3] = c.r;
    cols[i * 3 + 1] = c.g;
    cols[i * 3 + 2] = c.b;
    cage[i] = s.mark === "cage" ? 1 : 0;
  });
  const last = styles.length;
  c.setHex(0xe8e6df);
  cols[last * 3] = c.r;
  cols[last * 3 + 1] = c.g;
  cols[last * 3 + 2] = c.b;
  cage[last] = 1;
  return { cols, cage };
}

const fmt = (n: number) => n.toLocaleString("en-US");

/** Rolling frame-time stats for the dev report (window.__replayFrameStats). */
function makeFrameStats() {
  const samples = new Float32Array(240);
  let n = 0;
  let i = 0;
  return {
    push(dtMs: number) {
      samples[i] = dtMs;
      i = (i + 1) % samples.length;
      if (n < samples.length) n++;
      if (i % 60 === 0) {
        const arr = Array.from(samples.slice(0, n)).sort((a, b) => a - b);
        (window as unknown as Record<string, unknown>).__replayFrameStats = {
          p50: arr[Math.floor(n * 0.5)],
          p95: arr[Math.floor(n * 0.95)],
          worst: arr[n - 1],
          n,
        };
      }
    },
    reset() {
      n = 0;
      i = 0;
      (window as unknown as Record<string, unknown>).__replayFrameStats = null;
    },
  };
}

function WebglReplay({
  replay,
  lever,
  xray,
  geometry,
  idle,
  onFail,
}: {
  replay: Replay;
  lever: Vec3;
  xray: XrayData | null;
  geometry: ApertureGeometry | null;
  /** Cells that neither moved nor received an update all cycle. */
  idle: Vec3[] | null;
  onFail: (err: string) => void;
}) {
  const { simTicks } = replay;
  const mountRef = useRef<HTMLDivElement>(null);
  const readoutRef = useRef<HTMLElement>(null);
  const rangeRef = useRef<HTMLInputElement>(null);
  const seqRef = useRef<HTMLElement>(null);
  const seqStateRef = useRef<HTMLElement>(null);
  const seqRangeRef = useRef<HTMLInputElement>(null);

  const timeRef = useRef(0);
  const playingRef = useRef(
    typeof window === "undefined" ||
      !window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );
  const speedRef = useRef(1);
  const [playing, setPlaying] = useState(playingRef.current);
  const [speed, setSpeed] = useState(1);
  const [ready, setReady] = useState(false);
  // Bumped on WebGL context loss so the stage rebuilds its renderer.
  const [glNonce, setGlNonce] = useState(0);

  // X-ray state. Everything the rAF loop reads lives in a ref as well, so
  // toggling a mode never rebuilds the scene and never disturbs playback.
  const [xrayOn, setXrayOn] = useState(false);
  const [channel, setChannel] = useState<Channel>("phase");
  const [subOn, setSubOn] = useState(false);
  const [subTick, setSubTick] = useState(0);
  const [subPlaying, setSubPlaying] = useState(true);
  const [subSpeed, setSubSpeed] = useState(1);
  const xrayOnRef = useRef(false);
  const channelRef = useRef<Channel>("phase");
  const subOnRef = useRef(false);
  const subTickRef = useRef(0);
  const subPlayingRef = useRef(true);
  const subSpeedRef = useRef(1);
  const cursorRef = useRef(0);
  const applyRef = useRef<((on: boolean) => void) | null>(null);

  // Doorway overlay. Independent of the x-ray in both directions: it is the
  // derived geometry, not the update stream, and it is arguably most useful on
  // the plain replay where you can watch the door blocks leave the passage
  // they are drawn in.
  const facts = useMemo(() => (geometry ? doorwayFacts(geometry) : null), [geometry]);
  const [doorwayOn, setDoorwayOn] = useState(false);
  const [dark, setDark] = useState(pageIsDark);
  const doorwayRef = useRef<((on: boolean) => void) | null>(null);
  // Dead weight. Its own toggle rather than a third state on the doorway's,
  // because the two answer unrelated questions and a builder compacting a
  // design wants to see the idle blocks WHILE watching the doorway work.
  const idleCells = idle && idle.length > 0 ? idle : null;
  const [idleOn, setIdleOn] = useState(false);
  const idleRef = useRef<((on: boolean) => void) | null>(null);
  const paintRef = useRef<((dark: boolean, xrayOn: boolean) => void) | null>(null);
  const focusRef = useRef<(() => void) | null>(null);

  // The overlay's violet has a light step and a dark step, so the stage has to
  // know which plate it is on. `data-theme` covers the app's own toggle; the
  // media query covers a viewer who never touched it.
  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const sync = () => setDark(pageIsDark());
    mq.addEventListener("change", sync);
    const obs = new MutationObserver(sync);
    obs.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    return () => {
      mq.removeEventListener("change", sync);
      obs.disconnect();
    };
  }, []);

  const cast = useMemo(() => {
    const c = buildCast(
      replay.blocks.map((b) => ({ pos: b.pos, state: b.state })),
      replay.changes,
      simTicks + 1,
    );
    // Dev/verification aid: lets tooling find piston move windows to
    // scrub-capture (see scripts/verify-arm-clamp.mjs).
    (window as unknown as Record<string, unknown>).__doorCast = c;
    return c;
  }, [replay, simTicks]);

  const wave = xray?.waves[subTick] ?? null;
  const busiestTick = useMemo(() => {
    if (!xray) return 0;
    let best = 0;
    let n = -1;
    for (const w of xray.waves) {
      if (w.n > n) {
        n = w.n;
        best = w.tick;
      }
    }
    return best;
  }, [xray]);

  useEffect(() => {
    const mount = mountRef.current;
    if (!mount) return;

    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    } catch (e) {
      onFail(String(e));
      return;
    }
    renderer.setClearColor(0x000000, 0);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    renderer.setSize(mount.clientWidth, STAGE_H);
    // Piston-head arms are clipped at their base's front face (see
    // headClipHalfSpace) so they never poke out the back mid-move.
    renderer.localClippingEnabled = true;
    mount.appendChild(renderer.domElement);
    const onLost = (e: Event) => {
      e.preventDefault();
      console.warn("[gl] replay context lost — recreating");
      setGlNonce((n) => n + 1);
    };
    renderer.domElement.addEventListener("webglcontextlost", onLost);

    const scene = new THREE.Scene();
    addDefaultLights(scene);
    const camera = new THREE.PerspectiveCamera(
      36,
      mount.clientWidth / STAGE_H,
      0.1,
      1000,
    );

    // World bounds over every cell a member ever occupies (incl. sources).
    const box = new THREE.Box3();
    for (const m of cast) {
      box.expandByPoint(new THREE.Vector3(m.x, m.y, m.z));
      box.expandByPoint(new THREE.Vector3(m.x + 1, m.y + 1, m.z + 1));
      if (m.motion) {
        box.expandByPoint(new THREE.Vector3(m.motion.fx, m.motion.fy, m.motion.fz));
        box.expandByPoint(
          new THREE.Vector3(m.motion.fx + 1, m.motion.fy + 1, m.motion.fz + 1),
        );
      }
    }
    const center = fitCamera(camera, box);
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.target.copy(center);
    controls.enableDamping = true;
    controls.dampingFactor = 0.1;
    controls.update();

    // Ground grid at the base level, native-style. Its lines have to fall on
    // the block boundaries, which are the integers: a grid square is a cell
    // footprint, and a reader uses it to count blocks off the floor.
    //
    // A GridHelper puts its lines at `-size/2 + i`, so where they land depends
    // on the PARITY of the size as much as on the centre. An even size gives
    // integer offsets and so needs an integer centre; an odd size gives
    // half-integer offsets and needs a half-integer one. The old centre was
    // `(box.min + box.max) / 2`, and box.max is a max corner PLUS ONE, so that
    // landed on a half-integer for half of all builds — the grid was aligned by
    // coincidence. Forcing the size even and the centre to a whole block makes
    // it aligned by construction, at any build size.
    let span = Math.ceil(
      Math.max(box.max.x - box.min.x, box.max.z - box.min.z) + 4,
    );
    if (span % 2 !== 0) span += 1;
    const grid = new THREE.GridHelper(span, span, 0x888888, 0x888888);
    const gridMat = grid.material as THREE.Material;
    gridMat.transparent = true;
    gridMat.opacity = 0.18;
    grid.position.set(
      Math.round((box.min.x + box.max.x) / 2),
      box.min.y + 0.001,
      Math.round((box.min.z + box.max.z) / 2),
    );
    scene.add(grid);

    // Measured lever cell, seal red.
    const leverBox = new THREE.LineSegments(
      new THREE.EdgesGeometry(new THREE.BoxGeometry(1.02, 1.02, 1.02)),
      new THREE.LineBasicMaterial({ color: 0xb3282d }),
    );
    leverBox.position.set(lever[0] + 0.5, lever[1] + 0.5, lever[2] + 0.5);
    scene.add(leverBox);

    /* -------------------------------------------------------- doorway --- */

    // The derived doorway, drawn from the very cells the timing was taken
    // over. Two marks, nested: the passage as a wireframe LATTICE on the cell
    // boundaries (the hole), and the closed pattern as HATCHED panes inset
    // inside it (the blocks that fill the hole). Because the panes are inset,
    // the violet edge survives all the way around each filled cell and the
    // subset relationship is visible rather than covered up.
    //
    // Both are static: built once, posed once, and never touched again by the
    // frame loop. Two extra draw calls, no per-frame work.
    let doorway: THREE.Group | null = null;
    let hatch: THREE.Texture | null = null;
    let edgeGeo: THREE.BufferGeometry | null = null;
    let paneGeo: THREE.BufferGeometry | null = null;
    let coreGeo: THREE.BufferGeometry | null = null;
    const edgeMat = new THREE.LineBasicMaterial({
      transparent: true,
      opacity: 0.92,
      // Drawn THROUGH the build on purpose. The doorway is a derivation, not
      // a block: burying it inside an opaque wall would answer "what did the
      // classifier find?" only on the ticks the door happens to be open.
      depthTest: false,
      depthWrite: false,
      toneMapped: false,
    });
    const paneMat = new THREE.MeshBasicMaterial({
      transparent: true,
      opacity: PANE_ALPHA,
      depthTest: false,
      depthWrite: false,
      side: THREE.DoubleSide,
      toneMapped: false,
    });
    // Carriers: same hue, no hatch, small and dim. A door block that faces
    // nobody should recede behind the ones that do.
    const coreMat = new THREE.MeshBasicMaterial({
      transparent: true,
      opacity: CARRIER_ALPHA,
      depthTest: false,
      depthWrite: false,
      toneMapped: false,
    });
    const instances = (cells: Vec3[], geo: THREE.BufferGeometry, mat: THREE.Material) => {
      const mesh = new THREE.InstancedMesh(geo, mat, cells.length);
      mesh.frustumCulled = false;
      mesh.renderOrder = 3;
      const o = new THREE.Object3D();
      cells.forEach((p, i) => {
        // Cell CENTRE — the same corner→centre convention the meshed blocks
        // are placed with, so a pane lands exactly on the block it marks.
        o.position.set(p[0] + 0.5, p[1] + 0.5, p[2] + 0.5);
        o.updateMatrix();
        mesh.setMatrixAt(i, o.matrix);
      });
      mesh.instanceMatrix.needsUpdate = true;
      return mesh;
    };
    if (geometry && facts) {
      doorway = new THREE.Group();
      doorway.visible = false;
      // Under the flares (renderOrder 4) so an update landing in the doorway
      // still reads as an update. The hatch is what makes that survivable:
      // half the pane is holes, so a flare behind it is never fully masked.
      doorway.renderOrder = 3;

      edgeGeo = passageEdges(geometry.passage);
      const edges = new THREE.LineSegments(edgeGeo, edgeMat);
      edges.frustumCulled = false;
      edges.renderOrder = 3;
      doorway.add(edges);

      if (geometry.visible.length > 0) {
        hatch = hatchTexture();
        paneMat.map = hatch;
        paneGeo = new THREE.BoxGeometry(PANE, PANE, PANE);
        doorway.add(instances(geometry.visible, paneGeo, paneMat));
      }
      if (geometry.carriers.length > 0) {
        coreGeo = new THREE.BoxGeometry(CORE, CORE, CORE);
        const cores = instances(geometry.carriers, coreGeo, coreMat);
        // A carrier is BEHIND the pattern block that hides it — that is the
        // definition — so with everything depth-tested off and sorted back to
        // front, the pane in front would always paint over it and the third
        // mark would be invisible on exactly the doors it exists for. Drawn
        // after the panes and still under the flares, so the pointing-out
        // works without the overlay outranking an update.
        cores.renderOrder = 3.5;
        doorway.add(cores);
      }
      scene.add(doorway);
    }

    /* ---------------------------------------------------- dead weight --- */

    // A small achromatic dot in every block that neither moved nor received
    // an update. Depth-tested off like the doorway marks: dead weight is
    // usually buried inside the machine, and a mark you can only see when the
    // camera happens to have line of sight answers nothing.
    let idleMesh: THREE.InstancedMesh | null = null;
    let idleGeo: THREE.BufferGeometry | null = null;
    const idleMat = new THREE.MeshBasicMaterial({
      transparent: true,
      opacity: 0.85,
      depthTest: false,
      depthWrite: false,
      toneMapped: false,
    });
    if (idleCells) {
      idleGeo = new THREE.BoxGeometry(IDLE_DOT, IDLE_DOT, IDLE_DOT);
      idleMesh = instances(idleCells, idleGeo, idleMat);
      idleMesh.visible = false;
      // Below the doorway marks: a block can be both idle and part of the
      // frame the doorway outlines, and the doorway is the louder answer.
      idleMesh.renderOrder = 2.5;
      scene.add(idleMesh);
    }

    /** Repaint for the current plate. Called on theme change and on every
     * x-ray toggle, because the x-ray's darkroom is dark in both themes. */
    const paintDoorway = (isDark: boolean, xrayNow: boolean) => {
      const [passage, pattern, carrier] = doorwayStyles(isDark || xrayNow);
      edgeMat.color.setHex(passage.hex);
      paneMat.color.setHex(pattern.hex);
      coreMat.color.setHex(carrier.hex);
      paneMat.opacity = xrayNow ? PANE_ALPHA_XRAY : PANE_ALPHA;
      coreMat.opacity = xrayNow ? CARRIER_ALPHA_XRAY : CARRIER_ALPHA;
      idleMat.color.setHex(idleStyle(isDark || xrayNow).hex);
    };
    paintDoorway(pageIsDark(), false);
    paintRef.current = paintDoorway;

    doorwayRef.current = (on: boolean) => {
      if (doorway) doorway.visible = on;
    };
    idleRef.current = (on: boolean) => {
      if (idleMesh) idleMesh.visible = on;
    };

    // Verification aid (scripts/verify-doorway.mjs): frame the doorway so a
    // screenshot can show the hole itself rather than the whole machine.
    focusRef.current = () => {
      if (!facts) return;
      const c = new THREE.Vector3(
        (facts.min[0] + facts.max[0]) / 2 + 0.5,
        (facts.min[1] + facts.max[1]) / 2 + 0.5,
        (facts.min[2] + facts.max[2]) / 2 + 0.5,
      );
      const r = Math.max(facts.w, facts.h, facts.depth);
      controls.target.copy(c);
      camera.position.set(c.x + r * 1.5, c.y + r * 0.85, c.z + r * 1.9);
      camera.lookAt(c);
      controls.update();
    };

    // One group per cast member; real models attach as meshing resolves.
    //
    // CELL_MID is the half-block that reconciles two conventions. A schematic
    // block at (x, y, z) occupies the unit cube from that CORNER — that is what
    // every other mark on this stage assumes (the passage shell, the door-block
    // panes, the flares, the lever box, the world bounds above). But the mesher
    // returns each block's GLB centred on its own origin, spanning -0.5..+0.5,
    // so a group parked at the raw corner draws the block half a cell low, left
    // and back of the cell it belongs to, and every overlay reads as offset by
    // (0.5, 0.5, 0.5). Placing the group at the cell CENTRE puts the meshed
    // block back in [x, x+1] where the rest of the scene expects it.
    //
    // The piston-arm clip planes in `headClipHalfSpace` are world-space and
    // already written in corner terms (`headBase + 0.75`), so they come right
    // with this too rather than needing their own shift.
    const CELL_MID = 0.5;
    const groups: THREE.Group[] = cast.map((m) => {
      const g = new THREE.Group();
      g.position.set(m.x + CELL_MID, m.y + CELL_MID, m.z + CELL_MID);
      g.visible = false;
      scene.add(g);
      return g;
    });
    let disposed = false;
    let pending = cast.length;
    cast.forEach((m, i) => {
      instanceFor(m.state)
        .then((inst) => {
          if (disposed) return;
          const clip = headClipHalfSpace(m);
          if (clip) {
            // Clone materials for this instance only (the cache shares
            // them) and confine the vanilla model's fixed-length arm to the
            // front of its base cell.
            const plane = new THREE.Plane(
              new THREE.Vector3(clip.nx, clip.ny, clip.nz),
              -clip.min,
            );
            inst.traverse((o: THREE.Object3D) => {
              const mesh = o as THREE.Mesh;
              if (!mesh.isMesh) return;
              const mat = (mesh.material as THREE.Material).clone();
              mat.clippingPlanes = [plane];
              mesh.material = mat;
            });
          }
          groups[i].add(inst);
          // A model that lands after the mode was switched on still has to
          // become a ghost.
          if (xrayOnRef.current) applyXray(true);
        })
        .catch((e) => {
          // A state the pack can't mesh: leave the slot empty, keep going.
          console.warn("mesh failed for", m.state, e);
        })
        .finally(() => {
          pending -= 1;
          if (pending === 0 && !disposed) {
            setReady(true);
            console.info(
              `[mesh] ${meshStats.states} unique states, ` +
                `${(meshStats.totalMs / Math.max(meshStats.states, 1)).toFixed(1)} ms avg`,
            );
          }
        });
    });

    /* ---------------------------------------------------------- x-ray --- */

    const X = xray;
    const cellN = X ? X.cells.length / 3 : 0;
    // Per-cell flare energy and the channel category that last lit it.
    const energy = new Float32Array(cellN);
    const slot = new Int16Array(cellN);
    const dirty = new Uint8Array(cellN);
    const dirtyList: number[] = [];
    const heatByTick = new Map<number, number>();
    if (X) X.ticks.forEach((t, i) => heatByTick.set(t.tick, i));
    // Which cells the build's own blocks occupy — an inert block recedes.
    const litCells = new Set<string>();
    if (X)
      for (let i = 0; i < cellN; i++)
        litCells.add(`${X.cells[i * 3]},${X.cells[i * 3 + 1]},${X.cells[i * 3 + 2]}`);

    const flareGeo = new THREE.BoxGeometry(1, 1, 1);
    const cageGeo = cageGeometry();
    // Flares are OPAQUE, depth-tested marks — not emissive glow. Both
    // blending modes were built and compared on the busiest tick: additive
    // clips a five-deep stack to white, and `max(src, dst)` keeps brightness
    // but invents hues at overlaps (an orange flare behind a blue one comes
    // out magenta, a colour the legend does not have). A depth-tested mark
    // can only ever be one of the legend's colours, which is the property
    // this view cannot trade away. The marks stay well under a full cell, so
    // the lattice is still see-through and the interior reads.
    const flareMatOf = () =>
      new THREE.MeshBasicMaterial({ toneMapped: false });
    const flares = new THREE.InstancedMesh(flareGeo, flareMatOf(), Math.max(1, cellN));
    const cages = new THREE.InstancedMesh(cageGeo, flareMatOf(), Math.max(1, cellN));
    for (const m of [flares, cages]) {
      m.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      m.frustumCulled = false;
      m.renderOrder = 4;
      m.visible = false;
      m.count = 0;
      scene.add(m);
    }
    // Allocate the per-instance colour buffers up front.
    const seed = new THREE.Color(1, 1, 1);
    flares.setColorAt(0, seed);
    cages.setColorAt(0, seed);

    const tables = {
      phase: X ? colourTable(X, "phase") : null,
      kind: X ? colourTable(X, "kind") : null,
    };
    const dummy = new THREE.Object3D();
    const col = new THREE.Color();

    /** Push the current `energy`/`slot` state into the two instanced meshes.
     * `ref` normalises intensity — the 95th-percentile cell count for tick
     * playback, a small constant for the sub-tick sweep. */
    const writeFlares = (ref: number) => {
      const table = tables[channelRef.current];
      if (!table || !X) return;
      const logRef = Math.log1p(ref);
      let gi = 0;
      let ci = 0;
      for (let i = 0; i < cellN; i++) {
        const e = energy[i];
        if (e < 0.015) continue;
        const s = slot[i];
        if (s < 0) continue;
        const a = Math.min(1, Math.log1p(e) / logRef);
        const isCage = table.cage[s] === 1;
        // Weak flares are small as well as dim, so a wavefront has an edge.
        // The cap stays well under a full cell: additive marks that touch
        // merge into one mass and the hue — the whole point of the view —
        // washes to white.
        dummy.scale.setScalar(isCage ? 1.04 : 0.24 + 0.52 * a);
        dummy.position.set(
          X.cells[i * 3] + 0.5,
          X.cells[i * 3 + 1] + 0.5,
          X.cells[i * 3 + 2] + 0.5,
        );
        dummy.updateMatrix();
        // Intensity rides on size AND brightness; under MAX blending the
        // brightest mark wins outright, so a full-strength flare is the
        // palette hue itself and a faint one is a dim step of it.
        const g = 0.3 + 0.7 * a;
        col.setRGB(
          table.cols[s * 3] * g,
          table.cols[s * 3 + 1] * g,
          table.cols[s * 3 + 2] * g,
        );
        if (isCage) {
          cages.setMatrixAt(ci, dummy.matrix);
          cages.setColorAt(ci, col);
          ci++;
        } else {
          flares.setMatrixAt(gi, dummy.matrix);
          flares.setColorAt(gi, col);
          gi++;
        }
      }
      flares.count = gi;
      cages.count = ci;
      flares.instanceMatrix.needsUpdate = true;
      cages.instanceMatrix.needsUpdate = true;
      if (flares.instanceColor) flares.instanceColor.needsUpdate = true;
      if (cages.instanceColor) cages.instanceColor.needsUpdate = true;
      (window as unknown as Record<string, unknown>).__xrayFlares = gi + ci;
    };

    /** Dominant category of a heat cell — the phase (or kind) that delivered
     * most of that cell's updates this tick. A cell can be told by several
     * phases in one tick; the sub-tick readout is where the exact answer
     * lives, and this is the summary. */
    const heatSlot = (t: XrayHeatTick, i: number, P: number) => {
      if (channelRef.current === "kind") return t.nb[i] >= t.sh[i] ? 0 : 1;
      let best = 0;
      let bv = -1;
      for (let p = 0; p < P; p++) {
        const v = t.ph[i * P + p];
        if (v > bv) {
          bv = v;
          best = p;
        }
      }
      return best;
    };

    const injectTick = (k: number) => {
      const idx = heatByTick.get(k);
      if (idx === undefined || !X) return;
      const t = X.ticks[idx];
      const P = X.phases.length;
      for (let i = 0; i < t.cell.length; i++) {
        const c = t.cell[i];
        energy[c] += t.n[i];
        slot[c] = heatSlot(t, i, P);
      }
    };

    let lastHeatTick = -1;
    const drawHeat = (t: number, dt: number) => {
      if (!X) return;
      const ti = Math.max(0, Math.min(X.ticks.length - 1, Math.floor(t)));
      if (!playingRef.current) {
        // Paused: hold the selected tick's heat instead of letting the decay
        // tail run to black. A paused x-ray has to show something.
        energy.fill(0);
        injectTick(ti);
        lastHeatTick = ti;
        writeFlares(X.heatRef);
        return;
      }
      if (ti !== lastHeatTick) {
        if (ti < lastHeatTick) {
          // Scrubbed backwards: the decay tail is meaningless, start clean.
          energy.fill(0);
          lastHeatTick = ti - 1;
        }
        for (let k = lastHeatTick + 1; k <= ti; k++) injectTick(k);
        lastHeatTick = ti;
      }
      const decay = Math.exp(-dt / FLARE_TAU);
      for (let i = 0; i < cellN; i++) energy[i] *= decay;
      writeFlares(X.heatRef);
    };

    const drawWave = (dt: number) => {
      if (!X) return;
      const w = X.waves[subTickRef.current];
      if (!w || w.n === 0) {
        flares.count = 0;
        cages.count = 0;
        if (seqRef.current) seqRef.current.textContent = "no updates on this tick";
        if (seqStateRef.current) seqStateRef.current.textContent = "";
        return;
      }
      if (subPlayingRef.current) {
        cursorRef.current += (dt * w.n * subSpeedRef.current) / SWEEP_SEC;
        if (cursorRef.current >= w.n) cursorRef.current = 0;
      }
      const c = Math.max(0, Math.min(w.n - 1, Math.floor(cursorRef.current)));
      // A trailing window, not a single update: at 19,834 updates a tick one
      // update is a pixel, and the wavefront is what carries the meaning. The
      // cap is what keeps it a FRONT — a door has only ~1,700 cells that ever
      // receive an update, so a window much past ~250 lights most of the build
      // at once and the motion stops reading.
      const win = Math.max(4, Math.min(260, Math.round(w.n * 0.013)));
      for (const i of dirtyList) {
        energy[i] = 0;
        dirty[i] = 0;
      }
      dirtyList.length = 0;
      const lo = Math.max(0, c - win + 1);
      for (let i = lo; i <= c; i++) {
        const cell = w.cell[i];
        const age = (c - i) / win;
        const r = 1 - age;
        // Cubic falloff: the newest updates are the front, the rest is wake.
        const k = r * r * r;
        if (!dirty[cell]) {
          dirty[cell] = 1;
          dirtyList.push(cell);
        }
        energy[cell] = Math.min(6, energy[cell] + k * 2.4);
        slot[cell] = channelRef.current === "kind" ? w.kind[i] : w.phase[i];
      }
      writeFlares(3);

      if (seqRangeRef.current) seqRangeRef.current.value = String(c);
      if (seqRef.current) {
        const ph = X.phases[w.phase[c]] ?? "—";
        const kd = X.kinds[w.kind[c]] === "shape" ? "shape" : "neighbour";
        const d = w.from[c] < X.dirs.length ? `from ${X.dirs[w.from[c]]}` : "no source";
        seqRef.current.textContent =
          `update ${fmt(c + 1)} / ${fmt(w.n)} · phase ${ph} · ${kd} · ${d}`;
      }
      if (seqStateRef.current) {
        const cell = w.cell[c];
        seqStateRef.current.textContent =
          `${w.states[w.state[c]] ?? "?"} at ` +
          `(${X.cells[cell * 3]}, ${X.cells[cell * 3 + 1]}, ${X.cells[cell * 3 + 2]})`;
      }
    };

    /** Enter/leave the ghost. Materials are cloned once per mesh and both
     * sets kept, so the toggle after that is a pointer swap — no reload, no
     * lost playback position. */
    const applyXray = (on: boolean) => {
      renderer.setClearColor(on ? XRAY_SURFACE : 0x000000, on ? 1 : 0);
      gridMat.opacity = on ? 0.05 : 0.18;
      for (let i = 0; i < groups.length; i++) {
        const m = cast[i];
        const live = litCells.has(`${m.x},${m.y},${m.z}`);
        groups[i].traverse((o: THREE.Object3D) => {
          const mesh = o as THREE.Mesh;
          if (!mesh.isMesh) return;
          const ud = mesh.userData as {
            normMat?: THREE.Material;
            xrayMat?: THREE.Material;
            normOrder?: number;
          };
          if (!ud.normMat) {
            ud.normMat = mesh.material as THREE.Material;
            ud.normOrder = mesh.renderOrder;
          }
          if (on) {
            if (!ud.xrayMat) {
              const xm = ud.normMat.clone() as THREE.MeshStandardMaterial;
              xm.transparent = true;
              xm.depthWrite = false;
              // A cutout material tests alpha AFTER opacity, so leaving the
              // 0.5 test on would discard the entire ghost.
              xm.alphaTest = 0;
              xm.opacity = live ? GHOST_LIVE : GHOST_INERT;
              xm.color.multiplyScalar(GHOST_TINT);
              ud.xrayMat = xm;
            }
            mesh.material = ud.xrayMat;
            mesh.renderOrder = 2;
          } else {
            mesh.material = ud.normMat;
            mesh.renderOrder = ud.normOrder ?? 0;
          }
        });
      }
      flares.visible = on;
      cages.visible = on;
      if (!on) {
        flares.count = 0;
        cages.count = 0;
      } else {
        energy.fill(0);
        lastHeatTick = -1;
      }
    };
    applyRef.current = applyXray;

    const stats = makeFrameStats();
    // Verification aid (scripts/verify-xray.mjs): payload sizes as the engine
    // emitted them, plus a way to measure a clean window of frame times.
    const w = window as unknown as Record<string, unknown>;
    w.__replayFrameReset = () => stats.reset();
    // Verification aid (scripts/verify-doorway.mjs): the counts the legend is
    // drawn from, so a test can prove the picture and the certificate's
    // aperture line came off the same cells.
    w.__doorway = facts
      ? { ...facts, focus: () => focusRef.current?.() }
      : null;
    // Same, for the dead-weight overlay: how many dots were instanced, so a
    // test can prove the picture matches the certificate's idle count.
    w.__idle = idleMesh ? { drawn: idleMesh.count } : null;
    // Verification aid (scripts/verify-align.mjs): the numeric half-block
    // check. For a named world cell, report where the MESHED block actually
    // sits and where the overlay drew its passage box, both as world-space
    // centres. The two must agree to within a rounding error; a 0.5 gap on
    // any axis is the bug this hook exists to catch.
    w.__align = (cell?: Vec3) => {
      const target = cell ?? (geometry?.passage?.[0] as Vec3 | undefined);
      if (!target) return null;
      // A member with `motion` is parked at its FROM cell for most of the
      // stroke, so its group is not over `target` at an arbitrary tick and its
      // box says nothing about the convention. Report it and let the caller
      // skip it rather than silently measuring the wrong block.
      const i = cast.findIndex(
        (m) => m.x === target[0] && m.y === target[1] && m.z === target[2],
      );
      const member = i >= 0 ? cast[i] : null;
      const bb = new THREE.Box3();
      let groupPos: number[] | null = null;
      if (i >= 0) {
        groups[i].updateMatrixWorld(true);
        groupPos = groups[i].position.toArray();
        if (groups[i].children.length) bb.setFromObject(groups[i]);
      }
      // Where the OVERLAY drew this cell. Read off the drawn instances rather
      // than recomputed, so the check is against what is actually on screen.
      // The passage shell culls every face with another passage cell behind
      // it, so an interior cell has no edge geometry at all — the pane/core
      // instance is the mark that is always there.
      let overlayCentre: number[] | null = null;
      const mid = new THREE.Vector3();
      const mat4 = new THREE.Matrix4();
      doorway?.traverse((o: THREE.Object3D) => {
        const im = o as THREE.InstancedMesh;
        if (!im.isInstancedMesh || overlayCentre) return;
        for (let k = 0; k < im.count; k++) {
          im.getMatrixAt(k, mat4);
          mid.setFromMatrixPosition(mat4);
          if (
            Math.abs(mid.x - (target[0] + 0.5)) < 0.01 &&
            Math.abs(mid.y - (target[1] + 0.5)) < 0.01 &&
            Math.abs(mid.z - (target[2] + 0.5)) < 0.01
          ) {
            overlayCentre = mid.toArray();
            return;
          }
        }
      });
      const gridLines: number[] = [];
      {
        const gp = (grid.geometry as THREE.BufferGeometry).getAttribute("position");
        const seen2 = new Set<number>();
        for (let v = 0; v < gp.count; v++) seen2.add(+(gp.getX(v) + grid.position.x).toFixed(4));
        gridLines.push(...[...seen2].sort((a, b) => a - b));
      }
      return {
        cell: target,
        state: member?.state ?? null,
        /** True while this block is being carried by a piston: its group is
         *  somewhere between two cells and its box is not a convention test. */
        moving: !!member?.motion,
        /** Where the group itself sits — must be the cell CENTRE. */
        groupPos,
        // Where the mesher put the block: its own world bounding box.
        meshMin: i >= 0 && !bb.isEmpty() ? bb.min.toArray() : null,
        meshMax: i >= 0 && !bb.isEmpty() ? bb.max.toArray() : null,
        // Centre of the mark the overlay drew on the same cell.
        overlayCentre,
        // Distinct x-coordinates of the floor grid's lines, in world space.
        gridX: gridLines,
      };
    };
    // …and a sample of blocks that never move, whose meshes therefore pin the
    // corner convention exactly rather than being caught mid-stroke.
    w.__alignStatic = (want = 6) => {
      const out: unknown[] = [];
      const bb = new THREE.Box3();
      for (let i = 0; i < cast.length && out.length < want; i++) {
        const m = cast[i];
        if (m.motion || !groups[i].children.length) continue;
        groups[i].updateMatrixWorld(true);
        bb.makeEmpty();
        bb.setFromObject(groups[i]);
        if (bb.isEmpty()) continue;
        out.push({
          cell: [m.x, m.y, m.z],
          state: m.state,
          meshMin: bb.min.toArray(),
          meshMax: bb.max.toArray(),
        });
      }
      return out;
    };
    w.__xray = X
      ? {
          heatBytes: X.bytes.heat,
          waveBytes: X.bytes.waves,
          cells: cellN,
          ticks: X.ticks.length,
          totalUpdates: X.totalUpdates,
          heatRef: X.heatRef,
          phases: X.phases,
          kinds: X.kinds,
          updatesPerTick: X.waves.map((v) => v.n),
        }
      : null;
    const members: CastMember[] = cast;
    let raf = 0;
    let last = performance.now();
    const step = (now: number) => {
      raf = requestAnimationFrame(step);
      const dt = Math.min((now - last) / 1000, 0.25);
      stats.push(now - last);
      last = now;
      if (playingRef.current) {
        let t = timeRef.current + dt * 20 * speedRef.current;
        if (t >= simTicks) t = 0;
        timeRef.current = t;
        if (rangeRef.current) rangeRef.current.value = String(t);
      }
      const t = timeRef.current;
      if (readoutRef.current) {
        readoutRef.current.textContent = String(Math.floor(t)).padStart(3, "0");
      }
      for (let i = 0; i < members.length; i++) {
        const m = members[i];
        const g = groups[i];
        const vis = t >= m.start && t < m.end;
        g.visible = vis;
        if (!vis) continue;
        if (m.motion) {
          const span2 = m.motion.until - m.start;
          const progress =
            span2 > 0 ? Math.min(1, Math.max(0, (t - m.start) / span2)) : 1;
          const remaining = 1 - progress;
          // Same corner→centre shift as the initial pose above; the lerp is
          // between two cell corners, so the half-block rides on the result.
          g.position.set(
            m.x + (m.motion.fx - m.x) * remaining + CELL_MID,
            m.y + (m.motion.fy - m.y) * remaining + CELL_MID,
            m.z + (m.motion.fz - m.z) * remaining + CELL_MID,
          );
        }
      }
      if (xrayOnRef.current && X) {
        if (subOnRef.current) drawWave(dt);
        else drawHeat(t, dt);
      }
      controls.update();
      renderer.render(scene, camera);
    };
    raf = requestAnimationFrame(step);

    const ro = new ResizeObserver(() => {
      const w = mount.clientWidth;
      renderer.setSize(w, STAGE_H);
      camera.aspect = w / STAGE_H;
      camera.updateProjectionMatrix();
    });
    ro.observe(mount);

    return () => {
      disposed = true;
      applyRef.current = null;
      doorwayRef.current = null;
      idleRef.current = null;
      paintRef.current = null;
      focusRef.current = null;
      ro.disconnect();
      cancelAnimationFrame(raf);
      controls.dispose();
      edgeGeo?.dispose();
      paneGeo?.dispose();
      coreGeo?.dispose();
      hatch?.dispose();
      idleGeo?.dispose();
      edgeMat.dispose();
      paneMat.dispose();
      coreMat.dispose();
      idleMat.dispose();
      flareGeo.dispose();
      cageGeo.dispose();
      (flares.material as THREE.Material).dispose();
      (cages.material as THREE.Material).dispose();
      renderer.domElement.removeEventListener("webglcontextlost", onLost);
      renderer.dispose();
      renderer.forceContextLoss();
      mount.removeChild(renderer.domElement);
    };
  }, [cast, lever, simTicks, onFail, glNonce, xray, geometry, facts, idleCells]);

  // Mode changes are applied to the LIVE scene rather than rebuilding it, so
  // the x-ray toggle keeps the camera, the tick and the play state.
  useEffect(() => {
    xrayOnRef.current = xrayOn;
    applyRef.current?.(xrayOn);
  }, [xrayOn, glNonce, cast, xray]);
  // The overlay is a visibility flip on a pre-built group, so it survives the
  // x-ray toggle in either order and costs nothing to leave on.
  useEffect(() => {
    doorwayRef.current?.(doorwayOn);
  }, [doorwayOn, glNonce, geometry, facts]);
  useEffect(() => {
    idleRef.current?.(idleOn);
  }, [idleOn, glNonce, idleCells]);
  useEffect(() => {
    paintRef.current?.(dark, xrayOn);
  }, [dark, xrayOn, glNonce, geometry, facts]);
  useEffect(() => {
    channelRef.current = channel;
  }, [channel]);
  useEffect(() => {
    subOnRef.current = subOn;
  }, [subOn]);
  useEffect(() => {
    subTickRef.current = subTick;
  }, [subTick]);
  useEffect(() => {
    subPlayingRef.current = subPlaying;
  }, [subPlaying]);
  useEffect(() => {
    subSpeedRef.current = subSpeed;
  }, [subSpeed]);

  const enterSubTick = (tick: number) => {
    const t = Math.max(0, Math.min(simTicks - 1, tick));
    playingRef.current = false;
    setPlaying(false);
    timeRef.current = t;
    if (rangeRef.current) rangeRef.current.value = String(t);
    cursorRef.current = 0;
    setSubTick(t);
    subTickRef.current = t;
    setSubOn(true);
    subOnRef.current = true;
    setSubPlaying(true);
    subPlayingRef.current = true;
  };

  const nudge = (by: number) => {
    if (!wave || wave.n === 0) return;
    setSubPlaying(false);
    subPlayingRef.current = false;
    cursorRef.current = Math.max(0, Math.min(wave.n - 1, Math.floor(cursorRef.current) + by));
  };

  const legend = xray ? channelStyles(xray, channel) : [];
  const [passageStyle, patternStyle, carrierStyle] = doorwayStyles(dark || xrayOn);
  const idleDotStyle = idleStyle(dark || xrayOn);

  return (
    <div>
      <div
        className={"replay-stage replay-stage-webgl" + (xrayOn ? " xray" : "")}
        ref={mountRef}
        style={{ position: "relative", height: STAGE_H }}
        data-testid="mesh-replay-stage"
        data-ready={ready ? "1" : "0"}
        data-xray={xrayOn ? "1" : "0"}
      >
        {!ready && (
          <div
            style={{
              position: "absolute",
              inset: 0,
              display: "grid",
              placeItems: "center",
              pointerEvents: "none",
              fontSize: 13,
              opacity: 0.7,
            }}
          >
            meshing block models…
          </div>
        )}
      </div>

      <div className="replay-controls">
        <button
          className="replay-btn"
          onClick={() =>
            setPlaying((p) => {
              playingRef.current = !p;
              if (!p) {
                setSubOn(false);
                subOnRef.current = false;
              }
              return !p;
            })
          }
          aria-label={playing ? "Pause replay" : "Play replay"}
        >
          {playing ? "❚❚" : "▶"}
        </button>
        <span className="replay-readout">
          tick <b ref={readoutRef}>000</b>/{simTicks}
        </span>
        <div className="replay-track">
          <input
            ref={rangeRef}
            type="range"
            min={0}
            max={simTicks}
            step={0.05}
            defaultValue={0}
            aria-label="Replay tick"
            onChange={(e) => {
              playingRef.current = false;
              setPlaying(false);
              setSubOn(false);
              subOnRef.current = false;
              timeRef.current = Number(e.target.value);
            }}
          />
          <div className="replay-marks" aria-hidden>
            {replay.flips.map((f) => (
              <i
                key={f.tick + f.label}
                className={"replay-mark" + (f.measured ? " measured" : "")}
                style={{ left: `${(f.tick / simTicks) * 100}%` }}
                title={`${f.label} · t=${f.tick}`}
              />
            ))}
          </div>
        </div>
        <span className="replay-rate">playback ×1 = in-game speed</span>
        <div className="replay-speeds" role="group" aria-label="Playback speed">
          {SPEEDS.map((s) => (
            <button
              key={s}
              type="button"
              className={"replay-speed" + (s === speed ? " on" : "")}
              aria-pressed={s === speed}
              onClick={() => {
                speedRef.current = s;
                setSpeed(s);
              }}
            >
              {s}×
            </button>
          ))}
        </div>
        {/* The three overlays travel together at the right of the control bar:
            loose, they wrap one-at-a-time as the bar narrows and the group
            reads as three unrelated buttons on two lines. */}
        <div className="replay-modes">
        <button
          type="button"
          className={"door-toggle" + (doorwayOn ? " on" : "")}
          aria-pressed={doorwayOn}
          disabled={!facts}
          data-testid="doorway-toggle"
          title={
            facts
              ? "Outline the doorway the classifier derived"
              : "No walkable passage was extracted from this door"
          }
          onClick={() => setDoorwayOn((v) => !v)}
        >
          <i className="door-dot" aria-hidden />
          Doorway
        </button>
        <button
          type="button"
          className={"idle-toggle" + (idleOn ? " on" : "")}
          aria-pressed={idleOn}
          disabled={!idleCells}
          data-testid="idle-toggle"
          title={
            idleCells
              ? "Mark the blocks that neither moved nor took an update this cycle"
              : "Every block in this build did something this cycle"
          }
          onClick={() => setIdleOn((v) => !v)}
        >
          <i className="idle-dot" aria-hidden />
          Dead weight
        </button>
        <button
          type="button"
          className={"xray-toggle" + (xrayOn ? " on" : "")}
          aria-pressed={xrayOn}
          disabled={!xray}
          data-testid="xray-toggle"
          title={
            xray
              ? "Show the recorded redstone update stream"
              : "Re-run this door to record its update stream"
          }
          onClick={() => {
            setXrayOn((v) => {
              if (v) {
                setSubOn(false);
                subOnRef.current = false;
              }
              return !v;
            });
          }}
        >
          <i className="xray-dot" aria-hidden />
          X-ray
        </button>
        </div>
      </div>

      {doorwayOn && facts && (
        <div
          className={"door-panel" + (xrayOn ? " xr" : "")}
          data-testid="doorway-panel"
        >
          <span className="xray-label">doorway</span>
          <ul className="door-legend" data-testid="doorway-legend">
            <li>
              <i
                className="door-swatch edges"
                style={{ borderColor: hexCss(passageStyle.hex) }}
                aria-hidden
              />
              {passageStyle.label}
              <b>{fmt(facts.passageCells)}</b>
            </li>
            <li>
              <i
                className="door-swatch hatch"
                style={{
                  // Same 45° hatch as the panes, at swatch scale.
                  backgroundImage:
                    `repeating-linear-gradient(45deg, ${hexCss(patternStyle.hex)} 0 2px,` +
                    ` transparent 2px 4px)`,
                  borderColor: hexCss(patternStyle.hex),
                }}
                aria-hidden
              />
              {patternStyle.label}
              <b>{fmt(facts.visibleCells)}</b>
            </li>
            {facts.carrierCells > 0 && (
              <li>
                <i
                  className="door-swatch core"
                  style={{ ["--core" as string]: hexCss(carrierStyle.hex) }}
                  aria-hidden
                />
                {carrierStyle.label}
                <b>{fmt(facts.carrierCells)}</b>
              </li>
            )}
          </ul>
          <span className="door-summary" data-testid="doorway-summary">
            {doorwaySummary(facts)}
          </span>
          {!facts.rectangular && (
            <span className="door-warn" data-testid="doorway-ragged">
              the passage does not fill its own {facts.w} × {facts.h} × {facts.depth}
              {" "}box — it is not a clean rectangular opening
            </span>
          )}
        </div>
      )}

      {idleOn && idleCells && (
        <div
          className={"door-panel" + (xrayOn ? " xr" : "")}
          data-testid="idle-panel"
        >
          <span className="xray-label">dead weight</span>
          <ul className="door-legend" data-testid="idle-legend">
            <li>
              <i
                className="door-swatch dot"
                style={{ ["--core" as string]: hexCss(idleDotStyle.hex) }}
                aria-hidden
              />
              {idleDotStyle.label}
              <b>{fmt(idleCells.length)}</b>
            </li>
          </ul>
          <span className="door-summary" data-testid="idle-summary">
            no update landed here and nothing moved — decoration, redundancy, or support
          </span>
        </div>
      )}

      {xrayOn && xray && (
        <div className="xray-panel" data-testid="xray-panel">
          <div className="xray-row">
            <span className="xray-label">colour by</span>
            <div className="xray-seg" role="group" aria-label="Colour channel">
              {(["phase", "kind"] as Channel[]).map((c) => (
                <button
                  key={c}
                  type="button"
                  className={"xray-segbtn" + (c === channel ? " on" : "")}
                  aria-pressed={c === channel}
                  data-testid={`xray-channel-${c}`}
                  onClick={() => setChannel(c)}
                >
                  {c === "phase" ? "tick phase" : "update kind"}
                </button>
              ))}
            </div>
            <ul className="xray-legend" data-testid="xray-legend">
              {legend.map((s) => (
                <li key={s.label}>
                  <i
                    className={"xray-swatch" + (s.mark === "cage" ? " cage" : "")}
                    style={
                      s.mark === "cage"
                        ? { borderColor: hexCss(s.hex) }
                        : { background: hexCss(s.hex) }
                    }
                    aria-hidden
                  />
                  {s.label}
                </li>
              ))}
            </ul>
            <span className="xray-note">
              {fmt(xray.totalUpdates)} updates recorded
            </span>
          </div>

          <div className="xray-row">
            {!subOn ? (
              <>
                <button
                  type="button"
                  className="xray-btn"
                  data-testid="xray-enter-subtick"
                  onClick={() => enterSubTick(Math.floor(timeRef.current))}
                >
                  Step inside this tick
                </button>
                <button
                  type="button"
                  className="xray-btn"
                  data-testid="xray-enter-busiest"
                  onClick={() => enterSubTick(busiestTick)}
                >
                  Go to busiest tick ({busiestTick})
                </button>
                <span className="xray-note">
                  Cells flare as updates land on them, brightest where most
                  arrive — including cells that are told and do not react.
                </span>
              </>
            ) : (
              <>
                <div className="xray-tickpick">
                  <button
                    type="button"
                    className="xray-btn"
                    aria-label="Previous tick"
                    onClick={() => enterSubTick(subTick - 1)}
                    disabled={subTick <= 0}
                  >
                    ◀
                  </button>
                  <b>tick {String(subTick).padStart(2, "0")}</b>
                  <button
                    type="button"
                    className="xray-btn"
                    aria-label="Next tick"
                    onClick={() => enterSubTick(subTick + 1)}
                    disabled={subTick >= simTicks - 1}
                  >
                    ▶
                  </button>
                </div>
                <button
                  className="replay-btn"
                  onClick={() =>
                    setSubPlaying((p) => {
                      subPlayingRef.current = !p;
                      return !p;
                    })
                  }
                  aria-label={subPlaying ? "Pause sub-tick sweep" : "Play sub-tick sweep"}
                >
                  {subPlaying ? "❚❚" : "▶"}
                </button>
                <div className="xray-steps" role="group" aria-label="Step through updates">
                  <button
                    type="button"
                    className="xray-btn"
                    onClick={() => nudge(-Math.max(1, Math.round((wave?.n ?? 1) / 100)))}
                    aria-label="Back 1%"
                  >
                    ⏪
                  </button>
                  <button
                    type="button"
                    className="xray-btn"
                    data-testid="xray-step-back"
                    onClick={() => nudge(-1)}
                    aria-label="Previous update"
                  >
                    ◂
                  </button>
                  <button
                    type="button"
                    className="xray-btn"
                    data-testid="xray-step-fwd"
                    onClick={() => nudge(1)}
                    aria-label="Next update"
                  >
                    ▸
                  </button>
                  <button
                    type="button"
                    className="xray-btn"
                    onClick={() => nudge(Math.max(1, Math.round((wave?.n ?? 1) / 100)))}
                    aria-label="Forward 1%"
                  >
                    ⏩
                  </button>
                </div>
                <div className="xray-track">
                  <input
                    ref={seqRangeRef}
                    type="range"
                    min={0}
                    max={Math.max(0, (wave?.n ?? 1) - 1)}
                    step={1}
                    defaultValue={0}
                    aria-label="Update sequence within the tick"
                    data-testid="xray-seq"
                    onChange={(e) => {
                      setSubPlaying(false);
                      subPlayingRef.current = false;
                      cursorRef.current = Number(e.target.value);
                    }}
                  />
                </div>
                <div className="xray-speeds" role="group" aria-label="Sweep speed">
                  {SUB_SPEEDS.map((s) => (
                    <button
                      key={s}
                      type="button"
                      className={"replay-speed" + (s === subSpeed ? " on" : "")}
                      aria-pressed={s === subSpeed}
                      onClick={() => {
                        subSpeedRef.current = s;
                        setSubSpeed(s);
                      }}
                    >
                      {s}×
                    </button>
                  ))}
                </div>
                <button
                  type="button"
                  className="xray-btn"
                  data-testid="xray-exit-subtick"
                  onClick={() => {
                    setSubOn(false);
                    subOnRef.current = false;
                  }}
                >
                  done
                </button>
              </>
            )}
          </div>

          {subOn && (
            <div className="xray-readout" data-testid="xray-readout">
              <b ref={seqRef}>—</b>
              <span ref={seqStateRef} className="xray-readout-state" />
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/** The WebGL mesh replay. The isometric painter is kept only as an automatic
 * fallback for browsers without a usable WebGL context. */
export function MeshReplay({
  replay,
  lever,
  xray = null,
  geometry = null,
  idle = null,
}: {
  replay: Replay;
  lever: Vec3;
  xray?: XrayData | null;
  geometry?: ApertureGeometry | null;
  idle?: Vec3[] | null;
}) {
  const [glFail, setGlFail] = useState<string | null>(null);
  if (glFail !== null) return <VoxelReplay replay={replay} lever={lever} />;
  return (
    <WebglReplay
      replay={replay}
      lever={lever}
      xray={xray}
      geometry={geometry}
      idle={idle}
      onFail={setGlFail}
    />
  );
}
