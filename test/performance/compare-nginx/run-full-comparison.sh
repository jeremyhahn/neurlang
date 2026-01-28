#!/bin/bash
# Full worker count comparison: Neurlang (0, 1, 2, 4 workers) vs nginx
# All services run in Docker on the same network for fair comparison

set -e
cd "$(dirname "$0")"

REQUESTS=${REQUESTS:-10000}
CONCURRENCY=${CONCURRENCY:-50}

echo ""
echo "=============================================="
echo "Neurlang Worker Count Comparison vs nginx"
echo "Requests: $REQUESTS, Concurrency: $CONCURRENCY"
echo "=============================================="
echo ""

# Clean up any existing containers
docker compose down -v 2>/dev/null || true
mkdir -p results

# Start nginx
echo "Starting nginx..."
docker compose up -d nginx
sleep 3

# Wait for nginx to be healthy
for i in {1..30}; do
    if docker compose exec -T nginx wget -qO- http://127.0.0.1:80/ > /dev/null 2>&1; then
        echo "  nginx: ready"
        break
    fi
    sleep 1
done

# Function to run benchmark from within Docker network
run_docker_bench() {
    local name=$1
    local url=$2

    # Run ab from the benchmark container
    docker compose run --rm -T benchmark sh -c "
        # Warmup
        ab -n 1000 -c $CONCURRENCY -q '$url' > /dev/null 2>&1 || true
        sleep 1
        # Actual benchmark
        ab -n $REQUESTS -c $CONCURRENCY '$url' 2>&1 | grep 'Requests per second' | awk '{print \$4}'
    "
}

# Benchmark nginx
echo ""
echo "Benchmarking nginx (multi-worker)..."
NGINX_RPS=$(run_docker_bench "nginx" "http://nginx:80/")
echo "  nginx: $NGINX_RPS req/sec"

# Store results
declare -A RESULTS
RESULTS["nginx"]=$NGINX_RPS

# Test different Neurlang worker counts
for WORKERS in 0 1 2 4; do
    echo ""
    echo "Benchmarking Neurlang (workers=$WORKERS)..."

    # Stop existing Neurlang container
    docker compose stop nl 2>/dev/null || true
    docker compose rm -f nl 2>/dev/null || true

    # Start with new worker count
    Neurlang_WORKERS=$WORKERS docker compose up -d nl

    # Wait for Neurlang to be healthy
    for i in {1..30}; do
        if docker compose exec -T nl curl -sf http://127.0.0.1:8080/ > /dev/null 2>&1; then
            break
        fi
        sleep 1
    done
    sleep 2

    # Run benchmark
    Neurlang_RPS=$(run_docker_bench "nl" "http://nl:8080/")
    RESULTS["nl_w$WORKERS"]=$Neurlang_RPS

    # Calculate comparison
    if [ -n "$NGINX_RPS" ] && [ -n "$Neurlang_RPS" ]; then
        RATIO=$(echo "scale=2; $Neurlang_RPS / $NGINX_RPS" | bc)
        DIFF=$(echo "scale=1; (($Neurlang_RPS - $NGINX_RPS) / $NGINX_RPS) * 100" | bc)
        if (( $(echo "$DIFF > 0" | bc -l) )); then
            echo "  Neurlang (w=$WORKERS): $Neurlang_RPS req/sec (+${DIFF}% vs nginx, ${RATIO}x)"
        else
            echo "  Neurlang (w=$WORKERS): $Neurlang_RPS req/sec (${DIFF}% vs nginx, ${RATIO}x)"
        fi
    else
        echo "  Neurlang (w=$WORKERS): $Neurlang_RPS req/sec"
    fi
done

# Summary
echo ""
echo "=============================================="
echo "SUMMARY"
echo "=============================================="
echo "nginx (multi-worker): ${RESULTS[nginx]} req/sec"
echo ""
for WORKERS in 0 1 2 4; do
    RPS=${RESULTS["nl_w$WORKERS"]}
    if [ -n "$RPS" ] && [ -n "$NGINX_RPS" ]; then
        RATIO=$(echo "scale=2; $RPS / $NGINX_RPS" | bc)
        echo "Neurlang (workers=$WORKERS): $RPS req/sec (${RATIO}x nginx)"
    fi
done
echo "=============================================="

# Save JSON results
cat > results/worker_comparison.json << EOF
{
    "config": {
        "requests": $REQUESTS,
        "concurrency": $CONCURRENCY
    },
    "nginx_rps": ${RESULTS[nginx]},
    "nl_workers_0_rps": ${RESULTS[nl_w0]:-0},
    "nl_workers_1_rps": ${RESULTS[nl_w1]:-0},
    "nl_workers_2_rps": ${RESULTS[nl_w2]:-0},
    "nl_workers_4_rps": ${RESULTS[nl_w4]:-0}
}
EOF

echo ""
echo "Results saved to results/worker_comparison.json"

# Cleanup
docker compose down
