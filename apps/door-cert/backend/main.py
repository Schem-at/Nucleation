"""Door Certification Bureau — real backend.

Replaces the frontend's vite mock (frontend/mock/plugin.ts) at identical
routes, serving the built frontend dist/ with SPA fallback, and running the
real analysis pipeline on nucleation's TickSimulation (headless mc-tick).

Routes:
  POST /api/doors                    (multipart "file") -> { id }
  GET  /api/doors/:id                -> { status, step, error }
  GET  /api/doors/:id/certificate    -> certificate JSON
  GET  /api/doors/:id/animation.mp4  -> rendered video (range-aware)
  GET  /*                            -> frontend dist with SPA fallback

Run:  .venv/bin/python main.py          (binds 0.0.0.0:8430)
"""

from __future__ import annotations

import json
import re
import secrets
import subprocess
import threading
import time
import traceback
from pathlib import Path

from fastapi import FastAPI, Request, UploadFile
from fastapi.responses import FileResponse, JSONResponse, Response

import nucleation

HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parent.parent.parent  # .../Nucleation
JOBS = HERE / "jobs"
DIST = REPO_ROOT / "apps" / "door-cert" / "frontend" / "dist"
CLIENT_JAR = REPO_ROOT / "tools" / "gametest" / "work" / "client.jar"

SEED = 12345
ALLOWED_EXT = {".litematic", ".schem", ".schematic", ".snbt"}

JOBS.mkdir(exist_ok=True)

# id -> {"status": ..., "step": ..., "error": ...}
_state: dict[str, dict] = {}
_state_lock = threading.Lock()


def _set(job_id: str, status: str, step: str | None = None, error: str | None = None):
    doc = {"status": status, "step": step, "error": error}
    with _state_lock:
        _state[job_id] = doc
    (JOBS / job_id / "status.json").write_text(json.dumps(doc))


def _get(job_id: str) -> dict | None:
    with _state_lock:
        if job_id in _state:
            return _state[job_id]
    p = JOBS / job_id / "status.json"
    if p.exists():
        return json.loads(p.read_text())
    return None


# ---------------------------------------------------------------------------
# Analysis pipeline
# ---------------------------------------------------------------------------

def _camera_yaw(lever: tuple, dims: tuple) -> float:
    """Yaw that faces the side of the build the lever sits on."""
    lx, _ly, lz = lever
    w, _h, l = dims
    sides = {
        225.0: lz,            # north face (z = 0)
        45.0: l - 1 - lz,     # south face (z = max)
        315.0: lx,            # west face (x = 0)
        135.0: w - 1 - lx,    # east face (x = max)
    }
    return min(sides, key=sides.get)


def _base_name(state: str) -> str:
    return state.split("[", 1)[0]


def _snapshot_set(sim) -> set:
    snap = json.loads(sim.world_snapshot_json())
    return {(tuple(b["pos"]), b["state"]) for b in snap}


def _last_change_tick_after(sim, after_tick: int) -> int | None:
    ticks = [c["tick"] for c in json.loads(sim.changes_json()) if c["tick"] >= after_tick]
    return max(ticks) if ticks else None


