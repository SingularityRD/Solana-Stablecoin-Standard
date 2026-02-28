# =============================================================================
# Solana Stablecoin Standard - Backend Dockerfile
# =============================================================================
# Multi-stage build for minimal production image
# Build stage: Compiles the Rust application
# Runtime stage: Minimal image with only the binary

# =============================================================================
# Build Stage
# =============================================================================
FROM rust:1.82-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create dummy main.rs to cache dependencies
RUN mkdir -p backend/src && \
    echo "fn main() {}" > backend/src/main.rs

# Copy workspace files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/

# Build dependencies (this layer will be cached)
RUN cargo build --release -p sss-backend --locked && \
    rm -rf backend/src

# Copy actual source code
COPY backend/src backend/src

# Build the actual application
RUN touch backend/src/main.rs && \
    cargo build --release -p sss-backend --locked

# Strip the binary for smaller size
RUN strip --strip-unneeded /app/target/release/sss-backend

# =============================================================================
# Runtime Stage
# =============================================================================
FROM debian:bullseye-slim AS runtime

# Create non-root user for security
RUN groupadd --gid 1000 sss && \
    useradd --uid 1000 --gid sss --shell /bin/bash --create-home sss

WORKDIR /app

# Install runtime dependencies only
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Copy the binary from builder
COPY --from=builder --chown=sss:sss /app/target/release/sss-backend /usr/local/bin/sss-backend

# Create data directory for any file-based operations
RUN mkdir -p /app/data && chown -R sss:sss /app/data

# Switch to non-root user
USER sss

# Expose the application port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -sf http://localhost:3001/health || exit 1

# Set environment variables
ENV RUST_LOG=sss_backend=info \
    SERVER_ADDR=0.0.0.0:3001

# Run the application
ENTRYPOINT ["sss-backend"]