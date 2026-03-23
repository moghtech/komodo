FROM rust:1.94.0-bullseye as builder
WORKDIR /builder

COPY . .

RUN cargo build -p update_logger --release

FROM gcr.io/distroless/debian-cc

COPY --from=builder /builder/target/release/update_logger /

EXPOSE 7000

CMD ["./update_logger"]