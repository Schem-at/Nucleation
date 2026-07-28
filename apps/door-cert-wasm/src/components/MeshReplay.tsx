/** Exhibit A, native-quality: the recorded change log replayed through
 * nucleation's REAL mesher (wasm) into a three.js WebGL scene. Every unique
 * blockstate in the cast is meshed once as a 1-block schematic (levers, dust,
 * repeaters, torches, slabs get their real models, not cubes) and instanced
 * per cast member; poses come from the same cast/castAt animation brain as the
 * classic view (g4mespeed pause-at-end easing, fractional wall-clock ticks).
 *
 * Smoothness contract: the rAF loop never goes through React — time lives in
 * a ref, member groups are posed in place (zero per-frame allocations), and
 * the scrubber/readout are poked imperatively. React only re-renders on
 * play/pause/speed/mode changes. */

import { useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { buildCast, headClipHalfSpace, type CastMember } from "../lib/cast";
import { instanceFor, addDefaultLights, fitCamera, meshStats } from "../lib/mesher";
import type { Replay, Vec3 } from "../lib/types";
import { VoxelReplay } from "./VoxelReplay";

const SPEEDS = [0.5, 1, 2, 4];
const STAGE_H = 380;

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
      if (i === 0) {
        const arr = Array.from(samples.slice(0, n)).sort((a, b) => a - b);
        (window as unknown as Record<string, unknown>).__replayFrameStats = {
          p50: arr[Math.floor(n * 0.5)],
          p95: arr[Math.floor(n * 0.95)],
          n,
        };
      }
    },
  };
}

function WebglReplay({
  replay,
  lever,
  onFail,
}: {
  replay: Replay;
  lever: Vec3;
  onFail: (err: string) => void;
}) {
  const { simTicks } = replay;
  const mountRef = useRef<HTMLDivElement>(null);
  const readoutRef = useRef<HTMLElement>(null);
  const rangeRef = useRef<HTMLInputElement>(null);

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

    // Ground grid at the base level, native-style.
    const span = Math.ceil(
      Math.max(box.max.x - box.min.x, box.max.z - box.min.z) + 4,
    );
    const grid = new THREE.GridHelper(span, span, 0x888888, 0x888888);
    (grid.material as THREE.Material).transparent = true;
    (grid.material as THREE.Material).opacity = 0.18;
    grid.position.set(
      (box.min.x + box.max.x) / 2,
      box.min.y + 0.001,
      (box.min.z + box.max.z) / 2,
    );
    scene.add(grid);

    // Measured lever cell, seal red.
    const leverBox = new THREE.LineSegments(
      new THREE.EdgesGeometry(new THREE.BoxGeometry(1.02, 1.02, 1.02)),
      new THREE.LineBasicMaterial({ color: 0xb3282d }),
    );
    leverBox.position.set(lever[0] + 0.5, lever[1] + 0.5, lever[2] + 0.5);
    scene.add(leverBox);

    // One group per cast member; real models attach as meshing resolves.
    const groups: THREE.Group[] = cast.map((m) => {
      const g = new THREE.Group();
      g.position.set(m.x, m.y, m.z);
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

    const stats = makeFrameStats();
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
          g.position.set(
            m.x + (m.motion.fx - m.x) * remaining,
            m.y + (m.motion.fy - m.y) * remaining,
            m.z + (m.motion.fz - m.z) * remaining,
          );
        }
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
      ro.disconnect();
      cancelAnimationFrame(raf);
      controls.dispose();
      renderer.domElement.removeEventListener("webglcontextlost", onLost);
      renderer.dispose();
      renderer.forceContextLoss();
      mount.removeChild(renderer.domElement);
    };
  }, [cast, lever, simTicks, onFail, glNonce]);

  return (
    <div>
      <div
        className="replay-stage replay-stage-webgl"
        ref={mountRef}
        style={{ position: "relative", height: STAGE_H }}
        data-testid="mesh-replay-stage"
        data-ready={ready ? "1" : "0"}
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
              return !p;
            })
          }
          aria-label={playing ? "Pause replay" : "Play replay"}
        >
          {playing ? "❚❚" : "▶"}
        </button>
        <span className="replay-readout">
          t=<b ref={readoutRef}>000</b>/{simTicks}
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
        <select
          className="replay-speed"
          value={speed}
          aria-label="Replay speed"
          onChange={(e) => {
            const s = Number(e.target.value);
            speedRef.current = s;
            setSpeed(s);
          }}
        >
          {SPEEDS.map((s) => (
            <option key={s} value={s}>
              {s}×
            </option>
          ))}
        </select>
      </div>
    </div>
  );
}

/** Primary WebGL replay with the classic isometric painter as a fallback
 * toggle (and automatic fallback when WebGL is unavailable). */
export function MeshReplay({ replay, lever }: { replay: Replay; lever: Vec3 }) {
  const [mode, setMode] = useState<"webgl" | "iso">("webgl");
  const [glFail, setGlFail] = useState<string | null>(null);
  const iso = mode === "iso" || glFail !== null;

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "flex-end", marginBottom: 6 }}>
        <button
          className="replay-btn"
          style={{ fontSize: 12, width: "auto", padding: "2px 10px" }}
          onClick={() => setMode(iso ? "webgl" : "iso")}
          disabled={glFail !== null}
          data-testid="replay-mode-toggle"
        >
          {iso ? "▲ WebGL view" : "▤ classic iso view"}
        </button>
      </div>
      {iso ? (
        <VoxelReplay replay={replay} lever={lever} />
      ) : (
        <WebglReplay replay={replay} lever={lever} onFail={setGlFail} />
      )}
    </div>
  );
}
