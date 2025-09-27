#!/bin/bash

# Tiered Cap'n Web Test Runner
# Progressively tests implementation from basic protocol to complex applications

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
TEST_PORT=${1:-9005}
SERVER_EXAMPLE=${2:-stateful_server}
SERVER_TIMEOUT=15
TEST_TIMEOUT=120

# Global state
SERVER_PID=""
TIER1_PASSED=false
TIER2_PASSED=false
TIER3_PASSED=false

echo -e "${CYAN}🏗️  Tiered Cap'n Web Test Framework${NC}"
echo -e "${CYAN}====================================${NC}"
echo -e "Port: ${TEST_PORT}"
echo -e "Server: ${SERVER_EXAMPLE}"
echo -e "Timeout: ${TEST_TIMEOUT}s per test"
echo

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up test environment...${NC}"
    if [ ! -z "$SERVER_PID" ]; then
        echo "Stopping server process $SERVER_PID"
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi

    # Kill any remaining processes on the test port
    lsof -ti:$TEST_PORT | xargs kill -9 2>/dev/null || true

    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Set up trap for cleanup
trap cleanup EXIT INT TERM

# Function to start server
start_server() {
    echo -e "${BLUE}🚀 Starting $SERVER_EXAMPLE on port $TEST_PORT...${NC}"

    # Build the server first
    cargo build --example $SERVER_EXAMPLE -p capnweb-server --quiet
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Server build failed${NC}"
        exit 1
    fi

    # Start server in background
    PORT=$TEST_PORT cargo run --example $SERVER_EXAMPLE -p capnweb-server > server.log 2>&1 &
    SERVER_PID=$!

    # Wait for server to be ready
    echo -e "${YELLOW}⏳ Waiting for server to start...${NC}"
    for i in $(seq 1 $SERVER_TIMEOUT); do
        if curl -s http://127.0.0.1:$TEST_PORT/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Server is ready on port $TEST_PORT${NC}"
            return 0
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

    echo -e "${RED}❌ Server failed to start within $SERVER_TIMEOUT seconds${NC}"
    echo -e "${RED}Server logs:${NC}"
    cat server.log
    exit 1
}

# Function to stop server
stop_server() {
    if [ ! -z "$SERVER_PID" ]; then
        echo -e "${YELLOW}🛑 Stopping server...${NC}"
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
        SERVER_PID=""
    fi
}

# Function to run a test tier
run_tier() {
    local tier_num=$1
    local tier_name="$2"
    local test_script="$3"
    local required_for_next="$4"

    echo -e "\n${PURPLE}📊 TIER $tier_num: $tier_name${NC}"
    echo -e "${PURPLE}==================================================${NC}"

    # Build TypeScript if needed
    if [ ! -f "dist/$test_script" ]; then
        echo -e "${BLUE}🔨 Building TypeScript tests...${NC}"
        cd typescript-interop
        npm run build --silent
        cd ..
    fi

    # Run the tier test
    echo -e "${BLUE}🧪 Running $tier_name tests...${NC}"
    cd typescript-interop

    local exit_code=0
    timeout $TEST_TIMEOUT node "dist/$test_script" $TEST_PORT || exit_code=$?

    cd ..

    # Interpret results
    case $exit_code in
        0)
            echo -e "${GREEN}🎉 TIER $tier_num PASSED: $tier_name complete!${NC}"
            return 0
            ;;
        1)
            echo -e "${YELLOW}⚠️  TIER $tier_num PARTIAL: Some issues remain${NC}"
            if [ "$required_for_next" = "true" ]; then
                echo -e "${YELLOW}🚧 Stopping here - this tier required for next tier${NC}"
                return 1
            else
                echo -e "${YELLOW}➡️  Continuing to next tier despite partial results${NC}"
                return 0
            fi
            ;;
        2)
            echo -e "${RED}💥 TIER $tier_num FAILED: Fundamental issues${NC}"
            if [ "$required_for_next" = "true" ]; then
                echo -e "${RED}🚨 Stopping here - this tier required for next tier${NC}"
                return 2
            else
                echo -e "${YELLOW}➡️  Continuing to next tier for diagnostic purposes${NC}"
                return 0
            fi
            ;;
        124)
            echo -e "${RED}⏰ TIER $tier_num TIMEOUT: Tests took longer than ${TEST_TIMEOUT}s${NC}"
            return 2
            ;;
        *)
            echo -e "${RED}💥 TIER $tier_num ERROR: Unexpected exit code $exit_code${NC}"
            return 2
            ;;
    esac
}

