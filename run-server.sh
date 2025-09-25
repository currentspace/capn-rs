#!/bin/bash
#
# Run script for Cap'n Web Rust server
# Starts the unified test server with configurable options

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
DEFAULT_PORT=9000
DEFAULT_HOST="127.0.0.1"
DEFAULT_LOG_LEVEL="info,capnweb_server=debug,capnweb_core=debug"

# Parse command line arguments
PORT=${1:-$DEFAULT_PORT}
HOST=${2:-$DEFAULT_HOST}
LOG_LEVEL=${3:-$DEFAULT_LOG_LEVEL}

echo "🚀 Starting Cap'n Web Rust Server"
echo "================================="
echo ""

# Check if the binary exists
if [ ! -f "target/release/examples/unified_test_server" ]; then
    echo -e "${YELLOW}⚠️  Server binary not found. Building...${NC}"
    ./build-server.sh
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
fi

# Kill any existing servers
echo -e "${YELLOW}🔍 Checking for existing servers...${NC}"
pkill -f "unified_test_server" 2>/dev/null && echo "   Stopped existing server" || echo "   No existing server found"
pkill -f "basic_server" 2>/dev/null || true
pkill -f "cargo run" 2>/dev/null || true

# Wait a moment for ports to be released
sleep 1

# Start the server
echo ""
echo -e "${GREEN}📡 Starting server with:${NC}"
echo "   Host: $HOST"
echo "   Port: $PORT"
echo "   Log Level: $LOG_LEVEL"
echo ""
echo "📍 Endpoints:"
echo "   HTTP Batch: http://$HOST:$PORT/rpc/batch"
echo "   Health: http://$HOST:$PORT/health"
echo ""

# Export environment variables
export PORT=$PORT
export HOST=$HOST
export RUST_LOG=$LOG_LEVEL

# Run the server
echo -e "${GREEN}✨ Server starting...${NC}"
echo "================================="
echo ""

# Run in foreground so we can see logs
./target/release/examples/unified_test_server