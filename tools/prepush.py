#!/usr/bin/env python3
"""Nucleation Pre-Push Verification — Rich TUI edition.

Runs compilation checks, tests, WASM builds, and benchmarks in parallel lanes
with a flicker-free Rich progress display.

Usage:
    python3 tools/prepush.py [OPTIONS]
    ./pre-push.sh [OPTIONS]          # thin wrapper

Options:
    --skip-bench        Skip benchmark lane
    --bench-only        Only run benchmarks
    --update-baseline   Force-record current benchmark results
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

# ── Rich dependency check ────────────────────────────────────────────────────

try:
    from rich.console import Console
    from rich.live import Live
    from rich.panel import Panel
    from rich.progress import (
        BarColumn,
        MofNCompleteColumn,
        Progress,
        SpinnerColumn,
        TextColumn,
        TimeElapsedColumn,
    )
    from rich.table import Table
    from rich.text import Text
except ImportError:
    print(
        "ERROR: 'rich' library is required.\n"
        "  pip install rich\n"
        "  or: pip install -r requirements-dev.txt",
        file=sys.stderr,
    )
    sys.exit(1)

# ── Project root ─────────────────────────────────────────────────────────────

ROOT = Path(__file__).resolve().parent.parent
os.chdir(ROOT)

# ── Configuration ────────────────────────────────────────────────────────────

BENCH_WARN_PCT = 15
BENCH_FAIL_PCT = 50
BASELINE_DIR = ROOT / ".bench-baselines"
BASELINE_FILE = BASELINE_DIR / "history.json"

console = Console()

# ── Data classes ─────────────────────────────────────────────────────────────


@dataclass
class Check:
    name: str
    command: list[str]
    status: str = "pending"  # pending | running | passed | failed | warned | skipped
    elapsed: float = 0.0
    output: str = ""


@dataclass
class Lane:
    name: str
    color: str
    checks: list[Check] = field(default_factory=list)
    elapsed: float = 0.0
    failed: bool = False


@dataclass
class BenchComparison:
    name: str
    mean_ns: float
    base_ns: Optional[float] = None
    pct_change: Optional[float] = None
    status: str = "new"  # pass | warn | fail | new


# ── Helpers ──────────────────────────────────────────────────────────────────


def cargo_version() -> str:
    with open(ROOT / "Cargo.toml") as f:
        for line in f:
            m = re.match(r'^version\s*=\s*"([^"]+)"', line)
            if m:
                return m.group(1)
    return "unknown"


def pyproject_version() -> str:
    path = ROOT / "pyproject.toml"
    if not path.exists():
        return "unknown"
    with open(path) as f:
        for line in f:
            m = re.match(r'^version\s*=\s*"([^"]+)"', line)
            if m:
                return m.group(1)
    return "unknown"


def fmt_ns(ns: float) -> str:
    if ns < 1_000:
        return f"{ns:.0f}ns"
    if ns < 1_000_000:
        return f"{ns / 1_000:.1f}\u00b5s"
    if ns < 1_000_000_000:
        return f"{ns / 1_000_000:.1f}ms"
    return f"{ns / 1_000_000_000:.2f}s"


def fmt_secs(s: float) -> str:
    return f"{s:.1f}s"


# ── Gate: format check with auto-fix ─────────────────────────────────────────


class GateError(Exception):
    pass


def run_gate() -> tuple[str, float]:
    """Run cargo fmt --check; auto-fix if needed. Returns (status, elapsed)."""
    start = time.monotonic()
    result = subprocess.run(
        ["cargo", "fmt", "--", "--check"],
        capture_output=True,
        text=True,
        cwd=ROOT,
    )
    if result.returncode == 0:
        return "ok", time.monotonic() - start

    # Auto-fix
    fix = subprocess.run(
        ["cargo", "fmt"],
        capture_output=True,
        text=True,
        cwd=ROOT,
    )
    if fix.returncode != 0:
        raise GateError(f"cargo fmt failed:\n{fix.stderr}")

    # Re-verify
    recheck = subprocess.run(
        ["cargo", "fmt", "--", "--check"],
        capture_output=True,
        text=True,
        cwd=ROOT,
    )
    if recheck.returncode != 0:
        raise GateError("cargo fmt --check still fails after auto-fix")

    return "auto-fixed", time.monotonic() - start


# ── Lane execution ───────────────────────────────────────────────────────────


def run_check(check: Check) -> bool:
    """Run a single check, updating its fields in place. Returns True on success."""
    check.status = "running"
    start = time.monotonic()
    try:
        result = subprocess.run(
            check.command,
            capture_output=True,
            text=True,
            cwd=ROOT,
            timeout=600,
        )
        check.elapsed = time.monotonic() - start
        check.output = result.stdout + result.stderr
        if result.returncode == 0:
            check.status = "passed"
            return True
        else:
            check.status = "failed"
            return False
    except subprocess.TimeoutExpired:
        check.elapsed = time.monotonic() - start
        check.status = "failed"
        check.output = "TIMEOUT (10 min)"
        return False
    except Exception as e:
        check.elapsed = time.monotonic() - start
        check.status = "failed"
        check.output = str(e)
        return False


def run_lane(lane: Lane, progress: Progress, task_id) -> None:
    """Execute all checks in a lane sequentially, updating the progress bar."""
    lane_start = time.monotonic()
    for check in lane.checks:
        progress.update(task_id, description=f"[dim]{check.name}[/dim]")
        ok = run_check(check)
        progress.advance(task_id)
        if not ok:
            lane.failed = True
            # Mark remaining checks as skipped
            for remaining in lane.checks:
                if remaining.status == "pending":
                    remaining.status = "skipped"
            break
    lane.elapsed = time.monotonic() - lane_start
    progress.update(
        task_id,
        description="[green]\u2713 Done[/green]" if not lane.failed else "[red]\u2717 Failed[/red]",
    )


# ── Lane definitions ─────────────────────────────────────────────────────────


def build_native_lane() -> Lane:
    lane = Lane(name="Native", color="cyan")
    lane.checks = [
        Check("cargo check (default)", ["cargo", "check"]),
        Check("cargo check (simulation)", ["cargo", "check", "--features", "simulation"]),
        Check("cargo check (ffi+meshing)", ["cargo", "check", "--features", "ffi,meshing"]),
        Check("cargo check (ffi+simulation)", ["cargo", "check", "--features", "ffi,simulation"]),
        Check("cargo check (python+simulation)", ["cargo", "check", "--features", "python,simulation"]),
        Check("cargo check (python+meshing)", ["cargo", "check", "--features", "python,meshing"]),
        Check("cargo test (default)", ["cargo", "test"]),
        Check("cargo test (simulation)", ["cargo", "test", "--features", "simulation"]),
        Check(
            "cargo test (insign IO)",
            ["cargo", "test", "--lib", "--features", "simulation", "typed_executor::insign_io"],
        ),
    ]
    if shutil.which("maturin"):
        lane.checks.append(
            Check("maturin build", ["maturin", "build", "--features", "python,simulation"])
        )
    return lane


def build_wasm_lane() -> Lane:
    lane = Lane(name="WASM", color="magenta")
    lane.checks = [
        Check(
            "cargo check (wasm+simulation)",
            ["cargo", "check", "--target", "wasm32-unknown-unknown", "--features", "wasm,simulation"],
        ),
    ]
    if (ROOT / "build-wasm.sh").exists():
        lane.checks.append(Check("build-wasm.sh", ["./build-wasm.sh"]))
    if (ROOT / "tests" / "node_wasm_test.js").exists():
        lane.checks.append(Check("node WASM tests", ["node", "tests/node_wasm_test.js"]))
    return lane


def build_quick_lane() -> Lane:
    """Version consistency + API parity — no subprocess for version check."""
    lane = Lane(name="Quick", color="yellow")

    # Version consistency is handled as a pseudo-check (see _run_quick_lane)
    lane.checks = [
        Check("version consistency", []),  # special: no command
        Check("API parity", []),  # special: no command
    ]
    return lane


def _run_quick_lane(lane: Lane, progress: Progress, task_id) -> None:
    """Custom execution for Quick lane (version + parity are not simple subprocesses)."""
    lane_start = time.monotonic()

    # ── Version consistency ────────────────────────────────────────────
    vc = lane.checks[0]
    vc.status = "running"
    progress.update(task_id, description=f"[dim]{vc.name}[/dim]")
    start = time.monotonic()
    cv = cargo_version()
    pv = pyproject_version()
    vc.elapsed = time.monotonic() - start
    if cv == pv:
        vc.status = "passed"
        vc.name = f"version consistency ({cv})"
    else:
        vc.status = "failed"
        vc.name = f"version mismatch ({cv} vs {pv})"
        vc.output = f"Cargo.toml={cv}  pyproject.toml={pv}"
        lane.failed = True
    progress.advance(task_id)

    if lane.failed:
        for remaining in lane.checks[1:]:
            remaining.status = "skipped"
        lane.elapsed = time.monotonic() - lane_start
        progress.update(task_id, description="[red]\u2717 Failed[/red]")
        return

    # ── API parity ─────────────────────────────────────────────────────
    ap = lane.checks[1]
    ap.status = "running"
    progress.update(task_id, description=f"[dim]{ap.name}[/dim]")
    start = time.monotonic()

    parity_bin = ROOT / "target" / "check_api_parity"
    parity_src = ROOT / "tools" / "check_api_parity.rs"

    # Compile if needed
    if parity_src.exists():
        needs_compile = (
            not parity_bin.exists()
            or parity_src.stat().st_mtime > parity_bin.stat().st_mtime
        )
        if needs_compile:
            subprocess.run(
                ["rustc", str(parity_src), "-o", str(parity_bin)],
                capture_output=True,
                cwd=ROOT,
            )

    if parity_bin.exists():
        result = subprocess.run(
            [str(parity_bin)],
            capture_output=True,
            text=True,
            cwd=ROOT,
        )
        ap.elapsed = time.monotonic() - start
        ap.output = result.stdout + result.stderr
        if result.returncode == 0:
            ap.status = "passed"
        else:
            ap.status = "failed"
            lane.failed = True
    else:
        ap.elapsed = time.monotonic() - start
        ap.status = "warned"
        ap.name = "API parity (compiler unavailable)"

    progress.advance(task_id)
    lane.elapsed = time.monotonic() - lane_start
    if lane.failed:
        progress.update(task_id, description="[red]\u2717 Failed[/red]")
    else:
        progress.update(task_id, description="[green]\u2713 Done[/green]")


# ── Benchmark system ─────────────────────────────────────────────────────────


def build_bench_lane() -> Lane:
    lane = Lane(name="Bench", color="blue")
    lane.checks = [
        Check("cargo bench (snapshot)", ["cargo", "bench", "--bench", "snapshot_bench"]),
        Check("cargo bench (region)", ["cargo", "bench", "--bench", "region_bench"]),
    ]
    return lane


def extract_bench_results() -> list[dict]:
    """Walk target/criterion/ and extract mean.point_estimate from estimates.json."""
    criterion_dir = ROOT / "target" / "criterion"
    if not criterion_dir.is_dir():
        return []

    results = []
    for est_file in criterion_dir.rglob("new/estimates.json"):
        parts = est_file.parts
        try:
            crit_idx = parts.index("criterion")
            new_idx = parts.index("new")
        except ValueError:
            continue
        bench_parts = parts[crit_idx + 1 : new_idx]
        if "report" in bench_parts:
            continue
        bench_name = "/".join(bench_parts)

        try:
            with open(est_file) as f:
                data = json.load(f)
            mean_ns = data["mean"]["point_estimate"]
            results.append({"name": bench_name, "mean_ns": round(mean_ns, 1)})
        except (KeyError, json.JSONDecodeError):
            pass

    return results


def compare_benchmarks(current: list[dict]) -> Optional[dict]:
    """Compare current results against baseline. Returns None if no baseline."""
    if not BASELINE_FILE.exists():
        return None

    try:
        with open(BASELINE_FILE) as f:
            history = json.load(f)
    except (json.JSONDecodeError, OSError):
        return None

    if not history:
        return None

    cv = cargo_version()
    baseline = history[-1]
    if baseline.get("version") == cv and len(history) > 1:
        baseline = history[-2]

    base_benchmarks = baseline.get("benchmarks", {})
    if not base_benchmarks:
        return None

    comparisons: list[BenchComparison] = []
    has_warn = False
    has_fail = False

    for bench in current:
        name = bench["name"]
        mean_ns = bench["mean_ns"]
        if name in base_benchmarks:
            base_ns = base_benchmarks[name]
            pct = ((mean_ns - base_ns) / base_ns * 100) if base_ns > 0 else 0.0
            status = "pass"
            if pct > BENCH_FAIL_PCT:
                status = "fail"
                has_fail = True
            elif pct > BENCH_WARN_PCT:
                status = "warn"
                has_warn = True
            comparisons.append(BenchComparison(name, mean_ns, base_ns, round(pct, 1), status))
        else:
            comparisons.append(BenchComparison(name, mean_ns))

    return {
        "results": comparisons,
        "has_warn": has_warn,
        "has_fail": has_fail,
        "baseline_version": baseline.get("version", "?"),
    }


def update_baseline(bench_results: list[dict]) -> str:
    """Update or append to history.json. Returns a status message."""
    BASELINE_DIR.mkdir(parents=True, exist_ok=True)

    cv = cargo_version()
    commit = "unknown"
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            cwd=ROOT,
        )
        if result.returncode == 0:
            commit = result.stdout.strip()
    except Exception:
        pass

    try:
        with open(BASELINE_FILE) as f:
            history = json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        history = []

    benchmarks = {r["name"]: r["mean_ns"] for r in bench_results}
    from datetime import datetime, timezone

    new_entry = {
        "version": cv,
        "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "commit": commit,
        "benchmarks": benchmarks,
    }

    existing_idx = None
    for i, entry in enumerate(history):
        if entry.get("version") == cv:
            existing_idx = i
            break

    if existing_idx is not None:
        history[existing_idx] = new_entry
    else:
        history.append(new_entry)

    with open(BASELINE_FILE, "w") as f:
        json.dump(history, f, indent=2)
        f.write("\n")

    return f"Baseline updated for v{cv} ({len(benchmarks)} benchmarks)"


def _run_bench_lane(
    lane: Lane,
    progress: Progress,
    task_id,
    flag_update_baseline: bool,
) -> None:
    """Execute bench lane: run benchmarks, extract, compare, update baseline."""
    lane_start = time.monotonic()

    # Run the actual benchmark checks
    for check in lane.checks:
        progress.update(task_id, description=f"[dim]{check.name}[/dim]")
        ok = run_check(check)
        progress.advance(task_id)
        if not ok:
            lane.failed = True
            for remaining in lane.checks:
                if remaining.status == "pending":
                    remaining.status = "skipped"
            break

    if not lane.failed:
        # Extract + compare + possibly update baseline
        bench_results = extract_bench_results()
        comparison = compare_benchmarks(bench_results)

        # Store on lane for summary display
        lane._bench_results = bench_results  # type: ignore[attr-defined]
        lane._bench_comparison = comparison  # type: ignore[attr-defined]

        # Check for regressions
        if comparison and comparison["has_fail"]:
            lane.failed = True

        # Auto-record on version bump
        cv = cargo_version()
        if BASELINE_FILE.exists():
            try:
                with open(BASELINE_FILE) as f:
                    history = json.load(f)
                last_ver = history[-1]["version"] if history else ""
            except (json.JSONDecodeError, OSError, KeyError, IndexError):
                last_ver = ""
            if cv != last_ver:
                update_baseline(bench_results)
        else:
            update_baseline(bench_results)

        if flag_update_baseline:
            update_baseline(bench_results)

    lane.elapsed = time.monotonic() - lane_start
    progress.update(
        task_id,
        description="[green]\u2713 Done[/green]" if not lane.failed else "[red]\u2717 Failed[/red]",
    )


# ── Summary display ──────────────────────────────────────────────────────────

SYM_PASS = "\u2713"
SYM_FAIL = "\u2717"
SYM_WARN = "\u26a0"
SYM_NEW = "\u25cb"
SYM_SKIP = "\u2500"


def _status_symbol(status: str) -> str:
    return {
        "passed": f"[green]{SYM_PASS}[/green]",
        "failed": f"[red]{SYM_FAIL}[/red]",
        "warned": f"[yellow]{SYM_WARN}[/yellow]",
        "skipped": f"[dim]{SYM_SKIP}[/dim]",
    }.get(status, "?")


def print_lane_panel(lane: Lane) -> Panel:
    """Build a Rich Panel for one lane's results."""
    table = Table(show_header=False, box=None, padding=(0, 1), expand=True)
    table.add_column("sym", width=2, no_wrap=True)
    table.add_column("name", ratio=1)
    table.add_column("time", justify="right", width=8, no_wrap=True)

    for check in lane.checks:
        sym = _status_symbol(check.status)
        t = fmt_secs(check.elapsed) if check.status not in ("pending", "skipped") else ""
        table.add_row(sym, check.name, f"[dim]{t}[/dim]")

    border = "red" if lane.failed else "green"
    return Panel(
        table,
        title=f"[bold {lane.color}]{lane.name}[/bold {lane.color}]",
        subtitle=f"[dim]{fmt_secs(lane.elapsed)}[/dim]",
        border_style=border,
        expand=True,
    )