# Function to show final summary
show_summary() {
    echo -e "\n${CYAN}📈 FINAL SUMMARY${NC}"
    echo -e "${CYAN}=================${NC}"

    local total_tiers=3
    local passed_tiers=0

    if [ "$TIER1_PASSED" = "true" ]; then
        echo -e "${GREEN}✅ Tier 1: Basic Protocol Compliance - PASSED${NC}"
        passed_tiers=$((passed_tiers + 1))
    else
        echo -e "${RED}❌ Tier 1: Basic Protocol Compliance - FAILED${NC}"
    fi

    if [ "$TIER2_PASSED" = "true" ]; then
        echo -e "${GREEN}✅ Tier 2: Stateful Session Management - PASSED${NC}"
        passed_tiers=$((passed_tiers + 1))
    else
        echo -e "${RED}❌ Tier 2: Stateful Session Management - FAILED${NC}"
    fi

    if [ "$TIER3_PASSED" = "true" ]; then
        echo -e "${GREEN}✅ Tier 3: Complex Application Logic - PASSED${NC}"
        passed_tiers=$((passed_tiers + 1))
    else
        echo -e "${RED}❌ Tier 3: Complex Application Logic - FAILED${NC}"
    fi

    echo
    echo -e "${BLUE}📊 Progress: $passed_tiers/$total_tiers tiers passed${NC}"

    if [ $passed_tiers -eq $total_tiers ]; then
        echo -e "${GREEN}🏆 COMPLETE SUCCESS: Full Cap'n Web implementation!${NC}"
        echo -e "${GREEN}🚀 Ready for production use${NC}"
        return 0
    elif [ $passed_tiers -eq 2 ]; then
        echo -e "${YELLOW}🥈 STRONG IMPLEMENTATION: Core features working${NC}"
        echo -e "${YELLOW}💡 Consider implementing advanced features${NC}"
        return 1
    elif [ $passed_tiers -eq 1 ]; then
        echo -e "${YELLOW}🥉 BASIC IMPLEMENTATION: Protocol working${NC}"
        echo -e "${YELLOW}🔧 Needs session management improvements${NC}"
        return 2
    else
        echo -e "${RED}💥 IMPLEMENTATION INCOMPLETE: Protocol issues${NC}"
        echo -e "${RED}🚨 Requires fundamental fixes${NC}"
        return 3
    fi
}

# Main execution flow
main() {
    # Start the server
    start_server

    echo -e "\n${CYAN}🧪 Starting Tiered Testing Sequence${NC}"
    echo -e "${CYAN}====================================${NC}"

    # TIER 1: Basic Protocol Compliance
    if run_tier 1 "Basic Protocol Compliance" "tier1-protocol-compliance.js" "true"; then
        TIER1_PASSED=true
    else
        echo -e "${RED}🛑 Stopping: Tier 1 is required for all subsequent tests${NC}"
        show_summary
        exit 1
    fi

    # TIER 2: Stateful Session Management
    if run_tier 2 "Stateful Session Management" "tier2-stateful-sessions.js" "false"; then
        TIER2_PASSED=true
    fi

    # TIER 3: Complex Application Logic
    if run_tier 3 "Complex Application Logic" "tier3-complex-applications.js" "false"; then
        TIER3_PASSED=true
    fi

    # Stop server for clean summary
    stop_server

    # Show final results
    show_summary
    return $?
}

# Run main function
main