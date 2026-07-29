#!/usr/bin/env bash
# Answers: does Entity.setDeltaMovement(Vec3) guard against non-finite vectors,
# and in which Minecraft version did that guard appear?
#
# Pre-26.2 server jars are obfuscated, but the version manifest carries
# `server_mappings` (ProGuard, deobf -> obf) through 1.21.11, so `javap` on the
# obfuscated class + a name lookup is enough. 26.2 ships unobfuscated.
#
#   bash tools/gametest/nan-motion-bisect.sh 1.21.3
#
# Requires: java/javap (any JDK 21+), unzip, python3. No EULA, no server boot.
set -euo pipefail

VER="${1:-1.21.3}"
W="$(cd "$(dirname "$0")" && pwd)/work-bisect/$VER"
mkdir -p "$W"

# --- fetch (curl/wget are blocked in this environment; use python's urllib) ---
python3 - "$VER" "$W" <<'PY'
import json, sys, urllib.request, os
ver, W = sys.argv[1], sys.argv[2]
man = json.load(urllib.request.urlopen(
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json"))
entry = next(v for v in man["versions"] if v["id"] == ver)
meta = json.load(urllib.request.urlopen(entry["url"]))
d = meta["downloads"]
for key, name in (("server", "server.jar"), ("server_mappings", "server.txt")):
    if key not in d:
        print(f"NOTE: {ver} has no {key} (jar is unobfuscated)"); continue
    out = os.path.join(W, name)
    if not os.path.exists(out) or os.path.getsize(out) != d[key]["size"]:
        urllib.request.urlretrieve(d[key]["url"], out)
    print(f"{name}: {os.path.getsize(out)} bytes")
PY

# --- unpack the bundler: real server classes live in META-INF/versions/<ver>/ ---
rm -rf "$W/inner" "$W/srv"
unzip -q -o "$W/server.jar" -d "$W/inner"
unzip -q -o "$W/inner/META-INF/versions/$VER/server-$VER.jar" -d "$W/srv"
echo "world_version: $(python3 -c "import json;print(json.load(open('$W/srv/version.json'))['world_version'])")"

# --- resolve obfuscated names (no-op when mappings are absent, e.g. 26.x) ---
read -r ENT VEC SETTER <<<"$(python3 - "$W" <<'PY'
import os, sys
W = sys.argv[1]
p = os.path.join(W, "server.txt")
if not os.path.exists(p):
    print("net.minecraft.world.entity.Entity net.minecraft.world.phys.Vec3 setDeltaMovement"); raise SystemExit
cur = ent = vec = setter = None
for ln in open(p):
    ln = ln.rstrip("\n")
    if not ln.startswith("    ") and ln.endswith(":"):
        de, ob = ln[:-1].split(" -> ")
        cur = de
        if de == "net.minecraft.world.entity.Entity": ent = ob
        if de == "net.minecraft.world.phys.Vec3":     vec = ob
    elif cur == "net.minecraft.world.entity.Entity" and \
         "setDeltaMovement(net.minecraft.world.phys.Vec3)" in ln:
        setter = ln.strip().rsplit(" -> ", 1)[1]
print(ent, vec, setter)
PY
)"
echo "Entity -> $ENT | Vec3 -> $VEC | setDeltaMovement(Vec3) -> $SETTER"

echo
echo "===== $VER  Entity.setDeltaMovement(Vec3) ====="
javap -p -c -cp "$W/srv" "$ENT" \
  | awk -v s="$SETTER" -v v="$VEC" '$0 ~ ("void " s "\\(" v "\\);"), /^$/'

echo "===== $VER  Vec3.isFinite (absent before 1.21.11) ====="
javap -p -c -cp "$W/srv" "$VEC" | awk '/isFinite/,/^$/' || true

echo "===== $VER  Entity.load: Motion clamp + which fields are finite-checked ====="
javap -p -c -cp "$W/srv" "$ENT" \
  | grep -E 'String (Motion|Pos|Rotation)|Double\.isFinite|invalid (position|rotation)' || true
