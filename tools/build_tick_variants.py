#!/usr/bin/env python3
"""Build stage binaries in a NEW disposable worktree. Never resets user work."""
import argparse,pathlib,subprocess,os,json
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--worktree',type=pathlib.Path,required=True);p.add_argument('--patches',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);p.add_argument('--target',type=pathlib.Path,required=True);a=p.parse_args()
assert not a.worktree.exists(),'worktree must be new'
a.patches=a.patches.resolve();a.output=a.output.resolve();a.target=a.target.resolve();a.worktree=a.worktree.resolve()
manifest=json.loads((a.patches/'manifest.json').read_text());base=manifest['baseline'];optimized=manifest['optimized']
subprocess.run(['git','worktree','add','--detach',str(a.worktree),base],check=True)
for f in ['Cargo.lock','crates/mc-tick/Cargo.toml','crates/mc-tick/examples/bench_circuits.rs']:
    (a.worktree/f).write_bytes(subprocess.check_output(['git','show',optimized+':'+f]))
a.output.mkdir(parents=True,exist_ok=True)
env=os.environ.copy();env.update(CARGO_BUILD_JOBS='4',CARGO_PROFILE_RELEASE_DEBUG='1',CARGO_TARGET_DIR=str(a.target),RUSTC_WRAPPER='')
for stage in ['baseline','diagnostics','tables','dust','wire','world','power']:
    # This worktree was created above; these are exclusively our generated files.
    subprocess.run(['git','restore','--source',base,'--worktree','--','crates/mc-tick/src'],cwd=a.worktree,check=True)
    for f in ['diagnostics.rs']:
        path=a.worktree/'crates/mc-tick/src'/f
        if path.exists():path.unlink()
    subprocess.run(['git','apply',str(a.patches/(stage+'.patch'))],cwd=a.worktree,check=True)
    with (a.output/(stage+'-build.log')).open('w') as log:
        subprocess.run(['cargo','build','--locked','--release','-p','mc-tick','--example','bench_circuits'],cwd=a.worktree,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
    import shutil
    shutil.copy2(a.target/'release/examples/bench_circuits',a.output/stage)
    with (a.output/(stage+'-verify.jsonl')).open('w') as log:
        subprocess.run([str(a.output/stage),'--verify'],stdout=log,check=True)
    assert (a.output/(stage+'-verify.jsonl')).read_bytes()==(a.output/'baseline-verify.jsonl').read_bytes(),stage+' differs from baseline'
    print(stage+' built and verified',flush=True)
