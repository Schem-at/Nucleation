/** The flight stage — the lab's signature element. Primary view: the
 * champion's machine meshed by nucleation's REAL mesher (wasm → GLB →
 * three.js), animated from the member cast over one detected period.
 * Translate-compensated the native way: the CAMERA advances +dx per period
 * over a static world (blocks + grid), so the machine flies in place while
 * the corridor floor scrolls underneath. Seamless by construction: at wrap
 * the camera jumps -dx exactly when the cast resets -dx, and the 1-block
 * grid is invariant under integer-dx shifts.
 *
 * The classic isometric painter remains as a toggle/fallback and still does
 * GIF export (gifenc reads its 2D canvas). Smoothness: the rAF loop drives a
 * fractional wall-clock tick, poses members in place with zero per-frame
 * allocations, and never touches React state. */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { headClipHalfSpace } from "../cast";
import { downloadBlob, exportLoopGif } from "../gif";
import { drawScene, type CanvasPalette } from "../iso";
import { addDefaultLights, fitCamera, instanceFor, meshStats } from "../mesher";
import { speedOf } from "../metrics";
import { onTexturesChanged } from "../textures";
import type { BestRecord } from "../types";

const TPS = 10; // playback ticks per second (engine flies 1 block / ~10 ticks)
const CSS_H = 320;

function readPalette(): CanvasPalette {
  const cs = getComputedStyle(document.documentElement);
  return {
    page: cs.getPropertyValue("--page").trim() || "#f4f4f1",
    grid: cs.getPropertyValue("--grid-iso").trim() || "rgba(0,0,0,0.06)",
    edge: "rgba(0,0,0,0.25)",
  };
}

interface Props {
  best: BestRecord | null;
  /** True when `best` is the reigning champion (vs a filmstrip pick). */
  isChampion: boolean;
  loading: boolean;
  /** Eval window in ticks — blk/s = fitness / (evalTicks / 20). */
  evalTicks: number | null;
  /** User grabbed the stage's orbit controls — parent pins the machine so a
   * new champion can't swap it mid-drag. */
  onInteract?: () => void;
  /** Chip click: unpin and follow the reigning champion again. */
  onResumeFollow?: () => void;
}

