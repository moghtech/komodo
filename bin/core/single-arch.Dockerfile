## Assumes the latest binaries for the required arch are already built (by binaries.Dockerfile).
## Sets up the necessary runtime container dependencies for Komodo Core.

ARG BINARIES_IMAGE=ghcr.io/moghtech/komodo-binaries:latest

# This is required to work with COPY --from
FROM ${BINARIES_IMAGE} AS binaries

# Build UI
FROM node:22.12-alpine AS ui-builder
WORKDIR /builder
COPY ./ui ./ui
COPY ./client/core/ts ./client
RUN cd client && yarn && yarn build && yarn link
RUN cd ui && yarn link komodo_client && yarn && yarn build

FROM debian:trixie-slim

COPY ./bin/core/starship.toml /starship.toml
COPY ./bin/core/debian-deps.sh .
RUN sh ./debian-deps.sh && rm ./debian-deps.sh
	
# Copy
COPY ./config/core.config.toml /config/.default.config.toml
COPY --from=ui-builder /builder/ui/dist /app/ui
COPY --from=binaries /core /usr/local/bin/core
COPY --from=binaries /km /usr/local/bin/km
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
