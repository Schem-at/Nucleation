#!/usr/bin/env python3
"""Build isolated feature candidates and save validation evidence on the benchmark host."""
import argparse, hashlib, json, os, pathlib, shutil, subprocess, tarfile

FEATURES = {
    'reference': [], 'directional': ['experiment-directional'],
    'cursor': ['experiment-cursor'], 'pool': ['experiment-update-pool'],
    'dispatch': ['experiment-wire-dispatch'], 'plan': ['experiment-wire-plan'],
    'certificate': ['experiment-wire-certificate'],
    'counters': ['experiment-wire-certificate', 'experiment-wire-plan-counters'],
}
FEATURES['combined'] = list(dict.fromkeys(f for name, fs in FEATURES.items()
                                         if name != 'counters' for f in fs))
FEATURES['plan_pool'] = ['experiment-wire-plan', 'experiment-update-pool']
FEATURES['plan_pool_directional'] = FEATURES['plan_pool'] + ['experiment-directional']
FEATURES['certificate_pool_directional'] = ['experiment-wire-certificate', 'experiment-update-pool', 'experiment-directional']
FEATURES['all_no_dispatch'] = [f for f in FEATURES['combined'] if f != 'experiment-wire-dispatch']
FEATURES['all_no_cursor'] = [f for f in FEATURES['combined'] if f != 'experiment-cursor']
FEATURES['all_no_certificate'] = [f for f in FEATURES['combined'] if f != 'experiment-wire-certificate']
FEATURES['owner_selected'] = FEATURES['plan_pool_directional'] + ['experiment-world-plan-cache']
FEATURES['owner_combined'] = FEATURES['combined'] + ['experiment-world-plan-cache']
FEATURES['owner_counters'] = FEATURES['counters'] + ['experiment-world-plan-cache']
FEATURES['next_reference'] = FEATURES['owner_selected']
FEATURES['next_view'] = FEATURES['owner_selected'] + ['experiment-execution-view']
FEATURES['next_arena'] = FEATURES['owner_selected'] + ['experiment-update-arena']
FEATURES['next_piston'] = FEATURES['owner_selected'] + ['experiment-piston-plan']
FEATURES['next_lazy'] = FEATURES['owner_selected'] + ['experiment-lazy-journal']
FEATURES['next_combined'] = FEATURES['owner_selected'] + ['experiment-execution-view',
    'experiment-update-arena', 'experiment-piston-plan', 'experiment-lazy-journal']
FEATURES['next_counters'] = FEATURES['next_combined'] + ['experiment-piston-plan-counters']
FEATURES['next_view_lazy'] = FEATURES['next_view'] + ['experiment-lazy-journal']
FEATURES['next_piston_local'] = FEATURES['next_piston']
FEATURES['next_plan_combo'] = FEATURES['next_view_lazy'] + ['experiment-piston-plan']
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--variants', nargs='+', choices=FEATURES, required=True)
p.add_argument('--test-features', default=None)
p.add_argument('--fixtures', type=pathlib.Path, required=True)
p.add_argument('--fuzz-seeds', type=int, default=0, help='Untimed randomized differential sequences per fixture')
a = p.parse_args()
root = pathlib.Path(__file__).resolve().parents[1]
a.output = a.output.resolve(); a.output.mkdir(parents=True, exist_ok=True)
env = os.environ.copy()
env.update(CARGO_BUILD_JOBS='4', RUST_TEST_THREADS='4', CARGO_PROFILE_RELEASE_DEBUG='1',
           CARGO_PROFILE_RELEASE_OPT_LEVEL='3', CARGO_PROFILE_RELEASE_LTO='true',
           MC_TICK_BENCH_FIXTURES=str(a.fixtures.resolve()), RUSTFLAGS='')
env.pop('CARGO_ENCODED_RUSTFLAGS', None)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def run(args, log, extra=None):
    print('Running', ' '.join(args), flush=True)
    with (a.output / log).open('w') as f:
        if log.endswith('.jsonl'):
            with (a.output / (log + '.stderr')).open('w') as err:
                subprocess.run(args, cwd=root, env=env | (extra or {}), stdout=f, stderr=err, check=True)
        else:
            subprocess.run(args, cwd=root, env=env | (extra or {}), stdout=f, stderr=subprocess.STDOUT, check=True)
sources = {str(f.relative_to(root)):sha(f) for f in sorted((root/'crates/mc-tick/src').rglob('*.rs'))}
for f in ['Cargo.lock','crates/mc-tick/Cargo.toml','crates/mc-tick/examples/bench_circuits.rs']:
    sources[f] = sha(root/f)
for f in sorted((root/'crates/mc-tick/examples/bench_circuits').rglob('*.rs')):
    sources[str(f.relative_to(root))] = sha(f)
archive=a.output/'sources.tar.gz'
with tarfile.open(archive,'w:gz') as tar:
    for f in sources: tar.add(root/f,arcname=f)
manifest = {'rustc':subprocess.check_output(['rustc','-vV'],text=True),
            'base':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),
            'source_sha256':sources, 'source_archive_sha256':sha(archive),
            'profile':{'opt_level':3,'lto':True,'debug':1,'cpu':'generic'},
            'variants':{}}
if a.test_features is not None:
    args=['cargo','test','--locked','--no-default-features','-p','mc-tick','-p','mc-test']
    if a.test_features: args += ['--features',a.test_features]
    run(args,'tests.log',{'MC_TICK_WIRE_PLAN':'shadow', 'MC_TICK_PISTON_PLAN':'shadow'})
for name in a.variants:
    features = FEATURES[name]
    args=['cargo','build','--locked','--release','--no-default-features','-p','mc-tick','--example','bench_circuits']
    if features: args += ['--features',','.join(features)]
    run(args,name+'-build.log')
    binary=a.output/name
    target=pathlib.Path(env.get('CARGO_TARGET_DIR',str(root/'target')))
    shutil.copy2(target/'release/examples/bench_circuits',binary)
    shadow={'MC_TICK_WIRE_PLAN':'shadow', 'MC_TICK_PISTON_PLAN':'shadow'}
    run([str(binary),'--verify','--expanded'],name+'-verify.jsonl',shadow)
    if a.fuzz_seeds:
        run([str(binary),'--fuzz',str(a.fuzz_seeds)],name+'-fuzz.jsonl',shadow)
    manifest['variants'][name]={'features':features,'sha256':sha(binary),'path':str(binary)}
    (a.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('Builds and expanded verification complete:',a.output,flush=True)
