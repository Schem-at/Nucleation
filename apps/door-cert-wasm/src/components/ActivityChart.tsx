import { useRef, useState } from "react";
import type { ReactElement } from "react";
import type { LeverFlip, TickEvents } from "../lib/types";

// Per-tick activity, stacked. The question this chart answers is "when does
// the machine go quiet", and that is a question about the TOTAL — so the
// series share one bar per tick and the stack height is the answer. Two
// aligned panels made you add them up in your head.
//
// Series are the first three categorical slots, validated against both
// surfaces. Items is carried at zero until the engine reports item movement.
const W = 1000;
const PLOT_H = 190;
const M = { top: 26, right: 14, bottom: 44, left: 46 };
const H = M.top + PLOT_H + M.bottom;

type Key = "piston" | "redstone" | "items";
type Series = { key: Key; label: string; short: string; color: string };

/** Stacking order is fixed: pistons are the mass and sit on the baseline,
 *  the signal rides above them. */
const SERIES: Series[] = [
  { key: "piston", label: "Piston events", short: "Piston", color: "var(--series-piston)" },
  { key: "redstone", label: "Redstone events", short: "Redstone", color: "var(--series-redstone)" },
  { key: "items", label: "Item events", short: "Items", color: "var(--series-items)" },
];

function niceTicks(max: number): number[] {
  const raw = max / 4;
  const mag = Math.pow(10, Math.floor(Math.log10(Math.max(1, raw))));
  const step = [1, 2, 2.5, 5, 10].map((s) => s * mag).find((s) => s >= raw) ?? mag * 10;
  const out: number[] = [];
  for (let v = step; v <= max; v += step) out.push(Math.round(v));
  return out;
}

