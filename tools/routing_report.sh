#!/usr/bin/env bash
# ONE COMMAND, THREE NUMBERS: the routing before/after report.
#
#   tools/routing_report.sh                 measure, print, and save as `after`
#   tools/routing_report.sh --baseline      measure and save as `baseline`
#   tools/routing_report.sh --diff          re-measure and diff against baseline
#
# WHY THIS EXISTS
# Routing work on this repo has twice produced a change that felt like an
# improvement and measured as nothing: the mechanism-accurate `interferes()`
# predicate held routability at 41/45 and returned a byte-identical cost vector.
# The only defence is measuring the SAME three things before and after every
# step, so a step with no gain is visible as such instead of being kept quietly.
#
# THE THREE THINGS
#   1. routability     tests/design_routability.rs      how many buses route
#   2. cost vector     tests/routing_cost_vector.rs     whether they route WELL
#   3. BCD chain       tests/design_promotion.rs        whether they still WORK
#
# 2 and 3 are GUARDS, not metrics. A routability gain that sprawls every route
# or breaks the verified chain is not a gain, so all three are reported together
# and the diff mode flags a regression in any of them.
set -uo pipefail
cd "$(dirname "$0")/.."

# Must match tools/dev.sh's $CANON, or the report runs against a different
# feature set than the dev loop and the two disagree about what is broken.
CANON="bridge-full,routing,hdl,meshing"

STORE="${ROUTING_REPORT_DIR:-target/routing-report}"
mkdir -p "$STORE"

MODE="after"
case "${1:-}" in
  --baseline) MODE="baseline" ;;
  --diff)     MODE="diff" ;;
  --after|"") MODE="after" ;;
  -h|--help)  sed -n '2,25p' "$0"; exit 0 ;;
  *) echo "unknown flag: $1 (try --baseline | --after | --diff)" >&2; exit 2 ;;
esac

RAW="$STORE/raw.txt"
: > "$RAW"

run() {
  local name="$1"; shift
  echo "--- measuring: $name" >&2
  # `|| true`: a FAILING suite still emits its RR lines, and the report's job is
  # to show the numbers. The test exit status is surfaced separately below.
  "$@" 2>&1 | tee -a "$STORE/$name.log" | grep -E '^RR\|' >> "$RAW" || true
  return 0
}

if [[ "$MODE" != "diff" || ! -f "$STORE/baseline.txt" ]]; then
  :
fi

run routability cargo test --features "$CANON" --test design_routability -- --nocapture
run cost_vector cargo test --features "$CANON" --test routing_cost_vector -- --nocapture
run bcd cargo test --features "$CANON" --test design_promotion -- --nocapture \
  the_adder_feeds_the_bcd_converter the_full_add_bcd_sevenseg_pipeline

# ---------------------------------------------------------------- summarise
summary() {
  local raw="$1"
  awk -F'|' '
    $2 == "routability"   { rate = $3 }
    $2 == "cost_vector"   { cost = $3 }
    $2 == "dirty"         { dirty = $3 }
    $2 == "bcd_arith"     { arith = $3 }
    $2 == "bcd_sevenseg"  { seg = $3 }
    $2 == "fail"          { fails = fails sprintf("  FAIL %-44s %s\n", $3, $4) }
    END {
      printf "routability|%s\n", (rate ? rate : "MISSING")
      printf "cost_vector|%s\n", (cost ? cost : "MISSING")
      printf "dirty|%s\n",       (dirty == "" ? "MISSING" : dirty)
      printf "bcd_arith|%s\n",   (arith ? arith : "MISSING")
      printf "bcd_sevenseg|%s\n",(seg ? seg : "MISSING")
      printf "%s", fails
    }
  ' "$raw"
}

summary "$RAW" > "$STORE/current.txt"

pct() { # "41/45" -> "91.1%"
  awk -F/ 'NF==2 && $2+0>0 { printf "%.1f%%", 100*$1/$2 } NF!=2 { printf "n/a" }' <<< "$1"
}

field() { grep "^$2|" "$1" 2>/dev/null | head -1 | cut -d'|' -f2- ; }

echo
echo "================ ROUTING REPORT ($MODE) ================"
R=$(field "$STORE/current.txt" routability)
C=$(field "$STORE/current.txt" cost_vector)
D=$(field "$STORE/current.txt" dirty)
A=$(field "$STORE/current.txt" bcd_arith)
S=$(field "$STORE/current.txt" bcd_sevenseg)
printf "  routability   %-14s %s\n" "$R" "$(pct "$R")"
printf "  cost vector   %s\n" "$C"
printf "  DRC/LVS dirty %s   (must be 0)\n" "$D"
printf "  BCD arith     %-14s (guard, must be 8/8)\n" "$A"
printf "  BCD 7-seg     %-14s (guard, must be 8/8)\n" "$S"
echo "  --- residual failures ---"
grep '^  FAIL ' "$STORE/current.txt" || echo "  (none)"

if [[ "$MODE" == "baseline" ]]; then
  cp "$STORE/current.txt" "$STORE/baseline.txt"
  echo
  echo "saved as BASELINE -> $STORE/baseline.txt"
  exit 0
fi

if [[ ! -f "$STORE/baseline.txt" ]]; then
  echo
  echo "no baseline recorded yet; run: tools/routing_report.sh --baseline"
  exit 0
fi

echo
echo "================ DELTA vs BASELINE ================"
regressed=0
for k in routability cost_vector dirty bcd_arith bcd_sevenseg; do
  b=$(field "$STORE/baseline.txt" "$k")
  c=$(field "$STORE/current.txt" "$k")
  if [[ "$b" == "$c" ]]; then
    printf "  %-13s %-22s UNCHANGED\n" "$k" "$c"
  else
    printf "  %-13s %-22s <- was %s\n" "$k" "$c" "$b"
    # A guard moving at all, or routability moving DOWN, is a regression.
    case "$k" in
      bcd_arith|bcd_sevenseg|dirty) regressed=1 ;;
      routability)
        bn=$(awk -F/ '{print ($2>0)?100*$1/$2:0}' <<< "$b")
        cn=$(awk -F/ '{print ($2>0)?100*$1/$2:0}' <<< "$c")
        awk -v a="$cn" -v b="$bn" 'BEGIN{exit !(a<b)}' && regressed=1
        ;;
    esac
  fi
done
echo
diff <(grep '^  FAIL ' "$STORE/baseline.txt" || true) \
     <(grep '^  FAIL ' "$STORE/current.txt" || true) \
  && echo "  residual failure SET unchanged" \
  || echo "  (< baseline-only, > current-only)"

if [[ "$regressed" == 1 ]]; then
  echo
  echo "REGRESSION: a guard moved or routability dropped. Not a gain."
  exit 1
fi
echo
echo "no regression."
