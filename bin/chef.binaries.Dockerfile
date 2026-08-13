## Builds the Komodo Core, Periphery, and Util binaries
## for a specific architecture. Requires OpenSSL 3 or later.

## Uses chef for dependency caching to help speed up back-to-back builds.

FROM lukemathwalker/cargo-chef:latest-rust-1.97.1-bookworm AS chef
WORKDIR /builder

# Plan just the RECIPE to see if things have changed
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN cargo install cargo-strip cargo-edit

# Build JUST dependencies - cached layer
COPY --from=planner /builder/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# NOW copy again (this time into builder), set version, and build app
COPY . .

# Set Version
ARG VERSION="0.0.0"
ARG IMAGE_TAG=""
RUN cargo set-version ${VERSION}${IMAGE_TAG:+-${IMAGE_TAG}}

RUN \
  cargo build --release --bin core && \
  cargo build --release --bin periphery && \
  cargo build --release --bin km && \
  cargo strip

# Copy just the binaries to scratch image
FROM scratch

COPY --from=builder /builder/target/release/core /core
COPY --from=builder /builder/target/release/periphery /periphery
COPY --from=builder /builder/target/release/km /km

LABEL org.opencontainers.image.source="https://github.com/moghtech/komodo"
LABEL org.opencontainers.image.description="Komodo Binaries"
LABEL org.opencontainers.image.licenses="GPL-3.0"