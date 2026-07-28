import { useMemo, useRef, useState } from "react";
import type { HistoryPoint } from "../api";

const W = 760;
const H = 280;
const PAD = { top: 14, right: 66, bottom: 30, left: 46 };

function niceStep(raw: number): number {
  const pow = Math.pow(10, Math.floor(Math.log10(raw)));
  const n = raw / pow;
  if (n <= 1) return pow;
  if (n <= 2) return 2 * pow;
  if (n <= 5) return 5 * pow;
  return 10 * pow;
}

interface Props {
  history: HistoryPoint[];
  totalGenerations: number;
}

/** Fitness over generations: best + mean lines, crosshair hover, direct
 *  end labels. Two series -> palette slots 1 (blue) and 2 (orange). */
export default function FitnessChart({ history, totalGenerations }: Props) {
  const wrapRef = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<number | null>(null);

  const iw = W - PAD.left - PAD.right;
  const ih = H - PAD.top - PAD.bottom;

  const { xOf, yOf, yTicks, xTicks, maxY } = useMemo(() => {
    const maxGen = Math.max(totalGenerations - 1, 1);
    const rawMax = Math.max(1, ...history.map((h) => h.best));
    const yStep = niceStep(rawMax / 4);
    const maxY = Math.ceil((rawMax * 1.08) / yStep) * yStep;
    const xOf = (g: number) => PAD.left + (g / maxGen) * iw;
    const yOf = (v: number) => PAD.top + ih - (v / maxY) * ih;
    const yTicks: number[] = [];
    for (let v = 0; v <= maxY + 1e-9; v += yStep) yTicks.push(v);
    const xStep = niceStep(maxGen / 6);
    const xTicks: number[] = [];
    for (let g = 0; g <= maxGen + 1e-9; g += xStep) xTicks.push(Math.round(g));
    return { xOf, yOf, yTicks, xTicks, maxY };
  }, [history, totalGenerations, iw, ih]);

  if (history.length === 0) {
    return (
      <div className="chart-empty">
        No generations yet — configure a run and press Start.
      </div>
    );
  }

  const path = (key: "best" | "mean") =>
    history
      .map((h, i) => `${i === 0 ? "M" : "L"}${xOf(h.gen).toFixed(1)},${yOf(h[key]).toFixed(1)}`)
      .join("");

  const last = history[history.length - 1];
  const hovered = hover !== null ? history[hover] : null;

  const onMove = (e: React.PointerEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const px = ((e.clientX - rect.left) / rect.width) * W;
    const maxGen = Math.max(totalGenerations - 1, 1);
    const g = ((px - PAD.left) / iw) * maxGen;
    let best = 0;
    let bd = Infinity;
    history.forEach((h, i) => {
      const d = Math.abs(h.gen - g);
      if (d < bd) {
        bd = d;
        best = i;
      }
    });
    setHover(best);
  };

  // Keep the two end labels from colliding.
  const bestLabY = yOf(last.best);
  let meanLabY = yOf(last.mean);
  if (meanLabY - bestLabY < 16) meanLabY = bestLabY + 16;

  // Tooltip placement in % of container.
  const tipLeft = hovered ? (xOf(hovered.gen) / W) * 100 : 0;
  const tipFlip = hovered ? xOf(hovered.gen) > W * 0.62 : false;

  return (
    <div className="chart-wrap" ref={wrapRef}>
      <svg
        className="chart-svg"
        viewBox={`0 0 ${W} ${H}`}
        role="img"
        aria-label={`Fitness over generations. Best ${last.best}, mean ${last.mean} at generation ${last.gen}.`}
        onPointerMove={onMove}
        onPointerLeave={() => setHover(null)}
      >
        {/* horizontal gridlines */}
        {yTicks.map((v) => (
          <g key={v}>
            <line
              x1={PAD.left}
              x2={W - PAD.right}
              y1={yOf(v)}
              y2={yOf(v)}
              stroke={v === 0 ? "var(--baseline)" : "var(--hairline)"}
              strokeWidth={1}
            />
            <text
              x={PAD.left - 8}
              y={yOf(v) + 3.5}
              textAnchor="end"
              fontSize={10.5}
              fontFamily="var(--font-mono)"
              fill="var(--muted)"
            >
              {maxY >= 10 ? Math.round(v) : v}
            </text>
          </g>
        ))}
        {/* x ticks */}
        {xTicks.map((g) => (
          <text
            key={g}
            x={xOf(g)}
            y={H - PAD.bottom + 16}
            textAnchor="middle"
            fontSize={10.5}
            fontFamily="var(--font-mono)"
            fill="var(--muted)"
          >
            {g}
          </text>
        ))}
        <text
          x={W - PAD.right}
          y={H - 4}
          textAnchor="end"
          fontSize={10}
          fontFamily="var(--font-mono)"
          fill="var(--muted)"
          letterSpacing="0.08em"
        >
          GENERATION
        </text>

        <path d={path("mean")} fill="none" stroke="var(--series-mean)" strokeWidth={2} strokeLinejoin="round" />
        <path d={path("best")} fill="none" stroke="var(--series-best)" strokeWidth={2} strokeLinejoin="round" />

        {/* direct end labels: text in ink, identity via swatch dot */}
        <circle cx={xOf(last.gen)} cy={yOf(last.best)} r={3.5} fill="var(--series-best)" />
        <circle cx={xOf(last.gen)} cy={yOf(last.mean)} r={3.5} fill="var(--series-mean)" />
        <text x={xOf(last.gen) + 8} y={bestLabY + 3.5} fontSize={11} fontFamily="var(--font-mono)" fill="var(--ink)" fontWeight={700}>
          {last.best.toFixed(1)}
        </text>
        <text x={xOf(last.gen) + 8} y={meanLabY + 3.5} fontSize={11} fontFamily="var(--font-mono)" fill="var(--ink-2)">
          {last.mean.toFixed(1)}
        </text>

        {/* crosshair */}
        {hovered && (
          <g pointerEvents="none">
            <line
              x1={xOf(hovered.gen)}
              x2={xOf(hovered.gen)}
              y1={PAD.top}
              y2={H - PAD.bottom}
              stroke="var(--baseline)"
              strokeWidth={1}
            />
            <circle cx={xOf(hovered.gen)} cy={yOf(hovered.best)} r={4.5} fill="var(--series-best)" stroke="var(--surface)" strokeWidth={2} />
            <circle cx={xOf(hovered.gen)} cy={yOf(hovered.mean)} r={4.5} fill="var(--series-mean)" stroke="var(--surface)" strokeWidth={2} />
          </g>
        )}
      </svg>

      {hovered && (
        <div
          className="chart-tip"
          style={{
            left: `${tipLeft}%`,
            top: 8,
            transform: tipFlip ? "translateX(calc(-100% - 12px))" : "translateX(12px)",
          }}
        >
          <div className="t">gen {hovered.gen}</div>
          <div className="r">
            <span className="lbl">
              <span className="swatch" style={{ background: "var(--series-best)" }} />
              best
            </span>
            <b>{hovered.best.toFixed(1)}</b>
          </div>
          <div className="r">
            <span className="lbl">
              <span className="swatch" style={{ background: "var(--series-mean)" }} />
              mean
            </span>
            <b>{hovered.mean.toFixed(1)}</b>
          </div>
        </div>
      )}
    </div>
  );
}
