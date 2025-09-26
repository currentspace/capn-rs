#!/bin/bash

# Cap'n Web TypeScript Compatibility Validation Script
# Tests the Rust server against TypeScript examples

set -e

echo "========================================="
echo "Cap'n Web TypeScript Compatibility Tests"
echo "========================================="
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"

    echo -n "Testing $test_name... "

    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASSED${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ FAILED${NC}"
        ((TESTS_FAILED++))
    fi
}

# Check if server is running
echo "Checking server status..."
if curl -s http://localhost:3000/health > /dev/null; then
    echo -e "${GREEN}✅ Server is running on port 3000${NC}"
else
    echo -e "${YELLOW}⚠️ Server not running. Starting typescript_examples_server...${NC}"
    PORT=3000 cargo run --example typescript_examples_server -p capnweb-server 2>&1 &
    SERVER_PID=$!
    sleep 5
    if curl -s http://localhost:3000/health > /dev/null; then
        echo -e "${GREEN}✅ Server started successfully${NC}"
    else
        echo -e "${RED}❌ Failed to start server${NC}"
        exit 1
    fi
fi

echo ""
echo "Running compatibility tests..."
echo "================================"

# Test 1: Basic call expression
run_test "Basic call expression" 'curl -s -X POST http://localhost:3000/rpc/batch \
  -H "Content-Type: text/plain" \
  -d '\''["push",["call",1,["authenticate"],["cookie-123"]]]
["pull",1]'\'' | grep -q "Ada Lovelace"'

# Test 2: Pipeline expression
run_test "Pipeline expression" 'curl -s -X POST http://localhost:3000/rpc/batch \
  -H "Content-Type: text/plain" \
  -d '\''["push",["pipeline",1,["authenticate"],["cookie-123"]]]
["push",["pipeline",1,["getUserProfile"],[["pipeline",1,["id"]]]]]
["pull",1]
["pull",2]'\'' | grep -q "Mathematician"'

# Test 3: Multiple methods
run_test "Multiple methods" 'curl -s -X POST http://localhost:3000/rpc/batch \
  -H "Content-Type: text/plain" \
  -d '\''["push",["call",1,["authenticate"],["cookie-456"]]]
["push",["call",1,["getNotifications"],["u_2"]]]
["pull",1]
["pull",2]'\'' | grep -q "Alan Turing"'

# Test 4: Error handling - invalid session
run_test "Error handling" 'curl -s -X POST http://localhost:3000/rpc/batch \
  -H "Content-Type: text/plain" \
  -d '\''["push",["call",1,["authenticate"],["invalid-token"]]]
["pull",1]'\'' | grep -q "reject"'

# Test 5: Complex pipelining
run_test "Complex pipelining" 'response=$(curl -s -X POST http://localhost:3000/rpc/batch \
  -H "Content-Type: text/plain" \
  -d '\''["push",["pipeline",1,["authenticate"],["cookie-123"]]]
["push",["pipeline",1,["getUserProfile"],[["pipeline",1,["id"]]]]]
["push",["pipeline",1,["getNotifications"],[["pipeline",1,["id"]]]]]
["pull",1]
["pull",2]
["pull",3]'\'')
echo "$response" | grep -q "u_1" && echo "$response" | grep -q "first programmer" && echo "$response" | grep -q "Welcome"'

# Test 6: Array responses
run_test "Array responses" 'curl -s -X POST http://localhost:3000/rpc/batch \
  -H "Content-Type: text/plain" \
  -d '\''["push",["call",1,["getNotifications"],["u_1"]]]
["pull",1]'\'' | grep -q "\[\"Welcome to jsrpc!\",\"You have 2 new followers\"\]"'

# Test 7: Both expression types in same batch
run_test "Mixed expressions" 'curl -s -X POST http://localhost:3000/rpc/batch \
  -H "Content-Type: text/plain" \
  -d '\''["push",["call",1,["authenticate"],["cookie-123"]]]
["push",["pipeline",1,["getUserProfile"],[["pipeline",1,["id"]]]]]
["pull",1]
["pull",2]'\'' | grep -q "Ada Lovelace" && grep -q "first programmer"'

echo ""
echo "Running TypeScript client test (if available)..."
echo "================================"

# Check if TypeScript client exists and can be run
if [ -f "typescript-interop/capnweb-github/examples/batch-pipelining/client.mjs" ]; then
    echo "Found TypeScript client. Testing against Rust server..."

    # Try to run the TypeScript client
    cd typescript-interop/capnweb-github

    # Build if needed
    if [ ! -d "dist" ]; then
        echo "Building TypeScript library..."
        npm run build 2>/dev/null || echo -e "${YELLOW}⚠️ Could not build TypeScript library${NC}"
    fi

    # Run client test
    if [ -f "examples/batch-pipelining/client.mjs" ]; then
        echo "Running batch-pipelining client..."
        export RPC_URL="http://localhost:3000/rpc"
        export SIMULATED_RTT_MS=0
        export SIMULATED_RTT_JITTER_MS=0

        if timeout 10 node examples/batch-pipelining/client.mjs 2>/dev/null | grep -q "Ada Lovelace"; then
            echo -e "${GREEN}✅ TypeScript client test PASSED${NC}"
            ((TESTS_PASSED++))
        else
            echo -e "${RED}❌ TypeScript client test FAILED${NC}"
            ((TESTS_FAILED++))
        fi
    fi

    cd - > /dev/null
else
    echo -e "${YELLOW}⚠️ TypeScript client not found. Skipping client test.${NC}"
fi

echo ""
echo "========================================="
echo "Test Results Summary"
echo "========================================="
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! The Rust server is fully compatible with TypeScript examples!${NC}"
    exit 0
else
    echo -e "${RED}⚠️ Some tests failed. Please review the implementation.${NC}"
    exit 1
fi