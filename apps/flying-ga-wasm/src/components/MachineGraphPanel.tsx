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
 * that hides a block behind another. */

import { useEffect, useMemo, useState } from "react";

import { X_OFF, genomeToSnbt, travelRoom } from "../ga/snbt";
import type { BBox, Genome } from "../ga/genome";
import { loadEngine } from "../workers/engine";

export interface MachineGraph {
  groups: Array<{ id: number; cells: [number, number, number][] }>;
  devices: Array<{
    id: number;
    pos: [number, number, number];
    kind: "sticky_piston" | "piston" | "observer" | "source";
    facing: string;
    group: number;
    extended: boolean;
    can_extend: boolean;
    push: [number, number, number][];
    pull: [number, number, number][];
    influence: [number, number, number][];
  }>;
  edges: Array<{ kind: string; from: string; to: string }>;
  engines: Array<{ devices: number[]; cells: [number, number, number][] }>;
  payload: [number, number, number][];
  kickers: number[];
  dead_weight: [number, number, number][];
  rejections: Array<{ code: string; unconditional: boolean; reason: string }>;
  rejected: boolean;
  rejected_for_sustained: boolean;
  engine_search_truncated: boolean;
}

type Role = "engine" | "payload" | "kicker" | "dead";

const ROLE_STYLE: Record<Role, { fill: string; label: string; hint: string }> = {
  engine: {
    fill: "#2f9e6b",
    label: "engine",
    hint: "the minimal set that shoves itself along",
  },
  payload: {
    fill: "#3d7fd6",
    label: "payload",
    hint: "carried by the engine, does no work",
  },
  kicker: {
    fill: "#d99a2b",
    label: "kicker",
    hint: "fires once to start it, then irrelevant",
  },
  dead: {
    fill: "#8a8f98",
    label: "dead weight",
    hint: "neither driven nor driving",
  },
};

const key = ([x, y, z]: [number, number, number]) => `${x},${y},${z}`;

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
  const roles = useMemo(() => {
    const map = new Map<string, Role>();
    if (!graph) return map;
    // Painted weakest-claim first, so the last writer wins. The precedence is
    // engine > kicker > payload > dead, and the kicker/payload order is the
    // part that is easy to get wrong: a kicker usually sits INSIDE the push
    // closure — it is bolted to the machine it starts — so painting payload
    // after it silently repainted it blue and the panel under-reported kickers
    // against the JSON (0 shown against 2, 1 against 3). Being carried does not
    // stop a device being the thing that starts the machine.
    for (const cell of graph.dead_weight) map.set(key(cell), "dead");
    for (const cell of graph.payload) map.set(key(cell), "payload");
    for (const id of graph.kickers) {
      const d = graph.devices[id];
      if (d) map.set(key(d.pos), "kicker");
    }
    // Engine last: a cell that is engine is engine, whatever else claimed it.
    for (const e of graph.engines) for (const cell of e.cells) map.set(key(cell), "engine");
    return map;
  }, [graph]);

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

  const counts: Record<Role, number> = { engine: 0, payload: 0, kicker: 0, dead: 0 };
  for (const role of roles.values()) counts[role] += 1;

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
                  const role = roles.get(key(cell)) ?? "dead";
                  const device = deviceAt.get(key(cell));
                  const [x, , z] = cell;
                  return (
                    <g key={key(cell)}>
                      <rect
                        x={(x - minX) * cellPx}
                        y={(z - minZ) * cellPx}
                        width={cellPx - 1}
                        height={cellPx - 1}
                        fill={ROLE_STYLE[role].fill}
                        rx={2}
                      >
                        <title>
                          {`${x},${y},${z} — ${ROLE_STYLE[role].label}` +
                            (device ? ` — ${device.kind} facing ${device.facing}` : "")}
                        </title>
                      </rect>
                      {device && (
                        <text
                          x={(x - minX) * cellPx + cellPx / 2}
                          y={(z - minZ) * cellPx + cellPx / 2 + 4}
                          textAnchor="middle"
                          fontSize={10}
                          fill="#fff"
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
        {(Object.keys(ROLE_STYLE) as Role[]).map((role) => (
          <li key={role} title={ROLE_STYLE[role].hint}>
            <span className="swatch" style={{ background: ROLE_STYLE[role].fill }} />
            {ROLE_STYLE[role].label}
            <b>{counts[role]}</b>
          </li>
        ))}
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
