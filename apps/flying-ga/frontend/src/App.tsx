import { useCallback, useEffect, useRef, useState } from "react";
import { createRun, getRun, stopRun } from "./api";
import type { RunConfigInput, RunState } from "./api";
import RunConfigForm from "./components/RunConfigForm";
import FitnessChart from "./components/FitnessChart";
import Leaderboard from "./components/Leaderboard";
import MachineViewer from "./components/MachineViewer";

const POLL_MS = 600;

function useTheme() {
  const [theme, setTheme] = useState<string | null>(
    () => document.documentElement.dataset.theme ?? null,
  );
  const toggle = () => {
    const dark =
      theme === "dark" ||
      (theme === null && matchMedia("(prefers-color-scheme: dark)").matches);
    const next = dark ? "light" : "dark";
    document.documentElement.dataset.theme = next;
    localStorage.setItem("fga-theme", next);
    setTheme(next);
  };
  return { toggle };
}

export default function App() {
  const { toggle } = useTheme();
  const [runId, setRunId] = useState<string | null>(null);
  const [run, setRun] = useState<RunState | null>(null);
  const [config, setConfig] = useState<RunConfigInput | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const userPicked = useRef(false);

  const start = useCallback(async (cfg: RunConfigInput) => {
    setError(null);
    try {
      const { id } = await createRun(cfg);
      setConfig(cfg);
      setRun(null);
      setSelectedId(null);
      userPicked.current = false;
      setRunId(id);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not start the run.");
    }
  }, []);

  const stop = useCallback(async () => {
    if (!runId) return;
    try {
      await stopRun(runId);
      setRun(await getRun(runId));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not stop the run.");
    }
  }, [runId]);

  // Poll while running.
  useEffect(() => {
    if (!runId) return;
    let live = true;
    let timer: number | undefined;
    const tick = async () => {
      try {
        const state = await getRun(runId);
        if (!live) return;
        setRun(state);
        if (state.status === "running") {
          timer = window.setTimeout(tick, POLL_MS);
        }
      } catch (e) {
        if (live) setError(e instanceof Error ? e.message : "Polling failed.");
      }
    };
    tick();
    return () => {
      live = false;
      if (timer) clearTimeout(timer);
    };
  }, [runId]);

  // Follow the leader until the user picks a machine manually.
  useEffect(() => {
    if (!run || run.leaderboard.length === 0) return;
    if (!userPicked.current) setSelectedId(run.leaderboard[0].id);
    else if (!run.leaderboard.some((m) => m.id === selectedId))
      setSelectedId(run.leaderboard[0].id);
  }, [run, selectedId]);

  const selected =
    run?.leaderboard.find((m) => m.id === selectedId) ?? null;
  const running = run?.status === "running";
  const totalGens = run?.total_generations ?? config?.generations ?? 1;

  return (
    <div className="shell">
      <header className="topbar">
        <svg className="logo" width="26" height="26" viewBox="0 0 32 32" aria-hidden="true">
          <path d="M16 3 29 10.5v11L16 29 3 21.5v-11z" fill="#5fbf4e" />
          <path d="M16 3 29 10.5 16 18 3 10.5z" fill="#85d977" />
          <path d="M16 18v11L3 21.5v-11z" fill="#3e8c33" />
        </svg>
        <div>
          <h1>Flight Lab</h1>
          <div className="sub">flying-machine evolution · slime + piston + observer</div>
        </div>
        <div className="spacer" />
        <button className="icon-btn" onClick={toggle} aria-label="Toggle color theme">
          light / dark
        </button>
      </header>

      <div className="layout">
        <aside className="panel">
          <div className="panel-head">
            <h2 className="eyebrow">Run config</h2>
          </div>
          <RunConfigForm running={!!running} onStart={start} onStop={stop} error={error} />
        </aside>

        <main className="main-col">
          <section className="stats" aria-label="Run status">
            <div className="stat">
              <div className="k">Generation</div>
              <div className="v">
                {run ? run.generation : "—"}
                <span className="unit">/ {run ? totalGens : "—"}</span>
              </div>
            </div>
            <div className="stat">
              <div className="k">Best fitness</div>
              <div className="v">
                {run && run.history.length > 0
                  ? run.history[run.history.length - 1].best.toFixed(1)
                  : "—"}
                <span className="unit">blocks</span>
              </div>
            </div>
            <div className="stat">
              <div className="k">Throughput</div>
              <div className="v">
                {run && running ? run.evals_per_sec : "—"}
                <span className="unit">evals/s</span>
              </div>
            </div>
            <div className="stat">
              <div className="k">Status</div>
              <div className="status-pill">
                <span
                  className={`status-dot ${run ? run.status : ""}`}
                  aria-hidden="true"
                />
                {run ? run.status : "idle"}
              </div>
            </div>
          </section>

          <section className="panel">
            <div className="panel-head">
              <h2 className="eyebrow">Fitness over generations</h2>
              <div className="chart-legend" role="list" aria-label="Series">
                <span className="item" role="listitem">
                  <span className="swatch" style={{ background: "var(--series-best)" }} />
                  best
                </span>
                <span className="item" role="listitem">
                  <span className="swatch" style={{ background: "var(--series-mean)" }} />
                  mean
                </span>
              </div>
            </div>
            <FitnessChart history={run?.history ?? []} totalGenerations={totalGens} />
          </section>

          <div className="duo">
            <section className="panel">
              <div className="panel-head">
                <h2 className="eyebrow">Leaderboard</h2>
                <span className="note">top machines by distance flown</span>
              </div>
              <Leaderboard
                entries={run?.leaderboard ?? []}
                selectedId={selectedId}
                onSelect={(id) => {
                  userPicked.current = true;
                  setSelectedId(id);
                }}
              />
            </section>

            <section className="panel">
              <div className="panel-head">
                <h2 className="eyebrow">Machine viewer</h2>
                {selected && <span className="note">{selected.name ?? selected.id}</span>}
              </div>
              <MachineViewer machine={selected} evalTicks={config?.eval_ticks ?? null} />
            </section>
          </div>
        </main>
      </div>
    </div>
  );
}
