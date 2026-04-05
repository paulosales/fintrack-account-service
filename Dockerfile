# Production multi-stage Dockerfile for account-service

FROM rust:1.94 as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        libmysqlclient-dev \
        build-essential \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Cache dependencies by copying manifests and building a placeholder
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() { println!("placeholder"); }' > src/main.rs
RUN cargo build --release || true

# Copy source and build release
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        libmariadb3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/account-service ./account-service

# Create non-root user
RUN useradd -m app && chown app:app ./account-service
USER app

EXPOSE 3001

CMD ["./account-service"]
