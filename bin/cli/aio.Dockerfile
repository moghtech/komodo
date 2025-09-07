FROM rust:1.89.0-bullseye AS builder
RUN cargo install cargo-strip

WORKDIR /builder
COPY Cargo.toml Cargo.lock ./
COPY ./lib ./lib
COPY ./client/core/rs ./client/core/rs
COPY ./client/periphery ./client/periphery
COPY ./bin/cli ./bin/cli

# Compile bin
RUN cargo build -p komodo_cli --release && cargo strip

# Copy binaries to distroless base
FROM gcr.io/distroless/cc

COPY --from=builder /builder/target/release/km /usr/local/bin/km

ENV KOMODO_CLI_CONFIG_PATHS="/config"

CMD [ "km" ]

LABEL org.opencontainers.image.source=https://github.com/moghtech/komodo
LABEL org.opencontainers.image.description="Komodo CLI"
LABEL org.opencontainers.image.licenses=GPL-3.0