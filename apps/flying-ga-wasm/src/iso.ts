/** Shared isometric projection + block identity colors.
 * Colors are fixed by subject (same in both themes), adapted from the
 * server edition's MachineViewer. */

import { blockKind } from "./ga/alphabet";
import { boxesForItem, castAt, type CastMember, type DrawItem } from "./cast";
import { ensureTextureIndex, faceCanvas, FACE_OVERLAY } from "./textures";
import type { Block } from "./types";

export const BLOCK_COLORS: Record<string, { base: string; label: string }> = {
  slime_block: { base: "#5fbf4e", label: "slime" },
  honey_block: { base: "#e8a23d", label: "honey" },
  sticky_piston: { base: "#b58b57", label: "sticky piston" },
  piston: { base: "#a08b62", label: "piston" },
  piston_head: { base: "#c9a877", label: "piston head" },
  moving_piston: { base: "#9c8a6b", label: "moving" },
  observer: { base: "#77756f", label: "observer" },
  note_block: { base: "#8a5a3b", label: "note block" },
  white_concrete: { base: "#e3e1da", label: "concrete" },
  oak_trapdoor: { base: "#b8945f", label: "trapdoor" },
  redstone_block: { base: "#d03b3b", label: "redstone (kick)" },
};
export const FALLBACK_COLOR = { base: "#8d86a8", label: "other" };

export function colorOf(state: string): { base: string; label: string } {
  return BLOCK_COLORS[blockKind(state)] ?? FALLBACK_COLOR;
}

