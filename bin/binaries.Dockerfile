## Builds the Komodo Core, Periphery, and Util binaries
## for a specific architecture.

FROM rust:1.89.0-bullseye AS builder

WORKDIR /builder
COPY Cargo.toml Cargo.lock ./
COPY ./lib ./lib
COPY ./client/core/rs ./client/core/rs
COPY ./client/periphery ./client/periphery
COPY ./bin/core ./bin/core
COPY ./bin/periphery ./bin/periphery
COPY ./bin/cli ./bin/cli

# Compile bin
RUN \
  cargo build -p komodo_core --release && \
  cargo build -p komodo_periphery --release && \
  cargo build -p komodo_cli --release

# Copy just the binaries to scratch image
FROM scratch

COPY --from=builder /builder/target/release/core /core
COPY --from=builder /builder/target/release/periphery /periphery
COPY --from=builder /builder/target/release/km /km

LABEL org.opencontainers.image.source=https://github.com/moghtech/komodo
LABEL org.opencontainers.image.description="Komodo Binaries"
LABEL org.opencontainers.image.licenses=GPL-3.0