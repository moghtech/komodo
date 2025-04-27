## Assumes the latest binaries for x86_64 and aarch64 are already built (by binaries.Dockerfile).
## Sets up the necessary runtime container dependencies for Komodo Core.
## Since theres no heavy build here, QEMU multi-arch builds are fine for this image.

ARG BINARIES_IMAGE=ghcr.io/moghtech/komodo-binaries:latest
ARG FRONTEND_IMAGE=ghcr.io/moghtech/komodo-frontend:latest
ARG X86_64_BINARIES=${BINARIES_IMAGE}-x86_64
ARG AARCH64_BINARIES=${BINARIES_IMAGE}-aarch64

# This is required to work with COPY --from
FROM ${X86_64_BINARIES} AS x86_64
FROM ${AARCH64_BINARIES} AS aarch64
FROM ${FRONTEND_IMAGE} AS frontend

# Final Image
FROM debian:bullseye-slim

COPY ./bin/core/starship.toml /config/starship.toml
COPY ./bin/core/debian-deps.sh .
RUN sh ./debian-deps.sh && rm ./debian-deps.sh

WORKDIR /app

# Copy both binaries initially, but only keep appropriate one for the TARGETPLATFORM.
COPY --from=x86_64 /core /app/arch/linux/amd64
COPY --from=aarch64 /core /app/arch/linux/arm64
ARG TARGETPLATFORM
RUN mv /app/arch/${TARGETPLATFORM} /usr/local/bin/core && rm -r /app/arch

# Copy default config / static frontend / deno binary
COPY ./config/core.config.toml /config/config.toml
COPY --from=frontend /frontend /app/frontend
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

CMD [ "core" ]