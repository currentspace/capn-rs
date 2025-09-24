#!/bin/bash

# Automated test runner for Cap'n Web Rust server
# This script starts a server, runs tests, and cleans up

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_PORT=${1:-9001}
SERVER_EXAMPLE=${2:-stateful_server}
SERVER_TIMEOUT=10
TEST_TIMEOUT=60

echo -e "${BLUE}🧪 Cap'n Web Test Runner${NC}"
echo -e "${BLUE}=========================${NC}"
echo -e "Port: ${TEST_PORT}"
echo -e "Server: ${SERVER_EXAMPLE}"
echo

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
    if [ ! -z "$SERVER_PID" ]; then
        echo "Killing server process $SERVER_PID"
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi

    # Kill any remaining processes on the test port
    lsof -ti:$TEST_PORT | xargs kill -9 2>/dev/null || true

    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Set up trap for cleanup
trap cleanup EXIT INT TERM

# Step 1: Build the server
echo -e "${BLUE}🔨 Building server...${NC}"
cargo build --example $SERVER_EXAMPLE -p capnweb-server
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Server build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Server built successfully${NC}"

# Step 2: Start the server
echo -e "${BLUE}🚀 Starting server on port $TEST_PORT...${NC}"
PORT=$TEST_PORT cargo run --example $SERVER_EXAMPLE -p capnweb-server > server.log 2>&1 &
SERVER_PID=$!

# Step 3: Wait for server to be ready
echo -e "${YELLOW}⏳ Waiting for server to start...${NC}"
for i in $(seq 1 $SERVER_TIMEOUT); do
    if curl -s http://127.0.0.1:$TEST_PORT/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Server is ready!${NC}"
        break
    fi
    if ! kill -0 $SERVER_PID 2>/dev/null; then
        echo -e "${RED}❌ Server process died during startup${NC}"
        echo -e "${RED}Server logs:${NC}"
        cat server.log
        exit 1
    fi
    sleep 1
    echo -n "."
done

# Check if server is actually ready
if ! curl -s http://127.0.0.1:$TEST_PORT/health > /dev/null 2>&1; then
    echo -e "${RED}❌ Server failed to start within $SERVER_TIMEOUT seconds${NC}"
    echo -e "${RED}Server logs:${NC}"
    cat server.log
    exit 1
fi

# Step 4: Build TypeScript tests
echo -e "${BLUE}🔨 Building TypeScript tests...${NC}"
cd typescript-interop
npm install
npm run build
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ TypeScript build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✅ TypeScript tests built${NC}"

# Step 5: Run tests
echo -e "${BLUE}🧪 Running tests against server...${NC}"

# Test 1: Official client test
echo -e "${YELLOW}📋 Running official client test...${NC}"
timeout $TEST_TIMEOUT node dist/official-client-test.js $TEST_PORT
OFFICIAL_TEST_RESULT=$?

# Test 2: Debug client test (basic connectivity)
echo -e "${YELLOW}📋 Running debug client test...${NC}"
timeout $TEST_TIMEOUT node dist/debug-client.js $TEST_PORT
DEBUG_TEST_RESULT=$?

# Test 3: Newline format test
echo -e "${YELLOW}📋 Running newline format test...${NC}"
timeout $TEST_TIMEOUT node dist/test-newline-format.js $TEST_PORT
NEWLINE_TEST_RESULT=$?

# Test 4: Comprehensive stateful test
echo -e "${YELLOW}📋 Running comprehensive stateful test...${NC}"
timeout $TEST_TIMEOUT node dist/comprehensive-stateful-test.js $TEST_PORT
COMPREHENSIVE_TEST_RESULT=$?

# Step 6: Report results
echo -e "\n${BLUE}📊 Test Results${NC}"
echo -e "${BLUE}===============${NC}"

if [ $OFFICIAL_TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ Official client test: PASSED${NC}"
else
    echo -e "${RED}❌ Official client test: FAILED (exit code: $OFFICIAL_TEST_RESULT)${NC}"
fi

if [ $DEBUG_TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ Debug client test: PASSED${NC}"
else
    echo -e "${RED}❌ Debug client test: FAILED (exit code: $DEBUG_TEST_RESULT)${NC}"
fi

if [ $NEWLINE_TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ Newline format test: PASSED${NC}"
else
    echo -e "${RED}❌ Newline format test: FAILED (exit code: $NEWLINE_TEST_RESULT)${NC}"
fi

if [ $COMPREHENSIVE_TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ Comprehensive stateful test: PASSED${NC}"
else
    echo -e "${RED}❌ Comprehensive stateful test: FAILED (exit code: $COMPREHENSIVE_TEST_RESULT)${NC}"
fi

# Calculate overall result
TOTAL_TESTS=4
PASSED_TESTS=0
[ $OFFICIAL_TEST_RESULT -eq 0 ] && PASSED_TESTS=$((PASSED_TESTS + 1))
[ $DEBUG_TEST_RESULT -eq 0 ] && PASSED_TESTS=$((PASSED_TESTS + 1))
[ $NEWLINE_TEST_RESULT -eq 0 ] && PASSED_TESTS=$((PASSED_TESTS + 1))
[ $COMPREHENSIVE_TEST_RESULT -eq 0 ] && PASSED_TESTS=$((PASSED_TESTS + 1))

echo -e "\n${BLUE}📈 Summary: $PASSED_TESTS/$TOTAL_TESTS tests passed${NC}"

if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}💥 Some tests failed${NC}"
    echo -e "${YELLOW}Check server logs in server.log for details${NC}"
    exit 1
fi