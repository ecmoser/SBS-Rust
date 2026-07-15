FROM rust:1.95.0-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin \
    && echo "fn main() {}" > src/bin/cli.rs \
    && echo "" > src/lib.rs
RUN cargo build --release
RUN rm -rf src

COPY . .
RUN touch src/lib.rs src/bin/cli.rs && cargo build --release

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/cli /app/cli
ENTRYPOINT ["/app/cli"]


