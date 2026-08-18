#!/usr/bin/env bash
#
# Build the pagina static mirror of docs/, by way of a portable article bundle.
#
# The same script CI runs and a person runs, so "it works on my machine" and "it works in the
# workflow" are the same claim. `.github/workflows/docs.yml` calls it with no arguments beyond the
# output directory.
#
#   tools/build-pagina-mirror.sh [outdir]
#
# Outputs:
#   <outdir>/                 the static site, ready to serve at PAGINA_SITE_URL
#   <outdir>.pgz              the bundle it was built from — the same file schemat.io imports
#
# Environment:
#   PAGINA_REF        pagina commit to build with        (pinned below)
#   KINEGLYPH_REF     kineglyph commit to build with     (pinned below)
#   PAGINA_SITE_URL   full deployment URL, path included (the path becomes the site base)
#   PAGINA_MIRROR_OF  primary copy's URL; set, canonical/og:url point there and no sitemap is
#                     written. Leave empty until the primary actually serves the article — a
#                     canonical pointing at a 404 is worse than no canonical.
#   PAGINA_WORK       where the toolchain is cloned and built (default: .pagina-toolchain)
#
# ## Why the site is built from a bundle rather than from docs/
#
# `pagina build docs` copies **every file in the folder** as an asset. `docs/` is not only an
# article: it also holds `superpowers/` agent reports, `planning/`, `plans/`, a LaTeX source and a
# 60 MB tail of unreferenced media. MkDocs never published those because `mkdocs.yml` lists them
# under `exclude_docs`; pagina has no equivalent, so a direct build would put internal working
# notes on a public site — 118 MB of it.
#
# `pagina pack` resolves instead of copying: it reads the nav, walks what the pages actually
# reference, and pulls in exactly that (also resolving the `--8<--` snippets that live outside
# `docs/`, under `examples/`). Building the unpacked bundle therefore publishes the article and
# nothing else — 41 MB — and it renders **byte-identically**, page for page, to building `docs/`
# directly. That equality was verified across all 29 pages before this script was written, and
# running the loop on every build is what keeps it true.
#
# The bundle is the point rather than a means: it is the same artefact that gets imported into
# schemat.io, so CI proves on every run that the file the CMS will import is the file the mirror
# was rendered from.
#
# Neither pagina nor kineglyph is published to npm, so the toolchain is built from pinned commits
# rather than installed. Pinning by full SHA is what makes a docs build reproducible: a branch name
# would let an unrelated upstream commit change this site without a commit here.
set -euo pipefail

PAGINA_REF="${PAGINA_REF:-d758d014e98c4248fb31107bf5cc68891b844fe1}"
KINEGLYPH_REF="${KINEGLYPH_REF:-1422ee8511d709962d01cffe86f1a255496ff6b2}"
PAGINA_REPO="${PAGINA_REPO:-https://github.com/Nano112/pagina.git}"
KINEGLYPH_REPO="${KINEGLYPH_REPO:-https://github.com/Nano112/kineglyph.git}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$REPO_ROOT/site-pagina}"
WORK="${PAGINA_WORK:-$REPO_ROOT/.pagina-toolchain}"
SITE_URL="${PAGINA_SITE_URL:-https://schem-at.github.io/Nucleation/pagina/}"
MIRROR_OF="${PAGINA_MIRROR_OF:-}"

# The deployment URL's path is the site base. pagina derives it for `build`; `pack` needs it told.
BASE="/$(printf '%s' "${SITE_URL#*://}" | cut -s -d/ -f2-)"
[ "$BASE" = "/" ] || BASE="/${BASE#/}"
case "$BASE" in */) ;; *) BASE="$BASE/" ;; esac

say() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }

# ---- 1. the toolchain, at pinned commits --------------------------------------------------------
clone_at() {           # clone_at <repo> <ref> <dir>
  local repo="$1" ref="$2" dir="$3"
  if [ -d "$dir/.git" ]; then
    git -C "$dir" fetch --quiet origin "$ref" || git -C "$dir" fetch --quiet origin
  else
    mkdir -p "$dir"
    git -C "$dir" init --quiet
    git -C "$dir" remote add origin "$repo"
    git -C "$dir" fetch --quiet --depth 1 origin "$ref"
  fi
  git -C "$dir" checkout --quiet --force "$ref"
}

