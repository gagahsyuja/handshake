FROM rust:1.92 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/handshake_auth /usr/local/bin/handshake_auth

EXPOSE 8001

CMD [ "handshake_auth" ]
