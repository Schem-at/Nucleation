import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import { loadRecord, loadXray } from "../lib/store";
import { Seal } from "../components/Seal";
import { StatTile } from "../components/StatTile";
import { ActivityChart } from "../components/ActivityChart";
import { ClassificationBlock } from "../components/ClassificationBlock";
import { Heatmap } from "../components/Heatmap";
import { MaterialsGrid, stackMath } from "../components/MaterialsGrid";
import { MeshReplay } from "../components/MeshReplay";
import type { Census, ResetTime } from "../lib/types";

/** Minecraft runs at 20 ticks a second, so ticks convert to in-game seconds.
 *  The simulation itself finishes in a fraction of that. */
const secs = (ticks: number) => (ticks / 20).toFixed(2);
const pct = (v: number) => (v < 1 ? v.toFixed(1) : String(Math.round(v)));
const tk = (n: number) => `${n} tick${n === 1 ? "" : "s"}`;

/** Redstone can move on the very tick the lever is thrown, and that reads
 *  better as a word than as a zero. */
const responds = (ticks: number | null) =>
  ticks === null
    ? "nothing moved"
    : ticks === 0
      ? "responds instantly"
      : `responds in ${tk(ticks)}`;

/** The doorway time next to the settle time it sits inside. The gap is the
 *  interesting part: a door can be walkable long before its tape stops. */
function strokeSub(doorway: number | null, settle: number | null, latency: number | null) {
  if (doorway === null)
    return settle === null ? "not measured" : `never cleared · settles at ${tk(settle)}`;
  const tail =
    settle === null
      ? ""
      : settle > doorway
        ? ` · settles at ${settle}`
        : " · quiet the same tick";
  return `${secs(doorway)} s · ${responds(latency)}${tail}`;
}

function resetSub(r: ResetTime | null, what: string) {
  if (!r) return "not measured";
  if (r.ticks === null) return r.note ?? `none found within ${r.searched} ticks`;
  if (r.stroke_ticks === null) return `before the lever may ${what} again`;
  return r.negative
    ? `${r.ticks} < ${r.stroke_ticks}-tick stroke — re-triggerable mid-stroke`
    : `${r.ticks} vs the ${r.stroke_ticks}-tick stroke`;
}

/** The parts that make it move, most structural first. Zero counts are left
 *  out — a door without observers should not have to say so. */
function censusRows(c: Census): { term: string; detail: string }[] {
  const rows: { term: string; detail: string }[] = [];
  const add = (n: number, one: string, many: string, detail = "") => {
    if (n > 0) rows.push({ term: `${n} ${n === 1 ? one : many}`, detail });
  };
  add(c.sticky_piston, "sticky piston", "sticky pistons");
  add(c.piston, "piston", "pistons");
  add(c.observer, "observer", "observers");
  add(
    c.repeater,
    "repeater",
    "repeaters",
    c.repeater_delays.length
      ? `delay ${c.repeater_delays.join(", ")}`
      : "",
  );
  add(c.comparator, "comparator", "comparators");
  add(c.redstone_block, "redstone block", "redstone blocks");
  add(c.redstone_torch, "redstone torch", "redstone torches");
  add(c.redstone_wire, "dust", "dust");
  add(c.slime_block, "slime block", "slime blocks");
  add(c.honey_block, "honey block", "honey blocks");
  return rows;
}

