#!/bin/bash
# Benchmark Neurlang with different worker counts vs nginx
# Runs inside Docker for fair comparison

set -e

REQUESTS=${REQUESTS:-10000}
CONCURRENCY=${CONCURRENCY:-50}
WARMUP=1000

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║         Neurlang Worker Count Comparison vs nginx                   ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Configuration: ${REQUESTS} requests, ${CONCURRENCY} concurrency"
echo ""

# Wait for services
echo -e "${YELLOW}Waiting for services...${NC}"
for i in {1..30}; do
    if curl -sf http://nginx:80/ > /dev/null 2>&1; then
        echo -e "  nginx: ${GREEN}ready${NC}"
        break
    fi
    sleep 1
done

for i in {1..30}; do
    if curl -sf http://nl:8080/ > /dev/null 2>&1; then
        echo -e "  Neurlang: ${GREEN}ready${NC}"
        break
    fi
    sleep 1
done

echo ""

# Benchmark function
run_bench() {
    local url=$1
    # Warmup
    ab -n $WARMUP -c $CONCURRENCY -q "$url" > /dev/null 2>&1 || true
    sleep 1
    # Actual benchmark
    ab -n $REQUESTS -c $CONCURRENCY "$url" 2>&1 | grep "Requests per second" | awk '{print $4}'
}

# Get nginx baseline
echo -e "${BOLD}Benchmarking nginx (baseline)...${NC}"
NGINX_RPS=$(run_bench "http://nginx:80/")
echo -e "  nginx: ${BLUE}${NGINX_RPS}${NC} req/sec"
echo ""

# Get Neurlang result
echo -e "${BOLD}Benchmarking Neurlang (WORKERS=${Neurlang_WORKERS:-0})...${NC}"
Neurlang_RPS=$(run_bench "http://nl:8080/")

# Calculate comparison
if [ -n "$NGINX_RPS" ] && [ -n "$Neurlang_RPS" ]; then
    DIFF=$(echo "scale=1; (($Neurlang_RPS - $NGINX_RPS) / $NGINX_RPS) * 100" | bc)
    RATIO=$(echo "scale=2; $Neurlang_RPS / $NGINX_RPS" | bc)

    if (( $(echo "$DIFF > 0" | bc -l) )); then
        echo -e "  Neurlang: ${GREEN}${Neurlang_RPS}${NC} req/sec (${GREEN}+${DIFF}%${NC} vs nginx)"
    else
        echo -e "  Neurlang: ${YELLOW}${Neurlang_RPS}${NC} req/sec (${YELLOW}${DIFF}%${NC} vs nginx)"
    fi
fi

echo ""
echo -e "${BOLD}Summary:${NC}"
echo "  nginx:  $NGINX_RPS req/sec"
echo "  Neurlang:  $Neurlang_RPS req/sec (workers=${Neurlang_WORKERS:-0})"
echo "  Ratio:  ${RATIO}x"
echo ""

# Save results
cat > /app/results/worker_comparison.json << EOF
{
    "workers": ${Neurlang_WORKERS:-0},
    "nginx_rps": $NGINX_RPS,
    "nl_rps": $Neurlang_RPS,
    "ratio": $RATIO,
    "diff_percent": $DIFF
}
EOF
