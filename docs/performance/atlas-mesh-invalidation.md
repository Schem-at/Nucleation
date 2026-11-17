# Atlas mesh invalidation — 17 November 2026

Atlas already meshed only dirty chunks, once per presentation batch rather than
once per simulation tick. However, it applied every intermediate block write and
kept rebuilds queued after changes cancelled one another. Updates to a later
chunk while an earlier GLB was parsing could also schedule a redundant second
rebuild of the later chunk.

The renderer now coalesces each batch by position, skips equivalent visible
states, cancels pending reversals, and starts each chunk's revision snapshot at
its actual mesh call. Entity sync, piston interpolation, recording and simulation
remain independent. The resource pack can override the normally invisible
properties; unknown definitions are handled conservatively. Source and engine
behavior remain untouched by the rendering change.

## Same-engine work counts

Baseline renderer: commit `336509a0`. Both variants used the same served WASM
(`c3361e60a4ea982ba4ded2219404d548db53bc81079c5191d6c8bced2bee18d6`),
resource pack, browser and eight-tick presentation batches. Initial meshing and
model warm-up were excluded from the counts. These are deterministic **work
counts**, not timing distributions or FPS speedups. Ordinary one-tick playback
will have different opportunities to combine writes.

| Workload | Ticks | Mirror writes, before → after | Chunk rebuilds, before → after |
|---|---:|---:|---:|
| Hipster door, inputs at 10/170 | 320 | 8,984 → 964 | 42 → 39 |
| Note-triggered mechanism, input at 21 | 160 | 20,590 → 2,945 | 251 → 243 |

Both the final simulation states and the final render schematics matched the
baseline by SHA-256. The latter mechanism still has the separate wall-update
vanilla mismatch documented in the scaffolding report; this experiment does not
certify it as vanilla-compatible. Native simulation throughput was not measured.

Targeted real-WASM/GLB checks changed one chunk rebuild to zero for each of:
within-batch reversal, note-pitch change, and scaffolding-distance-only change.
The real resource-pack GLB bytes for the note/scaffold pairs were identical.
Twenty-two browser assertions cover these skips, chunk seams, negative growth,
updates during GLB parsing, cancelled queued work and shared concurrent flushes.
The existing skin/motion suite passed all 808 assertions, including both player
skins and 48 flights across all six piston directions, with no shared-geometry
disposals or page errors. Pure checks cover custom variants, nested multipart
conditions, unsupported definitions, water/tint distinctions and the presentation
clock. TypeScript checking and the production Vite build passed.

The full compact results are in [atlas-mesh-invalidation.json](atlas-mesh-invalidation.json).
Raw checks and the baseline module are retained outside Git in the local archive's
`validation/atlas-mesh-invalidation/` directory. No schematics were added to Git.

## Reproduction and rollback

From `apps/sim-lab-wasm`, build the comparison-only baseline module with the
installed `esbuild` dependency, while Atlas serves this app:

```sh
node --input-type=module <<'JS'
import {build} from 'esbuild';
import {execFileSync} from 'node:child_process';
await build({stdin:{contents:execFileSync('git',
  ['show','336509a0:apps/sim-lab-wasm/src/world.ts'],{encoding:'utf8'}),
  resolveDir:process.cwd()+'/src',sourcefile:'baseline-world.ts',loader:'ts'},
  bundle:true,format:'esm',outfile:'dist/mesh-baseline-test.mjs'});
JS
CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
  node verify-mesh-updates.mjs http://127.0.0.1:8767 /tmp/atlas-mesh-updates \
  /sim-lab/mesh-baseline-test.mjs
CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' \
  node profile-mesh-updates.mjs /path/to/hipster.litematic hipster \
  /sim-lab/mesh-baseline-test.mjs
```

Use `note` and the original note-triggered litematic for the second workload.
The profiling script checks both engine and render-mirror hashes before accepting
a comparison. Remove the temporary baseline module after testing. Rollback is a
frontend revert and rebuild; the Rust/WASM engine is unchanged. Reload Atlas to
pick up the new bundle. It is served locally; this is not a package release.