export function CertificatePage() {
  const { id = "" } = useParams();
  const rec = useMemo(() => loadRecord(id), [id]);
  const xray = useMemo(() => loadXray(id), [id]);

  const certUrl = useMemo(() => {
    if (!rec) return "";
    const blob = new Blob([JSON.stringify(rec.certificate, null, 1)], {
      type: "application/json",
    });
    return URL.createObjectURL(blob);
  }, [rec]);

  if (!rec)
    return (
      <div className="error-page">
        <p>Certificate {id} is not in this browser's records.</p>
        <p>
          <Link to="/">Submit a door</Link>
        </p>
      </div>
    );

  const cert = rec.certificate;
  const [dx, dy, dz] = cert.dims;
  const movingBlocks = cert.heatmap.values.flat().filter((v) => v > 0).length;
  const totalBlocks = cert.materials.reduce((a, m) => a + m.count, 0);
  const stacks = Math.ceil(totalBlocks / 64);
  const ap = cert.aperture;
  const rows = censusRows(cert.census);
  const ro = cert.reset_open;
  const rc = cert.reset_close;
  // The doorway cycle: click to walkable, click to shut again. Null the
  // moment either half is unmeasured — half a cycle is not a cycle.
  const cycleTicks =
    cert.open_ticks !== null && cert.close_ticks !== null
      ? cert.open_ticks + cert.close_ticks
      : null;
  const settleCycle =
    cert.open_settle_ticks !== null && cert.close_settle_ticks !== null
      ? cert.open_settle_ticks + cert.close_settle_ticks
      : null;
  const resetCycle =
    ro?.ticks != null && rc?.ticks != null ? ro.ticks + rc.ticks : null;
  const negatives = [
    ro?.negative ? { r: ro, verb: "opening" } : null,
    rc?.negative ? { r: rc, verb: "closing" } : null,
  ].filter(Boolean) as { r: ResetTime; verb: string }[];

  return (
    <div className="sheet">
      <div className="sheet-band">
        <span>
          report <b>{id}</b>
        </span>
        <span>
          seed <b>{cert.seed}</b>
        </span>
        <span>
          verdict <b>{cert.verdict}</b>
        </span>
        <span>
          run <b>{new Date().toISOString().slice(0, 10)}</b>
        </span>
      </div>

      <div className="sheet-body">
        <div className="hero">
          <div className="hero-main">
            <p className="eyebrow">Piston door validation report</p>
            <h1>{cert.name}</h1>
            {ap ? (
              <p className="hero-aperture" data-testid="aperture">
                <b>
                  {ap.w} × {ap.h}
                </b>{" "}
                aperture
                <span>
                  {ap.cells} cells clear{ap.depth > 1 ? ` · ${ap.depth} deep` : ""}
                  {ap.note ? ` · ${ap.note}` : ""}
                </span>
              </p>
            ) : (
              <p className="hero-aperture" data-testid="aperture">
                <b>No aperture</b>
                <span>nothing opened when the lever was thrown</span>
              </p>
            )}
            <p className="hero-sub">
              One cycle measured end to end:{" "}
              {cert.rest_is_closed
                ? "lever on, door opens, lever off, door shuts."
                : "lever on, door shuts, lever off, door opens."}{" "}
              Opening and closing are timed at the <b>doorway</b> — the tick the passage
              becomes walkable, and the tick the pattern is solid again — not the tick the
              machine finally goes quiet, which is reported separately as settle.
            </p>
            <div className="hero-dims">
              <span>
                bounds{" "}
                <b>
                  {dx} × {dy} × {dz}
                </b>{" "}
                blocks
              </span>
              <span>
                lever at <b>({cert.lever.join(", ")})</b>
              </span>
              <span>
                {cycleTicks !== null ? (
                  <>
                    doorway cycle <b>{cycleTicks} ticks</b> · {secs(cycleTicks)} s in game
                  </>
                ) : (
                  <>
                    doorway cycle <b>not measured</b>
                  </>
                )}
              </span>
              {settleCycle !== null && (
                <span>
                  settles after <b>{settleCycle} ticks</b>
                </span>
              )}
            </div>
            {cert.timing_note && (
              <p className="hero-note">
                <b>Doorway timing incomplete.</b> {cert.timing_note}. The settle times below
                are still the machine's, but they are not opening times and are not quoted
                as such.
              </p>
            )}
            {negatives.map((n) => (
              <p className="hero-note note-flag" key={n.verb}>
                <span className="badge">negative reset</span> The lever can be thrown again{" "}
                <b>{tk(n.r.ticks!)}</b> after the {n.verb} click, while the{" "}
                {n.r.stroke_ticks}-tick {n.verb} stroke is still running — and the door
                still finishes it and returns to state. Rare.
              </p>
            ))}
            {!cert.rest_is_closed && (
              <p className="hero-note">
                <b>Saved open.</b> The doorway already stands clear in the file, so the
                first lever click closes it and the second opens it. Opening and closing
                below are named for what the door does, not for which click came first.
              </p>
            )}
            {cert.needed_priming && (
              <p className="hero-note">
                <b>Saved mid-cycle.</b> The door did not return to its saved state, so it
                was run to its steady state first; timings are measured from there.
              </p>
            )}
          </div>
          <Seal openTicks={cert.open_ticks} verdict={cert.verdict} />
        </div>
      </div>

      {cert.classification && (
        <div className="sheet-section">
          <ClassificationBlock cls={cert.classification} />
        </div>
      )}

      <div className="sheet-section">
        <p className="eyebrow">Exhibit A — the measured cycle</p>
        <MeshReplay replay={rec.replay} lever={cert.lever} xray={xray} />
        <p className="exhibit-caption">
          Drawn from the recorded block changes. The two lime marks on the scrubber are the
          lever clicks; it starts on the one that{" "}
          {cert.rest_is_closed ? "opens" : "closes"} the door.
        </p>
      </div>

      <div className="sheet-section">
        <p className="eyebrow">Measurements</p>
        <div className="tiles">
          <StatTile
            label="Opens in"
            value={cert.open_ticks ?? "—"}
            unit={cert.open_ticks !== null ? " ticks" : undefined}
            sub={strokeSub(cert.open_ticks, cert.open_settle_ticks, cert.open_latency)}
          />
          <StatTile
            label="Closes in"
            value={cert.close_ticks ?? "—"}
            unit={cert.close_ticks !== null ? " ticks" : undefined}
            sub={strokeSub(cert.close_ticks, cert.close_settle_ticks, cert.close_latency)}
          />
          <StatTile
            label="Settles at"
            value={settleCycle ?? "—"}
            unit={settleCycle !== null ? " ticks" : undefined}
            sub={
              settleCycle === null
                ? "the machine never went quiet"
                : `${cert.open_settle_ticks ?? "—"} opening + ${
                    cert.close_settle_ticks ?? "—"
                  } closing · when the last block stops`
            }
          />
          <StatTile
            label="Reset after opening"
            value={ro?.ticks ?? "—"}
            unit={ro?.ticks != null ? " ticks" : undefined}
            sub={resetSub(ro, "open")}
          />
          <StatTile
            label="Reset after closing"
            value={rc?.ticks ?? "—"}
            unit={rc?.ticks != null ? " ticks" : undefined}
            sub={resetSub(rc, "close")}
          />
          <StatTile
            label="Cycle rate"
            value={Math.round(cert.cycles_per_minute)}
            unit=" /min"
            sub={
              resetCycle !== null
                ? `rate-limited by reset: ${resetCycle} ticks between opens`
                : cycleTicks !== null
                  ? `doorway cycle of ${cycleTicks} ticks — reset not measured`
                  : `from settle: ${settleCycle ?? "—"} ticks`
            }
          />
          <StatTile
            label="Stroke mass"
            value={cert.moved_cells}
            unit=" cells"
            sub={`travel per stroke · ${movingBlocks} columns active`}
          />
          <StatTile
            label="Peak in flight"
            value={cert.peak_changes}
            unit=" cells"
            sub={`moving at tick ${cert.peak_tick} · ${cert.peak_signal} wire updates fire on tick ${cert.peak_signal_tick}`}
          />
          <StatTile
            label="Aperture cost"
            value={ap ? `${pct((ap.cells / cert.volume) * 100)}%` : "—"}
            sub={
              ap
                ? `${ap.cells}-cell doorway in a ${cert.volume}-cell build`
                : "no doorway measured"
            }
          />
        </div>
      </div>

      <div className="sheet-section chart-block">
        <p className="eyebrow">Activity trace</p>
        <h3>Events per tick of the cycle</h3>
        <p className="chart-note">
          Stacked, so the height is everything happening on that tick. One burst per lever
          click; the marker under the axis is where the stroke actually finishes, and shaded
          columns are ticks with nothing at all.
        </p>
        <ActivityChart events={cert.events_per_tick} flips={rec.replay.flips} />
      </div>

      <div className="sheet-section chart-block">
        <p className="eyebrow">Bill of materials</p>
        <h3>
          {totalBlocks} blocks · {stacks} {stacks === 1 ? "stack" : "stacks"}
        </h3>
        <p className="chart-note">
          What to bring, counted at rest and most-used first.{" "}
          {stackMath(totalBlocks) ? `Gather ${stackMath(totalBlocks)}.` : ""}
        </p>
        <MaterialsGrid materials={cert.materials} />
      </div>

      <div className="sheet-section">
        <div className="split">
          <div className="chart-block">
            <p className="eyebrow">Change footprint</p>
            <h3>Block changes per column</h3>
            <p className="chart-note">
              The door's XY silhouette; the stronger the blue, the more a column churned.
            </p>
            <Heatmap heatmap={cert.heatmap} lever={cert.lever} />
          </div>
          {rows.length > 0 && (
            <div className="chart-block">
              <p className="eyebrow">Mechanism</p>
              <h3>What moves it</h3>
              <p className="chart-note">
                The working parts, counted from the door at rest.
              </p>
              <dl className="census" data-testid="census">
                {rows.map((r) => (
                  <div key={r.term}>
                    <dt>{r.term}</dt>
                    {r.detail ? <dd>{r.detail}</dd> : <dd aria-hidden />}
                  </div>
                ))}
              </dl>
            </div>
          )}
        </div>
      </div>

      <div className="sheet-foot">
        <span>schemat.io · door validator — simulated in this browser</span>
        <span>
          seed {cert.seed} ·{" "}
          <a href={certUrl} download={`${cert.name}.certificate.json`}>
            download certificate JSON
          </a>
        </span>
      </div>
    </div>
  );
}
