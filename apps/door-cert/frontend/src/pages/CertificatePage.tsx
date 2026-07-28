import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { getCertificate } from "../lib/api";
import type { Certificate } from "../lib/types";
import { Seal } from "../components/Seal";
import { StatTile } from "../components/StatTile";
import { ActivityChart } from "../components/ActivityChart";
import { Heatmap } from "../components/Heatmap";
import { MaterialsTable } from "../components/MaterialsTable";

const secs = (ticks: number) => (ticks / 20).toFixed(2);

export function CertificatePage() {
  const { id = "" } = useParams();
  const [cert, setCert] = useState<Certificate | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    getCertificate(id)
      .then((c) => live && setCert(c))
      .catch((e) => live && setError(e instanceof Error ? e.message : "failed to load"));
    return () => {
      live = false;
    };
  }, [id]);

  if (error)
    return (
      <div className="error-page">
        <p>Certificate {id} could not be loaded ({error}).</p>
        <p>
          <Link to="/">Submit a door</Link>
        </p>
      </div>
    );
  if (!cert) return <div className="loading">Retrieving certificate {id}…</div>;

  const [dx, dy, dz] = cert.dims;
  const movingBlocks = cert.heatmap.values.flat().filter((v) => v > 0).length;
  const peak = Math.max(...cert.events_per_tick.map((e) => e.piston + e.redstone));
  const totalBlocks = cert.materials.reduce((a, m) => a + m.count, 0);

  return (
    <div className="sheet">
      <div className="sheet-band">
        <span>Certificate no. {id}</span>
        <span>Seed {cert.seed}</span>
        <span>Issued {new Date().toISOString().slice(0, 10)}</span>
      </div>

      <div className="sheet-body">
        <div className="hero">
          <div>
            <p className="eyebrow">Piston door performance certificate</p>
            <h1>{cert.name}</h1>
            <p className="hero-sub">
              Simulated headless for {cert.sim_ticks} ticks at 20 tps. One full cycle observed:
              lever on, door open, lever off, door shut. All timings measured, not estimated.
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
          </div>
          <Seal openTicks={cert.open_ticks} />
        </div>
      </div>

      <div className="sheet-section">
        <p className="eyebrow">Exhibit A — cycle recording</p>
        <div className="exhibit-frame">
          <video src={cert.animation_url} controls autoPlay muted loop playsInline />
        </div>
        <p className="exhibit-caption">
          Full open–close cycle, rendered from the headless simulation.
        </p>
      </div>

      <div className="sheet-section">
        <p className="eyebrow">Measurements</p>
        <div className="tiles">
          <StatTile label="Open time" value={cert.open_ticks} unit=" ticks" sub={`${secs(cert.open_ticks)} s at 20 tps`} />
          <StatTile label="Close time" value={cert.close_ticks} unit=" ticks" sub={`${secs(cert.close_ticks)} s at 20 tps`} />
          <StatTile label="Moving blocks" value={movingBlocks} sub="columns with block changes" />
          <StatTile label="Sim length" value={cert.sim_ticks} unit=" ticks" sub={`peak ${peak} events in one tick`} />
        </div>
      </div>

      <div className="sheet-section chart-block">
        <p className="eyebrow">Activity trace</p>
        <h3>Events per simulation tick</h3>
        <p className="chart-note">
          Two bursts, one per lever flip: redstone settles first, then pistons walk the door.
        </p>
        <ActivityChart events={cert.events_per_tick} />
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
        <span>Door Certification Bureau · headless verified</span>
        <span>
          {cert.sim_ticks} ticks · seed {cert.seed} · nucleation engine
        </span>
      </div>
    </div>
  );
}
