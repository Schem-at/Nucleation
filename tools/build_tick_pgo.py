#!/usr/bin/env python3
"""Build portable PGO and native-CPU experiments, preserving flags and hashes.

Run on the benchmark host. All training and compilation finish before timings.
Uses a separate Cargo target directory. The output directory must be new.
"""
import argparse, hashlib, json, os, pathlib, shutil, subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=pathlib.Path,required=True)
p.add_argument('--target-dir',type=pathlib.Path,required=True)
p.add_argument('--fixtures',type=pathlib.Path,required=True)
p.add_argument('--jobs',type=int,default=4)
p.add_argument('--features',default='',help='Comma-separated mc-tick candidate features; reference is empty')
p.add_argument('--skip-native',action='store_true',help='Only rebuild PGO after a native-CPU pilot has been rejected')
a=p.parse_args();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
raw=a.output/'raw';raw.mkdir()
root=pathlib.Path(__file__).resolve().parents[1]
def capture(*args):return subprocess.check_output(args,cwd=root,text=True).strip()
def sha(path):return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()
rustc=capture('rustc','-vV');host=next(x.split(': ',1)[1] for x in rustc.splitlines() if x.startswith('host:'))
profdata=pathlib.Path(capture('rustc','--print','sysroot'))/'lib/rustlib'/host/'bin/llvm-profdata'
assert profdata.exists(),'rustup component add llvm-tools-preview'
env=os.environ.copy();env.pop('CARGO_ENCODED_RUSTFLAGS',None)
env.update(CARGO_BUILD_JOBS=str(a.jobs),CARGO_TARGET_DIR=str(a.target_dir.resolve()),
    CARGO_PROFILE_RELEASE_OPT_LEVEL='3',CARGO_PROFILE_RELEASE_LTO='true',CARGO_PROFILE_RELEASE_DEBUG='1',
    MC_TICK_BENCH_FIXTURES=str(a.fixtures.resolve()))
command=['cargo','build','--locked','--release','--no-default-features','--target',host,'-p','mc-tick','--example','bench_circuits']
if a.features: command += ['--features', a.features]
exe=a.target_dir.resolve()/host/'release/examples/bench_circuits'
manifest={'rustc':rustc,'head':capture('git','rev-parse','HEAD'),
    'diff_sha256':hashlib.sha256(subprocess.check_output(['git','diff','HEAD'],cwd=root)).hexdigest(),
    'lock_sha256':sha(root/'Cargo.lock'),'harness_sha256':sha(root/'crates/mc-tick/examples/bench_circuits.rs'),
    'fixture_manifest':json.loads((a.fixtures/'manifest.json').read_text()),
    'command':command,'profile':{'opt_level':3,'lto':True,'debug':1},'builds':{}}
def build(name,flags):
    env['RUSTFLAGS']=flags
    with (a.output/(name+'-build.log')).open('w') as log:
        subprocess.run(command,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
    dest=a.output/name;shutil.copy2(exe,dest)
    manifest['builds'][name]={'flags':flags,'sha256':sha(dest)}
    return dest
instrumented=build('instrumented','-Cprofile-generate='+str(raw))
# Broad training; trapdoor, transfer-speed and comparator collection are held out.
training={'hipster':16,'door6':24,'flyer':128,'bb':16,'adder_idle':4096,
    'item_sorter':12,'hopper_storagecart_interaction':12}
manifest['training']=training
with (a.output/'training.jsonl').open('w') as log:
    for case,iterations in training.items():
        subprocess.run([str(instrumented),case,'1',str(iterations)],cwd=root,env=env,stdout=log,check=True)
profile=a.output/'merged.profdata'
subprocess.run([str(profdata),'merge','-o',str(profile),*[str(f) for f in sorted(raw.glob('*.profraw'))]],check=True)
manifest['profile_sha256']=sha(profile)
builds=[('pgo','-Cprofile-use='+str(profile)+' -Cllvm-args=-pgo-warn-missing-function')]
if not a.skip_native: builds.append(('native','-Ctarget-cpu=native'))
for name,flags in builds:
    binary=build(name,flags)
    with (a.output/(name+'-verify.jsonl')).open('w') as log:
        subprocess.run([str(binary),'--verify','--expanded'],cwd=root,env=env,stdout=log,check=True)
(a.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('Built and verified compiler experiments:',a.output,flush=True)