def print_bench_panel(lane: Lane) -> Optional[Panel]:
    """Build a Rich Panel for benchmark comparison results."""
    bench_results = getattr(lane, "_bench_results", None)
    comparison = getattr(lane, "_bench_comparison", None)

    if bench_results is None:
        return None

    table = Table(show_header=False, box=None, padding=(0, 1), expand=True)
    table.add_column("sym", width=2, no_wrap=True)
    table.add_column("name", ratio=1)
    table.add_column("time", width=10, justify="right", no_wrap=True)
    table.add_column("vs_base", width=28, justify="right", no_wrap=True)

    subtitle = "[dim]No baseline[/dim]"

    if comparison is None:
        # No baseline — just show raw results
        for r in sorted(bench_results, key=lambda x: x["name"]):
            table.add_row(
                f"[dim]{SYM_NEW}[/dim]",
                r["name"],
                fmt_ns(r["mean_ns"]),
                "[dim](new)[/dim]",
            )
    else:
        base_ver = comparison["baseline_version"]
        subtitle = f"[dim]vs v{base_ver}[/dim]"
        for bc in sorted(comparison["results"], key=lambda x: x.name):
            if bc.base_ns is not None and bc.pct_change is not None:
                sign = "+" if bc.pct_change >= 0 else ""
                if bc.status == "fail":
                    sym = f"[red]{SYM_FAIL}[/red]"
                    pct_str = f"[red]{sign}{bc.pct_change:.1f}%[/red]"
                elif bc.status == "warn":
                    sym = f"[yellow]{SYM_WARN}[/yellow]"
                    pct_str = f"[yellow]{sign}{bc.pct_change:.1f}%[/yellow]"
                else:
                    sym = f"[green]{SYM_PASS}[/green]"
                    pct_str = f"[dim]{sign}{bc.pct_change:.1f}%[/dim]"
                vs = f"[dim](base {fmt_ns(bc.base_ns):<8s}[/dim] {pct_str}[dim])[/dim]"
                table.add_row(sym, bc.name, fmt_ns(bc.mean_ns), vs)
            else:
                table.add_row(f"[dim]{SYM_NEW}[/dim]", bc.name, fmt_ns(bc.mean_ns), "[dim](new)[/dim]")

    border = "red" if lane.failed else ("yellow" if comparison and comparison["has_warn"] else "green")
    return Panel(
        table,
        title="[bold blue]Benchmarks[/bold blue]",
        subtitle=subtitle,
        border_style=border,
        expand=True,
    )


