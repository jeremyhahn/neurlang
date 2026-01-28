#!/bin/bash
# Direct worker count comparison
set -e
cd "$(dirname "$0")"

REQUESTS=${REQUESTS:-10000}
CONCURRENCY=${CONCURRENCY:-50}

echo ""
echo "=============================================="
echo "Neurlang Worker Count Comparison vs nginx"
echo "Requests: $REQUESTS, Concurrency: $CONCURRENCY"
echo "=============================================="

# Cleanup
docker compose down -v 2>/dev/null || true

# Start nginx
echo ""
echo "Starting nginx..."
docker compose up -d nginx
sleep 3

# Wait for healthy
for i in {1..30}; do
    if docker exec perf-nginx wget -qO- http://127.0.0.1:80/ >/dev/null 2>&1; then
        echo "nginx ready"
        break
    fi
    sleep 1
done

# Benchmark nginx
echo ""
echo "Benchmarking nginx..."
NGINX_RPS=$(docker run --rm --network compare-nginx_perf-net alpine:latest sh -c "
    apk add --no-cache apache2-utils >/dev/null 2>&1
    ab -n 1000 -c $CONCURRENCY -q http://nginx:80/ >/dev/null 2>&1
    sleep 1
    ab -n $REQUESTS -c $CONCURRENCY http://nginx:80/ 2>&1 | grep 'Requests per second' | awk '{print \$4}'
")
echo "nginx: $NGINX_RPS req/sec"

# Test Neurlang with different worker counts
declare -A RESULTS
RESULTS["nginx"]=$NGINX_RPS

for WORKERS in 0 1 2 4; do
    echo ""
    echo "Testing Neurlang with workers=$WORKERS..."

    # Stop existing Neurlang
    docker compose stop nl 2>/dev/null || true
    docker compose rm -f nl 2>/dev/null || true

    # Start with worker count
    Neurlang_WORKERS=$WORKERS docker compose up -d nl
    sleep 3

    # Wait for healthy
    for i in {1..30}; do
        if docker exec perf-nl curl -sf http://127.0.0.1:8080/ >/dev/null 2>&1; then
            break
        fi
        sleep 1
    done
    sleep 1

    # Benchmark
    Neurlang_RPS=$(docker run --rm --network compare-nginx_perf-net alpine:latest sh -c "
        apk add --no-cache apache2-utils >/dev/null 2>&1
        ab -n 1000 -c $CONCURRENCY -q http://nl:8080/ >/dev/null 2>&1
        sleep 1
        ab -n $REQUESTS -c $CONCURRENCY http://nl:8080/ 2>&1 | grep 'Requests per second' | awk '{print \$4}'
    ")

    RESULTS["w$WORKERS"]=$Neurlang_RPS

    if [ -n "$NGINX_RPS" ] && [ -n "$Neurlang_RPS" ]; then
        RATIO=$(echo "scale=2; $Neurlang_RPS / $NGINX_RPS" | bc)
        DIFF=$(echo "scale=1; (($Neurlang_RPS - $NGINX_RPS) / $NGINX_RPS) * 100" | bc)
        echo "Neurlang (workers=$WORKERS): $Neurlang_RPS req/sec (${DIFF}% vs nginx, ${RATIO}x)"
    else
        echo "Neurlang (workers=$WORKERS): $Neurlang_RPS req/sec"
    fi
done

echo ""
echo "=============================================="
echo "SUMMARY"
echo "=============================================="
printf "%-20s %15s %10s\n" "Server" "Requests/sec" "vs nginx"
echo "----------------------------------------------"
printf "%-20s %15s %10s\n" "nginx (multi-worker)" "${RESULTS[nginx]}" "baseline"

for WORKERS in 0 1 2 4; do
    RPS=${RESULTS["w$WORKERS"]}
    if [ -n "$RPS" ] && [ -n "$NGINX_RPS" ]; then
        RATIO=$(echo "scale=2; $RPS / $NGINX_RPS" | bc)x
        printf "%-20s %15s %10s\n" "Neurlang (workers=$WORKERS)" "$RPS" "$RATIO"
    fi
done
echo "=============================================="

# Cleanup
docker compose down
