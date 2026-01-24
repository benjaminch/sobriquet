#!/usr/bin/env bash
# Generate shell completions for alxrs
# Usage: ./scripts/generate-completions.sh <output-directory>

set -euo pipefail

OUTPUT_DIR="${1:-.}"

# Create completions directory
mkdir -p "$OUTPUT_DIR/completions"

# Generate Bash completion
./target/release/alxrs generate completion-bash > "$OUTPUT_DIR/completions/alxrs.bash"

# Generate Zsh completion
./target/release/alxrs generate completion-zsh > "$OUTPUT_DIR/completions/_alxrs"

# Generate Fish completion
./target/release/alxrs generate completion-fish > "$OUTPUT_DIR/completions/alxrs.fish"

echo "Completions generated in $OUTPUT_DIR/completions"
