#!/usr/bin/env bash
# Fetch lithium's gametest structures into tests/corpus/lithium/ (git-ignored).
#
# Lithium is LGPL-3.0 and this repository is MIT, so the structures are
# fetched at a pinned commit rather than vendored. Our test descriptors for
# them live in tests/corpus/lithium-specs/, matched by relative path; run the
# pair with:
#
#   cargo run -p nucleation-cli -- \
#       test tests/corpus/lithium --specs tests/corpus/lithium-specs
set -euo pipefail

# develop HEAD as of 2026-08-03. Override with LITHIUM_REF=<commit> to move.
PIN="${LITHIUM_REF:-c42972b6e9d21c8ff45559df6b271802050a22e2}"
SRC_SUBDIR="common/src/gametest/resources/data/lithium-gametest"
DEST="$(cd "$(dirname "$0")/.." && pwd)/tests/corpus/lithium"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

git clone --filter=blob:none --no-checkout https://github.com/CaffeineMC/lithium.git "$TMP/lithium"
git -C "$TMP/lithium" sparse-checkout set "$SRC_SUBDIR"
git -C "$TMP/lithium" checkout "$PIN"

rm -rf "$DEST"
mkdir -p "$DEST"
(cd "$TMP/lithium/$SRC_SUBDIR" \
    && rsync -a --include='*/' --include='*.snbt' --exclude='*' ./ "$DEST/")
# rsync leaves empty directories behind for non-snbt subtrees; drop them.
find "$DEST" -type d -empty -delete

echo "fetched $(find "$DEST" -name '*.snbt' | wc -l | tr -d ' ') structures at $PIN into $DEST"
