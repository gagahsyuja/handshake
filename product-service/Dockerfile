FROM rust:1.92 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations/ ./migrations/
COPY diesel.toml ./

RUN cargo build --release

# Install diesel CLI in builder stage
RUN cargo install diesel_cli --no-default-features --features postgres

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libpq5 \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/handshake_product /usr/local/bin/handshake_product
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/diesel.toml /app/diesel.toml

WORKDIR /app

EXPOSE 8002

CMD [ "handshake_product" ]
