#!/usr/bin/env python3
"""Build isolated feature candidates and save validation evidence on the benchmark host."""
import argparse, hashlib, json, os, pathlib, shutil, subprocess

FEATURES = {
    'reference': [], 'directional': ['experiment-directional'],
    'cursor': ['experiment-cursor'], 'pool': ['experiment-update-pool'],
    'dispatch': ['experiment-wire-dispatch'], 'plan': ['experiment-wire-plan'],
    'certificate': ['experiment-wire-certificate'],
    'counters': ['experiment-wire-certificate', 'experiment-wire-plan-counters'],
}
FEATURES['combined'] = list(dict.fromkeys(f for name, fs in FEATURES.items()
                                         if name != 'counters' for f in fs))
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=pathlib.Path, required=True)
p.add_argument('--variants', nargs='+', choices=FEATURES, required=True)
p.add_argument('--test-features', default=None)
p.add_argument('--fixtures', type=pathlib.Path, required=True)
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
        subprocess.run(args, cwd=root, env=env | (extra or {}), stdout=f,
                       stderr=subprocess.STDOUT, check=True)
sources = {str(f.relative_to(root)):sha(f) for f in sorted((root/'crates/mc-tick/src').rglob('*.rs'))}
for f in ['Cargo.lock','crates/mc-tick/Cargo.toml','crates/mc-tick/examples/bench_circuits.rs']:
    sources[f] = sha(root/f)
manifest = {'rustc':subprocess.check_output(['rustc','-vV'],text=True),
            'base':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),
            'source_sha256':sources, 'profile':{'opt_level':3,'lto':True,'debug':1,'cpu':'generic'},
            'variants':{}}
if a.test_features is not None:
    args=['cargo','test','--locked','--no-default-features','-p','mc-tick','-p','mc-test']
    if a.test_features: args += ['--features',a.test_features]
    run(args,'tests.log',{'MC_TICK_WIRE_PLAN':'shadow'})
for name in a.variants:
    features = FEATURES[name]
    args=['cargo','build','--locked','--release','--no-default-features','-p','mc-tick','--example','bench_circuits']
    if features: args += ['--features',','.join(features)]
    run(args,name+'-build.log')
    binary=a.output/name
    target=pathlib.Path(env.get('CARGO_TARGET_DIR',str(root/'target')))
    shutil.copy2(target/'release/examples/bench_circuits',binary)
    run([str(binary),'--verify','--expanded'],name+'-verify.jsonl',{'MC_TICK_WIRE_PLAN':'shadow'})
    manifest['variants'][name]={'features':features,'sha256':sha(binary),'path':str(binary)}
    (a.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('Builds and expanded verification complete:',a.output,flush=True)
