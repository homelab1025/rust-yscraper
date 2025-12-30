FROM debian:bookworm-slim

# Install runtime dependencies (like OpenSSL for reqwest/sqlx)
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY target/release/web_server /app/web_server

EXPOSE 3000

# Set the entrypoint
ENTRYPOINT ["/app/web_server"]
