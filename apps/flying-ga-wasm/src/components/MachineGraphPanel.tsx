/** What a machine is *made of*, drawn.
 *
 * The rest of the app answers "how far did it go". This panel answers "which
 * part of it is the engine" — and it answers it without simulating anything.
 * Everything here is read straight out of `TickSimulation.machineGraphJson`,
 * which is `crates/mc-tick/src/machine_graph.rs` talking: adhesion groups,
 * resolved push sets, the drive graph, and the minimal self-translating
 * subgraph that *is* the engine.
 *
 * The projection is deliberately flat. Flying machines are small and mostly
 * planar, and a legible top-down slice per y-layer beats a pretty isometric one
 * that hides a block behind another. What it cannot show is the graph itself —
 * a slice has nowhere to put an edge between two layers — so the same
 * classification is also drawn on the 3-D viewer above (`meta.ts` + `metaGl.ts`
 * + `GLBlocks`). Both read their roles, precedence and palette out of `meta.ts`
 * so the two projections cannot drift apart and quietly disagree. */

import { useEffect, useMemo, useState } from "react";

import { X_OFF, genomeToSnbt, travelRoom } from "../ga/snbt";
import type { BBox, Genome } from "../ga/genome";
import {
  ROLE_ORDER,
  cellKey as key,
  hexCss,
  roleCounts,
  roleMap,
  rolePalette,
  useIsDark,
  type MachineGraph,
  type Role,
} from "../meta";
import { loadEngine } from "../workers/engine";

export type { MachineGraph } from "../meta";

/** Fetch the static graph for one genome. */
export function useMachineGraph(
  genome: Genome | null,
  bbox: BBox,
  evalTicks: number,
): { graph: MachineGraph | null; error: string | null } {
  const [graph, setGraph] = useState<MachineGraph | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    if (!genome) {
      setGraph(null);
      return;
    }
    (async () => {
      try {
        const eng = await loadEngine();
        const travel = travelRoom(evalTicks);
        const sim = eng.TickSimulation.fromSnbt(
          genomeToSnbt(genome, bbox, travel),
          eng.TickSettleMode.Quiet,
          0,
          0,
          0,
          "",
        );
        const parsed = JSON.parse(sim.machineGraphJson()) as MachineGraph;
        if (live) {
          setGraph(parsed);
          setError(null);
        }
      } catch (e) {
        if (live) {
          setGraph(null);
          setError(e instanceof Error ? e.message : String(e));
        }
      }
    })();
    return () => {
      live = false;
    };
  }, [genome, bbox, evalTicks]);

  return { graph, error };
}

interface Props {
  graph: MachineGraph | null;
  error?: string | null;
}

