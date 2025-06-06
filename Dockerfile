# Stage 1: Build
FROM rust:1.81-slim AS builder
WORKDIR /app

# Install build dependencies in the BUILD stage
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source and build
COPY src ./src
COPY templates ./templates
COPY assets ./assets
# COPY db ./db
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Install only runtime dependencies (no build tools needed)
RUN apt-get update && \
    apt-get install -y openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy artifacts
COPY --from=builder /app/target/release/hrm_app /app/hrm_app
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/assets /app/assets
# COPY --from=builder /app/db /app/db

# Setup permissions
RUN chmod +x /app/hrm_app

ENV PORT=8000
EXPOSE $PORT
CMD ["/app/hrm_app"]
