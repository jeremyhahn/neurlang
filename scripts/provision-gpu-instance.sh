#!/bin/bash
# Neurlang GPU Instance Provisioning Script
# ==========================================
# Complete setup for a fresh GPU instance.
#
# Usage (run locally):
#   ./scripts/provision-gpu-instance.sh root@<host>
#
# Requirements on remote:
#   - Ubuntu 22.04 or 24.04
#   - NVIDIA GPU with drivers pre-installed
#   - 50GB+ disk space (100GB recommended)
#   - SSH access with key authentication

set -e

HOST="${1:?Usage: $0 root@hostname}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"

echo "=============================================="
echo "  Neurlang GPU Instance Provisioning"
echo "=============================================="
echo "Host: $HOST"
echo "Repo: $REPO_DIR"
echo ""

# Add host key
echo "=== Adding SSH host key ==="
ssh-keyscan -H "${HOST#*@}" >> ~/.ssh/known_hosts 2>/dev/null || true

# Check disk space
echo "=== Checking disk space ==="
DISK_AVAIL=$(ssh -o StrictHostKeyChecking=no "$HOST" "df / --output=avail | tail -1")
DISK_GB=$((DISK_AVAIL / 1024 / 1024))
echo "Available: ${DISK_GB}GB"

if [ "$DISK_GB" -lt 20 ]; then
    echo "ERROR: Need at least 20GB free (have ${DISK_GB}GB)"
    echo "Recommended: 50-100GB"
    exit 1
fi

# Upload setup script and run it
echo ""
echo "=== Running remote setup ==="
ssh "$HOST" 'bash -s' << 'REMOTE_SETUP'
set -e

echo ">>> Installing system packages..."
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq python3-pip python3.12-venv git curl wget htop

echo ">>> Installing PyTorch and dependencies..."
pip install --break-system-packages --no-cache-dir torch numpy tqdm

echo ">>> Verifying GPU..."
python3 << 'PYCHECK'
import torch
print(f"PyTorch: {torch.__version__}")
print(f"CUDA available: {torch.cuda.is_available()}")
if torch.cuda.is_available():
    print(f"GPU: {torch.cuda.get_device_name(0)}")
    print(f"Memory: {torch.cuda.get_device_properties(0).total_memory / 1e9:.1f}GB")
else:
    print("WARNING: CUDA not available!")
    exit(1)
PYCHECK

echo ">>> Creating directories..."
mkdir -p ~/neurlang/train/{parallel,models}

echo ">>> Setup complete!"
REMOTE_SETUP

# Upload training scripts
echo ""
echo "=== Uploading training scripts ==="
scp "$REPO_DIR/train/parallel/"*.py "$HOST:~/neurlang/train/parallel/"

# Generate training data on remote
echo ""
echo "=== Generating training data (500K samples) ==="
ssh "$HOST" "cd ~/neurlang/train && python3 parallel/generate_comprehensive_data.py training_data.jsonl"

# Start training
echo ""
echo "=== Starting training ==="
ssh "$HOST" 'cd ~/neurlang/train && PYTHONPATH=. nohup python3 -u parallel/train.py \
    --data training_data.jsonl \
    --output models/model.pt \
    --epochs 50 \
    --batch-size 256 \
    --lr 1e-3 \
    --device cuda \
    > training.log 2>&1 &'

echo ""
echo "=============================================="
echo "  Provisioning Complete!"
echo "=============================================="
echo ""
echo "Training started in background."
echo ""
echo "Useful commands:"
echo "  Monitor training:  ssh $HOST 'tail -f ~/neurlang/train/training.log'"
echo "  Check GPU usage:   ssh $HOST 'nvidia-smi'"
echo "  Check progress:    ssh $HOST 'grep -E \"Epoch|loss\" ~/neurlang/train/training.log | tail -20'"
echo "  Download model:    scp $HOST:~/neurlang/train/models/parallel.onnx train/models/"
echo ""
