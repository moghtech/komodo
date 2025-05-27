#!/bin/bash
set -e

# Komodo Periphery musl installation script
# Downloads and installs pre-built musl binaries for host systems

VERSION="${1:-latest}"
TARGET_ARCH="${2:-$(uname -m)}"
INSTALL_DIR="${3:-/usr/local/bin}"

echo "🦎 Installing Komodo Periphery (musl) on host system"
echo "Version: $VERSION"
echo "Architecture: $TARGET_ARCH"
echo "Install directory: $INSTALL_DIR"

# Map architecture names
case "$TARGET_ARCH" in
    "x86_64" | "amd64")
        ARCH_SUFFIX="x86_64"
        ;;
    "aarch64" | "arm64")
        ARCH_SUFFIX="aarch64"
        ;;
    *)
        echo "❌ Unsupported architecture: $TARGET_ARCH"
        echo "Supported: x86_64, amd64, aarch64, arm64"
        exit 1
        ;;
esac

# Check if running on musl system
if ldd --version 2>&1 | grep -q musl; then
    echo "✅ Detected musl-based system"
    BINARY_TYPE="musl"
elif ldd --version 2>&1 | grep -q glibc; then
    echo "⚠️  Detected glibc-based system"
    echo "Note: Installing musl binary on glibc system (should work due to static linking)"
    BINARY_TYPE="musl"
else
    echo "❓ Could not detect libc type, proceeding with musl binary"
    BINARY_TYPE="musl"
fi

# Determine download URL
if [ "$VERSION" = "latest" ]; then
    # For now, instruct manual build until releases include musl binaries
    echo "❌ Pre-built musl binaries not yet available in releases"
    echo ""
    echo "🔨 To build locally:"
    echo "  git clone https://github.com/moghtech/komodo.git"
    echo "  cd komodo"
    echo "  ./scripts/build-musl.sh $ARCH_SUFFIX"
    echo ""
    echo "📦 Or use Docker container:"
    echo "  docker pull ghcr.io/moghtech/periphery:latest-musl-$ARCH_SUFFIX"
    exit 1
else
    DOWNLOAD_URL="https://github.com/moghtech/komodo/releases/download/$VERSION/periphery-musl-$ARCH_SUFFIX"
fi

# Future implementation for when releases include musl binaries:
# TEMP_DIR=$(mktemp -d)
# trap "rm -rf $TEMP_DIR" EXIT
# 
# echo "📥 Downloading periphery-musl-$ARCH_SUFFIX..."
# curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_DIR/periphery"
# 
# # Verify it's a valid binary
# if ! file "$TEMP_DIR/periphery" | grep -q "executable"; then
#     echo "❌ Downloaded file is not a valid executable"
#     exit 1
# fi
# 
# # Install binary
# echo "📦 Installing to $INSTALL_DIR/periphery..."
# sudo cp "$TEMP_DIR/periphery" "$INSTALL_DIR/periphery"
# sudo chmod +x "$INSTALL_DIR/periphery"
# 
# # Verify installation
# if "$INSTALL_DIR/periphery" --version >/dev/null 2>&1; then
#     echo "✅ Installation successful!"
#     echo "📍 Installed: $INSTALL_DIR/periphery"
#     echo "🧪 Test: periphery --version"
# else
#     echo "❌ Installation verification failed"
#     exit 1
# fi