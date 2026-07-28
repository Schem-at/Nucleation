/** Mutation-rate trajectory sparkline — the anneal, made visible. One
 * series (no legend needed); text wears text tokens, the line carries the
 * series color. */

import { useMemo } from "react";
import type { RatePoint } from "../types";

interface Props {
  points: RatePoint[];
}

const W = 232;
const H = 44;
const PAD = { l: 2, r: 40, t: 4, b: 4 };

export default function RateSparkline({ points }: Props) {
  const model = useMemo(() => {
    if (points.length < 2) return null;
    // Sample long runs down so the path stays light.
    const stride = Math.max(1, Math.ceil(points.length / 240));
    const pts = points.filter(
      (_, i) => i % stride === 0 || i === points.length - 1,
    );
    const g0 = pts[0].gen;
    const g1 = pts[pts.length - 1].gen;
    const rMax = Math.max(...pts.map((p) => p.rate)) * 1.15 || 1;
    const x = (g: number) =>
      PAD.l + ((g - g0) / Math.max(1, g1 - g0)) * (W - PAD.l - PAD.r);
    const y = (r: number) => PAD.t + (1 - r / rMax) * (H - PAD.t - PAD.b);
    const d = pts
      .map((p, i) => `${i === 0 ? "M" : "L"}${x(p.gen).toFixed(1)},${y(p.rate).toFixed(1)}`)
      .join("");
    const last = pts[pts.length - 1];
    return { d, lastX: x(last.gen), lastY: y(last.rate), last };
  }, [points]);

  if (!model) return null;
  return (
    <svg
      className="rate-spark"
      viewBox={`0 0 ${W} ${H}`}
      data-testid="rate-sparkline"
      role="img"
      aria-label={`Mutation rate over the run, now ${model.last.rate.toFixed(3)} at generation ${model.last.gen}`}
    >
      <line
        x1={PAD.l}
        y1={H - PAD.b}
        x2={W - PAD.r}
        y2={H - PAD.b}
        className="spark-base"
      />
      <path d={model.d} className="spark-line" />
      <circle cx={model.lastX} cy={model.lastY} r={2.5} className="spark-dot" />
      <text
        x={model.lastX + 5}
        y={Math.max(9, model.lastY + 3)}
        className="spark-label"
      >
        {model.last.rate.toFixed(3)}
      </text>
    </svg>
  );
}
