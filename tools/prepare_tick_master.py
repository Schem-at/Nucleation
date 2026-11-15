#!/usr/bin/env python3
"""Overlay only the common benchmark/dev dependency onto a clean master checkout.

No production source is copied. Use a new detached worktree, then build with
--cfg bench_master --check-cfg=cfg(bench_master). The missing unrecorded API is
excluded from master comparisons rather than adding an optimization to master.
"""
import argparse
import hashlib
import json
import pathlib
import shutil
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--repo', type=pathlib.Path, required=True)
p.add_argument('--commit', required=True)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args()
source = pathlib.Path(__file__).resolve().parents[1]
destination = a.repo.resolve()


def git(*args):
    return subprocess.check_output(['git', '-C', str(destination), *args], text=True).strip()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


assert git('rev-parse', 'HEAD') == a.commit
assert not git('status', '--porcelain'), 'Use a clean isolated worktree'
before = {str(f.relative_to(destination)): sha(f)
          for crate in ['mc-tick', 'mc-test']
          for f in (destination / f'crates/{crate}/src').rglob('*.rs')}
files = ['Cargo.lock', 'crates/mc-tick/examples/bench_circuits.rs']
files += [str(f.relative_to(source)) for f in
          sorted((source / 'crates/mc-tick/examples/bench_circuits').rglob('*.rs'))]
for name in files:
    target = destination / name
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source / name, target)
cargo = destination / 'crates/mc-tick/Cargo.toml'
text = cargo.read_text()
assert 'cpu-time' not in text
cargo.write_text(text.replace('[dev-dependencies]\n', '[dev-dependencies]\ncpu-time = "=1.0.0"\n', 1))
files.append('crates/mc-tick/Cargo.toml')
assert all(sha(destination / name) == value for name, value in before.items())
assert not git('diff', 'HEAD', '--', 'crates/mc-tick/src', 'crates/mc-test/src')
a.output.parent.mkdir(parents=True, exist_ok=True)
a.output.write_text(json.dumps({
    'master_commit': a.commit,
    'source_commit': subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip(),
    'copied_or_modified': {name: sha(destination / name) for name in files},
    'production_source_sha256': before,
    'production_source_unchanged': True,
    'compatibility_flags': '--cfg bench_master --check-cfg=cfg(bench_master)',
    'unsupported_case': 'hipster_unrecorded',
}, indent=2) + '\n')
print('Prepared identical harness; master production source unchanged:', destination)
