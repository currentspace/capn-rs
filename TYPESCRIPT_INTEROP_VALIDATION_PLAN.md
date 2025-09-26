# Cap'n Web Rust ↔ TypeScript Interoperability Validation Plan

This comprehensive plan validates that the Rust implementation of the Cap'n Web protocol is fully compatible with the official TypeScript client, ensuring 100% protocol compliance and feature parity.

## Overview

The validation process tests the complete Cap'n Web protocol implementation through a tiered testing approach:

- **Tier 1**: Core protocol compliance (HTTP batch transport)
- **Tier 2**: Stateful sessions and WebSocket support
- **Tier 3**: Advanced features (capability composition, stress testing, complex applications)

## Prerequisites

### System Requirements
- **Rust**: 1.75+ with cargo
- **Node.js**: 18+ with npm/pnpm
- **System**: macOS/Linux (tested on Darwin)
- **Ports**: 8080 (default), 9000-9005 (configurable)

### Repository Structure
```
capn-rs/
├── capnweb-server/               # Rust server implementation
│   └── examples/
│       ├── typescript_interop_server.rs     # Main interop server
│       ├── basic_server.rs                  # Basic functionality
│       ├── advanced_stateful_server.rs      # Advanced features
│       └── advanced_capability_marshaling.rs # Capability composition
├── typescript-interop/          # TypeScript test suite
│   ├── src/                     # Test implementations
│   ├── package.json             # Dependencies and test scripts
│   ├── run-all-tests.sh         # Sequential test runner
│   └── run-complete-tests.sh    # Complete feature validation
└── capnweb-core/                # Protocol implementation
```

## Phase 1: Environment Setup and Build

### Step 1: Clean Environment
```bash
# Navigate to project root
cd /path/to/capn-rs

# Clean any running processes
pkill -f "cargo run" 2>/dev/null || true
pkill -f "node" 2>/dev/null || true

# Verify no processes are using test ports
lsof -ti:8080,9000,9001,9005 | xargs kill -9 2>/dev/null || true
```

### Step 2: Build Rust Components
```bash
# Build all Rust crates
cargo build --workspace

# Build optimized versions for performance testing
cargo build --workspace --release

# Verify core examples compile
cargo check -p capnweb-server --examples
```

### Step 3: Setup TypeScript Test Environment
```bash
# Navigate to TypeScript test directory
cd typescript-interop/

# Install dependencies
npm install
# or
pnpm install

# Build TypeScript test suite
npm run build

# Verify capnweb dependency is available
npm run build:capnweb 2>/dev/null || echo "Will build during first test run"
```

## Phase 2: Core Protocol Validation (Tier 1)

### Step 1: Start Primary Rust Server
```bash
# From project root
cargo run --example typescript_interop_server -p capnweb-server 2>&1 &
RUST_SERVER_PID=$!
echo "Server PID: $RUST_SERVER_PID"

# Wait for server startup
sleep 5

# Verify server is responding
curl -f http://localhost:8080/health || echo "Server not ready"
```

**Expected Server Capabilities:**
- **Calculator** (CapId: 1): `add`, `subtract`, `multiply`, `divide`, `echo`, `store`, `recall`
- **UserManager** (CapId: 2): `getUser`, `getName`, `getAge`, `getValue`, `createUser`
- **HTTP Batch Transport**: `/rpc/batch` endpoint
- **Health Check**: `/health` endpoint

### Step 2: Tier 1 Protocol Compliance Tests
```bash
# From typescript-interop directory
cd typescript-interop/

# Run protocol compliance validation
npm run build && node dist/tier1-protocol-compliance.js

# Expected validations:
# ✅ Basic HTTP batch request/response
# ✅ JSON wire format compliance
# ✅ Error code standardization (bad_request, not_found, cap_revoked, etc.)
# ✅ Capability reference handling
# ✅ Message serialization/deserialization
```

**Success Criteria:**
- All basic RPC calls complete successfully
- Error responses match Cap'n Web specification
- JSON wire format matches TypeScript implementation
- Response time < 50ms per call

## Phase 3: HTTP Batch Transport Validation (Tier 2)

