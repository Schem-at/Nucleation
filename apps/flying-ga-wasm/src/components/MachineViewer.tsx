/** Static machine inspector for the leaderboard selection (the animated
 * champion lives in FlightLoop). */

import { useEffect, useMemo, useReducer } from "react";
import { colorOf } from "../iso";
import { blockKind } from "../ga/alphabet";
import { ensureTextureIndex, onTexturesChanged, textureURL } from "../textures";
import type { LeaderboardEntry } from "../types";
import IsoThumb from "./IsoThumb";

interface Props {
  machine: LeaderboardEntry | null;
  evalTicks: number | null;
}

export default function MachineViewer({ machine, evalTicks }: Props) {
  // Legend swatches pick up real textures as they decode.
  const [, bump] = useReducer((n: number) => n + 1, 0);
  useEffect(() => {
    ensureTextureIndex();
    return onTexturesChanged(bump);
  }, []);

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
        <div className="big">
          {machine.fitness.toFixed(1)}
          <span className="unit">
            blocks flown{evalTicks ? ` / ${evalTicks} ticks` : ""}
          </span>
        </div>
        <div className="kv">
          blocks<b>{machine.blocks.length}</b>
        </div>
        <div className="kv">
          size<b>{meta.dims.join("×")}</b>
        </div>
        <div className="kv">
          found gen<b>{machine.gen}</b>
        </div>
      </div>

      <div className="viewer-stage">
        <IsoThumb
          blocks={machine.blocks}
          width={340}
          label={`Voxel structure of ${machine.name ?? machine.id}: ${machine.blocks.length} blocks`}
        />
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
