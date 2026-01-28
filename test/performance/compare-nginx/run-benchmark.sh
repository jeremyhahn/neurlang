#!/bin/bash
# Neurlang vs nginx Full Multi-Worker Benchmark
# Tests Neurlang with workers=0,1,2,4 against nginx and outputs summary table
set -e
cd "$(dirname "$0")"

REQUESTS=${REQUESTS:-10000}
CONCURRENCY=${CONCURRENCY:-50}
WARMUP=1000

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║            Neurlang Multi-Worker Performance Comparison vs nginx                ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${CYAN}Configuration:${NC}"
echo -e "  Requests:    ${REQUESTS}"
echo -e "  Concurrency: ${CONCURRENCY}"
echo -e "  Warmup:      ${WARMUP} requests"
echo -e "  Workers:     0 (single), 1, 2, 4 (SO_REUSEPORT)"
echo ""

# Cleanup
docker compose down -v 2>/dev/null || true
mkdir -p results

# Build images
echo -e "${YELLOW}Building Docker images...${NC}"
docker compose build --quiet 2>&1 | tail -2

# Start nginx
echo ""
echo -e "${YELLOW}Starting nginx...${NC}"
docker compose up -d nginx
sleep 2

# Wait for nginx
for i in {1..30}; do
    if docker exec perf-nginx wget -qO- http://127.0.0.1:80/ >/dev/null 2>&1; then
        echo -e "  nginx: ${GREEN}ready${NC}"
        break
    fi
    sleep 1
done

# Function to run benchmark and show full ab output
run_bench_verbose() {
    local url=$1
    local name=$2

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}ApacheBench: ${CYAN}$name${NC}"
    echo -e "${BOLD}URL: $url${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo ""

    docker run --rm --network compare-nginx_perf-net alpine:latest sh -c "
        apk add --no-cache apache2-utils >/dev/null 2>&1

        echo 'Warming up with $WARMUP requests...'
        ab -n $WARMUP -c $CONCURRENCY -q '$url' >/dev/null 2>&1 || true
        sleep 1

        echo 'Running benchmark ($REQUESTS requests, $CONCURRENCY concurrency)...'
        echo ''
        ab -n $REQUESTS -c $CONCURRENCY '$url' 2>&1
    "
    echo ""
}

