#!/bin/bash
#
# Build script for Cap'n Web Rust server
# Builds the unified test server with all features

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔨 Building Cap'n Web Rust Server..."
echo "===================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Not in the root directory of capn-rs${NC}"
    echo "Please run this script from the capn-rs root directory"
    exit 1
fi

# Clean previous builds
echo -e "${YELLOW}🧹 Cleaning previous builds...${NC}"
cargo clean 2>/dev/null || true

# Build the workspace in release mode
echo -e "${YELLOW}📦 Building workspace in release mode...${NC}"
cargo build --release --workspace

# Build the unified test server specifically
echo -e "${YELLOW}🎯 Building unified test server...${NC}"
cargo build --release --example unified_test_server -p capnweb-server

# Check if build succeeded
if [ -f "target/release/examples/unified_test_server" ]; then
    echo -e "${GREEN}✅ Build successful!${NC}"
    echo ""
    echo "📍 Server binary location:"
    echo "   target/release/examples/unified_test_server"
    echo ""
    echo "📝 Next steps:"
    echo "   1. Run the server: ./run-server.sh"
    echo "   2. Run tests: ./run-tests.sh"
else
    echo -e "${RED}❌ Build failed!${NC}"
    echo "Please check the error messages above"
    exit 1
fi