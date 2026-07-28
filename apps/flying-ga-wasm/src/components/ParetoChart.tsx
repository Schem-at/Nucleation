/** Pareto front scatter for two selected objectives. The archive (all
 * non-dominated machines found so far) is the highlighted series joined by
 * a step line; this generation's population fliers are a recessive cloud.
 * Archive points are clickable — they load the machine into the viewer and
 * onto the flight stage. */

import { useMemo, useState } from "react";
import {
  dirOf,
  OBJECTIVES,
  type EvalMetrics,
  type ObjectiveChoice,
} from "../metrics";
import type { LeaderboardEntry } from "../types";

const W = 760;
const H = 300;
const PAD = { top: 16, right: 24, bottom: 40, left: 54 };

function niceStep(raw: number): number {
  const pow = Math.pow(10, Math.floor(Math.log10(Math.max(raw, 1e-9))));
  const n = raw / pow;
  if (n <= 1) return pow;
  if (n <= 2) return 2 * pow;
  if (n <= 5) return 5 * pow;
  return 10 * pow;
}

interface Props {
  archive: LeaderboardEntry[];
  cloud: EvalMetrics[];
  choices: [ObjectiveChoice, ObjectiveChoice];
  selectedId: string | null;
  onPick: (id: string) => void;
}

export default function ParetoChart({
  archive,
  cloud,
  choices,
  selectedId,
  onPick,
}: Props) {
  const [hover, setHover] = useState<string | null>(null);
  const [xDef, yDef] = [OBJECTIVES[choices[0].key], OBJECTIVES[choices[1].key]];
  const [xDir, yDir] = [dirOf(choices[0]), dirOf(choices[1])];

  const front = useMemo(
    () => archive.filter((e) => e.metrics),
    [archive],
  );

  const scale = useMemo(() => {
    const xs: number[] = [];
    const ys: number[] = [];
    for (const e of front) {
      xs.push(xDef.value(e.metrics!));
      ys.push(yDef.value(e.metrics!));
    }
    for (const m of cloud) {
      xs.push(xDef.value(m));
      ys.push(yDef.value(m));
    }
    if (xs.length === 0) return null;
    const pad = (lo: number, hi: number): [number, number] => {
      const span = Math.max(hi - lo, 1e-6);
      return [Math.max(0, lo - span * 0.08), hi + span * 0.1];
    };
    const [x0, x1] = pad(Math.min(...xs), Math.max(...xs));
    const [y0, y1] = pad(Math.min(...ys), Math.max(...ys));
    const iw = W - PAD.left - PAD.right;
    const ih = H - PAD.top - PAD.bottom;
    const xOf = (v: number) => PAD.left + ((v - x0) / (x1 - x0)) * iw;
    const yOf = (v: number) => PAD.top + ih - ((v - y0) / (y1 - y0)) * ih;
    const xStep = niceStep((x1 - x0) / 5);
    const yStep = niceStep((y1 - y0) / 4);
    const xTicks: number[] = [];
    for (let v = Math.ceil(x0 / xStep) * xStep; v <= x1 + 1e-9; v += xStep)
      xTicks.push(v);
    const yTicks: number[] = [];
    for (let v = Math.ceil(y0 / yStep) * yStep; v <= y1 + 1e-9; v += yStep)
      yTicks.push(v);
    return { xOf, yOf, xTicks, yTicks };
  }, [front, cloud, xDef, yDef]);

  if (!scale || front.length === 0) {
    return (
      <div className="chart-empty" data-testid="pareto-empty">
        The Pareto front appears once the first machine flies.
      </div>
    );
  }

  // Step line through the front, sorted along x in the "better" direction.
  const sorted = [...front].sort(
    (a, b) => xDef.value(a.metrics!) - xDef.value(b.metrics!),
  );
  const stepPath = sorted
    .map((e, i) => {
      const x = scale.xOf(xDef.value(e.metrics!)).toFixed(1);
      const y = scale.yOf(yDef.value(e.metrics!)).toFixed(1);
      if (i === 0) return `M${x},${y}`;
      const py = scale.yOf(yDef.value(sorted[i - 1].metrics!)).toFixed(1);
      return `L${x},${py}L${x},${y}`;
    })
    .join("");

  const hoveredEntry = hover ? front.find((e) => e.id === hover) ?? null : null;
  const tipLeft = hoveredEntry
    ? (scale.xOf(xDef.value(hoveredEntry.metrics!)) / W) * 100
    : 0;
  const tipFlip = hoveredEntry
    ? scale.xOf(xDef.value(hoveredEntry.metrics!)) > W * 0.6
    : false;

  const dirNote = (d: "min" | "max") =>
    d === "max" ? "more is better" : "fewer is better";

  return (
    <div className="chart-wrap">
      <svg
        className="chart-svg"
        viewBox={`0 0 ${W} ${H}`}
        role="img"
        aria-label={`Pareto front: ${xDef.label} vs ${yDef.label}. ${front.length} non-dominated machines.`}
        onPointerLeave={() => setHover(null)}
      >
        {scale.yTicks.map((v) => (
          <g key={`y${v}`}>
            <line
              x1={PAD.left}
              x2={W - PAD.right}
              y1={scale.yOf(v)}
              y2={scale.yOf(v)}
              stroke="var(--hairline)"
              strokeWidth={1}
            />
            <text
              x={PAD.left - 8}
              y={scale.yOf(v) + 3.5}
              textAnchor="end"
              fontSize={10.5}
              fontFamily="var(--font-mono)"
              fill="var(--muted)"
            >
              {yDef.fmt(v)}
            </text>
          </g>
        ))}
        {scale.xTicks.map((v) => (
          <g key={`x${v}`}>
            <line
              x1={scale.xOf(v)}
              x2={scale.xOf(v)}
              y1={PAD.top}
              y2={H - PAD.bottom}
              stroke="var(--hairline)"
              strokeWidth={1}
            />
            <text
              x={scale.xOf(v)}
              y={H - PAD.bottom + 16}
              textAnchor="middle"
              fontSize={10.5}
              fontFamily="var(--font-mono)"
              fill="var(--muted)"
            >
              {xDef.fmt(v)}
            </text>
          </g>
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
          {xDef.label.toUpperCase()} ({xDef.unit.toUpperCase()}) — {dirNote(xDir).toUpperCase()}
        </text>
        <text
          x={PAD.left}
          y={PAD.top - 4}
          fontSize={10}
          fontFamily="var(--font-mono)"
          fill="var(--muted)"
          letterSpacing="0.08em"
        >
          {yDef.label.toUpperCase()} ({yDef.unit.toUpperCase()}) — {dirNote(yDir).toUpperCase()}
        </text>

        {/* Population cloud — this generation's fliers, recessive. */}
        {cloud.map((m, i) => (
          <circle
            key={`c${i}`}
            cx={scale.xOf(xDef.value(m))}
            cy={scale.yOf(yDef.value(m))}
            r={3}
            fill="var(--muted)"
            opacity={0.35}
            pointerEvents="none"
          />
        ))}

        {/* Front step line + points. */}
        <path
          d={stepPath}
          fill="none"
          stroke="var(--series-best)"
          strokeWidth={1.5}
          opacity={0.55}
          pointerEvents="none"
        />
        {front.map((e) => {
          const x = scale.xOf(xDef.value(e.metrics!));
          const y = scale.yOf(yDef.value(e.metrics!));
          const active = e.id === selectedId;
          return (
            <g key={e.id}>
              {/* invisible fat hit target */}
              <circle
                cx={x}
                cy={y}
                r={12}
                fill="transparent"
                style={{ cursor: "pointer" }}
                onPointerEnter={() => setHover(e.id)}
                onClick={() => onPick(e.id)}
                data-testid="pareto-point"
                aria-label={`Machine ${e.name ?? e.id}: ${xDef.fmt(xDef.value(e.metrics!))} ${xDef.unit}, ${yDef.fmt(yDef.value(e.metrics!))} ${yDef.unit}`}
                role="button"
              />
              <circle
                cx={x}
                cy={y}
                r={active ? 7 : 5.5}
                fill="var(--series-best)"
                stroke="var(--surface)"
                strokeWidth={2}
                pointerEvents="none"
              />
              {active && (
                <circle
                  cx={x}
                  cy={y}
                  r={10.5}
                  fill="none"
                  stroke="var(--series-best)"
                  strokeWidth={1.5}
                  pointerEvents="none"
                />
              )}
            </g>
          );
        })}
      </svg>

      {hoveredEntry && hoveredEntry.metrics && (
        <div
          className="chart-tip"
          style={{
            left: `${tipLeft}%`,
            top: 8,
            transform: tipFlip
              ? "translateX(calc(-100% - 12px))"
              : "translateX(12px)",
          }}
        >
          <div className="t">
            {hoveredEntry.name ?? hoveredEntry.id} · gen {hoveredEntry.gen}
          </div>
          <div className="r">
            <span className="lbl">{xDef.label}</span>
            <b>
              {xDef.fmt(xDef.value(hoveredEntry.metrics))} {xDef.unit}
            </b>
          </div>
          <div className="r">
            <span className="lbl">{yDef.label}</span>
            <b>
              {yDef.fmt(yDef.value(hoveredEntry.metrics))} {yDef.unit}
            </b>
          </div>
          <div className="r">
            <span className="lbl">distance</span>
            <b>{hoveredEntry.fitness.toFixed(1)} blocks</b>
          </div>
        </div>
      )}

      <div className="chart-legend pareto-legend" role="list" aria-label="Series">
        <span className="item" role="listitem">
          <span className="swatch" style={{ background: "var(--series-best)" }} />
          Pareto front ({front.length})
        </span>
        <span className="item" role="listitem">
          <span className="swatch dim" style={{ background: "var(--muted)" }} />
          this generation's fliers
        </span>
        <span className="item note-item">click a front point to inspect it</span>
      </div>
    </div>
  );
}
