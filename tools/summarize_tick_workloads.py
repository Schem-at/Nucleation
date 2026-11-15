#!/usr/bin/env python3
"""Derive workload medians/TPS and plotting inputs from saved paired analyses."""
import json
import pathlib
import statistics

root = pathlib.Path(__file__).resolve().parents[1]
results = root / 'docs/performance/workload-specialization/results'
raw = [json.loads(line) for line in (results / 'final-timings.jsonl').read_text().splitlines()]
samples = [row for row in raw if row['kind'] == 'sample']
cases = list(dict.fromkeys(row['case'] for row in samples))
medians = []
for case in cases:
    for variant in raw[0]['variants']:
        rows = [row for row in samples if row['case'] == case and row['variant'] == variant]
        assert len(rows) == 21
        cpu = statistics.median(row['samples_cpu_ns'][0] / row['iterations'] / 1e6 for row in rows)
        wall = statistics.median(row['samples_ns'][0] / row['iterations'] / 1e6 for row in rows)
        ticks = rows[0]['ticks']
        medians.append({'case': case, 'variant': variant, 'ticks': ticks,
                        'cpu_ms': cpu, 'wall_ms': wall,
                        'elapsed_tps': ticks * 1000 / wall if ticks else None})
(results / 'final-medians.json').write_text(json.dumps({'source': 'final-timings.jsonl', 'results': medians}, indent=2) + '\n')
for input_name, output_name, reference in [
    ('final-summary.json', 'final-incremental-plot.json', 'matching previous build'),
    ('final-master-comparison.json', 'final-master-plot.json', 'matching master build'),
]:
    analysis = json.loads((results / input_name).read_text())
    rows = []
    for case in cases:
        for variant, label in [('selected', 'Standard'), ('selected_pgo', 'PGO')]:
            row = next(row.copy() for row in analysis['results'] if row['case'] == case and row['variant'] == variant)
            row['variant'] = label
            rows.append(row)
    (results / output_name).write_text(json.dumps({
        'baseline': analysis['baseline'], 'source': input_name,
        'comparison_label': reference, 'results': rows,
    }, indent=2) + '\n')
print('Saved all medians/TPS and both plotting inputs.')
