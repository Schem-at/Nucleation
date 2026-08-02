/** Static machine inspector for the leaderboard selection (the animated
 * champion lives in FlightLoop). */

import { useCallback, useEffect, useMemo, useReducer, useState } from "react";
import { colorOf } from "../iso";
import { blockKind } from "../ga/alphabet";
import { X_OFF } from "../ga/snbt";
import {
  EDGE_STYLE,
  ROLE_ORDER,
  graphInk,
  hexCss,
  metaStructure,
  rolePalette,
  useIsDark,
  type EdgeKind,
  type MachineGraph,
} from "../meta";
import { speedOf } from "../metrics";
import { ensureTextureIndex, onTexturesChanged, textureURL } from "../textures";
import type { LeaderboardEntry } from "../types";
import GLBlocks from "./GLBlocks";
import IsoThumb from "./IsoThumb";

interface Props {
  machine: LeaderboardEntry | null;
  evalTicks: number | null;
  /** Static machine graph for `machine`, or null while it loads / on error. */
  graph?: MachineGraph | null;
  /** Why the graph is missing, if it is. */
  graphError?: string | null;
  /** True while the viewer auto-follows the leaderboard leader. */
  following?: boolean;
  /** User grabbed the orbit controls — parent should stop auto-follow. */
  onInteract?: () => void;
  /** Chip click: re-enable follow-the-leader (camera re-fits). */
  onResumeFollow?: () => void;
}

/** The legend's copy of a stage edge mark — the same four distinctions the
 * three.js overlay draws (solid / solid-with-head / long-dash / short-dash),
 * in the same achromatic ink, so the legend is a key and not an approximation. */
function EdgeMark({
  mark,
  ink,
}: {
  mark: (typeof EDGE_STYLE)[EdgeKind]["mark"];
  ink: string;
}) {
  return (
    <svg width="26" height="12" aria-hidden="true">
      {mark === "arrow" ? (
        <>
          <rect x="1" y="5" width="16" height="2" fill={ink} />
          <polygon points="17,2 25,6 17,10" fill={ink} />
        </>
      ) : mark === "bond" ? (
        <rect x="1" y="5" width="24" height="2" fill={ink} />
      ) : (
        <line
          x1="1"
          y1="6"
          x2="25"
          y2="6"
          stroke={ink}
          strokeWidth="2"
          strokeDasharray={mark === "dash" ? "5 3" : "1.5 2.5"}
        />
      )}
    </svg>
  );
}

