#!/usr/bin/env bash
# Run the vanilla oracle: structure tests executed inside real Minecraft, headless.
#
# See README.md for why this needs no Fabric, Loom, Yarn, Mixins or EULA — 26.2
# ships its server jar unobfuscated, and GameTestServer runs in-process.
set -euo pipefail
cd "$(dirname "$0")"

MC_VERSION="${MC_VERSION:-26.2}"
WORK="${WORK:-work}"
PACK_STAGE="$WORK/packs/nucleation_tests"
REPORT="$WORK/report.xml"

mkdir -p "$WORK"

# Each run creates a full Minecraft save — three dimensions with region files —
# and a session capturing many traces will fill a disk. The jar and compiled
# classes are worth caching; the world is not.
trap 'rm -rf "$WORK/universe"' EXIT

# --- acquire the server jar (cached) -----------------------------------------
# Mirrors the approach in tools/mc-data/refresh-block-data.rs, which already
# downloads and runs this jar for block data.
JAR="$WORK/server.jar"
if [[ ! -f "$JAR" ]]; then
    echo "==> fetching Minecraft $MC_VERSION server jar"
    URL=$(python3 - "$MC_VERSION" <<'PY'
import json, sys, urllib.request
version = sys.argv[1]
manifest = json.load(urllib.request.urlopen(
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"))
entry = next(v for v in manifest["versions"] if v["id"] == version)
print(json.load(urllib.request.urlopen(entry["url"]))["downloads"]["server"]["url"])
PY
)
    curl -sL --fail -o "$JAR" "$URL"
fi

# --- unpack the bundle and build a classpath ---------------------------------
# The published jar is a bundler: the real server and its libraries live inside.
INNER="$WORK/inner/META-INF/versions/$MC_VERSION/server-$MC_VERSION.jar"
if [[ ! -f "$INNER" ]]; then
    echo "==> extracting bundled server and libraries"
    rm -rf "$WORK/inner"
    unzip -o -q "$JAR" "META-INF/versions/*" "META-INF/libraries/*" -d "$WORK/inner"
fi
CP="$INNER:$(find "$WORK/inner/META-INF/libraries" -name '*.jar' | tr '\n' ':')"

# --- compile the two tiny drivers --------------------------------------------
mkdir -p "$WORK/classes"
echo "==> compiling"
javac -nowarn -cp "$CP" -d "$WORK/classes" src/Snbt2Nbt.java src/RunGameTests.java

# --- stage the datapack, converting .snbt structures to binary .nbt ----------
# Structures are authored as SNBT because text is reviewable; datapacks only load
# binary, so convert with the game's own parser.
echo "==> staging datapack"
rm -rf "$WORK/packs"
mkdir -p "$PACK_STAGE"
cp -R pack/. "$PACK_STAGE/"

EXPECTED=()
while IFS= read -r snbt; do
    rel="${snbt#pack/}"
    out="$PACK_STAGE/${rel%.snbt}.nbt"
    java -cp "$WORK/classes:$CP" Snbt2Nbt "$snbt" "$out" 2>/dev/null \
        || { echo "FAILED converting $snbt" >&2; exit 1; }
    rm -f "$PACK_STAGE/$rel"
done < <(find pack -name '*.snbt')

# Every declared test instance must show up in the report; see the note below.
while IFS= read -r json; do
    EXPECTED+=("$(basename "$json" .json)")
done < <(find pack -path '*/test_instance/*.json')

# --- run ----------------------------------------------------------------------
echo "==> running ${#EXPECTED[@]} test(s) in Minecraft $MC_VERSION"
# Each run makes a full Minecraft save; leaving them around fills disks.
rm -rf "$WORK/universe"
java -cp "$WORK/classes:$CP" RunGameTests \
    --universe "$WORK/universe" \
    --packs "$WORK/packs" \
    --report "$REPORT" \
    "$@" 2>&1 | grep -viE 'WARNING:|sun\.misc|Saving |chunks are saved|dimensions are saved|Preparing ' || true

# --- verify the tests actually ran -------------------------------------------
# The dangerous failure mode is a green run that tested nothing: with a malformed
# --packs layout the server happily reports "All 1 required tests passed" using
# only vanilla's own always_pass. Passing is not enough — the tests we declared
# have to be present in the report.
python3 - "$REPORT" "${EXPECTED[@]}" <<'PY'
import re, sys, pathlib

report = pathlib.Path(sys.argv[1])
expected = sys.argv[2:]

if not report.exists():
    sys.exit(f"no report written at {report}")

text = report.read_text()
ran = set(re.findall(r'<testcase[^>]*name="([^"]+)"', text))
failures = len(re.findall(r"<failure", text))

missing = [name for name in expected
           if not any(entry.split(":")[-1] == name for entry in ran)]

print(f"    ran: {sorted(ran)}")
if missing:
    sys.exit(f"FAIL: declared tests never ran: {missing}\n"
             f"      (a pass here would have been meaningless)")
if failures:
    sys.exit(f"FAIL: {failures} test failure(s) in {report}")
print(f"    OK: all {len(expected)} declared test(s) ran and passed")
PY
