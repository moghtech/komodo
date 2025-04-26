## Assumes the latest binaries for x86_64 and aarch64 are already built (by binaries.Dockerfile).
## Sets up the necessary runtime container dependencies for Komodo Periphery.
## Since theres no heavy build here, QEMU multi-arch builds are fine for this image.

ARG BINARIES_IMAGE=ghcr.io/moghtech/komodo-binaries:latest
ARG X86_64_BINARIES=${BINARIES_IMAGE}-x86_64
ARG AARCH64_BINARIES=${BINARIES_IMAGE}-aarch64

# This is required to work with COPY --from
FROM ${X86_64_BINARIES} AS x86_64
FROM ${AARCH64_BINARIES} AS aarch64

FROM debian:bullseye-slim

COPY ./bin/periphery/starship.toml /config/starship.toml
COPY ./bin/periphery/debian-deps.sh .
RUN sh ./debian-deps.sh && rm ./debian-deps.sh

WORKDIR /app

## Copy both binaries initially, but only keep appropriate one for the TARGETPLATFORM.
COPY --from=x86_64 /periphery /app/arch/linux/amd64
COPY --from=aarch64 /periphery /app/arch/linux/arm64

ARG TARGETPLATFORM
RUN mv /app/arch/${TARGETPLATFORM} /usr/local/bin/periphery && rm -r /app/arch

EXPOSE 8120

LABEL org.opencontainers.image.source=https://github.com/moghtech/komodo
LABEL org.opencontainers.image.description="Komodo Periphery"
LABEL org.opencontainers.image.licenses=GPL-3.0

CMD [ "periphery" ]