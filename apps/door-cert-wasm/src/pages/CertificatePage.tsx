import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import { loadRecord } from "../lib/store";
import { Seal } from "../components/Seal";
import { StatTile } from "../components/StatTile";
import { ActivityChart } from "../components/ActivityChart";
import { Heatmap } from "../components/Heatmap";
import { MaterialsTable } from "../components/MaterialsTable";
import { MeshReplay } from "../components/MeshReplay";

const secs = (ticks: number) => (ticks / 20).toFixed(2);

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
  const peak = Math.max(...cert.events_per_tick.map((e) => e.piston + e.redstone));
  const totalBlocks = cert.materials.reduce((a, m) => a + m.count, 0);

  return (
    <div className="sheet">
      <div className="sheet-band">
        <span>Certificate no. {id}</span>
        <span>Seed {cert.seed}</span>
        <span>Verdict {cert.verdict}</span>
        <span>Issued {new Date().toISOString().slice(0, 10)}</span>
      </div>

      <div className="sheet-body">
        <div className="hero">
          <div>
            <p className="eyebrow">Piston door performance certificate</p>
            <h1>{cert.name}</h1>
            <p className="hero-sub">
              Simulated for {cert.sim_ticks} ticks at 20 tps, entirely in your browser. One full
              cycle observed: lever on, door open, lever off, door shut. All timings measured,
              not estimated.
            </p>
            <div className="hero-dims">
              <span>
                bounds <b>{dx} × {dy} × {dz}</b> blocks
              </span>
              <span>
                lever at <b>({cert.lever.join(", ")})</b>
              </span>
              <span>
                closes in <b>{cert.close_ticks} ticks</b> · {secs(cert.close_ticks)} s
              </span>
            </div>
            {!cert.paste_safe && (
              <p className="hero-caveat">
                <b>Needs priming after pasting.</b> Measured as built. Placed by a
                paste, this door's stroke drops from {cert.moved_cells} to{" "}
                {cert.paste_moved_cells} cells — vanilla re-derives redstone state on
                placement and its memory cell comes up unlatched. Cycle the lever once
                after placing it.
              </p>
            )}
          </div>
          <Seal openTicks={cert.open_ticks} verdict={cert.verdict} />
        </div>
      </div>

      <div className="sheet-section">
        <p className="eyebrow">Exhibit A — cycle replay</p>
        <MeshReplay replay={rec.replay} lever={cert.lever} />
        <p className="exhibit-caption">
          Conditioning cycle, then the measured open and close — replayed from the recorded
          block changes, drawn live in this tab. Red marks on the scrubber are the measured
          lever flips.
        </p>
      </div>

      <div className="sheet-section">
        <p className="eyebrow">Measurements</p>
        <div className="tiles">
          <StatTile label="Open time" value={cert.open_ticks} unit=" ticks" sub={`${secs(cert.open_ticks)} s at 20 tps`} />
          <StatTile label="Close time" value={cert.close_ticks} unit=" ticks" sub={`${secs(cert.close_ticks)} s at 20 tps`} />
          <StatTile
            label="Stroke mass"
            value={cert.moved_cells}
            sub={`cells travel per stroke · ${movingBlocks} columns active`}
          />
          <StatTile label="Sim length" value={cert.sim_ticks} unit=" ticks" sub={`peak ${peak} events in one tick`} />
        </div>
      </div>

      <div className="sheet-section chart-block">
        <p className="eyebrow">Activity trace</p>
        <h3>Events per simulation tick</h3>
        <p className="chart-note">
          One burst per lever flip: redstone settles first, then pistons walk the door.
        </p>
        <ActivityChart events={cert.events_per_tick} flips={rec.replay.flips} />
      </div>

      <div className="sheet-section">
        <div className="split">
          <div className="chart-block">
            <p className="eyebrow">Bill of materials</p>
            <h3>{totalBlocks} blocks, {cert.materials.length} kinds</h3>
            <p className="chart-note">Everything inside the scanned bounds, most used first.</p>
            <MaterialsTable materials={cert.materials} />
          </div>
          <div className="chart-block">
            <p className="eyebrow">Change footprint</p>
            <h3>Block changes per column</h3>
            <p className="chart-note">
              The door's XY silhouette; the stronger the blue, the more a column churned.
            </p>
            <Heatmap heatmap={cert.heatmap} lever={cert.lever} />
          </div>
        </div>
      </div>

      <div className="sheet-foot">
        <span>Door Certification Bureau · in-browser verified · no data left this tab</span>
        <span>
          {cert.sim_ticks} ticks · seed {cert.seed} ·{" "}
          <a href={certUrl} download={`${cert.name}.certificate.json`}>
            download certificate JSON
          </a>
        </span>
      </div>
    </div>
  );
}
