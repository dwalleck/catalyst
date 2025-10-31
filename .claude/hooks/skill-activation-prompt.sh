#!/bin/bash
set -e

# Skill activation hook - calls Rust implementation
# For standalone installation (recommended):
#   Install once: ./install.sh
#   Binaries installed to ~/.claude-hooks/bin/

# Check if standalone binary exists
if [ -f "$HOME/.claude-hooks/bin/skill-activation-prompt" ]; then
    cat | "$HOME/.claude-hooks/bin/skill-activation-prompt"
# Otherwise check for project-local build
elif [ -f "$CLAUDE_PROJECT_DIR/target/release/skill-activation-prompt" ]; then
    cat | "$CLAUDE_PROJECT_DIR/target/release/skill-activation-prompt"
else
    echo "❌ Rust hook not found!" >&2
    echo "Install with: ./install.sh" >&2
    echo "Or build locally: cargo build --release" >&2
    exit 1
fi