export function ActivityChart({ events, flips }: { events: TickEvents[]; flips: LeverFlip[] }) {
  const wrapRef = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<{ tick: number; px: number; py: number } | null>(null);

  const nTicks = Math.max(1, events.length);
  const plotW = W - M.left - M.right;
  const xOf = (t: number) => M.left + (t / nTicks) * plotW;
  const barW = Math.max(2, plotW / nTicks - 2); // 2px surface gap between bars

  const totalOf = (e: TickEvents) => SERIES.reduce((a, s) => a + (e[s.key] || 0), 0);
  const max = Math.max(1, ...events.map(totalOf));
  const yOf = (v: number) => M.top + PLOT_H - (v / max) * PLOT_H;
  const totals = SERIES.map((s) => events.reduce((a, e) => a + (e[s.key] || 0), 0));

  // The point of the trace is when the machine stops. Every silent tick is
  // shaded, and each stroke gets a marker on the tick after its last event —
  // the moment the door is actually done moving, which is not the same as
  // the moment the lever was thrown.
  const quiet: { from: number; to: number }[] = [];
  for (let i = 0; i < events.length; i++) {
    if (totalOf(events[i]) > 0) continue;
    let j = i;
    while (j + 1 < events.length && totalOf(events[j + 1]) === 0) j++;
    quiet.push({ from: i, to: j });
    i = j;
  }

  const bounds = [...new Set(flips.map((f) => f.tick))].sort((a, b) => a - b);
  const strokes = bounds.map((start, i) => ({
    start,
    end: i + 1 < bounds.length ? bounds[i + 1] - 1 : events.length - 1,
  }));
  const settled = strokes
    .map((s) => {
      let last = -1;
      for (let t = s.start; t <= s.end && t < events.length; t++)
        if (totalOf(events[t]) > 0) last = t;
      return last < 0 ? null : { at: last + 1, start: s.start };
    })
    .filter(Boolean) as { at: number; start: number }[];

  // Direct labels: each series is called out at its own busiest tick, so
  // identity never rests on colour alone.
  const callouts = SERIES.map((s, i) => {
    let peak = -1;
    let peakV = 0;
    for (const e of events)
      if ((e[s.key] || 0) > peakV) {
        peakV = e[s.key] || 0;
        peak = e.tick;
      }
    if (peak < 0) return null;
    // stack base at that tick
    const e = events[peak];
    let below = 0;
    for (let j = 0; j < i; j++) below += e[SERIES[j].key] || 0;
    const mid = yOf(below + peakV / 2);
    const right = xOf(peak) + barW / 2 < W * 0.6;
    return { ...s, peak, peakV, mid, right, x: xOf(peak) + barW / 2 };
  }).filter(Boolean) as {
    key: Key;
    short: string;
    color: string;
    peak: number;
    peakV: number;
    mid: number;
    right: boolean;
    x: number;
  }[];
  // Two labels stacked on the same row collide; nudge the second one down.
  for (let i = 1; i < callouts.length; i++)
    if (Math.abs(callouts[i].mid - callouts[i - 1].mid) < 16)
      callouts[i].mid = callouts[i - 1].mid + 16;

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
  const flipLeft = hover !== null && hover.px > (wrapRef.current?.clientWidth ?? W) - 200;

  return (
    <>
      <div className="legend">
        {SERIES.map((s, i) => (
          <span className={"item" + (totals[i] === 0 ? " zero" : "")} key={s.key}>
            <i className="legend-swatch" style={{ background: s.color }} />
            {s.label}
            <b>{totals[i]}</b>
            {totals[i] === 0 && " (not yet reported)"}
          </span>
        ))}
      </div>
      <div
        className="chart-svg-wrap"
        ref={wrapRef}
        onMouseMove={onMove}
        onMouseLeave={() => setHover(null)}
      >
        <svg
          viewBox={`0 0 ${W} ${H}`}
          aria-label="Piston, redstone and item events per simulation tick, stacked"
        >
          {/* gridlines + y labels */}
          {niceTicks(max).map((v) => (
            <g key={v}>
              <line
                x1={M.left}
                x2={W - M.right}
                y1={yOf(v)}
                y2={yOf(v)}
                stroke="var(--grid)"
                strokeWidth="1"
              />
              <text className="axis-label" x={M.left - 8} y={yOf(v) + 3} textAnchor="end">
                {v}
              </text>
            </g>
          ))}
          <line
            x1={M.left}
            x2={W - M.right}
            y1={M.top + PLOT_H}
            y2={M.top + PLOT_H}
            stroke="var(--baseline)"
            strokeWidth="1"
          />
          <text className="panel-label" x={M.left} y={M.top - 10}>
            Events per tick
          </text>

          {/* the settled runs, shaded because they are the point */}
          {quiet.map((r) => (
            <rect
              key={r.from}
              x={xOf(r.from)}
              y={M.top}
              width={Math.max(1, xOf(r.to + 1) - xOf(r.from))}
              height={PLOT_H}
              fill="var(--surface-2)"
            />
          ))}

          {/* stacked bars: 2px surface gap between segments, rounded data end */}
          {events.map((e) => {
            const segs: ReactElement[] = [];
            let acc = 0;
            let topIndex = -1;
            SERIES.forEach((s, i) => {
              if ((e[s.key] || 0) > 0) topIndex = i;
            });
            SERIES.forEach((s, i) => {
              const v = e[s.key] || 0;
              if (v <= 0) return;
              const y0 = yOf(acc);
              const y1 = yOf(acc + v);
              acc += v;
              const gap = i === topIndex ? 0 : 2;
              const h = Math.max(1, y0 - y1 - gap);
              segs.push(
                i === topIndex ? (
                  <path
                    key={s.key}
                    d={roundedTopBar(xOf(e.tick) + 1, y1, barW, h)}
                    fill={s.color}
                  />
                ) : (
                  <rect
                    key={s.key}
                    x={xOf(e.tick) + 1}
                    y={y1 + gap}
                    width={barW}
                    height={h}
                    fill={s.color}
                  />
                ),
              );
            });
            return segs.length ? <g key={e.tick}>{segs}</g> : null;
          })}

          {/* lever annotations */}
          {flips.map((f) => (
            <g key={f.tick + f.label}>
              <line
                x1={xOf(f.tick) + barW / 2}
                x2={xOf(f.tick) + barW / 2}
                y1={M.top}
                y2={M.top + PLOT_H}
                stroke="var(--ink-2)"
                strokeWidth="1"
                strokeDasharray="3 3"
              />
              <text className="annot-label" x={xOf(f.tick) + barW / 2 + 5} y={M.top + 10}>
                {f.label} · t={f.tick}
              </text>
            </g>
          ))}

          {/* direct labels — identity without relying on colour */}
          {callouts.map((c) => (
            <g key={c.key}>
              <line
                x1={c.x}
                x2={c.right ? c.x + 12 : c.x - 12}
                y1={c.mid}
                y2={c.mid}
                stroke="var(--baseline)"
                strokeWidth="1"
              />
              <text
                className="callout-label"
                x={c.right ? c.x + 17 : c.x - 17}
                y={c.mid + 3.5}
                textAnchor={c.right ? "start" : "end"}
              >
                {c.short} peak {c.peakV}
              </text>
            </g>
          ))}

          {/* where each stroke actually finishes */}
          {settled.map((s) => {
            const x = Math.min(xOf(s.at), W - M.right);
            const left = x > W * 0.7;
            return (
              <g key={s.start}>
                <path d={`M ${x},${M.top + PLOT_H + 2} l 5,7 l -10,0 Z`} fill="var(--accent)" />
                <text
                  className="annot-label"
                  x={left ? x - 9 : x + 9}
                  y={M.top + PLOT_H + 12}
                  textAnchor={left ? "end" : "start"}
                  fill="var(--ink-2)"
                >
                  quiet at t={s.at} · {s.at - s.start} ticks
                </text>
              </g>
            );
          })}

          {/* x axis */}
          {axisTicks(nTicks).map((t) => (
            <text key={t} className="axis-label" x={xOf(t)} y={H - 8} textAnchor="middle">
              {t}
            </text>
          ))}
          <text className="axis-label" x={W - M.right} y={H - 8} textAnchor="end">
            tick
          </text>

          {/* crosshair */}
          {hover !== null && (
            <line
              x1={xOf(hover.tick) + barW / 2 + 1}
              x2={xOf(hover.tick) + barW / 2 + 1}
              y1={M.top}
              y2={M.top + PLOT_H}
              stroke="var(--ink-2)"
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
            <div className="tooltip-title">
              tick {hovered.tick} · {(hovered.tick / 20).toFixed(2)} s
            </div>
            {SERIES.map((s) => (
              <div className="tooltip-row" key={s.key}>
                <i className="legend-swatch" style={{ background: s.color }} />
                {s.short}
                <b>{hovered[s.key] || 0}</b>
              </div>
            ))}
            <div className="tooltip-row total">
              Total
              <b>{totalOf(hovered)}</b>
            </div>
          </div>
        )}
      </div>
    </>
  );
}

function axisTicks(n: number): number[] {
  const step = n <= 20 ? 5 : n <= 60 ? 10 : n <= 140 ? 20 : 50;
  const out: number[] = [];
  for (let t = 0; t < n; t += step) out.push(t);
  return out;
}

/** Bar with a rounded top (data end) and a square base on the stack below. */
function roundedTopBar(x: number, y: number, w: number, h: number): string {
  const r = Math.min(4, w / 2, h);
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
