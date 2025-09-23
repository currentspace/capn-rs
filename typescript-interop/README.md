# Cap'n Web Rust ↔ TypeScript Interoperability Tests

This directory contains comprehensive interoperability tests that validate the Rust implementation of Cap'n Web against the official TypeScript client.

## Overview

The test suite ensures that:
- The Rust server can handle requests from the TypeScript client
- Message serialization/deserialization is compatible
- All Cap'n Web protocol features work correctly between implementations
- Error handling is consistent
- Performance meets expected standards

## Prerequisites

- Node.js 18+
- npm or yarn
- Rust toolchain (for building the server)

## Quick Start

```bash
# Set up the testing environment
./setup.sh

# Run all interoperability tests
npm test
```

## Test Categories

### 🧪 Core Protocol Tests
- **Basic HTTP Batch Call**: Validates fundamental RPC functionality
- **Message Serialization**: Tests various data types and structures
- **Error Handling**: Validates error propagation and codes
- **Capability Management**: Tests capability registration and calls
- **Complex Data Structures**: Tests nested objects and arrays
- **Batch Operations**: Tests multiple concurrent requests

### 🚀 Transport Tests
- **HTTP Batch Transport**: Tests the primary transport mechanism
- **WebSocket Transport**: Tests real-time bidirectional communication
- **WebTransport (H3)**: Tests modern HTTP/3 transport (when available)

### 🔄 Advanced Protocol Tests
- **Promise Pipelining**: Tests chained remote calls optimization
- **Capability Lifecycle**: Tests creation, disposal, and reference counting
- **Three-Party Handoffs**: Tests capability passing between sessions
- **E-Order Guarantees**: Tests execution order consistency

## Test Architecture

```
typescript-interop/
├── src/
│   ├── interop-tests.ts     # Main test runner and test implementations
│   └── test-runner.ts       # CLI test runner
├── package.json             # Dependencies and scripts
├── tsconfig.json           # TypeScript configuration
└── setup.sh               # Environment setup script
```

## Running Tests

### All Tests
```bash
npm test
```

### Individual Test Components
```bash
# Build TypeScript
npm run build

# Run specific test scenarios
node dist/interop-tests.js
```

### Development Mode
```bash
# Watch mode for development
npm run test:watch
```

## Test Results Format

The test runner provides detailed output including:
- ✅ **Passed Tests**: Individual test results with timing
- ❌ **Failed Tests**: Error details and stack traces
- 📊 **Test Suites**: Grouped results by category
- 📈 **Overall Results**: Success rate and total duration
- 🎯 **Coverage Metrics**: Protocol feature coverage

## Expected Output

```
🧪 Cap'n Web Rust ↔ TypeScript Interoperability Test Runner
================================================================

🚀 Starting Rust Cap'n Web server...
✅ Server is responding
🧪 Running test: Basic HTTP Batch Call
✅ Basic HTTP Batch Call - PASSED (45ms)
...

📊 Core Protocol Tests
   ✅ Passed: 6
   ❌ Failed: 0
   ⏱️  Duration: 234ms

📈 OVERALL RESULTS
   ✅ Total Passed: 8
   ❌ Total Failed: 0
   ⏱️  Total Duration: 456ms
   🎯 Success Rate: 100%

🎉 ALL TESTS PASSED! Cap'n Web Rust implementation is fully compatible with TypeScript client!
```

## Test Configuration

### Server Configuration
The tests automatically start a Rust server with:
- **Host**: localhost
- **Port**: 8080
- **Endpoints**:
  - HTTP Batch: `/rpc/batch`
  - Health Check: `/health`

### Capabilities
The test server provides:
1. **Calculator** (ID: 1): `add`, `multiply`, `divide`, `echo`
2. **UserManager** (ID: 2): `getUser`, `getName`, `getAge`, `getValue`

## Troubleshooting

### Common Issues

**Server Startup Timeout**
```bash
# Check if port 8080 is available
lsof -i :8080

# Kill any processes using the port
kill -9 $(lsof -t -i:8080)
```

**Node.js Version Issues**
```bash
# Check Node.js version
node -v

# Install Node.js 18+ from https://nodejs.org
```

**TypeScript Compilation Errors**
```bash
# Clean and rebuild
npm run clean
npm run build
```

### Debug Mode

For detailed debug output:
```bash
# Enable debug logging
DEBUG=capnweb* npm test
```

## Integration with CI/CD

The test suite is designed to work in CI/CD environments:

```yaml
# GitHub Actions example
- name: Setup Node.js
  uses: actions/setup-node@v3
  with:
    node-version: '18'

- name: Setup TypeScript Interop Tests
  run: |
    cd typescript-interop
    ./setup.sh

- name: Run Interoperability Tests
  run: |
    cd typescript-interop
    npm test
```

## Contributing

When adding new tests:

1. **Follow the existing pattern** in `interop-tests.ts`
2. **Add comprehensive error handling** for edge cases
3. **Include timing measurements** for performance validation
4. **Update this README** with new test descriptions
5. **Ensure tests are deterministic** and can run in any order

## Performance Benchmarks

Expected performance targets:
- **Basic RPC Call**: < 50ms
- **Batch Operations**: < 100ms for 5 concurrent calls
- **Complex Data**: < 100ms for nested objects
- **Server Startup**: < 5 seconds

## Protocol Compliance

This test suite validates compliance with:
- **Cap'n Web Protocol Specification**
- **JSON Wire Format** compatibility
- **HTTP Transport** requirements
- **WebSocket Transport** requirements
- **Error Code** standardization

## Related Documentation

- [Cap'n Web Protocol Specification](https://github.com/cloudflare/capnweb/blob/main/protocol.md)
- [Comprehensive Test Plan](../COMPREHENSIVE_TEST_PLAN.md)
- [Rust Implementation Documentation](../README.md)