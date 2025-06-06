# Stage 1: Build
FROM rust:1.70-slim AS builder
WORKDIR /app

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
FROM debian:bullseye-slim
WORKDIR /app

# Install dependencies
RUN apt-get update && \
    apt-get install -y openssl sqlite3 && \
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

