/** Live evolution events — the run narrates itself: run starts, new
 * champions, new Pareto points. Newest first. */

import type { EvolutionEvent } from "../types";

interface Props {
  events: EvolutionEvent[];
}

const KIND_TAG: Record<EvolutionEvent["kind"], string> = {
  run: "run",
  champion: "champ",
  pareto: "front",
  slowpoke: "slow",
  config: "cfg",
  retired: "shelf",
  pause: "hold",
  mutation: "mut",
};

export default function EventsFeed({ events }: Props) {
  if (events.length === 0) {
    return (
      <div className="hist-empty">The run narrates itself here as it evolves.</div>
    );
  }
  return (
    <ol className="events" aria-label="Evolution events, newest first" data-testid="events-feed">
      {[...events].reverse().map((e, i) => (
        <li key={events.length - i} className={`ev ev-${e.kind}`}>
          <span className="ev-gen">g{e.gen}</span>
          <span className={`ev-tag ev-tag-${e.kind}`}>{KIND_TAG[e.kind]}</span>
          <span className="ev-text">{e.text}</span>
        </li>
      ))}
    </ol>
  );
}
