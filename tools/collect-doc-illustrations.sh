#!/usr/bin/env bash
# Rebuild the downloadable archive used by docs/illustrations.md.
set -euo pipefail

cd "$(dirname "$0")/.."

output="docs/downloads/illustrations/nucleation-system-illustrations.zip"
mkdir -p "$(dirname "$output")"
temporary_dir="$(mktemp -d /tmp/nucleation-system-illustrations.XXXXXX)"
temporary="$temporary_dir/nucleation-system-illustrations.zip"
trap 'rm -rf "$temporary_dir"' EXIT

illustrations=(
  docs/media/kineglyph/fast-generation.svg \
  docs/media/kineglyph/shapes-and-brushes.svg \
  docs/media/kineglyph/sdf-and-fields.svg \
  docs/media/kineglyph/palettes-and-color.svg \
  docs/media/kineglyph/smart-simulation.svg \
  docs/media/kineglyph/formats-and-io.svg \
  docs/media/kineglyph/bindings-and-languages.svg \
  docs/media/kineglyph/meshing-and-rendering.svg
)

zip -j -X -q "$temporary" "${illustrations[@]}"

test "$(unzip -Z1 "$temporary" | wc -l | tr -d ' ')" = "8"
mkdir "$temporary_dir/extracted"
unzip -q "$temporary" -d "$temporary_dir/extracted"
for source in "${illustrations[@]}"; do
  cmp "$source" "$temporary_dir/extracted/$(basename "$source")"
done
mv "$temporary" "$output"
echo "Collected 8 SVG illustrations in $output"