# Function to extract metrics (runs silently)
extract_metrics() {
    local url=$1
    docker run --rm --network compare-nginx_perf-net alpine:latest sh -c "
        apk add --no-cache apache2-utils >/dev/null 2>&1
        output=\$(ab -n $REQUESTS -c $CONCURRENCY -q '$url' 2>&1)
        rps=\$(echo \"\$output\" | grep 'Requests per second' | awk '{print \$4}')
        p99=\$(echo \"\$output\" | grep '99%' | awk '{print \$2}')
        echo \"\$rps|\$p99\"
    " 2>/dev/null
}

# Benchmark nginx with full output
echo ""
echo -e "${BOLD}Benchmarking nginx (multi-worker baseline)...${NC}"
run_bench_verbose "http://nginx:80/" "nginx"

# Extract nginx metrics
NGINX_METRICS=$(extract_metrics "http://nginx:80/")
NGINX_RPS=$(echo "$NGINX_METRICS" | cut -d'|' -f1)
NGINX_P99=$(echo "$NGINX_METRICS" | cut -d'|' -f2)
echo -e "  ${BOLD}nginx Result:${NC} ${BLUE}${NGINX_RPS}${NC} req/sec (p99: ${NGINX_P99}ms)"

# Arrays to store results
declare -a WORKER_COUNTS=(0 1 2 4)
declare -A Neurlang_RPS
declare -A Neurlang_P99
declare -A Neurlang_RATIO
declare -A Neurlang_DIFF

# Test each worker count
for WORKERS in "${WORKER_COUNTS[@]}"; do
    echo ""
    if [ "$WORKERS" -eq 0 ]; then
        CONFIG_NAME="Neurlang (single-threaded)"
    else
        CONFIG_NAME="Neurlang (workers=$WORKERS, SO_REUSEPORT)"
    fi
    echo -e "${BOLD}Benchmarking $CONFIG_NAME...${NC}"

    # Stop existing Neurlang
    docker compose stop nl 2>/dev/null || true
    docker compose rm -f nl 2>/dev/null || true

    # Start with worker count
    Neurlang_WORKERS=$WORKERS docker compose up -d nl
    sleep 2

    # Wait for healthy
    for i in {1..30}; do
        if docker exec perf-nl curl -sf http://127.0.0.1:8080/ >/dev/null 2>&1; then
            break
        fi
        sleep 1
    done
    sleep 1

    # Run benchmark with full output
    run_bench_verbose "http://nl:8080/" "$CONFIG_NAME"

    # Extract metrics
    METRICS=$(extract_metrics "http://nl:8080/")
    RPS=$(echo "$METRICS" | cut -d'|' -f1)
    P99=$(echo "$METRICS" | cut -d'|' -f2)

    Neurlang_RPS[$WORKERS]=$RPS
    Neurlang_P99[$WORKERS]=$P99

    if [ -n "$NGINX_RPS" ] && [ -n "$RPS" ]; then
        RATIO=$(echo "scale=2; $RPS / $NGINX_RPS" | bc)
        DIFF=$(echo "scale=1; (($RPS - $NGINX_RPS) / $NGINX_RPS) * 100" | bc)
        Neurlang_RATIO[$WORKERS]=$RATIO
        Neurlang_DIFF[$WORKERS]=$DIFF

        if (( $(echo "$DIFF > 0" | bc -l) )); then
            echo -e "  ${BOLD}Result:${NC} ${GREEN}${RPS}${NC} req/sec (${GREEN}+${DIFF}%${NC} vs nginx, ${RATIO}x)"
        else
            echo -e "  ${BOLD}Result:${NC} ${YELLOW}${RPS}${NC} req/sec (${YELLOW}${DIFF}%${NC} vs nginx, ${RATIO}x)"
        fi
    else
        echo -e "  ${BOLD}Result:${NC} ${RPS} req/sec"
    fi
done

echo ""
echo ""

# Find best configuration
BEST_WORKERS=0
BEST_RPS=0
for W in "${WORKER_COUNTS[@]}"; do
    if [ -n "${Neurlang_RPS[$W]}" ]; then
        if (( $(echo "${Neurlang_RPS[$W]} > $BEST_RPS" | bc -l) )); then
            BEST_RPS=${Neurlang_RPS[$W]}
            BEST_WORKERS=$W
        fi
    fi
done

# Print summary table
echo -e "${BOLD}╔════════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║                              RESULTS SUMMARY                                   ║${NC}"
echo -e "${BOLD}╠════════════════════════════════════════════════════════════════════════════════╣${NC}"
echo -e "${BOLD}║${NC}                                                                                ${BOLD}║${NC}"
printf "${BOLD}║${NC}  %-38s  %12s  %10s  %8s  ${BOLD}║${NC}\n" "Configuration" "Requests/sec" "vs nginx" "p99 (ms)"
echo -e "${BOLD}║${NC}  ────────────────────────────────────────────────────────────────────────────  ${BOLD}║${NC}"
printf "${BOLD}║${NC}  ${BLUE}%-38s${NC}  %12s  %10s  %8s  ${BOLD}║${NC}\n" "nginx (multi-worker)" "$NGINX_RPS" "baseline" "${NGINX_P99:-0}"
echo -e "${BOLD}║${NC}  ────────────────────────────────────────────────────────────────────────────  ${BOLD}║${NC}"

for W in "${WORKER_COUNTS[@]}"; do
    RPS=${Neurlang_RPS[$W]}
    RATIO=${Neurlang_RATIO[$W]}
    DIFF=${Neurlang_DIFF[$W]}
    P99=${Neurlang_P99[$W]}

    if [ "$W" -eq 0 ]; then
        CONFIG="Neurlang (workers=0, single-threaded)"
    else
        CONFIG="Neurlang (workers=$W, SO_REUSEPORT)"
    fi

    if [ "$W" -eq "$BEST_WORKERS" ]; then
        # Highlight best config
        printf "${BOLD}║${NC}  ${GREEN}%-38s${NC}  ${GREEN}%12s${NC}  ${GREEN}%10s${NC}  %8s  ${BOLD}║${NC}\n" "$CONFIG ★" "$RPS" "${RATIO}x" "${P99:-0}"
    elif (( $(echo "$DIFF > 0" | bc -l) )); then
        printf "${BOLD}║${NC}  ${GREEN}%-38s${NC}  %12s  ${GREEN}%10s${NC}  %8s  ${BOLD}║${NC}\n" "$CONFIG" "$RPS" "${RATIO}x" "${P99:-0}"
    else
        printf "${BOLD}║${NC}  ${YELLOW}%-38s${NC}  %12s  ${YELLOW}%10s${NC}  %8s  ${BOLD}║${NC}\n" "$CONFIG" "$RPS" "${RATIO}x" "${P99:-0}"
    fi
done

echo -e "${BOLD}║${NC}                                                                                ${BOLD}║${NC}"
echo -e "${BOLD}╠════════════════════════════════════════════════════════════════════════════════╣${NC}"

# Winner announcement
BEST_DIFF=${Neurlang_DIFF[$BEST_WORKERS]}
BEST_RATIO=${Neurlang_RATIO[$BEST_WORKERS]}
if (( $(echo "$BEST_DIFF > 0" | bc -l) )); then
    printf "${BOLD}║${NC}  ${GREEN}★ Best: Neurlang workers=%d is %.1f%% FASTER than nginx (%.2fx throughput)${NC}" "$BEST_WORKERS" "$BEST_DIFF" "$BEST_RATIO"
    echo -e "    ${BOLD}║${NC}"
else
    printf "${BOLD}║${NC}  ${YELLOW}◆ nginx is faster in all configurations${NC}"
    echo -e "                                  ${BOLD}║${NC}"
fi

echo -e "${BOLD}╚════════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Save comprehensive results to JSON
cat > results/worker_comparison.json << EOF
{
    "benchmark": {
        "date": "$(date -Iseconds)",
        "requests": $REQUESTS,
        "concurrency": $CONCURRENCY,
        "warmup": $WARMUP,
        "environment": "Docker bridge network"
    },
    "nginx": {
        "workers": "auto (multi-worker)",
        "requests_per_second": $NGINX_RPS,
        "p99_ms": ${NGINX_P99:-0}
    },
    "nl": {
        "workers_0": {
            "description": "Single-threaded",
            "requests_per_second": ${Neurlang_RPS[0]:-0},
            "vs_nginx_ratio": ${Neurlang_RATIO[0]:-0},
            "vs_nginx_percent": ${Neurlang_DIFF[0]:-0},
            "p99_ms": ${Neurlang_P99[0]:-0}
        },
        "workers_1": {
            "description": "1 worker (SO_REUSEPORT)",
            "requests_per_second": ${Neurlang_RPS[1]:-0},
            "vs_nginx_ratio": ${Neurlang_RATIO[1]:-0},
            "vs_nginx_percent": ${Neurlang_DIFF[1]:-0},
            "p99_ms": ${Neurlang_P99[1]:-0}
        },
        "workers_2": {
            "description": "2 workers (SO_REUSEPORT)",
            "requests_per_second": ${Neurlang_RPS[2]:-0},
            "vs_nginx_ratio": ${Neurlang_RATIO[2]:-0},
            "vs_nginx_percent": ${Neurlang_DIFF[2]:-0},
            "p99_ms": ${Neurlang_P99[2]:-0}
        },
        "workers_4": {
            "description": "4 workers (SO_REUSEPORT)",
            "requests_per_second": ${Neurlang_RPS[4]:-0},
            "vs_nginx_ratio": ${Neurlang_RATIO[4]:-0},
            "vs_nginx_percent": ${Neurlang_DIFF[4]:-0},
            "p99_ms": ${Neurlang_P99[4]:-0}
        }
    },
    "summary": {
        "best_config": "workers=$BEST_WORKERS",
        "best_rps": $BEST_RPS,
        "best_vs_nginx_percent": ${Neurlang_DIFF[$BEST_WORKERS]:-0},
        "recommendation": "Use --workers 1 or --workers 4 for server workloads"
    }
}
EOF

echo -e "${CYAN}Results saved to: results/worker_comparison.json${NC}"
echo ""

# Cleanup
docker compose down
