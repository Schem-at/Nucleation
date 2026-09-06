#!/usr/bin/env bash
# Nucleation dev loop — tiered checks.
#
#   tools/dev.sh fast [crate]     seconds   — the edit/compile loop
#   tools/dev.sh pre-land         minutes   — run before you hand work over
#   tools/dev.sh full             the merge gate — everything, exhaustive
#   tools/dev.sh doctor           build-environment health (tools/doctor.sh)
#
# WHY THIS EXISTS
# The repo can be built with many feature combinations (default / simulation /
# bridge / bridge-full / +routing / +hdl / +meshing), for three wasm32 crate
# targets, plus release wheels. Every distinct feature set is a distinct
# fingerprint for all 159 dependencies, so ad-hoc `cargo` invocations with
# ad-hoc features multiply both build time and disk. `fast` pins ONE canonical
# feature set so the loop keeps hitting the same warm artifacts.
#
# THE ONE RULE: iterate on $CANON. Other feature sets are checked by `full`.
set -uo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"

# The canonical loop feature set. bridge-full already implies meshing,
# simulation, rendering, scripting-*, voxelize, world-segment and mc-tick;
# meshing is named explicitly so the intent survives a bridge-full edit.
CANON="bridge-full,routing,hdl,meshing"

# Exhaustive sim suites cover complete four-bit truth tables. Production-width
# generators also receive structural audits, because an exhaustive eight-bit
# physical ALU would require 524,288 slow simulation cases. They are a merge
# gate, not an iteration cost. `full` sets EDA_EXHAUSTIVE=1; every other tier
# samples.
SAMPLE_CASES="${EDA_SAMPLE_CASES:-24}"

# A plain string, not an array: macOS ships bash 3.2, where expanding an empty
# array under `set -u` is itself an "unbound variable" error.
FAILED=""
step() { # step <name> <cmd...>
  local name="$1"; shift
  local start dur
  printf '\n\033[1;34m==> %s\033[0m\n' "$name"
  start=$SECONDS
  if "$@"; then
    dur=$((SECONDS - start))
    printf '\033[1;32m    ok\033[0m (%ss)\n' "$dur"
  else
    dur=$((SECONDS - start))
    printf '\033[1;31m    FAILED\033[0m (%ss)\n' "$dur"
    FAILED="$FAILED  $name"
    return 1
  fi
}

summary() {
  local total=$SECONDS
  if [[ -n "$FAILED" ]]; then
    printf '\n\033[1;31m%s FAILED\033[0m in %ss:%s\n' "$TIER" "$total" "$FAILED"
    exit 1
  fi
  printf '\n\033[1;32m%s passed\033[0m in %ss\n' "$TIER" "$total"
}

# ---------------------------------------------------------------- tier: fast
# Goal: seconds. A type error must surface without a full test build.
tier_fast() {
  local crate="${1:-}"
  step "check ($CANON)" cargo check --features "$CANON" --all-targets || true

  # Unit tests only (`--lib`). This is a RUNTIME decision, not a build one:
  # mc-tick's integration suites are dominated by a few long simulations —
  # measured `machine_graph` 89.3s and `timeline` 14.3s, against 0.48s for the
  # whole `--lib` set — and running them per keystroke is what made this tier
  # 63s. `pre-land` runs every integration suite, `full` runs everything.
  if [[ -n "$crate" ]]; then
    # Touched-crate tests only. crates/* deliberately do not depend on
    # nucleation, so `-p mc-tick` does not rebuild the 188k-line root crate.
    step "test -p $crate --lib" cargo test -p "$crate" --lib || true
  else
    # The root crate's own `--lib` set is ~1276 tests and ~3 MINUTES of runtime
    # (measured; it is runtime, not build — a filtered single-test run costs the
    # same). That belongs in pre-land, not in a per-keystroke tier. Here we run
    # every *other* workspace crate's unit tests, which are sub-second, and get
    # real signal for the crates most work actually happens in.
    step "test --workspace --exclude nucleation --lib" \
      cargo test --workspace --exclude nucleation --lib || true
    printf '    \033[2mroot-crate lib tests (~3 min) run in pre-land; pass a crate name to target one\033[0m\n'
  fi

  # Sampled sim cases — enough to catch a broken gadget, not a proof.
  # Measured ~1-2s each; the exhaustive forms of these are minutes and live in
  # the `full` tier. `--quick`/`--bits 2`/`--cases` are the sampling levers the
  # scripts already provide, so nothing here forks their logic.
  step "sim: adder (sampled, --quick)" \
    python3 redstone-eda/build_adder.py --quick --bits 4 || true
  step "sim: rca cells (sampled, --bits 2)" \
    python3 redstone-eda/rca_cells.py --bits 2 || true
  step "sim: hdl popcnt4 (sampled, --cases $SAMPLE_CASES)" \
    python3 redstone-eda/hdl/hdl2redstone.py \
      --verilog redstone-eda/hdl/popcnt4.v --top popcnt4 --cases "$SAMPLE_CASES" || true
}

