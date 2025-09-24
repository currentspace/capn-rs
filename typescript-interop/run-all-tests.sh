#!/bin/bash

# Run all TypeScript tests against the Rust server
PORT=9005

echo "🧪 Running Cap'n Web TypeScript Tests against Rust Server"
echo "=========================================================="
echo "Server: http://localhost:$PORT"
echo ""

# Track test results
PASSED=0
FAILED=0
TOTAL=0

# Function to run a test
run_test() {
    local test_name=$1
    local test_script=$2
    echo "🔧 Test: $test_name"
    echo "----------------------------------------"

    if node dist/$test_script.js $PORT 2>&1; then
        echo "✅ PASSED"
        ((PASSED++))
    else
        echo "❌ FAILED"
        ((FAILED++))
    fi
    ((TOTAL++))
    echo ""
}

# Run all tests that use capnweb
run_test "Official Client Test" "official-client-test"
run_test "Tier 1: Protocol Compliance" "tier1-protocol-compliance"
run_test "Promise Pipelining" "promise-pipelining-test"
run_test "Advanced Server Test" "advanced-server-test"
run_test "Comprehensive Stateful Test" "comprehensive-stateful-test"
run_test "Tier 2: Stateful Sessions" "tier2-stateful-sessions"
run_test "Tier 2: WebSocket" "tier2-websocket-tests"
run_test "Tier 3: Complex Applications" "tier3-complex-applications"
run_test "Tier 3: WebSocket Advanced" "tier3-websocket-advanced"
run_test "Tier 3: Capability Composition" "tier3-capability-composition"
run_test "Tier 3: Extreme Stress" "tier3-extreme-stress"
run_test "Cross-Transport Interop" "cross-transport-interop"

echo "=========================================================="
echo "📊 Test Results Summary"
echo "=========================================================="
echo "Total Tests: $TOTAL"
echo "✅ Passed: $PASSED"
echo "❌ Failed: $FAILED"
if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All tests passed!"
    exit 0
else
    echo ""
    echo "⚠️  Some tests failed"
    exit 1
fi