#!/usr/bin/env bash
# ==============================================================================
# LDrago CLI — Multi-GPU Distributed Pretraining Execution Script
# ==============================================================================
set -euo pipefail

BRAIN=${1:-"coding"}
SCALE=${2:-"base_125m"}
GPUS=${3:-$(nvidia-smi -L | wc -l)}

echo "=========================================================="
echo "  Launching LDrago Distributed Pretraining                "
echo "  Target Brain : ${BRAIN}                                 "
echo "  Model Scale  : ${SCALE}                                 "
echo "  Active GPUs  : ${GPUS}                                  "
echo "=========================================================="

cd /workspace/LDrago_CLI/training
source .venv/bin/activate

# Execute multi-GPU torchrun DDP pretraining harness
torchrun \
    --nproc_per_node="${GPUS}" \
    --master_port=29500 \
    -m src.train.train_distributed \
    --brain "${BRAIN}" \
    --scale "${SCALE}" \
    --fp16 \
    --steps 50000 \
    --batch-size 8

echo "Pretraining job completed successfully."
