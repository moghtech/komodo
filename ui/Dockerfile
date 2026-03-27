FROM node:22.12-alpine AS builder

WORKDIR /builder

COPY ./ui ./ui
COPY ./client/core/ts ./client

# Optionally specify a specific Komodo host.
ARG VITE_KOMODO_HOST=""
ENV VITE_KOMODO_HOST=$VITE_KOMODO_HOST

# Build and link the client
RUN cd client && yarn && yarn build && yarn link
RUN cd ui && yarn link komodo_client && yarn && yarn build

# Copy just the static ui to scratch image
FROM scratch

COPY --from=builder /builder/ui/dist /ui

LABEL org.opencontainers.image.source="https://github.com/moghtech/komodo"
LABEL org.opencontainers.image.description="Komodo UI"
LABEL org.opencontainers.image.licenses="GPL-3.0"