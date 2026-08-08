/** The activity strip: where the recorded run was busy, and — as of Task 4
 * — where a tick range gets selected for export and where a detected cycle
 * gets shown.
 *
 * Reads the same JSON `world.sim.timelineActivityJson()` produces — see
 * `TickSimulation::timeline_activity_json` in `src/bridge/mc_tick.rs` — and
 * nothing else. No engine reference, no polling of its own: `App.tsx` owns
 * the timer and the `timelineCyclesJson()` call, and hands this component
 * decoded JSON plus the current selection.
 */

import { useEffect, useMemo, useRef, useState } from "react";

/** A tick range. Used both for the export-range selection and for a
 * detected cycle's span. */
export type Span = { start: number; end: number };

/** `TickSimulation::timeline_activity_json`'s exact shape. Ticks are only
 * present when something happened on them — an idle tick is absent, not
 * present with zeroes. */
export type Activity = {
  start: number;
  end: number;
  ticks: { tick: number; changes: number; inputs: number; pistons: number }[];
};

/** One recurrence match, exactly as `cycle_json` in `src/bridge/mc_tick.rs`
 * writes it: `{"start":T,"end":T,"period":N,"drift":[x,y,z]}`. `start`/`end`
 * line up with `Span` on purpose — a `CycleMatch` IS a `Span`, plus the
 * period and drift the engine found. */
export type CycleMatch = { start: number; end: number; period: number; drift: [number, number, number] };

/** `TickSimulation::timeline_cycles_json`'s exact shape:
 * `{"exact":CycleMatch|null,"translated":CycleMatch|null}`. Either or both
 * may be `null` — most builds (an adder, a door) never repeat their own
 * state, and that is the ordinary outcome, not a failed search. `null` on
 * both sides, or the whole `Cycles` value being `null` before the button has
 * ever been pressed, must render nothing and raise no error. */
export type Cycles = { exact: CycleMatch | null; translated: CycleMatch | null };

/** One drawn column: either a single active tick or a bucket of adjacent
 * ones (see the bucketing comment on `bucketColumns`). */
type Column = {
  firstTick: number;
  lastTick: number;
  changes: number;
  inputs: number;
  pistons: number;
};

/** Below this many pixels of strip width per column, adjacent entries are
 * merged instead of drawn one-per-pixel — past that point columns are
 * thinner than they can usefully render and the eye cannot resolve them
 * anyway. */
const MIN_COLUMN_PX = 2;

/** Turn `activity.ticks` into columns, bucketing when there would be more
 * active ticks than the strip has pixels for.
 *
 * One column per **active** tick, never per tick in `start..end` — an idle
 * tick has no entry in `activity.ticks` at all, so a still build produces
 * zero columns and the strip stays flat. When the active-tick count exceeds
 * what `widthPx` can draw at `MIN_COLUMN_PX` each, adjacent entries are
 * merged in order (activity is already tick-ordered) into fixed-size
 * buckets, summing their counts and keeping the first and last tick each
 * bucket covers — so a selection built on top of a bucketed column still
 * resolves to the exact ticks inside it, and the summed counts account for
 * every entry (nothing dropped, nothing double-counted).
 */
function bucketColumns(ticks: Activity["ticks"], widthPx: number): Column[] {
  const maxColumns = Math.max(1, Math.floor(widthPx / MIN_COLUMN_PX));
  if (ticks.length <= maxColumns) {
    return ticks.map((t) => ({
      firstTick: t.tick,
      lastTick: t.tick,
      changes: t.changes,
      inputs: t.inputs,
      pistons: t.pistons,
    }));
  }
  const perBucket = Math.ceil(ticks.length / maxColumns);
  const out: Column[] = [];
  for (let i = 0; i < ticks.length; i += perBucket) {
    const slice = ticks.slice(i, i + perBucket);
    out.push({
      firstTick: slice[0].tick,
      lastTick: slice[slice.length - 1].tick,
      changes: slice.reduce((s, t) => s + t.changes, 0),
      inputs: slice.reduce((s, t) => s + t.inputs, 0),
      pistons: slice.reduce((s, t) => s + t.pistons, 0),
    });
  }
  return out;
}

/** How short a column may render and still be visible and selectable. Not
 * just a stylistic minimum: Task 4 builds a click-to-select target on top of
 * these columns, and a `height: 0` column is not clickable. */
const PRESENCE_FLOOR = 0.06;

/** Log-scaled column height, 0..1. Linear would flatten every tick beside a
 * 500-change one to invisible — a piston tick (single-digit changes) and a
 * mass-update tick differ by orders of magnitude, and log scaling is what
 * keeps both readable in the same strip.
 *
 * `changes` may legitimately be zero here: `TickSimulation::timeline_activity_json`
 * puts a tick in `ticks` when it has changes, inputs, *or* pistons — an
 * `InputAction::UseBlock` on a block whose `on_used` writes nothing (a
 * plain right-click that changes no state) is exactly that case, and it is
 * reachable through the app's own click handler, not just a theoretical
 * engine output. Presence in `ticks` means something happened, so every
 * such column renders at or above the floor regardless of `changes` — the
 * floor is about being there at all, not about magnitude. Only the
 * changes-bearing range above it is log-scaled.
 */
