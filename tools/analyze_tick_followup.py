#!/usr/bin/env python3
"""Paired bootstrap ratios of medians; preserves every observation."""
import argparse,json,random,statistics,pathlib
p=argparse.ArgumentParser(description=__doc__);p.add_argument('input',type=pathlib.Path)
p.add_argument('--baseline',default='baseline');p.add_argument('--output',type=pathlib.Path,required=True)
p.add_argument('--resamples',type=int,default=10000);p.add_argument('--compare',action='append',default=[],help='VARIANT=REFERENCE for selected paired comparisons');a=p.parse_args()
references=dict(v.split('=',1) for v in a.compare)
rows=[json.loads(x) for x in a.input.read_text().splitlines()]
data={}
for row in rows:
    if row['kind']=='sample':
        key=(row['case'],row['variant']);data.setdefault(key,{})[row['round']]={
            'cpu':row['samples_cpu_ns'][0]/row['iterations']/1e6,
            'wall':row['samples_ns'][0]/row['iterations']/1e6}
rng=random.Random(20261114);result=[]
for (case,variant),values in data.items():
    if references and variant not in references:continue
    reference=references.get(variant,a.baseline)
    baseline=data[(case,reference)]
    assert set(values)==set(baseline),'unpaired rounds'
    rounds=sorted(values);n=len(rounds)
    row={'case':case,'variant':variant,'reference':reference,'rounds':n}
    # Identical paired resamples for a comparison regardless of output ordering.
    rng=random.Random(20261114)
    for metric in ['cpu','wall']:
        base=[baseline[r][metric] for r in rounds];test=[values[r][metric] for r in rounds]
        row[metric+'_ms']=statistics.median(test);row[metric+'_speedup']=statistics.median(base)/statistics.median(test)
        ratios=[]
        for _ in range(a.resamples):
            indices=rng.choices(range(n),k=n)
            ratios.append(statistics.median(base[i] for i in indices)/statistics.median(test[i] for i in indices))
        ratios.sort();row[metric+'_ci95']=[ratios[int(.025*a.resamples)],ratios[int(.975*a.resamples)]]
    result.append(row)
a.output.write_text(json.dumps({'baseline':a.baseline,'source':str(a.input),'resamples':a.resamples,'results':result},indent=2)+'\n')
for r in result:
    print(f"{r['case']:20} {r['variant']:12} {r['cpu_ms']:9.4f} ms  {r['cpu_speedup']:.3f}x [{r['cpu_ci95'][0]:.3f}, {r['cpu_ci95'][1]:.3f}]")
