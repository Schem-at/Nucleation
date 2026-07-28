import { speedOf } from "../metrics";
import type { LeaderboardEntry } from "../types";

interface Props {
  entries: LeaderboardEntry[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  /** Eval window (ticks) — blk/s fallback for records without metrics. */
  evalTicks: number | null;
}

export default function Leaderboard({ entries, selectedId, onSelect, evalTicks }: Props) {
  if (entries.length === 0) {
    return <div className="lb-empty">No machines evaluated yet.</div>;
  }
  const bps = (m: LeaderboardEntry): number =>
    m.speed ?? (evalTicks ? speedOf(m.fitness, evalTicks) : 0);
  return (
    <table className="lb-table">
      <thead>
        <tr>
          <th className="rank">#</th>
          <th>Machine</th>
          <th className="num">blk/s</th>
          <th className="num">Dist (blocks)</th>
          <th className="num">Size</th>
          <th className="num">Gen</th>
        </tr>
      </thead>
      <tbody>
        {entries.map((m, i) => (
          <tr
            key={m.id}
            className={`row${m.id === selectedId ? " sel" : ""}`}
            onClick={() => onSelect(m.id)}
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onSelect(m.id);
              }
            }}
            aria-selected={m.id === selectedId}
          >
            <td className="rank">{i + 1}</td>
            <td className="name">{m.name ?? m.id}</td>
            <td className="num fit" data-testid="lb-speed">
              {bps(m).toFixed(2)}
            </td>
            <td className="num">{m.fitness.toFixed(1)}</td>
            <td className="num">{m.blocks.length}</td>
            <td className="num">{m.gen}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
