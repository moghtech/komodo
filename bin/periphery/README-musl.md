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

Replace the periphery image in your compose files:

```yaml
periphery:
  image: ghcr.io/moghtech/periphery:latest-musl
  # ... rest of config
```

## Technical Details

- Uses `rust:1.82-alpine` base image for modern Cargo.lock compatibility
- Builds native architecture binaries (no cross-compilation needed)
- Statically links with musl for self-contained binaries
- Results in minimal scratch-based runtime images
- Compilation time: ~5-10 minutes depending on architecture

## Benefits

- **Smaller images**: Static linking with musl creates minimal container sizes
- **Security**: Reduced attack surface with fewer dependencies  
- **Portability**: Self-contained binaries work across different environments
- **Performance**: Potential performance improvements in some scenarios

## Migration Notes

For existing deployments, update your image references gradually:

1. Test with single-arch builds first (`-amd64` or `-arm64` suffixes)
2. Monitor functionality with the new musl-based images
3. Switch to multi-arch builds (`-musl` suffix) for production