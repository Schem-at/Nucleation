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

# WORLD=<path to a save> records that world in place rather than pasting into a
# fresh one. Every paste disturbs a machine — placeInWorld recomputes repeater
# LOCKED and wire shapes, and loads block-entity NBT after the block write — so
# a door built and latched in a world cannot be reproduced by stamping it down.
# Copying the save in and ticking it is the only way the reference is the door
# that was actually built.
rm -rf "$UNIVERSE"
mkdir -p "$UNIVERSE/gametestworld/datapacks"
#
# A save from before the layout change needs relocating first. 26.2 keeps every
# dimension under `dimensions/<namespace>/<path>/`; saves up to at least 1.21
# put the overworld's `region/`, `entities/` and `poi/` at the world root and
# the other two in `DIM-1`/`DIM1`. The server does not look at the old paths and
# does not complain about them: it simply generates fresh terrain over the top,
# and the recording then shows an empty world with no error anywhere. That cost
# a whole investigation once — a capture of the 3x3 record door that recorded
# nothing, which read as "the door is at rest" rather than "the door is not
# there". Move the folders, and let the game's own DataFixers handle the rest.
if [[ -n "${WORLD:-}" ]]; then
    echo "  world: recording $WORLD in place"
    rsync -a --exclude session.lock "$WORLD/" "$UNIVERSE/gametestworld/"
    mkdir -p "$UNIVERSE/gametestworld/datapacks"
    WORLDDIR="$UNIVERSE/gametestworld"
    if [[ -d "$WORLDDIR/region" ]]; then
        echo "  world: pre-26.2 save layout — relocating into dimensions/"
        mkdir -p "$WORLDDIR/dimensions/minecraft/overworld"
        for part in region entities poi data; do
            [[ -d "$WORLDDIR/$part" ]] || continue
            # `data/` at the root is world-wide (scoreboard, random sequences)
            # in both layouts, so only the dimension-scoped parts move.
            [[ "$part" == "data" ]] && continue
            mv "$WORLDDIR/$part" "$WORLDDIR/dimensions/minecraft/overworld/$part"
        done
        for pair in "DIM-1:the_nether" "DIM1:the_end"; do
            old="${pair%%:*}"; new="${pair##*:}"
            [[ -d "$WORLDDIR/$old" ]] || continue
            mkdir -p "$WORLDDIR/dimensions/minecraft/$new"
            for part in region entities poi; do
                [[ -d "$WORLDDIR/$old/$part" ]] || continue
                mv "$WORLDDIR/$old/$part" "$WORLDDIR/dimensions/minecraft/$new/$part"
            done
            rm -rf "$WORLDDIR/$old"
        done
    fi
fi
cp -R "$STAGE" "$UNIVERSE/gametestworld/datapacks/nucleation_tests"

java -cp "$WORK/classes:$CP" TraceCapture --universe "$UNIVERSE" "$@" \
    2>&1 | grep -viE 'WARNING:|sun\.misc|Saving |chunks are saved|dimensions are saved|Preparing ' || true
