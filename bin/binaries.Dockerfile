## Builds the Komodo Core and Periphery binaries
## for a specific architecture.

FROM rust:1.84.1-bullseye AS builder

WORKDIR /builder
COPY Cargo.toml Cargo.lock ./
COPY ./lib ./lib
COPY ./client/core/rs ./client/core/rs
COPY ./client/periphery ./client/periphery
COPY ./bin/core ./bin/core
COPY ./bin/periphery ./bin/periphery

# Compile bin
RUN \
  cargo build -p komodo_core --release && \
  cargo build -p komodo_periphery --release

# Copy just the binaries to scratch image
FROM scratch

COPY --from=builder /builder/target/release/core /core
COPY --from=builder /builder/target/release/periphery /periphery

LABEL org.opencontainers.image.source=https://github.com/mbecker20/komodo
LABEL org.opencontainers.image.description="Komodo Periphery"
LABEL org.opencontainers.image.licenses=GPL-3.0