/** The Hall of Fame — seven plinths for the strangest and finest fliers
 * this browser has ever evolved, across every run (including the Slowpoke:
 * the slowest engine that still flies). Inductees are challenged live while
 * a run is evolving. */

import { HOF_META, HOF_ORDER, type Hof, type HofEntry } from "../hof";
import IsoThumb from "./IsoThumb";

interface Props {
  hof: Hof;
  onClear: () => void;
  onStage: (entry: HofEntry) => void;
}

const fmtDate = (t: number) =>
  new Date(t).toLocaleDateString(undefined, { month: "short", day: "numeric" });

export default function HallOfFame({ hof, onClear, onStage }: Props) {
  const inductees = HOF_ORDER.filter((c) => hof[c]);
  return (
    <div className="hof" data-testid="hall-of-fame">
      <div className="hof-intro">
        <p>
          Every run feeds the hall: one plinth per discipline, held by the best
          machine this browser has ever evolved. Click a plinth to fly it.
        </p>
        {inductees.length > 0 && (
          <button className="icon-btn" onClick={onClear}>
            clear hall
          </button>
        )}
      </div>
      <div className="hof-grid">
        {HOF_ORDER.map((cat) => {
          const meta = HOF_META[cat];
          const e = hof[cat];
          if (!e) {
            return (
              <div className="hof-card empty" key={cat}>
                <div className="hof-cat">{meta.title}</div>
                <div className="hof-blurb">{meta.blurb}</div>
                <div className="hof-vacant">plinth vacant</div>
              </div>
            );
          }
          return (
            <button
              className="hof-card"
              key={cat}
              onClick={() => onStage(e)}
              data-testid={`hof-${cat}`}
            >
              <div className="hof-cat">{meta.title}</div>
              <div className="hof-blurb">{meta.blurb}</div>
              <div className="hof-stage">
                <IsoThumb
                  blocks={e.blocks}
                  width={170}
                  label={`${meta.title} inductee: ${e.blocks.length} blocks`}
                />
              </div>
              <div className="hof-hero">{meta.hero(e.metrics)}</div>
              <div className="hof-sub">
                {e.metrics.speed.toFixed(2)} blk/s · {e.metrics.blocks} blocks ·
                gen {e.gen} · {fmtDate(e.when)}
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
