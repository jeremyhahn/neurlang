# Neurlang Makefile
# Build, test, and manage the AI-Optimized Binary Programming Language

# Python interpreter (use python3 explicitly)
PYTHON := python3

# Version (from VERSION file)
VERSION := $(shell cat VERSION)

# Model configuration
MODEL_VERSION := v$(VERSION)
MODEL_BASE_URL := https://github.com/jeremyhahn/neurlang/releases/download/$(MODEL_VERSION)
MODEL_FILE := best_model.pt
MODEL_PATH := train/models/$(MODEL_FILE)
MODEL_SHA256 := 180ad0d7bff92ad629ad35136b13df48fb01d4057903a7db171095735e33c522

.PHONY: all build build-release test test-unit integration-test coverage coverage-ir \
        bench clean fmt lint check doc install examples help \
        train generate-data export-onnx download-model verify-model

# Default target
all: build

# ============================================================================
# Build Targets
# ============================================================================

# Build the nl binary only (release, recommended for use)
build:
	@echo "Building nl binary..."
	cargo build --release --bin nl
	@echo "Build complete: ./target/release/nl"

# Debug build (for development)
build-debug:
	cargo build --bin nl

# Full build with all features (ONNX + training support)
build-full:
	cargo build --release --features "ort-backend,train"

# Minimal build (tract inference only, smaller binary ~7MB)
build-minimal:
	cargo build --release --features "tract"

# Check compilation without producing binaries
check:
	cargo check --all-targets

# ============================================================================
# Test Targets
# ============================================================================

# Run unit tests (Rust library tests)
test:
	cargo test --lib

# Run integration tests
integration-test: build
	cargo test --test opcode_tests
	cargo test --test interpreter_exec
	cargo test --test integration_runner -- --test-threads=1
	cargo test --test crypto_integration
	cargo test --test examples_integration
	cargo test --test cli_integration

# Run all tests (unit + integration)
test-all: test integration-test

# ============================================================================
# Coverage Targets
# ============================================================================

# Run Rust code coverage (requires cargo-tarpaulin)
coverage:
	@command -v cargo-tarpaulin >/dev/null 2>&1 || { \
		echo "Installing cargo-tarpaulin..."; \
		cargo install cargo-tarpaulin; \
	}
	cargo tarpaulin --out Html --output-dir target/coverage
	@echo "Coverage report: target/coverage/tarpaulin-report.html"

# Run IR-level coverage (nl test on examples and lib)
coverage-ir: build
	@echo "Running IR coverage on examples..."
	./target/release/nl test -p examples/ --coverage
	@echo ""
	@echo "Running IR coverage on lib..."
	./target/release/nl test -p lib/ --coverage 2>/dev/null || echo "(lib/ may not exist yet)"

# ============================================================================
# Code Quality
# ============================================================================

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt -- --check

# Run clippy lints
lint:
	cargo clippy --all-targets -- -D warnings

# Full CI check (format + lint + test)
ci: fmt-check lint test

# ============================================================================
# Benchmarks
# ============================================================================

bench:
	cargo bench

bench-compile:
	cargo bench --bench compile_bench

# ============================================================================
# Documentation
# ============================================================================

# Generate and open Rust documentation
doc:
	cargo doc --no-deps --open

# ============================================================================
# Examples
# ============================================================================

# Test all examples
examples: build
	./target/release/nl test -p examples/

# Run a specific example
run-example: build
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make run-example FILE=examples/algorithm/factorial.nl"; \
		exit 1; \
	fi
	./target/release/nl run -i $(FILE)

# ============================================================================
# Training (Simple Targets)
# ============================================================================

# Download pre-trained model from GitHub releases
download-model:
	@mkdir -p train/models
	@if [ -f $(MODEL_PATH) ]; then \
		echo "Model already exists at $(MODEL_PATH)"; \
		echo "Run 'make verify-model' to check integrity"; \
	else \
		echo "Downloading model $(MODEL_VERSION)..."; \
		curl -L --progress-bar \
			$(MODEL_BASE_URL)/$(MODEL_FILE) \
			-o $(MODEL_PATH); \
		curl -L -s \
			$(MODEL_BASE_URL)/model.config.json \
			-o train/models/model.config.json; \
		echo "Download complete. Verifying..."; \
		$(MAKE) verify-model; \
	fi

