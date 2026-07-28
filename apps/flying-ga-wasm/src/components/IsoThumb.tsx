/** Small static isometric SVG of a machine — used by the filmstrip and the
 * leaderboard-side machine viewer. */

import { useEffect, useMemo, useReducer } from "react";
import { boxesFor } from "../cast";
import { colorOf, makeProj, shade } from "../iso";
import {
  ensureTextureIndex,
  onTexturesChanged,
  faceURL,
  FACE_OVERLAY,
} from "../textures";
import type { Block } from "../types";

interface Props {
  blocks: Block[];
  /** Pixel width budget; height follows the scene. */
  width?: number;
  className?: string;
  label?: string;
}

export default function IsoThumb({ blocks, width = 120, className, label }: Props) {
  // Re-render as textures finish decoding (flat colours fill in meanwhile).
  const [, bump] = useReducer((n: number) => n + 1, 0);
  useEffect(() => {
    ensureTextureIndex();
    return onTexturesChanged(bump);
  }, []);

  const scene = useMemo(() => {
    if (blocks.length === 0) return null;
    const { UH, proj } = makeProj(22);
    const sorted = [...blocks].sort(
      (a, b) => a.x + a.z - (b.x + b.z) || a.y - b.y,
    );
    const pts: Array<[number, number]> = [];
    for (const b of sorted) {
      pts.push(proj(b.x, b.y + 1, b.z));
      const f = proj(b.x + 1, b.y + 1, b.z + 1);
      pts.push([f[0], f[1] + UH]);
      const r = proj(b.x + 1, b.y + 1, b.z);
      pts.push([r[0], r[1] + UH]);
      const l = proj(b.x, b.y + 1, b.z + 1);
      pts.push([l[0], l[1] + UH]);
    }
    const xs = pts.map((p) => p[0]);
    const ys = pts.map((p) => p[1]);
    const pad = 6;
    const vb = {
      x: Math.min(...xs) - pad,
      y: Math.min(...ys) - pad,
      w: Math.max(...xs) - Math.min(...xs) + pad * 2,
      h: Math.max(...ys) - Math.min(...ys) + pad * 2,
    };
    return { sorted, vb, UH, proj };
  }, [blocks]);

  if (!scene) return null;
  const { sorted, vb, UH, proj } = scene;

  /* Parallelogram face: flat under-fill (fallback + ground behind
   * transparent texels), texture via an affine map of the unit square,
   * brightness overlay, edge stroke on top. */
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
    <svg
      viewBox={`${vb.x} ${vb.y} ${vb.w} ${vb.h}`}
      width={width}
      height={(width * vb.h) / vb.w}
      className={className}
      role="img"
      aria-label={label ?? `Machine with ${blocks.length} blocks`}
    >
      {sorted.map((b, i) => {
        const c = colorOf(b.state).base;
        // Sub-box geometry (piston shells, head plate + arm) with per-face
        // facing-aware textures — see cast.ts / textures.ts.
        return (
          <g key={i}>
            {boxesFor(b.state).map((bx, j) => {
              const x0 = b.x + bx.x0;
              const x1 = b.x + bx.x1;
              const y0 = b.y + bx.y0;
              const y1 = b.y + bx.y1;
              const z0 = b.z + bx.z0;
              const z1 = b.z + bx.z1;
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
          </g>
        );
      })}
    </svg>
  );
}
