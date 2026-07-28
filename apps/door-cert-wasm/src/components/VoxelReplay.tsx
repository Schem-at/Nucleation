// Exhibit A — the cycle, replayed. The recorded change log is deterministic
// evidence, so instead of a rendered video the certificate carries a live
// instrument: the t=0 world plus every block change, reconstructed into a
// member cast (see lib/cast.ts, ported from examples/render_simulation_video.rs)
// and drawn as painter-sorted isometric voxels. moving_piston placeholders are
// never drawn: the blocks they carry slide between cells with g4mespeed's
// pause-at-end easing, piston bases open into their extended shells, and heads
// render as plate + arm. A scrubber (fractional ticks) with the measured lever
// flips stamped in seal red.
import { useEffect, useMemo, useReducer, useRef, useState } from "react";
import { boxesForItem, buildCast, castAt } from "../lib/cast";
import {
  ensureTextureIndex,
  onTexturesChanged,
  faceURL,
  FACE_OVERLAY,
} from "../lib/textures";
import type { Replay, Vec3 } from "../lib/types";

/* Isometric projection: +x runs right-down, +z runs left-down, +y is up. */
const UX = 22; // half cube width
const UY = 11; // half cube depth (screen)
const UH = 24; // cube height

function proj(x: number, y: number, z: number): [number, number] {
  return [(x - z) * UX, (x + z) * UY - y * UH];
}

/** Block identity colors — fixed by subject, same in both themes. */
const DYE: Record<string, string> = {
  white: "#dfe0e0", orange: "#d97a2a", magenta: "#bb50b8", light_blue: "#5aa3d4",
  yellow: "#e0c33f", lime: "#7bbf35", pink: "#d98fa6", gray: "#5b5f63",
  light_gray: "#94948c", cyan: "#278a91", purple: "#8340b5", blue: "#3f4ba0",
  brown: "#795236", green: "#5d7530", red: "#a3352d", black: "#25272b",
};
const EXACT: Record<string, string> = {
  sticky_piston: "#b58b57",
  piston: "#a08b62",
  piston_head: "#c9b98a",
  moving_piston: "#8f7c50",
  observer: "#77756f",
  slime_block: "#5fbf4e",
  honey_block: "#e8a23d",
  redstone_wire: "#c22f21",
  redstone_torch: "#d8412c",
  redstone_wall_torch: "#d8412c",
  redstone_block: "#a5231a",
  redstone_lamp: "#b98a4a",
  repeater: "#9b9ba3",
  comparator: "#9b9ba3",
  lever: "#8a7a5c",
  target: "#d6cfc2",
  note_block: "#6d4a30",
  obsidian: "#241d33",
  glass: "#c8dfe1",
};

