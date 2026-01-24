#!/usr/bin/env bash
# Generate shell completions for alx
# Usage: ./scripts/generate-completions.sh <output-directory>

set -euo pipefail

OUTPUT_DIR="${1:-.}"

# Create completions directory
mkdir -p "$OUTPUT_DIR/completions"

# Generate Bash completion
./target/release/alx generate completion-bash > "$OUTPUT_DIR/completions/alx.bash"

# Generate Zsh completion
./target/release/alx generate completion-zsh > "$OUTPUT_DIR/completions/_alx"

# Generate Fish completion
./target/release/alx generate completion-fish > "$OUTPUT_DIR/completions/alx.fish"

echo "Completions generated in $OUTPUT_DIR/completions"