### Step 1: HTTP Batch Corrected Tests
```bash
# Run corrected HTTP batch protocol tests
node dist/tier2-http-batch-corrected.js

# Tests validate:
# ✅ Batch request processing
# ✅ Session lifecycle (sessions end after batch)
# ✅ Concurrent request handling
# ✅ Complex data structure marshaling
# ✅ Promise.all() compatibility patterns
```

### Step 2: Stateful Session Tests
```bash
# Run stateful session validation
node dist/tier2-stateful-sessions.js

# Validates:
# ✅ Capability state persistence within session
# ✅ Variable storage and retrieval
# ✅ Session isolation between clients
# ✅ Proper session cleanup
```

**Success Criteria:**
- Batch operations complete in < 100ms for 5 concurrent calls
- State management works correctly within session boundaries
- No memory leaks or resource cleanup issues

## Phase 4: WebSocket Transport Validation (Tier 2)

### Step 1: Basic WebSocket Tests
```bash
# Run WebSocket transport tests
node dist/tier2-websocket-tests.js

# Validates:
# ✅ WebSocket connection establishment
# ✅ Persistent session capabilities
# ✅ Real-time bidirectional communication
# ✅ WebSocket-specific protocol adaptations
```

**Note**: WebSocket support may be optional - mark as non-critical if not implemented.

## Phase 5: Advanced Feature Validation (Tier 3)

### Step 1: Capability Composition Tests
```bash
# Run advanced capability composition tests
node dist/tier3-capability-composition.js

# Validates:
# ✅ Three-party capability handoffs
# ✅ Capability lifecycle management
# ✅ Reference counting and disposal
# ✅ Capability chaining and composition
# ✅ Promise pipelining optimizations
```

### Step 2: Complex Application Tests
```bash
# Run complex application scenario tests
node dist/tier3-complex-applications.js

# Validates:
# ✅ Multi-step workflows
# ✅ Nested capability interactions
# ✅ Complex data structure handling
# ✅ Real-world usage patterns
```

### Step 3: Extreme Stress Testing
```bash
# Run extreme stress tests
node dist/tier3-extreme-stress.js

# Validates:
# ✅ High-volume concurrent requests (100+ simultaneous)
# ✅ Large data payload handling
# ✅ Resource exhaustion scenarios
# ✅ Error recovery under load
# ✅ Memory usage stability
```

### Step 4: WebSocket Advanced Features
```bash
# Run advanced WebSocket features (if implemented)
node dist/tier3-websocket-advanced.js

# Validates:
# ✅ Advanced WebSocket-specific patterns
# ✅ Connection recovery mechanisms
# ✅ WebSocket capability streaming
```

**Success Criteria:**
- All capability compositions work correctly
- System remains stable under extreme load
- Memory usage remains bounded
- Response times degrade gracefully under load

## Phase 6: Comprehensive Validation Suite

### Step 1: Run Complete Test Suite
```bash
# Execute all tests in sequence
./run-all-tests.sh

# Or run comprehensive test runner
npm run test:comprehensive

# Or run complete advanced features test
./run-complete-tests.sh
```

### Step 2: Cross-Transport Interoperability
```bash
# Test interaction between different transports
node dist/cross-transport-interop.js

# Validates:
# ✅ HTTP batch → WebSocket capability passing
# ✅ Consistent behavior across transports
# ✅ Transport-agnostic capability references
```

## Phase 7: Performance and Compliance Validation

### Step 1: Performance Benchmarks
```bash
# Run performance validation
npm run test:complete -- --benchmark

# Validate against targets:
# ✅ Basic RPC Call: < 50ms
# ✅ Batch Operations: < 100ms for 5 concurrent calls
# ✅ Complex Data: < 100ms for nested objects
# ✅ Server Startup: < 5 seconds
```

### Step 2: Protocol Compliance Report
```bash
# Generate compliance report
npm run test:comprehensive -- --report

# Validates compliance with:
# ✅ Cap'n Web Protocol Specification
# ✅ JSON Wire Format compatibility
# ✅ HTTP Transport requirements
# ✅ WebSocket Transport requirements (if implemented)
# ✅ Error Code standardization
```

## Validation Matrix

