#!/bin/bash
# Compare Neurlang performance with different worker counts vs nginx

set -e

REQUESTS=${REQUESTS:-10000}
CONCURRENCY=${CONCURRENCY:-50}

echo "=============================================="
echo "Neurlang Worker Comparison vs nginx"
echo "Requests: $REQUESTS, Concurrency: $CONCURRENCY"
echo "=============================================="
echo ""

# Function to run benchmark
run_benchmark() {
    local name=$1
    local url=$2

    # Warmup
    ab -n 1000 -c $CONCURRENCY -q "$url" > /dev/null 2>&1 || true
    sleep 1

    # Benchmark
    local result=$(ab -n $REQUESTS -c $CONCURRENCY "$url" 2>&1 | grep "Requests per second" | awk '{print $4}')
    echo "$result"
}

# Start nginx
echo "Starting nginx..."
docker compose up -d nginx
sleep 3

# Get nginx baseline
echo -n "nginx (multi-worker):        "
NGINX_RPS=$(run_benchmark "nginx" "http://localhost:8089/")
echo "$NGINX_RPS req/sec"

# Test different Neurlang worker counts
for WORKERS in 0 1 2 4 8; do
    echo -n "Neurlang (workers=$WORKERS):         "

    # Stop any existing Neurlang container
    docker compose stop nl 2>/dev/null || true
    docker compose rm -f nl 2>/dev/null || true

    # Start with new worker count
    WORKERS=$WORKERS docker compose up -d nl
    sleep 2

    # Run benchmark
    Neurlang_RPS=$(run_benchmark "nl" "http://localhost:8088/")

    # Calculate comparison
    if [ -n "$NGINX_RPS" ] && [ -n "$Neurlang_RPS" ]; then
        RATIO=$(echo "scale=2; $Neurlang_RPS / $NGINX_RPS" | bc)
        DIFF=$(echo "scale=1; (($Neurlang_RPS - $NGINX_RPS) / $NGINX_RPS) * 100" | bc)
        echo "$Neurlang_RPS req/sec (${DIFF}% vs nginx, ${RATIO}x)"
    else
        echo "$Neurlang_RPS req/sec"
    fi
done

# Cleanup
docker compose down

echo ""
echo "Done!"
