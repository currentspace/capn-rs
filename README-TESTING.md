# Cap'n Web Rust Implementation - Testing Guide

## Quick Start

The Cap'n Web Rust implementation now includes a unified test server and comprehensive testing scripts.

### 1. Build the Server
```bash
./build-server.sh
```

### 2. Run Tests
```bash
# Run all tests (comprehensive suite)
./run-tests.sh

# Run specific test tiers
./run-tests.sh 9000 127.0.0.1 tier1  # Basic protocol compliance
./run-tests.sh 9000 127.0.0.1 tier2  # HTTP batch transport
./run-tests.sh 9000 127.0.0.1 tier3  # Advanced features

# Quick smoke test (just tier1)
./run-tests.sh 9000 127.0.0.1 quick
```

### 3. Run Server Standalone
```bash
./run-server.sh [PORT] [HOST] [LOG_LEVEL]

# Example:
./run-server.sh 9000 127.0.0.1 debug
```

## Test Coverage

### ✅ Tier 1: Protocol Compliance (100% PASSING)
- Basic connectivity
- Message format validation
- Response structure verification
- Error handling

### ✅ Tier 2: HTTP Batch Transport (100% PASSING)
- Batch operations (all in single request)
- Session isolation between clients
- Concurrent operations in single batch
- Error handling within batch
- Multiple sequential batches (new sessions)

### ⚠️ Tier 2: WebSocket Transport (Not Yet Implemented)
- Persistent sessions
- Real-time bidirectional communication

### 🔄 Tier 3: Advanced Features (In Development)
- Capability composition
- Complex application scenarios

## Protocol Features Tested

### Message Types
- ✅ Push - Evaluate expressions and assign import IDs
- ✅ Pull - Request resolution of promises
- ✅ Resolve - Provide successful results
- ✅ Reject - Provide error results
- ✅ Release - Dispose of capabilities

### ID Management
- ✅ Sequential import IDs (1, 2, 3...)
- ✅ Export IDs match pull request IDs
- ✅ Capability ID 0 reserved for main interface
- ✅ Session state management

### Transport Behaviors
- ✅ HTTP Batch - Stateless, ends after sending batch
- ✅ Promise.all() for batching operations
- ✅ New sessions for sequential batches
- ⚠️ WebSocket - Persistent sessions (coming soon)

### Error Handling
- ✅ Method not found errors
- ✅ Invalid argument errors
- ✅ Division by zero handling
- ✅ Custom error types
- ✅ Mixed success/error in batch

## Unified Test Server

The unified test server (`unified_test_server`) includes:

### Capabilities
1. **Calculator** (ID: 1) - Basic arithmetic operations
   - add, subtract, multiply, divide
   - echo, concat

2. **StatefulCalculator** (ID: 2) - Calculator with memory
   - All basic operations
   - store, recall, clear
   - history, clearHistory

3. **GlobalCounter** (ID: 3) - Shared counter
   - increment, decrement
   - get, set, reset

4. **KeyValueStore** (ID: 4) - Persistent storage
   - set, get, delete
   - keys, values, clear, size

5. **ErrorTest** (ID: 5) - Error handling tests
   - throwError, throwBadRequest
   - throwNotFound, throwCustom
   - success

## Server Configuration

Environment variables:
- `PORT` - Server port (default: 9000)
- `HOST` - Server host (default: 127.0.0.1)
- `RUST_LOG` - Log level (default: info,capnweb_server=debug,capnweb_core=debug)

## Architecture

```
capn-rs/
├── build-server.sh          # Build the unified test server
├── run-server.sh            # Run the server
├── run-tests.sh             # Run TypeScript interop tests
├── capnweb-server/
│   └── examples/
│       └── unified_test_server.rs  # All capabilities in one server
└── typescript-interop/
    ├── src/
    │   ├── tier1-protocol-compliance.ts
    │   ├── tier2-http-batch-corrected.ts
    │   ├── tier2-websocket-tests.ts
    │   └── tier3-*.ts
    └── dist/                # Compiled tests
```

## Development Workflow

1. Make changes to the Rust server
2. Run `./build-server.sh` to rebuild
3. Run `./run-tests.sh 9000 127.0.0.1 quick` for quick validation
4. Run `./run-tests.sh` for comprehensive testing

## Troubleshooting

### Server won't start
- Check if port is already in use: `lsof -i:9000`
- Check server.log for error messages
- Kill existing servers: `pkill -f unified_test_server`

### Tests fail
- Verify server is running: `curl http://localhost:9000/health`
- Check server.log for error messages
- Rebuild TypeScript tests: `cd typescript-interop && npm run build`

### Build issues
- Clean and rebuild: `cargo clean && ./build-server.sh`
- Update dependencies: `cargo update`

## Status Summary

✅ **PROTOCOL COMPLIANT** - The Rust implementation correctly implements the Cap'n Web wire protocol and is compatible with the official TypeScript reference client.

### What's Working
- Official wire protocol (newline-delimited JSON arrays)
- HTTP batch transport with correct semantics
- All basic protocol operations
- Error handling
- Session isolation
- Capability management

### What's Next
- WebSocket transport implementation
- Promise pipelining
- Advanced capability features
- Performance optimizations