#!/bin/bash
# Neurlang vs nginx Multi-Worker Performance Benchmark
# Tests Neurlang with different worker counts against nginx baseline
set -e

# Configuration from environment variables
REQUESTS=${REQUESTS:-10000}
CONCURRENCY=${CONCURRENCY:-50}
WARMUP_REQUESTS=1000

# URLs (both on Docker internal network for fair comparison)
NGINX_URL=${NGINX_URL:-http://nginx:80/}
Neurlang_URL=${Neurlang_URL:-http://nl:8080/}

# Worker counts to test
WORKER_COUNTS="${WORKER_COUNTS:-0 1 2 4}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║        Neurlang Worker Count Comparison vs nginx                    ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${CYAN}Configuration:${NC}"
echo -e "  Requests:    ${REQUESTS}"
echo -e "  Concurrency: ${CONCURRENCY}"
echo -e "  Warmup:      ${WARMUP_REQUESTS} requests"
echo -e "  Workers:     ${WORKER_COUNTS}"
echo ""

# Wait for services to be ready
echo -e "${YELLOW}Waiting for services...${NC}"

# Wait for nginx
for i in {1..30}; do
    if curl -s -o /dev/null -w "%{http_code}" "$NGINX_URL" 2>/dev/null | grep -q "200"; then
        echo -e "  nginx: ${GREEN}ready${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "  nginx: ${RED}timeout${NC}"
        exit 1
    fi
    sleep 1
done

# Wait for Neurlang
for i in {1..30}; do
    if curl -s -o /dev/null -w "%{http_code}" "$Neurlang_URL" 2>/dev/null | grep -q "200"; then
        echo -e "  Neurlang: ${GREEN}ready${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "  Neurlang: ${RED}timeout${NC}"
        exit 1
    fi
    sleep 1
done

echo ""

# Function to run benchmark and return RPS
run_bench() {
    local url=$1
    local name=$2

    # Warmup
    ab -n $WARMUP_REQUESTS -c $CONCURRENCY -q "$url" > /dev/null 2>&1 || true
    sleep 1

    # Actual benchmark - capture output
    local output=$(ab -n $REQUESTS -c $CONCURRENCY "$url" 2>&1)

    # Extract metrics
    local rps=$(echo "$output" | grep "Requests per second" | awk '{print $4}')
    local mean_time=$(echo "$output" | grep "Time per request.*mean\]$" | head -1 | awk '{print $4}')
    local p99=$(echo "$output" | grep "99%" | awk '{print $2}')

    echo "$rps|$mean_time|$p99"
}

# Benchmark nginx
echo -e "${BOLD}Benchmarking nginx (baseline)...${NC}"
NGINX_RESULT=$(run_bench "$NGINX_URL" "nginx")
NGINX_RPS=$(echo "$NGINX_RESULT" | cut -d'|' -f1)
NGINX_TIME=$(echo "$NGINX_RESULT" | cut -d'|' -f2)
NGINX_P99=$(echo "$NGINX_RESULT" | cut -d'|' -f3)
echo -e "  nginx: ${BLUE}${NGINX_RPS}${NC} req/sec"
echo ""

# Store results
declare -A RESULTS
declare -A TIMES
declare -A P99S
RESULTS["nginx"]=$NGINX_RPS
TIMES["nginx"]=$NGINX_TIME
P99S["nginx"]=$NGINX_P99

# Note: In Docker benchmark container, we test single Neurlang config
# The main comparison script handles multiple worker configs
echo -e "${BOLD}Benchmarking Neurlang (workers=${Neurlang_WORKERS:-0})...${NC}"
Neurlang_RESULT=$(run_bench "$Neurlang_URL" "nl")
Neurlang_RPS=$(echo "$Neurlang_RESULT" | cut -d'|' -f1)
Neurlang_TIME=$(echo "$Neurlang_RESULT" | cut -d'|' -f2)
Neurlang_P99=$(echo "$Neurlang_RESULT" | cut -d'|' -f3)

if [ -n "$NGINX_RPS" ] && [ -n "$Neurlang_RPS" ]; then
    RATIO=$(echo "scale=2; $Neurlang_RPS / $NGINX_RPS" | bc)
    DIFF=$(echo "scale=1; (($Neurlang_RPS - $NGINX_RPS) / $NGINX_RPS) * 100" | bc)
    if (( $(echo "$DIFF > 0" | bc -l) )); then
        echo -e "  Neurlang: ${GREEN}${Neurlang_RPS}${NC} req/sec (${GREEN}+${DIFF}%${NC} vs nginx, ${RATIO}x)"
    else
        echo -e "  Neurlang: ${YELLOW}${Neurlang_RPS}${NC} req/sec (${YELLOW}${DIFF}%${NC} vs nginx, ${RATIO}x)"
    fi
fi

echo ""

# Print summary table
echo -e "${BOLD}╔════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║                           RESULTS SUMMARY                                  ║${NC}"
echo -e "${BOLD}╠════════════════════════════════════════════════════════════════════════════╣${NC}"
echo -e "${BOLD}║${NC}                                                                            ${BOLD}║${NC}"
printf "${BOLD}║${NC}  %-30s  %12s  %10s  %8s  ${BOLD}║${NC}\n" "Configuration" "Requests/sec" "vs nginx" "p99 (ms)"
echo -e "${BOLD}║${NC}  ──────────────────────────────────────────────────────────────────────    ${BOLD}║${NC}"
printf "${BOLD}║${NC}  ${BLUE}%-30s${NC}  %12s  %10s  %8s  ${BOLD}║${NC}\n" "nginx (multi-worker)" "$NGINX_RPS" "baseline" "${NGINX_P99:-0}"
printf "${BOLD}║${NC}  ${GREEN}%-30s${NC}  %12s  %10s  %8s  ${BOLD}║${NC}\n" "Neurlang (workers=${Neurlang_WORKERS:-0})" "$Neurlang_RPS" "${RATIO}x" "${Neurlang_P99:-0}"
echo -e "${BOLD}║${NC}                                                                            ${BOLD}║${NC}"
echo -e "${BOLD}╠════════════════════════════════════════════════════════════════════════════╣${NC}"

# Winner announcement
if [ -n "$DIFF" ]; then
    if (( $(echo "$DIFF > 0" | bc -l) )); then
        printf "${BOLD}║${NC}  ${GREEN}★ Neurlang is %.1f%% FASTER than nginx${NC}" "$DIFF"
        echo -e "                                     ${BOLD}║${NC}"
    else
        DIFF_ABS=$(echo "$DIFF * -1" | bc)
        printf "${BOLD}║${NC}  ${YELLOW}◆ nginx is %.1f%% faster than Neurlang${NC}" "$DIFF_ABS"
        echo -e "                                      ${BOLD}║${NC}"
    fi
fi

echo -e "${BOLD}╚════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Save results to JSON
cat > /app/results/summary.json << EOF
{
    "benchmark": {
        "requests": $REQUESTS,
        "concurrency": $CONCURRENCY,
        "warmup": $WARMUP_REQUESTS,
        "nl_workers": ${Neurlang_WORKERS:-0}
    },
    "nginx": {
        "requests_per_second": $NGINX_RPS,
        "mean_time_ms": ${NGINX_TIME:-0},
        "p99_ms": ${NGINX_P99:-0}
    },
    "nl": {
        "requests_per_second": $Neurlang_RPS,
        "mean_time_ms": ${Neurlang_TIME:-0},
        "p99_ms": ${Neurlang_P99:-0}
    },
    "comparison": {
        "nl_to_nginx_ratio": $RATIO,
        "nl_faster_by_percent": $DIFF
    }
}
EOF

echo -e "${CYAN}Results saved to: /app/results/summary.json${NC}"
