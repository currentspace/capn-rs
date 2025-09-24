# Cap'n Web Advanced Features - Complete Testing Guide

## 🎯 Overview

This guide provides comprehensive instructions for testing all advanced Cap'n Web features with the official TypeScript client to ensure full protocol compliance and interoperability.

## 📋 Test Coverage

Our comprehensive test suite validates:

### 1. **Resume Tokens** ✅
- Session serialization and persistence
- Token generation with encryption
- Session restoration after disconnect
- Token expiration handling
- State preservation across sessions

### 2. **Nested Capabilities** ✅
- Dynamic capability creation via factories
- Capability hierarchy and graphs
- Sub-capability lifecycle management
- Reference counting and disposal
- Metadata and introspection

### 3. **Advanced IL Plan Runner** ✅
- Complex multi-step execution plans
- Plan validation and optimization
- Parallel operation execution
- Timeout and operation limits
- Complexity analysis

### 4. **HTTP/3 & WebTransport** ✅
- QUIC-based multiplexing
- Bidirectional streaming
- Connection pooling
- Load balancing
- Transport statistics

### 5. **Cross-Transport Interoperability** ✅
- Capability sharing across transports
- Automatic transport negotiation
- Fallback mechanisms
- Seamless transport switching

## 🚀 Quick Start

### Prerequisites

```bash
# Install Node.js 18+ and pnpm
curl -fsSL https://get.pnpm.io/install.sh | sh -

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and setup the repository
git clone https://github.com/currentspace/capn-rs.git
cd capn-rs
```

### Running Complete Test Suite

```bash
# Navigate to TypeScript interop directory
cd typescript-interop

# Install dependencies (including official Cap'n Web client)
pnpm install

# Run the complete advanced features test suite
pnpm test:complete
```

This command will:
1. Build the Rust server with all features enabled
2. Start the server with all transports (HTTP, WebSocket, HTTP/3, WebTransport)
3. Run comprehensive tests for each advanced feature
4. Generate detailed test reports

## 📊 Test Commands

### Individual Feature Tests

```bash
# Test Resume Tokens only
pnpm test:complete-advanced -- --feature resume-tokens

# Test Nested Capabilities only
pnpm test:complete-advanced -- --feature nested-capabilities

# Test IL Plan Runner only
pnpm test:complete-advanced -- --feature il-plans

# Test HTTP/3 & WebTransport only
pnpm test:complete-advanced -- --feature transports
```

### Performance & Stress Tests

```bash
# Run tier 3 extreme stress tests
pnpm test:tier3-extreme

# Run capability composition tests
pnpm test:tier3-capability

# Run cross-transport interop tests
pnpm test:cross-transport
```

### Complete Validation

```bash
# Run ALL tests including stress, performance, and validation
pnpm test:full-validation
```

## 🧪 Test Structure

```
typescript-interop/
├── src/
│   ├── complete-advanced-features-test.ts  # Main comprehensive test suite
│   ├── tier1-protocol-compliance.ts        # Basic protocol tests
│   ├── tier2-stateful-sessions.ts          # Stateful session tests
│   ├── tier2-websocket-tests.ts            # WebSocket specific tests
│   ├── tier3-capability-composition.ts     # Advanced capability tests
│   ├── tier3-extreme-stress.ts             # Stress and performance tests
│   └── cross-transport-interop.ts          # Transport interoperability
├── run-complete-tests.sh                   # Master test runner script
├── logs/                                    # Test execution logs
└── test-results/                            # Test reports and artifacts
```

## 📈 Test Reports

After running tests, reports are generated in multiple formats:

### Console Output
Real-time test execution with colored output showing:
- ✅ Passed tests
- ❌ Failed tests
- ⏩ Skipped tests
- 📊 Performance metrics

### JSON Report
`test-results/advanced-features-report.json`
```json
{
  "timestamp": "2024-09-23T10:00:00.000Z",
  "results": [
    {
      "feature": "Resume Tokens",
      "status": "passed",
      "message": "Session persistence working correctly",
      "details": { ... }
    },
    ...
  ],
  "summary": {
    "passed": 6,
    "failed": 0,
    "skipped": 0,
    "successRate": 100
  }
}
```