def process(job_id: str, upload_path: Path, name: str) -> None:
    job_dir = JOBS / job_id
    timings = {}
    try:
        # -- parsing --------------------------------------------------------
        _set(job_id, "processing", "parsing")
        t0 = time.time()
        data = upload_path.read_bytes()
        snbt_path = job_dir / "door.snbt"
        schem = None
        if upload_path.suffix == ".snbt":
            # Vanilla gametest-style structure SNBT ("blocks:" flavor) is
            # parsed by mc-tick itself; Schematic.from_data only knows the
            # nucleation "data:" flavor, so go straight to the simulator.
            snbt_text = data.decode("utf-8")
            snbt_path.write_bytes(data)
            m = re.search(r"size:\s*\[([^\]]+)\]", snbt_text)
            if not m:
                _set(job_id, "error", None, "structure SNBT has no size field")
                return
            w, h, l = [int(re.sub(r"[^-\d]", "", v)) for v in m.group(1).split(",")]
        else:
            schem = nucleation.Schematic.from_data(data)
            dims = schem.dimensions()
            w, h, l = int(dims.x), int(dims.y), int(dims.z)
            snbt_path.write_text(nucleation.TickSimulation.gametest_snbt(schem))
        timings["parsing"] = time.time() - t0

        # -- simulating -----------------------------------------------------
        _set(job_id, "processing", "simulating")
        t0 = time.time()
        if schem is not None:
            sim = nucleation.TickSimulation.from_schematic(
                schem, nucleation.TickSettleMode.Placement, 0, 0, 0, ""
            )
        else:
            sim = nucleation.TickSimulation.from_snbt(
                snbt_text, nucleation.TickSettleMode.Placement, 0, 0, 0, ""
            )
        sim.set_rng_seed(SEED)

        initial_snapshot = json.loads(sim.world_snapshot_json())

        lever = None
        for b in initial_snapshot:
            if b["state"].startswith("minecraft:lever"):
                lever = tuple(b["pos"])
                break
        if lever is None:
            _set(job_id, "error", None, "no lever found")
            return
        lx, ly, lz = lever

        # Settle any placement cascade, then run one full conditioning cycle
        # (open + close) before taking the reference snapshot: a schematic can
        # be saved with its panels anywhere, and doors are certified against
        # their steady-state cycle, not the accident of how they were saved.
        sim.run_until_quiescent(200)
        t_cond_open = sim.tick_count()
        sim.use_block(lx, ly, lz)
        sim.run_until_quiescent(300)
        t_cond_close = sim.tick_count()
        sim.use_block(lx, ly, lz)
        sim.run_until_quiescent(300)
        snapshot_a = _snapshot_set(sim)

        # -- measuring ------------------------------------------------------
        _set(job_id, "processing", "measuring")
        t_open_click = sim.tick_count()
        sim.use_block(lx, ly, lz)
        sim.run_until_quiescent(300)
        last_open = _last_change_tick_after(sim, t_open_click)
        if last_open is None:
            _set(job_id, "error", None, "lever click caused no block changes")
            return
        open_ticks = last_open - t_open_click

        t_close_click = sim.tick_count()
        sim.use_block(lx, ly, lz)
        sim.run_until_quiescent(300)
        last_close = _last_change_tick_after(sim, t_close_click)
        close_ticks = (last_close - t_close_click) if last_close is not None else 0

        snapshot_final = _snapshot_set(sim)
        verdict = "CERTIFIED" if snapshot_final == snapshot_a else "DID NOT RESET"

        sim_ticks = sim.tick_count()
        changes = json.loads(sim.changes_json())
        summary = json.loads(sim.events_summary_json())
        timings["simulating"] = time.time() - t0

        # Stats: events per tick, zero-filled over the whole run.
        by_tick = {r["tick"]: r for r in summary}
        events_per_tick = [
            {
                "tick": t,
                "piston": by_tick.get(t, {}).get("piston", 0),
                "redstone": by_tick.get(t, {}).get("redstone", 0),
                "changes": by_tick.get(t, {}).get("changes", 0),
            }
            for t in range(sim_ticks)
        ]

        # Heatmap: per-(x,y) column counts of block changes across the run.
        values = [[0] * w for _ in range(h)]
        for c in changes:
            x, y, _z = c["pos"]
            if 0 <= x < w and 0 <= y < h:
                values[y][x] += 1

        # Materials: base-name counts from the initial snapshot, air excluded.
        counts: dict[str, int] = {}
        for b in initial_snapshot:
            base = _base_name(b["state"])
            if base.endswith("air"):
                continue
            counts[base] = counts.get(base, 0) + 1
        materials = [
            {"id": k, "count": v}
            for k, v in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))
        ]

        certificate = {
            "name": name,
            "dims": [w, h, l],
            "lever": [lx, ly, lz],
            "open_ticks": open_ticks,
            "close_ticks": close_ticks,
            "materials": materials,
            "events_per_tick": events_per_tick,
            "heatmap": {"w": w, "h": h, "values": values},
            "animation_url": f"/api/doors/{job_id}/animation.mp4",
            "sim_ticks": sim_ticks,
            "seed": SEED,
            "verdict": verdict,
        }
        (job_dir / "certificate.json").write_text(json.dumps(certificate, indent=1))

        # -- rendering ------------------------------------------------------
        _set(job_id, "processing", "rendering")
        t0 = time.time()
        total = sim_ticks + 20
        cmd = [
            "cargo", "run", "--release", "--example", "render_simulation_video",
            "--features", "rendering", "--",
            str(CLIENT_JAR), str(snbt_path), str(job_dir / "animation.mp4"),
            "--settle",
            "--ticks", str(total),
            # The measurement ran a conditioning cycle before the measured one;
            # the render must replay the SAME click schedule or its world
            # diverges and the animation shows a different door.
            "--click", f"{lx},{ly},{lz}@{t_cond_open}",
            "--click", f"{lx},{ly},{lz}@{t_cond_close}",
            "--click", f"{lx},{ly},{lz}@{t_open_click}",
            "--click", f"{lx},{ly},{lz}@{t_close_click}",
            "--frames-per-tick", "6",
            "--fps", "30",
            # Camera on the lever's side of the build: the lever marks the
            # door's front face, and the default yaw (45°, looking at the
            # south+east faces) hides a north- or west-facing doorway — the
            # whole animation then plays on the far side of the build.
            "--yaw", str(_camera_yaw(lever, (w, h, l))),
            "--zoom", "1.5",
        ]
        proc = subprocess.run(
            cmd, cwd=REPO_ROOT, capture_output=True, text=True, timeout=600
        )
        timings["rendering"] = time.time() - t0
        if proc.returncode != 0 or not (job_dir / "animation.mp4").exists():
            tail = (proc.stderr or "")[-500:]
            _set(job_id, "error", None, f"render failed: {tail}")
            return

        (job_dir / "timings.json").write_text(json.dumps(timings, indent=1))
        _set(job_id, "done")
    except Exception as exc:  # noqa: BLE001
        (job_dir / "traceback.txt").write_text(traceback.format_exc())
        _set(job_id, "error", None, str(exc) or exc.__class__.__name__)


