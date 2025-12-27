# Use a multi-stage build to keep the final image small
FROM rust:1.92-slim-bookworm AS builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl

WORKDIR /app

# Copy the entire workspace
COPY . .

# Build the web_server in release mode
RUN cargo build --release -p web_server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies (like OpenSSL for reqwest/sqlx)
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/web_server /usr/local/bin/web_server

# The application looks for config.properties by default, 
# but we can also use environment variables as per src/config.rs logic.
# We'll expose the default port.
EXPOSE 3000

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/web_server"]
