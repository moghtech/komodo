## Assumes the latest binaries for the required arch are already built (by binaries.Dockerfile).

ARG BINARIES_IMAGE=ghcr.io/moghtech/komodo-binaries:latest

# This is required to work with COPY --from
FROM ${BINARIES_IMAGE} AS binaries

FROM gcr.io/distroless/cc

COPY --from=binaries /util /usr/local/bin/util

LABEL org.opencontainers.image.source=https://github.com/moghtech/komodo
LABEL org.opencontainers.image.description="Komodo Util"
LABEL org.opencontainers.image.licenses=GPL-3.0

CMD [ "util" ]