# Multi-stage build for Cap'n Web Server
# This creates a minimal container with just the server binary

# Build stage
FROM rust:1.85-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    binutils \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/capnweb

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY capnweb-core/Cargo.toml ./capnweb-core/
COPY capnweb-transport/Cargo.toml ./capnweb-transport/
COPY capnweb-server/Cargo.toml ./capnweb-server/
COPY capnweb-client/Cargo.toml ./capnweb-client/
COPY capnweb-interop-tests/Cargo.toml ./capnweb-interop-tests/

# Create dummy source files to build dependencies
RUN mkdir -p capnweb-core/src && echo "fn main() {}" > capnweb-core/src/lib.rs
RUN mkdir -p capnweb-transport/src && echo "fn main() {}" > capnweb-transport/src/lib.rs
RUN mkdir -p capnweb-server/src && echo "fn main() {}" > capnweb-server/src/lib.rs && echo "fn main() {}" > capnweb-server/src/main.rs
RUN mkdir -p capnweb-client/src && echo "fn main() {}" > capnweb-client/src/lib.rs
RUN mkdir -p capnweb-interop-tests/src && echo "fn main() {}" > capnweb-interop-tests/src/lib.rs

# Build dependencies
RUN cargo build --release --bin capnweb-server

# Remove dummy source files
RUN rm -rf capnweb-core/src capnweb-transport/src capnweb-server/src capnweb-client/src capnweb-interop-tests/src

# Copy actual source code
COPY capnweb-core/src ./capnweb-core/src
COPY capnweb-transport/src ./capnweb-transport/src
COPY capnweb-server/src ./capnweb-server/src
COPY capnweb-server/examples ./capnweb-server/examples
COPY capnweb-client/src ./capnweb-client/src
COPY capnweb-interop-tests/src ./capnweb-interop-tests/src

# Build the actual binary
RUN touch capnweb-server/src/main.rs && \
    cargo build --release --bin capnweb-server && \
    strip /usr/src/capnweb/target/release/capnweb-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/sh capnweb

# Copy binary from builder
COPY --from=builder /usr/src/capnweb/target/release/capnweb-server /usr/local/bin/capnweb-server

# Set ownership and permissions
RUN chown capnweb:capnweb /usr/local/bin/capnweb-server && \
    chmod 755 /usr/local/bin/capnweb-server

# Switch to non-root user
USER capnweb

# Expose default port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV CAPNWEB_HOST=0.0.0.0
ENV CAPNWEB_PORT=8080

# Run the server
ENTRYPOINT ["/usr/local/bin/capnweb-server"]