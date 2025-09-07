## All in one, multi stage compile + runtime Docker build for your architecture.

FROM rust:1.89.0-bullseye AS builder
RUN cargo install cargo-strip

WORKDIR /builder
COPY Cargo.toml Cargo.lock ./
COPY ./lib ./lib
COPY ./client/core/rs ./client/core/rs
COPY ./client/periphery ./client/periphery
COPY ./bin/periphery ./bin/periphery

# Compile app
RUN cargo build -p komodo_periphery --release && cargo strip

# Final Image
FROM debian:bullseye-slim

COPY ./bin/periphery/starship.toml /starship.toml
COPY ./bin/periphery/debian-deps.sh .
RUN sh ./debian-deps.sh && rm ./debian-deps.sh

COPY --from=builder /builder/target/release/periphery /usr/local/bin/periphery

EXPOSE 8120

CMD [ "periphery" ]

LABEL org.opencontainers.image.source=https://github.com/moghtech/komodo
LABEL org.opencontainers.image.description="Komodo Periphery"
LABEL org.opencontainers.image.licenses=GPL-3.0
