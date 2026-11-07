#!/usr/bin/env bash
# ==============================================================================
# LDrago CLI — RunPod A100/H100 Automated Cloud GPU Provisioning Script
# ==============================================================================
set -euo pipefail

echo "=========================================================="
echo "  LDrago CLI: Bootstrapping Cloud A100 Pretraining Cluster "
echo "=========================================================="

# 1. System updates & package installation
apt-get update -y && apt-get install -y --no-install-recommends \
    build-essential \
    git \
    curl \
    nvtop \
    htop \
    ca-certificates

# 2. Install modern uv package manager
if ! command -v uv &> /dev/null; then
    echo ">> Installing uv package manager..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# 3. Check NVIDIA driver and CUDA availability
echo ">> Inspecting GPU hardware:"
nvidia-smi

# 4. Initialize Python virtual environment with PyTorch & CUDA support
cd /workspace/LDrago_CLI/training
echo ">> Setting up Python 3.12 environment..."
uv venv .venv --python 3.12
source .venv/bin/activate
uv pip install -e ".[dev]"

echo ">> Verifying PyTorch CUDA compatibility:"
python3 -c "import torch; print(f'PyTorch {torch.__version__} | CUDA Available: {torch.cuda.is_available()} | Device: {torch.cuda.get_device_name(0) if torch.cuda.is_available() else \"None\"}')"

echo "=========================================================="
echo "  Setup Complete! Ready for distributed pretraining.     "
echo "=========================================================="
