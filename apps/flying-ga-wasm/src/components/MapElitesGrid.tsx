/** The illuminated grid: the MAP-Elites archive drawn as a heat-grid over
 * two behaviour dimensions. Sequential blue ramp on quality (one hue,
 * light→dark on the light surface, anchor-flipped on dark). Empty cells are
 * NOT a ramp step — they're a recessed, unfilled void, so "not discovered
 * yet" can never be mistaken for "discovered, low quality". Clicking a lit
 * cell stages its machine on the flight loop. */

import { useMemo, useState } from "react";
import { binSpanLabel, type GridCell, type GridSnapshot } from "../ga/mapelites";
import { BEHAVIOURS, QUALITIES } from "../metrics";

/** Sequential ramp, 7 steps, near-zero → high. Light and dark are chosen
 * per mode against their own surface, not flipped programmatically. */
const RAMP_STEPS = 7;

interface Props {
  grid: GridSnapshot;
  /** Id of the machine currently staged / selected, if it is an elite. */
  selectedId: string | null;
  onPick: (cell: GridCell) => void;
}

const CELL = 26;
const GAP = 2;
const PAD = { top: 14, right: 14, bottom: 42, left: 62 };

export default function MapElitesGrid({ grid, selectedId, onPick }: Props) {
  const [hover, setHover] = useState<GridCell | null>(null);
  const { spec } = grid;
  const xDef = BEHAVIOURS[spec.x];
  const yDef = BEHAVIOURS[spec.y];
  const qDef = QUALITIES[spec.quality];

  const byKey = useMemo(() => {
    const m = new Map<number, GridCell>();
    for (const c of grid.cells) m.set(c.iy * spec.binsX + c.ix, c);
    return m;
  }, [grid.cells, spec.binsX]);

  // Ramp step by quality. Quantile-free (a linear split on the observed
  // max) keeps the legend readable as a number line; the ramp's own
  // lightness does the ordering work.
  const maxQ = Math.max(grid.bestQuality, 1e-6);
  const stepOf = (q: number) =>
    Math.max(
      0,
      Math.min(RAMP_STEPS - 1, Math.floor((Math.max(0, q) / maxQ) * RAMP_STEPS)),
    );

  const W = PAD.left + spec.binsX * (CELL + GAP) + PAD.right;
  const H = PAD.top + spec.binsY * (CELL + GAP) + PAD.bottom;

  const xAt = (ix: number) => PAD.left + ix * (CELL + GAP);
  // y grows UP the chart: bin 0 at the bottom, like every other axis here.
  const yAt = (iy: number) => PAD.top + (spec.binsY - 1 - iy) * (CELL + GAP);

  const xTickEvery = Math.max(1, Math.ceil(spec.binsX / 8));
  const yTickEvery = Math.max(1, Math.ceil(spec.binsY / 6));
  const xw = (grid.xRange[1] - grid.xRange[0]) / spec.binsX;
  const yw = (grid.yRange[1] - grid.yRange[0]) / spec.binsY;

  if (grid.cells.length === 0) {
    return (
      <div className="chart-empty" data-testid="mapelites-empty">
        The grid lights up as machines land in new behaviour cells — the first
        generation usually fills the grounded column.
      </div>
    );
  }

  return (
    <div className="chart-wrap me-wrap">
      <svg
        className="chart-svg me-grid"
        viewBox={`0 0 ${W} ${H}`}
        role="img"
        aria-label={`MAP-Elites archive: ${xDef.label} by ${yDef.label}, ${grid.filled} of ${grid.total} cells illuminated, coloured by ${qDef.label}.`}
        onPointerLeave={() => setHover(null)}
        data-testid="mapelites-grid"
      >
        {Array.from({ length: spec.binsY }, (_, iy) =>
          Array.from({ length: spec.binsX }, (_, ix) => {
            const cell = byKey.get(iy * spec.binsX + ix);
            const x = xAt(ix);
            const y = yAt(iy);
            if (!cell)
              return (
                <rect
                  key={`e${ix}-${iy}`}
                  x={x}
                  y={y}
                  width={CELL}
                  height={CELL}
                  rx={2}
                  fill="var(--me-empty)"
                  stroke="var(--hairline)"
                  strokeWidth={1}
                  pointerEvents="none"
                  data-testid="me-cell-empty"
                />
              );
            const active = cell.id === selectedId;
            return (
              <g key={`f${ix}-${iy}`}>
                <rect
                  x={x}
                  y={y}
                  width={CELL}
                  height={CELL}
                  rx={2}
                  fill={`var(--me-${stepOf(cell.q) + 1})`}
                  stroke="var(--surface)"
                  strokeWidth={1}
                  style={{ cursor: "pointer" }}
                  role="button"
                  tabIndex={0}
                  data-testid="me-cell"
                  data-cell={`${ix},${iy}`}
                  data-mid={cell.id}
                  aria-label={`${xDef.label} ${binSpanLabel(spec.x, grid.xRange, spec.binsX, ix)}, ${yDef.label} ${binSpanLabel(spec.y, grid.yRange, spec.binsY, iy)}: ${qDef.fmt(cell.q)} ${qDef.unit}`}
                  onPointerEnter={() => setHover(cell)}
                  onClick={() => onPick(cell)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") onPick(cell);
                  }}
                />
                {active && (
                  <rect
                    x={x - 2}
                    y={y - 2}
                    width={CELL + 4}
                    height={CELL + 4}
                    rx={3}
                    fill="none"
                    stroke="var(--accent)"
                    strokeWidth={2}
                    pointerEvents="none"
                  />
                )}
              </g>
            );
          }),
        )}

        {/* Axes: sparse edge labels, recessive. */}
        {Array.from({ length: spec.binsX + 1 }, (_, i) => i)
          .filter((i) => i % xTickEvery === 0 || i === spec.binsX)
          .map((i) => (
            <text
              key={`xt${i}`}
              x={xAt(i) - GAP / 2}
              y={H - PAD.bottom + 16}
              textAnchor="middle"
              fontSize={10}
              fontFamily="var(--font-mono)"
              fill="var(--muted)"
            >
              {xDef.fmt(grid.xRange[0] + xw * i)}
            </text>
          ))}
        {Array.from({ length: spec.binsY + 1 }, (_, i) => i)
          .filter((i) => i % yTickEvery === 0 || i === spec.binsY)
          .map((i) => (
            <text
              key={`yt${i}`}
              x={PAD.left - 8}
              y={yAt(i - 1) - GAP / 2 + 3.5}
              textAnchor="end"
              fontSize={10}
              fontFamily="var(--font-mono)"
              fill="var(--muted)"
            >
              {yDef.fmt(grid.yRange[0] + yw * i)}
            </text>
          ))}
        <text
          x={W - PAD.right}
          y={H - 6}
          textAnchor="end"
          fontSize={10}
          fontFamily="var(--font-mono)"
          fill="var(--muted)"
          letterSpacing="0.08em"
        >
          {xDef.label.toUpperCase()} ({xDef.unit.toUpperCase()})
        </text>
        <text
          x={PAD.left}
          y={PAD.top - 3}
          fontSize={10}
          fontFamily="var(--font-mono)"
          fill="var(--muted)"
          letterSpacing="0.08em"
        >
          {yDef.label.toUpperCase()} ({yDef.unit.toUpperCase()})
        </text>
      </svg>

      {hover && (
        <div
          className="chart-tip me-tip"
          style={{
            left: `${((xAt(hover.ix) + CELL) / W) * 100}%`,
            top: `${(yAt(hover.iy) / H) * 100}%`,
            transform:
              xAt(hover.ix) > W * 0.6
                ? "translate(calc(-100% - 34px), -8px)"
                : "translate(10px, -8px)",
          }}
        >
          <div className="t">
            cell {hover.ix},{hover.iy} · gen {hover.gen}
          </div>
          <div className="r">
            <span className="lbl">{xDef.label}</span>
            <b>{binSpanLabel(spec.x, grid.xRange, spec.binsX, hover.ix)}</b>
          </div>
          <div className="r">
            <span className="lbl">{yDef.label}</span>
            <b>{binSpanLabel(spec.y, grid.yRange, spec.binsY, hover.iy)}</b>
          </div>
          <div className="r">
            <span className="lbl">{qDef.label}</span>
            <b>
              {qDef.fmt(hover.q)} {qDef.unit}
            </b>
          </div>
          <div className="r">
            <span className="lbl">measured</span>
            <b>
              {hover.speed.toFixed(2)} blk/s · {hover.blocks} blocks
            </b>
          </div>
        </div>
      )}

      <div className="chart-legend me-legend" role="list" aria-label="Legend">
        <span className="item ramp-item" role="listitem">
          <span className="ramp" aria-hidden="true">
            {Array.from({ length: RAMP_STEPS }, (_, i) => (
              <i key={i} style={{ background: `var(--me-${i + 1})` }} />
            ))}
          </span>
          {qDef.label} 0 → {qDef.fmt(grid.bestQuality)} {qDef.unit}
        </span>
        <span className="item" role="listitem">
          <span className="swatch empty" aria-hidden="true" />
          not discovered
        </span>
        <span className="item note-item">click a cell to fly its machine</span>
      </div>
    </div>
  );
}
