FROM docker.io/rust:1.63.0-buster as builder
WORKDIR /app
COPY . .
RUN rustup target add x86_64-unknown-linux-musl; \
    apt update && apt install -y musl-tools musl-dev; \
    update-ca-certificates

RUN cargo build --target x86_64-unknown-linux-musl --release; \
    strip /app/target/x86_64-unknown-linux-musl/release/ozes

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ozes /app/ozes
EXPOSE 7656
ENTRYPOINT ["/app/ozes"]
