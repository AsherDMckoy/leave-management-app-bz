# Stage 1: Build
FROM rust:1.81 AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && \
    echo "fn main() {println!(\"dummy main\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy real source
COPY src ./src
COPY templates ./templates
COPY assets ./assets
RUN touch src/main.rs
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y openssl ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Copy artifacts
COPY --from=builder /app/target/release/hrm_app /app/hrm_app
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/assets /app/assets

# Create non-root user
RUN useradd -m appuser && \
    chown -R appuser:appuser /app
USER appuser

ENV PORT=8000
EXPOSE $PORT

# Add healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT}/ || exit 1

CMD ["/app/hrm_app"]
