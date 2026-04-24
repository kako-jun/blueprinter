#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fixtures_dir="$repo_root/tests/fixtures/mermaid"
output_root="${1:-$repo_root/target/poc}"
seed="${BLUEPRINTER_POC_SEED:-42}"

baseline_dir="$output_root/baseline"
transformed_dir="$output_root/blueprinter"

mkdir -p "$baseline_dir" "$transformed_dir"

run_mmdc() {
  env -u NODE_OPTIONS mmdc "$@"
}

echo "Generating Mermaid PoC outputs..."
echo "  fixtures: $fixtures_dir"
echo "  output:   $output_root"
echo "  seed:     $seed"

for fixture in "$fixtures_dir"/*.mmd; do
  name="$(basename "$fixture" .mmd)"
  baseline_svg="$baseline_dir/$name.svg"
  transformed_svg="$transformed_dir/$name.svg"

  echo
  echo "[$name] Mermaid -> SVG"
  run_mmdc -i "$fixture" -o "$baseline_svg" -b transparent

  echo "[$name] SVG -> blueprinter"
  cargo run --manifest-path "$repo_root/Cargo.toml" -- \
    transform \
    --input "$baseline_svg" \
    --output "$transformed_svg" \
    --theme blueprint \
    --seed "$seed"
done

cat <<EOF

PoC generation complete.

Compare these pairs side by side:
  baseline:    $baseline_dir/*.svg
  transformed: $transformed_dir/*.svg

Evaluation prompts:
  - Does the transformed version feel more human without losing legibility?
  - Which diagram type benefits most from jitter?
  - Where do labels, arrows, or dense edges become noisy instead of charming?
  - Is blueprint already interesting enough to justify deeper theme work?
EOF
