#!/usr/bin/env python3
"""Export PNG/SVG benchmark figures from analyze_tick_followup.py output."""
import argparse,json,pathlib
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
p=argparse.ArgumentParser(description=__doc__);p.add_argument('input',type=pathlib.Path);p.add_argument('--output',type=pathlib.Path,required=True);p.add_argument('--variants',nargs='+');p.add_argument('--cases',nargs='+');p.add_argument('--points',action='store_true');a=p.parse_args()
doc=json.loads(a.input.read_text());rows=doc['results'];baseline=doc['baseline']
if a.variants:rows=[r for r in rows if r['variant'] in a.variants]
if a.cases:rows=[r for r in rows if r['case'] in a.cases]
variants=a.variants or list(dict.fromkeys(r['variant'] for r in rows if r['variant']!=baseline));cases=a.cases or list(dict.fromkeys(r['case'] for r in rows))
labels={'diagnostics':'Diagnostic options','tables':'Dense state tables','dust':'Stack dust ordering','wire':'Wire shape indexing','world':'Cached world layout','power':'Retained engine','cache_on':'Cache enabled'}
fig,ax=plt.subplots(figsize=(11,6.5));height=.8/max(len(variants),1)
for i,variant in enumerate(variants):
    entries=[next(r for r in rows if r['variant']==variant and r['case']==case) for case in cases]
    y=[j-.4+height*(i+.5) for j in range(len(cases))];values=[r['cpu_speedup'] for r in entries]
    lo=[max(0,r['cpu_speedup']-r['cpu_ci95'][0]) for r in entries];hi=[max(0,r['cpu_ci95'][1]-r['cpu_speedup']) for r in entries]
    if a.points:ax.errorbar(values,y,xerr=[lo,hi],fmt='o',capsize=3,label=labels.get(variant,variant))
    else:ax.barh(y,values,height*.9,label=labels.get(variant,variant),xerr=[lo,hi],capsize=2)
ax.set_yticks(range(len(cases)),cases);ax.invert_yaxis();ax.axvline(1,color='gray',lw=1)
reference={'baseline':'v0.12.2','cache_off':'cache disabled'}.get(baseline,baseline)
ax.set_xlabel(f'CPU-time speedup relative to {reference} (higher is faster)')
ax.set_title('Nucleation circuit simulation · harrison-work\nPaired bootstrap 95% intervals; all samples retained')
ax.legend();ax.grid(axis='x',alpha=.2);ax.set_axisbelow(True);fig.tight_layout()
a.output.parent.mkdir(parents=True,exist_ok=True)
for ext in ['png','svg']:fig.savefig(a.output.with_suffix('.'+ext),dpi=180)
