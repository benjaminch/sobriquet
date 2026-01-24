#!/usr/bin/env bash
# Generate shell completions for sobriquet
# Usage: ./scripts/generate-completions.sh <output-directory>

set -euo pipefail

OUTPUT_DIR="${1:-.}"

# Create completions directory
mkdir -p "$OUTPUT_DIR/completions"

# Generate Bash completion
./target/release/sobriquet generate completion-bash > "$OUTPUT_DIR/completions/sobriquet.bash"

# Generate Zsh completion
./target/release/sobriquet generate completion-zsh > "$OUTPUT_DIR/completions/_sobriquet"

# Generate Fish completion
./target/release/sobriquet generate completion-fish > "$OUTPUT_DIR/completions/sobriquet.fish"

echo "Completions generated in $OUTPUT_DIR/completions"
