## Assumes the latest binaries for the required arch are already built (by binaries.Dockerfile).
## Sets up the necessary runtime container dependencies for Komodo Periphery.

ARG BINARIES_IMAGE=ghcr.io/moghtech/komodo-binaries:latest

# This is required to work with COPY --from
FROM ${BINARIES_IMAGE} AS binaries

FROM debian:bullseye-slim

COPY ./bin/periphery/starship.toml /config/starship.toml
COPY ./bin/periphery/debian-deps.sh .
RUN sh ./debian-deps.sh && rm ./debian-deps.sh

COPY --from=binaries /periphery /usr/local/bin/periphery

EXPOSE 8120

LABEL org.opencontainers.image.source=https://github.com/moghtech/komodo
LABEL org.opencontainers.image.description="Komodo Periphery"
LABEL org.opencontainers.image.licenses=GPL-3.0

CMD [ "periphery" ]