## Assumes the latest binaries for x86_64 and aarch64 are already built (by binaries.Dockerfile).
## Sets up the necessary runtime container dependencies for Komodo Core.
## Since theres no heavy build here, QEMU multi-arch builds are fine for this image.

ARG BINARIES_IMAGE=ghcr.io/moghtech/komodo-binaries:2
ARG UI_IMAGE=ghcr.io/moghtech/komodo-ui:2
ARG X86_64_BINARIES=${BINARIES_IMAGE}-x86_64
ARG AARCH64_BINARIES=${BINARIES_IMAGE}-aarch64

# This is required to work with COPY --from
FROM ${X86_64_BINARIES} AS x86_64
FROM ${AARCH64_BINARIES} AS aarch64
FROM ${UI_IMAGE} AS ui

# Final Image
FROM debian:trixie-slim

COPY ./bin/core/starship.toml /starship.toml
COPY ./bin/core/debian-deps.sh .
RUN sh ./debian-deps.sh && rm ./debian-deps.sh

WORKDIR /app

ARG TARGETPLATFORM

# Copy both binaries initially, but only keep appropriate one for the TARGETPLATFORM.
COPY --from=x86_64 /core /app/core/linux/amd64
COPY --from=aarch64 /core /app/core/linux/arm64
RUN mv /app/core/${TARGETPLATFORM} /usr/local/bin/core && rm -r /app/core

# Same for km
COPY --from=x86_64 /km /app/km/linux/amd64
COPY --from=aarch64 /km /app/km/linux/arm64
RUN mv /app/km/${TARGETPLATFORM} /usr/local/bin/km && rm -r /app/km

# Copy default config / static ui / deno binary
COPY ./config/core.config.toml /config/.default.config.toml
COPY --from=ui /ui /app/ui
COPY --from=denoland/deno:bin /deno /usr/local/bin/deno

# Set $DENO_DIR and preload external Deno deps
ENV DENO_DIR=/action-cache/deno
RUN mkdir /action-cache && \
  cd /action-cache && \
  deno install jsr:@std/yaml jsr:@std/toml

COPY ./bin/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Hint at the port
EXPOSE 9120

ENV KOMODO_CLI_CONFIG_PATHS="/config"
# This ensures any `komodo.cli.*` takes precedence over the Core `/config/*config.*`
ENV KOMODO_CLI_CONFIG_KEYWORDS="*config.*,*komodo.cli*.*"

ENTRYPOINT [ "entrypoint.sh" ]
CMD [ "core" ]

# Label to prevent Komodo from stopping with StopAllContainers
LABEL komodo.skip="true"
# Label for Ghcr
LABEL org.opencontainers.image.source="https://github.com/moghtech/komodo"
LABEL org.opencontainers.image.description="Komodo Core"
LABEL org.opencontainers.image.licenses="GPL-3.0"
