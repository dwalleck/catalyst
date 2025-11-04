#!/bin/bash
set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Catalyst - Claude Code Hooks Installer"
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

# Parse options
BUILD_SQLITE=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --sqlite)
            BUILD_SQLITE=true
            shift
            ;;
        --help)
            echo "Usage: ./install.sh [OPTIONS]"
            echo
            echo "Options:"
            echo "  --sqlite    Build with SQLite support for state management"
            echo "  --help      Show this help message"
            echo
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

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

# Check if we're in the catalyst directory
if [ ! -f "Cargo.toml" ]; then
    echo "⚠️  Cargo.toml not found"
    echo "Please run this script from the catalyst repository root"
    exit 1
fi

# Copy source code
echo "📦 Copying source code..."
cp -r . "$SRC_DIR/"
cd "$SRC_DIR"

# Build release binaries
echo
if [ "$BUILD_SQLITE" = true ]; then
    echo "🔨 Building release binaries with SQLite support..."
    cargo build --release --features sqlite
else
    echo "🔨 Building release binaries (core only)..."
    echo "   (Use --sqlite to enable SQLite-backed state management)"
    cargo build --release
fi

# Copy core binaries
echo
echo "📦 Installing binaries to $BIN_DIR..."
for binary in target/release/skill-activation-prompt target/release/file-analyzer target/release/cargo-check; do
    if [ -f "$binary" ]; then
        cp "$binary" "$BIN_DIR/"
        chmod +x "$BIN_DIR/$(basename $binary)"
        echo "   ✅ Installed: $(basename $binary)"
    fi
done

# Copy SQLite binaries if built
if [ "$BUILD_SQLITE" = true ]; then
    if [ -f "target/release/post-tool-use-tracker-sqlite" ]; then
        cp "target/release/post-tool-use-tracker-sqlite" "$BIN_DIR/"
        chmod +x "$BIN_DIR/post-tool-use-tracker-sqlite"
        echo "   ✅ Installed: post-tool-use-tracker-sqlite"
    fi
fi

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Installation Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Binaries installed to: $BIN_DIR"
echo
echo "Next steps:"
echo
echo "1. (Optional) Add to your PATH:"
echo "   echo 'export PATH=\"\$HOME/.claude-hooks/bin:\$PATH\"' >> ~/.bashrc"
echo
echo "2. In your Claude Code project, create wrapper scripts:"
echo
echo "   cd your-project/.claude/hooks/"
echo
echo "   # Skill activation hook (essential)"
echo "   cat > skill-activation-prompt.sh << 'EOF'"
echo "#!/bin/bash"
echo "cat | ~/.claude-hooks/bin/skill-activation-prompt"
echo "EOF"
echo "   chmod +x skill-activation-prompt.sh"
echo
echo "3. Configure .claude/settings.json:"
echo "   See docs/standalone-installation.md for details"
echo
echo "📚 Documentation:"
echo "   - README.md                       - Getting started"
echo "   - docs/standalone-installation.md - Full setup guide"
echo "   - docs/databases.md               - SQLite state management"
echo
