# Build Core
FROM rust:1.81.0-alpine AS core-builder
WORKDIR /builder
COPY . .
RUN apk update && apk --no-cache add musl-dev openssl-dev openssl-libs-static
RUN cargo build -p komodo_core --release

# Build Frontend
FROM node:20.12-alpine AS frontend-builder
WORKDIR /builder
COPY ./frontend ./frontend
COPY ./client/core/ts ./client
RUN cd client && yarn && yarn build && yarn link
RUN cd frontend && yarn link @komodo/client && yarn && yarn build

# Final Image
FROM alpine:3.20

# Install Deps
RUN apk update && apk add openssl ca-certificates git git-lfs

# Setup an application directory
WORKDIR /app

# Copy
COPY ./config/core.config.toml /config/config.toml
COPY --from=core-builder /builder/target/release/core /app
COPY --from=frontend-builder /builder/frontend/dist /app/frontend

# Hint at the port
EXPOSE 9120 

# Label for Ghcr
LABEL org.opencontainers.image.source=https://github.com/mbecker20/komodo
LABEL org.opencontainers.image.description="Komodo Core"
LABEL org.opencontainers.image.licenses=GPL-3.0

# Using ENTRYPOINT allows cli args to be passed, eg using "command" in docker compose.
ENTRYPOINT [ "/app/core" ]