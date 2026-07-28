/**
 * Mock dev backend for the flying-machine evolution lab.
 *
 * Served as Vite middleware at the SAME routes a real backend would own:
 *   POST /api/runs                -> { id }
 *   GET  /api/runs/:id            -> run state (history grows over ~30 s)
 *   POST /api/runs/:id/stop       -> freezes the run
 *
 * A run "evolves" in wall-clock time: the full history is precomputed from the
 * seed at creation, then revealed as time passes (~30 s to finish).
 */
import type { Plugin, Connect } from "vite";
import type { ServerResponse } from "node:http";

interface RunConfig {
  population: number;
  generations: number;
  mutation_rate: number;
  bbox: [number, number, number];
  eval_ticks: number;
  seed: number;
}

interface Block {
  x: number;
  y: number;
  z: number;
  state: string;
}

interface Machine {
  id: string;
  name: string;
  fitness: number;
  gen: number;
  blocks: Block[];
}

interface HistoryPoint {
  gen: number;
  best: number;
  mean: number;
}

interface Run {
  id: string;
  config: RunConfig;
  startedAt: number;
  durationMs: number;
  history: HistoryPoint[]; // full, precomputed; revealed by elapsed time
  machines: Machine[]; // discovery gen per machine; revealed by gen
  stoppedAtGen: number | null;
}

/* ---------------------------------------------------------------- machines */

const B = (x: number, y: number, z: number, state: string): Block => ({
  x,
  y,
  z,
  state: `minecraft:${state}`,
});

/** Hand-authored plausible machines inside a 5x3x3 box. Fitness is a scale
 *  factor (0..1) of the run's final best — so numbers stay coherent for any
 *  config. */
const MACHINE_LIBRARY: Array<{
  name: string;
  discoveryFrac: number; // fraction of total generations when it appears
  fitnessFrac: number; // fraction of the run's final best fitness
  blocks: Block[];
}> = [
  {
    // Early junk: an engine that stutters a few blocks then stalls.
    name: "sputter-4",
    discoveryFrac: 0.05,
    fitnessFrac: 0.12,
    blocks: [
      B(1, 1, 1, "slime_block"),
      B(2, 1, 1, "sticky_piston[facing=west]"),
      B(3, 1, 1, "observer[facing=east]"),
      B(3, 0, 1, "slime_block"),
    ],
  },
  {
    // The classic 2-piston flying machine: piston/observer engine pair
    // pulling a slime nose, single axis of travel.
    name: "classic 2-piston",
    discoveryFrac: 0.2,
    fitnessFrac: 0.61,
    blocks: [
      B(0, 1, 1, "slime_block"),
      B(1, 1, 1, "sticky_piston[facing=west]"),
      B(2, 1, 1, "observer[facing=east]"),
      B(3, 1, 1, "sticky_piston[facing=west]"),
      B(4, 1, 1, "observer[facing=east]"),
    ],
  },
  {
    // Classic engine towing honey cargo underneath.
    name: "honey hauler",
    discoveryFrac: 0.5,
    fitnessFrac: 0.8,
    blocks: [
      B(0, 1, 1, "slime_block"),
      B(1, 1, 1, "sticky_piston[facing=west]"),
      B(2, 1, 1, "observer[facing=east]"),
      B(3, 1, 1, "sticky_piston[facing=west]"),
      B(4, 1, 1, "observer[facing=east]"),
      B(0, 0, 1, "honey_block"),
      B(2, 0, 1, "honey_block"),
      B(4, 0, 1, "honey_block"),
    ],
  },
  {
    // Twin engines on parallel rails, slime-bridged: the run's champion.
    name: "twin-rail v2",
    discoveryFrac: 0.78,
    fitnessFrac: 1.0,
    blocks: [
      B(0, 1, 0, "slime_block"),
      B(1, 1, 0, "sticky_piston[facing=west]"),
      B(2, 1, 0, "observer[facing=east]"),
      B(3, 1, 0, "sticky_piston[facing=west]"),
      B(4, 1, 0, "observer[facing=east]"),
      B(0, 1, 2, "slime_block"),
      B(1, 1, 2, "sticky_piston[facing=west]"),
      B(2, 1, 2, "observer[facing=east]"),
      B(3, 1, 2, "sticky_piston[facing=west]"),
      B(4, 1, 2, "observer[facing=east]"),
      B(2, 2, 1, "slime_block"),
      B(0, 2, 1, "slime_block"),
    ],
  },
];

/* ------------------------------------------------------------------- utils */

