import { useMemo } from "react";
import type { LeaderboardEntry } from "../api";

/** Block identity colors — fixed by subject, same in both themes. */
const BLOCK_COLORS: Record<string, { base: string; label: string }> = {
  slime_block: { base: "#5fbf4e", label: "slime" },
  sticky_piston: { base: "#b58b57", label: "sticky piston" },
  piston: { base: "#a08b62", label: "piston" },
  observer: { base: "#77756f", label: "observer" },
  honey_block: { base: "#e8a23d", label: "honey" },
};
const FALLBACK = { base: "#8d86a8", label: "other" };

function blockKind(state: string): string {
  return state.replace(/^minecraft:/, "").replace(/\[.*$/, "");
}

function shade(hex: string, f: number): string {
  const n = parseInt(hex.slice(1), 16);
  const ch = (v: number) => Math.max(0, Math.min(255, Math.round(v * f)));
  const r = ch((n >> 16) & 255);
  const g = ch((n >> 8) & 255);
  const b = ch(n & 255);
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, "0")}`;
}

/* Isometric projection: +x runs right-down, +z runs left-down, +y is up. */
const UX = 22; // half cube width
const UY = 11; // half cube depth (screen)
const UH = 24; // cube height

function proj(x: number, y: number, z: number): [number, number] {
  return [(x - z) * UX, (x + z) * UY - y * UH];
}

interface Props {
  machine: LeaderboardEntry | null;
  evalTicks: number | null;
}

export default function MachineViewer({ machine, evalTicks }: Props) {
  const scene = useMemo(() => {
    if (!machine || machine.blocks.length === 0) return null;

    const blocks = [...machine.blocks].sort(
      (a, b) => a.x + a.z - (b.x + b.z) || a.y - b.y,
    );

    const dims = ["x", "y", "z"].map((k) => {
      const vs = blocks.map((b) => b[k as "x" | "y" | "z"]);
      return Math.max(...vs) - Math.min(...vs) + 1;
    });

    // Floor extent (padded one cell around the build).
    const minX = Math.min(...blocks.map((b) => b.x)) - 1;
    const maxX = Math.max(...blocks.map((b) => b.x)) + 2;
    const minZ = Math.min(...blocks.map((b) => b.z)) - 1;
    const maxZ = Math.max(...blocks.map((b) => b.z)) + 2;
    const minY = Math.min(...blocks.map((b) => b.y));

    const pts: number[] = [];
    const push = (p: [number, number]) => pts.push(p[0], p[1]);
    for (const [gx, gz] of [
      [minX, minZ],
      [maxX, minZ],
      [minX, maxZ],
      [maxX, maxZ],
    ]) {
      push(proj(gx, minY, gz));
    }
    for (const b of blocks) {
      push(proj(b.x, b.y + 1, b.z));
      push(proj(b.x + 1, b.y + 1, b.z + 1));
    }
    const xs = pts.filter((_, i) => i % 2 === 0);
    const ys = pts.filter((_, i) => i % 2 === 1);
    const pad = 14;
    const vb = {
      x: Math.min(...xs) - pad,
      y: Math.min(...ys) - pad,
      w: Math.max(...xs) - Math.min(...xs) + pad * 2,
      h: Math.max(...ys) - Math.min(...ys) + pad * 2,
    };

    // Floor grid lines at the build's base level.
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

    const kinds = [...new Set(blocks.map((b) => blockKind(b.state)))];

    return { blocks, dims, vb, floor, kinds };
  }, [machine]);

  if (!machine || !scene) {
    return (
      <div className="viewer-empty">
        Select a machine from the leaderboard to inspect it.
      </div>
    );
  }

  const { blocks, dims, vb, floor, kinds } = scene;

  return (
    <div className="viewer">
      <div className="viewer-stats">
        <div className="big">
          {machine.fitness.toFixed(1)}
          <span className="unit">blocks flown{evalTicks ? ` / ${evalTicks} ticks` : ""}</span>
        </div>
        <div className="kv">
          blocks<b>{machine.blocks.length}</b>
        </div>
        <div className="kv">
          size<b>{dims.join("×")}</b>
        </div>
        <div className="kv">
          found gen<b>{machine.gen}</b>
        </div>
      </div>

      <div className="viewer-stage">
        <svg
          viewBox={`${vb.x} ${vb.y} ${vb.w} ${vb.h}`}
          width={Math.min(460, vb.w * 1.5)}
          role="img"
          aria-label={`Voxel structure of ${machine.name ?? machine.id}: ${machine.blocks.length} blocks`}
        >
          {floor.map(([x1, y1, x2, y2], i) => (
            <line key={i} x1={x1} y1={y1} x2={x2} y2={y2} stroke="var(--grid-iso)" strokeWidth={1} />
          ))}
          {blocks.map((b, i) => {
            const kind = blockKind(b.state);
            const c = BLOCK_COLORS[kind] ?? FALLBACK;
            const top = proj(b.x, b.y + 1, b.z);
            const right = proj(b.x + 1, b.y + 1, b.z);
            const front = proj(b.x + 1, b.y + 1, b.z + 1);
            const left = proj(b.x, b.y + 1, b.z + 1);
            const h = UH;
            const face = (pts: Array<[number, number]>, fill: string) =>
              pts.map((p) => `${p[0]},${p[1]}`).join(" ") && (
                <polygon
                  points={pts.map((p) => `${p[0].toFixed(1)},${p[1].toFixed(1)}`).join(" ")}
                  fill={fill}
                  stroke="rgba(0,0,0,0.25)"
                  strokeWidth={0.6}
                  strokeLinejoin="round"
                />
              );
            return (
              <g key={i}>
                {/* left face (toward +z) */}
                {face(
                  [left, front, [front[0], front[1] + h], [left[0], left[1] + h]],
                  shade(c.base, 0.82),
                )}
                {/* right face (toward +x) */}
                {face(
                  [front, right, [right[0], right[1] + h], [front[0], front[1] + h]],
                  shade(c.base, 0.62),
                )}
                {/* top face */}
                {face([top, right, front, left], shade(c.base, 1.08))}
              </g>
            );
          })}
        </svg>
      </div>

      <div className="block-legend">
        {kinds.map((k) => {
          const c = BLOCK_COLORS[k] ?? FALLBACK;
          return (
            <span className="item" key={k}>
              <span className="swatch" style={{ background: c.base }} />
              {c.label}
            </span>
          );
        })}
      </div>
    </div>
  );
}
