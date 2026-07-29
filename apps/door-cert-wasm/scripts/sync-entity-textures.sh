#!/usr/bin/env bash
# Inject vanilla entity textures into the app's mesher pack.
#
# WHY THIS EXISTS
#
# `apps/*/public/pack/` is gitignored build output, and the pack as generated
# contains only `assets/minecraft/textures/block` — no entity textures at all.
# The mesher does not fail on a missing texture: it atlases the magenta
# "missing" tile, so a minecart renders as a magenta hull and nothing anywhere
# reports a problem. Entity rendering therefore breaks silently every time the
# pack is regenerated, unless this runs after it.
#
# Run it after regenerating the pack, or whenever a cart/shulker turns magenta.
#
#   ./scripts/sync-entity-textures.sh [path-to-vanilla-resource-pack-root]
#
# The argument is a directory containing `assets/minecraft/textures/entity`.
# Defaults to a local vanilla pack if one is present.
set -euo pipefail
cd "$(dirname "$0")/.."

PACK="public/pack/mesher-pack.zip"
SRC="${1:-$HOME/Documents/VanillaDefault 1.21.5}"

if [[ ! -f "$PACK" ]]; then
    echo "no pack at $PACK — regenerate it first" >&2
    exit 1
fi
if [[ ! -d "$SRC/assets/minecraft/textures/entity" ]]; then
    echo "no entity textures under '$SRC'" >&2
    echo "pass the root of a vanilla resource pack (the dir holding assets/)" >&2
    exit 1
fi

# Only the families the mesher actually draws. Adding the whole entity tree
# would bloat the pack the browser downloads for no benefit.
WANT=(entity/minecart.png entity/shulker entity/chest)

before=$(unzip -l "$PACK" | tail -1)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
for rel in "${WANT[@]}"; do
    src="$SRC/assets/minecraft/textures/$rel"
    [[ -e "$src" ]] || { echo "skip (absent): $rel"; continue; }
    mkdir -p "$tmp/assets/minecraft/textures/$(dirname "$rel")"
    cp -R "$src" "$tmp/assets/minecraft/textures/$rel"
done
(cd "$tmp" && zip -q -r "$OLDPWD/$PACK" assets)

echo "before: $before"
echo "after : $(unzip -l "$PACK" | tail -1)"
echo "entity textures now in pack: $(unzip -l "$PACK" | grep -c '/entity/')"
