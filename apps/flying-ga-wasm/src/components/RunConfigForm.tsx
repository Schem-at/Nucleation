/** The config rail, as a flight checklist: five collapsible instrument
 * groups (Run · Genome · Objectives · Constraints · Schedules) plus an
 * Advanced disclosure for the rarely-touched fields. Each collapsed
 * summary is a live readout, so the rail still reads as an instrument
 * panel when folded shut. Live-adjustable groups wear a "live" tick during
 * a run; genome-space fields stay locked with their tooltip. */

import { useEffect, useMemo, useRef, useState } from "react";
import { PLACEABLE_KINDS } from "../ga/alphabet";
import {
  impliedGateBy,
  naturalDir,
  normalizeSpec,
  BEHAVIOURS,
  BEHAVIOUR_ORDER,
  DEFAULT_MAP_ELITES,
  MAP_ELITES_BIN_MAX,
  MAP_ELITES_BIN_MIN,
  OBJECTIVES,
  OBJECTIVE_ORDER,
  QUALITIES,
  QUALITY_ORDER,
  type BehaviourKey,
  type Constraints,
  type MapElitesSpec,
  type ObjectiveChoice,
  type ObjectiveDir,
  type ObjectiveKey,
  type OptimizeMode,
  type QualityKey,
} from "../metrics";
import type { LiveConfigPatch } from "../ga/runner";
import type {
  MutationSchedule,
  MutationScheduleKind,
  RatePoint,
  RunConfig,
  SeedMode,
} from "../types";
import RateSparkline from "./RateSparkline";

interface Props {
  running: boolean;
  paused: boolean;
  onStart: (cfg: RunConfig) => void;
  onStop: () => void;
  onPause: () => void;
  onResume: () => void;
  /** Mid-run regime change (objectives / constraints / mutation regime).
   * Fields that change the genome space stay locked. */
  onReconfigure?: (patch: LiveConfigPatch) => void;
  /** Mutation-rate trajectory of the live run (sparkline + readout). */
  rates: RatePoint[];
  error: string | null;
}

const KIND_LABEL: Record<string, string> = {
  slime_block: "slime",
  honey_block: "honey",
  sticky_piston: "sticky piston",
  piston: "piston",
  observer: "observer",
  note_block: "note block",
  white_concrete: "concrete",
  oak_trapdoor: "trapdoor",
};

const LOCKED_TIP = "locked during a run — this changes the genome space";

/** Weights are kept as raw strings so the field can be cleared/edited freely
 * (the old numeric state rejected intermediate values and the input stuck). */
type ObjState = Record<
  ObjectiveKey,
  { on: boolean; weight: string; dir: ObjectiveDir }
>;

const initialObjectives = (): ObjState =>
  Object.fromEntries(
    OBJECTIVE_ORDER.map((k) => [
      k,
      { on: k === "speed", weight: "1", dir: naturalDir(k) },
    ]),
  ) as ObjState;

const parseWeight = (raw: string): number => {
  const w = parseFloat(raw);
  return Number.isFinite(w) && w >= 0 ? w : 1;
};

const posInt = (raw: string, fallback: number): number => {
  const v = parseInt(raw, 10);
  return Number.isFinite(v) && v > 0 ? v : fallback;
};

type GroupId =
  | "run"
  | "genome"
  | "objectives"
  | "constraints"
  | "schedules"
  | "advanced";