# ---------------------------------------------------------------------------
# HTTP
# ---------------------------------------------------------------------------

app = FastAPI()


@app.post("/api/doors")
async def upload_door(file: UploadFile):
    filename = file.filename or "door"
    suffix = Path(filename).suffix.lower()
    if suffix not in ALLOWED_EXT:
        return JSONResponse({"error": f"unsupported file type {suffix!r}"}, status_code=400)
    job_id = secrets.token_hex(4)
    job_dir = JOBS / job_id
    job_dir.mkdir(parents=True)
    upload_path = job_dir / f"upload{suffix}"
    upload_path.write_bytes(await file.read())
    name = Path(filename).stem
    _set(job_id, "processing", "parsing")
    threading.Thread(
        target=process, args=(job_id, upload_path, name), daemon=True
    ).start()
    return {"id": job_id}


@app.get("/api/doors/{job_id}")
def door_status(job_id: str):
    doc = _get(job_id)
    if doc is None:
        return JSONResponse({"error": "not found"}, status_code=404)
    return doc


@app.get("/api/doors/{job_id}/certificate")
def door_certificate(job_id: str):
    p = JOBS / job_id / "certificate.json"
    doc = _get(job_id)
    if doc is None or not p.exists() or doc["status"] != "done":
        return JSONResponse({"error": "not found"}, status_code=404)
    body = json.loads(p.read_text())
    body["animation_url"] = f"/api/doors/{job_id}/animation.mp4"
    return body


@app.get("/api/doors/{job_id}/animation.mp4")
def door_animation(job_id: str, request: Request):
    p = JOBS / job_id / "animation.mp4"
    if not p.exists():
        return JSONResponse({"error": "not found"}, status_code=404)
    size = p.stat().st_size
    range_header = request.headers.get("range")
    if range_header:
        m = re.match(r"bytes=(\d*)-(\d*)", range_header)
        start = int(m.group(1)) if m and m.group(1) else 0
        end = int(m.group(2)) if m and m.group(2) else size - 1
        end = min(end, size - 1)
        with p.open("rb") as fh:
            fh.seek(start)
            chunk = fh.read(end - start + 1)
        return Response(
            chunk,
            status_code=206,
            media_type="video/mp4",
            headers={
                "Content-Range": f"bytes {start}-{end}/{size}",
                "Accept-Ranges": "bytes",
            },
        )
    return FileResponse(p, media_type="video/mp4", headers={"Accept-Ranges": "bytes"})


@app.get("/api/{rest:path}")
def api_not_found(rest: str):
    return JSONResponse({"error": "not found"}, status_code=404)


# Frontend: static dist with SPA fallback (so /door/:id deep links work).
@app.get("/{path:path}")
def spa(path: str):
    candidate = (DIST / path).resolve()
    if path and candidate.is_relative_to(DIST) and candidate.is_file():
        return FileResponse(candidate)
    return FileResponse(DIST / "index.html")


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8430)
