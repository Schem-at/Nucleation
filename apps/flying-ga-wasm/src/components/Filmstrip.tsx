/** Gallery filmstrip of every generation that produced a new champion.
 * Click (or Enter) a frame to load that machine's flight loop in the stage. */

import type { BestRecord } from "../types";
import IsoThumb from "./IsoThumb";

interface Props {
  bests: BestRecord[];
  /** gen of the record currently on stage, or null when following the champion. */
  selectedGen: number | null;
  onSelect: (gen: number | null) => void;
}

export default function Filmstrip({ bests, selectedGen, onSelect }: Props) {
  if (bests.length === 0) {
    return (
      <div className="film-empty">
        Each new champion is pinned here as the run evolves.
      </div>
    );
  }
  const championGen = bests[bests.length - 1].gen;
  return (
    <div className="filmstrip" role="listbox" aria-label="Generation champions">
      {bests.map((b) => {
        const isChampion = b.gen === championGen;
        const active =
          selectedGen === b.gen || (selectedGen === null && isChampion);
        return (
          <button
            key={b.gen}
            className={`film-cell${active ? " active" : ""}`}
            role="option"
            aria-selected={active}
            onClick={() => onSelect(isChampion ? null : b.gen)}
            title={`gen ${b.gen} — ${b.fitness.toFixed(1)} blocks`}
          >
            <IsoThumb blocks={b.blocks} width={86} />
            <span className="film-label">
              <b>g{b.gen}</b> {b.fitness.toFixed(1)}
            </span>
            {isChampion && <span className="film-crown">champion</span>}
          </button>
        );
      })}
    </div>
  );
}
