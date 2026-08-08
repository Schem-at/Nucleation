# Redstone EDA demos

> See also `../showcase/` — a gallery of six sim-verified, baked-at-rest
> `.schem` pieces (router feature gallery, 4-bit cell RCA, stacked 4x4
> multiplier, 32-bit Kogge-Stone, 8-bit ALU, 8-bit bus riser) with a
> verification-evidence table in its README.

Four small scripts that exercise the `feat/redstone-eda` umbrella branch end
to end: the new routing bridge (`Routing.route_net` / `drc` / `sta`), the
sim-introspection bridge (`conduction_trace` / `read_probes` / `bake_to`),
and the Python cell library from this directory tree.

## Setup

Build the Python wheel from the branch checkout (the stale vendored copy
under `bindings/python/rust/` must not exist — it is gitignored staging from
`tools/stage-python-sdist.py` and silently pins an old core; move it aside):

```sh
python3 -m venv ~/eda-venv
NUCLEATION_FEATURES=bridge-full,routing ~/eda-venv/bin/pip install ./bindings/python
```

`routing` is NOT part of `bridge-full`; forgetting it builds a wheel whose
`Routing.*` calls fail at link level.

## Run

```sh
cd redstone-eda/demos
~/eda-venv/bin/python demo1_route.py
~/eda-venv/bin/python demo2_introspect.py
~/eda-venv/bin/python demo3_compose.py
~/eda-venv/bin/python demo4_analyse.py
```

All four exit 0. Outputs (`routed.schem`, `introspected_baked.schem`,
`rca4_composed.schem`) land in this directory.

## What each demo shows

- **demo1_route.py** — lever and target rail separated by a stone wall;
  `Routing.route_net` finds a path (it dives under the wall), emits dust +
  supports into the schematic, `Routing.drc` comes back clean, and a
  TickSimulation proves the routed wire conducts when the lever flips
  (power 0 → 12 at the far end).
- **demo2_introspect.py** — lever → dust run → lamp; `conduction_trace`
  pretty-prints the recursive who-powers-this tree from the lamp back to the
  lever, `read_probes` batch-reads the dust run in one call, and `bake_to`
  writes the live settled state (wire powers, lit lamp) back into a
  schematic for saving.
- **demo3_compose.py** — `cells.build_half_adder` / `build_full_adder` +
  `rca_cells.build_rca` stamp four truth-tabled FA cells at pitch; the carry
  chain connects by abutment, levers drive the inputs, and six spot-checked
  sums (up to 15+15+1=31) come out right in-sim.
- **demo4_analyse.py** — `Routing.drc` over the real `rca4_cells.schem`
  (0 violations), `Routing.sta` over its gate netlist (sum arrivals 2/4/6/8
  rt, critical path `cin → c1 → c2 → c3 → c4`), then a deliberately built
  dust-ring latch that the repeater-cycle check catches (`repeater_cycle`
  naming the diode).

## Gotchas encoded in `_common.py`

- `bounding_box_json` reports the ALLOCATED region, padded far beyond the
  content (routing can blow it out to ±129); sim offsets must come from the
  actual non-air content minimum (`get_all_blocks_json`, filtered).
- `from_schematic` sizes the world from the allocated region too, so builds
  are round-tripped through `.schem` (which re-normalises tight) before
  simulation — same trap `rs.Build.sim` documents.
- Every block state the sim may reach must be interned up front
  (`rs.EXTRA_STATES`); states first appearing mid-sim sit inert.

## schemat.io / WASM integration path

The same Diplomat bridge that produced these Python bindings generates the
JS/WASM surface: `tools/gen-bindings.sh` (run on this branch with the fork's
`diplomat-tool`) emitted `bindings/js/Routing.mjs` + `.d.ts` alongside the
existing `TickSimulation` bindings, and both `pnr-core` and
`nucleation-routing` check clean for `wasm32-unknown-unknown` (as does the
main crate with `--features routing`). schemat.io therefore consumes the
router exactly as it consumes the rest of the API: build the wasm package
with the `routing` feature enabled, import `Routing` from the generated JS
index, and call `Routing.routeNet(schematic, …)` / `drc` / `sta` on the
schematic objects it already holds — the JSON-in/JSON-out shape means no new
marshalling layer is needed on the site.

## Branch layout (for review)

- **`feat/redstone-eda`** (main checkout) — the umbrella. Contains:
  `crates/pnr-core` + `crates/nucleation-routing`, the `routing` feature +
  `src/routing.rs` seam + `src/bridge/routing.rs`; the two mc-tick
  introspection commits (`conduction_trace`/`read_probes`/`bake_to`,
  error-detail store) plus their bindings regen (cherry-picked from the
  worktree branch); the merge of `feat/io-contracts`; test fixes for the new
  optional `IoMapping.face`/`direction` fields; a full bindings regen adding
  `Routing` + `Schematic.compile_io_contracts_json` to every backend; and
  these demos.
- **`feat/mc-tick-introspection`** (worktree) — the two introspection
  commits + bindings regen only; fully contained in the umbrella, kept for
  isolated review.
- **`feat/io-contracts`** (worktree) — the `src/io_contract/` module
  (CellContract faces, physical sidecar, buses, DEF-style routing regions,
  insign vocabulary, `compile_io_contracts_json`); merged into the umbrella.

## Known deferred items

- **sim-backend trait** — explicitly deferred pending the other agent's
  in-flight bridge feature; the routing crates stay sim-free and conduction
  verification happens through mc-tick from the caller's side (as demo1 does).
- Bridge routing exposes default rules only (single net, default via/stair/
  refresh budgets); `route_all` congestion negotiation, buses, cell stamping
  and rule overrides remain native-crate APIs.
- Bridge `sta` over a bare schematic has no net labels, so per-net repeater
  delays contribute 0 through this path; label-aware short checking likewise
  needs a native `Workspace`.
- Python wheel exposes `TickSimulation.last_error_detail` instead of
  `NucleationError.detail()` (the nanobind template does not emit enum
  methods).
- Regenerated non-Python bindings (JS/C/C++/Kotlin/PHP) are
  generation-verified but not runtime-tested on this branch.
