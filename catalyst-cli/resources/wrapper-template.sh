#!/bin/bash
# Auto-generated wrapper for {{BINARY_NAME}}
# Created by Catalyst CLI

# Try to find the binary in standard locations
BINARY_NAME="{{BINARY_NAME}}"

# Check standard installation location
BINARY_PATH="$HOME/.claude-hooks/bin/$BINARY_NAME"

# If not found, try local project build
if [ ! -f "$BINARY_PATH" ]; then
    # Try to find catalyst project directory
    if [ -n "$CATALYST_PROJECT_DIR" ]; then
        BINARY_PATH="$CATALYST_PROJECT_DIR/target/release/$BINARY_NAME"
    fi

    # If still not found, try relative to this script
    if [ ! -f "$BINARY_PATH" ]; then
        SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
        BINARY_PATH="$PROJECT_ROOT/catalyst/target/release/$BINARY_NAME"
    fi
fi

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: $BINARY_NAME binary not found" >&2
    echo "Searched locations:" >&2
    echo "  - $HOME/.claude-hooks/bin/$BINARY_NAME" >&2
    echo "  - \$CATALYST_PROJECT_DIR/target/release/$BINARY_NAME" >&2
    echo "" >&2
    echo "Please run: cd catalyst && ./install.sh" >&2
    exit 1
fi

# Execute the binary with stdin and arguments
cat | "$BINARY_PATH" "$@"
