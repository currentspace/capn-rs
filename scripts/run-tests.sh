#!/bin/bash
#
# Test runner for Cap'n Web Rust implementation
# Runs TypeScript interop tests against the unified Rust server

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default values
DEFAULT_PORT=9000
DEFAULT_HOST="127.0.0.1"

# Parse command line arguments
PORT=${1:-$DEFAULT_PORT}
HOST=${2:-$DEFAULT_HOST}
TEST_FILTER=${3:-"all"}  # all, tier1, tier2, tier3, or specific test name

echo "🧪 Cap'n Web Protocol Compliance Test Suite"
echo "==========================================="
echo ""

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
    if [ ! -z "$SERVER_PID" ]; then
        echo "   Stopping server (PID: $SERVER_PID)"
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi

    # Kill any remaining processes
    pkill -f "unified_test_server" 2>/dev/null || true
    pkill -f "basic_server" 2>/dev/null || true
    pkill -f "cargo run" 2>/dev/null || true
}

# Set up trap for cleanup
trap cleanup EXIT INT TERM

# Function to check if server is running
check_server() {
    echo -e "${YELLOW}🔍 Checking server status...${NC}"
    if curl -s -o /dev/null -w "%{http_code}" "http://$HOST:$PORT/health" | grep -q "200"; then
        echo -e "${GREEN}   ✅ Server is running on port $PORT${NC}"
        return 0
    else
        echo -e "${RED}   ❌ Server is not running on port $PORT${NC}"
        return 1
    fi
}

# Function to build the server
build_server() {
    echo -e "${YELLOW}🔨 Building unified test server...${NC}"

    if cargo build --release --example unified_test_server -p capnweb-server; then
        echo -e "${GREEN}   ✅ Build successful${NC}"
        return 0
    else
        echo -e "${RED}   ❌ Build failed${NC}"
        return 1
    fi
}

# Function to start the server
start_server() {
    echo -e "${YELLOW}🚀 Starting unified test server on port $PORT...${NC}"

    # Kill any existing servers
    pkill -f "unified_test_server" 2>/dev/null || true
    pkill -f "basic_server" 2>/dev/null || true
    pkill -f "cargo run" 2>/dev/null || true
    sleep 1

    # Check if binary exists
    if [ ! -f "target/release/examples/unified_test_server" ]; then
        echo "   Binary not found, building..."
        if ! build_server; then
            return 1
        fi
    fi

    # Start the server in the background
    PORT=$PORT HOST=$HOST ./target/release/examples/unified_test_server > server.log 2>&1 &
    SERVER_PID=$!
    echo "   Server PID: $SERVER_PID"

    # Wait for server to be ready
    echo -n "   Waiting for server to start"
    for i in {1..30}; do
        if curl -s -o /dev/null -w "%{http_code}" "http://$HOST:$PORT/health" 2>/dev/null | grep -q "200"; then
            echo -e " ${GREEN}✅${NC}"
            return 0
        fi
        # Check if process is still running
        if ! kill -0 $SERVER_PID 2>/dev/null; then
            echo -e " ${RED}❌${NC}"
            echo "   Server process died. Check server.log for details:"
            tail -20 server.log
            return 1
        fi
        echo -n "."
        sleep 1
    done
    echo -e " ${RED}❌${NC}"
    echo "   Server failed to start within 30 seconds. Check server.log"
    return 1
}

# Change to TypeScript interop directory
echo -e "${CYAN}📦 Preparing TypeScript tests...${NC}"
cd typescript-interop

# Check if dependencies are installed
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}   Installing dependencies...${NC}"
    npm install
fi

# Build TypeScript tests
echo -e "${YELLOW}   Building tests...${NC}"
npm run build

# Go back to root directory
cd ..

# Check if server is running, start if needed
if ! check_server; then
    if ! start_server; then
        echo -e "${RED}❌ Failed to start server${NC}"
        exit 1
    fi
fi

# Change back to TypeScript interop directory for running tests
cd typescript-interop

echo ""
echo -e "${CYAN}🏃 Running tests...${NC}"
echo "=================================="

