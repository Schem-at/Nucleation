/** The activity strip: where the recorded run was busy.
 *
 * Reads the same JSON `world.sim.timelineActivityJson()` produces — see
 * `TickSimulation::timeline_activity_json` in `src/bridge/mc_tick.rs` — and
 * nothing else. No engine reference, no polling of its own: `App.tsx` owns
 * the timer and hands this component the decoded JSON.
 */

import { useEffect, useMemo, useRef, useState } from "react";

/** A tick range. Reused by the export-range selection this strip will grow
 * in a later task; read-only here (see Step 1 of the task brief). */
export type Span = { start: number; end: number };

/** `TickSimulation::timeline_activity_json`'s exact shape. Ticks are only
 * present when something happened on them — an idle tick is absent, not
 * present with zeroes. */
export type Activity = {
  start: number;
  end: number;
  ticks: { tick: number; changes: number; inputs: number; pistons: number }[];
};

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

/** Log-scaled column height, 0..1. Linear would flatten every tick beside a
 * 500-change one to invisible — a piston tick (single-digit changes) and a
 * mass-update tick differ by orders of magnitude, and log scaling is what
 * keeps both readable in the same strip. */
function heightFrac(changes: number, maxChanges: number): number {
  if (changes <= 0 || maxChanges <= 0) return 0;
  const v = Math.log1p(changes) / Math.log1p(maxChanges);
  return Math.max(0.06, Math.min(1, v));
}

/** Used for the very first render, before the strip's own width has been
 * measured — see the `ResizeObserver` below. Bucketing may look one frame
 * coarser than necessary until then; it is never wrong, only briefly
 * conservative. */
const FALLBACK_WIDTH_PX = 800;

export function TimelineStrip({ activity }: { activity: Activity | null }): JSX.Element {
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

  return (
    <div className="timeline" aria-label="run timeline activity">
      <div className="timeline-columns" ref={trackRef}>
        {columns.length === 0 ? (
          <span className="timeline-empty">
            {activity ? "no activity recorded yet" : "not recording"}
          </span>
        ) : (
          columns.map((c, i) => (
            <span
              key={i}
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
          ))
        )}
      </div>
    </div>
  );
}
