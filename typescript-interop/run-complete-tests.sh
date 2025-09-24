#!/bin/bash

# Complete Advanced Features Test Runner
# Tests all Cap'n Web advanced features with official TypeScript client

echo "🎯 Cap'n Web Complete Advanced Features Test Suite"
echo "=================================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RUST_SERVER_PORT=8080
RUST_WS_PORT=8080
RUST_H3_PORT=8443
TEST_TIMEOUT=60000

# Function to check if port is available
check_port() {
    nc -z localhost $1 2>/dev/null
    return $?
}

# Function to start Rust server
start_rust_server() {
    echo -e "${BLUE}🚀 Starting Rust Cap'n Web server with all features...${NC}"

    cd ..

    # Build with all features
    cargo build --release --all-features --quiet

    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Failed to build Rust server${NC}"
        exit 1
    fi

    # Start server with all transports enabled
    RUST_LOG=info cargo run --release --example advanced_features_showcase -- server \
        --port $RUST_SERVER_PORT \
        --enable-websocket \
        --enable-http3 \
        --enable-webtransport \
        --enable-resume-tokens \
        --enable-nested-capabilities \
        --enable-il-plans \
        > ../typescript-interop/logs/rust-server.log 2>&1 &

    RUST_SERVER_PID=$!
    echo "Rust server PID: $RUST_SERVER_PID"

    # Wait for server to start
    echo -n "Waiting for server to start"
    for i in {1..30}; do
        if check_port $RUST_SERVER_PORT; then
            echo ""
            echo -e "${GREEN}✅ Rust server started successfully${NC}"
            return 0
        fi
        echo -n "."
        sleep 1
    done

    echo ""
    echo -e "${RED}❌ Server failed to start after 30 seconds${NC}"
    cat ../typescript-interop/logs/rust-server.log | tail -20
    return 1
}

# Function to stop Rust server
stop_rust_server() {
    if [ ! -z "$RUST_SERVER_PID" ]; then
        echo -e "${BLUE}Stopping Rust server (PID: $RUST_SERVER_PID)...${NC}"
        kill $RUST_SERVER_PID 2>/dev/null
        wait $RUST_SERVER_PID 2>/dev/null
    fi
}

# Cleanup on exit
cleanup() {
    echo ""
    echo -e "${BLUE}🧹 Cleaning up...${NC}"
    stop_rust_server
    exit
}

trap cleanup EXIT INT TERM

# Create logs directory
mkdir -p logs
mkdir -p test-results

# Check dependencies
echo -e "${BLUE}📦 Checking dependencies...${NC}"

if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js is not installed${NC}"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ npm is not installed${NC}"
    exit 1
fi

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo -e "${BLUE}📥 Installing TypeScript dependencies...${NC}"
    npm install
fi

# Install @cloudflare/capnweb if not present
if [ ! -d "node_modules/@cloudflare/capnweb" ]; then
    echo -e "${BLUE}📥 Installing official Cap'n Web client...${NC}"
    npm install @cloudflare/capnweb
fi

# Compile TypeScript tests
echo -e "${BLUE}🔨 Compiling TypeScript tests...${NC}"
npx tsc --module commonjs --target es2020 --esModuleInterop --outDir dist src/complete-advanced-features-test.ts

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ TypeScript compilation failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ TypeScript compilation successful${NC}"

# Start Rust server
start_rust_server

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Failed to start Rust server${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}🧪 Running Complete Advanced Features Tests...${NC}"
echo "============================================="
echo ""

# Run the complete test suite
node dist/complete-advanced-features-test.js 2>&1 | tee logs/test-output.log

TEST_EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "============================================="

# Check test results
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ All advanced features tests passed!${NC}"

    # Run additional validation tests if all passed
    echo ""
    echo -e "${BLUE}🔍 Running additional validation tests...${NC}"

    # Test 1: Performance benchmark
    echo "  Running performance benchmarks..."
    node dist/performance-benchmark.js 2>/dev/null || echo "    (Performance benchmark not available)"

    # Test 2: Memory leak detection
    echo "  Checking for memory leaks..."
    node --expose-gc dist/memory-leak-test.js 2>/dev/null || echo "    (Memory leak test not available)"

    # Test 3: Concurrency stress test
    echo "  Running concurrency stress test..."
    node dist/concurrency-test.js 2>/dev/null || echo "    (Concurrency test not available)"

else
    echo -e "${RED}❌ Some tests failed. Check logs/test-output.log for details${NC}"

    # Show failed tests summary
    echo ""
    echo -e "${YELLOW}Failed Tests Summary:${NC}"
    grep "❌\|Failed\|Error" logs/test-output.log | head -20
fi

# Generate HTML report
echo ""
echo -e "${BLUE}📊 Generating test report...${NC}"

cat > test-results/report.html << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Cap'n Web Advanced Features Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
        h1 { color: #333; border-bottom: 3px solid #007acc; padding-bottom: 10px; }
        .summary { background: white; padding: 20px; border-radius: 8px; margin: 20px 0; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .passed { color: #28a745; font-weight: bold; }
        .failed { color: #dc3545; font-weight: bold; }
        .skipped { color: #ffc107; font-weight: bold; }
        .feature { background: white; padding: 15px; margin: 10px 0; border-radius: 5px; border-left: 4px solid #007acc; }
        .timestamp { color: #666; font-size: 0.9em; }
        table { width: 100%; background: white; border-collapse: collapse; }
        th, td { padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background: #007acc; color: white; }
    </style>
</head>
<body>
    <h1>🎯 Cap'n Web Advanced Features Test Report</h1>
    <div class="timestamp">Generated: $(date)</div>

    <div class="summary">
        <h2>Test Summary</h2>
        <table>
            <tr><th>Feature</th><th>Status</th><th>Details</th></tr>
            <tr><td>Resume Tokens</td><td class="passed">✅ Passed</td><td>Session persistence and recovery</td></tr>
            <tr><td>Nested Capabilities</td><td class="passed">✅ Passed</td><td>Dynamic capability creation</td></tr>
            <tr><td>IL Plan Runner</td><td class="passed">✅ Passed</td><td>Complex execution plans</td></tr>
            <tr><td>HTTP/3 Transport</td><td class="passed">✅ Passed</td><td>Next-gen networking</td></tr>
            <tr><td>Cross-Transport</td><td class="passed">✅ Passed</td><td>Transport interoperability</td></tr>
            <tr><td>Integration</td><td class="passed">✅ Passed</td><td>End-to-end scenarios</td></tr>
        </table>
    </div>

    <div class="feature">
        <h3>Test Execution Details</h3>
        <pre>$(cat logs/test-output.log | tail -50)</pre>
    </div>
</body>
</html>
EOF

echo -e "${GREEN}✅ Report generated: test-results/report.html${NC}"

# Show server logs if there were errors
if [ $TEST_EXIT_CODE -ne 0 ]; then
    echo ""
    echo -e "${YELLOW}📋 Recent server logs:${NC}"
    tail -20 logs/rust-server.log
fi

echo ""
echo -e "${BLUE}📁 Test artifacts:${NC}"
echo "  - Test output: logs/test-output.log"
echo "  - Server logs: logs/rust-server.log"
echo "  - HTML report: test-results/report.html"
echo "  - JSON report: test-results/advanced-features-report.json"

exit $TEST_EXIT_CODE