# Verify model integrity
verify-model:
	@if [ ! -f $(MODEL_PATH) ]; then \
		echo "Error: Model not found at $(MODEL_PATH)"; \
		echo "Run 'make download-model' first."; \
		exit 1; \
	fi
	@echo "Verifying model checksum..."
	@ACTUAL=$$(sha256sum $(MODEL_PATH) | cut -d' ' -f1); \
	if [ "$$ACTUAL" = "$(MODEL_SHA256)" ]; then \
		echo "✓ Model verified: $(MODEL_PATH)"; \
	else \
		echo "✗ Checksum mismatch!"; \
		echo "  Expected: $(MODEL_SHA256)"; \
		echo "  Got:      $$ACTUAL"; \
		exit 1; \
	fi

# Generate training data (balanced dataset: lib + examples + extensions + HTTP patterns)
generate-data: build
	@echo "Generating training data..."
	$(PYTHON) train/generate_training_data.py train/training_data.jsonl \
		--lib-samples 150 \
		--examples-samples 150 \
		--extension-samples 30000 \
		--http-samples 10000 \
		--synthetic-samples 0 \
		--diverse-samples 0
	@echo "Training data saved to train/training_data.jsonl"
	@wc -l train/training_data.jsonl

# Train the model (requires GPU, see train/PROFILES.md)
train:
	@if [ ! -f train/training_data.jsonl ]; then \
		echo "Training data not found. Run 'make generate-data' first."; \
		exit 1; \
	fi
	@echo "Starting training... (see train/PROFILES.md for GPU options)"
	cd train && PYTHONPATH=. $(PYTHON) parallel/train.py \
		--data training_data.jsonl \
		--output models/model.pt \
		--epochs 100 \
		--batch-size 512 \
		--mixed-precision \
		--device cuda

# Export model to ONNX format
export-onnx:
	@if [ ! -f train/models/best_model.pt ]; then \
		echo "Model not found. Run 'make download-model' or 'make train' first."; \
		exit 1; \
	fi
	cd train && PYTHONPATH=. $(PYTHON) parallel/export_onnx.py \
		--checkpoint models/best_model.pt \
		--output models/model.onnx
	@echo "Model exported to train/models/model.onnx"

# Full training pipeline
train-full: generate-data train export-onnx
	@echo "Training complete!"

# ============================================================================
# Installation
# ============================================================================

# Install nl binary to ~/.cargo/bin
install: build
	cargo install --path . --bin nl

# Uninstall
uninstall:
	cargo uninstall neurlang

# ============================================================================
# Development
# ============================================================================

# Watch for changes and rebuild
watch:
	cargo watch -x 'build --bin nl'

# Run interactive agent
agent: build
	./target/release/nl agent --interactive

# ============================================================================
# Cleanup
# ============================================================================

# Clean build artifacts (preserves training data)
clean:
	cargo clean
	rm -rf target/examples target/coverage
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true

# Clean everything including training data
clean-all: clean
	rm -f train/*.jsonl train/*.pt train/*.onnx
	rm -f model.onnx

# ============================================================================
# Help
# ============================================================================

help:
	@echo "Neurlang Build System"
	@echo ""
	@echo "BUILD:"
	@echo "  make build          Build nl binary (release)"
	@echo "  make build-debug    Build nl binary (debug)"
	@echo "  make build-full     Build with all features (ONNX + training)"
	@echo "  make build-minimal  Build minimal binary (tract only, ~7MB)"
	@echo "  make check          Check compilation without building"
	@echo ""
	@echo "TEST:"
	@echo "  make test           Run unit tests"
	@echo "  make integration-test  Run integration tests"
	@echo "  make test-all       Run all tests"
	@echo ""
	@echo "COVERAGE:"
	@echo "  make coverage       Run Rust code coverage"
	@echo "  make coverage-ir    Run nl test coverage on examples/lib"
	@echo ""
	@echo "QUALITY:"
	@echo "  make fmt            Format code"
	@echo "  make lint           Run clippy lints"
	@echo "  make ci             Full CI check (fmt + lint + test)"
	@echo ""
	@echo "EXAMPLES:"
	@echo "  make examples       Test all examples"
	@echo "  make run-example FILE=path/to/file.nl"
	@echo ""
	@echo "MODEL:"
	@echo "  make download-model Download pre-trained model from GitHub"
	@echo "  make verify-model   Verify model checksum"
	@echo ""
	@echo "TRAINING (requires GPU):"
	@echo "  make generate-data  Generate training data"
	@echo "  make train          Train the model"
	@echo "  make export-onnx    Export to ONNX"
	@echo "  make train-full     Full pipeline (generate + train + export)"
	@echo ""
	@echo "INSTALL:"
	@echo "  make install        Install nl to ~/.cargo/bin"
	@echo "  make uninstall      Uninstall nl"
	@echo ""
	@echo "CLEANUP:"
	@echo "  make clean          Clean build artifacts"
	@echo "  make clean-all      Clean everything including training data"
	@echo ""
	@echo "For detailed training documentation, see docs/training/README.md"
