#!/bin/bash
# Publish a new Neurlang release with model artifacts
#
# Usage:
#   ./scripts/publish-release.sh [version]
#
# If version is not provided, reads from VERSION file.
#
# Prerequisites:
#   - gh CLI installed and authenticated
#   - Model trained and verified (train/models/best_model.pt)
#   - All tests passing

set -e

# Get version from argument or VERSION file
if [ -n "$1" ]; then
    VERSION="$1"
else
    VERSION="v$(cat VERSION)"
fi

# Validate version format
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format vX.Y.Z (e.g., v0.1.0)"
    echo "Got: $VERSION"
    exit 1
fi

echo "=== Publishing Neurlang $VERSION ==="
echo ""

# Check prerequisites
echo "Checking prerequisites..."

if ! command -v gh &> /dev/null; then
    echo "Error: gh CLI not installed. Install from https://cli.github.com/"
    exit 1
fi

if [ ! -f "train/models/best_model.pt" ]; then
    echo "Error: Model not found at train/models/best_model.pt"
    echo "Run 'make train' first or download existing model."
    exit 1
fi

# Verify model
echo ""
echo "Verifying model..."
make verify-model

# Export to ONNX if not exists
if [ ! -f "train/models/model.onnx" ]; then
    echo ""
    echo "Exporting model to ONNX..."
    make export-onnx
fi

# Run tests
echo ""
echo "Running tests..."
make test
timeout 60s ./target/release/nl test -p examples || {
    echo "Warning: Example tests failed or timed out"
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
}

# Calculate checksums
echo ""
echo "Calculating checksums..."
cd train/models
sha256sum best_model.pt model.onnx model.config.json > checksums.txt
cat checksums.txt
cd ../..

MODEL_SHA=$(sha256sum train/models/best_model.pt | cut -d' ' -f1)
ONNX_SHA=$(sha256sum train/models/model.onnx | cut -d' ' -f1)

echo ""
echo "=== Checksums ==="
echo "  best_model.pt: $MODEL_SHA"
echo "  model.onnx:    $ONNX_SHA"

# Create release notes
RELEASE_NOTES=$(cat <<EOF
## Neurlang $VERSION

### Model
- **Parameters:** 5.7M
- **Accuracy:** 99.86% opcode prediction
- **Formats:** PyTorch (.pt) and ONNX (.onnx)

### Installation

\`\`\`bash
# Download binary (Linux x86_64)
curl -LO https://github.com/jeremyhahn/neurlang/releases/download/$VERSION/nl-linux-x86_64.tar.gz
tar -xzf nl-linux-x86_64.tar.gz
sudo mv nl /usr/local/bin/

# Download model
make download-model
\`\`\`

### Docker

\`\`\`bash
docker pull ghcr.io/jeremyhahn/neurlang:$VERSION
docker run --rm -it ghcr.io/jeremyhahn/neurlang:$VERSION nl --help
\`\`\`

### Checksums (SHA256)
\`\`\`
$(cat train/models/checksums.txt)
\`\`\`
EOF
)

echo ""
echo "=== Release Notes ==="
echo "$RELEASE_NOTES"
echo ""

# Confirm
read -p "Create release $VERSION? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Create git tag
echo ""
echo "Creating git tag $VERSION..."
git tag -a "$VERSION" -m "Release $VERSION"

echo ""
echo "=== Next Steps ==="
echo ""
echo "1. Push the tag to trigger the release workflow:"
echo "   git push origin $VERSION"
echo ""
echo "2. The GitHub Actions workflow will:"
echo "   - Build binaries for all platforms"
echo "   - Build and push Docker image"
echo "   - Create GitHub release with artifacts"
echo ""
echo "3. After release, update Makefile with new checksum:"
echo "   MODEL_SHA256 := $MODEL_SHA"
echo ""
