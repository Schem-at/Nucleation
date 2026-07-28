import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import FitnessChart from "./components/FitnessChart";
import Filmstrip from "./components/Filmstrip";
import FlightLoop from "./components/FlightLoop";
import Leaderboard from "./components/Leaderboard";
import MachineViewer from "./components/MachineViewer";
import RunConfigForm from "./components/RunConfigForm";
import RunHistory from "./components/RunHistory";
import { GaRunner } from "./ga/runner";
import { ReplayClient } from "./replay/replayClient";
import { deleteRun, listRuns, loadRun, saveRun, type RunSummary } from "./storage";
import type {
  BestRecord,
  HistoryPoint,
  LeaderboardEntry,
  RunConfig,
  RunRecord,
  RunStatus,
} from "./types";

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
    localStorage.setItem("fgaw-theme", next);
    setTheme(next);
  };
  return { toggle };
}

const SAVE_EVERY_MS = 4000;

export default function App() {
  const { toggle } = useTheme();

  const runnerRef = useRef<GaRunner | null>(null);
  const replayRef = useRef<ReplayClient | null>(null);
  const runner = () => (runnerRef.current ??= new GaRunner());
  const replayer = () => (replayRef.current ??= new ReplayClient());

  const [status, setStatus] = useState<RunStatus>("idle");
  const [config, setConfig] = useState<RunConfig | null>(null);
  const [history, setHistory] = useState<HistoryPoint[]>([]);
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
  const [bests, setBests] = useState<BestRecord[]>([]);
  const [gen, setGen] = useState(0);
  const [eps, setEps] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [selectedLbId, setSelectedLbId] = useState<string | null>(null);
  const [filmGen, setFilmGen] = useState<number | null>(null);
  const [replayLoading, setReplayLoading] = useState(false);
  const [runsIndex, setRunsIndex] = useState<RunSummary[]>(() => listRuns());
  const [browsing, setBrowsing] = useState<RunRecord | null>(null);
  const userPicked = useRef(false);

  // Authoritative run accumulator (callbacks mutate this, then mirror to state).
  const recRef = useRef<RunRecord | null>(null);
  const lastSaveRef = useRef(0);

  const persist = useCallback((force = false) => {
    const rec = recRef.current;
    if (!rec) return;
    const now = Date.now();
    if (!force && now - lastSaveRef.current < SAVE_EVERY_MS) return;
    lastSaveRef.current = now;
    saveRun(rec);
    setRunsIndex(listRuns());
  }, []);

  useEffect(() => {
    const onUnload = () => persist(true);
    window.addEventListener("beforeunload", onUnload);
    return () => window.removeEventListener("beforeunload", onUnload);
  }, [persist]);

  const requestReplay = useCallback(
    (rec: BestRecord, cfg: RunConfig) => {
      setReplayLoading(true);
      replayer()
        .replay(rec.genome, cfg.bbox, cfg.eval_ticks, cfg.seed)
        .then((loop) => {
          const r = recRef.current;
          if (r) {
            const hit = r.bests.find((b) => b.gen === rec.gen);
            if (hit) hit.loop = loop;
          }
          setBests((bs) =>
            bs.map((b) => (b.gen === rec.gen ? { ...b, loop } : b)),
          );
          setBrowsing((br) =>
            br
              ? {
                  ...br,
                  bests: br.bests.map((b) =>
                    b.gen === rec.gen ? { ...b, loop } : b,
                  ),
                }
              : br,
          );
        })
        .catch(() => undefined)
        .finally(() => setReplayLoading(false));
    },
    [],
  );

  const start = useCallback(
    (cfg: RunConfig) => {
      setError(null);
      setBrowsing(null);
      setConfig(cfg);
      setHistory([]);
      setLeaderboard([]);
      setBests([]);
      setGen(0);
      setEps(0);
      setFilmGen(null);
      setSelectedLbId(null);
      userPicked.current = false;
      setStatus("starting");

      const rec: RunRecord = {
        id: `run-${Date.now().toString(36)}`,
        startedAt: Date.now(),
        config: cfg,
        history: [],
        leaderboard: [],
        bests: [],
        generation: 0,
      };
      recRef.current = rec;

      runner().start(cfg, {
        onGeneration: (u) => {
          setStatus("running");
          rec.generation = u.gen;
          rec.history.push(u.point);
          rec.leaderboard = u.leaderboard;
          setGen(u.gen);
          setEps(u.evalsPerSec);
          setHistory((h) => [...h, u.point]);
          setLeaderboard(u.leaderboard);
          if (u.newBest) {
            rec.bests.push(u.newBest);
            setBests((bs) => [...bs, u.newBest!]);
            requestReplay(u.newBest, cfg);
            persist(true);
          } else {
            persist();
          }
        },
        onDone: () => {
          rec.stoppedAt = Date.now();
          setStatus("done");
          persist(true);
        },
        onError: (message) => {
          setError(message);
          rec.stoppedAt = Date.now();
          setStatus("done");
          persist(true);
        },
      });
    },
    [persist, requestReplay],
  );

  const stop = useCallback(() => {
    runner().stop();
  }, []);

  useEffect(
    () => () => {
      runnerRef.current?.dispose();
      replayRef.current?.dispose();
    },
    [],
  );

  // Browse a stored run.
  const openRun = useCallback((id: string) => {
    const rec = loadRun(id);
    if (rec) {
      setBrowsing(rec);
      setFilmGen(null);
      setSelectedLbId(null);
      userPicked.current = false;
    }
  }, []);

  const removeRun = useCallback((id: string) => {
    deleteRun(id);
    setRunsIndex(listRuns());
    setBrowsing((b) => (b?.id === id ? null : b));
  }, []);

  // ---- view model: live run or the browsed record --------------------------
  const view = useMemo(() => {
    if (browsing)
      return {
        history: browsing.history,
        leaderboard: browsing.leaderboard,
        bests: browsing.bests,
        gen: browsing.generation,
        config: browsing.config,
        eps: null as number | null,
        status: "archived" as const,
      };
    return {
      history,
      leaderboard,
      bests,
      gen,
      config,
      eps,
      status,
    };
  }, [browsing, history, leaderboard, bests, gen, config, eps, status]);

  // Follow the leader until the user picks a machine manually.
  useEffect(() => {
    const lb = view.leaderboard;
    if (lb.length === 0) return;
    if (!userPicked.current) setSelectedLbId(lb[0].id);
    else if (!lb.some((m) => m.id === selectedLbId)) setSelectedLbId(lb[0].id);
  }, [view.leaderboard, selectedLbId]);

  const selectedMachine =
    view.leaderboard.find((m) => m.id === selectedLbId) ?? null;

  const stageRecord: BestRecord | null = useMemo(() => {
    const bs = view.bests;
    if (bs.length === 0) return null;
    if (filmGen !== null) {
      const hit = bs.find((b) => b.gen === filmGen);
      if (hit) return hit;
    }
    return bs[bs.length - 1];
  }, [view.bests, filmGen]);

  const isChampionOnStage =
    stageRecord !== null &&
    view.bests.length > 0 &&
    stageRecord.gen === view.bests[view.bests.length - 1].gen;

  // Lazily re-simulate a filmstrip pick that has no stored loop.
  useEffect(() => {
    if (!stageRecord || stageRecord.loop || replayLoading) return;
    const cfg = view.config;
    if (!cfg) return;
    // Live champions get their replay queued on discovery; this covers
    // filmstrip picks and archived runs.
    requestReplay(stageRecord, cfg);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [stageRecord?.gen, stageRecord?.loop, view.config]);

  const running = status === "running" || status === "starting";
  const lastBest =
    view.history.length > 0 ? view.history[view.history.length - 1].best : null;
  const target = view.config?.generations ?? null;

  return (
    <div className="shell">
      <header className="topbar">
        <svg className="logo" width="26" height="26" viewBox="0 0 32 32" aria-hidden="true">
          <path d="M16 3 29 10.5v11L16 29 3 21.5v-11z" fill="#5fbf4e" />
          <path d="M16 3 29 10.5 16 18 3 10.5z" fill="#85d977" />
          <path d="M16 18v11L3 21.5v-11z" fill="#3e8c33" />
        </svg>
        <div>
          <h1>Flight Lab · WASM</h1>
          <div className="sub">
            flying-machine evolution · engine + GA entirely in your browser
          </div>
        </div>
        <div className="spacer" />
        <button className="icon-btn" onClick={toggle} aria-label="Toggle color theme">
          light / dark
        </button>
      </header>

      <div className="layout">
        <aside className="side-col">
          <section className="panel">
            <div className="panel-head">
              <h2 className="eyebrow">Run config</h2>
            </div>
            <RunConfigForm running={running} onStart={start} onStop={stop} error={error} />
          </section>

          <section className="panel">
            <div className="panel-head">
              <h2 className="eyebrow">Past runs</h2>
            </div>
            <RunHistory
              runs={runsIndex}
              activeId={running && recRef.current ? recRef.current.id : null}
              browsingId={browsing?.id ?? null}
              onOpen={openRun}
              onDelete={removeRun}
            />
          </section>
        </aside>

        <main className="main-col">
          {browsing && (
            <div className="browse-banner">
              <span>
                Browsing an archived run from{" "}
                {new Date(browsing.startedAt).toLocaleString()}
              </span>
              <button className="icon-btn" onClick={() => setBrowsing(null)}>
                back to session
              </button>
            </div>
          )}

          <section className="stats" aria-label="Run status">
            <div className="stat">
              <div className="k">Generation</div>
              <div className="v" data-testid="stat-generation">
                {view.history.length ? view.gen : "—"}
                <span className="unit">/ {target ?? "∞"}</span>
              </div>
            </div>
            <div className="stat">
              <div className="k">Best fitness</div>
              <div className="v" data-testid="stat-best">
                {lastBest !== null
                  ? Math.max(...view.history.map((h) => h.best)).toFixed(1)
                  : "—"}
                <span className="unit">blocks</span>
              </div>
            </div>
            <div className="stat">
              <div className="k">Throughput</div>
              <div className="v" data-testid="stat-eps">
                {view.eps !== null && running ? view.eps : "—"}
                <span className="unit">evals/s</span>
              </div>
            </div>
            <div className="stat">
              <div className="k">Status</div>
              <div className="status-pill" data-testid="stat-status">
                <span
                  className={`status-dot ${view.status === "archived" ? "done" : view.status}`}
                  aria-hidden="true"
                />
                {view.status}
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
            <FitnessChart history={view.history} targetGenerations={target} />
          </section>

          <section className="panel" data-testid="flight-stage">
            <div className="panel-head">
              <h2 className="eyebrow">Flight stage</h2>
              <span className="note">
                one period, translate-compensated — the corridor scrolls, the machine flies in place
              </span>
            </div>
            <FlightLoop
              best={stageRecord}
              isChampion={isChampionOnStage}
              loading={replayLoading && stageRecord !== null && !stageRecord.loop}
            />
            <div className="panel-head film-head">
              <h2 className="eyebrow">Champions</h2>
              <span className="note">every generation that set a new best</span>
            </div>
            <Filmstrip
              bests={view.bests}
              selectedGen={filmGen}
              onSelect={setFilmGen}
            />
          </section>

          <div className="duo">
            <section className="panel">
              <div className="panel-head">
                <h2 className="eyebrow">Leaderboard</h2>
                <span className="note">top machines by distance flown</span>
              </div>
              <Leaderboard
                entries={view.leaderboard}
                selectedId={selectedLbId}
                onSelect={(id) => {
                  userPicked.current = true;
                  setSelectedLbId(id);
                }}
              />
            </section>

            <section className="panel">
              <div className="panel-head">
                <h2 className="eyebrow">Machine viewer</h2>
                {selectedMachine && (
                  <span className="note">{selectedMachine.name ?? selectedMachine.id}</span>
                )}
              </div>
              <MachineViewer
                machine={selectedMachine}
                evalTicks={view.config?.eval_ticks ?? null}
              />
            </section>
          </div>
        </main>
      </div>
    </div>
  );
}