say "kineglyph @ $KINEGLYPH_REF"
clone_at "$KINEGLYPH_REPO" "$KINEGLYPH_REF" "$WORK/kineglyph"
( cd "$WORK/kineglyph" && npm ci --no-audit --no-fund && npm run build )

say "pagina @ $PAGINA_REF"
clone_at "$PAGINA_REPO" "$PAGINA_REF" "$WORK/pagina"
(
  cd "$WORK/pagina"
  npm ci --no-audit --no-fund
  # kineglyph is an unpublished peer dependency: resolved from the sibling checkout rather than the
  # registry. Symlinked inside the work tree, so nothing is written to a global npm prefix that a
  # CI runner may not have.
  mkdir -p node_modules/@kineglyph
  for p in core svg anime plot scenes web export; do
    rm -rf "node_modules/@kineglyph/$p"
    ln -s "$WORK/kineglyph/packages/$p" "node_modules/@kineglyph/$p"
  done
  npm run build
)

PAGINA="node $WORK/pagina/packages/cli/dist/cli.js"
LOG="$(mktemp)"
trap 'rm -f "$LOG"' EXIT

# Fails the build on pagina diagnostics — the `[severity] code page: message` lines. Plain
# `pagina:` lines are build output (the sub-path robots.txt note among them) and are information,
# not faults. A strict build already fails on content *errors*; this additionally fails on
# warnings, because a docs site that deploys with a dead link, a missing description or an omitted
# canonical has published its defects silently, which is worse than a red build.
#
# One code is tolerated, and only from `pack`: `bundle-external-ref`. A bundle cannot inline the
# internet, so pagina reports each external reference by design rather than as a fault — the
# gallery pages legitimately point at raw.githubusercontent.com. Naming the single code rather than
# waiving warnings wholesale means any *new* kind of warning still stops the build.
refuse_diagnostics() {          # refuse_diagnostics <log> [tolerated-code]
  local log="$1" tolerated="${2:-}" found
  found="$(grep -E '^\[(error|warning)\] ' "$log" || true)"
  if [ -n "$tolerated" ]; then
    found="$(printf '%s\n' "$found" | grep -v "^\[warning\] $tolerated " || true)"
  fi
  if [ -n "$found" ]; then
    echo >&2
    echo "pagina reported diagnostics; refusing to publish the mirror:" >&2
    printf '%s\n' "$found" >&2
    exit 1
  fi
}

run_pagina() {                  # run_pagina <tolerated-code> <args...>
  local tolerated="$1"; shift
  set +e
  $PAGINA "$@" 2>&1 | tee "$LOG"
  local status="${PIPESTATUS[0]}"
  set -e
  [ "$status" -eq 0 ] || { echo "pagina $1 failed (exit $status)" >&2; exit "$status"; }
  refuse_diagnostics "$LOG" "$tolerated"
}

# ---- 2. pack the article out of the repo --------------------------------------------------------
say "packing docs/ into a bundle (base $BASE)"
BUNDLE="$OUT.pgz"
rm -f "$BUNDLE"
run_pagina bundle-external-ref pack "$REPO_ROOT/docs" -o "$BUNDLE" --base "$BASE"

# ---- 3. unpack it, and build *that* -------------------------------------------------------------
# Deliberately built from the unpacked copy rather than from `docs/`: that is the arrow the bundle
# format exists to make work, and exercising it here means a bundle that has stopped round-tripping
# fails the docs build instead of failing silently at import time.
say "unpacking the bundle"
UNPACKED="$WORK/unpacked"
rm -rf "$UNPACKED"
run_pagina "" unpack "$BUNDLE" "$UNPACKED"

say "building the mirror into $OUT"
rm -rf "$OUT"
# shellcheck disable=SC2086
run_pagina "" build "$UNPACKED" \
  --out "$OUT" \
  --site-url "$SITE_URL" \
  ${MIRROR_OF:+--mirror-of "$MIRROR_OF"}

say "mirror built: $(find "$OUT" -type f | wc -l | tr -d ' ') files at $SITE_URL"
say "bundle: $BUNDLE ($(du -h "$BUNDLE" | cut -f1))"
