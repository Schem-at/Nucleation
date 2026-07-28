"""Flying-machine evolution lab — real backend.

Replaces the frontend's vite mock (frontend/mock/api.ts) at identical routes,
serving the built frontend dist/ with SPA fallback, and evolving real
flying machines on nucleation's TickSimulation (headless mc-tick).

Routes:
  POST /api/runs           {population, generations, mutation_rate, bbox,
                            eval_ticks, seed}                    -> { id }
  GET  /api/runs/:id       -> { status, generation, total_generations,
                                history:[{gen,best,mean}], evals_per_sec,
                                leaderboard:[{id,name?,fitness,gen,blocks}] }
  POST /api/runs/:id/stop  -> { ok: true }
  GET  /*                  -> frontend dist with SPA fallback

Run:  .venv/bin/python main.py          (binds 0.0.0.0:8440)
"""

from __future__ import annotations

import json
import multiprocessing as mp
import os
import random
import threading
import time
from pathlib import Path

from fastapi import FastAPI, Request
from fastapi.responses import FileResponse, JSONResponse

import ga

HERE = Path(__file__).resolve().parent
DIST = HERE.parent / "frontend" / "dist"

# Parallelism: nucleation evals are native and release the GIL only inside
# the sim, so we use a spawn Pool of worker processes. Set FLYING_GA_WORKERS=1
# to force single-process evaluation.
WORKERS = int(os.environ.get("FLYING_GA_WORKERS", max(1, (os.cpu_count() or 4) - 2)))

ELITISM = 2
TOURNAMENT_K = 3
LEADERBOARD_SIZE = 10

app = FastAPI()

_runs: dict[str, "Run"] = {}
_runs_lock = threading.Lock()


class Run:
    def __init__(self, run_id: str, cfg: dict):
        self.id = run_id
        self.cfg = cfg
        self.lock = threading.Lock()
        self.status = "running"
        self.generation = 0
        self.history: list[dict] = []
        self.evals_per_sec = 0.0
        self.leaderboard: list[dict] = []
        self.stop_flag = threading.Event()
        self.thread = threading.Thread(target=self._evolve, daemon=True)
        self.thread.start()

    # ------------------------------------------------------------- state

    def snapshot(self) -> dict:
        with self.lock:
            return {
                "status": self.status,
                "generation": self.generation,
                "total_generations": self.cfg["generations"],
                "history": list(self.history),
                "evals_per_sec": round(self.evals_per_sec, 1),
                "leaderboard": [dict(e) for e in self.leaderboard],
            }

    # --------------------------------------------------------- evolution

    def _evolve(self) -> None:
        cfg = self.cfg
        bbox = tuple(cfg["bbox"])
        pop_size = cfg["population"]
        rng = random.Random(cfg["seed"])
        pool = None
        try:
            if WORKERS > 1:
                pool = mp.get_context("spawn").Pool(WORKERS)

            pop: list[ga.Genome] = []
            names: dict[ga.Genome, str] = {}
            for mirror, name in ((False, "engine-b"), (True, "engine-b-mirror")):
                g = ga.engine_b_genome(bbox, mirror)
                if g is not None:
                    pop.append(g)
                    names[g] = name
            while len(pop) < pop_size:
                pop.append(ga.random_genome(bbox, rng))
            pop = pop[:pop_size]

            board: dict[ga.Genome, dict] = {}

            for gen in range(cfg["generations"]):
                if self.stop_flag.is_set():
                    break
                t0 = time.time()
                tasks = [(g, bbox, cfg["eval_ticks"], cfg["seed"]) for g in pop]
                if pool is not None:
                    fits = pool.map(ga.evaluate, tasks, chunksize=2)
                else:
                    fits = [ga.evaluate(t) for t in tasks]
                dt = max(time.time() - t0, 1e-9)

                best = max(fits)
                mean = sum(fits) / len(fits)

                for g, f in zip(pop, fits):
                    if f <= 0:
                        continue
                    prev = board.get(g)
                    if prev is None or f > prev["fitness"]:
                        board[g] = {
                            "id": f"m{abs(hash(g)) % 10**8:08d}",
                            "name": names.get(g),
                            "fitness": round(f, 2),
                            "gen": prev["gen"] if prev else gen,
                            "blocks": [
                                {"x": x, "y": y, "z": z, "state": ga.ALPHABET[s]}
                                for x, y, z, s in ga.genome_blocks(g, bbox)
                            ],
                        }
                top = sorted(board.values(), key=lambda e: -e["fitness"])
                top = top[:LEADERBOARD_SIZE]

                with self.lock:
                    self.generation = gen
                    self.history.append(
                        {"gen": gen, "best": round(best, 2), "mean": round(mean, 2)}
                    )
                    self.evals_per_sec = len(pop) / dt
                    self.leaderboard = [
                        {k: v for k, v in e.items() if not (k == "name" and v is None)}
                        for e in top
                    ]

                if gen == cfg["generations"] - 1:
                    break

                # Next generation: elitism + tournament/uniform-crossover/mutation.
                order = sorted(range(len(pop)), key=lambda i: -fits[i])
                nxt = [pop[i] for i in order[:ELITISM]]
                while len(nxt) < pop_size:
                    a = ga.tournament(pop, fits, rng, TOURNAMENT_K)
                    b = ga.tournament(pop, fits, rng, TOURNAMENT_K)
                    child = ga.crossover(a, b, rng)
                    nxt.append(ga.mutate(child, cfg["mutation_rate"], rng))
                pop = nxt
        except Exception:
            import traceback

            traceback.print_exc()
        finally:
            if pool is not None:
                pool.terminate()
                pool.join()
            with self.lock:
                self.status = "done"
                self.evals_per_sec = 0.0


# ------------------------------------------------------------------ routes


@app.post("/api/runs")
async def create_run(request: Request):
    try:
        body = await request.json()
    except Exception:
        body = {}
    cfg = {
        "population": int(body.get("population", 96)),
        "generations": int(body.get("generations", 60)),
        "mutation_rate": float(body.get("mutation_rate", 0.05)),
        "bbox": [int(v) for v in body.get("bbox", [5, 3, 3])],
        "eval_ticks": int(body.get("eval_ticks", 600)),
        "seed": int(body.get("seed", 42)),
    }
    with _runs_lock:
        run_id = f"run{len(_runs) + 1}"
        _runs[run_id] = Run(run_id, cfg)
    return {"id": run_id}


@app.get("/api/runs/{run_id}")
async def get_run(run_id: str):
    run = _runs.get(run_id)
    if run is None:
        return JSONResponse({"error": "run not found"}, status_code=404)
    return run.snapshot()


@app.post("/api/runs/{run_id}/stop")
async def stop_run(run_id: str):
    run = _runs.get(run_id)
    if run is None:
        return JSONResponse({"error": "run not found"}, status_code=404)
    run.stop_flag.set()
    return {"ok": True}


# ------------------------------------------------- frontend dist + fallback


@app.get("/{path:path}")
async def spa(path: str):
    target = (DIST / path) if path else DIST / "index.html"
    if path and target.is_file() and DIST in target.resolve().parents:
        return FileResponse(target)
    return FileResponse(DIST / "index.html")


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8440, log_level="warning")
