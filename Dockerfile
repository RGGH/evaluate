# Stage 1: Build the application
# We use a specific Rust version for reproducibility.
# The 'bookworm' variant uses Debian 12 (current stable).
FROM rust:1.89-bookworm AS builder
# Create a new empty shell project to cache dependencies
WORKDIR /usr/src/app
RUN USER=root cargo init --bin .
# Copy over your manifests
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs
# Copy your actual source code
COPY src ./src
# Build for release.
RUN cargo build --release
# Stage 2: Create the final, minimal image
# Using a slim Debian image for a small footprint.
FROM debian:bookworm-slim AS final
# Install runtime dependencies.
# ca-certificates is needed for making HTTPS requests.
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/local/bin
# Copy the compiled binary from the builder stage.
COPY --from=builder /usr/src/app/target/release/evaluate .
# Copy static assets and configuration needed at runtime.
COPY static ./static
COPY src/config.toml .
# Expose the port the server listens on.
EXPOSE 8080
# Set the entrypoint to run the application.
# The DATABASE_URL should be passed as an environment variable
# when you run the container.
CMD ["./evaluate"]
