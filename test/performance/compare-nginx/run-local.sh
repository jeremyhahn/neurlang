#!/bin/bash
set -e

# Local benchmark script (no Docker required)
# Usage: ./run-local.sh [requests] [concurrency]

REQUESTS=${1:-10000}
CONCURRENCY=${2:-10}
WARMUP_REQUESTS=1000

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"

mkdir -p "$RESULTS_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Check for ab (ApacheBench)
if ! command -v ab &> /dev/null; then
    echo -e "${RED}Error: ApacheBench (ab) not found. Install with: sudo apt install apache2-utils${NC}"
    exit 1
fi

echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║           Neurlang vs nginx Performance Comparison                  ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${CYAN}Configuration:${NC}"
echo -e "  Requests:    ${REQUESTS}"
echo -e "  Concurrency: ${CONCURRENCY}"
echo -e "  Warmup:      ${WARMUP_REQUESTS} requests"
echo ""

# Check if Neurlang server is running
Neurlang_URL="http://127.0.0.1:8080/values"
NGINX_URL="http://127.0.0.1:80/values"

# Start Neurlang server if not running
if ! curl -s -o /dev/null "$Neurlang_URL" 2>/dev/null; then
    echo -e "${YELLOW}Starting Neurlang server...${NC}"
    rm -f "$PROJECT_ROOT/state.db"
    "$PROJECT_ROOT/target/release/nl" run -i "$PROJECT_ROOT/examples/rest_api_crud.nl" &
    Neurlang_PID=$!
    sleep 2
    echo -e "  Neurlang PID: $Neurlang_PID"
else
    echo -e "${GREEN}Neurlang server already running${NC}"
    Neurlang_PID=""
fi

# Check if nginx is running
if ! curl -s -o /dev/null "http://127.0.0.1:80/" 2>/dev/null; then
    echo -e "${YELLOW}Note: nginx not running on port 80. Skipping nginx benchmark.${NC}"
    SKIP_NGINX=1
else
    echo -e "${GREEN}nginx server detected on port 80${NC}"
    SKIP_NGINX=0
fi

echo ""

# Function to run benchmark
run_benchmark() {
    local name=$1
    local url=$2
    local output_file=$3

    echo -e "${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}Benchmarking: ${CYAN}$name${NC}"
    echo -e "${BOLD}URL: ${url}${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════════════════${NC}"
    echo ""

    # Warmup
    echo -e "${YELLOW}Warming up with ${WARMUP_REQUESTS} requests...${NC}"
    ab -n $WARMUP_REQUESTS -c $CONCURRENCY -q "$url" > /dev/null 2>&1 || true
    sleep 1

    # Actual benchmark
    echo -e "${GREEN}Running benchmark...${NC}"
    echo ""
    ab -n $REQUESTS -c $CONCURRENCY "$url" 2>&1 | tee "$output_file"
    echo ""
}

# Function to extract metrics
extract_rps() {
    grep "Requests per second" "$1" 2>/dev/null | awk '{print $4}'
}

extract_mean_time() {
    grep "Time per request.*mean\]$" "$1" 2>/dev/null | head -1 | awk '{print $4}'
}

extract_p99() {
    grep "99%" "$1" 2>/dev/null | awk '{print $2}'
}

# Run Neurlang benchmark
Neurlang_OUTPUT="$RESULTS_DIR/nl_benchmark.txt"
run_benchmark "Neurlang" "$Neurlang_URL" "$Neurlang_OUTPUT"

Neurlang_RPS=$(extract_rps "$Neurlang_OUTPUT")
Neurlang_TIME=$(extract_mean_time "$Neurlang_OUTPUT")
Neurlang_P99=$(extract_p99 "$Neurlang_OUTPUT")

# Run nginx benchmark if available
if [ "$SKIP_NGINX" = "0" ]; then
    NGINX_OUTPUT="$RESULTS_DIR/nginx_benchmark.txt"

    # For nginx, we need to hit a valid endpoint - try /values or /
    if curl -s -o /dev/null -w "%{http_code}" "$NGINX_URL" 2>/dev/null | grep -q "200"; then
        run_benchmark "nginx" "$NGINX_URL" "$NGINX_OUTPUT"
    else
        echo -e "${YELLOW}nginx /values endpoint not configured, using root endpoint${NC}"
        NGINX_URL="http://127.0.0.1:80/"
        run_benchmark "nginx" "$NGINX_URL" "$NGINX_OUTPUT"
    fi

    NGINX_RPS=$(extract_rps "$NGINX_OUTPUT")
    NGINX_TIME=$(extract_mean_time "$NGINX_OUTPUT")
    NGINX_P99=$(extract_p99 "$NGINX_OUTPUT")
