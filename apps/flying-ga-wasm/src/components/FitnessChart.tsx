import { useMemo, useRef, useState } from "react";
import type { HistoryPoint } from "../types";
import { GAME_TPS } from "../metrics";
import { decimateHistory } from "../storage";

const W = 760;
const H = 260;
const PAD = { top: 14, right: 66, bottom: 30, left: 46 };
/** Points beyond this are decimated for rendering (uncapped runs). */
const RENDER_CAP = 1200;

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
  /** null = uncapped run: the x-domain follows the newest generation. */
  targetGenerations: number | null;
  /** Eval window in ticks — converts blocks to blk/s for the axis option. */
  evalTicks: number | null;
}

type Axis = "blocks" | "bps";

/** Fitness over generations: best + mean lines, crosshair hover, direct
 *  end labels, and an axis toggle between displacement (blocks) and speed
 *  (blk/s at 20 ticks/s — the sim is not realtime). Two series -> palette
 *  slots 1 (blue) and 2 (orange). */
export default function FitnessChart({ history, targetGenerations, evalTicks }: Props) {
  const wrapRef = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<number | null>(null);
  const [axis, setAxis] = useState<Axis>("blocks");

  const iw = W - PAD.left - PAD.right;
  const ih = H - PAD.top - PAD.bottom;

  /** blocks -> displayed unit. Speed = disp / (evalTicks / 20). */
  const k = axis === "bps" && evalTicks ? GAME_TPS / evalTicks : 1;
  const unit = axis === "bps" ? "BLK/S" : "BLOCKS";
  const fmt = (v: number) => (axis === "bps" ? v.toFixed(2) : v.toFixed(1));

  const drawn = useMemo(
    () => decimateHistory(history, RENDER_CAP),
    [history],
  );

  const { xOf, yOf, yTicks, xTicks, maxY } = useMemo(() => {
    const lastGen = drawn.length ? drawn[drawn.length - 1].gen : 0;
    const maxGen = Math.max(
      targetGenerations !== null ? targetGenerations - 1 : lastGen,
      10,
    );
    const rawMax = Math.max(k, ...drawn.map((h) => h.best * k));
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
  }, [drawn, targetGenerations, iw, ih, k]);

  if (drawn.length === 0) {
    return (
      <div className="chart-empty">
        No generations yet — configure a run and press Start.
      </div>
    );
  }

  const path = (key: "best" | "mean") =>
    drawn
      .map(
        (h, i) =>
          `${i === 0 ? "M" : "L"}${xOf(h.gen).toFixed(1)},${yOf(h[key] * k).toFixed(1)}`,
      )
      .join("");

  const last = drawn[drawn.length - 1];
  const hovered = hover !== null ? drawn[hover] : null;

  const onMove = (e: React.PointerEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const px = ((e.clientX - rect.left) / rect.width) * W;
    let best = 0;
    let bd = Infinity;
    drawn.forEach((h, i) => {
      const d = Math.abs(xOf(h.gen) - px);
      if (d < bd) {
        bd = d;
        best = i;
      }
    });
    setHover(best);
  };

  // Keep the two end labels from colliding.
  const bestLabY = yOf(last.best * k);
  let meanLabY = yOf(last.mean * k);
  if (meanLabY - bestLabY < 16) meanLabY = bestLabY + 16;

  const tipLeft = hovered ? (xOf(hovered.gen) / W) * 100 : 0;
  const tipFlip = hovered ? xOf(hovered.gen) > W * 0.62 : false;

  return (
    <div className="chart-wrap" ref={wrapRef}>
      <div
        className="axis-toggle"
        role="radiogroup"
        aria-label="Vertical axis unit"
      >
        <button
          role="radio"
          aria-checked={axis === "blocks"}
          className={axis === "blocks" ? "on" : ""}
          onClick={() => setAxis("blocks")}
        >
          blocks
        </button>
        <button
          role="radio"
          aria-checked={axis === "bps"}
          className={axis === "bps" ? "on" : ""}
          onClick={() => setAxis("bps")}
          disabled={!evalTicks}
          data-testid="axis-bps"
        >
          blk/s
        </button>
      </div>
      <svg
        className="chart-svg"
        viewBox={`0 0 ${W} ${H}`}
        role="img"
        aria-label={`Fitness over generations. Best ${fmt(last.best * k)} ${unit.toLowerCase()}, mean ${fmt(last.mean * k)} at generation ${last.gen}.`}
        onPointerMove={onMove}
        onPointerLeave={() => setHover(null)}
      >
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
        <text
          x={PAD.left}
          y={PAD.top - 3}
          fontSize={10}
          fontFamily="var(--font-mono)"
          fill="var(--muted)"
          letterSpacing="0.08em"
        >
          {unit}
        </text>

        <path d={path("mean")} fill="none" stroke="var(--series-mean)" strokeWidth={2} strokeLinejoin="round" />
        <path d={path("best")} fill="none" stroke="var(--series-best)" strokeWidth={2} strokeLinejoin="round" />

        <circle cx={xOf(last.gen)} cy={yOf(last.best * k)} r={3.5} fill="var(--series-best)" />
        <circle cx={xOf(last.gen)} cy={yOf(last.mean * k)} r={3.5} fill="var(--series-mean)" />
        <text x={xOf(last.gen) + 8} y={bestLabY + 3.5} fontSize={11} fontFamily="var(--font-mono)" fill="var(--ink)" fontWeight={700}>
          {fmt(last.best * k)}
        </text>
        <text x={xOf(last.gen) + 8} y={meanLabY + 3.5} fontSize={11} fontFamily="var(--font-mono)" fill="var(--ink-2)">
          {fmt(last.mean * k)}
        </text>

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
            <circle cx={xOf(hovered.gen)} cy={yOf(hovered.best * k)} r={4.5} fill="var(--series-best)" stroke="var(--surface)" strokeWidth={2} />
            <circle cx={xOf(hovered.gen)} cy={yOf(hovered.mean * k)} r={4.5} fill="var(--series-mean)" stroke="var(--surface)" strokeWidth={2} />
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
            <b>
              {fmt(hovered.best * k)} {axis === "bps" ? "blk/s" : ""}
            </b>
          </div>
          <div className="r">
            <span className="lbl">
              <span className="swatch" style={{ background: "var(--series-mean)" }} />
              mean
            </span>
            <b>
              {fmt(hovered.mean * k)} {axis === "bps" ? "blk/s" : ""}
            </b>
          </div>
        </div>
      )}
    </div>
  );
}
