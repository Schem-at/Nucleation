#!/usr/bin/env python3
"""Regenerate cumulative reconstruction patches without changing the checkout.

The optimized revision must be the reconstruction checkpoint, before caching.
All stages include the same recording-control API for the common harness.
"""
import argparse,difflib,pathlib,re,subprocess,json,hashlib
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--baseline',default='ee89008d');p.add_argument('--optimized',default='5a92c5be');p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
prefix='crates/mc-tick/src/'
files=['behaviour.rs','components.rs','diagnostics.rs','lib.rs','piston.rs','sim.rs','state.rs','vanilla.rs','wire.rs','world.rs']
def source(rev,name):
    result=subprocess.run(['git','show',f'{rev}:{prefix}{name}'],capture_output=True,text=True)
    if result.returncode and name!='diagnostics.rs':raise RuntimeError(result.stderr)
    return result.stdout
base={f:source(a.baseline,f) for f in files};opt={f:source(a.optimized,f) for f in files}
def method(source,name):
    start=source.index('    fn '+name+'(');end=source.index('\n    }\n',start)+7
    return source[start:end]
def save(name,tree):
    patch=''
    for f in files:
        patch+=''.join(difflib.unified_diff(base[f].splitlines(True),tree[f].splitlines(True),fromfile='a/'+prefix+f if base[f] else '/dev/null',tofile='b/'+prefix+f))
    (a.output/(name+'.patch')).write_text(patch)
a.output.mkdir(parents=True,exist_ok=True)
tree=base.copy()
start=opt['sim.rs'].index('    /// Discard and disable ordinary recording');end=opt['sim.rs'].index('    /// Start a rich run timeline',start)
tree['sim.rs']=tree['sim.rs'].replace('    /// Start a rich run timeline',opt['sim.rs'][start:end]+'    /// Start a rich run timeline')
save('baseline',tree)
for f in ['behaviour.rs','components.rs','piston.rs','sim.rs','wire.rs']:
    tree[f]=tree[f].replace('std::env::var_os(', 'crate::diagnostics::var_os(').replace('std::env::var(', 'crate::diagnostics::var(')
tree['lib.rs']=opt['lib.rs'];tree['diagnostics.rs']=opt['diagnostics.rs'];save('diagnostics',tree)
tree['state.rs']=opt['state.rs']
s=tree['vanilla.rs'].replace('StateRegistry, StateSet','StateRegistry, StateSet, StateTable')
s=re.sub(r'HashMap<StateId, (.+)>,',r'StateTable<\1>,',s)
s=s.replace('''rules
            .wires
            .iter()
            .map(|(id, (_, sides))| (*id, *sides))''','''(0..registry.len()).filter_map(|i| {
                let id = StateId(i as u16);
                rules.wires.get(&id).map(|(_, sides)| (id, *sides))
            })''')
tree['vanilla.rs']=s;save('tables',tree)
tree['wire.rs']=opt['wire.rs'];save('dust',tree)
s=opt['vanilla.rs']
for name in ['emitted','direct_signal_to','signal_no_wire','is_powered','signal_strength']:
    s=s.replace(method(s,name),method(base['vanilla.rs'],name),1)
s=s.replace(method(s,'emitted_state'),'')
tree['vanilla.rs']=s;save('wire',tree)
tree['world.rs']=opt['world.rs'];save('world',tree)
save('power',opt)
(a.output/'manifest.json').write_text(json.dumps({'baseline':subprocess.check_output(['git','rev-parse',a.baseline],text=True).strip(),'optimized':subprocess.check_output(['git','rev-parse',a.optimized],text=True).strip(),'patches':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(a.output.glob('*.patch'))}},indent=2)+'\n')