### HTML Report
`test-results/report.html` - Visual test report with:
- Summary dashboard
- Detailed test results
- Execution timeline
- Performance metrics

## 🔧 Configuration

### Server Configuration

The test server can be configured via environment variables:

```bash
# Set custom ports
export RUST_SERVER_PORT=8080
export RUST_WS_PORT=8080
export RUST_H3_PORT=8443

# Enable debug logging
export RUST_LOG=debug

# Run with custom config
pnpm test:complete
```

### Client Configuration

Modify `complete-advanced-features-test.ts` to customize:
- Test timeouts
- Retry attempts
- Transport preferences
- Feature flags

## 🐛 Troubleshooting

### Common Issues

#### Server fails to start
```bash
# Check if ports are already in use
lsof -i :8080

# Kill existing processes
pkill -f "capnweb-server"

# Retry with different port
RUST_SERVER_PORT=9090 pnpm test:complete
```

#### WebSocket connection fails
```bash
# Ensure WebSocket support is enabled
cargo build --features websocket

# Check firewall settings
sudo ufw allow 8080/tcp
```

#### HTTP/3 not available
```bash
# Install required dependencies
cargo build --features http3

# Generate certificates for HTTPS
cd certs && ./generate-certs.sh
```

### Debug Mode

Enable detailed logging:

```bash
# Run tests with debug output
RUST_LOG=debug DEBUG=* pnpm test:complete

# Save debug logs
RUST_LOG=trace pnpm test:complete 2>&1 | tee debug.log
```

## 📝 Test Scenarios

### Scenario 1: Session Recovery
```typescript
// Create session with state
const session = await client.createSession();
await session.setValue('user', 'alice');

// Get resume token
const token = await session.getResumeToken();

// Disconnect and reconnect
await client.disconnect();
const newClient = new Client({ resumeToken: token });

// Verify state preserved
const user = await newClient.getValue('user');
assert(user === 'alice');
```

### Scenario 2: Nested Capabilities
```typescript
// Create capability hierarchy
const factory = await client.getCapabilityFactory();
const root = await factory.create('processor');
const child = await root.createChild('analyzer');

// Use nested capabilities
const result = await child.analyze(data);

// Clean up
await root.disposeChild(child.id);
```

### Scenario 3: IL Plan Execution
```typescript
// Build complex plan
const plan = {
  ops: [
    { call: { target: cap1, method: 'process', args: [data] }},
    { call: { target: cap2, method: 'analyze', args: [{ result: 0 }] }},
    { object: { fields: { data: { result: 0 }, analysis: { result: 1 }}}}
  ],
  result: { result: 2 }
};

// Execute plan
const result = await client.executePlan(plan);
```

## 🏆 Validation Criteria

Tests are considered passing when:

1. **Functional Requirements** ✅
   - All advanced features work as specified
   - Protocol compliance validated
   - Cross-transport compatibility confirmed

2. **Performance Requirements** ✅
   - Response time < 100ms for basic operations
   - Plan execution < 500ms for complex scenarios
   - Memory usage stable under load

3. **Reliability Requirements** ✅
   - Session recovery works 100% of the time
   - No memory leaks detected
   - Graceful handling of errors

4. **Interoperability** ✅
   - Works with official TypeScript client
   - Compatible with all transport types
   - Follows Cap'n Web protocol specification

## 📚 Additional Resources

- [Cap'n Web Protocol Specification](https://github.com/cloudflare/capnweb/blob/main/protocol.md)
- [Official TypeScript Client Docs](https://github.com/cloudflare/capnweb)
- [Rust Implementation Guide](./CLAUDE.md)
- [API Documentation](./docs/api.md)

## 🤝 Contributing

To add new tests:

1. Create test file in `typescript-interop/src/`
2. Follow the pattern in `complete-advanced-features-test.ts`
3. Update `package.json` with new test command
4. Document test scenarios in this guide
5. Submit PR with test results

## 📞 Support

For issues or questions:
- File issues at: https://github.com/currentspace/capn-rs/issues
- Check existing tests: `typescript-interop/src/`
- Review logs: `typescript-interop/logs/`

---

**Last Updated**: September 2024
**Version**: 1.0.0
**Status**: ✅ All Advanced Features Fully Tested