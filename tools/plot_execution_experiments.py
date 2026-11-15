#!/usr/bin/env python3
"""Plot all paired candidate speedups and their uncertainty from saved analysis."""
import argparse, json, pathlib
import matplotlib.pyplot as plt
from matplotlib.colors import TwoSlopeNorm

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('input', type=pathlib.Path)
p.add_argument('--output', type=pathlib.Path, required=True)
a = p.parse_args()
doc = json.loads(a.input.read_text()); rows = doc['results']
cases = list(dict.fromkeys(r['case'] for r in rows))
variants = list(dict.fromkeys(r['variant'] for r in rows if r['variant'] != doc['baseline']))
entries = [[next(r for r in rows if r['case'] == case and r['variant'] == variant)
            for variant in variants] for case in cases]
values = [[r['cpu_speedup'] for r in row] for row in entries]
flat = [v for row in values for v in row]
fig, ax = plt.subplots(figsize=(max(9, len(variants)*1.65), max(6, len(cases)*.55)))
im = ax.imshow(values, aspect='auto', cmap='RdYlGn',
               norm=TwoSlopeNorm(vmin=min(.97,min(flat)), vcenter=1, vmax=max(1.03,max(flat))))
for y, row in enumerate(entries):
    for x, r in enumerate(row):
        lo, hi = r['cpu_ci95']
        ax.text(x,y,f"{r['cpu_speedup']:.3f}×\n[{lo:.3f}, {hi:.3f}]",ha='center',va='center',fontsize=8, color='white' if im.norm(r['cpu_speedup']) < .16 or im.norm(r['cpu_speedup']) > .85 else 'black')
ax.set_xticks(range(len(variants)), variants)
ax.set_yticks(range(len(cases)), cases)
ax.tick_params(top=True, labeltop=True, bottom=False, labelbottom=False)
reference_label=doc.get('comparison_label','reference')
ax.set_title(f'Nucleation execution experiments · harrison-work\nCPU speedup vs {reference_label}; paired bootstrap 95% intervals',pad=36)
fig.colorbar(im,ax=ax,label='Higher is faster',fraction=.025,pad=.025)
fig.tight_layout()
a.output.parent.mkdir(parents=True,exist_ok=True)
for suffix in ['.png','.svg']: fig.savefig(a.output.with_suffix(suffix),dpi=170)