function heightFrac(changes: number, maxChanges: number): number {
  if (changes <= 0 || maxChanges <= 0) return PRESENCE_FLOOR;
  const v = Math.log1p(changes) / Math.log1p(maxChanges);
  return Math.max(PRESENCE_FLOOR, Math.min(1, v));
}

/** Used for the very first render, before the strip's own width has been
 * measured — see the `ResizeObserver` below. Bucketing may look one frame
 * coarser than necessary until then; it is never wrong, only briefly
 * conservative. */
const FALLBACK_WIDTH_PX = 800;

/** Map an exact tick span onto the *current* column layout as a left/width
 * fraction of the track (0..1 each), or `null` when the span does not
 * overlap any drawn column (e.g. a stale selection from before the
 * recording was cleared).
 *
 * Columns are laid out as equal-width flex items, so a tick's pixel
 * position depends on which column *index* holds it, not on the tick number
 * directly — this walks the current `columns` array to find that index.
 * Deliberately takes `columns` fresh and does no caching of its own:
 * `activity` re-polls every 250 ms while recording, so bucketing can shift
 * under a live selection between renders, and the only thing that stays
 * valid across that shift is the exact tick range itself (Decision 3, task
 * brief hand-off — this is why `Span` is stored as ticks, never as column
 * indices or pixel offsets).
 */
function ticksToFraction(columns: Column[], span: Span): { left: number; width: number } | null {
  if (columns.length === 0) return null;
  let startIdx = -1;
  let endIdx = -1;
  for (let i = 0; i < columns.length; i++) {
    if (startIdx === -1 && columns[i].lastTick >= span.start) startIdx = i;
    if (columns[i].firstTick <= span.end) endIdx = i;
  }
  if (startIdx === -1 || endIdx === -1 || endIdx < startIdx) return null;
  const n = columns.length;
  return { left: startIdx / n, width: (endIdx - startIdx + 1) / n };
}

const pct = (v: number) => `${(v * 100).toFixed(3)}%`;

