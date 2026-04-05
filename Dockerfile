# Production multi-stage Dockerfile for account-service

FROM rust:1.94-alpine as builder

# Install build dependencies on Alpine
RUN apk add --no-cache \
    build-base \
    openssl-dev \
    mysql-dev \
    pkgconfig \
    musl-dev \
    linux-headers \
    ca-certificates

WORKDIR /usr/src/app

# Cache dependencies by copying manifests and building a placeholder
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() { println!("placeholder"); }' > src/main.rs
RUN cargo build --release || true

# Copy source and build release
COPY . .
RUN cargo build --release --locked

FROM alpine:3.18

# Runtime dependencies (MariaDB connector provides MySQL client libs)
RUN apk add --no-cache \
    ca-certificates \
    mariadb-connector-c \
    openssl

WORKDIR /usr/local/bin

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/account-service ./account-service

# Create non-root user and set ownership
RUN adduser -D app && chown app:app ./account-service
USER app

EXPOSE 3001

CMD ["./account-service"]
