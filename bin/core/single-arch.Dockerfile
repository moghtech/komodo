## Assumes the latest binaries for the required arch are already built (by binaries.Dockerfile).
## Sets up the necessary runtime container dependencies for Komodo Core.

ARG BINARIES_IMAGE=ghcr.io/mbecker20/komodo-binaries:latest

# This is required to work with COPY --from
FROM ${BINARIES_IMAGE} AS binaries

# Build Frontend
FROM node:20.12-alpine AS frontend-builder
WORKDIR /builder
COPY ./frontend ./frontend
COPY ./client/core/ts ./client
RUN cd client && yarn && yarn build && yarn link
RUN cd frontend && yarn link komodo_client && yarn && yarn build

FROM debian:bullseye-slim

# Install Deps
RUN apt update && \
	apt install -y git ca-certificates && \
	rm -rf /var/lib/apt/lists/*
	
# Copy
COPY ./config/core.config.toml /config/config.toml
COPY --from=frontend-builder /builder/frontend/dist /app/frontend
COPY --from=binaries /core /usr/local/bin/core
COPY --from=denoland/deno:bin /deno /usr/local/bin/deno

# Set $DENO_DIR and preload external Deno deps
ENV DENO_DIR=/action-cache/deno
RUN mkdir /action-cache && \
	cd /action-cache && \
	deno install jsr:@std/yaml jsr:@std/toml

# Hint at the port
EXPOSE 9120

# Label for Ghcr
LABEL org.opencontainers.image.source=https://github.com/mbecker20/komodo
LABEL org.opencontainers.image.description="Komodo Core"
LABEL org.opencontainers.image.licenses=GPL-3.0

CMD [ "core" ]