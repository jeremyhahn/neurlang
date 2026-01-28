# Neurlang Performance Benchmarks

## Model Specifications

| Spec | Value |
|------|-------|
| Parameters | 5,754,637 (5.75M) |
| Architecture | Parallel Instruction Prediction (64 slots) |
| Input | 256 tokens |
| Model size | 67 MB (PyTorch), 23 MB (ONNX) |
| Training accuracy | 99.86% |
| Training time | ~10 minutes (RTX 6000 Pro) |

## GPU Benchmarks

### RTX PRO 6000 Blackwell 98GB (Measured 2026-01-29)

| Batch | Latency | Throughput | Per-Sample |
|-------|---------|------------|------------|
| 1 | 0.55 ms | 1,835/sec | 0.55 ms |
| 8 | 0.94 ms | 8,538/sec | 0.12 ms |
| 32 | 1.93 ms | 16,605/sec | 0.06 ms |

**Iterations achievable (batch=1):** 9,176 in 5 sec, 55,061 in 30 sec

## CPU Benchmarks

| CPU | Runtime | Latency | Throughput |
|-----|---------|---------|------------|
| Xeon (server) | ONNX | 18 ms | 55/sec |
| Xeon 8260 (dev) | ONNX | 93 ms | 10.7/sec |
| Xeon (dev) | PyTorch | 300 ms | 1.8/sec |

**Note:** ONNX Runtime is 25x faster than PyTorch on CPU.

## Summary

| Hardware | Latency | Throughput | Cost |
|----------|---------|------------|------|
| RTX PRO 6000 Blackwell | 0.55 ms | 1,835/sec | $0.48/hr (Verda) |
| Server CPU (ONNX) | 18 ms | 55/sec | - |
| Desktop CPU (PyTorch) | 300 ms | 1.8/sec | - |

GPU required for production workloads. CPU viable for development/testing.
