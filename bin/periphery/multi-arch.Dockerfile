## Assumes the latest binaries for x86_64 and aarch64 are already built (by binaries.Dockerfile).
## Sets up the necessary runtime container dependencies for Komodo Periphery.
## Since theres no heavy build here, QEMU multi-arch builds are fine for this image.

ARG BINARIES_IMAGE=ghcr.io/moghtech/komodo-binaries:2
ARG X86_64_BINARIES=${BINARIES_IMAGE}-x86_64
ARG AARCH64_BINARIES=${BINARIES_IMAGE}-aarch64

# This is required to work with COPY --from
FROM ${X86_64_BINARIES} AS x86_64
FROM ${AARCH64_BINARIES} AS aarch64

FROM debian:trixie-slim

COPY ./bin/periphery/starship.toml /starship.toml
COPY ./bin/periphery/debian-deps.sh .
RUN sh ./debian-deps.sh && rm ./debian-deps.sh

WORKDIR /app

## Copy both binaries initially, but only keep appropriate one for the TARGETPLATFORM.
COPY --from=x86_64 /periphery /app/arch/linux/amd64
COPY --from=aarch64 /periphery /app/arch/linux/arm64

ARG TARGETPLATFORM
RUN mv /app/arch/${TARGETPLATFORM} /usr/local/bin/periphery && rm -r /app/arch

COPY ./bin/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 8120

# Can mount config file to /config/*config*.toml and it will be picked up.
ENV PERIPHERY_CONFIG_PATHS="/config"
# Change the default in container to /config/keys to match Core
ENV PERIPHERY_PRIVATE_KEY="file:/config/keys/periphery.key"

ENTRYPOINT [ "entrypoint.sh" ]
CMD [ "periphery" ]

# Label to prevent Komodo from stopping with StopAllContainers
LABEL komodo.skip="true"
# Label for ghcr
LABEL org.opencontainers.image.source="https://github.com/moghtech/komodo"
LABEL org.opencontainers.image.description="Komodo Periphery"
LABEL org.opencontainers.image.licenses="GPL-3.0"