export default function MachineViewer({
  machine,
  evalTicks,
  graph = null,
  graphError = null,
  following = true,
  onInteract,
  onResumeFollow,
}: Props) {
  // Legend swatches pick up real textures as they decode.
  const [, bump] = useReducer((n: number) => n + 1, 0);
  const [glFail, setGlFail] = useState(false);
  const [fitNonce, setFitNonce] = useState(0);
  // The overlay is the point of the panel, so it is on by default — including
  // the x-ray, without which the graph layer is buried in the build on
  // anything denser than a bar. Each layer switches off independently.
  const [showRoles, setShowRoles] = useState(true);
  const [showGraph, setShowGraph] = useState(true);
  const [ghost, setGhost] = useState(true);
  const dark = useIsDark();
  const palette = rolePalette(dark);
  const ink = hexCss(graphInk(dark));

  const metaStruct = useMemo(() => metaStructure(graph, X_OFF), [graph]);
  const edgeCounts = useMemo(() => {
    const c = {} as Record<EdgeKind, number>;
    for (const e of metaStruct.edges) c[e.kind] = (c[e.kind] ?? 0) + 1;
    return c;
  }, [metaStruct]);
  useEffect(() => {
    ensureTextureIndex();
    return onTexturesChanged(bump);
  }, []);
  const onFail = useCallback(() => setGlFail(true), []);
  const resume = useCallback(() => {
    setFitNonce((n) => n + 1);
    onResumeFollow?.();
  }, [onResumeFollow]);

  const meta = useMemo(() => {
    if (!machine || machine.blocks.length === 0) return null;
    const dims = (["x", "y", "z"] as const).map((k) => {
      const vs = machine.blocks.map((b) => b[k]);
      return Math.max(...vs) - Math.min(...vs) + 1;
    });
    const kinds = [...new Set(machine.blocks.map((b) => blockKind(b.state)))];
    return { dims, kinds };
  }, [machine]);

  if (!machine || !meta) {
    return (
      <div className="viewer-empty">
        Select a machine from the leaderboard to inspect it.
      </div>
    );
  }

  return (
    <div className="viewer">
      <div className="viewer-stats">
        <div className="big" data-testid="viewer-speed">
          {(machine.speed ?? (evalTicks ? speedOf(machine.fitness, evalTicks) : 0)).toFixed(2)}
          <span className="unit">blk/s @ 20 tps</span>
        </div>
        <div className="kv">
          distance
          <b>
            {machine.fitness.toFixed(1)} blk
            {evalTicks ? ` / ${evalTicks}t` : ""}
          </b>
        </div>
        <div className="kv">
          blocks<b>{machine.blocks.length}</b>
        </div>
        <div className="kv">
          size<b>{meta.dims.join("×")}</b>
        </div>
        {machine.metrics && machine.metrics.cargo > 0 && (
          <div className="kv">
            cargo<b>{machine.metrics.cargo}</b>
          </div>
        )}
        {machine.metrics && machine.metrics.robustness >= 0 && (
          <div className="kv">
            kick tol.<b>{Math.round(machine.metrics.robustness * 100)}%</b>
          </div>
        )}
        <div className="kv">
          found gen<b>{machine.gen}</b>
        </div>
      </div>

      {!glFail && (
        <div className="meta-controls" data-testid="meta-controls">
          <span className="meta-controls-title">meta structure</span>
          <label title="Colour every block by the role the static analysis gave it">
            <input
              type="checkbox"
              checked={showRoles}
              onChange={(e) => setShowRoles(e.target.checked)}
              data-testid="meta-roles"
            />
            roles
          </label>
          <label title="Node markers at group centroids and devices, with the edges between them">
            <input
              type="checkbox"
              checked={showGraph}
              onChange={(e) => setShowGraph(e.target.checked)}
              data-testid="meta-graph"
            />
            graph
          </label>
          <label title="Drop the build to a ghost so interior nodes and edges stop being hidden by it">
            <input
              type="checkbox"
              checked={ghost}
              onChange={(e) => setGhost(e.target.checked)}
              data-testid="meta-ghost"
            />
            x-ray
          </label>
          {/* Three checkboxes that quietly do nothing are worse than no
              checkboxes. The engine refuses some structures outright — a
              note block makes TickSimulation.fromSnbt throw — and when it
              does, say so here rather than only in the panel below. */}
          {!graph && (
            <span className="meta-controls-unavailable" data-testid="meta-unavailable">
              {graphError
                ? `no analysis for this machine — ${graphError}`
                : "analysing…"}
            </span>
          )}
        </div>
      )}

      <div className="viewer-stage">
        {glFail ? (
          <IsoThumb
            blocks={machine.blocks}
            width={340}
            label={`Voxel structure of ${machine.name ?? machine.id}: ${machine.blocks.length} blocks`}
          />
        ) : (
          <GLBlocks
            blocks={machine.blocks}
            height={260}
            label={`Voxel structure of ${machine.name ?? machine.id}: ${machine.blocks.length} blocks`}
            onFail={onFail}
            onUserInteract={onInteract}
            fitNonce={fitNonce}
            debugId="viewer"
            meta={graph ? metaStruct : null}
            showRoles={showRoles}
            showGraph={showGraph}
            ghost={ghost}
            dark={dark}
          />
        )}
        {!following && (
          <button
            type="button"
            className="follow-chip"
            onClick={resume}
            data-testid="viewer-follow-chip"
            title="Camera and selection are yours — click to re-follow the leaderboard leader and re-frame"
          >
            following leader ⏸ — resume
          </button>
        )}
      </div>

      {!glFail && graph && (showRoles || showGraph) && (
        <div className="meta-legend" data-testid="meta-legend">
          {showRoles && (
            <ul className="meta-roles-legend">
              {ROLE_ORDER.map((role) => {
                const s = palette[role];
                return (
                  <li key={role} title={s.hint}>
                    {/* Dead weight's swatch is a hollow box, not a filled one,
                        because its mark on the stage is a cage and a legend
                        that promised a colour would be promising a colour that
                        does not exist. */}
                    <span
                      className={"swatch" + (s.mark === "cage" ? " cage" : "")}
                      style={
                        s.mark === "cage"
                          ? { borderColor: hexCss(s.hex) }
                          : { background: hexCss(s.hex) }
                      }
                    />
                    {s.label}
                    <b>{metaStruct.counts[role]}</b>
                  </li>
                );
              })}
            </ul>
          )}

          {showGraph && (
            <ul className="meta-graph-legend" style={{ color: ink }}>
              <li title="One maximal adhesion component — a body that moves as one">
                <svg width="26" height="12" aria-hidden="true">
                  <polygon points="13,2 18,6 13,10 8,6" fill={ink} />
                </svg>
                group
                <b>{metaStruct.nodes.filter((n) => n.kind === "group").length}</b>
              </li>
              {/* A device wears no body marker — its cell already carries a
                  role core — so the legend shows what the stage shows: a stub
                  leaving the cell centre in the direction it faces. */}
              <li title="A piston, observer or power source; the stub points the way it faces">
                <svg width="26" height="12" aria-hidden="true">
                  <circle cx="6" cy="6" r="1.6" fill={ink} />
                  <rect x="7" y="5" width="13" height="2" fill={ink} />
                </svg>
                device facing
                <b>{metaStruct.nodes.filter((n) => n.kind === "device").length}</b>
              </li>
              {(Object.keys(EDGE_STYLE) as EdgeKind[]).map((kind) => {
                const s = EDGE_STYLE[kind];
                return (
                  <li key={kind} title={s.hint}>
                    <EdgeMark mark={s.mark} ink={ink} />
                    {s.label}
                    <b>{edgeCounts[kind] ?? 0}</b>
                  </li>
                );
              })}
            </ul>
          )}

          {/* The one thing a picture like this can quietly lie about. */}
          <p className="meta-phase-note" data-testid="meta-phase-note">
            Static analysis of the <b>rest state</b> — resolved push sets, not a
            simulation, and not re-derived per tick. The viewer shows that same
            rest state, so the colouring is of the machine you are looking at.
            The animated stage is not overlaid: there the classification would
            be carried along rather than recomputed.
          </p>
        </div>
      )}

      <div className="block-legend">
        {meta.kinds.map((k) => {
          const c = colorOf(`minecraft:${k}`);
          const tex = textureURL(`minecraft:${k}`, "side");
          return (
            <span className="item" key={k}>
              <span
                className="swatch"
                style={
                  tex
                    ? {
                        background: `${c.base} url(${tex}) center / cover`,
                        imageRendering: "pixelated",
                      }
                    : { background: c.base }
                }
              />
              {c.label}
            </span>
          );
        })}
      </div>
    </div>
  );
}
