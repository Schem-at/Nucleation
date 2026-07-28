import type { LeaderboardEntry } from "../api";

interface Props {
  entries: LeaderboardEntry[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}

export default function Leaderboard({ entries, selectedId, onSelect }: Props) {
  if (entries.length === 0) {
    return <div className="lb-empty">No machines evaluated yet.</div>;
  }
  return (
    <table className="lb-table">
      <thead>
        <tr>
          <th className="rank">#</th>
          <th>Machine</th>
          <th className="num">Fitness (blocks)</th>
          <th className="num">Blocks</th>
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
            <td className="num fit">{m.fitness.toFixed(1)}</td>
            <td className="num">{m.blocks.length}</td>
            <td className="num">{m.gen}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
