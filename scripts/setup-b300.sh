#!/bin/bash
# Neurlang B300/Blackwell GPU Training Setup Script
# Installs PyTorch nightly with CUDA 12.8 for Blackwell architecture
#
# Usage:
#   scp scripts/setup-b300.sh root@<B300-IP>:/root/
#   ssh root@<B300-IP> "bash /root/setup-b300.sh"
#
# Or via nl CLI:
#   nl train --profile b300 --remote root@<B300-IP> --data training_data.jsonl

set -e

echo "=== Neurlang B300 Training Environment Setup ==="
echo ""

# Check for GPU
echo "[1/5] Detecting GPU..."
if ! command -v nvidia-smi &> /dev/null; then
    echo "ERROR: nvidia-smi not found. Is NVIDIA driver installed?"
    exit 1
fi

GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
GPU_MEM=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader | head -1)
echo "  Found: $GPU_NAME ($GPU_MEM)"

# Install system dependencies
echo ""
echo "[2/5] Installing system dependencies..."
apt-get update -qq
apt-get install -y python3-venv python3-pip 2>/dev/null || {
    PYVER=$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')
    apt-get install -y python3.${PYVER#*.}-venv python3-pip
}

# Create virtual environment
echo ""
echo "[3/5] Creating Python virtual environment..."
rm -rf /root/venv  # Clean start
python3 -m venv /root/venv

# Install PyTorch nightly with CUDA 12.8 for Blackwell
echo ""
echo "[4/5] Installing PyTorch nightly (cu128 for Blackwell)..."
/root/venv/bin/pip install --no-cache-dir --upgrade pip -q
/root/venv/bin/pip install --no-cache-dir --pre torch \
    --index-url https://download.pytorch.org/whl/nightly/cu128
/root/venv/bin/pip install --no-cache-dir numpy tqdm -q

# Verify installation
echo ""
echo "[5/5] Verifying installation..."
/root/venv/bin/python3 << 'VERIFY'
import torch
print(f'PyTorch version: {torch.__version__}')
print(f'CUDA available: {torch.cuda.is_available()}')
if torch.cuda.is_available():
    print(f'GPU: {torch.cuda.get_device_name(0)}')
    print(f'VRAM: {torch.cuda.get_device_properties(0).total_memory / 1e9:.1f} GB')
    cap = torch.cuda.get_device_capability(0)
    print(f'Compute capability: {cap[0]}.{cap[1]}')
    # Quick CUDA test
    x = torch.randn(100, 100, device='cuda')
    y = torch.matmul(x, x)
    print('CUDA test: PASSED')
VERIFY

# Create helper script
cat > /root/run-training.sh << 'HELPER'
#!/bin/bash
# Run Neurlang training on B300
set -e
source /root/venv/bin/activate
cd ~/neurlang/train

# Generate dataset if not exists
if [ ! -f "training_data.jsonl" ]; then
    echo "Generating balanced training data (500K samples)..."
    PYTHONPATH=. python3 parallel/generate_balanced_data.py training_data.jsonl
fi

# Run training
echo "Starting training..."
mkdir -p models
PYTHONPATH=. python3 parallel/train.py \
    --data training_data.jsonl \
    --output models/model.pt \
    --epochs ${EPOCHS:-100} \
    --batch-size ${BATCH_SIZE:-256} \
    --lr ${LR:-1e-3} \
    --device cuda \
    --mixed-precision \
    --num-workers 8 \
    2>&1 | tee training.log

echo ""
echo "Training complete. Model saved to models/model.pt"
HELPER
chmod +x /root/run-training.sh

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Next steps:"
echo ""
echo "  1. FROM LOCAL - sync training files:"
echo "     rsync -avz --progress train/parallel/ root@<B300-IP>:~/neurlang/train/parallel/"
echo ""
echo "  2. FROM LOCAL - sync dataset (if generated locally):"
echo "     rsync -avz --progress train/training_data_500k.jsonl root@<B300-IP>:~/neurlang/train/training_data.jsonl"
echo ""
echo "  3. ON B300 - run training:"
echo "     bash /root/run-training.sh"
echo ""
echo "  4. FROM LOCAL - download trained model:"
echo "     rsync -avz root@<B300-IP>:~/neurlang/train/models/ train/models/"
echo ""
echo "Or use the CLI:"
echo "     nl train --profile b300 --remote root@<B300-IP> --data training_data_500k.jsonl"
echo ""
