import { useRef, useState } from "react";
import type { Certificate } from "../lib/types";

// Change footprint: the door's literal XY silhouette, one cell per column,
// sequential single-hue ramp (magnitude). y=0 renders at the bottom, like
// build coordinates. The lever's column is ringed.
const CELL = 26;
const GAP = 2; // 2px surface gap between fills
const M = { left: 30, bottom: 24, top: 6, right: 6 };

const RAMP = [1, 2, 3, 4, 5, 6, 7] as const;

export function Heatmap({
  heatmap,
  lever,
}: {
  heatmap: Certificate["heatmap"];
  lever: Certificate["lever"];
}) {
  const wrapRef = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<{ x: number; y: number; px: number; py: number } | null>(null);

  const { w, h, values } = heatmap;
  const max = Math.max(1, ...values.flat());
  const bucket = (v: number) => (v <= 0 ? 0 : Math.min(7, Math.ceil((v / max) * 7)));

  const W = M.left + w * CELL + M.right;
  const H = M.top + h * CELL + M.bottom;
  const cx = (x: number) => M.left + x * CELL;
  const cy = (y: number) => M.top + (h - 1 - y) * CELL; // flip: y=0 at bottom

  function onMove(e: React.MouseEvent) {
    const el = wrapRef.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    const s = W / r.width;
    const vx = (e.clientX - r.left) * s;
    const vy = (e.clientY - r.top) * (H / r.height);
    const x = Math.floor((vx - M.left) / CELL);
    const yTop = Math.floor((vy - M.top) / CELL);
    const y = h - 1 - yTop;
    if (x < 0 || x >= w || y < 0 || y >= h) return setHover(null);
    setHover({ x, y, px: e.clientX - r.left, py: e.clientY - r.top });
  }

  const hv = hover ? values[hover.y][hover.x] : 0;
  const flipLeft = hover !== null && hover.px > (wrapRef.current?.clientWidth ?? W) - 150;

  return (
    <div>
      <div className="chart-svg-wrap" ref={wrapRef} onMouseMove={onMove} onMouseLeave={() => setHover(null)}>
        <svg viewBox={`0 0 ${W} ${H}`} aria-label="Block changes per column of the door footprint">
          {values.map((row, y) =>
            row.map((v, x) => {
              const b = bucket(v);
              return (
                <rect
                  key={`${x}-${y}`}
                  x={cx(x) + GAP / 2}
                  y={cy(y) + GAP / 2}
                  width={CELL - GAP}
                  height={CELL - GAP}
                  rx={2}
                  fill={b === 0 ? "var(--surface-2)" : `var(--heat-${b})`}
                />
              );
            }),
          )}
          {/* lever column ring */}
          <rect
            x={cx(lever[0]) + GAP / 2 + 1}
            y={cy(lever[1]) + GAP / 2 + 1}
            width={CELL - GAP - 2}
            height={CELL - GAP - 2}
            rx={2}
            fill="none"
            stroke="var(--ink)"
            strokeWidth="1.5"
          />
          {/* axes */}
          {[0, 5, 10, 15, 20].map((x) =>
            x < w ? (
              <text key={`x${x}`} className="axis-label" x={cx(x) + CELL / 2} y={H - 8} textAnchor="middle">
                {x}
              </text>
            ) : null,
          )}
          {[0, 4, 8, 12].map((y) =>
            y < h ? (
              <text key={`y${y}`} className="axis-label" x={M.left - 8} y={cy(y) + CELL / 2 + 3} textAnchor="end">
                {y}
              </text>
            ) : null,
          )}
        </svg>

        {hover && (
          <div
            className="tooltip"
            style={{
              left: flipLeft ? undefined : hover.px + 14,
              right: flipLeft ? (wrapRef.current?.clientWidth ?? W) - hover.px + 14 : undefined,
              top: Math.max(0, hover.py - 34),
            }}
          >
            <div className="tooltip-title">
              column ({hover.x}, {hover.y})
              {hover.x === lever[0] && hover.y === lever[1] ? " · lever" : ""}
            </div>
            <div className="tooltip-row">
              Changes<b>{hv}</b>
            </div>
          </div>
        )}
      </div>

      <div className="heat-legend">
        <span>0</span>
        <span className="heat-ramp" aria-hidden>
          {RAMP.map((i) => (
            <i key={i} style={{ background: `var(--heat-${i})` }} />
          ))}
        </span>
        <span>{max} changes</span>
        <span style={{ marginLeft: "auto" }}>▢ lever column</span>
      </div>
    </div>
  );
}
