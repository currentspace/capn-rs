# Cap'n Web TypeScript ↔ Rust Interoperability Tests

Comprehensive interoperability tests between TypeScript and Rust Cap'n Web implementations, verifying full protocol compatibility and cross-language communication.

## Overview

This test suite verifies that:
- TypeScript clients can communicate with Rust servers
- Rust clients can communicate with TypeScript servers
- Message formats are identical between implementations
- Error handling behaves consistently
- Performance characteristics are comparable
- Complex data structures serialize correctly

## Quick Start

### Prerequisites

- **Node.js 24+** (for TypeScript implementation)
- **Rust 1.75+** (for Rust implementation)
- **pnpm** (package manager)

### Setup

```bash
# Install dependencies
pnpm install

# Build TypeScript code
pnpm build

# Verify setup
pnpm typecheck
pnpm lint
```

## Running Tests

### Option 1: Full Interoperability Test Suite

```bash
# Start Rust server (in another terminal)
cd .. && cargo run --example calculator_server

# Run all interop tests
pnpm test:all
```

### Option 2: Individual Test Scenarios

**TypeScript Client → Rust Server Tests:**
```bash
# Terminal 1: Start Rust server
cd .. && cargo run --example calculator_server

# Terminal 2: Run TypeScript client tests
pnpm test:client
```

**TypeScript Server ← Rust Client Tests:**
```bash
# Terminal 1: Start TypeScript server
pnpm start:ts-server

# Terminal 2: Run Rust client (simulated)
cd .. && cargo run --example calculator_client
```

### Option 3: Interactive Testing

**Start TypeScript Server:**
```bash
pnpm start:ts-server
# Server runs on ws://localhost:8081/ws
# Connect with any Cap'n Web client
```

**Run TypeScript Client:**
```bash
# Ensure Rust server is running first
node dist/index.js --client-only
```

## Test Scenarios

### Basic Protocol Compatibility

- ✅ **Message Serialization** - JSON format compatibility
- ✅ **Capability Calls** - Method invocation across languages
- ✅ **Error Handling** - Consistent error propagation
- ✅ **Connection Management** - WebSocket lifecycle

### Advanced Features

- ✅ **Calculator Operations** - All arithmetic functions
- ✅ **User Management** - CRUD operations
- ✅ **Complex Data Types** - Objects, arrays, nested structures
- ✅ **Concurrent Operations** - Multiple simultaneous calls
- ✅ **Performance Testing** - High-volume operation handling
- ✅ **Edge Cases** - Error conditions, invalid inputs

### Capability Tests

**Calculator (ID: 1, 2):**
- `add(a, b)` - Addition
- `subtract(a, b)` - Subtraction
- `multiply(a, b)` - Multiplication
- `divide(a, b)` - Division (with zero-check)
- `power(base, exp)` - Exponentiation
- `sqrt(n)` - Square root (positive numbers only)
- `factorial(n)` - Factorial (0-20 range)

**User Manager (ID: 100):**
- `getUser(id)` - Retrieve user by ID
- `createUser(userData)` - Create new user

## Test Results Example

```
🌟 Cap'n Web TypeScript ↔ Rust Interoperability Test Suite
======================================================================

🚀 PHASE 1: TypeScript Client → Rust Server Tests
--------------------------------------------------
✅ Rust server is available, proceeding with client tests...

📋 Running: Connection Tests
✅ PASSED: testConnection (123ms)
✅ PASSED: testConnectionTimeout (456ms)

📋 Running: Basic Capability Tests
✅ PASSED: testBasicCalculatorOperations (89ms)
✅ PASSED: testCalculatorErrorHandling (67ms)
✅ PASSED: testMultipleSequentialCalls (234ms)
✅ PASSED: testConcurrentCalls (156ms)

🏁 FINAL INTEROPERABILITY REPORT
======================================================================
📊 Overall Success Rate: 100.0%

🎉 INTEROPERABILITY VERIFIED!
✅ TypeScript and Rust Cap'n Web implementations are fully compatible!
```

## Architecture

### TypeScript Implementation

```
src/
├── capnweb/
│   ├── types.ts              # Protocol type definitions
│   ├── client.ts             # Cap'n Web client implementation
│   ├── server.ts             # Cap'n Web server implementation
│   └── websocket-transport.ts # WebSocket transport layer
├── tests/
│   ├── test-framework.ts     # Test utilities and assertions
│   ├── client-tests.ts       # TS client → Rust server tests
│   └── server-tests.ts       # TS server ← Rust client tests
├── index.ts                  # Main test runner
└── typescript-server.ts     # Standalone TS server
```

### Key Components

**Transport Layer:**
- WebSocket transport with automatic reconnection
- HTTP batch transport for compatibility
- Message queuing and response handling

