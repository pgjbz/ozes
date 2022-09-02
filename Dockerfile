FROM docker.io/rust:1.63.0-slim as builder
LABEL stage="build"
WORKDIR /app
COPY . .

RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build --target x86_64-unknown-linux-musl --release && \
    strip /app/target/x86_64-unknown-linux-musl/release/ozes

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ozes /app/ozes
EXPOSE 7656
ENTRYPOINT ["/app/ozes"]
