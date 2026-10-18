#!/usr/bin/env bash
# Criterion baselines for the root crate's Rust benches.
#
#   tools/bench.sh save <name> [bench...]      record a named baseline
#   tools/bench.sh compare <name> [bench...]   rerun and diff against it
#   tools/bench.sh run [bench...]              plain run, no baseline
#
# With no bench names, every root bench runs. Baselines live under
# target/criterion/<group>/<case>/<name>; they are only comparable on the
# same machine, power mode and toolchain. Typical use:
#
#   git switch main && tools/bench.sh save main
#   git switch my-branch && tools/bench.sh compare main
set -euo pipefail

cd "$(dirname "$0")/.."

ALL=(block_placement region_bench snapshot_bench fingerprint_bench performance_test
     voxelize_bench scripting_bench)
FEATURES=voxelize,scripting

cmd=${1:-}
case "$cmd" in
  save | compare)
    name=${2:?"usage: tools/bench.sh $cmd <name> [bench...]"}
    shift 2
    flag=$([ "$cmd" = save ] && echo --save-baseline || echo --baseline)
    extra=("$flag" "$name")
    ;;
  run)
    shift
    extra=()
    ;;
  *)
    sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'
    exit 2
    ;;
esac

benches=("$@")
[ ${#benches[@]} -eq 0 ] && benches=("${ALL[@]}")
for bench in "${benches[@]}"; do
  cargo bench --features "$FEATURES" --bench "$bench" -- --noplot "${extra[@]}"
done