export default function MachineGraphPanel({ graph, error }: Props) {
  const dark = useIsDark();
  const ROLE_STYLE = rolePalette(dark);
  // Precedence lives in meta.ts, shared with the 3-D overlay.
  const roles = useMemo(() => roleMap(graph), [graph]);

  if (error) return <p className="graph-panel-error">graph unavailable: {error}</p>;
  if (!graph) return <p className="graph-panel-empty">no machine selected</p>;

  const cells = [...roles.keys()].map((k) => k.split(",").map(Number) as [number, number, number]);
  if (cells.length === 0) return <p className="graph-panel-empty">empty build</p>;

  const layers = [...new Set(cells.map(([, y]) => y))].sort((a, b) => b - a);
  const xs = cells.map(([x]) => x);
  const zs = cells.map(([, , z]) => z);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minZ = Math.min(...zs);
  const maxZ = Math.max(...zs);
  const cellPx = 18;
  const w = (maxX - minX + 1) * cellPx;
  const h = (maxZ - minZ + 1) * cellPx;

  const counts = roleCounts(roles);

  const deviceAt = new Map(graph.devices.map((d) => [key(d.pos), d]));

  return (
    <div className="graph-panel">
      <div className="graph-panel-verdict">
        {graph.rejected ? (
          <span className="graph-verdict bad">
            provably immobile — {graph.rejections.find((r) => r.unconditional)?.reason}
          </span>
        ) : graph.rejected_for_sustained ? (
          <span className="graph-verdict warn">
            cannot sustain flight — {graph.rejections[0]?.reason}
          </span>
        ) : (
          <span className="graph-verdict ok">
            not disproved — {graph.engines.length} engine
            {graph.engines.length === 1 ? "" : "s"} found
          </span>
        )}
      </div>

      <div className="graph-panel-layers">
        {layers.map((y) => (
          <figure key={y} className="graph-layer">
            <svg width={w} height={h} role="img" aria-label={`machine sections at y=${y}`}>
              {cells
                .filter(([, cy]) => cy === y)
                .map((cell) => {
                  const role: Role = roles.get(key(cell)) ?? "dead";
                  const style = ROLE_STYLE[role];
                  const device = deviceAt.get(key(cell));
                  const [x, , z] = cell;
                  // Dead weight is a CAGE here too, for the same reason it is
                  // one on the 3-D stage: it has no hue to spend (meta.ts).
                  const caged = style.mark === "cage";
                  return (
                    <g key={key(cell)}>
                      <rect
                        x={(x - minX) * cellPx + (caged ? 0.75 : 0)}
                        y={(z - minZ) * cellPx + (caged ? 0.75 : 0)}
                        width={cellPx - 1 - (caged ? 1.5 : 0)}
                        height={cellPx - 1 - (caged ? 1.5 : 0)}
                        fill={caged ? "none" : hexCss(style.hex)}
                        stroke={caged ? hexCss(style.hex) : "none"}
                        strokeWidth={caged ? 1.5 : 0}
                        rx={2}
                      >
                        <title>
                          {`${x},${y},${z} — ${style.label}` +
                            (device ? ` — ${device.kind} facing ${device.facing}` : "")}
                        </title>
                      </rect>
                      {device && (
                        <text
                          x={(x - minX) * cellPx + cellPx / 2}
                          y={(z - minZ) * cellPx + cellPx / 2 + 4}
                          textAnchor="middle"
                          fontSize={10}
                          /* An unfilled cage has no dark plate to knock the
                             glyph out of, so it wears the cage's own ink. */
                          fill={caged ? hexCss(style.hex) : "#fff"}
                        >
                          {device.kind === "observer"
                            ? "o"
                            : device.kind === "source"
                              ? "r"
                              : device.kind === "sticky_piston"
                                ? "P"
                                : "p"}
                        </text>
                      )}
                    </g>
                  );
                })}
            </svg>
            <figcaption>y = {y}</figcaption>
          </figure>
        ))}
      </div>

      <ul className="graph-legend">
        {ROLE_ORDER.map((role) => {
          const s = ROLE_STYLE[role];
          return (
            <li key={role} title={s.hint}>
              <span
                className={"swatch" + (s.mark === "cage" ? " cage" : "")}
                style={
                  s.mark === "cage"
                    ? { borderColor: hexCss(s.hex) }
                    : { background: hexCss(s.hex) }
                }
              />
              {s.label}
              <b>{counts[role]}</b>
            </li>
          );
        })}
      </ul>

      <details className="graph-edges">
        <summary>
          {graph.groups.length} group{graph.groups.length === 1 ? "" : "s"},{" "}
          {graph.devices.length} device{graph.devices.length === 1 ? "" : "s"},{" "}
          {graph.edges.length} edge{graph.edges.length === 1 ? "" : "s"}
        </summary>
        <table>
          <tbody>
            {graph.edges.map((e, i) => (
              <tr key={i}>
                <td>{e.from}</td>
                <td>{e.kind}</td>
                <td>{e.to}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </details>

      {graph.engine_search_truncated && (
        <p className="graph-panel-note">
          engine search hit its candidate cap — the list may be incomplete
        </p>
      )}
    </div>
  );
}

export { X_OFF };
