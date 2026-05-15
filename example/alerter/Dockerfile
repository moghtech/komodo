FROM rust:1.95.0-trixie as builder
WORKDIR /builder

COPY . .

RUN cargo build -p alert_logger --release

FROM gcr.io/distroless/debian-cc

COPY --from=builder /builder/target/release/alert_logger /

EXPOSE 7000

CMD ["./alert_logger"]