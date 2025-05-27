#!/bin/bash
set -e

# Komodo Periphery musl build script
# Builds statically linked musl binaries for host installation

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TARGET_ARCH="${1:-$(uname -m)}"

echo "🦎 Building Komodo Periphery with musl for host installation"
echo "Target architecture: $TARGET_ARCH"

# Map architecture names
case "$TARGET_ARCH" in
    "x86_64" | "amd64")
        RUST_TARGET="x86_64-unknown-linux-musl"
        ;;
    "aarch64" | "arm64")
        RUST_TARGET="aarch64-unknown-linux-musl"
        ;;
    *)
        echo "❌ Unsupported architecture: $TARGET_ARCH"
        echo "Supported: x86_64, amd64, aarch64, arm64"
        exit 1
        ;;
esac

echo "Rust target: $RUST_TARGET"

# Check if musl-tools is installed
if ! command -v musl-gcc &> /dev/null; then
    echo "❌ musl-gcc not found. Please install musl development tools:"
    echo ""
    echo "Ubuntu/Debian:"
    echo "  sudo apt update && sudo apt install musl-tools"
    echo ""
    echo "Alpine:"
    echo "  apk add musl-dev gcc"
    echo ""
    echo "macOS (via Homebrew):"
    echo "  brew install FiloSottile/musl-cross/musl-cross"
    echo ""
    exit 1
fi

# Install Rust target if not present
if ! rustup target list --installed | grep -q "$RUST_TARGET"; then
    echo "📦 Installing Rust target: $RUST_TARGET"
    rustup target add "$RUST_TARGET"
fi

# Set up environment for static linking
export RUSTFLAGS="-C target-feature=+crt-static"

# Handle cross-compilation linker
case "$RUST_TARGET" in
    "aarch64-unknown-linux-musl")
        if command -v aarch64-linux-musl-gcc &> /dev/null; then
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="aarch64-linux-musl-gcc"
            export CC="aarch64-linux-musl-gcc"
        elif command -v aarch64-linux-gnu-gcc &> /dev/null; then
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="aarch64-linux-gnu-gcc"
            export CC="aarch64-linux-gnu-gcc"
        else
            echo "⚠️  Cross-compilation linker not found for ARM64"
            echo "Install: sudo apt install gcc-aarch64-linux-gnu"
        fi
        ;;
esac

cd "$PROJECT_ROOT"

echo "🔨 Building periphery binary..."
cargo build --release --bin periphery --target "$RUST_TARGET"

# Create output directory
OUTPUT_DIR="$PROJECT_ROOT/target/musl-release"
mkdir -p "$OUTPUT_DIR"

# Copy binary with architecture suffix
BINARY_PATH="$PROJECT_ROOT/target/$RUST_TARGET/release/periphery"
OUTPUT_PATH="$OUTPUT_DIR/periphery-musl-$TARGET_ARCH"

if [ -f "$BINARY_PATH" ]; then
    cp "$BINARY_PATH" "$OUTPUT_PATH"
    
    # Make executable
    chmod +x "$OUTPUT_PATH"
    
    # Get binary info
    BINARY_SIZE=$(du -h "$OUTPUT_PATH" | cut -f1)
    
    echo "✅ Build complete!"
    echo "📍 Binary location: $OUTPUT_PATH"
    echo "📏 Binary size: $BINARY_SIZE"
    echo ""
    echo "📋 Installation:"
    echo "  sudo cp $OUTPUT_PATH /usr/local/bin/periphery"
    echo "  sudo chmod +x /usr/local/bin/periphery"
    echo ""
    echo "🧪 Test installation:"
    echo "  $OUTPUT_PATH --version"
else
    echo "❌ Build failed: Binary not found at $BINARY_PATH"
    exit 1
fi