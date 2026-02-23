FROM rust:1.82-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/

# Build backend
COPY backend/src backend/src
RUN cargo build --release -p sss-backend

# Runtime image
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sss-backend /usr/local/bin/

EXPOSE 3001

CMD ["sss-backend"]