export function shade(hex: string, f: number): string {
  const n = parseInt(hex.slice(1), 16);
  const ch = (v: number) => Math.max(0, Math.min(255, Math.round(v * f)));
  const r = ch((n >> 16) & 255);
  const g = ch((n >> 8) & 255);
  const b = ch(n & 255);
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, "0")}`;
}

/* Isometric projection: +x runs right-down, +z runs left-down, +y is up. */
export function makeProj(ux: number) {
  const UX = ux; // half cube width
  const UY = ux / 2; // half cube depth (screen)
  const UH = ux * 1.09; // cube height
  const proj = (x: number, y: number, z: number): [number, number] => [
    (x - z) * UX,
    (x + z) * UY - y * UH,
  ];
  return { UX, UY, UH, proj };
}

export interface CanvasPalette {
  page: string;
  grid: string;
  edge: string;
}

/** Painter-sorted cube render onto a 2D canvas at an arbitrary (fractional)
 * world x-offset — the flight-loop compensation.
 *
 * When a member `cast` and fractional `time` are given (a replay loop), the
 * draw list comes from the cast instead of `blocks`: moving_piston is never
 * drawn as its own cube — carried blocks slide between cells with the
 * g4mespeed pause-at-end easing, and piston bases/heads draw their real
 * extended geometry (see cast.ts). */
export function drawScene(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  blocks: Block[],
  shiftX: number,
  pal: CanvasPalette,
  ux = 20,
  floor?: { minX: number; maxX: number; minZ: number; maxZ: number; y: number },
  focus: [number, number, number] = [2.5, 1, 1.5],
  cast?: CastMember[] | null,
  time = 0,
): void {
  ensureTextureIndex();
  const { UH, proj } = makeProj(ux);
  ctx.fillStyle = pal.page;
  ctx.fillRect(0, 0, w, h);

  ctx.save();
  ctx.translate(w / 2, h / 2);

  // Keep the camera pinned on a fixed world-space focus so only the floor
  // appears to move.
  const f = proj(focus[0], focus[1], focus[2]);
  ctx.translate(-f[0], -f[1]);

  // Corridor floor: gridlines at integer world coords, scrolled by shiftX.
  if (floor) {
    ctx.strokeStyle = pal.grid;
    ctx.lineWidth = 1;
    ctx.beginPath();
    const x0 = Math.floor(floor.minX + shiftX);
    const x1 = Math.ceil(floor.maxX + shiftX);
    for (let gx = x0; gx <= x1; gx++) {
      const a = proj(gx - shiftX, floor.y, floor.minZ);
      const b = proj(gx - shiftX, floor.y, floor.maxZ);
      ctx.moveTo(a[0], a[1]);
      ctx.lineTo(b[0], b[1]);
    }
    for (let gz = floor.minZ; gz <= floor.maxZ; gz++) {
      const a = proj(floor.minX - (shiftX % 1), floor.y, gz);
      const b = proj(floor.maxX - (shiftX % 1), floor.y, gz);
      ctx.moveTo(a[0], a[1]);
      ctx.lineTo(b[0], b[1]);
    }
    ctx.stroke();
  }

  const items: DrawItem[] = cast
    ? castAt(cast, time)
    : blocks.map((b, id) => ({
        id,
        x: b.x,
        y: b.y,
        z: b.z,
        ox: 0,
        oy: 0,
        oz: 0,
        state: b.state,
      }));
  items.sort(
    (a, b) =>
      a.x + a.ox + a.z + a.oz - (b.x + b.ox + b.z + b.oz) || a.y + a.oy - (b.y + b.oy),
  );

  const path = (pts: Array<[number, number]>) => {
    ctx.beginPath();
    ctx.moveTo(pts[0][0], pts[0][1]);
    for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i][0], pts[i][1]);
    ctx.closePath();
  };
  /** Parallelogram face: flat under-fill (also the universal fallback and
   * the ground behind transparent texels), the texture mapped by the
   * affine (o,u,v) frame, a brightness overlay for depth, edge stroke. */
  const face = (
    pts: Array<[number, number]>,
    fill: string,
    tex: HTMLCanvasElement | null,
    o: [number, number],
    u: [number, number],
    v: [number, number],
    overlay: string | null,
  ) => {
    path(pts);
    ctx.fillStyle = fill;
    ctx.fill();
    if (tex) {
      ctx.save();
      ctx.imageSmoothingEnabled = false;
      ctx.transform(u[0], u[1], v[0], v[1], o[0], o[1]);
      ctx.drawImage(tex, 0, 0, tex.width, tex.height, 0, 0, 1, 1);
      ctx.restore();
      if (overlay) {
        path(pts);
        ctx.fillStyle = overlay;
        ctx.fill();
      }
    }
    path(pts);
    ctx.strokeStyle = pal.edge;
    ctx.lineWidth = 0.6;
    ctx.stroke();
  };

  for (const item of items) {
    const wx = item.x + item.ox - shiftX;
    const wy = item.y + item.oy;
    const wz = item.z + item.oz;
    const c = colorOf(item.state).base;
    for (const bx of boxesForItem(item)) {
      const x0 = wx + bx.x0;
      const x1 = wx + bx.x1;
      const y0 = wy + bx.y0;
      const y1 = wy + bx.y1;
      const z0 = wz + bx.z0;
      const z1 = wz + bx.z1;
      // Top ring corners: A back, B right (+x), C front (+x,+z), D left (+z).
      const A = proj(x0, y1, z0);
      const B = proj(x1, y1, z0);
      const C = proj(x1, y1, z1);
      const D = proj(x0, y1, z1);
      const hpx = (y1 - y0) * UH;
      const tUp = bx.tex ? faceCanvas(bx.tex, "up") : null;
      const tSouth = bx.tex ? faceCanvas(bx.tex, "south") : null;
      const tEast = bx.tex ? faceCanvas(bx.tex, "east") : null;
      face(
        [D, C, [C[0], C[1] + hpx], [D[0], D[1] + hpx]],
        shade(c, 0.82),
        tSouth,
        D,
        [C[0] - D[0], C[1] - D[1]],
        [0, hpx],
        FACE_OVERLAY.left,
      );
      face(
        [C, B, [B[0], B[1] + hpx], [C[0], C[1] + hpx]],
        shade(c, 0.62),
        tEast,
        C,
        [B[0] - C[0], B[1] - C[1]],
        [0, hpx],
        FACE_OVERLAY.right,
      );
      face(
        [A, B, C, D],
        shade(c, 1.08),
        tUp,
        A,
        [B[0] - A[0], B[1] - A[1]],
        [D[0] - A[0], D[1] - A[1]],
        FACE_OVERLAY.top,
      );
    }
  }
  ctx.restore();
}
