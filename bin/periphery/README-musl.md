# Komodo Periphery - Musl Builds

This directory contains Docker configurations for building Komodo Periphery with musl support, creating statically linked binaries for smaller and more secure container images.

## Image Tags

The musl builds create unique image names to allow migration:

- `ghcr.io/moghtech/periphery:latest-musl` - Multi-arch musl build
- `ghcr.io/moghtech/periphery:latest-musl-amd64` - x86_64 musl build 
- `ghcr.io/moghtech/periphery:latest-musl-arm64` - aarch64 musl build

## Build Commands

### Multi-arch build:
```bash
docker buildx build --platform linux/amd64,linux/arm64 \
  -f bin/periphery/musl.Dockerfile \
  -t ghcr.io/moghtech/periphery:latest-musl \
  --push .
```

### Single architecture builds:
```bash
# AMD64
docker buildx build --platform linux/amd64 \
  -f bin/periphery/musl.Dockerfile \
  -t ghcr.io/moghtech/periphery:latest-musl-amd64 \
  --push .

# ARM64  
docker buildx build --platform linux/arm64 \
  -f bin/periphery/musl.Dockerfile \
  -t ghcr.io/moghtech/periphery:latest-musl-arm64 \
  --push .
```

## Usage

### Docker Containers

Replace the periphery image in your compose files:

```yaml
periphery:
  image: ghcr.io/moghtech/periphery:latest-musl
  # ... rest of config
```

### Host Installation

#### Option 1: Build from source
```bash
# Clone repository
git clone https://github.com/moghtech/komodo.git
cd komodo

# Install musl development tools (Ubuntu/Debian)
sudo apt update && sudo apt install musl-tools

# Build musl binary
./scripts/build-musl.sh

# Install binary
sudo cp target/musl-release/periphery-musl-$(uname -m) /usr/local/bin/periphery
sudo chmod +x /usr/local/bin/periphery

# Verify installation
periphery --version
```

#### Option 2: Pre-built binaries (future)
```bash
# Download and install (when available in releases)
curl -fsSL https://github.com/moghtech/komodo/raw/main/scripts/install-musl.sh | sudo bash
```

## Technical Details

- Uses `rust:1.82-alpine` base image for modern Cargo.lock compatibility
- Builds native architecture binaries (no cross-compilation needed)
- Statically links with musl for self-contained binaries
- Results in minimal scratch-based runtime images
- Compilation time: ~5-10 minutes depending on architecture

## Benefits

- **Smaller images**: Static linking with musl creates minimal container sizes (~15MB vs ~100MB+)
- **Security**: Reduced attack surface with fewer dependencies  
- **Portability**: Self-contained binaries work across different environments
- **Performance**: Potential performance improvements in some scenarios
- **Host compatibility**: Static binaries work on both musl and glibc systems

## Migration Notes

For existing deployments, update your image references gradually:

1. Test with single-arch builds first (`-amd64` or `-arm64` suffixes)
2. Monitor functionality with the new musl-based images
3. Switch to multi-arch builds (`-musl` suffix) for production