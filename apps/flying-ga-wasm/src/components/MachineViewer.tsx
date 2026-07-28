/** Static machine inspector for the leaderboard selection (the animated
 * champion lives in FlightLoop). */

import { useCallback, useEffect, useMemo, useReducer, useState } from "react";
import { colorOf } from "../iso";
import { blockKind } from "../ga/alphabet";
import { speedOf } from "../metrics";
import { ensureTextureIndex, onTexturesChanged, textureURL } from "../textures";
import type { LeaderboardEntry } from "../types";
import GLBlocks from "./GLBlocks";
import IsoThumb from "./IsoThumb";

interface Props {
  machine: LeaderboardEntry | null;
  evalTicks: number | null;
  /** True while the viewer auto-follows the leaderboard leader. */
  following?: boolean;
  /** User grabbed the orbit controls — parent should stop auto-follow. */
  onInteract?: () => void;
  /** Chip click: re-enable follow-the-leader (camera re-fits). */
  onResumeFollow?: () => void;
}

export default function MachineViewer({
  machine,
  evalTicks,
  following = true,
  onInteract,
  onResumeFollow,
}: Props) {
  // Legend swatches pick up real textures as they decode.
  const [, bump] = useReducer((n: number) => n + 1, 0);
  const [glFail, setGlFail] = useState(false);
  const [fitNonce, setFitNonce] = useState(0);
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