def print_summary(
    lanes: list[Lane],
    bench_lane: Optional[Lane],
    gate_status: str,
    gate_elapsed: float,
    total_elapsed: float,
) -> bool:
    """Print the detailed summary. Returns True if everything passed."""
    console.print()

    # Lane detail panels
    for lane in lanes:
        console.print(print_lane_panel(lane))
        console.print()

    # Bench panel
    if bench_lane is not None:
        panel = print_bench_panel(bench_lane)
        if panel:
            console.print(panel)
            console.print()

    # Totals
    total_pass = sum(1 for l in lanes for c in l.checks if c.status == "passed")
    total_fail = sum(1 for l in lanes for c in l.checks if c.status == "failed")
    total_warn = sum(1 for l in lanes for c in l.checks if c.status == "warned")

    bench_ok = True
    bench_summary = ""
    if bench_lane is not None:
        comparison = getattr(bench_lane, "_bench_comparison", None)
        if comparison:
            n_warn = sum(1 for bc in comparison["results"] if bc.status == "warn")
            n_fail = sum(1 for bc in comparison["results"] if bc.status == "fail")
            if n_fail:
                bench_summary = f"[red]{n_fail} regressions[/red]"
                bench_ok = False
            elif n_warn:
                bench_summary = f"[yellow]{n_warn} warnings[/yellow]"
            else:
                bench_summary = "[green]ok[/green]"
        else:
            bench_summary = "[dim]no baseline[/dim]"

        if bench_lane.failed:
            bench_ok = False

    all_passed = total_fail == 0 and bench_ok

    # Build status line
    parts = [f"Total: {fmt_secs(total_elapsed)}"]
    checks_str = f"Checks: [green]{total_pass} passed[/green]"
    if total_fail:
        checks_str = f"Checks: [red]{total_fail} failed[/red]  [green]{total_pass} passed[/green]"
    if total_warn:
        checks_str += f"  [yellow]{total_warn} warnings[/yellow]"
    parts.append(checks_str)
    if bench_summary:
        parts.append(f"Bench: {bench_summary}")

    console.print(f"  {'   '.join(parts)}")

    if all_passed:
        console.print(f"  [bold green]{SYM_PASS} Ready to push[/bold green]")
    else:
        console.print(f"  [bold red]{SYM_FAIL} Fix issues before pushing[/bold red]")

        # Show first failure output for debugging
        for lane in lanes + ([bench_lane] if bench_lane else []):
            for check in lane.checks:
                if check.status == "failed" and check.output.strip():
                    console.print()
                    console.print(
                        Panel(
                            check.output[-2000:] if len(check.output) > 2000 else check.output,
                            title=f"[red]Failed: {check.name}[/red]",
                            border_style="red",
                            expand=True,
                        )
                    )
                    break
            else:
                continue
            break

    console.print()
    return all_passed


