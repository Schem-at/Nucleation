/** QD progress: two stacked small multiples over the same generation axis —
 * QD-score (sum of cell qualities) and archive fill rate. Deliberately NOT
 * one chart with two y-scales: the two measures have different units and a
 * dual axis would let their crossings imply a relationship that isn't there.
 * A shared crosshair ties them back together on hover. */

import { useMemo, useState } from "react";
import { QUALITIES, type QualityKey } from "../metrics";
import type { HistoryPoint } from "../types";

const W = 760;
const ROW = 108;
const PAD = { top: 12, right: 18, bottom: 22, left: 54 };

function niceStep(raw: number): number {
  const pow = Math.pow(10, Math.floor(Math.log10(Math.max(raw, 1e-9))));
  const n = raw / pow;
  if (n <= 1) return pow;
  if (n <= 2) return 2 * pow;
  if (n <= 5) return 5 * pow;
  return 10 * pow;
}

interface Props {
  history: HistoryPoint[];
  quality: QualityKey;
}

export default function QdChart({ history, quality }: Props) {
  const [hoverGen, setHoverGen] = useState<number | null>(null);
  const pts = useMemo(
    () => history.filter((p) => p.qd !== undefined && p.fill !== undefined),
    [history],
  );

  if (pts.length < 2) {
    return (
      <div className="chart-empty" data-testid="qd-empty">
        QD-score and fill rate plot from the second generation.
      </div>
    );
  }

  const g0 = pts[0].gen;
  const g1 = Math.max(pts[pts.length - 1].gen, g0 + 1);
  const H = PAD.top + ROW * 2 + 18 + PAD.bottom;
  const iw = W - PAD.left - PAD.right;
  const xOf = (g: number) => PAD.left + ((g - g0) / (g1 - g0)) * iw;

  const rows: Array<{
    label: string;
    unit: string;
    color: string;
    top: number;
    max: number;
    fmt: (v: number) => string;
    value: (p: HistoryPoint) => number;
    testid: string;
  }> = [
    {
      label: "QD-score",
      unit: `${QUALITIES[quality].label} summed over cells`,
      color: "var(--series-best)",
      top: PAD.top,
      max: Math.max(...pts.map((p) => p.qd!), 1e-6),
      // One precision for the whole axis — mixing "1000" with "0.0" reads
      // as two different scales.
      fmt: (v) =>
        Math.max(...pts.map((p) => p.qd!)) >= 100 ? v.toFixed(0) : v.toFixed(1),
      value: (p) => p.qd!,
      testid: "qd-row",
    },
    {
      label: "fill rate",
      unit: "% of cells illuminated",
      color: "var(--series-mean)",
      top: PAD.top + ROW + 18,
      max: Math.max(...pts.map((p) => p.fill! * 100), 1e-6),
      fmt: (v) => `${v.toFixed(0)}%`,
      value: (p) => p.fill! * 100,
      testid: "fill-row",
    },
  ];

  const hovered = hoverGen === null ? null : pts.reduce((a, b) =>
    Math.abs(b.gen - hoverGen) < Math.abs(a.gen - hoverGen) ? b : a,
  );

  return (
    <div className="chart-wrap">
      <svg
        className="chart-svg"
        viewBox={`0 0 ${W} ${H}`}
        role="img"
        aria-label={`Quality-diversity progress: QD-score and archive fill rate over ${pts.length} generations.`}
        onPointerMove={(e) => {
          const r = (e.target as SVGElement).ownerSVGElement?.getBoundingClientRect();
          if (!r) return;
          const px = ((e.clientX - r.left) / r.width) * W;
          setHoverGen(g0 + ((px - PAD.left) / iw) * (g1 - g0));
        }}
        onPointerLeave={() => setHoverGen(null)}
        data-testid="qd-chart"
      >
        {rows.map((row) => {
          const yOf = (v: number) =>
            row.top + ROW - (v / row.max) * (ROW - 14);
          const step = niceStep(row.max / 2);
          const ticks: number[] = [];
          for (let v = 0; v <= row.max + 1e-9; v += step) ticks.push(v);
          const line = pts
            .map(
              (p, i) =>
                `${i === 0 ? "M" : "L"}${xOf(p.gen).toFixed(1)},${yOf(row.value(p)).toFixed(1)}`,
            )
            .join("");
          const last = pts[pts.length - 1];
          return (
            <g key={row.label} data-testid={row.testid}>
              {ticks.map((v) => (
                <g key={v}>
                  <line
                    x1={PAD.left}
                    x2={W - PAD.right}
                    y1={yOf(v)}
                    y2={yOf(v)}
                    stroke="var(--hairline)"
                    strokeWidth={1}
                  />
                  <text
                    x={PAD.left - 8}
                    y={yOf(v) + 3.5}
                    textAnchor="end"
                    fontSize={10}
                    fontFamily="var(--font-mono)"
                    fill="var(--muted)"
                  >
                    {row.fmt(v)}
                  </text>
                </g>
              ))}
              <path
                d={line}
                fill="none"
                stroke={row.color}
                strokeWidth={2}
                strokeLinejoin="round"
                strokeLinecap="round"
                pointerEvents="none"
              />
              <circle
                cx={xOf(last.gen)}
                cy={yOf(row.value(last))}
                r={4}
                fill={row.color}
                stroke="var(--surface)"
                strokeWidth={2}
                pointerEvents="none"
              />
              <text
                x={PAD.left}
                y={row.top - 2}
                fontSize={10}
                fontFamily="var(--font-mono)"
                fill="var(--muted)"
                letterSpacing="0.08em"
              >
                {row.label.toUpperCase()} — {row.unit.toUpperCase()}
              </text>
              {hovered && (
                <circle
                  cx={xOf(hovered.gen)}
                  cy={yOf(row.value(hovered))}
                  r={4}
                  fill="var(--surface)"
                  stroke={row.color}
                  strokeWidth={2}
                  pointerEvents="none"
                />
              )}
            </g>
          );
        })}
        {hovered && (
          <line
            x1={xOf(hovered.gen)}
            x2={xOf(hovered.gen)}
            y1={PAD.top}
            y2={PAD.top + ROW * 2 + 18}
            stroke="var(--baseline)"
            strokeWidth={1}
            pointerEvents="none"
          />
        )}
        <text
          x={PAD.left}
          y={H - 4}
          fontSize={10}
          fontFamily="var(--font-mono)"
          fill="var(--muted)"
        >
          {g0}
        </text>
        <text
          x={(PAD.left + W - PAD.right) / 2}
          y={H - 4}
          textAnchor="middle"
          fontSize={10}
          fontFamily="var(--font-mono)"
          fill="var(--muted)"
          letterSpacing="0.08em"
        >
          GENERATION
        </text>
        <text
          x={W - PAD.right}
          y={H - 4}
          textAnchor="end"
          fontSize={10}
          fontFamily="var(--font-mono)"
          fill="var(--muted)"
        >
          {pts[pts.length - 1].gen}
        </text>
      </svg>

      {hovered && (
        <div
          className="chart-tip"
          style={{
            left: `${(xOf(hovered.gen) / W) * 100}%`,
            top: 8,
            transform:
              xOf(hovered.gen) > W * 0.6
                ? "translateX(calc(-100% - 12px))"
                : "translateX(12px)",
          }}
        >
          <div className="t">gen {hovered.gen}</div>
          <div className="r">
            <span className="lbl">QD-score</span>
            <b>{hovered.qd!.toFixed(1)}</b>
          </div>
          <div className="r">
            <span className="lbl">fill rate</span>
            <b>{(hovered.fill! * 100).toFixed(1)}%</b>
          </div>
        </div>
      )}
    </div>
  );
}