**Client Implementation:**
- Promise-based API
- Automatic timeout handling
- Concurrent call management
- Error propagation

**Server Implementation:**
- WebSocket server with capability registration
- Mock capabilities for testing
- Automatic capability lifecycle management

**Test Framework:**
- Comprehensive assertion library
- Performance measurement utilities
- Detailed test reporting
- Error analysis and reporting

## Configuration

### TypeScript Test Configuration

```typescript
interface TestConfiguration {
  runClientTests: boolean      // Run TS client → Rust server tests
  runServerTests: boolean      // Run TS server ← Rust client tests
  waitForRustServer: number    // Server startup wait time (ms)
  verbose: boolean             // Enable verbose logging
}
```

### Server Configuration

```typescript
interface ServerConfig {
  port?: number               // Server port (default: 8081)
  host?: string              // Server host (default: localhost)
  path?: string              // WebSocket path (default: /ws)
}
```

### Client Configuration

```typescript
interface ClientConfig {
  timeout?: number           // Call timeout (default: 30000ms)
  maxRetries?: number        // Retry attempts (default: 3)
  retryDelay?: number        // Retry delay (default: 1000ms)
}
```

## Command Line Options

```bash
node dist/index.js [options]

Options:
  --client-only     Run only TypeScript client → Rust server tests
  --server-only     Run only TypeScript server ← Rust client tests
  --wait <ms>       Wait time for Rust server startup (default: 3000ms)
  --verbose         Enable verbose logging
  --help           Show help message
```

## Development

### Building

```bash
# Development build with watch
pnpm build:watch

# Production build
pnpm build

# Type checking
pnpm typecheck

# Linting
pnpm lint
```

### Testing

```bash
# Run individual test components
node dist/tests/client-tests.js
node dist/tests/server-tests.js

# Run with specific configuration
node dist/index.js --client-only --verbose --wait 5000
```

### Debugging

Enable verbose logging to see detailed protocol messages:

```bash
node dist/index.js --verbose
```

This will show:
- WebSocket connection events
- Message serialization/deserialization
- Test execution details
- Performance measurements
- Error stack traces

## Integration with Rust Implementation

### Server Compatibility

The TypeScript client connects to the Rust server at:
- **URL**: `ws://localhost:8080/ws`
- **Capabilities**: Calculator (1,2), UserManager (100)
- **Protocol**: Identical JSON message format

### Client Compatibility

The TypeScript server accepts Rust clients at:
- **URL**: `ws://localhost:8081/ws`
- **Capabilities**: Calculator (1,2), UserManager (100)
- **Protocol**: Identical JSON message format

### Message Format Verification

Both implementations use identical JSON serialization:

```typescript
// Call message format (both directions)
{
  "call": {
    "id": 1,
    "target": { "cap": 42 },
    "member": "add",
    "args": [5, 3]
  }
}

// Result message format (both directions)
{
  "result": {
    "id": 1,
    "success": { "value": 8 }
  }
}
```

## Troubleshooting

### Common Issues

**Connection Refused:**
```bash
# Ensure Rust server is running
cd .. && cargo run --example calculator_server
```

**Port Conflicts:**
```bash
# Check if ports are in use
lsof -i :8080  # Rust server
lsof -i :8081  # TypeScript server
```

**Build Errors:**
```bash
# Clean and rebuild
rm -rf dist/
pnpm build
```

### Debugging Protocol Issues

1. **Enable Verbose Logging**:
   ```bash
   node dist/index.js --verbose
   ```

2. **Check Message Format**:
   - Compare JSON outputs between implementations
   - Verify field names and types match exactly

3. **Validate Capability IDs**:
   - Ensure both implementations use same capability IDs
   - Check capability registration order

4. **Test Individual Components**:
   ```bash
   # Test only TypeScript client
   node dist/index.js --client-only

   # Test only TypeScript server
   node dist/index.js --server-only
   ```

## Performance Benchmarks

The test suite includes performance measurements:

- **Individual Call Latency**: < 50ms typical
- **Concurrent Operations**: 100 calls in < 5 seconds
- **Sequential Operations**: 50 calls in < 10 seconds
- **Connection Establishment**: < 2 seconds
- **Error Handling Overhead**: < 10ms additional

## Contributing

To add new test scenarios:

1. **Add Test Cases** in `src/tests/client-tests.ts` or `src/tests/server-tests.ts`
2. **Update Capabilities** in `src/capnweb/server.ts` if needed
3. **Build and Test**:
   ```bash
   pnpm build
   pnpm test:all
   ```

4. **Update Documentation** in this README

## License

MIT OR Apache-2.0 (same as parent project)

---

**Ready for comprehensive Cap'n Web interoperability testing! 🚀**