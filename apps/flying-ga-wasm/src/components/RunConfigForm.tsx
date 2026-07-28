import { useState } from "react";
import type { RunConfig } from "../types";

interface Props {
  running: boolean;
  onStart: (cfg: RunConfig) => void;
  onStop: () => void;
  error: string | null;
}

export default function RunConfigForm({ running, onStart, onStop, error }: Props) {
  const [population, setPopulation] = useState(96);
  /** Blank = run forever until Stop. */
  const [generations, setGenerations] = useState<string>("");
  const [mutationRate, setMutationRate] = useState(0.05);
  const [bx, setBx] = useState(5);
  const [by, setBy] = useState(3);
  const [bz, setBz] = useState(3);
  const [evalTicks, setEvalTicks] = useState(600);
  const [seed, setSeed] = useState(42);
  const [workers, setWorkers] = useState(4);

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const gens = generations.trim() === "" ? null : parseInt(generations, 10);
    onStart({
      population,
      generations: gens !== null && Number.isFinite(gens) && gens > 0 ? gens : null,
      mutation_rate: mutationRate,
      bbox: [bx, by, bz],
      eval_ticks: evalTicks,
      seed,
      workers: Math.max(1, Math.min(6, workers)),
    });
  };

  const num =
    (set: (v: number) => void, int = true) =>
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const v = int ? parseInt(e.target.value, 10) : parseFloat(e.target.value);
      if (!Number.isNaN(v)) set(v);
    };

  return (
    <form className="cfg" onSubmit={submit}>
      <div className="row2">
        <div className="field">
          <label htmlFor="cfg-pop">Population</label>
          <input id="cfg-pop" type="number" min={2} max={4096} value={population} onChange={num(setPopulation)} disabled={running} />
        </div>
        <div className="field">
          <label htmlFor="cfg-gens">Target gens</label>
          <input
            id="cfg-gens"
            type="number"
            min={1}
            placeholder="∞"
            value={generations}
            onChange={(e) => setGenerations(e.target.value)}
            disabled={running}
          />
        </div>
      </div>
      <p className="hint">Leave target blank to evolve until you press Stop.</p>

      <div className="field">
        <label htmlFor="cfg-mut">Mutation rate</label>
        <input id="cfg-mut" type="number" min={0} max={1} step={0.01} value={mutationRate} onChange={num(setMutationRate, false)} disabled={running} />
      </div>

      <div className="field">
        <label>Bounding box (x·y·z)</label>
        <div className="row3">
          <input aria-label="Bounding box x" type="number" min={1} max={16} value={bx} onChange={num(setBx)} disabled={running} />
          <input aria-label="Bounding box y" type="number" min={1} max={16} value={by} onChange={num(setBy)} disabled={running} />
          <input aria-label="Bounding box z" type="number" min={1} max={16} value={bz} onChange={num(setBz)} disabled={running} />
        </div>
        <p className="hint">Machines evolve inside this volume.</p>
      </div>

      <div className="row2">
        <div className="field">
          <label htmlFor="cfg-ticks">Eval ticks</label>
          <input id="cfg-ticks" type="number" min={20} max={20000} step={20} value={evalTicks} onChange={num(setEvalTicks)} disabled={running} />
        </div>
        <div className="field">
          <label htmlFor="cfg-seed">Seed</label>
          <input id="cfg-seed" type="number" value={seed} onChange={num(setSeed)} disabled={running} />
        </div>
      </div>

      <div className="field">
        <label htmlFor="cfg-workers">Workers ({workers})</label>
        <input
          id="cfg-workers"
          type="range"
          min={1}
          max={6}
          value={workers}
          onChange={num(setWorkers)}
          disabled={running}
        />
        <p className="hint">Each worker loads its own 11 MB engine, once.</p>
      </div>

      {error && <p className="cfg-error">{error}</p>}

      {running ? (
        <button type="button" className="btn-stop" onClick={onStop} data-testid="stop-run">
          Stop run
        </button>
      ) : (
        <button type="submit" className="btn-start" data-testid="start-run">
          Start evolution
        </button>
      )}
    </form>
  );
}
