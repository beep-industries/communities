# Build stage - Alpine with latest Rust
FROM alpine:3.21 AS builder

WORKDIR /app

# Install build dependencies and Rust via rustup
RUN apk add --no-cache \
    curl \
    gcc \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    protobuf-dev \
    protoc

# Install Rust (latest stable)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml api/Cargo.toml
COPY core/Cargo.toml core/Cargo.toml
COPY outbox_dispatch/Cargo.toml outbox_dispatch/Cargo.toml

# Create dummy source files to cache dependencies
RUN mkdir -p api/src core/src outbox_dispatch/src && \
    echo "fn main() {}" > api/src/main.rs && \
    echo "pub fn dummy() {}" > api/src/lib.rs && \
    echo "pub fn dummy() {}" > core/src/lib.rs && \
    echo "fn main() {}" > outbox_dispatch/src/main.rs

# Build dependencies only (cached layer)
ENV OPENSSL_STATIC=1
RUN cargo build --release --bin api && \
    rm -rf api/src core/src outbox_dispatch/src

# Copy actual source code
COPY api/src api/src
COPY core/src core/src
COPY outbox_dispatch/src outbox_dispatch/src

# Copy sqlx offline data and migrations for compile-time query checking
COPY .sqlx .sqlx
COPY core/migrations core/migrations

# Copy config files needed at runtime
COPY config config

# Touch to invalidate cached main.rs and rebuild
RUN touch api/src/main.rs core/src/lib.rs

# Build the actual binary with sqlx offline mode
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin api

# Runtime stage - minimal scratch image
FROM scratch

WORKDIR /app

# Copy CA certificates for HTTPS (Keycloak calls)
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the statically linked binary
COPY --from=builder /app/target/release/api /app/api

# Copy config files
COPY --from=builder /app/config /app/config

# Expose ports (API and Health)
EXPOSE 3003 9090

# Set default environment variables
ENV RUST_LOG=info
ENV API_PORT=3003
ENV HEALTH_PORT=9090
ENV ROUTING_CONFIG_PATH=/app/config/routing.yaml

# Run the API server
ENTRYPOINT ["/app/api"]