fi

# Print summary
echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║                        RESULTS SUMMARY                           ║${NC}"
echo -e "${BOLD}╠══════════════════════════════════════════════════════════════════╣${NC}"
echo -e "${BOLD}║${NC}                                                                  ${BOLD}║${NC}"
printf "${BOLD}║${NC}  %-12s %12s req/sec  %8s ms/req  %6s ms p99  ${BOLD}║${NC}\n" "Server" "Throughput" "Latency" ""
echo -e "${BOLD}║${NC}  ────────────────────────────────────────────────────────────    ${BOLD}║${NC}"

if [ "$SKIP_NGINX" = "0" ] && [ -n "$NGINX_RPS" ]; then
    printf "${BOLD}║${NC}  ${BLUE}%-12s${NC} %12s req/sec  %8s ms/req  %6s ms p99  ${BOLD}║${NC}\n" "nginx" "$NGINX_RPS" "$NGINX_TIME" "$NGINX_P99"
fi

printf "${BOLD}║${NC}  ${GREEN}%-12s${NC} %12s req/sec  %8s ms/req  %6s ms p99  ${BOLD}║${NC}\n" "Neurlang" "$Neurlang_RPS" "$Neurlang_TIME" "$Neurlang_P99"
echo -e "${BOLD}║${NC}                                                                  ${BOLD}║${NC}"

# Comparison if both available
if [ "$SKIP_NGINX" = "0" ] && [ -n "$NGINX_RPS" ] && [ -n "$Neurlang_RPS" ]; then
    DIFF=$(echo "scale=2; (($Neurlang_RPS - $NGINX_RPS) / $NGINX_RPS) * 100" | bc)
    RATIO=$(echo "scale=2; $Neurlang_RPS / $NGINX_RPS" | bc)

    echo -e "${BOLD}╠══════════════════════════════════════════════════════════════════╣${NC}"

    if (( $(echo "$DIFF > 0" | bc -l) )); then
        printf "${BOLD}║${NC}  ${GREEN}★ Neurlang is %.1f%% FASTER than nginx (%.2fx throughput)${NC}" "$DIFF" "$RATIO"
        echo -e "          ${BOLD}║${NC}"
    elif (( $(echo "$DIFF < 0" | bc -l) )); then
        DIFF_ABS=$(echo "$DIFF * -1" | bc)
        printf "${BOLD}║${NC}  ${YELLOW}◆ nginx is %.1f%% faster than Neurlang${NC}" "$DIFF_ABS"
        echo -e "                          ${BOLD}║${NC}"
    else
        echo -e "${BOLD}║${NC}  ${CYAN}◆ Performance is equal${NC}                                        ${BOLD}║${NC}"
    fi

    # Save summary JSON
    cat > "$RESULTS_DIR/summary.json" << EOF
{
    "nginx": {
        "requests_per_second": $NGINX_RPS,
        "mean_time_ms": $NGINX_TIME,
        "p99_ms": ${NGINX_P99:-0}
    },
    "nl": {
        "requests_per_second": $Neurlang_RPS,
        "mean_time_ms": $Neurlang_TIME,
        "p99_ms": ${Neurlang_P99:-0}
    },
    "comparison": {
        "nl_faster_by_percent": $DIFF,
        "nl_to_nginx_ratio": $RATIO
    }
}
EOF
else
    # Save Neurlang-only summary
    cat > "$RESULTS_DIR/summary.json" << EOF
{
    "nl": {
        "requests_per_second": $Neurlang_RPS,
        "mean_time_ms": $Neurlang_TIME,
        "p99_ms": ${Neurlang_P99:-0}
    }
}
EOF
fi

echo -e "${BOLD}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${CYAN}Results saved to: $RESULTS_DIR/${NC}"

# Cleanup
if [ -n "$Neurlang_PID" ]; then
    echo -e "${YELLOW}Stopping Neurlang server (PID: $Neurlang_PID)...${NC}"
    kill $Neurlang_PID 2>/dev/null || true
fi

echo -e "${GREEN}Done!${NC}"
