# Stage 1: Build the application
FROM rust:1.89-bookworm AS builder
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (cache layer)
RUN mkdir src/ \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src/ target/release/deps/evaluate*

# Copy source code AND required directories
COPY src ./src
COPY migrations ./migrations
COPY static ./static

# Build actual application
RUN cargo build --release

# Stage 2: Create the final, minimal image
FROM debian:bookworm-slim AS final
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/evaluate .
COPY --from=builder /usr/src/app/static ./static

# Create the data directory for SQLite
RUN mkdir -p /usr/local/bin/data

EXPOSE 8080

ENV RUST_BACKTRACE=1
ENV RUST_LOG=info

CMD ["./evaluate"]