export default function RunConfigForm({
  running,
  paused,
  onStart,
  onStop,
  onPause,
  onResume,
  onReconfigure,
  rates,
  error,
}: Props) {
  const [population, setPopulation] = useState(96);
  /** Blank = run forever until Stop. */
  const [generations, setGenerations] = useState<string>("");
  const [mutationRate, setMutationRate] = useState(0.05);
  const [schedKind, setSchedKind] = useState<MutationScheduleKind>("constant");
  const [halfLife, setHalfLife] = useState("30");
  const [spanGens, setSpanGens] = useState("120");
  const [bx, setBx] = useState(5);
  const [by, setBy] = useState(3);
  const [bz, setBz] = useState(3);
  const [evalTicks, setEvalTicks] = useState(600);
  const [seed, setSeed] = useState(42);
  const [workers, setWorkers] = useState(4);

  const [mode, setMode] = useState<OptimizeMode>("scalar");
  const [beX, setBeX] = useState<BehaviourKey>(DEFAULT_MAP_ELITES.x);
  const [beY, setBeY] = useState<BehaviourKey>(DEFAULT_MAP_ELITES.y);
  const [binsX, setBinsX] = useState(DEFAULT_MAP_ELITES.binsX);
  const [binsY, setBinsY] = useState(DEFAULT_MAP_ELITES.binsY);
  const [quality, setQuality] = useState<QualityKey>(DEFAULT_MAP_ELITES.quality);
  const [seeding, setSeeding] = useState<SeedMode>("minimal");
  const [objs, setObjs] = useState<ObjState>(initialObjectives);
  const [maxBlocks, setMaxBlocks] = useState(14);
  const [minBlocks, setMinBlocks] = useState(1);
  const [pistonBudget, setPistonBudget] = useState<string>("");
  const [mustMoveBy, setMustMoveBy] = useState<string>("");
  const [sustained, setSustained] = useState(false);
  const [noStragglers, setNoStragglers] = useState(false);
  const [periodBand, setPeriodBand] = useState(false);
  const [targetPeriod, setTargetPeriod] = useState<string>("10");
  const [banned, setBanned] = useState<Set<string>>(new Set());
  const [required, setRequired] = useState<Set<string>>(new Set());

  const [open, setOpen] = useState<Record<GroupId, boolean>>({
    run: true,
    genome: false,
    objectives: true,
    constraints: true,
    schedules: true,
    advanced: false,
  });

  const volume = Math.max(1, bx * by * bz);
  const checkedKeys = OBJECTIVE_ORDER.filter((k) => objs[k].on);
  const nowRate =
    running && rates.length > 0 ? rates[rates.length - 1].rate : mutationRate;

  const toggleObj = (k: ObjectiveKey) =>
    setObjs((o) => ({ ...o, [k]: { ...o[k], on: !o[k].on } }));
  const setWeight = (k: ObjectiveKey, w: string) =>
    setObjs((o) => ({ ...o, [k]: { ...o[k], weight: w } }));
  /** Flip an objective's direction. Minimizing speed means hunting the
   * slowest machine that still flies, so the sustained-flight constraint
   * follows the toggle by default (still user-overridable below). */
  const flipDir = (k: ObjectiveKey) =>
    setObjs((o) => {
      const dir: ObjectiveDir = o[k].dir === "max" ? "min" : "max";
      if (k === "speed") setSustained(dir === "min");
      return { ...o, [k]: { ...o[k], dir } };
    });

  const toggleKind = (
    kind: string,
    set: Set<string>,
    setter: (s: Set<string>) => void,
    other: Set<string>,
    otherSetter: (s: Set<string>) => void,
  ) => {
    const next = new Set(set);
    if (next.has(kind)) next.delete(kind);
    else {
      next.add(kind);
      if (other.has(kind)) {
        const o = new Set(other);
        o.delete(kind);
        otherSetter(o);
      }
    }
    setter(next);
  };

  const buildObjectives = (): ObjectiveChoice[] => {
    const list = checkedKeys.map((key) => ({
      key,
      weight: parseWeight(objs[key].weight),
      dir: objs[key].dir,
    }));
    return list.length > 0
      ? list
      : [{ key: "speed" as ObjectiveKey, weight: 1, dir: "max" as ObjectiveDir }];
  };

  const targetPeriodVal = (): number | null => {
    const t = parseInt(targetPeriod, 10);
    const needed = objs.period.on || periodBand;
    return needed && Number.isFinite(t) && t > 0 ? t : null;
  };

  const buildConstraints = (): Constraints => {
    const budget = pistonBudget.trim() === "" ? null : parseInt(pistonBudget, 10);
    const moveBy = mustMoveBy.trim() === "" ? null : parseInt(mustMoveBy, 10);
    const cappedMax = Math.max(1, Math.min(volume, maxBlocks));
    return {
      maxBlocks: cappedMax,
      minBlocks: Math.max(1, Math.min(cappedMax, minBlocks)),
      pistonBudget:
        budget !== null && Number.isFinite(budget) && budget > 0 ? budget : null,
      mustMoveByTick:
        moveBy !== null && Number.isFinite(moveBy) && moveBy > 0 ? moveBy : null,
      requireSustained: sustained,
      noStragglers,
      periodBand: periodBand && targetPeriodVal() !== null,
      banned: [...banned],
      required: [...required],
    };
  };

  const buildSpec = (): MapElitesSpec =>
    normalizeSpec({ x: beX, y: beY, binsX, binsY, quality });

  const buildSchedule = (): MutationSchedule => {
    if (schedKind === "linear")
      return { kind: "linear", spanGens: posInt(spanGens, 120) };
    if (schedKind === "exponential")
      return { kind: "exponential", halfLifeGens: posInt(halfLife, 30) };
    return { kind: schedKind };
  };

  /** The gates implied by the currently selected objectives (locked chip). */
  const impliedBy = useMemo(
    () => impliedGateBy(buildObjectives()),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [objs],
  );

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (running) return;
    const gens = generations.trim() === "" ? null : parseInt(generations, 10);
    onStart({
      population,
      generations: gens !== null && Number.isFinite(gens) && gens > 0 ? gens : null,
      mutation_rate: mutationRate,
      mutationSchedule: buildSchedule(),
      seeding,
      bbox: [bx, by, bz],
      eval_ticks: evalTicks,
      seed,
      workers: Math.max(1, Math.min(6, workers)),
      mode,
      objectives: buildObjectives(),
      constraints: buildConstraints(),
      targetPeriod: targetPeriodVal(),
      mapElites: buildSpec(),
    });
  };

  // Mid-run reconfiguration: objectives, constraints and the mutation
  // regime stream to the runner (applied at the next generation boundary).
  // The first pass after a run starts only records the baseline.
  const patchJson = JSON.stringify({
    objectives: buildObjectives(),
    constraints: buildConstraints(),
    targetPeriod: targetPeriodVal(),
    mutationRate,
    mutationSchedule: buildSchedule(),
    mapElites: buildSpec(),
  });
  const sentRef = useRef<string | null>(null);
  useEffect(() => {
    if (!running || !onReconfigure) {
      sentRef.current = null;
      return;
    }
    if (sentRef.current === null) {
      sentRef.current = patchJson;
      return;
    }
    if (patchJson === sentRef.current) return;
    const t = setTimeout(() => {
      sentRef.current = patchJson;
      onReconfigure(JSON.parse(patchJson) as LiveConfigPatch);
    }, 350);
    return () => clearTimeout(t);
  }, [patchJson, running, onReconfigure]);

  const num =
    (set: (v: number) => void, int = true) =>
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const v = int ? parseInt(e.target.value, 10) : parseFloat(e.target.value);
      if (!Number.isNaN(v)) set(v);
    };

  const showPeriodTarget = objs.period.on || periodBand;

  // --- collapsed-summary readouts (the rail reads folded shut) ------------
  const extraConstraints =
    (pistonBudget.trim() !== "" ? 1 : 0) +
    (mustMoveBy.trim() !== "" ? 1 : 0) +
    (sustained || impliedBy.length > 0 ? 1 : 0) +
    (noStragglers ? 1 : 0) +
    (periodBand ? 1 : 0) +
    (banned.size > 0 ? 1 : 0) +
    (required.size > 0 ? 1 : 0);
  const reads: Record<GroupId, string> = {
    run: `pop ${population} · ${generations.trim() === "" ? "∞" : generations} gens · ${workers}w`,
    genome: `${bx}·${by}·${bz} · ${seeding}`,
    objectives:
      mode === "map-elites"
        ? `map-elites · ${BEHAVIOURS[beX].label}×${BEHAVIOURS[beY].label} ${binsX}×${binsY}`
        : `${
            checkedKeys.length > 0
              ? checkedKeys
                  .map(
                    (k) =>
                      `${OBJECTIVES[k].label}${objs[k].dir === "max" ? "↑" : "↓"}`,
                  )
                  .join(" + ")
              : "speed↑"
          } · ${mode}`,
    constraints: `${minBlocks}–${maxBlocks} blk${extraConstraints > 0 ? ` · ${extraConstraints} more` : ""}`,
    schedules: `${schedKind} · ${nowRate.toFixed(3)}`,
    advanced: `seed ${seed} · ${evalTicks}t`,
  };

  const group = (
    id: GroupId,
    title: string,
    live: boolean,
    children: React.ReactNode,
  ) => (
    <details
      className="cfg-group"
      open={open[id]}
      onToggle={(e) => {
        const isOpen = (e.target as HTMLDetailsElement).open;
        setOpen((o) => (o[id] === isOpen ? o : { ...o, [id]: isOpen }));
      }}
      data-testid={`group-${id}`}
    >
      <summary>
        <span className="g-title">{title}</span>
        {running && (
          <span
            className={`g-live${live ? " on" : ""}`}
            title={
              live
                ? "live — changes apply at the next generation boundary"
                : LOCKED_TIP
            }
          >
            {live ? "live" : "locked"}
          </span>
        )}
        <span className="g-read">{reads[id]}</span>
      </summary>
      <div className="g-body">{children}</div>
    </details>
  );

  return (
    <form className="cfg" onSubmit={submit}>
      <div className="cfg-groups">
        {group(
          "run",
          "Run",
          false,
          <>
            <div className="row2">
              <div className="field" title={running ? LOCKED_TIP : undefined}>
                <label htmlFor="cfg-pop">Population</label>
                <input id="cfg-pop" type="number" min={2} max={4096} value={population} onChange={num(setPopulation)} disabled={running} />
              </div>
              <div className="field" title={running ? LOCKED_TIP : undefined}>
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
            <div className="field" title={running ? LOCKED_TIP : undefined}>
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
          </>,
        )}

        {group(
          "genome",
          "Genome",
          false,
          <>
            <div className="field" title={running ? LOCKED_TIP : undefined}>
              <label>Bounding box (x·y·z)</label>
              <div className="row3">
                <input aria-label="Bounding box x" type="number" min={1} max={16} value={bx} onChange={num(setBx)} disabled={running} />
                <input aria-label="Bounding box y" type="number" min={1} max={16} value={by} onChange={num(setBy)} disabled={running} />
                <input aria-label="Bounding box z" type="number" min={1} max={16} value={bz} onChange={num(setBz)} disabled={running} />
              </div>
              <p className="hint">Machines evolve inside this volume.</p>
            </div>
            <div className="field" title={running ? LOCKED_TIP : undefined}>
              <label htmlFor="cfg-seeding">Gen-0 seed</label>
              <select
                id="cfg-seeding"
                value={seeding}
                onChange={(e) => setSeeding(e.target.value as SeedMode)}
                disabled={running}
                data-testid="cfg-seeding"
              >
                <option value="minimal">minimal — piston + slime, grow from there</option>
                <option value="engine-b">engine-b — start from a proven flier</option>
                <option value="random">random — no head start</option>
              </select>
              <p className="hint">
                minimal watches complexity grow from a 2-block seed; the rest of
                gen 0 is random either way.
              </p>
            </div>
          </>,
        )}

        {group(
          "objectives",
          "Objectives",
          true,
          <>
            <div
              className="mode-toggle"
              role="radiogroup"
              aria-label="Optimization mode"
              title={running ? "selection mode is locked during a run" : undefined}
            >
              <button
                type="button"
                role="radio"
                aria-checked={mode === "scalar"}
                className={mode === "scalar" ? "on" : ""}
                onClick={() => setMode("scalar")}
                disabled={running}
                data-testid="mode-scalar"
              >
                weighted
              </button>
              <button
                type="button"
                role="radio"
                aria-checked={mode === "pareto"}
                className={mode === "pareto" ? "on" : ""}
                onClick={() => setMode("pareto")}
                disabled={running}
                data-testid="mode-pareto"
              >
                Pareto
              </button>
              <button
                type="button"
                role="radio"
                aria-checked={mode === "map-elites"}
                className={mode === "map-elites" ? "on" : ""}
                onClick={() => setMode("map-elites")}
                disabled={running}
                data-testid="mode-map-elites"
              >
                MAP-Elites
              </button>
            </div>
            {mode === "map-elites" && (
              <div className="me-config" data-testid="me-config">
                <p className="hint">
                  Every machine is filed by BEHAVIOUR, one per cell, and parents
                  are drawn evenly from the filled cells. Slow and strange
                  designs keep breeding instead of being out-competed — that is
                  what turns them into stepping stones toward fliers.
                </p>
                <div className="row2">
                  <div className="field">
                    <label htmlFor="cfg-be-x">Grid across</label>
                    <select
                      id="cfg-be-x"
                      value={beX}
                      onChange={(e) => setBeX(e.target.value as BehaviourKey)}
                      data-testid="cfg-be-x"
                    >
                      {BEHAVIOUR_ORDER.filter((k) => k !== beY).map((k) => (
                        <option key={k} value={k}>
                          {BEHAVIOURS[k].label} ({BEHAVIOURS[k].unit})
                        </option>
                      ))}
                    </select>
                  </div>
                  <div className="field">
                    <label htmlFor="cfg-be-y">Grid up</label>
                    <select
                      id="cfg-be-y"
                      value={beY}
                      onChange={(e) => setBeY(e.target.value as BehaviourKey)}
                      data-testid="cfg-be-y"
                    >
                      {BEHAVIOUR_ORDER.filter((k) => k !== beX).map((k) => (
                        <option key={k} value={k}>
                          {BEHAVIOURS[k].label} ({BEHAVIOURS[k].unit})
                        </option>
                      ))}
                    </select>
                  </div>
                </div>
                <div className="row2">
                  <div className="field">
                    <label htmlFor="cfg-bins-x">Bins across ({binsX})</label>
                    <input
                      id="cfg-bins-x"
                      data-testid="cfg-bins-x"
                      type="range"
                      min={MAP_ELITES_BIN_MIN}
                      max={MAP_ELITES_BIN_MAX}
                      value={binsX}
                      onChange={num(setBinsX)}
                    />
                  </div>
                  <div className="field">
                    <label htmlFor="cfg-bins-y">Bins up ({binsY})</label>
                    <input
                      id="cfg-bins-y"
                      data-testid="cfg-bins-y"
                      type="range"
                      min={MAP_ELITES_BIN_MIN}
                      max={MAP_ELITES_BIN_MAX}
                      value={binsY}
                      onChange={num(setBinsY)}
                    />
                  </div>
                </div>
                <div className="field">
                  <label htmlFor="cfg-quality">Cell keeps its best by</label>
                  <select
                    id="cfg-quality"
                    value={quality}
                    onChange={(e) => setQuality(e.target.value as QualityKey)}
                    data-testid="cfg-quality"
                  >
                    {QUALITY_ORDER.map((k) => (
                      <option key={k} value={k}>
                        {QUALITIES[k].label} ({QUALITIES[k].unit})
                      </option>
                    ))}
                  </select>
                </div>
                <p className="hint">
                  {binsX * binsY} cells. Changing dimensions or bins mid-run
                  re-bins the machines the archive already holds — nothing is
                  discarded.
                </p>
              </div>
            )}
            {OBJECTIVE_ORDER.map((k) => {
              const def = OBJECTIVES[k];
              const inverted = objs[k].dir !== naturalDir(k);
              return (
                <div className="obj-row" key={k} title={def.hint}>
                  <label className="obj-check">
                    <input
                      type="checkbox"
                      checked={objs[k].on}
                      onChange={() => toggleObj(k)}
                      data-testid={`obj-${k}`}
                    />
                    <span>{def.label}</span>
                    <span className="obj-unit">{def.unit}</span>
                  </label>
                  {objs[k].on && (
                    <button
                      type="button"
                      className={`obj-dir${inverted ? " inverted" : ""}`}
                      onClick={() => flipDir(k)}
                      data-testid={`dir-${k}`}
                      aria-pressed={inverted}
                      aria-label={`${def.label}: ${objs[k].dir === "max" ? "maximize" : "minimize"} — click to flip`}
                      title={
                        inverted
                          ? `inverted: hunting ${objs[k].dir === "min" ? "the smallest" : "the largest"} ${def.label}`
                          : `natural direction — click to hunt the ${objs[k].dir === "max" ? "smallest" : "largest"} ${def.label} instead`
                      }
                    >
                      {objs[k].dir === "max" ? "max ↑" : "min ↓"}
                    </button>
                  )}
                  {mode === "scalar" && objs[k].on && (
                    <input
                      className="obj-weight"
                      type="number"
                      min={0}
                      step={0.1}
                      value={objs[k].weight}
                      onChange={(e) => setWeight(k, e.target.value)}
                      aria-label={`${def.label} weight`}
                      data-testid={`weight-${k}`}
                    />
                  )}
                </div>
              );
            })}
            {showPeriodTarget && (
              <div className="field">
                <label htmlFor="cfg-target-period">Target period (ticks)</label>
                <input
                  id="cfg-target-period"
                  type="number"
                  min={2}
                  max={80}
                  value={targetPeriod}
                  onChange={(e) => setTargetPeriod(e.target.value)}
                  data-testid="cfg-target-period"
                />
                <p className="hint">
                  detector: min-x rise-gap cadence over the flight's last 120
                  ticks — measured in-eval only while a period target is in play.
                </p>
              </div>
            )}
            {impliedBy.length > 0 && (
              <button
                type="button"
                className="chip lock on"
                data-testid="gate-chip"
                aria-disabled="true"
                title={`Implied gate — ${impliedBy.join(", ")} only mean anything on a machine that actually flies. Must-keep-flying + clean-flight are enforced; non-fliers become invalid (culled), not "great at ${impliedBy[0]}".`}
              >
                🔒 must keep flying — implied by {impliedBy.join(", ")}
              </button>
            )}
            <p className="hint">
              {mode === "map-elites"
                ? "MAP-Elites ignores the weights for selection — the checked objectives still rank the leaderboard and set the implied flight gates."
                : mode === "pareto"
                  ? checkedKeys.length === 2
                    ? "Pareto: front of trade-offs, archived across generations."
                    : "Pick exactly 2 objectives to plot the front (archive still works)."
                  : "Weighted sum of the checked objectives (fliers only)."}
            </p>
            {objs.robustness.on && (
              <p className="hint">robustness re-flies each machine — slower evals.</p>
            )}
          </>,
        )}

        {group(
          "constraints",
          "Constraints",
          true,
          <>
            <div className="field">
              <label htmlFor="cfg-maxb">
                Max blocks ({Math.max(1, Math.min(volume, maxBlocks))})
              </label>
              <input
                id="cfg-maxb"
                data-testid="cfg-maxb"
                type="range"
                min={1}
                max={volume}
                value={Math.min(volume, maxBlocks)}
                onChange={num(setMaxBlocks)}
              />
            </div>
            <div className="field">
              <label htmlFor="cfg-minb">
                Min blocks ({Math.max(1, Math.min(maxBlocks, minBlocks))})
              </label>
              <input
                id="cfg-minb"
                data-testid="cfg-minb"
                type="range"
                min={1}
                max={volume}
                value={Math.min(volume, minBlocks)}
                onChange={num(setMinBlocks)}
              />
            </div>
            <p className="hint">
              14 is just the default cap, not a ceiling: a piston pushes at most
              12 blocks per push LINE, but multi-engine machines push per row and
              can be far bigger (up to the {volume}-cell box). Unpushable designs
              simply never fly.
            </p>
            <div className="row2">
              <div className="field">
                <label htmlFor="cfg-pistons">Piston cap</label>
                <input
                  id="cfg-pistons"
                  type="number"
                  min={1}
                  placeholder="any"
                  value={pistonBudget}
                  onChange={(e) => setPistonBudget(e.target.value)}
                />
              </div>
              <div className="field">
                <label htmlFor="cfg-moveby">Must move by tick</label>
                <input
                  id="cfg-moveby"
                  type="number"
                  min={5}
                  placeholder="no deadline"
                  value={mustMoveBy}
                  onChange={(e) => setMustMoveBy(e.target.value)}
                />
              </div>
            </div>
            <div className="obj-row">
              <label
                className="obj-check"
                title={
                  impliedBy.length > 0
                    ? `auto-enabled by ${impliedBy.join(", ")} — flight-dependent objectives cull non-fliers`
                    : "second-half displacement must be ≥ 1 block and ≥ 25% of the total — lurch-and-stall machines score 0"
                }
              >
                <input
                  type="checkbox"
                  checked={sustained || impliedBy.length > 0}
                  disabled={impliedBy.length > 0}
                  onChange={() => setSustained((s) => !s)}
                  data-testid="cfg-sustained"
                />
                <span>must keep flying</span>
                <span className="obj-unit">
                  {impliedBy.length > 0 ? "🔒 implied" : "no lurch-and-stall"}
                </span>
              </label>
            </div>
            <div className="obj-row">
              <label
                className="obj-check"
                title="left-behind estimate must be exactly 0 — the machine arrives whole; debris-shedders are invalid, not penalized"
              >
                <input
                  type="checkbox"
                  checked={noStragglers}
                  onChange={() => setNoStragglers((s) => !s)}
                  data-testid="cfg-no-stragglers"
                />
                <span>no stragglers</span>
                <span className="obj-unit">arrive whole</span>
              </label>
            </div>
            <div className="obj-row">
              <label
                className="obj-check"
                title="detected gait period must land within ±1 tick of the target — set the target in Objectives"
              >
                <input
                  type="checkbox"
                  checked={periodBand}
                  onChange={() => setPeriodBand((s) => !s)}
                  data-testid="cfg-period-band"
                />
                <span>period within ±1 of target</span>
                <span className="obj-unit">gait band</span>
              </label>
            </div>
            <div className="field">
              <label>Banned kinds</label>
              <div className="chips" role="group" aria-label="Banned block kinds">
                {PLACEABLE_KINDS.map((k) => (
                  <button
                    type="button"
                    key={k}
                    className={`chip${banned.has(k) ? " on ban" : ""}`}
                    aria-pressed={banned.has(k)}
                    onClick={() => toggleKind(k, banned, setBanned, required, setRequired)}
                  >
                    {KIND_LABEL[k] ?? k}
                  </button>
                ))}
              </div>
            </div>
            <div className="field">
              <label>Required kinds</label>
              <div className="chips" role="group" aria-label="Required block kinds">
                {PLACEABLE_KINDS.map((k) => (
                  <button
                    type="button"
                    key={k}
                    className={`chip${required.has(k) ? " on req" : ""}`}
                    aria-pressed={required.has(k)}
                    onClick={() => toggleKind(k, required, setRequired, banned, setBanned)}
                  >
                    {KIND_LABEL[k] ?? k}
                  </button>
                ))}
              </div>
            </div>
            <p className="hint">Violating machines score 0 — hard filters.</p>
          </>,
        )}

        {group(
          "schedules",
          "Schedules",
          true,
          <>
            <div className="field">
              <label htmlFor="cfg-mut">
                Mutation rate ({mutationRate.toFixed(3)})
                {running && (
                  <span className="rate-now" data-testid="rate-now">
                    {" "}
                    · now {nowRate.toFixed(3)}
                  </span>
                )}
              </label>
              <input
                id="cfg-mut"
                data-testid="cfg-mutrate"
                type="range"
                min={0.005}
                max={0.5}
                step={0.005}
                value={mutationRate}
                onChange={num(setMutationRate, false)}
              />
              {running && (
                <p className="hint">
                  moving the slider mid-run overrides the schedule and pins the
                  rate from the next generation.
                </p>
              )}
            </div>
            <div className="field">
              <label htmlFor="cfg-mut-schedule">Rate schedule</label>
              <select
                id="cfg-mut-schedule"
                data-testid="cfg-mut-schedule"
                value={schedKind}
                onChange={(e) => setSchedKind(e.target.value as MutationScheduleKind)}
              >
                <option value="constant">constant — no anneal</option>
                <option value="linear">linear decay</option>
                <option value="exponential">exponential half-life</option>
                <option value="adaptive">adaptive — 1/5th-success rule</option>
              </select>
              {schedKind === "linear" && (
                <>
                  <label htmlFor="cfg-mut-span" className="sub-label">
                    Decay span (gens)
                  </label>
                  <input
                    id="cfg-mut-span"
                    data-testid="cfg-mut-span"
                    type="number"
                    min={1}
                    value={spanGens}
                    onChange={(e) => setSpanGens(e.target.value)}
                  />
                  <p className="hint">
                    decays to 10 % of the anchor rate over the span, then holds.
                  </p>
                </>
              )}
              {schedKind === "exponential" && (
                <>
                  <label htmlFor="cfg-mut-halflife" className="sub-label">
                    Half-life (gens)
                  </label>
                  <input
                    id="cfg-mut-halflife"
                    data-testid="cfg-mut-halflife"
                    type="number"
                    min={1}
                    value={halfLife}
                    onChange={(e) => setHalfLife(e.target.value)}
                  />
                  <p className="hint">rate halves every half-life, anchored where set.</p>
                </>
              )}
              {schedKind === "adaptive" && (
                <p className="hint">
                  1/5th-success rule: over the last 10 gens, a gen "succeeds"
                  when it sets a champion or adds a Pareto point; success &lt;
                  1/5 → rate ×1.15, &gt; 1/5 → ×0.87, clamped 0.005–0.5.
                </p>
              )}
            </div>
            {rates.length > 1 && (
              <div className="field">
                <label>Rate over the run</label>
                <RateSparkline points={rates} />
              </div>
            )}
          </>,
        )}

        {group(
          "advanced",
          "Advanced",
          false,
          <>
            <div className="row2">
              <div className="field" title={running ? LOCKED_TIP : undefined}>
                <label htmlFor="cfg-ticks">Eval ticks</label>
                <input id="cfg-ticks" type="number" min={20} max={20000} step={20} value={evalTicks} onChange={num(setEvalTicks)} disabled={running} />
              </div>
              <div className="field" title={running ? LOCKED_TIP : undefined}>
                <label htmlFor="cfg-seed">Seed</label>
                <input id="cfg-seed" type="number" value={seed} onChange={num(setSeed)} disabled={running} />
              </div>
            </div>
            <p className="hint">rarely touched — the eval window and the RNG seed.</p>
          </>,
        )}
      </div>

      {error && <p className="cfg-error">{error}</p>}

      {running && (
        <p className="hint" data-testid="live-edit-hint">
          run is live — objective, constraint &amp; mutation changes apply at
          the next generation boundary.
        </p>
      )}

      <div className="cfg-actions">
        {running ? (
          <>
            {paused ? (
              <button
                type="button"
                className="btn-resume"
                onClick={onResume}
                data-testid="resume-run"
              >
                Resume
              </button>
            ) : (
              <button
                type="button"
                className="btn-pause"
                onClick={onPause}
                data-testid="pause-run"
              >
                Pause
              </button>
            )}
            <button type="button" className="btn-stop" onClick={onStop} data-testid="stop-run">
              Stop run
            </button>
          </>
        ) : (
          <button type="submit" className="btn-start" data-testid="start-run">
            Start evolution
          </button>
        )}
      </div>
    </form>
  );
}