# ------------------------------------------------------------ tier: pre-land
# Goal: minutes. Must pass before work leaves your machine.
tier_pre_land() {
  step "wasm32 unsigned buffer offsets" node --test tests/node_wasm_large_offsets_test.mjs || true
  step "test (canonical, full)" cargo test --features "$CANON" || true

  # Every workspace crate's full suite, integration tests included — this is
  # where mc-tick's slow simulations (machine_graph ~89s, timeline ~14s) get
  # run, having been kept out of `fast`.
  step "test --workspace (all crates, integration suites)" cargo test --workspace || true

  # wasm32 x3. WASM compatibility is a REQUIREMENT for the routing crates and
  # the browser engine; these break on fs/thread/rng use that native never sees.
  rustup target list --installed | grep -q wasm32-unknown-unknown \
    || rustup target add wasm32-unknown-unknown
  step "wasm32: pnr-core"          cargo check -p pnr-core --target wasm32-unknown-unknown || true
  step "wasm32: nucleation-routing" cargo check -p nucleation-routing --target wasm32-unknown-unknown || true
  step "wasm32: browser engine" cargo check --target wasm32-unknown-unknown --no-default-features \
    --features bridge,simulation,mc-tick,routing,hdl,meshing || true

  # Pre-flight the preview port. `npm run verify` starts its own vite preview
  # server; when a stray one from an earlier session still holds the port, vite
  # logs "Port 8461 is already in use" and the run then dies many lines later
  # with an opaque `page.goto ... ERR_CONNECTION_REFUSED`. Say what is actually
  # wrong, and who is holding it, before spending 28s to find out.
  step "studio: preview port free" bash -c '
    port=$(grep -oE "localhost:[0-9]+" apps/eda-studio/scripts/verify.mjs | head -1 | cut -d: -f2)
    port=${port:-8461}
    holder=$(lsof -ti ":$port" 2>/dev/null | head -1)
    if [ -n "$holder" ]; then
      echo "port $port is held by pid $holder:"
      ps -o command= -p "$holder" 2>/dev/null | cut -c1-100
      echo "stop it first (a stray \`vite preview\`, often via apps/preview.pid), then re-run"
      exit 1
    fi
    echo "port $port is free"' || true

  step "studio: wasm engine" env \
    NUCLEATION_WASM_FEATURES="bridge,simulation,mc-tick,routing,hdl,meshing" \
    ./tools/package-npm.sh dist/npm-eda || true

  step "studio: build + headless verify" bash -c \
    'cd apps/eda-studio && { [ -d node_modules ] || npm ci; }; npm run build && npm run verify' || true

  step "python demo smoke" ./examples/bridge_smoke/python/run.sh || true
}

# ---------------------------------------------------------------- tier: full
# The merge gate. Nothing here may be sampled, skipped or narrowed.
tier_full() {
  export EDA_EXHAUSTIVE=1

  step "test (default features)"  cargo test || true
  step "test (canonical, full)"   cargo test --features "$CANON" || true

  # Feature permutations. Each of these has historically caught a break that no
  # union-of-features build could see (see tools/prepush.py for the war stories).
  step "perm: bridge"             cargo build --lib --features bridge || true
  step "perm: bridge,mc-tick"     cargo build --lib --features bridge,mc-tick || true
  step "perm: simulation"         cargo check --features simulation || true
  step "perm: meshing"            cargo check --features meshing || true
  step "perm: routing"            cargo check --features routing || true
  step "perm: hdl"                cargo check --features hdl || true
  step "perm: bridge-full"        cargo check --features bridge-full || true
  step "perm: cli render"         cargo check -p nucleation-cli --features render || true

  step "routing check suite"      ./tools/check_routing.sh || true

  # Exhaustive sim suites. Pin the scalable generators to four bits so every
  # input combination is checked in finite CI time, then audit production-size
  # geometry separately. Never invoke build_alu.py/build_ppa.py with their
  # larger defaults and no --cases: that is 524,288 ALU cases at 8 bits and an
  # effectively unbounded truth table at the PPA's 32-bit default.
  step "sim: adder exhaustive"    python3 redstone-eda/build_adder.py || true
  step "sim: rca exhaustive"      python3 redstone-eda/rca_cells.py || true
  step "sim: alu 4-bit exhaustive" python3 redstone-eda/build_alu.py --width 4 || true
  step "audit: alu 8-bit structure" python3 redstone-eda/build_alu.py --width 8 --no-sim || true
  step "sim: ppa 4-bit exhaustive" python3 redstone-eda/build_ppa.py --width 4 || true
  step "audit: ppa 32-bit structure" python3 redstone-eda/build_ppa.py --width 32 --no-sim || true
  step "sim: genlib cells"        python3 redstone-eda/genlib_map.py --cells || true
  for d in seg7 cmp4 popcnt4; do
    step "sim: genlib $d"         python3 redstone-eda/genlib_map.py --design "$d" || true
    step "sim: hdl $d exhaustive" python3 redstone-eda/hdl/hdl2redstone.py \
      --verilog "redstone-eda/hdl/$d.v" --top "$d" || true
  done

  # Wheel + npm rebuild. Only the bridge surface affects these; the guards in
  # package-npm.sh make a no-op rebuild cheap.
  step "npm package (wasm)"       ./tools/package-npm.sh dist/npm || true
  step "python wheel"             python3 -m build --wheel bindings/python \
    --outdir target/python-wheels || true

  # Everything CI gates on: bindings freshness/determinism, coverage, smokes.
  step "prepush gates"            python3 tools/prepush.py || true
}

TIER="${1:-fast}"
shift || true
case "$TIER" in
  fast)     tier_fast "${1:-}" ;;
  pre-land) tier_pre_land ;;
  full)     tier_full ;;
  doctor)   exec ./tools/doctor.sh ;;
  *) echo "usage: tools/dev.sh {fast [crate]|pre-land|full|doctor}" >&2; exit 2 ;;
esac
summary
