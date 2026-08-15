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
  docs/media/readme/fast-generation/operation-map.svg \
  docs/media/readme/shapes-brushes/shape-brush-map.svg \
  docs/media/readme/sdf-and-fields/sdf-field-pipeline.svg \
  docs/media/readme/palettes-and-color/color-pipeline.svg \
  docs/media/readme/smart-simulation/choose-engine.svg \
  docs/media/readme/formats-and-io/format-pipeline.svg \
  docs/media/readme/bindings-and-languages/binding-pipeline.svg \
  docs/media/readme/meshing-and-rendering/render-pipeline.svg
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
