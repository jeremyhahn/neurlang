# Neurlang Development Container
# Includes Rust, Python, and all required tooling

FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

# Install additional system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    python3 \
    python3-pip \
    python3-venv \
    llvm \
    clang \
    gdb \
    valgrind \
    linux-perf \
    && rm -rf /var/lib/apt/lists/*

# Install Rust components
RUN rustup component add \
    rustfmt \
    clippy \
    llvm-tools-preview

# Install cargo tools
RUN cargo install \
    cargo-watch \
    cargo-expand \
    cargo-criterion \
    cargo-tarpaulin \
    cargo-audit \
    cargo-deny

# Set up Python environment
RUN pip3 install --upgrade pip setuptools wheel

# Create workspace directory
WORKDIR /workspace

# Copy requirements and install Python dependencies
COPY train/requirements.txt /tmp/requirements.txt
RUN pip3 install -r /tmp/requirements.txt

# Set environment variables
ENV RUST_BACKTRACE=1
ENV CARGO_INCREMENTAL=1
ENV PYTHONUNBUFFERED=1

# Default command
CMD ["bash"]