export function TimelineStrip({
  activity,
  selection,
  onSelect,
  cycles,
}: {
  activity: Activity | null;
  /** The current export-range selection, as exact ticks — never column
   * indices. `null` means nothing is selected. */
  selection: Span | null;
  /** Fired on drag-release with the dragged span, or with `null` when a
   * plain click (no drag) should clear the selection. Also fired when a
   * cycle overlay is clicked, with that cycle's own span. */
  onSelect: (span: Span | null) => void;
  /** The last `timelineCyclesJson()` result, or `null` before the button has
   * been pressed (or after activity resets). Rendered as shaded overlays;
   * a `null` field within it is a normal "no recurrence found" outcome. */
  cycles: Cycles | null;
}): JSX.Element {
  const trackRef = useRef<HTMLDivElement | null>(null);
  const [width, setWidth] = useState(FALLBACK_WIDTH_PX);

  // The strip's own rendered width decides the pixel budget for bucketing —
  // not a guess and not the engine's. Re-measured on layout changes (a
  // window resize, a panel toggling) so bucketing always matches what is
  // actually on screen.
  useEffect(() => {
    const el = trackRef.current;
    if (!el) return;
    const ro = new ResizeObserver((entries) => {
      const w = entries[0]?.contentRect.width;
      if (w) setWidth(w);
    });
    ro.observe(el);
    setWidth(el.clientWidth || FALLBACK_WIDTH_PX);
    return () => ro.disconnect();
  }, []);

  const columns = useMemo(() => {
    if (!activity || activity.ticks.length === 0) return [];
    return bucketColumns(activity.ticks, width);
  }, [activity, width]);

  const maxChanges = useMemo(
    () => columns.reduce((m, c) => Math.max(m, c.changes), 0),
    [columns],
  );

  // Drag-to-select state. Anchor and current are captured as tick ranges
  // straight off the column the pointer is over at the moment of the event
  // — not as an index into `columns` — so a mid-drag re-render that
  // reshuffles bucketing (a live poll landing during the gesture) cannot
  // desync the drag from what the pointer is actually over.
  const [dragAnchor, setDragAnchor] = useState<{ firstTick: number; lastTick: number } | null>(null);
  const [dragCurrent, setDragCurrent] = useState<{ firstTick: number; lastTick: number } | null>(null);
  const dragging = dragAnchor !== null;

  useEffect(() => {
    if (!dragging) return;
    const commit = () => {
      if (dragAnchor && dragCurrent) {
        const sameColumn =
          dragAnchor.firstTick === dragCurrent.firstTick && dragAnchor.lastTick === dragCurrent.lastTick;
        if (sameColumn) {
          // A click, not a drag: clears the selection.
          onSelect(null);
        } else {
          // Normalise by tick order, not by which end was pressed first —
          // a right-to-left drag must yield the same {start,end} as the
          // equivalent left-to-right one (Decision 2). The outer edges of
          // the dragged span: the earlier column's first tick through the
          // later column's last tick (Decision 5).
          const [earlier, later] =
            dragAnchor.firstTick <= dragCurrent.firstTick ? [dragAnchor, dragCurrent] : [dragCurrent, dragAnchor];
          onSelect({ start: earlier.firstTick, end: later.lastTick });
        }
      }
      setDragAnchor(null);
      setDragCurrent(null);
    };
    window.addEventListener("pointerup", commit);
    return () => window.removeEventListener("pointerup", commit);
  }, [dragging, dragAnchor, dragCurrent, onSelect]);

  // Selection and cycle overlays are recomputed from the current `columns`
  // on every render — never cached, never keyed off a stored index. See
  // `ticksToFraction`'s doc comment for why (Decision 3).
  const selectionFrac = useMemo(
    () => (selection ? ticksToFraction(columns, selection) : null),
    [columns, selection],
  );
  const exactFrac = useMemo(
    () => (cycles?.exact ? ticksToFraction(columns, cycles.exact) : null),
    [columns, cycles],
  );
  const translatedFrac = useMemo(
    () => (cycles?.translated ? ticksToFraction(columns, cycles.translated) : null),
    [columns, cycles],
  );

  return (
    <div className="timeline" aria-label="run timeline activity">
      <div className="timeline-columns" ref={trackRef}>
        {columns.length === 0 ? (
          <span className="timeline-empty">
            {activity ? "no activity recorded yet" : "not recording"}
          </span>
        ) : (
          columns.map((c, i) => (
            // The full-height slot is the hit target, not the painted bar
            // inside it: a `{changes:0}` column renders at only the 6%
            // presence floor, and a pointer target limited to that sliver
            // would make those same ticks ungrabbable — the identical
            // defect Task 3 fixed one layer up for visibility (Decision 1).
            <div
              key={i}
              className="timeline-col-hit"
              data-first-tick={c.firstTick}
              data-last-tick={c.lastTick}
              onPointerDown={(e) => {
                e.preventDefault();
                const col = { firstTick: c.firstTick, lastTick: c.lastTick };
                setDragAnchor(col);
                setDragCurrent(col);
              }}
              onPointerEnter={() => {
                if (dragging) setDragCurrent({ firstTick: c.firstTick, lastTick: c.lastTick });
              }}
            >
              <span
                className={`timeline-col${c.inputs > 0 ? " has-input" : ""}${
                  c.pistons > 0 ? " has-piston" : ""
                }`}
                style={{ height: `${Math.round(heightFrac(c.changes, maxChanges) * 100)}%` }}
                // Plain data, not just a tooltip: a bucketed column's exact
                // tick range and conserved counts are asserted on directly by
                // `stripprobe.mjs`, which has no other way to read what a
                // column represents without reimplementing the bucketing.
                data-first-tick={c.firstTick}
                data-last-tick={c.lastTick}
                data-changes={c.changes}
                data-inputs={c.inputs}
                data-pistons={c.pistons}
                title={
                  (c.firstTick === c.lastTick
                    ? `tick ${c.firstTick}`
                    : `ticks ${c.firstTick}–${c.lastTick}`) +
                  ` · ${c.changes} change${c.changes === 1 ? "" : "s"}` +
                  (c.inputs ? ` · ${c.inputs} input${c.inputs === 1 ? "" : "s"}` : "") +
                  (c.pistons ? ` · ${c.pistons} piston${c.pistons === 1 ? "" : "s"}` : "")
                }
              />
            </div>
          ))
        )}
        {selectionFrac && (
          <span
            className="timeline-selection"
            data-testid="timeline-selection"
            style={{ left: pct(selectionFrac.left), width: pct(selectionFrac.width) }}
          />
        )}
        {exactFrac && cycles?.exact && (
          <span
            className="timeline-cycle"
            data-testid="timeline-cycle-exact"
            style={{ left: pct(exactFrac.left), width: pct(exactFrac.width) }}
            title={`exact cycle: ticks ${cycles.exact.start}–${cycles.exact.end} · period ${cycles.exact.period}`}
            onClick={() => onSelect({ start: cycles.exact!.start, end: cycles.exact!.end })}
          />
        )}
        {translatedFrac && cycles?.translated && (
          <span
            className="timeline-cycle"
            data-testid="timeline-cycle-translated"
            style={{ left: pct(translatedFrac.left), width: pct(translatedFrac.width) }}
            title={
              `translated cycle: ticks ${cycles.translated.start}–${cycles.translated.end} · ` +
              `period ${cycles.translated.period} · drift ${cycles.translated.drift.join(",")}`
            }
            onClick={() => onSelect({ start: cycles.translated!.start, end: cycles.translated!.end })}
          />
        )}
      </div>
      <span className="timeline-readout">
        {selection
          ? `sel ${selection.start}–${selection.end} (${selection.end - selection.start + 1} ticks)`
          : "no selection"}
      </span>
    </div>
  );
}