# ── Main ─────────────────────────────────────────────────────────────────────


def main() -> int:
    parser = argparse.ArgumentParser(description="Nucleation Pre-Push Verification")
    parser.add_argument("--skip-bench", action="store_true", help="Skip benchmark lane")
    parser.add_argument("--bench-only", action="store_true", help="Only run benchmarks")
    parser.add_argument("--update-baseline", action="store_true", help="Force-record benchmark baseline")
    args = parser.parse_args()

    total_start = time.monotonic()

    # ── Gate: format check ────────────────────────────────────────────
    gate_status = "skipped"
    gate_elapsed = 0.0

    if not args.bench_only:
        try:
            gate_status, gate_elapsed = run_gate()
        except GateError as e:
            console.print()
            console.print(
                Panel(
                    str(e),
                    title="[bold red]Format Gate Failed[/bold red]",
                    border_style="red",
                )
            )
            return 1

    # Show gate result
    console.print()
    console.print("[bold]  Nucleation Pre-Push Verification[/bold]")
    console.print()
    if gate_status == "ok":
        console.print(f"  [green]{SYM_PASS}[/green] Format check [dim]{fmt_secs(gate_elapsed)}[/dim]")
    elif gate_status == "auto-fixed":
        console.print(
            f"  [green]{SYM_PASS}[/green] Format check [yellow](auto-fixed)[/yellow] [dim]{fmt_secs(gate_elapsed)}[/dim]"
        )
    console.print()

    # ── Build lanes ───────────────────────────────────────────────────
    lanes: list[Lane] = []
    bench_lane: Optional[Lane] = None

    if not args.bench_only:
        lanes.append(build_native_lane())
        lanes.append(build_wasm_lane())
        lanes.append(build_quick_lane())

    if not args.skip_bench:
        bench_lane = build_bench_lane()

    all_lanes = lanes + ([bench_lane] if bench_lane else [])

    if not all_lanes:
        console.print("  Nothing to run.")
        return 0

    # ── Execute with Rich progress ────────────────────────────────────
    progress = Progress(
        SpinnerColumn(),
        TextColumn("[bold {task.fields[color]}]{task.fields[lane_name]:<7}[/bold {task.fields[color]}]"),
        BarColumn(bar_width=30),
        MofNCompleteColumn(),
        TextColumn("{task.description}"),
        TimeElapsedColumn(),
        console=console,
        expand=False,
    )

    # Create progress tasks for each lane
    task_ids = {}
    for lane in all_lanes:
        tid = progress.add_task(
            "",
            total=len(lane.checks),
            lane_name=lane.name,
            color=lane.color,
        )
        task_ids[lane.name] = tid

    def _execute_lane(lane: Lane) -> None:
        tid = task_ids[lane.name]
        if lane.name == "Quick":
            _run_quick_lane(lane, progress, tid)
        elif lane.name == "Bench":
            _run_bench_lane(lane, progress, tid, args.update_baseline)
        else:
            run_lane(lane, progress, tid)

    with Live(progress, console=console, refresh_per_second=10):
        with ThreadPoolExecutor(max_workers=len(all_lanes)) as pool:
            futures = [pool.submit(_execute_lane, lane) for lane in all_lanes]
            for future in futures:
                future.result()

    # ── Summary ───────────────────────────────────────────────────────
    total_elapsed = time.monotonic() - total_start
    ok = print_summary(lanes, bench_lane, gate_status, gate_elapsed, total_elapsed)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
