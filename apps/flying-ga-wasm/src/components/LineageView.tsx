/** Lineage view: a Muller-style stacked-area chart of species share per
 * generation (top species get fixed categorical colors, the rest fold into
 * a gray "other"), annotated with species births (▲) and extinctions (✕).
 * Clicking a band opens the species' dossier: representative machines
 * across its lifetime, its parent species, and its vitals. The registry
 * table below doubles as the accessible data view. */

import { useMemo, useState } from "react";
import type { SpeciesInfo, SpeciesPoint } from "../types";
import IsoThumb from "./IsoThumb";

const W = 720;
const H = 230;
const PAD = { l: 40, r: 12, t: 18, b: 24 };
/** Fixed categorical assignment order — species keep their color for life. */
const MAX_COLORED = 7;

interface Props {
  info: SpeciesInfo[];
  points: SpeciesPoint[];
  currentGen: number;
}

interface Band {
  id: number;
  label: string;
  colorVar: string;
  /** Upper/lower cumulative share per sampled point. */
  lower: number[];
  upper: number[];
}

export default function LineageView({ info, points, currentGen }: Props) {
  const [picked, setPicked] = useState<number | null>(null);
  const [hover, setHover] = useState<{ id: number; x: number; gen: number; share: number } | null>(null);

  const model = useMemo(() => {
    if (points.length < 2 || info.length === 0) return null;
    // Sample down long runs so the path stays light.
    const stride = Math.max(1, Math.ceil(points.length / 360));
    const pts = points.filter((_, i) => i % stride === 0 || i === points.length - 1);
    const gens = pts.map((p) => p.gen);
    const totals = pts.map((p) => p.counts.reduce((a, [, n]) => a + n, 0) || 1);

    // Top species by peak share get their own band, in FIRST-SEEN order so
    // younger species stack on top (Muller reading); the rest fold into
    // "other". Colors are assigned by that fixed order, never re-cycled.
    const top = [...info]
      .sort((a, b) => b.peakShare - a.peakShare)
      .slice(0, MAX_COLORED)
      .sort((a, b) => a.firstGen - b.firstGen || a.id - b.id);
    const topIds = new Set(top.map((s) => s.id));

    const shareOf = (pi: number, sid: number): number => {
      const hit = pts[pi].counts.find(([id]) => id === sid);
      return hit ? hit[1] / totals[pi] : 0;
    };

    const bands: Band[] = [];
    const running = new Array(pts.length).fill(0);
    const pushBand = (id: number, label: string, colorVar: string, shares: number[]) => {
      const lower = running.slice();
      const upper = lower.map((v, i) => v + shares[i]);
      for (let i = 0; i < running.length; i++) running[i] = upper[i];
      bands.push({ id, label, colorVar, lower, upper });
    };
    top.forEach((s, i) =>
      pushBand(
        s.id,
        s.label,
        `var(--cat-${i + 1})`,
        pts.map((_, pi) => shareOf(pi, s.id)),
      ),
    );
    const otherShares = pts.map((p, pi) => {
      let n = 0;
      for (const [id, c] of p.counts) if (!topIds.has(id)) n += c;
      return n / totals[pi];
    });
    if (otherShares.some((v) => v > 0))
      pushBand(-1, "other", "var(--cat-other)", otherShares);

    const g0 = gens[0];
    const g1 = gens[gens.length - 1];
    const xOf = (g: number) =>
      PAD.l + ((g - g0) / Math.max(1, g1 - g0)) * (W - PAD.l - PAD.r);
    const yOf = (v: number) => PAD.t + (1 - v) * (H - PAD.t - PAD.b);

    const areaPath = (b: Band) => {
      const upPts = gens.map((g, i) => `${xOf(g).toFixed(1)},${yOf(b.upper[i]).toFixed(1)}`);
      const loPts = gens
        .map((g, i) => `${xOf(g).toFixed(1)},${yOf(b.lower[i]).toFixed(1)}`)
        .reverse();
      return `M${upPts.join("L")}L${loPts.join("L")}Z`;
    };

    // Birth / extinction annotations for the colored species.
    const marks = top.flatMap((s) => {
      const out: Array<{ kind: "birth" | "extinct"; gen: number; label: string }> = [];
      if (s.firstGen > g0)
        out.push({ kind: "birth", gen: s.firstGen, label: `${s.label} born @ gen ${s.firstGen}` });
      if (s.lastGen < currentGen)
        out.push({ kind: "extinct", gen: s.lastGen, label: `${s.label} extinct after gen ${s.lastGen}` });
      return out;
    });

    return { pts, gens, bands, top, xOf, yOf, areaPath, marks, g0, g1 };
  }, [info, points, currentGen]);

  if (!model) {
    return (
      <div className="chart-empty">
        Species appear here as a run evolves — every structural family gets a
        band, born and dying in real time.
      </div>
    );
  }

  const pickedInfo = picked !== null ? info.find((s) => s.id === picked) ?? null : null;
  const parentInfo =
    pickedInfo && pickedInfo.parent !== null
      ? info.find((s) => s.id === pickedInfo.parent) ?? null
      : null;

  const onMove = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const fx = ((e.clientX - rect.left) / rect.width) * W;
    const fy = ((e.clientY - rect.top) / rect.height) * H;
    const g = Math.round(
      model.g0 + ((fx - PAD.l) / Math.max(1, W - PAD.l - PAD.r)) * (model.g1 - model.g0),
    );
    let idx = 0;
    let bestD = Infinity;
    model.gens.forEach((gg, i) => {
      const d = Math.abs(gg - g);
      if (d < bestD) {
        bestD = d;
        idx = i;
      }
    });
    const v = 1 - (fy - PAD.t) / (H - PAD.t - PAD.b);
    const band = model.bands.find((b) => v >= b.lower[idx] && v <= b.upper[idx]);
    if (band)
      setHover({
        id: band.id,
        x: fx,
        gen: model.gens[idx],
        share: band.upper[idx] - band.lower[idx],
      });
    else setHover(null);
  };

  const hoverBand = hover ? model.bands.find((b) => b.id === hover.id) : null;

  return (
    <div className="lineage">
      <div className="chart-legend" role="list" aria-label="Species">
        {model.bands.map((b) => (
          <span className="item" role="listitem" key={b.id}>
            <span className="swatch" style={{ background: b.colorVar }} />
            {b.label}
          </span>
        ))}
      </div>
      <div className="chart-wrap">
        <svg
          className="chart-svg"
          viewBox={`0 0 ${W} ${H}`}
          role="img"
          aria-label={`Species share per generation, ${model.bands.length} bands over generations ${model.g0} to ${model.g1}`}
          data-testid="muller-chart"
          onMouseMove={onMove}
          onMouseLeave={() => setHover(null)}
        >
          {[0, 0.5, 1].map((v) => (
            <g key={v}>
              <line
                x1={PAD.l}
                x2={W - PAD.r}
                y1={model.yOf(v)}
                y2={model.yOf(v)}
                className="grid-line"
              />
              <text x={PAD.l - 6} y={model.yOf(v) + 3} className="axis-label" textAnchor="end">
                {Math.round(v * 100)}%
              </text>
            </g>
          ))}
          {model.bands.map((b) => (
            <path
              key={b.id}
              d={model.areaPath(b)}
              fill={b.colorVar}
              opacity={picked !== null && picked !== b.id ? 0.35 : 0.92}
              stroke="var(--page)"
              strokeWidth={1}
              style={{ cursor: b.id >= 0 ? "pointer" : "default" }}
              data-testid={`species-band-${b.id}`}
              onClick={() => b.id >= 0 && setPicked(picked === b.id ? null : b.id)}
            >
              <title>{b.label}</title>
            </path>
          ))}
          {model.marks.map((m, i) => (
            <g key={i} data-testid={`mark-${m.kind}`}>
              {m.kind === "birth" ? (
                <path
                  d={`M${model.xOf(m.gen)},${PAD.t - 10}l4,7h-8z`}
                  className="mark-birth"
                >
                  <title>{m.label}</title>
                </path>
              ) : (
                <text
                  x={model.xOf(m.gen)}
                  y={PAD.t - 3}
                  textAnchor="middle"
                  className="mark-extinct"
                >
                  ✕<title>{m.label}</title>
                </text>
              )}
            </g>
          ))}
          <text x={W - PAD.r} y={H - 6} textAnchor="end" className="axis-label">
            generation →
          </text>
        </svg>
        {hover && hoverBand && (
          <div
            className="chart-tip"
            style={{
              left: `${(hover.x / W) * 100}%`,
              transform: hover.x > W * 0.6 ? "translateX(-105%)" : "none",
            }}
          >
            <div className="t">{hoverBand.label}</div>
            <div className="r">
              <span className="lbl">gen</span> {hover.gen}
            </div>
            <div className="r">
              <span className="lbl">share</span> {(hover.share * 100).toFixed(1)}%
            </div>
          </div>
        )}
      </div>

      {pickedInfo && (
        <div className="species-detail" data-testid="species-detail">
          <div className="species-head">
            <b>{pickedInfo.label}</b>
            <span className="note">
              {pickedInfo.dims.join("×")} · {pickedInfo.summary} · born gen{" "}
              {pickedInfo.firstGen} ·{" "}
              {pickedInfo.lastGen < currentGen
                ? `extinct after gen ${pickedInfo.lastGen}`
                : "alive"}
              {parentInfo ? ` · budded from ${parentInfo.label}` : ""}
            </span>
          </div>
          <div className="species-strip">
            {pickedInfo.exemplars.map((ex, i) => (
              <figure key={i} className="species-ex">
                <IsoThumb
                  blocks={ex.blocks}
                  width={120}
                  label={`${pickedInfo.label} representative at generation ${ex.gen}`}
                />
                <figcaption>gen {ex.gen}</figcaption>
              </figure>
            ))}
          </div>
        </div>
      )}

      <table className="lb-table species-table" data-testid="species-table">
        <thead>
          <tr>
            <th>Species</th>
            <th>Build</th>
            <th className="num">Born</th>
            <th className="num">Last seen</th>
            <th className="num">Peak share</th>
            <th>Parent</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {[...info]
            .sort((a, b) => b.peakShare - a.peakShare)
            .slice(0, 20)
            .map((s) => (
              <tr
                key={s.id}
                className={`row${picked === s.id ? " sel" : ""}`}
                onClick={() => setPicked(picked === s.id ? null : s.id)}
              >
                <td className="name">{s.label}</td>
                <td>{s.dims.join("×")} · {s.summary}</td>
                <td className="num">{s.firstGen}</td>
                <td className="num">{s.lastGen}</td>
                <td className="num">{(s.peakShare * 100).toFixed(1)}%</td>
                <td>{s.parent !== null ? `S${s.parent + 1}` : "—"}</td>
                <td>{s.lastGen < currentGen ? "extinct ✕" : "alive"}</td>
              </tr>
            ))}
        </tbody>
      </table>
    </div>
  );
}