function blockKind(state: string): string {
  return state.replace(/^minecraft:/, "").replace(/\[.*$/, "");
}

function colorFor(kind: string): string {
  if (EXACT[kind]) return EXACT[kind];
  const dye = Object.keys(DYE).find(
    (d) => kind.startsWith(d + "_") && /_(concrete|wool|terracotta|glass|glazed)/.test(kind),
  );
  if (dye) return DYE[dye];
  if (/leaves/.test(kind)) return "#4e7a32";
  if (/(log|wood|planks|fence|trapdoor|door)/.test(kind)) return "#7a5b34";
  if (/(slab|stone|brick|deepslate|andesite|diorite|granite|quartz)/.test(kind)) return "#9a9a94";
  if (/(dirt|mud|soul)/.test(kind)) return "#6b4f39";
  if (/grass/.test(kind)) return "#5d8f45";
  // deterministic fallback hue from the name
  let hsh = 0;
  for (let i = 0; i < kind.length; i++) hsh = (hsh * 31 + kind.charCodeAt(i)) >>> 0;
  return `hsl(${hsh % 360} 30% 55%)`;
}

function shade(color: string, f: number): string {
  if (!color.startsWith("#")) return color;
  const n = parseInt(color.slice(1), 16);
  const ch = (v: number) => Math.max(0, Math.min(255, Math.round(v * f)));
  const r = ch((n >> 16) & 255);
  const g = ch((n >> 8) & 255);
  const b = ch(n & 255);
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, "0")}`;
}

const isAir = (s: string) => s.endsWith("air");

const SPEEDS = [0.5, 1, 2, 4];

export function VoxelReplay({ replay, lever }: { replay: Replay; lever: Vec3 }) {
  const { simTicks } = replay;
  const reduced = useRef(
    typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );
  const [time, setTime] = useState(0); // fractional ticks
  const [playing, setPlaying] = useState(!reduced.current);
  const [speed, setSpeed] = useState(1);

  // Re-render as block textures finish decoding (flat colours fill in
  // meanwhile and remain the universal fallback).
  const [, bumpTex] = useReducer((n: number) => n + 1, 0);
  useEffect(() => {
    ensureTextureIndex();
    return onTexturesChanged(bumpTex);
  }, []);

  // Static scene: bounds over everything a block ever occupies, so the
  // frame never jumps mid-replay; floor grid at the base level.
  const scene = useMemo(() => {
    const pts: Vec3[] = [
      ...replay.blocks.map((b) => b.pos),
      ...replay.changes.map((c) => c.pos),
    ];
    if (pts.length === 0) return null;
    const minX = Math.min(...pts.map((p) => p[0])) - 1;
    const maxX = Math.max(...pts.map((p) => p[0])) + 2;
    const minZ = Math.min(...pts.map((p) => p[2])) - 1;
    const maxZ = Math.max(...pts.map((p) => p[2])) + 2;
    const minY = Math.min(...pts.map((p) => p[1]));
    const maxY = Math.max(...pts.map((p) => p[1])) + 1;

    const corners: number[] = [];
    const push = (p: [number, number]) => corners.push(p[0], p[1]);
    for (const [gx, gz] of [[minX, minZ], [maxX, minZ], [minX, maxZ], [maxX, maxZ]] as const) {
      push(proj(gx, minY, gz));
      push(proj(gx, maxY, gz));
    }
    const xs = corners.filter((_, i) => i % 2 === 0);
    const ys = corners.filter((_, i) => i % 2 === 1);
    const pad = 16;
    const vb = {
      x: Math.min(...xs) - pad,
      y: Math.min(...ys) - pad,
      w: Math.max(...xs) - Math.min(...xs) + pad * 2,
      h: Math.max(...ys) - Math.min(...ys) + pad * 2,
    };

    const floor: Array<[number, number, number, number]> = [];
    for (let gx = minX; gx <= maxX; gx++) {
      const a = proj(gx, minY, minZ);
      const b = proj(gx, minY, maxZ);
      floor.push([a[0], a[1], b[0], b[1]]);
    }
    for (let gz = minZ; gz <= maxZ; gz++) {
      const a = proj(minX, minY, gz);
      const b = proj(maxX, minY, gz);
      floor.push([a[0], a[1], b[0], b[1]]);
    }

    // moving_piston never renders as itself (the cast resolves it), so it
    // has no place in the legend either.
    const kinds = new Set<string>();
    for (const b of replay.blocks) if (!isAir(b.state)) kinds.add(blockKind(b.state));
    for (const c of replay.changes) if (!isAir(c.to)) kinds.add(blockKind(c.to));
    kinds.delete("moving_piston");
    return { vb, floor, kinds: [...kinds].sort() };
  }, [replay]);

  // The member cast, built once per replay (per-frame work is pose
  // interpolation only). `simTicks + 1` keeps the final states visible at
  // the scrubber's inclusive right edge.
  const cast = useMemo(
    () =>
      buildCast(
        replay.blocks.map((b) => ({ pos: b.pos, state: b.state })),
        replay.changes,
        simTicks + 1,
      ),
    [replay, simTicks],
  );

  // Playback clock: 20 tps scaled by speed, fractional, looping. rAF so
  // piston strokes interpolate between ticks.
  useEffect(() => {
    if (!playing || !scene) return;
    let raf = 0;
    let last = performance.now();
    const step = (now: number) => {
      const dt = Math.min((now - last) / 1000, 0.25);
      last = now;
      setTime((t) => {
        const nt = t + dt * 20 * speed;
        return nt >= simTicks ? 0 : nt;
      });
      raf = requestAnimationFrame(step);
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  }, [playing, speed, simTicks, scene]);

  // Visible members at the current (fractional) time, painter-sorted.
  const items = useMemo(() => {
    const list = castAt(cast, time);
    list.sort(
      (a, b) =>
        a.x + a.ox + a.z + a.oz - (b.x + b.ox + b.z + b.oz) ||
        a.y + a.oy - (b.y + b.oy),
    );
    return list;
  }, [cast, time]);

  if (!scene) return null;

  const tick = Math.floor(time);

  /* Parallelogram face: flat under-fill (fallback + ground behind
   * transparent texels), the block texture mapped by an affine (o,u,v)
   * frame of the unit square, a brightness overlay so depth still reads,
   * and the edge stroke on top. */
  const face = (
    pts: Array<[number, number]>,
    fill: string,
    url: string | null,
    o: [number, number],
    u: [number, number],
    v: [number, number],
    overlay: string | null,
    k: string,
  ) => {
    const points = pts
      .map((p) => `${p[0].toFixed(1)},${p[1].toFixed(1)}`)
      .join(" ");
    return (
      <g key={k}>
        <polygon points={points} fill={fill} />
        {url && (
          <image
            href={url}
            width={1}
            height={1}
            preserveAspectRatio="none"
            transform={`matrix(${u[0]} ${u[1]} ${v[0]} ${v[1]} ${o[0]} ${o[1]})`}
            style={{ imageRendering: "pixelated" }}
          />
        )}
        {url && overlay && <polygon points={points} fill={overlay} />}
        <polygon
          points={points}
          fill="none"
          stroke="rgba(0,0,0,0.25)"
          strokeWidth={0.6}
          strokeLinejoin="round"
        />
      </g>
    );
  };

  return (
    <div>
      <div className="replay-stage">
        <svg
          viewBox={`${scene.vb.x} ${scene.vb.y} ${scene.vb.w} ${scene.vb.h}`}
          role="img"
          aria-label={`Isometric replay of the door at tick ${tick} of ${simTicks}`}
        >
          {scene.floor.map(([x1, y1, x2, y2], i) => (
            <line key={i} x1={x1} y1={y1} x2={x2} y2={y2} stroke="var(--grid-iso)" strokeWidth={1} />
          ))}
          {items.map((it) => {
            const c = colorFor(blockKind(it.state));
            const wx = it.x + it.ox;
            const wy = it.y + it.oy;
            const wz = it.z + it.oz;
            const isLever =
              it.x === lever[0] && it.y === lever[1] && it.z === lever[2] &&
              it.ox === 0 && it.oy === 0 && it.oz === 0;
            return (
              <g key={it.id}>
                {boxesForItem(it).map((bx, j) => {
                  const x0 = wx + bx.x0;
                  const x1 = wx + bx.x1;
                  const y0 = wy + bx.y0;
                  const y1 = wy + bx.y1;
                  const z0 = wz + bx.z0;
                  const z1 = wz + bx.z1;
                  // Top ring: A back, B right (+x), C front (+x,+z), D left (+z).
                  const A = proj(x0, y1, z0);
                  const B = proj(x1, y1, z0);
                  const C = proj(x1, y1, z1);
                  const D = proj(x0, y1, z1);
                  const h = (y1 - y0) * UH;
                  const tUp = bx.tex ? faceURL(bx.tex, "up") : null;
                  const tSouth = bx.tex ? faceURL(bx.tex, "south") : null;
                  const tEast = bx.tex ? faceURL(bx.tex, "east") : null;
                  return (
                    <g key={j}>
                      {face(
                        [D, C, [C[0], C[1] + h], [D[0], D[1] + h]],
                        shade(c, 0.82),
                        tSouth,
                        D,
                        [C[0] - D[0], C[1] - D[1]],
                        [0, h],
                        FACE_OVERLAY.left,
                        "l",
                      )}
                      {face(
                        [C, B, [B[0], B[1] + h], [C[0], C[1] + h]],
                        shade(c, 0.62),
                        tEast,
                        C,
                        [B[0] - C[0], B[1] - C[1]],
                        [0, h],
                        FACE_OVERLAY.right,
                        "r",
                      )}
                      {face(
                        [A, B, C, D],
                        shade(c, 1.08),
                        tUp,
                        A,
                        [B[0] - A[0], B[1] - A[1]],
                        [D[0] - A[0], D[1] - A[1]],
                        FACE_OVERLAY.top,
                        "t",
                      )}
                    </g>
                  );
                })}
                {isLever && (
                  <polygon
                    points={[
                      proj(wx, wy + 1, wz),
                      proj(wx + 1, wy + 1, wz),
                      proj(wx + 1, wy + 1, wz + 1),
                      proj(wx, wy + 1, wz + 1),
                    ]
                      .map((p) => `${p[0]},${p[1]}`)
                      .join(" ")}
                    fill="none"
                    stroke="var(--seal)"
                    strokeWidth={1.6}
                  />
                )}
              </g>
            );
          })}
        </svg>
      </div>

      <div className="replay-controls">
        <button
          className="replay-btn"
          onClick={() => setPlaying((p) => !p)}
          aria-label={playing ? "Pause replay" : "Play replay"}
        >
          {playing ? "❚❚" : "▶"}
        </button>
        <span className="replay-readout">
          t=<b>{String(tick).padStart(3, "0")}</b>/{simTicks} · {(tick / 20).toFixed(2)} s
        </span>
        <div className="replay-track">
          <input
            type="range"
            min={0}
            max={simTicks}
            step={0.25}
            value={time}
            aria-label="Replay tick"
            onChange={(e) => {
              setPlaying(false);
              setTime(Number(e.target.value));
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
          onChange={(e) => setSpeed(Number(e.target.value))}
        >
          {SPEEDS.map((s) => (
            <option key={s} value={s}>
              {s}×
            </option>
          ))}
        </select>
      </div>

      <div className="block-legend">
        {scene.kinds.map((k) => {
          const tex = faceURL(k, "south");
          return (
            <span className="item" key={k}>
              <span
                className="swatch"
                style={
                  tex
                    ? {
                        background: `${colorFor(k)} url(${tex}) center / cover`,
                        imageRendering: "pixelated",
                      }
                    : { background: colorFor(k) }
                }
              />
              {k.replace(/_/g, " ")}
            </span>
          );
        })}
        <span className="item" style={{ marginLeft: "auto" }}>
          <span className="swatch" style={{ background: "var(--seal)" }} />
          measured lever flip
        </span>
      </div>
    </div>
  );
}
