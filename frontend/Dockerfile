FROM node:20.12-alpine AS builder

WORKDIR /builder

COPY ./frontend ./frontend
COPY ./client/core/ts ./client

# Optionally specify a specific Komodo host.
ARG VITE_KOMODO_HOST=""
ENV VITE_KOMODO_HOST=$VITE_KOMODO_HOST

# Build and link the client
RUN cd client && yarn && yarn build && yarn link
RUN cd frontend && yarn link komodo_client && yarn && yarn build

# Copy just the static frontend to scratch image
FROM scratch

COPY --from=builder /builder/frontend/dist /frontend

LABEL org.opencontainers.image.source=https://github.com/moghtech/komodo
LABEL org.opencontainers.image.description="Komodo Frontend"
LABEL org.opencontainers.image.licenses=GPL-3.0