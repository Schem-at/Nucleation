/** Current-generation population inspector: the ENTIRE population as a
 * virtualized grid of iso thumbnails with fitness, species color chip and
 * a validity badge naming the constraint that culled each failed genome.
 * Updates once per generation; a paused run is a stable, browsable
 * snapshot. Thumbnails are cached by genome fingerprint (memoized cell —
 * identical genomes across generations never re-render their SVG). */

import { memo, useEffect, useMemo, useRef, useState } from "react";
import IsoThumb from "./IsoThumb";
import type { Block, PopulationMember, SpeciesInfo } from "../types";

interface Props {
  members: PopulationMember[];
  species: SpeciesInfo[];
  gen: number;
  paused: boolean;
  onPick: (m: PopulationMember) => void;
}

const CELL_W = 118;
const CELL_H = 148;
const GAP = 8;
const VIEW_H = 470;
const OVERSCAN_ROWS = 2;
/** Top species (by peak share) that wear categorical colors — must match
 * LineageView's MAX_COLORED so chips agree with the Muller chart. */
const MAX_COLORED = 7;

const Thumb = memo(
  function Thumb({ blocks }: { blocks: Block[]; fp: string }) {
    return <IsoThumb blocks={blocks} width={86} />;
  },
  (a, b) => a.fp === b.fp,
);

export default function PopulationInspector({
  members,
  species,
  gen,
  paused,
  onPick,
}: Props) {
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [width, setWidth] = useState(900);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setWidth(el.clientWidth));
    ro.observe(el);
    setWidth(el.clientWidth);
    return () => ro.disconnect();
  }, []);

  /** Species color assignment, identical to LineageView: top species by
   * peak share, then first-seen order, get --cat-1..7; the rest fold into
   * --cat-other. Color follows the species for its whole life. */
  const colorOf = useMemo(() => {
    const top = [...species]
      .sort((a, b) => b.peakShare - a.peakShare)
      .slice(0, MAX_COLORED)
      .sort((a, b) => a.firstGen - b.firstGen || a.id - b.id);
    const map = new Map<number, string>(
      top.map((s, i) => [s.id, `var(--cat-${i + 1})`]),
    );
    return (id: number) => map.get(id) ?? "var(--cat-other)";
  }, [species]);
  const labelOf = useMemo(() => {
    const map = new Map(species.map((s) => [s.id, s.label]));
    return (id: number) => map.get(id) ?? "?";
  }, [species]);

  if (members.length === 0) {
    return (
      <div className="hist-empty" data-testid="population-empty">
        The whole current generation appears here once a run is live —
        archived runs keep no population snapshot.
      </div>
    );
  }

  const cols = Math.max(1, Math.floor((width - GAP) / (CELL_W + GAP)));
  const rows = Math.ceil(members.length / cols);
  const rowH = CELL_H + GAP;
  const firstRow = Math.max(0, Math.floor(scrollTop / rowH) - OVERSCAN_ROWS);
  const lastRow = Math.min(
    rows - 1,
    Math.ceil((scrollTop + VIEW_H) / rowH) + OVERSCAN_ROWS,
  );
  const invalid = members.filter((m) => m.violation).length;

  const cells = [];
  for (let r = firstRow; r <= lastRow; r++) {
    for (let c = 0; c < cols; c++) {
      const i = r * cols + c;
      if (i >= members.length) break;
      const m = members[i];
      cells.push(
        <button
          type="button"
          key={m.id}
          className={`pop-cell${m.violation ? " invalid" : ""}`}
          style={{
            left: GAP + c * (CELL_W + GAP),
            top: GAP + r * rowH,
            width: CELL_W,
            height: CELL_H,
          }}
          data-testid="pop-cell"
          onClick={() => onPick(m)}
          title={
            m.violation
              ? `#${i + 1} — invalid: ${m.violation}`
              : `#${i + 1} — ${m.fitness.toFixed(1)} blocks, ${m.speed.toFixed(2)} blk/s — click to inspect`
          }
        >
          <span className="pop-thumb">
            <Thumb blocks={m.blocks} fp={m.key} />
          </span>
          <span className="pop-meta">
            <span
              className="pop-chip"
              style={{ background: colorOf(m.speciesId) }}
              title={`species ${labelOf(m.speciesId)}`}
            />
            <span className="pop-fit">{m.fitness.toFixed(1)}</span>
            <span className="pop-unit">blk</span>
          </span>
          {m.violation ? (
            <span className="pop-badge" data-testid="pop-invalid">
              ✕ {m.violation}
            </span>
          ) : (
            <span className="pop-badge ok">✓ valid</span>
          )}
        </button>,
      );
    }
  }

  return (
    <div className="pop-wrap" data-testid="population-inspector">
      <div className="pop-head">
        <span className="pop-count" data-testid="pop-count">
          gen {gen} · {members.length} machines · {invalid} invalid
        </span>
        {paused && (
          <span className="paused-flag" data-testid="pop-paused-flag">
            paused — stable snapshot
          </span>
        )}
      </div>
      <div
        className="pop-scroll"
        ref={scrollRef}
        style={{ height: VIEW_H }}
        data-total={members.length}
        data-testid="pop-scroll"
        onScroll={(e) => setScrollTop((e.target as HTMLDivElement).scrollTop)}
      >
        <div
          className="pop-inner"
          style={{ height: GAP + rows * rowH, position: "relative" }}
        >
          {cells}
        </div>
      </div>
    </div>
  );
}
