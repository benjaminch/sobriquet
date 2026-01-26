#!/bin/sh
# Install git hooks

HOOKS_DIR=".githooks"
GIT_HOOKS_DIR=".git/hooks"

if [ ! -d "$GIT_HOOKS_DIR" ]; then
    echo "Error: Not in a git repository"
    exit 1
fi

echo "Installing git hooks..."

# Install pre-commit hook
if [ -f "$HOOKS_DIR/pre-commit" ]; then
    cp "$HOOKS_DIR/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
    chmod +x "$GIT_HOOKS_DIR/pre-commit"
    echo "✅ Installed pre-commit hook"
else
    echo "❌ pre-commit hook not found in $HOOKS_DIR"
    exit 1
fi

echo "✅ All hooks installed successfully!"
echo ""
echo "To bypass hooks when committing (not recommended), use:"
echo "  git commit --no-verify"