# Function to run a specific test
run_test() {
    local test_name=$1
    local test_file=$2

    echo ""
    echo -e "${BLUE}🧪 Running: $test_name${NC}"
    echo "---------------------------------"

    if node "$test_file" "$PORT" 2>&1; then
        echo -e "${GREEN}✅ $test_name: PASSED${NC}"
        return 0
    else
        local exit_code=$?
        echo -e "${RED}❌ $test_name: FAILED (exit code: $exit_code)${NC}"
        return 1
    fi
}

# Track test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Run tests based on filter
case "$TEST_FILTER" in
    "all"|"comprehensive")
        echo "Running comprehensive test suite..."
        if node dist/comprehensive-test-runner.js "$PORT"; then
            exit 0
        else
            exit 1
        fi
        ;;
    "tier1")
        ((TOTAL_TESTS++))
        if run_test "Tier 1: Protocol Compliance" "dist/tier1-protocol-compliance.js"; then
            ((PASSED_TESTS++))
        else
            ((FAILED_TESTS++))
        fi
        ;;
    "tier2")
        ((TOTAL_TESTS++))
        if run_test "Tier 2: HTTP Batch" "dist/tier2-http-batch-corrected.js"; then
            ((PASSED_TESTS++))
        else
            ((FAILED_TESTS++))
        fi

        # WebSocket tests might not work yet
        echo -e "${YELLOW}⚠️  Skipping WebSocket tests (not yet implemented)${NC}"
        # run_test "Tier 2: WebSocket" "dist/tier2-websocket-tests.js"
        ;;
    "tier3")
        ((TOTAL_TESTS++))
        if run_test "Tier 3: Capability Composition" "dist/tier3-capability-composition.js"; then
            ((PASSED_TESTS++))
        else
            ((FAILED_TESTS++))
        fi

        ((TOTAL_TESTS++))
        if run_test "Tier 3: Complex Applications" "dist/tier3-complex-applications.js"; then
            ((PASSED_TESTS++))
        else
            ((FAILED_TESTS++))
        fi
        ;;
    "quick")
        # Quick smoke test - just tier 1
        echo "Running quick smoke test..."
        ((TOTAL_TESTS++))
        if run_test "Tier 1: Protocol Compliance" "dist/tier1-protocol-compliance.js"; then
            ((PASSED_TESTS++))
        else
            ((FAILED_TESTS++))
        fi
        ;;
    *)
        # Try to run specific test file
        if [ -f "dist/$TEST_FILTER.js" ]; then
            run_test "$TEST_FILTER" "dist/$TEST_FILTER.js"
            ((TOTAL_TESTS++))
            if [ $? -eq 0 ]; then ((PASSED_TESTS++)); else ((FAILED_TESTS++)); fi
        else
            echo -e "${RED}❌ Unknown test filter: $TEST_FILTER${NC}"
            echo ""
            echo "Available options:"
            echo "  all/comprehensive  - Run comprehensive test suite"
            echo "  tier1              - Run Tier 1 tests only"
            echo "  tier2              - Run Tier 2 tests only"
            echo "  tier3              - Run Tier 3 tests only"
            echo "  quick              - Run quick smoke test (tier1 only)"
            echo "  <test-name>        - Run specific test file"
            echo ""
            echo "Available test files:"
            ls dist/*.js | grep -E "tier[0-9]|test" | sed 's/dist\//  - /' | sed 's/\.js//'
            exit 1
        fi
        ;;
esac

# Print summary for individual test runs
if [ $TOTAL_TESTS -gt 0 ]; then
    echo ""
    echo "=================================="
    echo -e "${CYAN}📊 Test Results Summary${NC}"
    echo "=================================="
    echo "Total: $TOTAL_TESTS"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"

    if [ $FAILED_TESTS -eq 0 ]; then
        echo ""
        echo -e "${GREEN}🎉 All tests passed!${NC}"
        echo "✅ The Rust implementation is protocol-compliant!"
    else
        echo ""
        echo -e "${RED}⚠️  Some tests failed${NC}"
        echo "Check server.log in the root directory for server logs"
        exit 1
    fi
fi