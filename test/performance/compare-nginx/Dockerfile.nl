# Neurlang Server for Performance Comparison
# Runs the static_server example in a container

FROM debian:bookworm-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy source and build
WORKDIR /build
COPY . .
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install curl for healthcheck
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary and example
COPY --from=builder /build/target/release/nl /app/nl
COPY --from=builder /build/examples/static_server.nl /app/static_server.nl

# Expose port
EXPOSE 8080

# Number of workers (0 = single-threaded, N = N workers)
ENV WORKERS=0

# Run the static server with configurable workers
CMD /app/nl run -i /app/static_server.nl --workers ${WORKERS}
