## All in one, multi stage compile + runtime Docker build for your architecture.

# Build Core
FROM rust:1.89.0-bullseye AS core-builder
RUN cargo install cargo-strip

WORKDIR /builder
COPY Cargo.toml Cargo.lock ./
COPY ./lib ./lib
COPY ./client/core/rs ./client/core/rs
COPY ./client/periphery ./client/periphery
COPY ./bin/core ./bin/core
COPY ./bin/cli ./bin/cli

# Compile app
RUN cargo build -p komodo_core --release && \
  cargo build -p komodo_cli --release && \
  cargo strip

# Build Frontend
FROM node:20.12-alpine AS frontend-builder
WORKDIR /builder
COPY ./frontend ./frontend
COPY ./client/core/ts ./client
RUN cd client && yarn && yarn build && yarn link
RUN cd frontend && yarn link komodo_client && yarn && yarn build

# Final Image
FROM debian:bullseye-slim

COPY ./bin/core/starship.toml /starship.toml
COPY ./bin/core/debian-deps.sh .
RUN sh ./debian-deps.sh && rm ./debian-deps.sh

# Setup an application directory
WORKDIR /app

# Copy
COPY ./config/core.config.toml /config/.default.config.toml
COPY --from=frontend-builder /builder/frontend/dist /app/frontend
COPY --from=core-builder /builder/target/release/core /usr/local/bin/core
COPY --from=core-builder /builder/target/release/km /usr/local/bin/km
COPY --from=denoland/deno:bin /deno /usr/local/bin/deno

# Set $DENO_DIR and preload external Deno deps
ENV DENO_DIR=/action-cache/deno
RUN mkdir /action-cache && \
  cd /action-cache && \
  deno install jsr:@std/yaml jsr:@std/toml

# Hint at the port
EXPOSE 9120

ENV KOMODO_CLI_CONFIG_PATHS="/config"
# This ensures any `komodo.cli.*` takes precedence over the Core `/config/*config.*`
ENV KOMODO_CLI_CONFIG_KEYWORDS="*config.*,*komodo.cli*.*"

CMD [ "core" ]

# Label for Ghcr
LABEL org.opencontainers.image.source=https://github.com/moghtech/komodo
LABEL org.opencontainers.image.description="Komodo Core"
LABEL org.opencontainers.image.licenses=GPL-3.0
