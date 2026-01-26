FROM rust:1.92 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/handshake_email /usr/local/bin/handshake_email
COPY --from=builder /app/templates /templates
COPY Rocket.toml .

EXPOSE 8004

CMD [ "handshake_email" ]