| Test Category | HTTP Batch | WebSocket | Expected Result |
|---------------|------------|-----------|-----------------|
| Basic RPC | ✅ Required | ⚠️ Optional | PASS |
| Error Handling | ✅ Required | ⚠️ Optional | PASS |
| Capability Management | ✅ Required | ⚠️ Optional | PASS |
| Promise Pipelining | ✅ Required | ⚠️ Optional | PASS |
| Batch Operations | ✅ Required | N/A | PASS |
| Persistent Sessions | N/A | ✅ Required | PASS |
| Capability Composition | ✅ Required | ⚠️ Optional | PASS |
| Stress Testing | ✅ Required | ⚠️ Optional | PASS |

## Troubleshooting Guide

### Server Startup Issues
```bash
# Check port availability
lsof -i :8080

# Check server logs
cargo run --example typescript_interop_server -p capnweb-server 2>&1 | tee server.log

# Verify server health
curl -v http://localhost:8080/health
```

### TypeScript Build Issues
```bash
# Clean build
npm run clean && npm run build

# Check TypeScript compilation
npx tsc --noEmit

# Verify dependencies
npm ls capnweb
```

### Test Failures
```bash
# Run individual test with debug output
DEBUG=capnweb* node dist/tier1-protocol-compliance.js

# Check server responses manually
curl -X POST http://localhost:8080/rpc/batch \
  -H "Content-Type: application/json" \
  -d '[{"call":{"cap":1,"member":"add","args":[2,3]}}]'
```

### Performance Issues
```bash
# Profile server performance
cargo run --example typescript_interop_server -p capnweb-server --release

# Monitor system resources
top -p $(pgrep -f typescript_interop_server)

# Check network latency
ping localhost
```

## Success Criteria Summary

### Critical Requirements (Must Pass)
1. **All Tier 1 tests pass** - Core protocol compliance
2. **HTTP batch transport works** - Primary transport mechanism
3. **Basic capabilities function** - Calculator and UserManager
4. **Error handling is correct** - All error codes match spec
5. **Performance targets met** - Response times within limits

### Optional Features (May Pass)
1. **WebSocket transport** - Real-time communication
2. **Advanced capabilities** - Complex compositions
3. **Stress test performance** - High-load scenarios

### Final Validation Report
```bash
# Generate final report
npm run test:comprehensive -- --final-report > INTEROP_VALIDATION_REPORT.txt

# Expected output format:
# 🎯 Cap'n Web Rust ↔ TypeScript Interoperability Validation Report
# ================================================================
# 📊 Test Results Summary
# - Tier 1 Tests: ✅ 12/12 PASSED
# - Tier 2 Tests: ✅ 8/8 PASSED
# - Tier 3 Tests: ✅ 15/18 PASSED (WebSocket optional features not implemented)
#
# 🏆 OVERALL RESULT: PROTOCOL COMPLIANT
# 🎉 Rust implementation is fully compatible with TypeScript client
```

## Post-Validation Actions

### If All Tests Pass
1. **Document capabilities** - Update README with confirmed feature support
2. **Performance baseline** - Record benchmark results for regression testing
3. **CI/CD integration** - Add validation to automated testing pipeline
4. **Release preparation** - Tag version as TypeScript-compatible

### If Tests Fail
1. **Isolate failures** - Run individual test categories to identify issues
2. **Debug protocol** - Compare wire format with TypeScript implementation
3. **Fix implementations** - Update Rust server code to match specification
4. **Retest incrementally** - Validate fixes before full rerun

## Automation Integration

### GitHub Actions Workflow
```yaml
name: TypeScript Interop Validation
on: [push, pull_request]
jobs:
  interop-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Setup and Run Validation
        run: |
          cd typescript-interop
          ./setup.sh
          npm test
      - name: Generate Report
        run: |
          npm run test:comprehensive -- --report > validation-report.txt
      - uses: actions/upload-artifact@v3
        with:
          name: validation-report
          path: typescript-interop/validation-report.txt
```

## Conclusion

This validation plan ensures comprehensive testing of the Rust Cap'n Web implementation against the TypeScript reference client. Following this plan will confirm that the Rust server provides complete protocol compatibility and can serve as a drop-in replacement for TypeScript-based Cap'n Web servers.

The tiered approach allows for progressive validation, with critical core features tested first and optional advanced features tested later. This enables incremental development and deployment while maintaining confidence in core functionality.