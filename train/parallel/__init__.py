"""
Parallel Instruction Prediction Model

This module contains the parallel slot prediction model architecture
and training code for predicting up to 64 instructions in a single forward pass.
"""

from .model import (
    ParallelInstructionModel,
    LightParallelModel,
    ParallelInstructionLoss,
    NUM_SLOTS,
    NUM_OPCODES,
    NUM_MODES,
    NUM_REGISTERS,
    IMM_BINS,
)

__all__ = [
    'ParallelInstructionModel',
    'LightParallelModel',
    'ParallelInstructionLoss',
    'NUM_SLOTS',
    'NUM_OPCODES',
    'NUM_MODES',
    'NUM_REGISTERS',
    'IMM_BINS',
]