function mulberry32(seed: number) {
  let a = seed >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const round1 = (v: number) => Math.round(v * 10) / 10;

/** Precompute the whole evolution story: best jumps at machine discoveries,
 *  plateaus with small refinements between, mean trails noisily below. */
function buildRun(id: string, config: RunConfig): Run {
  const rng = mulberry32(config.seed || 1);
  const G = Math.max(2, config.generations);

  // Final best scales with eval budget: a machine moving ~1 block / 4 ticks.
  const finalBest = Math.max(10, (config.eval_ticks / 4) * (0.55 + rng() * 0.2));

  const machines: Machine[] = MACHINE_LIBRARY.map((m, i) => ({
    id: `m${i + 1}`,
    name: m.name,
    fitness: round1(finalBest * m.fitnessFrac),
    gen: Math.max(1, Math.round(m.discoveryFrac * (G - 1))),
    blocks: m.blocks,
  }));

  const history: HistoryPoint[] = [];
  let best = 0;
  for (let gen = 0; gen < G; gen++) {
    const discovered = machines.filter((m) => m.gen === gen);
    for (const m of discovered) best = Math.max(best, m.fitness);
    // Plateau with occasional small refinements between discoveries.
    if (discovered.length === 0 && rng() < 0.3) {
      best += rng() * finalBest * 0.008;
    }
    const meanFrac = 0.25 + 0.3 * (gen / G) + (rng() - 0.5) * 0.1;
    const mean = Math.max(0, Math.min(best * 0.92, best * meanFrac));
    history.push({ gen, best: round1(best), mean: round1(mean) });
  }

  return {
    id,
    config,
    startedAt: Date.now(),
    durationMs: 30_000,
    history,
    machines,
    stoppedAtGen: null,
  };
}

function runSnapshot(run: Run) {
  const G = run.history.length;
  const elapsed = Date.now() - run.startedAt;
  let gen = Math.min(G - 1, Math.floor((elapsed / run.durationMs) * G));
  if (run.stoppedAtGen !== null) gen = run.stoppedAtGen;
  const done = run.stoppedAtGen !== null || gen >= G - 1;

  const jitter = 0.92 + 0.16 * Math.abs(Math.sin(Date.now() / 900));
  const evalsPerSec = done
    ? 0
    : Math.round(run.config.population * (9000 / run.config.eval_ticks) * jitter);

  const leaderboard = run.machines
    .filter((m) => m.gen <= gen)
    .sort((a, b) => b.fitness - a.fitness)
    .map((m) => ({
      id: m.id,
      name: m.name,
      fitness: m.fitness,
      gen: m.gen,
      blocks: m.blocks,
    }));

  return {
    status: done ? "done" : "running",
    generation: gen,
    total_generations: G,
    history: run.history.slice(0, gen + 1),
    evals_per_sec: evalsPerSec,
    leaderboard,
  };
}

/* ------------------------------------------------------------------ plugin */

function json(res: ServerResponse, code: number, body: unknown) {
  res.statusCode = code;
  res.setHeader("Content-Type", "application/json");
  res.end(JSON.stringify(body));
}

function readBody(req: Connect.IncomingMessage): Promise<string> {
  return new Promise((resolve) => {
    let data = "";
    req.on("data", (c) => (data += c));
    req.on("end", () => resolve(data));
  });
}

export function mockApiPlugin(): Plugin {
  const runs = new Map<string, Run>();
  let nextId = 1;

  const middleware: Connect.NextHandleFunction = async (req, res, next) => {
    const url = (req.url ?? "").split("?")[0];
    if (!url.startsWith("/api/")) return next();

    if (req.method === "POST" && url === "/api/runs") {
      const raw = await readBody(req);
      let cfg: Partial<RunConfig> = {};
      try {
        cfg = JSON.parse(raw || "{}");
      } catch {
        return json(res, 400, { error: "invalid JSON body" });
      }
      const config: RunConfig = {
        population: cfg.population ?? 96,
        generations: cfg.generations ?? 60,
        mutation_rate: cfg.mutation_rate ?? 0.05,
        bbox: cfg.bbox ?? [5, 3, 3],
        eval_ticks: cfg.eval_ticks ?? 600,
        seed: cfg.seed ?? 42,
      };
      const id = `run${nextId++}`;
      runs.set(id, buildRun(id, config));
      return json(res, 200, { id });
    }

    const stopMatch = url.match(/^\/api\/runs\/([^/]+)\/stop$/);
    if (req.method === "POST" && stopMatch) {
      const run = runs.get(stopMatch[1]);
      if (!run) return json(res, 404, { error: "run not found" });
      if (run.stoppedAtGen === null) {
        run.stoppedAtGen = runSnapshot(run).generation;
      }
      return json(res, 200, { ok: true });
    }

    const getMatch = url.match(/^\/api\/runs\/([^/]+)$/);
    if (req.method === "GET" && getMatch) {
      const run = runs.get(getMatch[1]);
      if (!run) return json(res, 404, { error: "run not found" });
      return json(res, 200, runSnapshot(run));
    }

    return json(res, 404, { error: "unknown route" });
  };

  return {
    name: "flying-ga-mock-api",
    configureServer(server) {
      server.middlewares.use(middleware);
    },
    configurePreviewServer(server) {
      server.middlewares.use(middleware);
    },
  };
}
