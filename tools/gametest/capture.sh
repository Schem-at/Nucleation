#!/usr/bin/env bash
# Capture a vanilla trace: stage the datapack into a fresh trace universe and run
# TraceCapture. All arguments are passed through, e.g.:
#
#   tools/gametest/capture.sh --structure nucleation:note_powered \
#       --pulse 1,0,0 --pulse-ticks 2 --out work/note_powered.json
#
# run.sh runs the pass/fail gametests; this runs the tick-by-tick tracer. The two
# share the jar, classpath and pack staging.
set -euo pipefail
cd "$(dirname "$0")"

MC_VERSION="${MC_VERSION:-26.2}"
WORK="${WORK:-work}"
UNIVERSE="$WORK/trace-universe"

INNER="$WORK/inner/META-INF/versions/$MC_VERSION/server-$MC_VERSION.jar"
if [[ ! -f "$INNER" ]]; then
    echo "run tools/gametest/run.sh once first — it fetches and unpacks the server jar" >&2
    exit 1
fi
CP="$INNER:$(find "$WORK/inner/META-INF/libraries" -name '*.jar' | tr '\n' ':')"

mkdir -p "$WORK/classes"
javac -nowarn -cp "$CP" -d "$WORK/classes" src/Snbt2Nbt.java src/TraceCapture.java

# Stage the datapack with .snbt converted to the binary .nbt datapacks require.
# The pack goes inside the universe's world datapack folder, where
# ServerPacksSource.createPackRepository will find it. It must be a complete pack
# — pack.mcmeta and all — or the repository logs "Found non-pack entry, ignoring"
# and TraceCapture dies with "no such structure".
STAGE="$WORK/packs/nucleation_tests"
rm -rf "$WORK/packs"
mkdir -p "$STAGE"
cp -R pack/. "$STAGE/"
while IFS= read -r snbt; do
    rel="${snbt#pack/}"
    out="$STAGE/${rel%.snbt}.nbt"
    java -cp "$WORK/classes:$CP" Snbt2Nbt "$snbt" "$out" >/dev/null 2>&1 \
        || { echo "FAILED converting $snbt" >&2; exit 1; }
    rm -f "$STAGE/$rel"
done < <(find pack -name '*.snbt')

rm -rf "$UNIVERSE"
mkdir -p "$UNIVERSE/gametestworld/datapacks"
cp -R "$STAGE" "$UNIVERSE/gametestworld/datapacks/nucleation_tests"

java -cp "$WORK/classes:$CP" TraceCapture --universe "$UNIVERSE" "$@" \
    2>&1 | grep -viE 'WARNING:|sun\.misc|Saving |chunks are saved|dimensions are saved|Preparing ' || true
