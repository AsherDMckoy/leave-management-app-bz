# Dockerfile

FROM rust:1.77 AS builder

WORKDIR /app

# Pre-cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual project
COPY . .

RUN cargo build --release

# ---

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/hrm_app .

COPY .env .env

EXPOSE 8000

CMD ["./hrm_app"]

