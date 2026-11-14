#!/usr/bin/env python3
"""Export PNG/SVG benchmark figures from analyze_tick_followup.py output."""
import argparse,json,pathlib
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
p=argparse.ArgumentParser(description=__doc__);p.add_argument('input',type=pathlib.Path);p.add_argument('--output',type=pathlib.Path,required=True);p.add_argument('--variants',nargs='+');p.add_argument('--cases',nargs='+');a=p.parse_args()
doc=json.loads(a.input.read_text());rows=doc['results'];baseline=doc['baseline']
if a.variants:rows=[r for r in rows if r['variant'] in a.variants]
if a.cases:rows=[r for r in rows if r['case'] in a.cases]
variants=list(dict.fromkeys(r['variant'] for r in rows if r['variant']!=baseline));cases=list(dict.fromkeys(r['case'] for r in rows))
fig,ax=plt.subplots(figsize=(11,6.5));height=.8/max(len(variants),1)
for i,variant in enumerate(variants):
    entries=[next(r for r in rows if r['variant']==variant and r['case']==case) for case in cases]
    y=[j-.4+height*(i+.5) for j in range(len(cases))];values=[r['cpu_speedup'] for r in entries]
    lo=[max(0,r['cpu_speedup']-r['cpu_ci95'][0]) for r in entries];hi=[max(0,r['cpu_ci95'][1]-r['cpu_speedup']) for r in entries]
    ax.barh(y,values,height*.9,label=variant,xerr=[lo,hi],capsize=2)
ax.set_yticks(range(len(cases)),cases);ax.invert_yaxis();ax.axvline(1,color='gray',lw=1)
ax.set_xlabel(f'CPU-time speedup relative to {baseline} (higher is faster)')
ax.set_title('Nucleation circuit simulation · harrison-work\nPaired bootstrap 95% intervals; all samples retained')
ax.legend();ax.grid(axis='x',alpha=.2);ax.set_axisbelow(True);fig.tight_layout()
a.output.parent.mkdir(parents=True,exist_ok=True)
for ext in ['png','svg']:fig.savefig(a.output.with_suffix('.'+ext),dpi=180)
