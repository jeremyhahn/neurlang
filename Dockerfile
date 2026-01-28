# Neurlang Docker Image
# Contains the nl binary and pre-trained model
#
# Build:
#   docker build -t neurlang .
#
# Run:
#   docker run --rm -it neurlang nl --help
#   docker run --rm -it neurlang nl prompt "compute factorial of 5"
#   docker run --rm -v $(pwd):/work neurlang nl run -i /work/program.nl

ARG VERSION=latest

# Build stage (if building from source)
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release --bin nl

# Runtime stage
FROM debian:bookworm-slim

ARG VERSION
LABEL org.opencontainers.image.title="Neurlang"
LABEL org.opencontainers.image.description="AI-Optimized Binary Programming Language"
LABEL org.opencontainers.image.version="${VERSION}"
LABEL org.opencontainers.image.source="https://github.com/jeremyhahn/neurlang"

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies for inference
RUN pip3 install --break-system-packages --no-cache-dir \
    torch --index-url https://download.pytorch.org/whl/cpu

# Create non-root user
RUN useradd -m -s /bin/bash neurlang

# Copy binary from builder (or use pre-built if available)
COPY --from=builder /build/target/release/nl /usr/local/bin/nl

# Alternative: Copy pre-built binary (uncomment for CI builds)
# COPY nl /usr/local/bin/nl

RUN chmod +x /usr/local/bin/nl

# Create directories
RUN mkdir -p /opt/neurlang/models /opt/neurlang/lib /opt/neurlang/examples

# Copy model files (if available)
COPY train/models/*.pt /opt/neurlang/models/ 2>/dev/null || true
COPY train/models/*.onnx /opt/neurlang/models/ 2>/dev/null || true
COPY train/models/*.json /opt/neurlang/models/ 2>/dev/null || true

# Copy stdlib and examples
COPY lib/ /opt/neurlang/lib/
COPY examples/ /opt/neurlang/examples/

# Copy inference scripts
COPY train/parallel/model.py /opt/neurlang/
COPY train/parallel/test_inference.py /opt/neurlang/

# Set environment
ENV NEURLANG_MODEL_PATH=/opt/neurlang/models/best_model.pt
ENV NEURLANG_LIB_PATH=/opt/neurlang/lib
ENV PATH="/usr/local/bin:${PATH}"

# Set working directory
WORKDIR /work

# Switch to non-root user
USER neurlang

# Default command
CMD ["nl", "--help"]

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s \
    CMD nl --version || exit 1
