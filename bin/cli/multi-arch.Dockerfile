## Assumes the latest binaries for x86_64 and aarch64 are already built (by binaries.Dockerfile).
## Since theres no heavy build here, QEMU multi-arch builds are fine for this image.

ARG BINARIES_IMAGE=ghcr.io/moghtech/komodo-binaries:2
ARG X86_64_BINARIES=${BINARIES_IMAGE}-x86_64
ARG AARCH64_BINARIES=${BINARIES_IMAGE}-aarch64

# This is required to work with COPY --from
FROM ${X86_64_BINARIES} AS x86_64
FROM ${AARCH64_BINARIES} AS aarch64

FROM debian:trixie-slim

WORKDIR /app

## Copy both binaries initially, but only keep appropriate one for the TARGETPLATFORM.
COPY --from=x86_64 /km /app/arch/linux/amd64
COPY --from=aarch64 /km /app/arch/linux/arm64

ARG TARGETPLATFORM
RUN mv /app/arch/${TARGETPLATFORM} /usr/local/bin/km && rm -r /app/arch

ENV KOMODO_CLI_CONFIG_PATHS="/config"

CMD [ "km" ]

LABEL org.opencontainers.image.source="https://github.com/moghtech/komodo"
LABEL org.opencontainers.image.description="Komodo CLI"
LABEL org.opencontainers.image.licenses="GPL-3.0"