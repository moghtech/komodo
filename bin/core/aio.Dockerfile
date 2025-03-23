## All in one, multi stage compile + runtime Docker build for your architecture.

# Build Core
FROM rust:1.85.1-bullseye AS core-builder

WORKDIR /builder
COPY Cargo.toml Cargo.lock ./
COPY ./lib ./lib
COPY ./client/core/rs ./client/core/rs
COPY ./client/periphery ./client/periphery
COPY ./bin/core ./bin/core

# Compile app
RUN cargo build -p komodo_core --release

# Build Frontend
FROM node:20.12-alpine AS frontend-builder
WORKDIR /builder
COPY ./frontend ./frontend
COPY ./client/core/ts ./client
RUN cd client && yarn && yarn build && yarn link
RUN cd frontend && yarn link komodo_client && yarn && yarn build

# Final Image
FROM debian:bullseye-slim

# Install Deps
RUN apt update && \
  apt install -y git ca-certificates && \
  rm -rf /var/lib/apt/lists/*

# Setup an application directory
WORKDIR /app

# Copy
COPY ./config/core.config.toml /config/config.toml
COPY --from=frontend-builder /builder/frontend/dist /app/frontend
COPY --from=core-builder /builder/target/release/core /usr/local/bin/core
COPY --from=denoland/deno:bin /deno /usr/local/bin/deno

# Set $DENO_DIR and preload external Deno deps
ENV DENO_DIR=/action-cache/deno
RUN mkdir /action-cache && \
  cd /action-cache && \
  deno install jsr:@std/yaml jsr:@std/toml

# Hint at the port
EXPOSE 9120

# Label for Ghcr
LABEL org.opencontainers.image.source=https://github.com/moghtech/komodo
LABEL org.opencontainers.image.description="Komodo Core"
LABEL org.opencontainers.image.licenses=GPL-3.0

ENTRYPOINT [ "core" ]
