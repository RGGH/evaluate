# Stage 1: Build
FROM rust:1.89 AS builder

WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (cache layer)
# Create BOTH lib.rs and main.rs since Cargo.toml declares both
RUN mkdir src/ \
    && echo "fn main() {}" > src/main.rs \
    && echo "// placeholder" > src/lib.rs \
    && cargo build --release \
    && rm -rf src/ target/release/deps/evaluate*

# Copy actual source and build
COPY . .
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary and static assets
COPY --from=builder /usr/src/app/target/release/evaluate /app/evaluate
COPY --from=builder /usr/src/app/static /app/static
COPY --from=builder /usr/src/app/migrations /app/migrations

# Set default environment (can be overridden)
ENV DATABASE_URL=sqlite:data/evals.db
ENV RUST_LOG=info

# Create data directory
RUN mkdir -p /app/data

EXPOSE 8080

CMD ["./evaluate"]
