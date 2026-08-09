"""Shared runner for showcase pieces that regenerate via the existing
self-verifying scripts (they audit, net-check, simulate, bake and save)."""
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)


def regen(script, args, out_name, verify_re, expect_total=None):
    out = os.path.join(HERE, out_name)
    cmd = [sys.executable, os.path.join(ROOT, script)] + args + ["--out", out]
    print("+", " ".join(cmd), flush=True)
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    sys.stdout.write(r.stdout)
    sys.stderr.write(r.stderr)
    if r.returncode != 0:
        print("FAIL: %s exited %d" % (script, r.returncode))
        return 1
    m = re.search(verify_re, r.stdout)
    if not m or m.group(1) != m.group(2):
        print("FAIL: verification line missing or imperfect:", m and m.group(0))
        return 1
    if expect_total is not None and m.group(2) != str(expect_total):
        print("FAIL: expected %s cases, saw %s" % (expect_total, m.group(2)))
        return 1
    if not os.path.exists(out):
        print("FAIL: no output at", out)
        return 1
    print("PASS: %s -> %s (%s)" % (script, out_name, m.group(0)))
    return 0
