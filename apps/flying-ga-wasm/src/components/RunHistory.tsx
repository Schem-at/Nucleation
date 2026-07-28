/** Past runs from localStorage — browsable after a fresh page load. */

import type { RunSummary } from "../storage";

interface Props {
  runs: RunSummary[];
  activeId: string | null;
  browsingId: string | null;
  onOpen: (id: string) => void;
  onDelete: (id: string) => void;
}

const fmtTime = (t: number) =>
  new Date(t).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });

export default function RunHistory({
  runs,
  activeId,
  browsingId,
  onOpen,
  onDelete,
}: Props) {
  if (runs.length === 0) {
    return <div className="hist-empty">Finished runs are saved here.</div>;
  }
  return (
    <ul className="hist-list">
      {runs.map((r) => {
        const live = r.id === activeId;
        return (
          <li
            key={r.id}
            className={`hist-item${r.id === browsingId ? " sel" : ""}${live ? " live" : ""}`}
          >
            <button
              className="hist-open"
              onClick={() => onOpen(r.id)}
              disabled={live}
            >
              <span className="hist-when">{fmtTime(r.startedAt)}</span>
              <span className="hist-meta">
                {r.best.toFixed(1)} blocks · {r.generation + 1} gens
                {live ? " · live" : ""}
              </span>
            </button>
            {!live && (
              <button
                className="hist-del"
                aria-label={`Delete run from ${fmtTime(r.startedAt)}`}
                onClick={() => onDelete(r.id)}
              >
                ×
              </button>
            )}
          </li>
        );
      })}
    </ul>
  );
}
