#!/usr/bin/env python
"""Run every README example; exit non-zero if any of them fails.

    ~/eda-venv/bin/python redstone-eda/docs/examples/run_all.py

Each example is self-checking (it asserts its own expected values), so a green
run here is the guarantee that every snippet quoted in the README still works.
"""
import glob
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def main() -> int:
    files = sorted(f for f in glob.glob(os.path.join(HERE, "*.py"))
                   if os.path.basename(f)[0].isdigit())
    failed = []
    for path in files:
        name = os.path.basename(path)
        r = subprocess.run([sys.executable, path], cwd=HERE,
                           capture_output=True, text=True)
        last = (r.stdout.strip().splitlines() or ["<no output>"])[-1]
        if r.returncode == 0:
            print("PASS %-24s %s" % (name, last))
        else:
            err = (r.stderr.strip().splitlines() or ["<no stderr>"])[-1]
            print("FAIL %-24s %s" % (name, err))
            failed.append(name)

    print("\n%d/%d examples passed" % (len(files) - len(failed), len(files)))
    if failed:
        print("failed: " + ", ".join(failed))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
