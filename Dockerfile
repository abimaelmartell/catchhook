# Build stage
FROM rust:1.83-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY public ./public
COPY tests ./tests

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/catchhook /usr/local/bin/catchhook

# Copy static files
COPY --from=builder /app/public /app/public

# Create data directory
RUN mkdir -p /app/catchhook-data

# Set environment variables
ENV CATCHHOOK_PORT=43999
ENV CATCHHOOK_DATA=/app/catchhook-data
ENV CATCHHOOK_MAX_REQS=10000

# Expose port
EXPOSE 43999

# Run the application
CMD ["catchhook"]