export default function FlightLoop({
  best,
  isChampion,
  loading,
  evalTicks,
  onInteract,
  onResumeFollow,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const glRef = useRef<HTMLDivElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const [exporting, setExporting] = useState(false);
  const [mode, setMode] = useState<"webgl" | "iso">("webgl");
  const [glFail, setGlFail] = useState(false);
  // Bumped on WebGL context loss so the stage rebuilds its renderer.
  const [glNonce, setGlNonce] = useState(0);
  const loop = best?.loop ?? null;
  const onInteractRef = useRef(onInteract);
  onInteractRef.current = onInteract;

  // WebGL needs the member cast (old stored records may predate it).
  const canWebgl = !glFail && !!loop && !!loop.cast && loop.cast.length > 0;
  const webgl = mode === "webgl" && canWebgl;

  /** Scene geometry in compensated space (machine ~static around x=0). */
  const geom = useMemo(() => {
    if (!loop || loop.frames.length === 0) return null;
    let minX = Infinity,
      maxX = -Infinity,
      minY = Infinity,
      maxY = -Infinity,
      minZ = Infinity,
      maxZ = -Infinity;
    const period = Math.max(loop.period, 1);
    loop.frames.forEach((f, t) => {
      const shift = loop.anchorX + (t / period) * loop.dx;
      for (const b of f.blocks) {
        minX = Math.min(minX, b.x - shift);
        maxX = Math.max(maxX, b.x - shift + 1);
        minY = Math.min(minY, b.y);
        maxY = Math.max(maxY, b.y + 1);
        minZ = Math.min(minZ, b.z);
        maxZ = Math.max(maxZ, b.z + 1);
      }
    });
    if (!Number.isFinite(minX)) return null;
    const focus: [number, number, number] = [
      (minX + maxX) / 2,
      (minY + maxY) / 2 - 0.5,
      (minZ + maxZ) / 2,
    ];
    const floor = {
      minX: minX - 5,
      maxX: maxX + 6,
      minZ: minZ - 2,
      maxZ: maxZ + 3,
      y: Math.min(minY, 0),
    };
    const bounds = { minX, maxX, minY, maxY, minZ, maxZ };
    return { focus, floor, bounds };
  }, [loop]);

  // WebGL stage: real-mesher models, camera-follow compensation.
  useEffect(() => {
    const mount = glRef.current;
    if (!webgl || !mount || !loop || !geom || !loop.cast) return;

    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    } catch {
      setGlFail(true);
      return;
    }
    renderer.setClearColor(0x000000, 0);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    renderer.setSize(mount.clientWidth, CSS_H);
    // Piston-head arms are clipped at their base's front face (see
    // headClipHalfSpace) so they never poke out the back mid-move.
    renderer.localClippingEnabled = true;
    mount.appendChild(renderer.domElement);
    const onLost = (e: Event) => {
      e.preventDefault();
      console.warn("[gl] stage context lost — recreating");
      setGlNonce((n) => n + 1);
    };
    renderer.domElement.addEventListener("webglcontextlost", onLost);

    const scene = new THREE.Scene();
    addDefaultLights(scene);
    const camera = new THREE.PerspectiveCamera(
      34,
      mount.clientWidth / CSS_H,
      0.1,
      1000,
    );
    // Frame the machine in COMPENSATED space, then translate the camera by
    // the running shift each frame (move the camera, not the world).
    const b = geom.bounds;
    const compBox = new THREE.Box3(
      new THREE.Vector3(b.minX, b.minY, b.minZ),
      new THREE.Vector3(b.maxX, b.maxY, b.maxZ),
    );
    const compCenter = fitCamera(camera, compBox, 1.35);
    // Orbit controls on the stage: the user may orbit/zoom freely while the
    // +x follow is applied as a per-frame DELTA to both camera and target —
    // the follow never fights the user's orientation, and a machine swap
    // (the only thing that recreates this scene) is the only auto re-fit.
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.1;
    controls.target.copy(compCenter);
    controls.update();
    const onCtrlStart = () => onInteractRef.current?.();
    controls.addEventListener("start", onCtrlStart);
    window.__fgaCameras = window.__fgaCameras ?? {};
    window.__fgaCameras.stage = camera;
    let prevShift = 0;

    // Corridor floor: a static world-space grid wide enough to cover one
    // full camera period plus margins (1-block cells → dx-shift invariant).
    const period = Math.max(loop.period, 1);
    const spanX = Math.ceil(b.maxX - b.minX + Math.abs(loop.dx) + 24);
    const spanZ = Math.ceil(b.maxZ - b.minZ + 12);
    const size = Math.max(spanX, spanZ);
    const grid = new THREE.GridHelper(size, size, 0x888888, 0x888888);
    (grid.material as THREE.Material).transparent = true;
    (grid.material as THREE.Material).opacity = 0.16;
    grid.position.set(
      (b.minX + b.maxX) / 2 + loop.anchorX + loop.dx / 2,
      geom.floor.y + 0.001,
      (b.minZ + b.maxZ) / 2,
    );
    scene.add(grid);

    const members = loop.cast;
    const groups: THREE.Group[] = members.map((m) => {
      const g = new THREE.Group();
      g.position.set(m.x, m.y, m.z);
      g.visible = false;
      scene.add(g);
      return g;
    });
    let disposed = false;
    let pending = members.length;
    members.forEach((m, i) => {
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
        })
        .catch((e) => console.warn("mesh failed for", m.state, e))
        .finally(() => {
          pending -= 1;
          if (pending === 0 && !disposed) {
            console.info(
              `[mesh] ${meshStats.states} unique states, ` +
                `${(meshStats.totalMs / Math.max(meshStats.states, 1)).toFixed(1)} ms avg`,
            );
          }
        });
    });

    const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
    let raf = 0;
    const t0 = performance.now();
    const render = (now: number) => {
      if (loop.period > 0 && !reduced) raf = requestAnimationFrame(render);
      const phase =
        loop.period > 0 && !reduced
          ? ((((now - t0) / 1000) * TPS) % period) / period
          : 0;
      const t = phase * period;
      const shift = loop.anchorX + phase * loop.dx;
      for (let i = 0; i < members.length; i++) {
        const m = members[i];
        const g = groups[i];
        const vis = t >= m.start && t < m.end;
        g.visible = vis;
        if (!vis) continue;
        if (m.motion) {
          const span = m.motion.until - m.start;
          const progress =
            span > 0 ? Math.min(1, Math.max(0, (t - m.start) / span)) : 1;
          const remaining = 1 - progress;
          g.position.set(
            m.x + (m.motion.fx - m.x) * remaining,
            m.y + (m.motion.fy - m.y) * remaining,
            m.z + (m.motion.fz - m.z) * remaining,
          );
        }
      }
      const d = shift - prevShift;
      prevShift = shift;
      camera.position.x += d;
      controls.target.x += d;
      controls.update();
      renderer.render(scene, camera);
    };
    raf = requestAnimationFrame(render);
    // Static scenes (no period / reduced motion) have no self-running loop,
    // so orbiting must repaint on control changes.
    const onCtrlChange = () => {
      if (!(loop.period > 0) || reduced) renderer.render(scene, camera);
    };
    controls.addEventListener("change", onCtrlChange);

    const ro = new ResizeObserver(() => {
      const w = mount.clientWidth;
      renderer.setSize(w, CSS_H);
      camera.aspect = w / CSS_H;
      camera.updateProjectionMatrix();
    });
    ro.observe(mount);

    return () => {
      disposed = true;
      ro.disconnect();
      cancelAnimationFrame(raf);
      if (window.__fgaCameras?.stage === camera) delete window.__fgaCameras.stage;
      controls.removeEventListener("change", onCtrlChange);
      controls.removeEventListener("start", onCtrlStart);
      controls.dispose();
      renderer.domElement.removeEventListener("webglcontextlost", onLost);
      renderer.dispose();
      renderer.forceContextLoss();
      mount.removeChild(renderer.domElement);
    };
  }, [webgl, loop, geom, glNonce]);

  // Classic iso playback (fallback + GIF preview parity).
  useEffect(() => {
    const canvas = canvasRef.current;
    const wrap = wrapRef.current;
    if (webgl || !canvas || !wrap || !loop || !geom) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
    let raf = 0;
    const t0 = performance.now();
    // Read the theme palette once per mount, not per frame (getComputedStyle
    // in the rAF loop forces a style recalc — one source of the old jank).
    let pal = readPalette();

    const render = (now: number) => {
      if (loop.period > 0 && !reduced) raf = requestAnimationFrame(render);
      const cssW = wrap.clientWidth;
      const dpr = window.devicePixelRatio || 1;
      if (canvas.width !== Math.round(cssW * dpr)) {
        canvas.width = Math.round(cssW * dpr);
        canvas.height = Math.round(CSS_H * dpr);
      }
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      const period = Math.max(loop.period, 1);
      const phase =
        loop.period > 0 && !reduced
          ? ((((now - t0) / 1000) * TPS) % period) / period
          : 0;
      const tick = Math.min(Math.floor(phase * period), loop.frames.length - 1);
      const frame = loop.frames[tick];
      if (!frame) return;
      const shift = loop.anchorX + phase * loop.dx;
      drawScene(
        ctx,
        cssW,
        CSS_H,
        frame.blocks ?? [],
        shift,
        pal,
        26,
        geom.floor,
        geom.focus,
        loop.cast ?? null,
        phase * period,
      );
    };
    raf = requestAnimationFrame(render);
    // Static scenes (reduced motion / no period) don't loop, so repaint once
    // whenever a block texture finishes decoding; the rAF loop covers the rest.
    const unsub = onTexturesChanged(() => {
      pal = readPalette();
      if (reduced || loop.period === 0) raf = requestAnimationFrame(render);
    });
    return () => {
      unsub();
      cancelAnimationFrame(raf);
    };
  }, [webgl, loop, geom]);

  const onExport = useCallback(() => {
    if (!loop || !geom || !best) return;
    setExporting(true);
    // Let the button repaint before the synchronous encode.
    setTimeout(() => {
      try {
        const blob = exportLoopGif(loop, readPalette(), geom.focus, geom.floor);
        downloadBlob(blob, `flight-gen${best.gen}-${best.fitness.toFixed(1)}.gif`);
      } finally {
        setExporting(false);
      }
    }, 30);
  }, [loop, geom, best]);

  if (!best) {
    return (
      <div className="viewer-empty stage-empty">
        The first champion's flight appears here once a machine flies.
      </div>
    );
  }

  return (
    <div className="stage">
      <div className="stage-meta">
        <div className="big" data-testid="stage-speed">
          {evalTicks ? speedOf(best.fitness, evalTicks).toFixed(2) : "—"}
          <span className="unit">blk/s @ 20 tps</span>
        </div>
        <div className="kv">
          distance<b>{best.fitness.toFixed(1)} blocks</b>
        </div>
        <div className="kv">
          champion of<b>gen {best.gen}</b>
        </div>
        <div className="kv">
          period
          <b>{loop && loop.period > 0 ? `${loop.period} ticks / +${loop.dx}x` : "—"}</b>
        </div>
        <div className="spacer" />
        {canWebgl && (
          <button
            className="icon-btn"
            onClick={() => setMode((m) => (m === "webgl" ? "iso" : "webgl"))}
            data-testid="stage-mode-toggle"
          >
            {webgl ? "classic iso" : "WebGL"}
          </button>
        )}
        <button
          className="icon-btn"
          onClick={onExport}
          disabled={!loop || loop.period === 0 || exporting}
          data-testid="export-gif"
        >
          {exporting ? "encoding…" : "Export GIF"}
        </button>
      </div>

      <div className="stage-canvas" ref={wrapRef}>
        {webgl ? (
          <div
            ref={glRef}
            style={{ width: "100%", height: CSS_H }}
            role="img"
            data-testid="stage-webgl"
            aria-label={`Looping flight of the generation ${best.gen} champion`}
          />
        ) : (
          <canvas
            ref={canvasRef}
            style={{ width: "100%", height: CSS_H }}
            role="img"
            aria-label={`Looping flight of the generation ${best.gen} champion`}
          />
        )}
        {(loading || !loop) && (
          <div className="stage-overlay">
            {loading ? "re-simulating flight…" : "no replay yet"}
          </div>
        )}
        {!isChampion && (
          <button
            type="button"
            className="stage-tag follow-chip"
            onClick={onResumeFollow}
            data-testid="stage-follow-chip"
            title="The stage is pinned to this machine — click to follow the reigning champion again"
          >
            following champion ⏸ — resume
          </button>
        )}
      </div>

      {loop && (
        <p className="stage-note">
          {loop.period > 0
            ? `loop = 1 period, detected via ${loop.method}`
            : "machine did not settle into a periodic gait — showing final state"}
        </p>
      )}
    </div>
  );
}
