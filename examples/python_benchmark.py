#!/usr/bin/env python3
"""Compatibility entry point for the maintained Python comparison benchmark.

Prefer ``python benches/bench_python.py``. This wrapper keeps the historical
example command working without maintaining a second, divergent benchmark.
"""

from __future__ import annotations

import runpy
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
runpy.run_path(str(ROOT / "benches" / "bench_python.py"), run_name="__main__")
