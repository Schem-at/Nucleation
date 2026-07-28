import { useRef, useState } from "react";
import type { LeverFlip, TickEvents } from "../lib/types";

// Per-tick activity trace. Piston and redstone are different measures, so
// they get two aligned panels sharing one x-axis (never a dual axis) with a
// shared crosshair + tooltip. Lever flips are annotated at their recorded
// ticks — the simulation hands us the exact clicks, no inference needed.
const W = 1000;
const PANEL_H = 104;
const GAP = 26;
const M = { top: 8, right: 12, bottom: 26, left: 44 };
const H = M.top + PANEL_H * 2 + GAP + M.bottom;

function niceTicks(max: number): number[] {
  const step = max <= 5 ? 2 : max <= 10 ? 5 : Math.ceil(max / 2 / 5) * 5;
  const out = [];
  for (let v = step; v <= max; v += step) out.push(v);
  return out;
}

type Series = { key: "piston" | "redstone"; label: string; color: string };
const SERIES: Series[] = [
  { key: "piston", label: "Piston events", color: "var(--series-piston)" },
  { key: "redstone", label: "Redstone events", color: "var(--series-redstone)" },
];

export function ActivityChart({ events, flips }: { events: TickEvents[]; flips: LeverFlip[] }) {
  const wrapRef = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<{ tick: number; px: number; py: number } | null>(null);

  const nTicks = events.length;
  const plotW = W - M.left - M.right;
  const xOf = (t: number) => M.left + (t / nTicks) * plotW;
  const barW = Math.max(2, plotW / nTicks - 2); // 2px surface gap between bars

  const panels = SERIES.map((s, i) => {
    const top = M.top + i * (PANEL_H + GAP);
    const max = Math.max(1, ...events.map((e) => e[s.key]));
    const yOf = (v: number) => top + PANEL_H - (v / max) * (PANEL_H - 18);
    return { ...s, top, max, yOf };
  });

  function onMove(e: React.MouseEvent) {
    const el = wrapRef.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    const sx = W / r.width;
    const vx = (e.clientX - r.left) * sx;
    const tick = Math.min(nTicks - 1, Math.max(0, Math.floor(((vx - M.left) / plotW) * nTicks)));
    setHover({ tick, px: e.clientX - r.left, py: e.clientY - r.top });
  }

  const hovered = hover ? events[hover.tick] : null;
  const flipLeft = hover !== null && hover.px > (wrapRef.current?.clientWidth ?? W) - 180;

  return (
    <div
      className="chart-svg-wrap"
      ref={wrapRef}
      onMouseMove={onMove}
      onMouseLeave={() => setHover(null)}
    >
      <svg viewBox={`0 0 ${W} ${H}`} aria-label="Piston and redstone events per simulation tick">
        {panels.map((p) => (
          <g key={p.key}>
            {/* gridlines + y labels */}
            {niceTicks(p.max).map((v) => (
              <g key={v}>
                <line x1={M.left} x2={W - M.right} y1={p.yOf(v)} y2={p.yOf(v)} stroke="var(--grid)" strokeWidth="1" />
                <text className="axis-label" x={M.left - 8} y={p.yOf(v) + 3} textAnchor="end">
                  {v}
                </text>
              </g>
            ))}
            {/* baseline */}
            <line x1={M.left} x2={W - M.right} y1={p.top + PANEL_H} y2={p.top + PANEL_H} stroke="var(--baseline)" strokeWidth="1" />
            {/* panel title = direct label (single series per panel) */}
            <rect x={M.left} y={p.top - 4} width={9} height={9} rx={2} fill={p.color} />
            <text className="panel-label" x={M.left + 15} y={p.top + 4}>
              {p.label}
            </text>
            {/* bars: thin marks, rounded data-end, anchored to baseline */}
            {events.map((e) =>
              e[p.key] > 0 ? (
                <path
                  key={e.tick}
                  d={roundedTopBar(xOf(e.tick) + 1, p.yOf(e[p.key]), barW, p.top + PANEL_H - p.yOf(e[p.key]))}
                  fill={p.color}
                />
              ) : null,
            )}
          </g>
        ))}

        {/* lever annotations span both panels */}
        {flips.map((f) => (
          <g key={f.tick + f.label}>
            <line
              x1={xOf(f.tick) + barW / 2}
              x2={xOf(f.tick) + barW / 2}
              y1={M.top + 14}
              y2={M.top + PANEL_H * 2 + GAP}
              stroke={f.measured ? "var(--seal)" : "var(--baseline)"}
              strokeWidth="1"
              strokeDasharray="3 3"
            />
            <text className="annot-label" x={xOf(f.tick) + barW / 2 + 5} y={M.top + 22}>
              {f.label} · t={f.tick}
            </text>
          </g>
        ))}

        {/* x axis */}
        {[0, 15, 30, 45, 60, 75].map((t) =>
          t < nTicks ? (
            <text key={t} className="axis-label" x={xOf(t)} y={H - 10} textAnchor="middle">
              {t}
            </text>
          ) : null,
        )}
        <text className="axis-label" x={W - M.right} y={H - 10} textAnchor="end">
          tick
        </text>

        {/* crosshair */}
        {hover !== null && (
          <line
            x1={xOf(hover.tick) + barW / 2 + 1}
            x2={xOf(hover.tick) + barW / 2 + 1}
            y1={M.top}
            y2={M.top + PANEL_H * 2 + GAP}
            stroke="var(--muted)"
            strokeWidth="1"
          />
        )}
      </svg>

      {hovered && hover && (
        <div
          className="tooltip"
          style={{
            left: flipLeft ? undefined : hover.px + 14,
            right: flipLeft ? (wrapRef.current?.clientWidth ?? W) - hover.px + 14 : undefined,
            top: Math.max(0, hover.py - 40),
          }}
        >
          <div className="tooltip-title">tick {hovered.tick} · {(hovered.tick / 20).toFixed(2)} s</div>
          {SERIES.map((s) => (
            <div className="tooltip-row" key={s.key}>
              <i className="legend-swatch" style={{ background: s.color }} />
              {s.label.split(" ")[0]}
              <b>{hovered[s.key]}</b>
            </div>
          ))}
          <div className="tooltip-row">
            <i className="legend-swatch" style={{ background: "var(--baseline)" }} />
            Block changes
            <b>{hovered.changes}</b>
          </div>
        </div>
      )}
    </div>
  );
}

/** Bar with a 2px-rounded top (data end) and a square base on the baseline. */
function roundedTopBar(x: number, y: number, w: number, h: number): string {
  const r = Math.min(2, w / 2, h);
  return [
    `M ${x},${y + h}`,
    `L ${x},${y + r}`,
    `Q ${x},${y} ${x + r},${y}`,
    `L ${x + w - r},${y}`,
    `Q ${x + w},${y} ${x + w},${y + r}`,
    `L ${x + w},${y + h}`,
    "Z",
  ].join(" ");
}
