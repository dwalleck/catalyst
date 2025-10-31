#!/bin/bash
set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Claude Code Rust Hooks Installer"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    echo
    echo "Install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo
    exit 1
fi

echo "✅ Rust found: $(rustc --version)"
echo

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

echo "Detected: $OS / $ARCH"
echo

# Installation directory
INSTALL_DIR="${HOME}/.claude-hooks"
BIN_DIR="${INSTALL_DIR}/bin"
SRC_DIR="${INSTALL_DIR}/src"

# Create directories
echo "📁 Creating directories..."
mkdir -p "$BIN_DIR"
mkdir -p "$SRC_DIR"

# Copy source if we're in the RustHooks directory
if [ -f "Cargo.toml" ]; then
    echo "📦 Copying source code..."
    cp -r . "$SRC_DIR/"
    cd "$SRC_DIR"
else
    echo "⚠️  Not in RustHooks directory"
    echo "Please cd to the RustHooks directory first"
    exit 1
fi

# Build release binaries
echo
echo "🔨 Building release binaries (this may take a minute)..."
cargo build --release

# Copy binaries to bin directory
echo "📦 Installing binaries to $BIN_DIR..."
for binary in target/release/skill-activation-prompt target/release/file-analyzer; do
    if [ -f "$binary" ]; then
        cp "$binary" "$BIN_DIR/"
        chmod +x "$BIN_DIR/$(basename $binary)"
        echo "   ✅ Installed: $(basename $binary)"
    fi
done

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Installation Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Binaries installed to: $BIN_DIR"
echo
echo "Next steps:"
echo "1. Add to your PATH (optional):"
echo "   echo 'export PATH=\"\$HOME/.claude-hooks/bin:\$PATH\"' >> ~/.bashrc"
echo
echo "2. In each project, create a wrapper script:"
echo "   cd your-project/.claude/hooks"
echo "   cat > skill-activation-prompt.sh << 'EOF'"
echo "   #!/bin/bash"
echo "   cat | ~/.claude-hooks/bin/skill-activation-prompt"
echo "   EOF"
echo "   chmod +x skill-activation-prompt.sh"
echo
echo "3. Update .claude/settings.json to use the wrapper"
echo
echo "See STANDALONE_INSTALLATION.md for detailed instructions."
echo
