import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import EventsFeed from "./components/EventsFeed";
import FitnessChart from "./components/FitnessChart";
import Filmstrip from "./components/Filmstrip";
import FlightLoop from "./components/FlightLoop";
import HallOfFame from "./components/HallOfFame";
import Leaderboard from "./components/Leaderboard";
import MachineViewer from "./components/MachineViewer";
import ParetoChart from "./components/ParetoChart";
import PopulationInspector from "./components/PopulationInspector";
import RunConfigForm from "./components/RunConfigForm";
import RunHistory from "./components/RunHistory";
import LineageView from "./components/LineageView";
import {
  GaRunner,
  type GenerationUpdate,
  type LiveConfigPatch,
} from "./ga/runner";
import type { BBox } from "./ga/genome";
import { clearHof, considerForHof, loadHof, type Hof, type HofEntry } from "./hof";
import {
  dirOf,
  speedOf,
  withConfigDefaults,
  OBJECTIVES,
  type EvalMetrics,
  type ObjectiveChoice,
} from "./metrics";
import { ReplayClient } from "./replay/replayClient";
import { deleteRun, listRuns, loadRun, saveRun, type RunSummary } from "./storage";
import type {
  BestRecord,
  EvolutionEvent,
  HistoryPoint,
  LeaderboardEntry,
  PopulationMember,
  RatePoint,
  RetiredEntry,
  RunConfig,
  RunRecord,
  RunStatus,
  SpeciesInfo,
  SpeciesPoint,
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
const EVENTS_CAP = 120;
/** UI reflection cadence during a run. Rendering every generation put the
 *  chart/inspector reconcile on the same thread the runner schedules from
 *  and measurably throttled evolution; ~3 Hz is visually indistinguishable. */
const UI_FLUSH_MS = 300;

interface StagedSim {
  bbox: BBox;
  evalTicks: number;
  seed: number;
}

export default function App() {
  const { toggle } = useTheme();

  const runnerRef = useRef<GaRunner | null>(null);
  const replayRef = useRef<ReplayClient | null>(null);
  const runner = () => (runnerRef.current ??= new GaRunner());
  const replayer = () => (replayRef.current ??= new ReplayClient());

  const [status, setStatus] = useState<RunStatus>("idle");
  // Config drawer: open on arrival (configuring is the first task), closed
  // automatically when a run launches, reopenable any time — live controls
  // inside keep working mid-run.
  const [configOpen, setConfigOpen] = useState(true);
  const prevStatusRef = useRef<RunStatus>("idle");
  useEffect(() => {
    if (prevStatusRef.current === "idle" && status === "running") {
      setConfigOpen(false);
    }
    prevStatusRef.current = status;
  }, [status]);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setConfigOpen(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  const [config, setConfig] = useState<RunConfig | null>(null);
  const [history, setHistory] = useState<HistoryPoint[]>([]);
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
  const [bests, setBests] = useState<BestRecord[]>([]);
  const [archive, setArchive] = useState<LeaderboardEntry[]>([]);
  const [cloud, setCloud] = useState<EvalMetrics[]>([]);
  const [events, setEvents] = useState<EvolutionEvent[]>([]);
  const [retired, setRetired] = useState<RetiredEntry[]>([]);
  const [speciesInfo, setSpeciesInfo] = useState<SpeciesInfo[]>([]);
  const [speciesPoints, setSpeciesPoints] = useState<SpeciesPoint[]>([]);
  const [rates, setRates] = useState<RatePoint[]>([]);
  const [population, setPopulation] = useState<PopulationMember[]>([]);
  /** A population-inspector pick shown in the machine viewer (takes
   * precedence over the leaderboard/archive selection). */
  const [inspected, setInspected] = useState<LeaderboardEntry | null>(null);
  const [gen, setGen] = useState(0);
  const [eps, setEps] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [selectedLbId, setSelectedLbId] = useState<string | null>(null);
  const [filmGen, setFilmGen] = useState<number | null>(null);
  const [replayLoading, setReplayLoading] = useState(false);
  const [runsIndex, setRunsIndex] = useState<RunSummary[]>(() => listRuns());
  const [browsing, setBrowsing] = useState<RunRecord | null>(null);
  const [viewTab, setViewTab] = useState<
    "lab" | "hof" | "lineage" | "population"
  >("lab");
  const [hof, setHof] = useState<Hof>(() => loadHof());
  /** Machine-viewer follows the leaderboard leader until the user picks a
   * machine or grabs the orbit controls. */
  const [followLeader, setFollowLeader] = useState(true);
  /** A machine pinned onto the flight stage from the Pareto front or the
   * Hall of Fame, overriding the champion / filmstrip pick. */
  const [staged, setStaged] = useState<BestRecord | null>(null);
  const [stagedSim, setStagedSim] = useState<StagedSim | null>(null);
  const hofRef = useRef(hof);
  hofRef.current = hof;

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

  /** Replay for a pinned (Pareto / Hall of Fame) machine. */
  const requestStagedReplay = useCallback(
    (rec: BestRecord, sim: StagedSim) => {
      setReplayLoading(true);
      replayer()
        .replay(rec.genome, sim.bbox, sim.evalTicks, sim.seed)
        .then((loop) => {
          setStaged((p) => (p && p.gen === rec.gen ? { ...p, loop } : p));
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
      setArchive([]);
      setCloud([]);
      setEvents([]);
      setRetired([]);
      setSpeciesInfo([]);
      setSpeciesPoints([]);
      setRates([]);
      setPopulation([]);
      setInspected(null);
      setGen(0);
      setEps(0);
      setFilmGen(null);
      setSelectedLbId(null);
      setStaged(null);
      setStagedSim(null);
      setViewTab("lab");
      setFollowLeader(true);
      setStatus("starting");

      const rec: RunRecord = {
        id: `run-${Date.now().toString(36)}`,
        startedAt: Date.now(),
        config: cfg,
        history: [],
        leaderboard: [],
        bests: [],
        archive: [],
        events: [],
        retired: [],
        species: { info: [], points: [] },
        rates: [],
        generation: 0,
      };
      recRef.current = rec;

      /** Append an app-side event (pause/resume) to feed + record. */
      const pushEvent = (e: EvolutionEvent) => {
        rec.events = [...(rec.events ?? []), e].slice(-EVENTS_CAP);
        setEvents((es) => [...es, e].slice(-EVENTS_CAP));
      };

      // Generation results land in `rec` immediately (exact, push-based);
      // React only sees them on a throttled flush so the main thread stays
      // free for the runner between generations.
      let latestUpdate: GenerationUpdate | null = null;
      let flushTimer: number | null = null;
      const flushUi = () => {
        if (flushTimer != null) {
          window.clearTimeout(flushTimer);
          flushTimer = null;
        }
        const v = latestUpdate;
        if (!v) return;
        setGen(v.gen);
        setEps(v.evalsPerSec);
        setHistory(rec.history.slice());
        setLeaderboard(v.leaderboard);
        setArchive(v.archive);
        setCloud(v.cloud);
        setRetired(v.retired);
        setSpeciesInfo(rec.species!.info);
        setSpeciesPoints(rec.species!.points.slice());
        setRates(rec.rates!.slice());
        setPopulation(v.population);
        setEvents((rec.events ?? []).slice());
        setBests(rec.bests.slice());
      };

      runner().start(cfg, {
        onPause: (g) => {
          flushUi();
          setStatus("paused");
          pushEvent({
            gen: g,
            kind: "pause",
            at: Date.now(),
            text: `paused @ gen ${g} — state held, live controls stay active`,
          });
          persist(true);
        },
        onResume: (g) => {
          setStatus("running");
          pushEvent({
            gen: g,
            kind: "pause",
            at: Date.now(),
            text: `resumed @ gen ${g}`,
          });
        },
        onGeneration: (u) => {
          setStatus((s) => (s === "paused" ? s : "running"));
          // Exact record keeping, all push-based: the previous per-generation
          // array spreads went quadratic over long runs (gen 2000 copied
          // multi-thousand-element arrays every generation).
          rec.generation = u.gen;
          rec.history.push(u.point);
          rec.leaderboard = u.leaderboard;
          rec.archive = u.archive;
          rec.retired = u.retired;
          // Both are initialised in the record literal above; the record type
          // keeps them optional for old stored runs.
          rec.species!.info = u.species.info;
          rec.species!.points.push({ gen: u.gen, counts: u.species.counts });
          rec.rates!.push({ gen: u.gen, rate: u.mutationRate });
          if (u.events.length > 0) {
            rec.events = [...(rec.events ?? []), ...u.events].slice(-EVENTS_CAP);
          }
          latestUpdate = u;
          // Challenge the Hall of Fame with everything notable this gen —
          // including the run's newest slowpoke, which may sit far below
          // the leaderboard when the run is chasing speed.
          const cands = [
            ...u.leaderboard,
            ...u.archive,
            ...(u.slowpoke ? [u.slowpoke] : []),
          ];
          if (cands.length > 0) {
            const res = considerForHof(hofRef.current, cands, rec.id, {
              bbox: cfg.bbox,
              evalTicks: cfg.eval_ticks,
              seed: cfg.seed,
            });
            if (res.changed) setHof(res.hof);
          }
          if (u.newBest) {
            rec.bests.push(u.newBest);
            // Test introspection (verify scripts): live champion list —
            // survives the localStorage quota fallback dropping the run.
            (window as unknown as { __fgaBests?: unknown }).__fgaBests =
              rec.bests;
            requestReplay(u.newBest, cfg);
            persist(true);
            flushUi(); // a new champion is worth an immediate frame
          } else {
            if (flushTimer == null) {
              flushTimer = window.setTimeout(flushUi, UI_FLUSH_MS);
            }
            persist();
          }
        },
        onDone: () => {
          flushUi();
          rec.stoppedAt = Date.now();
          setStatus("done");
          persist(true);
        },
        onError: (message) => {
          flushUi();
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

  const pause = useCallback(() => {
    runner().pause();
  }, []);

  const resume = useCallback(() => {
    runner().resume();
  }, []);

  /** Mid-run regime change: applied by the runner at the next generation
   * boundary; the config mirror updates so charts/labels follow. */
  const reconfigure = useCallback((patch: LiveConfigPatch) => {
    runner().reconfigure(patch);
    setConfig((c) =>
      c
        ? {
            ...c,
            objectives: patch.objectives,
            constraints: patch.constraints,
            targetPeriod: patch.targetPeriod,
            mutation_rate: patch.mutationRate,
            mutationSchedule: patch.mutationSchedule,
          }
        : c,
    );
    const rec = recRef.current;
    if (rec)
      rec.config = {
        ...rec.config,
        objectives: patch.objectives,
        constraints: patch.constraints,
        targetPeriod: patch.targetPeriod,
        mutation_rate: patch.mutationRate,
        mutationSchedule: patch.mutationSchedule,
      };
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
      setStaged(null);
      setStagedSim(null);
      setInspected(null);
      setViewTab("lab");
      setFollowLeader(true);
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
        archive: browsing.archive ?? [],
        cloud: [] as EvalMetrics[],
        events: browsing.events ?? [],
        retired: browsing.retired ?? [],
        speciesInfo: browsing.species?.info ?? [],
        speciesPoints: browsing.species?.points ?? [],
        rates: browsing.rates ?? [],
        population: [] as PopulationMember[],
        gen: browsing.generation,
        config: browsing.config,
        eps: null as number | null,
        status: "archived" as const,
      };
    return {
      history,
      leaderboard,
      bests,
      archive,
      cloud,
      events,
      retired,
      speciesInfo,
      speciesPoints,
      rates,
      population,
      gen,
      config,
      eps,
      status,
    };
  }, [browsing, history, leaderboard, bests, archive, cloud, events, retired, speciesInfo, speciesPoints, rates, population, gen, config, eps, status]);

  const viewCfg = useMemo(
    () => (view.config ? withConfigDefaults(view.config) : null),
    [view.config],
  );

  // Follow the leader until the user picks a machine manually or grabs the
  // viewer's orbit controls (either disengages follow; the chip re-enables).
  useEffect(() => {
    const lb = view.leaderboard;
    if (lb.length === 0) return;
    if (followLeader) setSelectedLbId(lb[0].id);
    else if (
      !lb.some((m) => m.id === selectedLbId) &&
      !view.archive.some((m) => m.id === selectedLbId)
    )
      setSelectedLbId(lb[0].id);
  }, [view.leaderboard, view.archive, selectedLbId, followLeader]);

  const selectedMachine =
    inspected ??
    view.leaderboard.find((m) => m.id === selectedLbId) ??
    view.archive.find((m) => m.id === selectedLbId) ??
    null;

  /** Population-inspector click-through: stage the member in the viewer. */
  const inspectMember = useCallback(
    (m: PopulationMember) => {
      setInspected({
        id: m.id,
        name: `gen ${view.gen} · #${m.id.split("-")[1] ?? "?"}`,
        fitness: m.fitness,
        gen: view.gen,
        genome: m.genome,
        blocks: m.blocks,
        speed: m.speed,
        metrics: m.metrics,
      });
      setFollowLeader(false);
      setViewTab("lab");
    },
    [view.gen],
  );

  const stageRecord: BestRecord | null = useMemo(() => {
    if (staged) return staged;
    const bs = view.bests;
    if (bs.length === 0) return null;
    if (filmGen !== null) {
      const hit = bs.find((b) => b.gen === filmGen);
      if (hit) return hit;
    }
    return bs[bs.length - 1];
  }, [staged, view.bests, filmGen]);

  const isChampionOnStage =
    staged === null &&
    stageRecord !== null &&
    view.bests.length > 0 &&
    stageRecord.gen === view.bests[view.bests.length - 1].gen;

  // Lazily re-simulate a stage pick that has no stored loop.
  useEffect(() => {
    if (!stageRecord || stageRecord.loop || replayLoading) return;
    if (staged && stageRecord === staged) {
      const sim =
        stagedSim ??
        (viewCfg
          ? { bbox: viewCfg.bbox, evalTicks: viewCfg.eval_ticks, seed: viewCfg.seed }
          : null);
      if (sim) requestStagedReplay(staged, sim);
      return;
    }
    const cfg = view.config;
    if (!cfg) return;
    // Live champions get their replay queued on discovery; this covers
    // filmstrip picks and archived runs.
    requestReplay(stageRecord, cfg);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [stageRecord?.gen, stageRecord?.loop, view.config, staged, stagedSim]);

  /** Click on a Pareto front point: inspect + fly that machine. */
  const pickParetoPoint = useCallback(
    (id: string) => {
      const entry = view.archive.find((e) => e.id === id);
      if (!entry) return;
      setFollowLeader(false);
      setInspected(null);
      setSelectedLbId(id);
      setFilmGen(null);
      setStaged({
        gen: entry.gen,
        fitness: entry.fitness,
        genome: entry.genome,
        blocks: entry.blocks,
      });
      setStagedSim(
        viewCfg
          ? { bbox: viewCfg.bbox, evalTicks: viewCfg.eval_ticks, seed: viewCfg.seed }
          : null,
      );
    },
    [view.archive, viewCfg],
  );

  /** Click a Hall of Fame plinth: fly the inductee on the lab stage. */
  const stageHofEntry = useCallback((e: HofEntry) => {
    setViewTab("lab");
    setFilmGen(null);
    setStaged({
      gen: e.gen,
      fitness: e.metrics.fit,
      genome: e.genome,
      blocks: e.blocks,
    });
    setStagedSim({ bbox: e.bbox, evalTicks: e.evalTicks, seed: e.seed });
  }, []);

  const clearHall = useCallback(() => {
    clearHof();
    setHof({});
  }, []);

  const running =
    status === "running" || status === "starting" || status === "paused";
  const isPaused = status === "paused";
  const lastBest =
    view.history.length > 0 ? view.history[view.history.length - 1].best : null;
  const bestOverall =
    view.history.length > 0
      ? Math.max(...view.history.map((h) => h.best))
      : null;
  const target = viewCfg?.generations ?? null;
  const evalTicks = viewCfg?.eval_ticks ?? null;
  const stageTicks = staged && stagedSim ? stagedSim.evalTicks : evalTicks;

  const paretoSel: ObjectiveChoice[] = viewCfg?.objectives ?? [];
  const paretoMode = viewCfg?.mode === "pareto";

  return (
    <div className="shell">
      <header className="topbar">
        <svg className="logo" width="26" height="26" viewBox="0 0 32 32" aria-hidden="true">
          <path d="M16 3 29 10.5v11L16 29 3 21.5v-11z" fill="#5fbf4e" />
          <path d="M16 3 29 10.5 16 18 3 10.5z" fill="#85d977" />
          <path d="M16 18v11L3 21.5v-11z" fill="#3e8c33" />
        </svg>
        <div>
          <h1>Flight Evolution</h1>
        
        </div>
        <nav className="tabs" aria-label="Views">
          <button
            className={viewTab === "lab" ? "on" : ""}
            aria-current={viewTab === "lab" ? "page" : undefined}
            onClick={() => setViewTab("lab")}
          >
            Lab
          </button>
          <button
            className={viewTab === "population" ? "on" : ""}
            aria-current={viewTab === "population" ? "page" : undefined}
            onClick={() => setViewTab("population")}
            data-testid="tab-population"
          >
            Population
          </button>
          <button
            className={viewTab === "lineage" ? "on" : ""}
            aria-current={viewTab === "lineage" ? "page" : undefined}
            onClick={() => setViewTab("lineage")}
            data-testid="tab-lineage"
          >
            Lineage
          </button>
          <button
            className={viewTab === "hof" ? "on" : ""}
            aria-current={viewTab === "hof" ? "page" : undefined}
            onClick={() => setViewTab("hof")}
            data-testid="tab-hof"
          >
            Hall of Fame
          </button>
        </nav>
        <div className="spacer" />
        <button
          className="icon-btn"
          onClick={() => setConfigOpen((o) => !o)}
          aria-expanded={configOpen}
          data-testid="config-drawer-toggle"
        >
          ⚙ config
        </button>
        <button className="icon-btn" onClick={toggle} aria-label="Toggle color theme">
          light / dark
        </button>
      </header>

      <div className="layout">
        {configOpen && (
          <button
            className="drawer-backdrop"
            aria-label="Close run config"
            onClick={() => setConfigOpen(false)}
          />
        )}
        <aside className={configOpen ? "side-col open" : "side-col"}>
          <div className="drawer-head">
            <h2 className="eyebrow">Run config</h2>
            <button className="icon-btn" onClick={() => setConfigOpen(false)}>
              close ✕
            </button>
          </div>
          <section className="panel">
            <RunConfigForm
              running={running}
              paused={isPaused}
              onStart={start}
              onStop={stop}
              onPause={pause}
              onResume={resume}
              onReconfigure={reconfigure}
              rates={rates}
              error={error}
            />
          </section>

          <section className="panel">
            <div className="panel-head">
              <h2 className="eyebrow">Evolution events</h2>
            </div>
            <EventsFeed events={view.events} />
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

        {viewTab === "hof" ? (
          <main className="main-col">
            <section className="panel">
              <div className="panel-head">
                <h2 className="eyebrow">Hall of Fame</h2>
                <span className="note">the finest and strangest fliers, across every run</span>
              </div>
              <HallOfFame hof={hof} onClear={clearHall} onStage={stageHofEntry} />
            </section>
          </main>
        ) : viewTab === "population" ? (
          <main className="main-col">
            <section className="panel" data-testid="population-panel">
              <div className="panel-head">
                <h2 className="eyebrow">Population</h2>
                <span className="note">
                  the entire current generation — every genome, valid or culled
                </span>
              </div>
              <PopulationInspector
                members={view.population}
                species={view.speciesInfo}
                gen={view.gen}
                paused={isPaused}
                onPick={inspectMember}
              />
            </section>
          </main>
        ) : viewTab === "lineage" ? (
          <main className="main-col">
            <section className="panel" data-testid="lineage-panel">
              <div className="panel-head">
                <h2 className="eyebrow">Lineage</h2>
                <span className="note">
                  species share per generation — births ▲ and extinctions ✕ of
                  structural families
                </span>
              </div>
              <LineageView
                info={view.speciesInfo}
                points={view.speciesPoints}
                currentGen={view.gen}
              />
            </section>
          </main>
        ) : (
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
              <div className="k">Best speed</div>
              <div className="v" data-testid="stat-speed">
                {bestOverall !== null && evalTicks
                  ? speedOf(bestOverall, evalTicks).toFixed(2)
                  : "—"}
                <span className="unit">blk/s</span>
              </div>
            </div>
            <div className="stat">
              <div className="k">Best distance</div>
              <div className="v" data-testid="stat-best">
                {lastBest !== null ? bestOverall!.toFixed(1) : "—"}
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
            <FitnessChart
              history={view.history}
              targetGenerations={target}
              evalTicks={evalTicks}
            />
          </section>

          {paretoMode && (
            <section className="panel" data-testid="pareto-panel">
              <div className="panel-head">
                <h2 className="eyebrow">
                  Pareto front
                  {paretoSel.length >= 2 &&
                    ` — ${OBJECTIVES[paretoSel[0].key].label} vs ${OBJECTIVES[paretoSel[1].key].label}`}
                </h2>
                <span className="note">
                  {view.archive.length} non-dominated machine
                  {view.archive.length === 1 ? "" : "s"} archived across all
                  generations
                </span>
              </div>
              {paretoSel.length === 2 ? (
                <ParetoChart
                  archive={view.archive}
                  cloud={view.cloud}
                  choices={[paretoSel[0], paretoSel[1]]}
                  selectedId={selectedLbId}
                  onPick={pickParetoPoint}
                />
              ) : (
                <div className="chart-empty">
                  Select exactly 2 objectives to plot the front — the archive is
                  still collecting non-dominated machines on{" "}
                  {paretoSel
                    .map(
                      (c) =>
                        `${dirOf(c) === "min" ? "min " : ""}${OBJECTIVES[c.key].label}`,
                    )
                    .join(" + ")}.
                </div>
              )}
              {view.retired.length > 0 && (
                <div className="retired-shelf" data-testid="retired-shelf">
                  <div className="retired-head">
                    Retired — pushed off the front by a mid-run rule change
                  </div>
                  <ul>
                    {view.retired.map((r, i) => (
                      <li key={`${r.entry.id}-${i}`} data-testid="retired-entry">
                        <b>{r.entry.name ?? r.entry.id}</b>
                        <span className="retired-meta">
                          {(r.entry.speed ?? 0).toFixed(2)} blk/s ·{" "}
                          {r.entry.blocks.length} blocks
                        </span>
                        <span className="retired-reason">{r.reason}</span>
                        <span className="retired-gen">@ gen {r.atGen}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </section>
          )}

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
              evalTicks={stageTicks}
              onInteract={() => {
                // Grabbing the stage pins the current machine: a new
                // champion no longer swaps it out from under the drag.
                if (staged === null && filmGen === null && stageRecord)
                  setFilmGen(stageRecord.gen);
              }}
              onResumeFollow={() => {
                setStaged(null);
                setStagedSim(null);
                setFilmGen(null);
              }}
            />
            <div className="panel-head film-head">
              <h2 className="eyebrow">Champions</h2>
              <span className="note">every generation that set a new best</span>
            </div>
            <Filmstrip
              bests={view.bests}
              selectedGen={filmGen}
              onSelect={(g) => {
                setStaged(null);
                setStagedSim(null);
                setFilmGen(g);
              }}
            />
          </section>

          <div className="duo">
            <section className="panel">
              <div className="panel-head">
                <h2 className="eyebrow">Leaderboard</h2>
                <span className="note">
                  {paretoMode ? "top machines by speed" : "top machines by score"}
                </span>
              </div>
              <Leaderboard
                entries={view.leaderboard}
                selectedId={selectedLbId}
                evalTicks={evalTicks}
                onSelect={(id) => {
                  setFollowLeader(false);
                  setInspected(null);
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
                evalTicks={evalTicks}
                following={followLeader}
                onInteract={() => setFollowLeader(false)}
                onResumeFollow={() => {
                  setFollowLeader(true);
                  setInspected(null);
                  if (view.leaderboard.length > 0)
                    setSelectedLbId(view.leaderboard[0].id);
                }}
              />
            </section>
          </div>
        </main>
        )}
      </div>
    </div>
  );
}
