import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import { loadRecord } from "../lib/store";
import { Seal } from "../components/Seal";
import { StatTile } from "../components/StatTile";
import { ActivityChart } from "../components/ActivityChart";
import { ClassificationBlock } from "../components/ClassificationBlock";
import { Heatmap } from "../components/Heatmap";
import { MaterialsGrid, stackMath } from "../components/MaterialsGrid";
import { MeshReplay } from "../components/MeshReplay";
import type { Census } from "../lib/types";

/** Minecraft runs at 20 ticks a second, so ticks convert to in-game seconds.
 *  The simulation itself finishes in a fraction of that. */
const secs = (ticks: number) => (ticks / 20).toFixed(2);
const pct = (v: number) => (v < 1 ? v.toFixed(1) : String(Math.round(v)));

/** Redstone can move on the very tick the lever is thrown, and that reads
 *  better as a word than as a zero. */
const responds = (ticks: number) =>
  ticks === 0 ? "responds instantly" : `responds in ${ticks} tick${ticks === 1 ? "" : "s"}`;

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
  const cycleTicks = cert.open_ticks + cert.close_ticks;
  const rows = censusRows(cert.census);

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
          <div>
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
                </span>
              </p>
            ) : (
              <p className="hero-aperture" data-testid="aperture">
                <b>No aperture</b>
                <span>nothing opened when the lever was thrown</span>
              </p>
            )}
            <p className="hero-sub">
              One cycle measured end to end: lever on, door opens, lever off, door shuts.
              Every timing below is read off that cycle.
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
                full cycle <b>{cycleTicks} ticks</b> · {secs(cycleTicks)} s in game
              </span>
            </div>
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
        <MeshReplay replay={rec.replay} lever={cert.lever} />
        <p className="exhibit-caption">
          Drawn from the recorded block changes. The two lime marks on the scrubber are the
          lever clicks: it starts on the one that opens the door.
        </p>
      </div>

      <div className="sheet-section">
        <p className="eyebrow">Measurements</p>
        <div className="tiles">
          <StatTile
            label="Opens in"
            value={cert.open_ticks}
            unit=" ticks"
            sub={`${responds(cert.open_latency)} · ${secs(cert.open_ticks)} s in game`}
          />
          <StatTile
            label="Closes in"
            value={cert.close_ticks}
            unit=" ticks"
            sub={`${responds(cert.close_latency)} · ${secs(cert.close_ticks)} s in game`}
          />
          <StatTile
            label="Cycle rate"
            value={Math.round(cert.cycles_per_minute)}
            unit=" /min"
            sub={`safe to re-trigger every ${cycleTicks} ticks`}
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
