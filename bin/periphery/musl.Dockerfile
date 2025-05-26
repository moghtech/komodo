## Dockerfile for building Komodo Periphery with musl target
FROM rust:1.82-alpine AS builder

# Install musl development tools and cross-compilation dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    gcc

# Set environment for static linking
ENV RUSTFLAGS="-C target-feature=+crt-static"

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY bin/ ./bin/
COPY lib/ ./lib/
COPY client/ ./client/

# Build for the native architecture only (no cross-compilation in alpine)
ARG TARGETPLATFORM
RUN mkdir -p target/release && \
    case "${TARGETPLATFORM}" in \
      "linux/amd64") \
        rustup target add x86_64-unknown-linux-musl && \
        cargo build --release --bin periphery --target x86_64-unknown-linux-musl && \
        cp target/x86_64-unknown-linux-musl/release/periphery target/release/periphery ;; \
      "linux/arm64") \
        rustup target add aarch64-unknown-linux-musl && \
        cargo build --release --bin periphery --target aarch64-unknown-linux-musl && \
        cp target/aarch64-unknown-linux-musl/release/periphery target/release/periphery ;; \
      *) echo "Unsupported platform: ${TARGETPLATFORM}" && exit 1 ;; \
    esac

# Runtime stage - use scratch for smallest possible image
FROM scratch

# Copy the statically linked binary
COPY --from=builder /app/target/release/periphery /periphery

EXPOSE 8120

LABEL org.opencontainers.image.source=https://github.com/mbecker20/komodo
LABEL org.opencontainers.image.description="Komodo Periphery (musl)"
LABEL org.opencontainers.image.licenses=GPL-3.0

CMD [ "/periphery" ]